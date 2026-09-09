use rand::RngExt;

use crate::{Map, Orientation, PCoord, Team, bot::Action::Move};

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

            let target = context.map.blue_coord + offset;
            if context.map.can_step(Team::Blue, target) {
                return Move(target);
            }
        }
    }
}
