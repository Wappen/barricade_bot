use rand::RngExt;

use crate::{
    bot::Action::Move,
    coord::{BCoord, PCoord},
    map::Map,
    path::{self, Path},
    types::{Orientation, Team},
};

pub struct Context {
    map: Map,
    inventory: usize,
    enemy_inventory: usize,
}

impl Context {
    pub fn new(map: Map, inventory: usize, enemy_inventory: usize) -> Self {
        Self {
            map,
            inventory,
            enemy_inventory,
        }
    }
}

pub enum Action {
    Move(PCoord),
    Barricade(PCoord, Orientation),
}

pub trait Bot {
    fn get_action(&mut self, context: Context) -> Action;
}

pub struct RandomBot;

impl Bot for RandomBot {
    fn get_action(&mut self, context: Context) -> Action {
        loop {
            let ori = rand::rng().random_bool(0.5);
            let sgn = rand::rng().random_bool(0.5);

            let offset = if ori {
                PCoord::new(0, if sgn { 1 } else { -1 })
            } else {
                PCoord::new(if sgn { 1 } else { -1 }, 0)
            };

            let coord = context.map.blue_coord;
            let targets = (0..9).map(|x| PCoord::new(x, 0)).collect();
            let paths = path::find_paths(&context.map, coord, &targets);

            for path in &paths {
                print_path(&context.map, path);
            }

            let shortest_path = paths.iter().min_by(|a, b| a.len().cmp(&b.len())).unwrap();

            // let target = coord + offset;
            let target = shortest_path[shortest_path.len() - 2];
            if context.map.can_step(Team::Blue, target) {
                return Move(target);
            }
        }
    }
}

use colored::*;
use std::collections::HashSet;

pub fn print_path(map: &Map, path: &Path) {
    let path_set: HashSet<PCoord> = path.iter().cloned().collect();

    print!(" ");
    for x in 0..17 {
        if x % 2 == 0 {
            let c = (b'a' + x / 2 as u8) as char;
            print!(" {} ", c.to_string().bright_black());
        } else {
            print!("{}", "╷".bright_black());
        }
    }
    println!();
    print!("{}", "1".bright_black());

    for y in 0..17 {
        for x in 0..17 {
            match (x % 2, y % 2) {
                (0, 0) => {
                    let pcoord = PCoord::new(x / 2, y / 2);
                    if let Some(team) = map.get_team_at(pcoord) {
                        match team {
                            Team::Red => print!("{}", " ● ".red()),
                            Team::Blue => print!("{}", " ● ".blue()),
                        }
                    } else if path_set.contains(&pcoord) {
                        print!("{}", " · ".yellow());
                    } else {
                        print!("   ");
                    }
                }
                (1, 0) => {
                    let bcoord_above = BCoord::new((x - 1) / 2, (y - 2) / 2);
                    let bcoord_below = BCoord::new((x - 1) / 2, y / 2);
                    let index_above = bcoord_above.to_index();
                    let index_below = bcoord_below.to_index();

                    let mut has_barricade = None;

                    if let Some(Some(barricade)) = map.barricade_grid.get(index_above) {
                        if barricade.orientation == Orientation::Vertical {
                            has_barricade = Some(barricade);
                        }
                    }
                    if let Some(Some(barricade)) = map.barricade_grid.get(index_below) {
                        if barricade.orientation == Orientation::Vertical {
                            has_barricade = Some(barricade);
                        }
                    }

                    if let Some(barricade) = has_barricade {
                        print!("{}", "┃".color(barricade.team));
                    } else {
                        print!("{}", "│".bright_black());
                    }
                }
                (0, 1) => {
                    let bcoord_left = BCoord::new((x - 2) / 2, (y - 1) / 2);
                    let bcoord_right = BCoord::new(x / 2, (y - 1) / 2);
                    let index_left = bcoord_left.to_index();
                    let index_right = bcoord_right.to_index();

                    let mut has_barricade = None;

                    if let Some(Some(barricade)) = map.barricade_grid.get(index_left) {
                        if barricade.orientation == Orientation::Horizontal {
                            has_barricade = Some(barricade);
                        }
                    }
                    if let Some(Some(barricade)) = map.barricade_grid.get(index_right) {
                        if barricade.orientation == Orientation::Horizontal {
                            has_barricade = Some(barricade);
                        }
                    }

                    if let Some(barricade) = has_barricade {
                        print!("{}", "━━━".color(barricade.team));
                    } else {
                        print!("{}", "───".bright_black());
                    }
                }
                (1, 1) => {
                    let bcoord = BCoord::new((x - 1) / 2, (y - 1) / 2);
                    let index = bcoord.to_index();

                    if let Some(barricade) = &map.barricade_grid[index] {
                        match barricade.orientation {
                            Orientation::Horizontal => print!("{}", "┿".color(barricade.team)),
                            Orientation::Vertical => print!("{}", "╂".color(barricade.team)),
                        }
                    } else {
                        print!("{}", "┼".bright_black());
                    }
                }
                _ => {}
            }

            if x == 16 {
                if y % 2 == 0 {
                    print!("{}", ((y + 1) / 2 + 1).to_string().bright_black());
                } else {
                    print!("{}", "╴".bright_black());
                }
                println!();
                if y % 2 == 0 {
                    if y != 16 {
                        print!("{}", "╶".bright_black());
                    } else {
                        print!(" ");
                    }
                } else {
                    print!("{}", ((y + 1) / 2 + 1).to_string().bright_black());
                }
            }
        }
    }

    for x in 0..17 {
        if x % 2 == 0 {
            let c = (b'a' + x / 2 as u8) as char;
            print!(" {} ", c.to_string().bright_black());
        } else {
            print!("{}", "╵".bright_black());
        }
    }
    println!();
}
