use colored::{Color, Colorize};
use petgraph::{stable_graph::StableUnGraph, visit::Bfs};
use std::{collections::HashMap, fmt::Display, io::stdin};

use glam::prelude::*;

fn main() {
    println!("Barricade!");
    let mut map = Map::new();
    // let _ = map.try_place_barricade(Team::Red, BCoord::new(0, 0), Orientation::Vertical);
    // let _ = map.try_place_barricade(Team::Red, BCoord::new(0, 1), Orientation::Vertical);
    // let _ = map.try_place_barricade(Team::Red, BCoord::new(0, 2), Orientation::Vertical);
    // let _ = map.try_place_barricade(Team::Blue, BCoord::new(0, 1), Orientation::Horizontal);
    // let _ = map.try_place_barricade(Team::Blue, BCoord::new(3, 7), Orientation::Vertical);
    // // let _ = map.try_place_barricade(Team::Blue, BCoord::new(4, 7), Orientation::Vertical);
    // let _ = map.try_place_barricade(Team::Blue, BCoord::new(3, 6), Orientation::Horizontal);

    // map.move_player(Team::Red, PCoord::new(4, 7));
    // let _ = map.try_step(Team::Blue, PCoord::new(5, 7));

    let mut turn = 0;

    let mut inventory = HashMap::from([(Team::Red, 10), (Team::Blue, 10)]);

    loop {
        let current_team = if turn % 2 == 0 { Team::Red } else { Team::Blue };

        println!("{:?}", inventory);
        println!("{map}");

        if map.blue_player.coord.y == 0 {
            // blue wins
            println!("Blue wins!");
            return;
        }
        if map.red_player.coord.y == 8 {
            // red wins
            println!("Red wins!");
            return;
        }

        println!("{}'s turn", if turn % 2 == 0 { "Red" } else { "Blue" });

        let mut input = String::new();
        if stdin().read_line(&mut input).is_err() {
            continue;
        }

        let input = input.trim();
        let mut chars = input.chars();

        let Some(cmd) = chars.next() else {
            continue;
        };
        let Some(c1) = chars.next() else {
            continue;
        };
        let Some(c2) = chars.next() else {
            continue;
        };

        let x = (c1.to_ascii_lowercase() as i32) - i32::from(b'a');
        let y = (c2 as i32) - i32::from(b'1');
        let coord = PCoord::new(x, y);

        let ok = match cmd.to_ascii_lowercase() {
            'm' => map.try_step(current_team, coord).is_ok(),
            'v' | 'h' => {
                let orientation = if cmd.to_ascii_lowercase() == 'v' {
                    Orientation::Vertical
                } else {
                    Orientation::Horizontal
                };

                if inventory.get(&current_team).is_some_and(|&n| n > 0) {
                    if map
                        .try_place_barricade(current_team, coord, orientation)
                        .is_ok()
                    {
                        if let Some(count) = inventory.get_mut(&current_team) {
                            *count -= 1;
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        };

        if ok {
            turn += 1;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Team {
    Red,
    Blue,
}

impl From<Team> for Color {
    fn from(team: Team) -> Self {
        match team {
            Team::Red => Color::Red,
            Team::Blue => Color::Blue,
        }
    }
}

pub struct Player {
    team: Team,
    coord: PCoord,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

pub type PCoord = IVec2; // Player grid coord
pub type BCoord = IVec2; // Barricade grid coord

pub type PGrid = [Option<Team>; 9 * 9];
pub type BGrid = [Option<Barricade>; 8 * 8];

pub type NavGraph = StableUnGraph<(), (), usize>;

pub struct Map {
    player_grid: PGrid,
    barricade_grid: BGrid,
    red_player: Player,
    blue_player: Player,
}

#[derive(Clone)]
pub struct Barricade {
    team: Team,
    orientation: Orientation,
}

impl Map {
    pub fn new() -> Self {
        let mut _self = Self {
            player_grid: [const { None }; 9 * 9],
            barricade_grid: [const { None }; 8 * 8],
            red_player: Player {
                team: Team::Red,
                coord: PCoord::new(4, 0),
            },
            blue_player: Player {
                team: Team::Blue,
                coord: PCoord::new(4, 8),
            },
        };

        _self.player_grid[pcoord_to_index(_self.red_player.coord)] = Some(Team::Red);
        _self.player_grid[pcoord_to_index(_self.blue_player.coord)] = Some(Team::Blue);

        _self
    }

    pub fn try_place_barricade(
        &mut self,
        team: Team,
        coord: BCoord,
        orientation: Orientation,
    ) -> Result<(), ()> {
        if self.can_place_barricade(coord, orientation) {
            self.barricade_grid[bcoord_to_index(coord)] = Some(Barricade { team, orientation });
            return Ok(());
        }
        Err(())
    }

    pub fn can_place_barricade(&self, coord: BCoord, orientation: Orientation) -> bool {
        // TODO: Path checks

        if !bcoord_in_bounds(coord) {
            return false;
        }

        let index = bcoord_to_index(coord);

        let in_place_occupied = self
            .barricade_grid
            .get(index)
            .is_some_and(|inner| inner.is_some());

        let neighbor_occupied = match orientation {
            Orientation::Horizontal => {
                let left_coord = BCoord::new(coord.x - 1, coord.y);
                let right_coord = BCoord::new(coord.x + 1, coord.y);

                let left_index = bcoord_to_index(left_coord);
                let right_index = bcoord_to_index(right_coord);

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

                let above_index = bcoord_to_index(above_coord);
                let below_index = bcoord_to_index(below_coord);

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

        let navgraph = get_navgraph(&future_grid);

        let red_can_finish = can_reach_y_level(&navgraph, self.red_player.coord, 8);
        let blue_can_finish = can_reach_y_level(&navgraph, self.blue_player.coord, 0);

        return !in_place_occupied && !neighbor_occupied && red_can_finish && blue_can_finish;
    }

    pub fn move_player(&mut self, team: Team, coord: PCoord) {
        let index = {
            let player = self.get_player(team);
            pcoord_to_index(player.coord)
        };
        self.player_grid[index] = None;
        self.player_grid[pcoord_to_index(coord)] = Some(team);

        let player = self.get_player_mut(team);
        player.coord = coord;
    }

    pub fn can_step(&self, team: Team, to: PCoord) -> bool {
        let player = self.get_player(team);

        if !pcoord_in_bounds(to) || self.player_grid[pcoord_to_index(to)].is_some() {
            return false;
        }

        let diff = to - player.coord;
        if diff.y == 0 {
            // horizontal move
            if diff.x.abs() == 1 {
                // standard move
                if !is_barricade_between(&self.barricade_grid, player.coord, to) {
                    return true;
                }
            } else if diff.x.abs() == 2 {
                // jump move
                let between = player.coord + diff / 2;
                let index_between = pcoord_to_index(between);
                if !is_barricade_between(&self.barricade_grid, player.coord, between)
                    && !is_barricade_between(&self.barricade_grid, between, to)
                    && self.player_grid[index_between].is_some_and(|team| team != player.team)
                {
                    return true;
                }
            }
        } else if diff.x == 0 {
            // vertical move
            if diff.y.abs() == 1 {
                // standard move
                if !is_barricade_between(&self.barricade_grid, player.coord, to) {
                    return true;
                }
            } else if diff.y.abs() == 2 {
                // jump move
                let between = player.coord + diff / 2;
                let index_between = pcoord_to_index(between);
                if !is_barricade_between(&self.barricade_grid, player.coord, between)
                    && !is_barricade_between(&self.barricade_grid, between, to)
                    && self.player_grid[index_between].is_some_and(|team| team != player.team)
                {
                    return true;
                }
            }
        } else if diff.y.abs() == 1 && diff.y.abs() == 1 {
            // diagonal move
            let vert_first_coord = player.coord + PCoord::new(0, diff.y);
            let hori_first_coord = player.coord + PCoord::new(diff.x, 0);

            let vert_first_index = pcoord_to_index(vert_first_coord);
            let hori_first_index = pcoord_to_index(hori_first_coord);

            if self.player_grid[vert_first_index].is_some_and(|team| team != player.team)
                && !is_barricade_between(&self.barricade_grid, player.coord, vert_first_coord)
                && !is_barricade_between(&self.barricade_grid, vert_first_coord, to)
            {
                // check if barricade is behind or there is the map edge
                let behind = player.coord + PCoord::new(0, diff.y * 2);
                return !pcoord_in_bounds(behind)
                    || is_barricade_between(&self.barricade_grid, vert_first_coord, behind);
            } else if self.player_grid[hori_first_index].is_some_and(|team| team != player.team)
                && !is_barricade_between(&self.barricade_grid, player.coord, hori_first_coord)
                && !is_barricade_between(&self.barricade_grid, hori_first_coord, to)
            {
                // check if barricade is behind or there is the map edge
                let behind = player.coord + PCoord::new(diff.x * 2, 0);
                return !pcoord_in_bounds(behind)
                    || is_barricade_between(&self.barricade_grid, hori_first_coord, behind);
            }
        }

        false
    }

    pub fn try_step(&mut self, team: Team, to: PCoord) -> Result<(), ()> {
        if self.can_step(team, to) {
            self.move_player(team, to);
            return Ok(());
        }
        Err(())
    }

    fn get_player(&self, team: Team) -> &Player {
        match team {
            Team::Red => &self.red_player,
            Team::Blue => &self.blue_player,
        }
    }

    fn get_player_mut(&mut self, team: Team) -> &mut Player {
        match team {
            Team::Red => &mut self.red_player,
            Team::Blue => &mut self.blue_player,
        }
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  ")?;
        for x in 0..17 {
            if x % 2 == 0 {
                let c = (b'a' + x / 2 as u8) as char;
                write!(f, " {} ", c.to_string().bright_black())?;
            } else {
                write!(f, "{}", "╷".bright_black())?;
            }
        }
        writeln!(f)?;
        write!(f, " {}", "1".bright_black())?;
        for y in 0..17 {
            for x in 0..17 {
                match (x % 2, y % 2) {
                    // draw cell
                    (0, 0) => {
                        let pcoord = PCoord::new(x / 2, y / 2);
                        let index = pcoord_to_index(pcoord);
                        if let Some(team) = &self.player_grid[index] {
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
                        let index_above = bcoord_to_index(bcoord_above);
                        let index_below = bcoord_to_index(bcoord_below);

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
                        let index_left = bcoord_to_index(bcoord_left);
                        let index_right = bcoord_to_index(bcoord_right);

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
                        let index = bcoord_to_index(bcoord);

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
                        write!(f, " {}", ((y + 1) / 2 + 1).to_string().bright_black())?;
                    } else {
                        write!(f, "{}", "╴ ".bright_black())?;
                    }
                    writeln!(f)?;
                    if y % 2 == 0 {
                        if y != 16 {
                            write!(f, "{}", " ╶".bright_black())?;
                        } else {
                            write!(f, "  ")?;
                        }
                    } else {
                        write!(f, " {}", ((y + 1) / 2 + 1).to_string().bright_black())?;
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

fn can_reach_y_level(graph: &NavGraph, from: PCoord, y: i32) -> bool {
    let mut bfs = Bfs::new(graph, pcoord_to_index(from).into());

    while let Some(node) = bfs.next(graph) {
        let coord = index_to_pcoord(node.index());

        if coord.y == y {
            return true;
        }
    }
    false
}

fn get_navgraph(grid: &BGrid) -> NavGraph {
    let mut edges = Vec::new();

    for y in 0..9 {
        for x in 0..9 {
            let coord = PCoord::new(x, y);
            let right = PCoord::new(x + 1, y);
            let below = PCoord::new(x, y + 1);

            let coord_index = pcoord_to_index(coord);
            let right_index = pcoord_to_index(right);
            let below_index = pcoord_to_index(below);

            if pcoord_in_bounds(right) && !is_barricade_between(grid, coord, right) {
                edges.push((coord_index, right_index));
            }
            if pcoord_in_bounds(below) && !is_barricade_between(grid, coord, below) {
                edges.push((coord_index, below_index));
            }
        }
    }

    NavGraph::from_edges(edges)
}

fn is_barricade_between(grid: &BGrid, a: PCoord, b: PCoord) -> bool {
    let diff = b - a;
    if diff.x.abs() == 1 && diff.y == 0 {
        // horizontal move
        let bcoord_above = BCoord::new(a.x.min(b.x), a.y - 1);
        let bcoord_below = BCoord::new(a.x.min(b.x), a.y);

        let index_above = bcoord_to_index(bcoord_above);
        let index_below = bcoord_to_index(bcoord_below);

        return grid.get(index_above).is_some_and(|inner| {
            inner
                .as_ref()
                .is_some_and(|barricade| barricade.orientation == Orientation::Vertical)
        }) | grid.get(index_below).is_some_and(|inner| {
            inner
                .as_ref()
                .is_some_and(|barricade| barricade.orientation == Orientation::Vertical)
        });
    } else if diff.x == 0 && diff.y.abs() == 1 {
        // vertical move
        let bcoord_left = BCoord::new(a.x - 1, a.y.min(b.y));
        let bcoord_right = BCoord::new(a.x, a.y.min(b.y));

        let index_left = bcoord_to_index(bcoord_left);
        let index_right = bcoord_to_index(bcoord_right);

        return grid.get(index_left).is_some_and(|inner| {
            inner
                .as_ref()
                .is_some_and(|barricade| barricade.orientation == Orientation::Horizontal)
        }) | grid.get(index_right).is_some_and(|inner| {
            inner
                .as_ref()
                .is_some_and(|barricade| barricade.orientation == Orientation::Horizontal)
        });
    } else {
        // illegal move
        return false;
    }
}

fn bcoord_to_index(coord: BCoord) -> usize {
    if !bcoord_in_bounds(coord) {
        return usize::MAX;
    }

    return (coord.x + coord.y * 8) as _;
}

fn pcoord_to_index(coord: PCoord) -> usize {
    if !pcoord_in_bounds(coord) {
        return usize::MAX;
    }

    return (coord.x + coord.y * 9) as _;
}

fn index_to_pcoord(index: usize) -> PCoord {
    return PCoord::new((index % 9) as _, (index / 9) as _);
}

fn bcoord_in_bounds(coord: BCoord) -> bool {
    coord.x >= 0 && coord.x < 8 && coord.y >= 0 && coord.y < 8
}

fn pcoord_in_bounds(coord: PCoord) -> bool {
    coord.x >= 0 && coord.x < 9 && coord.y >= 0 && coord.y < 9
}
