use std::ops::{Deref, DerefMut};

use petgraph::{stable_graph::StableUnGraph, visit::Bfs};

use crate::coord::PCoord;

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
