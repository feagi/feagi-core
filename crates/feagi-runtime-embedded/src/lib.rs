/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # FEAGI Runtime - Embedded (ESP32, RTOS, no_std)
//!
//! Platform adapter for embedded systems with no standard library.
//!
//! ## Features
//! - ✅ `no_std` compatible
//! - ✅ Fixed-size arrays (no heap allocation)
//! - ✅ Single-threaded execution
//! - ✅ Deterministic performance
//! - ✅ < 1 KB stack usage
//!
//! ## Targets
//! - ESP32 (FreeRTOS or bare-metal)
//! - ARM Cortex-M (Zephyr, FreeRTOS)
//! - RISC-V embedded
//!
//! ## Architecture
//! Uses platform-agnostic core (`feagi-neural`, `feagi-synapse`) internally,
//! providing fixed-size array-based storage for predictable memory usage.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod neuron_array;
pub mod synapse_array;

pub use neuron_array::NeuronArray;
pub use synapse_array::SynapseArray;

/// Runtime configuration for embedded platforms
#[derive(Debug, Clone, Copy)]
pub struct RuntimeConfig {
    /// Maximum neurons supported
    pub max_neurons: usize,
    
    /// Maximum synapses supported
    pub max_synapses: usize,
    
    /// Burst frequency (Hz)
    pub burst_frequency: u32,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_neurons: 1000,
            max_synapses: 5000,
            burst_frequency: 100, // 100 Hz = 10ms per burst
        }
    }
}


