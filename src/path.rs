use colored::*;
use std::ops::{Deref, DerefMut};

use crate::{
    coord::{BCoord, PCoord},
    map::Map,
    types::{Orientation, Team},
};

pub type WindingVec = Vec<i32>;

// #[derive(Clone, PartialEq, Eq)]
// pub struct HomotopySignature {
//     start: PCoord,
//     end: PCoord,
//     winding_vec: WindingVec,
// }

#[derive(Debug, Clone, Copy)]
pub struct BlockadeMetrics {
    pub length_increase: usize,
    pub is_proper_detour: bool,
    pub barricade: (BCoord, Orientation),
}

#[derive(Debug, Clone, Default)]
pub struct Path(Vec<PCoord>);

impl Path {
    pub fn calculate_winding(&self, map: &Map) -> WindingVec {
        let rays = map
            .barricade_grid
            .iter()
            .enumerate()
            .filter_map(|(index, option)| {
                option
                    .as_ref()
                    .map(|barricade| (BCoord::from_index(index).to_pcoord(), barricade.orientation))
            });

        rays.map(|r| {
            self.windows(2)
                .map(|w| step_ray_winding(&r, w[0], w[1]))
                .sum()
        })
        .collect()
    }

    // finds barricade spots that block the path
    // returns a list of placable barricades together with the index of the step behind which the barricade is placed
    pub fn find_blockades(&self, map: &Map) -> Vec<(usize, BCoord, Orientation)> {
        let mut blockades = vec![];

        for (index, step) in self.windows(2).enumerate() {
            let from = step[0];
            let to = step[1];

            let diff = to - from;

            if diff.x.abs() == 1 && diff.y.abs() == 0 {
                // horizontal move
                let left = from.x.min(to.x);
                let above = BCoord::new(left, from.y - 1);
                let below = BCoord::new(left, from.y);

                if map.can_place_barricade(above, Orientation::Vertical) {
                    blockades.push((index, above, Orientation::Vertical));
                }
                if map.can_place_barricade(below, Orientation::Vertical) {
                    blockades.push((index, below, Orientation::Vertical));
                }
            } else if diff.x.abs() == 0 && diff.y.abs() == 1 {
                // vertical move
                let upper = from.y.min(to.y);
                let left = BCoord::new(from.x - 1, upper);
                let right = BCoord::new(from.x, upper);

                if map.can_place_barricade(left, Orientation::Horizontal) {
                    blockades.push((index, left, Orientation::Horizontal));
                }
                if map.can_place_barricade(right, Orientation::Horizontal) {
                    blockades.push((index, right, Orientation::Horizontal));
                }
            }
        }

        blockades
    }

    // calculates the max length increase a single barricade can have on that path
    // in a way that simulates following that path until the worst barricade is placed in the last moment
    // which is why finish is needed, to calculate the actual impact on winnability for that path.
    // Also returns a bool indicating if the detour changed the homotopy class of the resulting detour
    // Also returns the best barricade
    pub fn calculate_blockability(&self, map: &Map, finish: &[PCoord]) -> Option<BlockadeMetrics> {
        let blockades = self.find_blockades(map);
        let mut max_length_increase = 0;
        let mut max_detour = Path::default();
        let mut best_barricade = None;

        for (index, coord, orientation) in &blockades {
            let mut test_map = map.clone();
            // TODO: team should not be necessary to test barricade impact
            test_map
                .try_place_barricade(Team::Red, *coord, *orientation)
                .expect("could not place barricade at calculated blockade spot");

            let graph = test_map.barricade_grid.get_navgraph();
            let start = self[*index];

            // if no shortest path is found the barricade should be illegal from that start
            if let Some(shortest_path_continuation) = graph.find_shortest_path(start, &finish) {
                if shortest_path_continuation.len() > max_length_increase {
                    max_length_increase = shortest_path_continuation.len();
                    max_detour = self[0..*index].into();
                    max_detour.extend_from_slice(&shortest_path_continuation[1..]);
                    best_barricade = Some((*coord, *orientation));
                }
            }
        }

        // calculate detour winding for original map for comparability
        let detour_winding = max_detour.calculate_winding(map);
        let original_winding = self.calculate_winding(map);

        let is_proper_detour = detour_winding != original_winding;
        best_barricade.map(|b| BlockadeMetrics {
            length_increase: max_length_increase,
            is_proper_detour,
            barricade: b,
        })
    }

    pub fn find_first_fork(paths: &[Path]) -> Option<PCoord> {
        if paths.len() < 2 {
            return None;
        }

        let min_len = paths.iter().map(|p| p.len()).min()?;
        if min_len == 0 {
            return None;
        }

        let mut last_common_idx = 0;

        for i in 0..min_len {
            let reference_coord = paths[0][i];
            if paths.iter().all(|path| path[i] == reference_coord) {
                last_common_idx = i;
            } else {
                break;
            }
        }

        if last_common_idx < min_len - 1 {
            Some(paths[0][last_common_idx])
        } else {
            let first_len = paths[0].len();
            if paths.iter().any(|p| p.len() != first_len) {
                Some(paths[0][min_len - 1])
            } else {
                None
            }
        }
    }

    pub fn print(&self, map: &Map) {
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
                        } else if let Some(index) = self.iter().position(|coord| *coord == pcoord) {
                            let prev = self[index - 1];
                            if let Some(next) = self.get(index + 1) {
                                let prev_diff = prev - pcoord;
                                let next_diff = *next - pcoord;

                                let left = prev_diff.x == -1 || next_diff.x == -1;
                                let right = prev_diff.x == 1 || next_diff.x == 1;
                                let up = prev_diff.y == -1 || next_diff.y == -1;
                                let down = prev_diff.y == 1 || next_diff.y == 1;

                                if left && right {
                                    print!("{}", "╴─╶".yellow());
                                } else if left && up {
                                    print!("{}", "╴╯ ".yellow());
                                } else if left && down {
                                    print!("{}", "╴╮ ".yellow());
                                } else if right && up {
                                    print!("{}", " ╰╶".yellow());
                                } else if right && down {
                                    print!("{}", " ╭╶".yellow());
                                } else if up && down {
                                    print!("{}", " ╎ ".yellow());
                                }
                            } else {
                                print!("{}", " ◈ ".yellow());
                            }
                            // print!("{}", " · ".yellow());
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
}

impl From<Vec<PCoord>> for Path {
    fn from(value: Vec<PCoord>) -> Self {
        Self(value)
    }
}

impl From<&[PCoord]> for Path {
    fn from(value: &[PCoord]) -> Self {
        Self(value.to_vec())
    }
}

impl DerefMut for Path {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for Path {
    type Target = Vec<PCoord>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// note: maybe could be smiplified by treating horizontal and vertical barricades the same
pub fn step_ray_winding(ray: &(PCoord, Orientation), from: PCoord, to: PCoord) -> i32 {
    match ray.1 {
        Orientation::Horizontal => {
            if from.x >= ray.0.x && to.x >= ray.0.x {
                // stays on the half space thats defined by the rays negative normal plane
                if from.y <= ray.0.y && to.y > ray.0.y {
                    // -1 winding
                    return -1;
                } else if from.y > ray.0.y && to.y <= ray.0.y {
                    // +1 winding
                    return 1;
                } else {
                    return 0;
                }
            } else {
                // note: unhandled case when move is diagonal
                return 0;
            }
        }
        Orientation::Vertical => {
            if from.y >= ray.0.y && to.y >= ray.0.y {
                // stays on the half space thats defined by the rays negative normal plane
                if from.x > ray.0.x && to.x <= ray.0.x {
                    // -1 winding
                    return -1;
                } else if from.x <= ray.0.x && to.x > ray.0.x {
                    // +1 winding
                    return 1;
                } else {
                    return 0;
                }
            } else {
                // note: unhandled case when move is diagonal
                return 0;
            }
        }
    }
}
