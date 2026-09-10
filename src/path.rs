use colored::*;
use petgraph::graph::NodeIndex;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    ops::{Deref, DerefMut},
};

use crate::{
    coord::{BCoord, PCoord},
    graph::NavGraph,
    map::Map,
    types::{Orientation, Team},
};

#[derive(Debug, Clone)]
pub struct Path {
    steps: Vec<PCoord>,
    windings: WindingVec,
}

impl Path {
    pub fn new(steps: Vec<PCoord>, windings: WindingVec) -> Self {
        Self { steps, windings }
    }

    pub fn is_homotopic_to(&self, other: &Path) -> bool {
        self.steps.first().eq(&other.first())
            && self.last().eq(&other.last())
            && self.windings.iter().eq(&other.windings)
    }

    pub fn reverse(&mut self) {
        self.steps.reverse();
        self.windings.iter_mut().for_each(|w| *w = -*w);
    }
}

impl DerefMut for Path {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.steps
    }
}

impl Deref for Path {
    type Target = Vec<PCoord>;

    fn deref(&self) -> &Self::Target {
        &self.steps
    }
}

pub fn find_paths(map: &Map, start: PCoord, targets: &[PCoord]) -> Vec<Path> {
    let rays: Vec<_> = map
        .barricade_grid
        .iter()
        .enumerate()
        .filter_map(|(index, option)| {
            option
                .as_ref()
                .map(|barricade| (BCoord::from_index(index).to_pcoord(), barricade.orientation))
        })
        .collect();

    println!("rays: {:?}", &rays);

    let graph = map.barricade_grid.get_navgraph();
    let start_nodes = targets
        .iter()
        .map(|target| NodeIndex::new(target.to_index()))
        .collect();

    return reverse_bfs_iterative(&graph, &rays, start, start_nodes);
}

type Ray = (PCoord, Orientation);
pub type WindingVec = Vec<i32>;

#[derive(Clone, Debug)]
struct SearchState {
    node: NodeIndex<usize>,
    distance: usize,
    windings: WindingVec,
    path: Vec<PCoord>,
}

fn reverse_bfs_iterative(
    graph: &NavGraph,
    rays: &[Ray],
    finish: PCoord,
    start_nodes: Vec<NodeIndex<usize>>,
) -> Vec<Path> {
    let mut queue = VecDeque::new();
    let finish_index = finish.to_index();

    // Initialize queue with target nodes (e.g., winning row)
    for node in start_nodes {
        queue.push_back(SearchState {
            node,
            distance: 0,
            windings: vec![0; rays.len()],
            path: vec![PCoord::from_index(node.index())],
        });
    }

    // Visited set tracks both position AND homotopy signature
    let mut visited: HashSet<(NodeIndex<usize>, WindingVec)> = HashSet::new();
    let mut homotopy_classes: HashMap<WindingVec, Vec<PCoord>> = HashMap::new();

    while let Some(state) = queue.pop_front() {
        if state.node.index() == finish_index {
            let is_simple = {
                let mut seen = HashSet::new();
                state.path.iter().all(|coord| seen.insert(*coord))
            };

            if is_simple {
                homotopy_classes
                    .entry(state.windings.clone())
                    .or_insert(state.path);
            }
            continue;
        }

        for neighbor in graph.neighbors(state.node) {
            let current_coord = PCoord::from_index(state.node.index());
            let next_coord = PCoord::from_index(neighbor.index());

            // Calculate step windings across all rays
            let mut new_windings = state.windings.clone();
            let mut valid_windings = true;

            for (i, ray) in rays.iter().enumerate() {
                let dw = ray_winding(ray, current_coord, next_coord);
                new_windings[i] += dw;

                // Prune paths that loop too many times around the same barricade ray
                if new_windings[i].abs() > 1 {
                    valid_windings = false;
                    break;
                }
            }

            if !valid_windings {
                continue;
            }

            let visit_key = (neighbor, new_windings.clone());
            if visited.insert(visit_key) {
                let mut next_path = state.path.clone();
                next_path.push(next_coord);

                queue.push_back(SearchState {
                    node: neighbor,
                    distance: state.distance + 1,
                    windings: new_windings,
                    path: next_path,
                });
            }
        }
    }

    homotopy_classes
        .into_iter()
        .map(|(windings, mut steps)| {
            steps.reverse();
            Path::new(steps, windings)
        })
        .collect()
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

// note: maybe could be smiplified by treating horizontal and vertical barricades the same
fn ray_winding(ray: &(PCoord, Orientation), from: PCoord, to: PCoord) -> i32 {
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

pub fn print_path(map: &Map, path: &[PCoord]) {
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
                    } else if let Some(index) = path.iter().position(|coord| *coord == pcoord) {
                        let prev = path[index - 1];
                        if let Some(next) = path.get(index + 1) {
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
