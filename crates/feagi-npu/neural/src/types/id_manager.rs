// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Neuron ID management for multi-model architecture
//!
//! Moved from feagi-types/src/id_manager/ (Phase 2c)
//! Note: This is a simplified placeholder. Full implementation will be copied later if needed.

/// Neuron array type identifier for multi-model support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeuronArrayType {
    Standard,
    Memory,
    Power,
}

