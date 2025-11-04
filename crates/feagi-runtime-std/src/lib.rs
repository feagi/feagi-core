/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

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

pub mod neuron_array;
pub mod synapse_array;

pub use neuron_array::NeuronArray;
pub use synapse_array::SynapseArray;

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


