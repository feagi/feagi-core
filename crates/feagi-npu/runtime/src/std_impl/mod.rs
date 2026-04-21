// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! # FEAGI Runtime - Standard (Desktop/Server)
//!
//! Platform adapter for desktop and server environments with full std library support.
//!
//! ## Features
//! - ✅ Dynamic allocation (`Vec`, `HashMap`)
//! - ✅ Parallel processing (Rayon)
//! - ✅ Unlimited neuron capacity
//! - ✅ Multi-threading
//!
//! ## Architecture
//! Uses platform-agnostic core (`feagi-neural`, `feagi-synapse`) internally,
//! providing a convenient API with standard library collections.
//!
//! This module is only available when the `std` feature is enabled.

pub mod neuron_array;
pub mod runtime;
pub mod synapse_array;

pub use neuron_array::NeuronArray;
pub use runtime::StdRuntime;
pub use synapse_array::SynapseArray;

// Re-export for backward compatibility
pub use NeuronArray as StdNeuronArray;
pub use SynapseArray as StdSynapseArray;

/// Runtime configuration for std platform
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Number of parallel threads (0 = auto-detect)
    pub num_threads: usize,

    /// Enable SIMD optimizations
    pub enable_simd: bool,

    /// Initial capacity hint for arrays
    pub initial_capacity: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            num_threads: 0, // Auto-detect
            enable_simd: true,
            initial_capacity: 1024,
        }
    }
}
