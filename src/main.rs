pub mod bot;
pub mod coord;
pub mod game;
pub mod graph;
pub mod grid;
pub mod map;
pub mod path;
pub mod types;

use crate::{
    bot::{Bot, EnemyPathMaximizerBot, RandomBot},
    game::Game,
};

fn main() {
    println!("Barricade!");
    let mut game = Game::new();
    // game.run();
    game.pve(EnemyPathMaximizerBot);

    // let mut red = EnemyPathMaximizerBot;
    // let mut blue = EnemyPathMaximizerBot;
    // game.play(
    //     Box::new(|ctx| Some(red.get_action(ctx))),
    //     Box::new(|ctx| Some(blue.get_action(ctx))),
    // );
    // game.pvp();
}
