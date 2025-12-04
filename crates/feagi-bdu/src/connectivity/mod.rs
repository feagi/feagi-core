// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Connectivity and synaptogenesis operations.

This module implements high-performance synapse creation based on morphology rules.
*/

pub mod rules;
pub mod synaptogenesis; // NPU-native synaptogenesis (zero-copy)

// Export NPU-native synaptogenesis functions
pub use synaptogenesis::{
    apply_block_connection_morphology, apply_expander_morphology, apply_patterns_morphology,
    apply_projector_morphology, apply_vectors_morphology,
};

pub use rules::{syn_projector, ProjectorParams};
