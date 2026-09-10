use rand::RngExt;

use crate::{
    bot::Action::Move,
    coord::PCoord,
    map::Map,
    path::{self, print_path},
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
