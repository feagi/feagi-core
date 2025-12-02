// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Core data models for FEAGI brain architecture.

This module contains the fundamental data structures that represent the
brain's architecture, including cortical areas, brain regions, and their
hierarchical organization.

These models are shared between:
- `feagi-evo`: For genome I/O and evolution
- `feagi-bdu`: For connectome management and synaptogenesis

## Design Principles

1. **Type Safety**: Use Rust's strong typing to prevent invalid states
2. **Immutability**: Core properties are immutable; updates create new instances
3. **Validation**: All constructors validate invariants
4. **Performance**: Optimized for hot-path queries
5. **Serializability**: All types implement Serialize/Deserialize

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod cortical_area;
pub mod brain_region;
pub mod brain_region_hierarchy;

// Re-export core types
pub use cortical_area::CorticalArea;
pub use brain_region::{BrainRegion, RegionType};
pub use brain_region_hierarchy::BrainRegionHierarchy;





