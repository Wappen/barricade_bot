pub mod bot;
pub mod coord;
pub mod game;
pub mod graph;
pub mod grid;
pub mod map;
pub mod path;
pub mod types;

use crate::{
    bot::{Bot, Context, EnemyPathMaximizerBot, RandomBot},
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

    // game.play(
    //     Box::new(|ctx: Context| {
    //         let start = ctx.map.get_player_coord(ctx.team);
    //         let paths = path::find_paths(&ctx.map, start, ctx.finish_line);

    //         // filter out paths which have inversely wound paths which are paths that lie behind the player
    //         // let filtered: Vec<_> = paths
    //         //     .iter()
    //         //     .filter(|&a| {
    //         //         !paths
    //         //             .iter()
    //         //             .any(|b| !a.is_homotopic_to(b) && a.has_inverse_winding(b))
    //         //     })
    //         //     .collect();
    //         // for path in filtered {
    //         //     print_path(&ctx.map, &path);
    //         // }
    //         Game::read_action_from_stdin()
    //     }),
    //     Box::new(|ctx| {
    //         let start = ctx.map.get_player_coord(ctx.team);
    //         let paths = path::find_paths(&ctx.map, start, ctx.finish_line);

    //         // filter out paths which have inversely wound paths which are paths that lie behind the player
    //         // let filtered: Vec<_> = paths
    //         //     .iter()
    //         //     .filter(|&a| {
    //         //         !paths
    //         //             .iter()
    //         //             .any(|b| !a.is_homotopic_to(b) && a.has_inverse_winding(b))
    //         //     })
    //         //     .collect();
    //         // for path in filtered {
    //         //     print_path(&ctx.map, &path);
    //         // }
    //         Game::read_action_from_stdin()
    //     }),
    // );
}
