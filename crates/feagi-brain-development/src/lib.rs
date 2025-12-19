// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
# FEAGI BDU (Brain Development Utilities)

This crate implements high-performance brain development operations including:
- Synaptogenesis (synapse creation based on morphology rules)
- Connectivity rules (projection, topology, patterns)
- Spatial hashing and coordinate transformations

## Architecture

Mirrors the Python structure:
- `feagi/bdu/connectivity/` → `feagi_brain_development::connectivity`
- `feagi/bdu/morton_spatial_hash.py` → `feagi_brain_development::spatial`

## Performance Goals

- 40x-100x faster than Python implementation
- Sub-second projection mappings for 128×128×20 areas
- SIMD-optimized coordinate transformations
- Parallel processing for large mappings

## Python Integration

NPU-native synaptogenesis functions are exposed via PyO3 bindings in `feagi-rust-py-libs`.
Python code calls these functions directly with area IDs - no FFI overhead.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

/// Crate version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod connectivity;
pub mod connectome_manager;
pub mod cortical_type_utils;
pub mod neuroembryogenesis;
mod rng;
pub mod spatial;
pub mod types;

// Note: models/ and genome/ have been moved to feagi-types and feagi-evo respectively

// Re-export NPU-native synaptogenesis functions (primary API)
pub use connectivity::{
    apply_block_connection_morphology, apply_expander_morphology, apply_patterns_morphology,
    apply_projector_morphology, apply_vectors_morphology,
};

pub use spatial::{morton_decode_3d, morton_encode_3d, MortonSpatialHash, SpatialHashStats};

// Re-export local BDU types
pub use types::{AreaId, BduError, BduResult, Weight};

// Re-export core types from feagi_data_structures (single source of truth)
pub use feagi_data_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions as Dimensions, CorticalID,
};
pub use feagi_data_structures::genomic::{BrainRegion, RegionType};
pub mod models;
pub use models::{BrainRegionHierarchy, CorticalAreaExt};

// Re-export Position from local types
pub use types::Position;

// Re-export genome operations from feagi-evo
pub use feagi_evolutionary::{GenomeParser, GenomeSaver, ParsedGenome};

// Re-export connectome manager
pub use connectome_manager::{ConnectomeConfig, ConnectomeManager};

// Re-export neuroembryogenesis
pub use neuroembryogenesis::{DevelopmentProgress, DevelopmentStage, Neuroembryogenesis};

// Re-export cortical type utilities (Phase 3)
pub use cortical_type_utils::{
    describe_cortical_type, get_io_data_type, uses_absolute_frames, uses_cartesian_encoding,
    uses_incremental_frames, uses_percentage_encoding, validate_connectivity,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_projection() {
        // Smoke test to ensure modules compile
        let result = connectivity::rules::syn_projector(
            "src_area",
            "dst_area",
            42,
            (128, 128, 3),
            (128, 128, 1),
            (0, 0, 0),
            None,
            None,
        );
        assert!(result.is_ok());
    }
}
