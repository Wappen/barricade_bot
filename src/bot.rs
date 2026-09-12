use std::{thread::sleep, time::Duration};

use crate::{
    bot::Action::Move,
    coord::PCoord,
    game::Action::{self, Barricade},
    map::Map,
    path::{self, calculate_blockability, find_blockades, print_path},
    types::Team,
};

pub struct Context {
    pub map: Map,
    pub inventory: usize,
    pub enemy_inventory: usize,
    pub team: Team,
    pub enemy_team: Team,
    pub finish_line: &'static [PCoord],
    pub enemy_finish_line: &'static [PCoord],
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
            let coord = context.map.get_player_coord(context.team);
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

        let rated_barricades = valid_barricades.iter().map(|(coord, orientation)| {
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

        let (best_rating, best_coord, best_orientation) = rated_barricades
            .max_by(|(rating, _, _), (other_rating, _, _)| rating.cmp(other_rating))
            .unwrap();

        if best_rating > current_rating && ctx.inventory > 0 {
            return Barricade(best_coord.to_pcoord(), *best_orientation);
        } else {
            let coord = ctx.map.get_player_coord(ctx.team);
            let paths = path::find_paths(&ctx.map, coord, ctx.finish_line);

            // What it does:
            // find stable paths
            // if stable paths is not empty => follow shortest stable path
            // else (stable paths is empty) find first fork
            // if at first fork => ???
            // else (we are not at first fork) continue shortest path
            //
            // note: counter blocking should be introduced to find real possibilities for the bot to fight for its shortest path

            let path_metrics: Vec<_> = paths
                .iter()
                .map(|path| {
                    (
                        path,
                        calculate_blockability(&ctx.map, path, ctx.finish_line),
                    )
                })
                .collect();

            let stable_paths: Vec<_> = path_metrics
                .iter()
                .filter_map(|(path, metrics)| {
                    let (length_increase, is_proper_detour) =
                        metrics.map_or((0, false), |m| (m.length_increase, m.is_proper_detour));

                    if is_proper_detour {
                        None
                    } else {
                        Some((length_increase, path))
                    }
                })
                .collect();

            if !stable_paths.is_empty() {
                // follow shortest stable path
                let best_stable_path = stable_paths
                    .iter()
                    .min_by(|path, other| (path.0 + path.1.len()).cmp(&(other.0 + other.1.len())))
                    .map(|(_, path)| *path).expect("could not find a best stable path although stable paths should be not empty");
                let step = best_stable_path[1];

                if ctx.map.can_step(ctx.team, step) {
                    return Move(step);
                } else {
                    // we are being blocked by the other player
                    // => find move along best_stable_path
                    let valid_moves = ctx.map.find_valid_moves(ctx.team);
                    if let Some(valid_step) = valid_moves
                        .iter()
                        .find(|valid_move| best_stable_path.contains(*valid_move))
                    {
                        // jump along best stable path if possible
                        return Move(*valid_step);
                    } else {
                        // jump to location with shortest path
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
            } else {
                let first_fork = path::find_first_fork(&paths);

                if first_fork.is_some_and(|fork| fork == coord) && ctx.inventory > 0 {
                    // do not move as we would commit to the fork
                    // instead we must find a way out of this dilemma
                    // =>
                    // The strategy employed is to block the worst of the routes until one remains:
                    // find barricade that blocks the longest path
                    // place a barricade at the choke point

                    let (coord, orientation) = path_metrics
                        .iter()
                        .map(|(p, m)| (p, m.expect("metric was none although that should have resulted in a stable path so that we shouldn't arrive here")))
                        .max_by(|(path, metrics), (other_path, other_metrics)| {
                    (path.len() + metrics.length_increase).cmp(&(other_path.len() + other_metrics.length_increase))
                        })
                        .map(|(_, metric)| (metric.barricade.0, metric.barricade.1)).expect("could not find longest path allthough there should exist at least a path");

                    return Barricade(coord.to_pcoord(), orientation);
                } else {
                    // we are safe to move because the path doesnt fork (yet) OR we have no choice but to move because our inventory is empty
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
    }
}
