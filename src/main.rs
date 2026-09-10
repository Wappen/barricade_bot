pub mod bot;
pub mod path;

use colored::{Color, Colorize};
use petgraph::{stable_graph::StableUnGraph, visit::Bfs};
use std::{
    fmt::Display,
    io::stdin,
    ops::{Deref, DerefMut},
    thread::sleep,
    time::Duration,
};

use derive_more::{Add, Div, Mul, Sub};

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

#[derive(Debug, Clone, Copy, Add, Sub, Mul, Div, PartialEq, Eq, Hash)]
pub struct PCoord(IVec2);

impl PCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self(IVec2 { x, y })
    }

    pub fn from_index(index: usize) -> Self {
        PCoord::new((index % 9) as _, (index / 9) as _)
    }

    pub fn to_index(&self) -> usize {
        if !self.is_in_bounds() {
            return usize::MAX;
        }

        (self.0.x + self.0.y * 9) as _
    }

    pub fn to_bcoord(&self) -> BCoord {
        BCoord::new(self.0.x, self.0.y)
    }

    pub fn is_in_bounds(&self) -> bool {
        self.0.x >= 0 && self.0.x < 9 && self.0.y >= 0 && self.0.y < 9
    }
}

impl DerefMut for PCoord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for PCoord {
    type Target = IVec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, Add, Sub, Mul, Div, PartialEq, Eq, Hash)]
pub struct BCoord(IVec2);

impl BCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self(IVec2 { x, y })
    }

    pub fn from_index(index: usize) -> Self {
        BCoord::new((index % 8) as _, (index / 8) as _)
    }

    pub fn to_index(&self) -> usize {
        if !self.is_in_bounds() {
            return usize::MAX;
        }

        (self.0.x + self.0.y * 8) as _
    }

    pub fn to_pcoord(&self) -> PCoord {
        PCoord::new(self.0.x, self.0.y)
    }

    pub fn is_in_bounds(&self) -> bool {
        self.0.x >= 0 && self.0.x < 8 && self.0.y >= 0 && self.0.y < 8
    }
}

impl DerefMut for BCoord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for BCoord {
    type Target = IVec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

type BGrid = [Option<Barricade>; 8 * 8];

#[derive(Debug, Clone)]
pub struct BarricadeGrid(BGrid);

impl BarricadeGrid {
    pub fn new() -> Self {
        Self([const { None }; 8 * 8])
    }

    pub fn get_navgraph(&self) -> NavigationGraph {
        let mut edges = Vec::new();

        for y in 0..9 {
            for x in 0..9 {
                let coord = PCoord::new(x, y);
                let right = PCoord::new(x + 1, y);
                let below = PCoord::new(x, y + 1);

                let coord_index = coord.to_index();
                let right_index = right.to_index();
                let below_index = below.to_index();

                if right.is_in_bounds() && !self.is_barricade_between(coord, right) {
                    edges.push((coord_index, right_index));
                }
                if below.is_in_bounds() && !self.is_barricade_between(coord, below) {
                    edges.push((coord_index, below_index));
                }
            }
        }

        NavGraph::from_edges(edges).into()
    }

    pub fn is_barricade_between(&self, a: PCoord, b: PCoord) -> bool {
        let diff = b - a;
        if diff.x.abs() == 1 && diff.y == 0 {
            // horizontal move
            let bcoord_above = BCoord::new(a.x.min(b.x), a.y - 1);
            let bcoord_below = BCoord::new(a.x.min(b.x), a.y);

            let index_above = bcoord_above.to_index();
            let index_below = bcoord_below.to_index();

            return self.0.get(index_above).is_some_and(|inner| {
                inner
                    .as_ref()
                    .is_some_and(|barricade| barricade.orientation == Orientation::Vertical)
            }) | self.0.get(index_below).is_some_and(|inner| {
                inner
                    .as_ref()
                    .is_some_and(|barricade| barricade.orientation == Orientation::Vertical)
            });
        } else if diff.x == 0 && diff.y.abs() == 1 {
            // vertical move
            let bcoord_left = BCoord::new(a.x - 1, a.y.min(b.y));
            let bcoord_right = BCoord::new(a.x, a.y.min(b.y));

            let index_left = bcoord_left.to_index();
            let index_right = bcoord_right.to_index();

            return self.0.get(index_left).is_some_and(|inner| {
                inner
                    .as_ref()
                    .is_some_and(|barricade| barricade.orientation == Orientation::Horizontal)
            }) | self.0.get(index_right).is_some_and(|inner| {
                inner
                    .as_ref()
                    .is_some_and(|barricade| barricade.orientation == Orientation::Horizontal)
            });
        } else {
            // illegal move
            return false;
        }
    }
}

impl DerefMut for BarricadeGrid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for BarricadeGrid {
    type Target = BGrid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

type NavGraph = StableUnGraph<(), (), usize>;

pub struct NavigationGraph(NavGraph);

impl NavigationGraph {
    pub fn can_reach_y_level(&self, from: PCoord, y: i32) -> bool {
        let mut bfs = Bfs::new(&self.0, from.to_index().into());

        while let Some(node) = bfs.next(&self.0) {
            let coord = PCoord::from_index(node.index());

            if coord.y == y {
                return true;
            }
        }
        false
    }
}

impl From<NavGraph> for NavigationGraph {
    fn from(navgraph: NavGraph) -> Self {
        NavigationGraph(navgraph)
    }
}

impl DerefMut for NavigationGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for NavigationGraph {
    type Target = NavGraph;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
                            .try_place_barricade(
                                current_player.team,
                                coord.to_bcoord(),
                                orientation,
                            )
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
                                .try_place_barricade(
                                    current_player.team,
                                    coord.to_bcoord(),
                                    orientation,
                                )
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
                                .try_place_barricade(
                                    current_player.team,
                                    coord.to_bcoord(),
                                    orientation,
                                )
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

#[derive(Clone, Debug)]
pub struct Map {
    barricade_grid: BarricadeGrid,
    red_coord: PCoord,
    blue_coord: PCoord,
}

#[derive(Clone, Debug)]
pub struct Barricade {
    team: Team,
    orientation: Orientation,
}

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
