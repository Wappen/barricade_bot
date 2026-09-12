use std::ops::{Deref, DerefMut};

use petgraph::{algo::astar, graph::NodeIndex, stable_graph::StableUnGraph, visit::Bfs};

use crate::{coord::PCoord, path::Path};

pub type NavGraph = StableUnGraph<(), (), usize>;

pub struct NavigationGraph(NavGraph);

impl NavigationGraph {
    pub fn can_reach_y_level(&self, from: PCoord, y: i32) -> bool {
        let mut bfs = Bfs::new(&self.0, from.to_index().into());

        while let Some(node) = bfs.next(&self.0) {
            let coord = PCoord::from_index(node.index());

            if coord.y == y {
                return true;
            }
        }
        false
    }

    pub fn find_shortest_path(&self, start: PCoord, finish: &[PCoord]) -> Option<Path> {
        let path = astar(
            &self.0,
            NodeIndex::new(start.to_index()),
            |n| finish.contains(&PCoord::from_index(n.index())),
            |_| 1,
            |_| 0,
        );

        path.map(|(_, nodes)| {
            nodes
                .iter()
                .map(|n| PCoord::from_index(n.index()))
                .collect::<Vec<_>>()
                .into()
        })
    }
}

impl From<NavGraph> for NavigationGraph {
    fn from(navgraph: NavGraph) -> Self {
        NavigationGraph(navgraph)
    }
}

impl DerefMut for NavigationGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for NavigationGraph {
    type Target = NavGraph;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
