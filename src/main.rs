pub mod bot;
pub mod path;

use colored::{Color, Colorize};
use petgraph::{stable_graph::StableUnGraph, visit::Bfs};
use std::{fmt::Display, io::stdin, thread::sleep, time::Duration};

use glam::prelude::*;

use crate::bot::{Bot, Context, RandomBot};

fn main() {
    println!("Barricade!");
    let mut game = Game::new();
    // game.run();
    game.pve(RandomBot);
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
    inventory: usize, // number of barricades left in inventory
}

impl Player {
    fn new(team: Team) -> Self {
        Self {
            team,
            inventory: 10,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

pub type PCoord = IVec2; // Player grid coord
pub type BCoord = IVec2; // Barricade grid coord

pub type BGrid = [Option<Barricade>; 8 * 8];

pub type NavGraph = StableUnGraph<(), (), usize>;

pub struct Game {
    map: Map,
    red_player: Player,
    blue_player: Player,
}

impl Game {
    fn new() -> Self {
        let red_player = Player::new(Team::Red);
        let blue_player = Player::new(Team::Blue);

        Self {
            map: Map::new(PCoord::new(4, 0), PCoord::new(4, 8)),
            red_player,
            blue_player,
        }
    }

    fn run(&mut self) {
        let mut turn = 0;

        loop {
            println!("Red: {}", self.red_player.inventory);
            println!("Blue: {}", self.blue_player.inventory);
            println!("{}", self.map);

            if self.map.blue_coord.y == 0 {
                // blue wins
                println!("Blue wins!");
                return;
            }
            if self.map.red_coord.y == 8 {
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

            let current_player = if turn % 2 == 0 {
                &mut self.red_player
            } else {
                &mut self.blue_player
            };

            let ok = match cmd.to_ascii_lowercase() {
                'm' => self.map.try_step(current_player.team, coord).is_ok(),
                'v' | 'h' => {
                    let orientation = if cmd.to_ascii_lowercase() == 'v' {
                        Orientation::Vertical
                    } else {
                        Orientation::Horizontal
                    };

                    if current_player.inventory > 0
                        && self
                            .map
                            .try_place_barricade(current_player.team, coord, orientation)
                            .is_ok()
                    {
                        current_player.inventory -= 1;
                        true
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

    fn pve<B: Bot>(&mut self, mut bot: B) {
        let mut turn = 0;

        loop {
            println!("Red: {}", self.red_player.inventory);
            println!("Blue: {}", self.blue_player.inventory);
            println!("{}", self.map);

            if self.map.blue_coord.y == 0 {
                // blue wins
                println!("Blue wins!");
                return;
            }
            if self.map.red_coord.y == 8 {
                // red wins
                println!("Red wins!");
                return;
            }

            println!("{}'s turn", if turn % 2 == 0 { "Red" } else { "Blue" });

            let ok = if turn % 2 == 0 {
                // player's turn
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

                let current_player = if turn % 2 == 0 {
                    &mut self.red_player
                } else {
                    &mut self.blue_player
                };

                match cmd.to_ascii_lowercase() {
                    'm' => self.map.try_step(current_player.team, coord).is_ok(),
                    'v' | 'h' => {
                        let orientation = if cmd.to_ascii_lowercase() == 'v' {
                            Orientation::Vertical
                        } else {
                            Orientation::Horizontal
                        };

                        if current_player.inventory > 0
                            && self
                                .map
                                .try_place_barricade(current_player.team, coord, orientation)
                                .is_ok()
                        {
                            current_player.inventory -= 1;
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            } else {
                // bot's turn
                let context = Context::new(
                    self.map.clone(),
                    self.blue_player.inventory,
                    self.red_player.inventory,
                );

                sleep(Duration::from_secs(1));

                let action = bot.get_action(context);

                let current_player = if turn % 2 == 0 {
                    &mut self.red_player
                } else {
                    &mut self.blue_player
                };

                match action {
                    bot::Action::Move(coord) => {
                        self.map.try_step(current_player.team, coord).is_ok()
                    }
                    bot::Action::Barricade(coord, orientation) => {
                        if current_player.inventory > 0
                            && self
                                .map
                                .try_place_barricade(current_player.team, coord, orientation)
                                .is_ok()
                        {
                            current_player.inventory -= 1;
                            true
                        } else {
                            false
                        }
                    }
                }
            };

            if ok {
                turn += 1;
            }
        }
    }
}

#[derive(Clone)]
pub struct Map {
    barricade_grid: BGrid,
    red_coord: PCoord,
    blue_coord: PCoord,
}

#[derive(Clone)]
pub struct Barricade {
    team: Team,
    orientation: Orientation,
}

impl Map {
    pub fn new(red_coord: PCoord, blue_coord: PCoord) -> Self {
        Self {
            barricade_grid: [const { None }; 8 * 8],
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
            self.barricade_grid[bcoord_to_index(coord)] = Some(Barricade { team, orientation });
            return Ok(());
        }
        Err(())
    }

    pub fn can_place_barricade(&self, coord: BCoord, orientation: Orientation) -> bool {
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

        let red_can_finish = can_reach_y_level(&navgraph, self.red_coord, 8);
        let blue_can_finish = can_reach_y_level(&navgraph, self.blue_coord, 0);

        return !in_place_occupied && !neighbor_occupied && red_can_finish && blue_can_finish;
    }

    pub fn move_player(&mut self, team: Team, coord: PCoord) {
        match team {
            Team::Red => self.red_coord = coord,
            Team::Blue => self.blue_coord = coord,
        }
    }

    pub fn is_occupied(&self, coord: PCoord) -> bool {
        return !pcoord_in_bounds(coord) || self.red_coord == coord || self.blue_coord == coord;
    }

    pub fn get_team_at(&self, coord: PCoord) -> Option<Team> {
        if self.red_coord == coord {
            return Some(Team::Red);
        } else if self.blue_coord == coord {
            return Some(Team::Blue);
        }
        None
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
                if !is_barricade_between(&self.barricade_grid, coord, to) {
                    return true;
                }
            } else if diff.x.abs() == 2 {
                // jump move
                let between = coord + diff / 2;
                if !is_barricade_between(&self.barricade_grid, coord, between)
                    && !is_barricade_between(&self.barricade_grid, between, to)
                    && self.is_occupied(between)
                {
                    return true;
                }
            }
        } else if diff.x == 0 {
            // vertical move
            if diff.y.abs() == 1 {
                // standard move
                if !is_barricade_between(&self.barricade_grid, coord, to) {
                    return true;
                }
            } else if diff.y.abs() == 2 {
                // jump move
                let between = coord + diff / 2;
                if !is_barricade_between(&self.barricade_grid, coord, between)
                    && !is_barricade_between(&self.barricade_grid, between, to)
                    && self.is_occupied(between)
                {
                    return true;
                }
            }
        } else if diff.y.abs() == 1 && diff.y.abs() == 1 {
            // diagonal move
            let vert_first_coord = coord + PCoord::new(0, diff.y);
            let hori_first_coord = coord + PCoord::new(diff.x, 0);

            if self.is_occupied(vert_first_coord)
                && !is_barricade_between(&self.barricade_grid, coord, vert_first_coord)
                && !is_barricade_between(&self.barricade_grid, vert_first_coord, to)
            {
                // check if barricade is behind or there is the map edge
                let behind = coord + PCoord::new(0, diff.y * 2);
                return !pcoord_in_bounds(behind)
                    || is_barricade_between(&self.barricade_grid, vert_first_coord, behind);
            } else if self.is_occupied(hori_first_coord)
                && !is_barricade_between(&self.barricade_grid, coord, hori_first_coord)
                && !is_barricade_between(&self.barricade_grid, hori_first_coord, to)
            {
                // check if barricade is behind or there is the map edge
                let behind = coord + PCoord::new(diff.x * 2, 0);
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

fn index_to_bcoord(index: usize) -> PCoord {
    return BCoord::new((index % 8) as _, (index / 8) as _);
}

fn bcoord_in_bounds(coord: BCoord) -> bool {
    coord.x >= 0 && coord.x < 8 && coord.y >= 0 && coord.y < 8
}

fn pcoord_in_bounds(coord: PCoord) -> bool {
    coord.x >= 0 && coord.x < 9 && coord.y >= 0 && coord.y < 9
}
