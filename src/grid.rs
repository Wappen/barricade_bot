use std::ops::{Deref, DerefMut};

use crate::{
    coord::{BCoord, PCoord},
    graph::{NavGraph, NavigationGraph},
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
