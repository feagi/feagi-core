// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Core data models for FEAGI BDU.

This module contains the fundamental data structures that represent the
brain's architecture, including cortical areas, brain regions, and their
hierarchical organization.

## Architecture

Mirrors the Python BDU models while leveraging Rust's type system for safety:
- `CorticalArea`: Individual processing areas (sensory, motor, memory)
- `BrainRegion`: Hierarchical organization of cortical areas
- `BrainRegionHierarchy`: Tree structure of brain regions

## Design Principles

1. **Type Safety**: Use Rust's strong typing to prevent invalid states
2. **Immutability**: Core properties are immutable; updates create new instances
3. **Validation**: All constructors validate invariants
4. **Performance**: Optimized for the hot path (connectome queries)
5. **Serializability**: All types implement Serialize/Deserialize for genome I/O

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod brain_region_hierarchy;
pub mod cortical_area;

// Re-export CorticalArea types from feagi_data_structures (single source of truth)
pub use feagi_data_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalID,
};

// Re-export extension trait for business logic
pub use cortical_area::CorticalAreaExt;

// BrainRegion and RegionType now come from feagi_data_structures
pub use brain_region_hierarchy::BrainRegionHierarchy;
pub use feagi_data_structures::genomic::{BrainRegion, RegionType};
