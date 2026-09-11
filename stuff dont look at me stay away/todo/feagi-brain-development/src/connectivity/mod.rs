// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Connectivity and synaptogenesis operations.

This module implements high-performance synapse creation based on morphology rules.
*/

pub mod core_morphologies;
pub mod rules;
pub mod synaptogenesis; // NPU-native synaptogenesis (zero-copy) - re-exports from core_morphologies

// Export NPU-native synaptogenesis functions (re-exported from synaptogenesis module for backward compatibility)
pub use synaptogenesis::{
    apply_block_connection_morphology, apply_block_connection_morphology_batched,
    apply_expander_morphology, apply_patterns_morphology, apply_projector_morphology,
    apply_vectors_morphology,
};

pub use rules::{syn_projector, ProjectorParams};
