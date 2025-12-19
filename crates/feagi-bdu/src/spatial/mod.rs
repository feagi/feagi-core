// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Spatial indexing utilities for efficient position-based queries.

Implements Morton encoding (Z-order curve) + Roaring bitmaps for:
- Fast neuron position lookups
- Region queries
- Spatial locality preservation
*/

pub mod hash;
pub mod morton;

pub use hash::{MortonSpatialHash, SpatialHashStats};
pub use morton::{morton_decode_3d, morton_encode_3d, morton_encode_region_3d};
