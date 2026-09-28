// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! # FEAGI Runtime - Embedded (ESP32, RTOS, no_std)
//!
//! Platform adapter for embedded systems with no standard library.
//!
//! ## Features
//! - ✅ `no_std` compatible
//! - ✅ Fixed-size arrays (no heap allocation)
//! - ✅ Single-threaded execution
//! - ✅ Deterministic performance
//! - ✅ Fixed compile-time capacity (see [`DEFAULT_MAX_NEURONS`])
//!
//! ## Targets
//! - ESP32 (FreeRTOS or bare-metal)
//! - ARM Cortex-M (Zephyr, FreeRTOS)
//! - RISC-V embedded
//!
//! ## Architecture
//! Uses platform-agnostic core (`feagi-neural`, `feagi-synapse`) internally,
//! providing fixed-size array-based storage for predictable memory usage.
//!
//! This module is only available when the `embedded` feature is enabled.

#[cfg(feature = "std")]
extern crate std;

/// Default neuron capacity for [`EmbeddedRuntime`] (compile-time array size).
pub const DEFAULT_MAX_NEURONS: usize = 1000;

/// Default synapse capacity for [`EmbeddedRuntime`] (compile-time array size).
pub const DEFAULT_MAX_SYNAPSES: usize = 5000;

pub mod neuron_array;
pub mod runtime;
pub mod synapse_array;

pub use neuron_array::NeuronArray;
pub use runtime::EmbeddedRuntime;
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
            max_neurons: DEFAULT_MAX_NEURONS,
            max_synapses: DEFAULT_MAX_SYNAPSES,
            burst_frequency: 100, // 100 Hz = 10ms per burst
        }
    }
}
