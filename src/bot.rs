use std::{thread::sleep, time::Duration};

use crate::{
    bot::Action::Move,
    coord::PCoord,
    game::Action::{self, Barricade},
    map::{BLUE_FINISH_LINE, Map},
    path::{self, print_path},
    types::Team,
};

pub struct Context {
    map: Map,
    inventory: usize,
    enemy_inventory: usize,
    team: Team,
    enemy_team: Team,
    finish_line: &'static [PCoord],
    enemy_finish_line: &'static [PCoord],
}

impl Context {
    pub fn new(
        map: Map,
        inventory: usize,
        enemy_inventory: usize,
        team: Team,
        enemy_team: Team,
        finish_line: &'static [PCoord],
        enemy_finish_line: &'static [PCoord],
    ) -> Self {
        Self {
            map,
            inventory,
            enemy_inventory,
            team,
            enemy_team,
            finish_line,
            enemy_finish_line,
        }
    }
}

pub trait Bot {
    fn get_action(&mut self, context: Context) -> Action;
}

pub struct RandomBot;

impl Bot for RandomBot {
    fn get_action(&mut self, context: Context) -> Action {
        sleep(Duration::from_secs_f32(0.5));
        loop {
            let coord = context.map.blue_coord;
            // let targets = (0..9).map(|x| PCoord::new(x, 0)).collect();
            let paths = path::find_paths(&context.map, coord, context.finish_line);
            let first_fork = path::find_first_fork(&paths);

            for (index, path) in paths.iter().enumerate() {
                println!("Path ({}/{})", index + 1, paths.len());
                print_path(&context.map, path);
            }

            if first_fork.is_some_and(|fork| fork == coord) {
                // dont walk but rather do something else
                todo!()
            } else {
                // walk the common path

                // let shortest_path = paths.iter().min_by(|a, b| a.len().cmp(&b.len())).unwrap();

                let next_step = paths[0][1];
                if context.map.can_step(context.team, next_step) {
                    return Move(next_step);
                }
            }
        }
    }
}

pub struct EnemyPathMaximizerBot;

impl Bot for EnemyPathMaximizerBot {
    fn get_action(&mut self, ctx: Context) -> Action {
        let valid_barricades = ctx.map.find_valid_barricades();

        let path = ctx.map.find_shortest_path_to_win(ctx.team);
        let enemy_path = ctx.map.find_shortest_path_to_win(ctx.enemy_team);
        let current_rating = enemy_path.len() as i32 - path.len() as i32;

        let ratings = valid_barricades.iter().map(|(coord, orientation)| {
            let mut test_map = ctx.map.clone();
            let _ = test_map.try_place_barricade(ctx.team, *coord, *orientation);
            let path = test_map.find_shortest_path_to_win(ctx.team);
            let enemy_path = test_map.find_shortest_path_to_win(ctx.enemy_team);

            (
                enemy_path.len() as i32 - path.len() as i32,
                coord,
                orientation,
            )
        });

        let (best_rating, best_coord, best_orientation) = ratings
            .max_by(|(rating, _, _), (other_rating, _, _)| rating.cmp(other_rating))
            .unwrap();

        if best_rating > current_rating && ctx.inventory > 0 {
            return Barricade(best_coord.to_pcoord(), *best_orientation);
        } else {
            let shortest_path = ctx.map.find_shortest_path_to_win(ctx.team);
            let step = shortest_path[1];

            if ctx.map.can_step(ctx.team, step) {
                return Move(step);
            } else {
                let valid_moves = ctx.map.find_valid_moves(ctx.team);
                let navgraph = ctx.map.barricade_grid.get_navgraph();
                let ratings = valid_moves.iter().map(|coord| {
                    (
                        navgraph
                            .find_shortest_path(*coord, &ctx.finish_line)
                            .map_or(0, |path| path.len()),
                        coord,
                    )
                });

                let (_, best_coord) = ratings
                    .min_by(|(a, _), (b, _)| a.cmp(b))
                    .expect("could not find move that yields the shortest path to finish");

                return Move(*best_coord);
            }
        }
    }
}
