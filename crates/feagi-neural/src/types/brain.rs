// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Brain architecture types
//!
//! Moved from feagi-types/src/models/ (Phase 2c)

use super::spatial::Dimensions;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::{collections::HashMap, string::String, vec::Vec};


/// Cortical area (runtime representation)
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CorticalArea {
    pub name: String,
    pub dimensions: Dimensions,
    pub neuron_count: usize,
    pub properties: HashMap<String, String>,
}

/// Brain region type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum RegionType {
    Cortical,
    Subcortical,
    Custom,
}

/// Brain region (runtime representation)
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BrainRegion {
    pub name: String,
    pub region_type: RegionType,
    pub areas: Vec<String>,
}

/// Brain region hierarchy
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BrainRegionHierarchy {
    pub regions: HashMap<String, BrainRegion>,
}

// Placeholder types for no_std
#[cfg(not(feature = "std"))]
pub struct CorticalArea;

#[cfg(not(feature = "std"))]
pub struct BrainRegion;

#[cfg(not(feature = "std"))]
pub struct BrainRegionHierarchy;

