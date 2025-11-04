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

pub mod cortical_area;
pub mod brain_region;
pub mod brain_region_hierarchy;

// Re-export core types
pub use cortical_area::CorticalArea;
pub use brain_region::BrainRegion;
pub use brain_region_hierarchy::BrainRegionHierarchy;





