// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Spatial types for 3D brain coordinates

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// 3D dimensions for cortical areas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
}

impl Dimensions {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        Self { width, height, depth }
    }

    pub fn from_tuple(tuple: (usize, usize, usize)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }

    pub fn to_tuple(&self) -> (usize, usize, usize) {
        (self.width, self.height, self.depth)
    }

    pub fn volume(&self) -> usize {
        self.width * self.height * self.depth
    }

    pub fn total_voxels(&self) -> usize {
        self.volume()
    }

    pub fn contains(&self, pos: (u32, u32, u32)) -> bool {
        pos.0 < self.width as u32
            && pos.1 < self.height as u32
            && pos.2 < self.depth as u32
    }
}

/// 3D position (x, y, z) in brain space
pub type Position = (i32, i32, i32);
