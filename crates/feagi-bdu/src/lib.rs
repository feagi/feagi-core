/*!
# FEAGI BDU (Brain Development Utilities)

This crate implements high-performance brain development operations including:
- Synaptogenesis (synapse creation based on morphology rules)
- Connectivity rules (projection, topology, patterns)
- Spatial hashing and coordinate transformations

## Architecture

Mirrors the Python structure:
- `feagi/bdu/connectivity/` → `feagi_bdu::connectivity`
- `feagi/bdu/morton_spatial_hash.py` → `feagi_bdu::spatial`

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

pub mod connectivity;
pub mod connectome_manager;
// pub mod ffi;  // DEPRECATED: Legacy Python-driven synaptogenesis (replaced by NPU-native functions)
pub mod models;
pub mod spatial;
pub mod types;

// Re-export NPU-native synaptogenesis functions (primary API)
pub use connectivity::{
    apply_block_connection_morphology, apply_expander_morphology, apply_patterns_morphology,
    apply_projector_morphology, apply_vectors_morphology,
};

// Re-export legacy types for backward compatibility (will be removed)
pub use connectivity::{
    find_candidate_neurons, CandidateNeuron, MorphologyParams, SynaptogenesisRequest,
    SynaptogenesisResult,
};

pub use spatial::{morton_decode_3d, morton_encode_3d, MortonSpatialHash, SpatialHashStats};

pub use types::{AreaId, BduError, BduResult, Dimensions, NeuronId, Position, Weight};

pub use models::{BrainRegion, BrainRegionHierarchy, CorticalArea};

pub use connectome_manager::{ConnectomeConfig, ConnectomeManager};

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
