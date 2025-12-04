// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Brain architecture types
//!
//! Moved from feagi-types/src/models/ (Phase 2c)

// Dimensions removed - use feagi_data_structures::genomic::cortical_area::CorticalAreaDimensions

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::{collections::HashMap, string::String, vec::Vec};


// CorticalArea stub removed - use feagi_data_structures or feagi-bdu versions
// This was a minimal runtime placeholder that's no longer needed

// BrainRegion, RegionType, and BrainRegionHierarchy moved to feagi_data_structures
// Use: feagi_data_structures::genomic::brain_regions::BrainRegion
// Use: feagi_data_structures::genomic::brain_regions::RegionType

// Placeholder types for no_std
#[cfg(not(feature = "std"))]
pub struct CorticalArea;

