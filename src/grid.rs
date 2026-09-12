use std::{
    collections::{HashMap, HashSet, VecDeque},
    ops::{Deref, DerefMut},
};

use petgraph::graph::NodeIndex;

use crate::{
    coord::{BCoord, PCoord},
    graph::{NavGraph, NavigationGraph},
    path::{Path, WindingVec, step_ray_winding},
    types::{Barricade, Orientation},
};

type BGrid = [Option<Barricade>; 8 * 8];

#[derive(Debug, Clone)]
pub struct BarricadeGrid(BGrid);

impl BarricadeGrid {
    pub fn new() -> Self {
        Self([const { None }; 8 * 8])
    }

    pub fn get_navgraph(&self) -> NavigationGraph {
        let mut edges = Vec::new();

        for y in 0..9 {
            for x in 0..9 {
                let coord = PCoord::new(x, y);
                let right = PCoord::new(x + 1, y);
                let below = PCoord::new(x, y + 1);

                let coord_index = coord.to_index();
                let right_index = right.to_index();
                let below_index = below.to_index();

                if right.is_in_bounds() && !self.is_barricade_between(coord, right) {
                    edges.push((coord_index, right_index));
                }
                if below.is_in_bounds() && !self.is_barricade_between(coord, below) {
                    edges.push((coord_index, below_index));
                }
            }
        }

        NavGraph::from_edges(edges).into()
    }

    pub fn is_barricade_between(&self, a: PCoord, b: PCoord) -> bool {
        let diff = b - a;
        if diff.x.abs() == 1 && diff.y == 0 {
            // horizontal move
            let bcoord_above = BCoord::new(a.x.min(b.x), a.y - 1);
            let bcoord_below = BCoord::new(a.x.min(b.x), a.y);

            let index_above = bcoord_above.to_index();
            let index_below = bcoord_below.to_index();

            return self.0.get(index_above).is_some_and(|inner| {
                inner
                    .as_ref()
                    .is_some_and(|barricade| barricade.orientation == Orientation::Vertical)
            }) | self.0.get(index_below).is_some_and(|inner| {
                inner
                    .as_ref()
                    .is_some_and(|barricade| barricade.orientation == Orientation::Vertical)
            });
        } else if diff.x == 0 && diff.y.abs() == 1 {
            // vertical move
            let bcoord_left = BCoord::new(a.x - 1, a.y.min(b.y));
            let bcoord_right = BCoord::new(a.x, a.y.min(b.y));

            let index_left = bcoord_left.to_index();
            let index_right = bcoord_right.to_index();

            return self.0.get(index_left).is_some_and(|inner| {
                inner
                    .as_ref()
                    .is_some_and(|barricade| barricade.orientation == Orientation::Horizontal)
            }) | self.0.get(index_right).is_some_and(|inner| {
                inner
                    .as_ref()
                    .is_some_and(|barricade| barricade.orientation == Orientation::Horizontal)
            });
        } else {
            // illegal move
            return false;
        }
    }

    pub fn find_paths(&self, start: PCoord, targets: &[PCoord]) -> HashMap<WindingVec, Path> {
        let rays: Vec<_> = self
            .iter()
            .enumerate()
            .filter_map(|(index, option)| {
                option
                    .as_ref()
                    .map(|barricade| (BCoord::from_index(index).to_pcoord(), barricade.orientation))
            })
            .collect();

        let graph = self.get_navgraph();
        let start_nodes = targets
            .iter()
            .map(|target| NodeIndex::new(target.to_index()))
            .collect();

        return reverse_bfs_iterative(&graph, &rays, start, start_nodes);
    }
}

type Ray = (PCoord, Orientation);

#[derive(Clone, Debug)]
struct SearchState {
    node: NodeIndex<usize>,
    distance: usize,
    windings: WindingVec,
    path: Path,
}

fn reverse_bfs_iterative(
    graph: &NavGraph,
    rays: &[Ray],
    finish: PCoord,
    start_nodes: Vec<NodeIndex<usize>>,
) -> HashMap<WindingVec, Path> {
    let mut queue = VecDeque::new();
    let finish_index = finish.to_index();

    // Initialize queue with target nodes (e.g., winning row)
    for node in start_nodes {
        queue.push_back(SearchState {
            node,
            distance: 0,
            windings: vec![0; rays.len()],
            path: vec![PCoord::from_index(node.index())].into(),
        });
    }

    // Visited set tracks both position AND homotopy signature
    let mut visited: HashSet<(NodeIndex<usize>, WindingVec)> = HashSet::new();
    let mut homotopy_classes: HashMap<WindingVec, Path> = HashMap::new();

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
                let dw = step_ray_winding(ray, current_coord, next_coord);
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
}

impl DerefMut for BarricadeGrid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for BarricadeGrid {
    type Target = BGrid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
