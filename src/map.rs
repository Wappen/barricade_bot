use colored::Colorize;
use std::fmt::Display;

use crate::{
    coord::{BCoord, PCoord},
    grid::BarricadeGrid,
    types::{Barricade, Orientation, Team},
};

#[derive(Clone, Debug)]
pub struct Map {
    pub barricade_grid: BarricadeGrid,
    pub red_coord: PCoord,
    pub blue_coord: PCoord,
}

pub const RED_FINISH_LINE: [PCoord; 9] = [
    PCoord::new(0, 8),
    PCoord::new(1, 8),
    PCoord::new(2, 8),
    PCoord::new(3, 8),
    PCoord::new(4, 8),
    PCoord::new(5, 8),
    PCoord::new(6, 8),
    PCoord::new(7, 8),
    PCoord::new(8, 8),
];

pub const BLUE_FINISH_LINE: [PCoord; 9] = [
    PCoord::new(0, 0),
    PCoord::new(1, 0),
    PCoord::new(2, 0),
    PCoord::new(3, 0),
    PCoord::new(4, 0),
    PCoord::new(5, 0),
    PCoord::new(6, 0),
    PCoord::new(7, 0),
    PCoord::new(8, 0),
];

impl Map {
    pub fn new(red_coord: PCoord, blue_coord: PCoord) -> Self {
        Self {
            barricade_grid: BarricadeGrid::new(),
            red_coord,
            blue_coord,
        }
    }

    pub fn try_place_barricade(
        &mut self,
        team: Team,
        coord: BCoord,
        orientation: Orientation,
    ) -> Result<(), ()> {
        if self.can_place_barricade(coord, orientation) {
            self.barricade_grid[coord.to_index()] = Some(Barricade { team, orientation });
            return Ok(());
        }
        Err(())
    }

    pub fn try_step(&mut self, team: Team, to: PCoord) -> Result<(), ()> {
        if self.can_step(team, to) {
            self.move_player(team, to);
            return Ok(());
        }
        Err(())
    }

    pub fn can_place_barricade(&self, coord: BCoord, orientation: Orientation) -> bool {
        if !coord.is_in_bounds() {
            return false;
        }

        let index = coord.to_index();

        let in_place_occupied = self
            .barricade_grid
            .get(index)
            .is_some_and(|inner| inner.is_some());

        let neighbor_occupied = match orientation {
            Orientation::Horizontal => {
                let left_coord = BCoord::new(coord.x - 1, coord.y);
                let right_coord = BCoord::new(coord.x + 1, coord.y);

                let left_index = left_coord.to_index();
                let right_index = right_coord.to_index();

                self.barricade_grid.get(left_index).is_some_and(|inner| {
                    inner
                        .as_ref()
                        .is_some_and(|barricade| barricade.orientation == Orientation::Horizontal)
                }) | self.barricade_grid.get(right_index).is_some_and(|inner| {
                    inner
                        .as_ref()
                        .is_some_and(|barricade| barricade.orientation == Orientation::Horizontal)
                })
            }
            Orientation::Vertical => {
                let above_coord = BCoord::new(coord.x, coord.y - 1);
                let below_coord = BCoord::new(coord.x, coord.y + 1);

                let above_index = above_coord.to_index();
                let below_index = below_coord.to_index();

                self.barricade_grid.get(above_index).is_some_and(|inner| {
                    inner
                        .as_ref()
                        .is_some_and(|barricade| barricade.orientation == Orientation::Vertical)
                }) | self.barricade_grid.get(below_index).is_some_and(|inner| {
                    inner
                        .as_ref()
                        .is_some_and(|barricade| barricade.orientation == Orientation::Vertical)
                })
            }
        };

        let mut future_grid = self.barricade_grid.clone();
        future_grid[index] = Some(Barricade {
            team: Team::Red,
            orientation,
        });

        let navgraph = future_grid.get_navgraph();

        let red_can_finish = navgraph.can_reach_y_level(self.red_coord, 8);
        let blue_can_finish = navgraph.can_reach_y_level(self.blue_coord, 0);

        return !in_place_occupied && !neighbor_occupied && red_can_finish && blue_can_finish;
    }

    pub fn can_step(&self, player_team: Team, to: PCoord) -> bool {
        if self.is_occupied(to) {
            return false;
        }

        let coord = self.get_player_coord(player_team);

        let diff = to - coord;
        if diff.y == 0 {
            // horizontal move
            if diff.x.abs() == 1 {
                // standard move
                if !self.barricade_grid.is_barricade_between(coord, to) {
                    return true;
                }
            } else if diff.x.abs() == 2 {
                // jump move
                let between = coord + diff / 2;
                if !self.barricade_grid.is_barricade_between(coord, between)
                    && !self.barricade_grid.is_barricade_between(between, to)
                    && self.is_occupied(between)
                {
                    return true;
                }
            }
        } else if diff.x == 0 {
            // vertical move
            if diff.y.abs() == 1 {
                // standard move
                if !self.barricade_grid.is_barricade_between(coord, to) {
                    return true;
                }
            } else if diff.y.abs() == 2 {
                // jump move
                let between = coord + diff / 2;
                if !self.barricade_grid.is_barricade_between(coord, between)
                    && !self.barricade_grid.is_barricade_between(between, to)
                    && self.is_occupied(between)
                {
                    return true;
                }
            }
        } else if diff.y.abs() == 1 && diff.x.abs() == 1 {
            // diagonal move
            let vert_first_coord = coord + PCoord::new(0, diff.y);
            let hori_first_coord = coord + PCoord::new(diff.x, 0);

            if self.is_occupied(vert_first_coord)
                && !self
                    .barricade_grid
                    .is_barricade_between(coord, vert_first_coord)
                && !self
                    .barricade_grid
                    .is_barricade_between(vert_first_coord, to)
            {
                // check if barricade is behind or there is the map edge
                let behind = coord + PCoord::new(0, diff.y * 2);
                return !behind.is_in_bounds()
                    || self
                        .barricade_grid
                        .is_barricade_between(vert_first_coord, behind);
            } else if self.is_occupied(hori_first_coord)
                && !self
                    .barricade_grid
                    .is_barricade_between(coord, hori_first_coord)
                && !self
                    .barricade_grid
                    .is_barricade_between(hori_first_coord, to)
            {
                // check if barricade is behind or there is the map edge
                let behind = coord + PCoord::new(diff.x * 2, 0);
                return !behind.is_in_bounds()
                    || self
                        .barricade_grid
                        .is_barricade_between(hori_first_coord, behind);
            }
        }

        false
    }

    pub fn find_shortest_path_to_win(&self, team: Team) -> Vec<PCoord> {
        let graph = self.barricade_grid.get_navgraph();
        let start = self.get_player_coord(team);

        let finish = match team {
            Team::Red => RED_FINISH_LINE,
            Team::Blue => BLUE_FINISH_LINE,
        };

        graph
            .find_shortest_path(start, &finish)
            .expect("could not find path to finish")
    }

    pub fn find_valid_barricades(&self) -> Vec<(BCoord, Orientation)> {
        let mut result = vec![];

        for y in 0..8 {
            for x in 0..8 {
                let coord = BCoord::new(x, y);
                if self.can_place_barricade(coord, Orientation::Horizontal) {
                    result.push((coord, Orientation::Horizontal));
                }
                if self.can_place_barricade(coord, Orientation::Vertical) {
                    result.push((coord, Orientation::Vertical));
                }
            }
        }

        result
    }

    pub fn find_valid_moves(&self, team: Team) -> Vec<PCoord> {
        let mut result = vec![];

        for y in 0..9 {
            for x in 0..9 {
                let coord = PCoord::new(x, y);
                if self.can_step(team, coord) {
                    result.push(coord);
                }
            }
        }

        result
    }

    pub fn move_player(&mut self, team: Team, coord: PCoord) {
        match team {
            Team::Red => self.red_coord = coord,
            Team::Blue => self.blue_coord = coord,
        }
    }

    pub fn is_occupied(&self, coord: PCoord) -> bool {
        return !coord.is_in_bounds() || self.red_coord == coord || self.blue_coord == coord;
    }

    pub fn get_team_at(&self, coord: PCoord) -> Option<Team> {
        if self.red_coord == coord {
            return Some(Team::Red);
        } else if self.blue_coord == coord {
            return Some(Team::Blue);
        }
        None
    }

    pub fn get_player_coord(&self, team: Team) -> PCoord {
        match team {
            Team::Red => self.red_coord,
            Team::Blue => self.blue_coord,
        }
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " ")?;
        for x in 0..17 {
            if x % 2 == 0 {
                let c = (b'a' + x / 2 as u8) as char;
                write!(f, " {} ", c.to_string().bright_black())?;
            } else {
                write!(f, "{}", "╷".bright_black())?;
            }
        }
        writeln!(f)?;
        write!(f, "{}", "1".bright_black())?;
        for y in 0..17 {
            for x in 0..17 {
                match (x % 2, y % 2) {
                    // draw cell
                    (0, 0) => {
                        let pcoord = PCoord::new(x / 2, y / 2);
                        if let Some(team) = self.get_team_at(pcoord) {
                            match team {
                                Team::Red => write!(f, "{}", " ● ".red())?,
                                Team::Blue => write!(f, "{}", " ● ".blue())?,
                            }
                        } else {
                            write!(f, "   ")?;
                            // write!(
                            //     f,
                            //     "{} {}",
                            //     ((b'A' + pcoord.x as u8) as char).to_string().black(),
                            //     (pcoord.y + 1).to_string().black()
                            // )?;
                        }
                    }

                    // draw vertical border
                    (1, 0) => {
                        // two possible barrier positions to be considered
                        // above and below

                        let bcoord_above = BCoord::new((x - 1) / 2, (y - 2) / 2);
                        let bcoord_below = BCoord::new((x - 1) / 2, y / 2);
                        let index_above = bcoord_above.to_index();
                        let index_below = bcoord_below.to_index();

                        let mut has_barricade = None;

                        if let Some(Some(barricade)) = &self.barricade_grid.get(index_above) {
                            match barricade.orientation {
                                Orientation::Vertical => has_barricade = Some(barricade),
                                Orientation::Horizontal => {}
                            }
                        }
                        if let Some(Some(barricade)) = &self.barricade_grid.get(index_below) {
                            match barricade.orientation {
                                Orientation::Vertical => has_barricade = Some(barricade),
                                Orientation::Horizontal => {}
                            }
                        }

                        if let Some(barricade) = has_barricade {
                            write!(f, "{}", "┃".color(barricade.team))?;
                        } else {
                            write!(f, "{}", "│".bright_black())?;
                        }
                    }

                    // draw horizontal border
                    (0, 1) => {
                        // two possible barrier positions to be considered
                        // left and right

                        let bcoord_left = BCoord::new((x - 2) / 2, (y - 1) / 2);
                        let bcoord_right = BCoord::new(x / 2, (y - 1) / 2);
                        let index_left = bcoord_left.to_index();
                        let index_right = bcoord_right.to_index();

                        let mut has_barricade = None;

                        if let Some(Some(barricade)) = &self.barricade_grid.get(index_left) {
                            match barricade.orientation {
                                Orientation::Horizontal => has_barricade = Some(barricade),
                                Orientation::Vertical => {}
                            }
                        }
                        if let Some(Some(barricade)) = &self.barricade_grid.get(index_right) {
                            match barricade.orientation {
                                Orientation::Horizontal => has_barricade = Some(barricade),
                                Orientation::Vertical => {}
                            }
                        }

                        if let Some(barricade) = has_barricade {
                            write!(f, "{}", "━━━".color(barricade.team))?;
                        } else {
                            write!(f, "{}", "───".bright_black())?;
                        }
                    }

                    // draw cross border
                    (1, 1) => {
                        let bcoord = BCoord::new((x - 1) / 2, (y - 1) / 2);
                        let index = bcoord.to_index();

                        if let Some(barricade) = &self.barricade_grid[index] {
                            match barricade.orientation {
                                Orientation::Horizontal => {
                                    write!(f, "{}", "┿".color(barricade.team))?
                                }
                                Orientation::Vertical => {
                                    write!(f, "{}", "╂".color(barricade.team))?
                                }
                            }
                        } else {
                            write!(f, "{}", "┼".bright_black())?
                        }
                    }
                    _ => {}
                }

                // newline
                if x == 16 {
                    if y % 2 == 0 {
                        write!(f, "{}", ((y + 1) / 2 + 1).to_string().bright_black())?;
                    } else {
                        write!(f, "{}", "╴".bright_black())?;
                    }
                    writeln!(f)?;
                    if y % 2 == 0 {
                        if y != 16 {
                            write!(f, "{}", "╶".bright_black())?;
                        } else {
                            write!(f, " ")?;
                        }
                    } else {
                        write!(f, "{}", ((y + 1) / 2 + 1).to_string().bright_black())?;
                    }
                }
            }
        }

        for x in 0..17 {
            if x % 2 == 0 {
                let c = (b'a' + x / 2 as u8) as char;
                write!(f, " {} ", c.to_string().bright_black())?;
            } else {
                write!(f, "{}", "╵".bright_black())?;
            }
        }

        Ok(())
    }
}
