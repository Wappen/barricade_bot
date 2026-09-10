use derive_more::{Add, Div, Mul, Sub};
use glam::IVec2;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy, Add, Sub, Mul, Div, PartialEq, Eq, Hash)]
pub struct PCoord(IVec2);

impl PCoord {
    pub const fn new(x: i32, y: i32) -> Self {
        Self(IVec2 { x, y })
    }

    pub fn from_index(index: usize) -> Self {
        PCoord::new((index % 9) as _, (index / 9) as _)
    }

    pub fn to_index(&self) -> usize {
        if !self.is_in_bounds() {
            return usize::MAX;
        }

        (self.0.x + self.0.y * 9) as _
    }

    pub fn to_bcoord(&self) -> BCoord {
        BCoord::new(self.0.x, self.0.y)
    }

    pub fn is_in_bounds(&self) -> bool {
        self.0.x >= 0 && self.0.x < 9 && self.0.y >= 0 && self.0.y < 9
    }
}

impl DerefMut for PCoord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for PCoord {
    type Target = IVec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, Add, Sub, Mul, Div, PartialEq, Eq, Hash)]
pub struct BCoord(IVec2);

impl BCoord {
    pub fn new(x: i32, y: i32) -> Self {
        Self(IVec2 { x, y })
    }

    pub fn from_index(index: usize) -> Self {
        BCoord::new((index % 8) as _, (index / 8) as _)
    }

    pub fn to_index(&self) -> usize {
        if !self.is_in_bounds() {
            return usize::MAX;
        }

        (self.0.x + self.0.y * 8) as _
    }

    pub fn to_pcoord(&self) -> PCoord {
        PCoord::new(self.0.x, self.0.y)
    }

    pub fn is_in_bounds(&self) -> bool {
        self.0.x >= 0 && self.0.x < 8 && self.0.y >= 0 && self.0.y < 8
    }
}

impl DerefMut for BCoord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for BCoord {
    type Target = IVec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
