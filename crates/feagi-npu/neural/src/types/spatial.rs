// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Spatial types for 3D brain coordinates

// Dimensions moved to feagi_structures::genomic::cortical_area::CorticalAreaDimensions
// Use: feagi_structures::genomic::cortical_area::CorticalAreaDimensions

/// 3D position (x, y, z) in brain space
pub type Position = (i32, i32, i32);
