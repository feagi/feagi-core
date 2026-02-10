// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Manual `n % divisor == 0` used instead of `n.is_multiple_of(divisor)` for stable Rust
// compatibility (is_multiple_of is unstable on older stable toolchains).
#![allow(clippy::manual_is_multiple_of)]
/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! # FEAGI Burst Engine
//!
//! High-performance neural burst processing engine.
//!
//! ## Performance Targets
//! - **30Hz burst frequency** with 1.2M neuron firings
//! - **50-100x faster** than Python implementation
//! - **SIMD-optimized** for modern CPUs
//! - **Cache-friendly** data structures
//!
//! ## Architecture
//! - Pure Rust, no Python overhead
//! - Rayon for multi-threading
//! - Zero-copy data access where possible
//! - Minimal allocations in hot paths

/// Crate version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ---------------------------------------------------------------------------------------------
// Simulation timestep (burst interval) telemetry support
//
// NOTE:
// - FEAGI is used in both real-time and batch workloads; we must not hardcode 15Hz/30Hz timing
//   assumptions into warning thresholds.
// - The burst loop owns the authoritative runtime frequency (Hz). We snapshot the derived
//   timestep (ns) into a single atomic so hot-path code (e.g., injection timing) can compare
//   durations against the current simulation timestep without plumbing frequency everywhere.
// ---------------------------------------------------------------------------------------------
use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;

/// Current simulation timestep (burst interval) in nanoseconds.
///
/// Updated by the burst loop thread once per iteration (or when frequency changes).
pub(crate) static SIM_TIMESTEP_NS: AtomicU64 = AtomicU64::new(0);

/// Update the global simulation timestep snapshot from the configured runtime frequency (Hz).
///
/// # Panics
/// Panics if `frequency_hz` is not positive. The burst loop uses this value for scheduling and
/// would also fail if invalid.
pub(crate) fn update_sim_timestep_from_hz(frequency_hz: f64) {
    assert!(
        frequency_hz.is_finite() && frequency_hz > 0.0,
        "frequency_hz must be finite and > 0"
    );
    let timestep_ns = (1_000_000_000.0 / frequency_hz) as u64;
    SIM_TIMESTEP_NS.store(timestep_ns, Ordering::Relaxed);
}

/// Get the current simulation timestep snapshot.
///
/// This is intended for logging thresholds (e.g., warn when injection exceeds timestep).
pub(crate) fn sim_timestep() -> Duration {
    let timestep_ns = SIM_TIMESTEP_NS.load(Ordering::Relaxed);
    // If this is called before the burst loop initializes the value, return 0ns (no threshold).
    // In normal operation, burst loop initializes this before the first burst.
    Duration::from_nanos(timestep_ns)
}

#[cfg(any(feature = "async-tokio", feature = "wasm"))]
pub mod async_burst_loop; // Pure Rust burst loop
pub mod backend;
#[cfg(feature = "std")]
pub mod burst_loop_runner;
pub use burst_loop_runner::EmbodimentSensoryPoller;
pub mod fire_ledger;
pub mod fire_structures;
pub mod fq_sampler;
pub mod motor_shm_writer;
pub mod neural_dynamics;
#[cfg(feature = "std")]
pub mod tracing_mutex;
// Neuron models moved to feagi-neural::models (Phase 2b)
pub mod dynamic_npu;
pub mod npu;
pub mod parameter_update_queue;
pub mod sensory; // Rust sensory injection system
                 // Disabled - uses DynamicNPU
                 // pub mod sleep; // Sleep manager for energy efficiency and memory optimization
pub mod synaptic_propagation;
pub mod viz_shm_writer; // Rust visualization SHM writer // Rust motor SHM writer

pub use backend::*;
#[cfg(feature = "std")]
pub use burst_loop_runner::*;
#[cfg(feature = "std")]
pub use dynamic_npu::DynamicNPU;
/// Conditional NPU mutex: TracingMutex if feature enabled, else wrapper around std::sync::Mutex
/// This allows zero-overhead when lock tracing is disabled
#[cfg(feature = "std")]
pub use tracing_mutex::TracingMutex;

pub use dynamic_npu::DynamicNPUGeneric;
pub use fire_ledger::*;
pub use fire_structures::*;
pub use fq_sampler::*;
pub use neural_dynamics::*;
// Neuron models now in feagi-neural::models
pub use npu::*;
pub use parameter_update_queue::{ParameterUpdate, ParameterUpdateQueue};
pub use sensory::*;
// pub use sleep::*;
pub use synaptic_propagation::*;
pub use viz_shm_writer::*;

/// Burst engine performance statistics
#[derive(Debug, Clone, Default)]
pub struct BurstEngineStats {
    pub total_bursts: u64,
    pub total_neurons_fired: u64,
    pub total_synapses_processed: u64,
    pub total_processing_time_us: u64,
}

impl BurstEngineStats {
    /// Get average neurons per burst
    pub fn avg_neurons_per_burst(&self) -> f64 {
        if self.total_bursts == 0 {
            0.0
        } else {
            self.total_neurons_fired as f64 / self.total_bursts as f64
        }
    }

    /// Get average processing time per burst (microseconds)
    pub fn avg_processing_time_us(&self) -> f64 {
        if self.total_bursts == 0 {
            0.0
        } else {
            self.total_processing_time_us as f64 / self.total_bursts as f64
        }
    }

    /// Get average synapses per neuron
    pub fn avg_synapses_per_neuron(&self) -> f64 {
        if self.total_neurons_fired == 0 {
            0.0
        } else {
            self.total_synapses_processed as f64 / self.total_neurons_fired as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_burst_stats() {
        let stats = BurstEngineStats {
            total_bursts: 100,
            total_neurons_fired: 10000,
            total_synapses_processed: 50000,
            total_processing_time_us: 1000000,
        };

        assert_eq!(stats.avg_neurons_per_burst(), 100.0);
        assert_eq!(stats.avg_processing_time_us(), 10000.0);
        assert_eq!(stats.avg_synapses_per_neuron(), 5.0);
    }
}
