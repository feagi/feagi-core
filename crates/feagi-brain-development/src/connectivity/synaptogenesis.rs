// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
NPU-Native Synaptogenesis - Zero-Copy Morphology Application

This module re-exports core morphology implementations from the core_morphologies module
for backward compatibility. All core morphologies have been refactored into separate
modules in the core_morphologies directory.

## Architecture

```text
Python: Call rust_apply_projector(npu, src_area_id, dst_area_id, params)
           ↓
Rust:   1. Query neurons from NPU (zero copy)
        2. Apply morphology rules (SIMD optimized)
        3. Create synapses directly in NPU
        4. Return synapse count
           ↓
Python: Receives u32 (synapse count only)
```

## Performance Impact

- **Eliminates:** 6+ seconds of FFI overhead per area pair
- **Eliminates:** Python list building and marshaling
- **Enables:** SIMD-optimized morphology application
- **Result:** ~50+ second improvement for typical genomes

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

// Re-export all core morphology functions from the refactored modules
pub use crate::connectivity::core_morphologies::{
    apply_block_connection_morphology, apply_block_connection_morphology_batched,
    apply_expander_morphology, apply_patterns_morphology, apply_projector_morphology,
    apply_vectors_morphology, apply_vectors_morphology_with_dimensions,
};
