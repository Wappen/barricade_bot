pub mod bot;
pub mod coord;
pub mod game;
pub mod graph;
pub mod grid;
pub mod map;
pub mod path;
pub mod types;

use crate::{bot::RandomBot, game::Game};

fn main() {
    println!("Barricade!");
    let mut game = Game::new();
    // game.run();
    game.pve(RandomBot);
}
