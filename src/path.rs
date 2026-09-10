use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::{BCoord, Map, NavGraph, Orientation, PCoord};

pub type Path = Vec<PCoord>;

pub fn find_paths(map: &Map, start: PCoord, targets: &Vec<PCoord>) -> Vec<Path> {
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
type WindingVec = Vec<i32>;

#[derive(Clone, Debug)]
struct SearchState {
    node: NodeIndex<usize>,
    distance: usize,
    windings: WindingVec,
    path: Vec<PCoord>, // Track path history if needed
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

    for class in homotopy_classes.iter() {
        println!("{:?}", class);
    }

    homotopy_classes.into_values().collect()
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
