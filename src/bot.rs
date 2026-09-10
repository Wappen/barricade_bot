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
            let paths = path::find_paths(&context.map, coord, &BLUE_FINISH_LINE);
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
                if context.map.can_step(Team::Blue, next_step) {
                    return Move(next_step);
                }
            }
        }
    }
}

pub struct EnemyPathMaximizerBot;

impl Bot for EnemyPathMaximizerBot {
    fn get_action(&mut self, context: Context) -> Action {
        let valid_barricades = context.map.find_valid_barricades();

        let ratings = valid_barricades.iter().map(|(coord, orientation)| {
            let mut test_map = context.map.clone();
            let _ = test_map.try_place_barricade(Team::Blue, *coord, *orientation);
            let path = test_map.find_shortest_path_to_win(Team::Blue);
            let enemy_path = test_map.find_shortest_path_to_win(Team::Red);

            (
                enemy_path.len() as i32 - path.len() as i32,
                coord,
                orientation,
            )
        });

        let (best_rating, best_coord, best_orientation) = ratings
            .max_by(|(rating, _, _), (other_rating, _, _)| rating.cmp(other_rating))
            .unwrap();

        if best_rating > 0 && context.inventory > 0 {
            return Barricade(best_coord.to_pcoord(), *best_orientation);
        } else {
            let shortest_path = context.map.find_shortest_path_to_win(Team::Blue);
            let step = shortest_path[1];

            if context.map.can_step(Team::Blue, step) {
                return Move(step);
            } else {
                let valid_moves = context.map.find_valid_moves(Team::Blue);
                let navgraph = context.map.barricade_grid.get_navgraph();
                let ratings = valid_moves.iter().map(|coord| {
                    (
                        navgraph
                            .find_shortest_path(*coord, &BLUE_FINISH_LINE)
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
