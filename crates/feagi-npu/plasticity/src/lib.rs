// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # FEAGI Plasticity Module
//!
//! This crate implements synaptic plasticity algorithms for FEAGI:
//! - STDP (Spike-Timing-Dependent Plasticity)
//! - Memory formation with pattern detection
//! - Memory neuron lifecycle management
//! - Neuron ID allocation system
//!
//! ## Architecture
//! - High-performance Rust implementation
//! - SIMD-friendly data structures
//! - Thread-safe operations
//! - RTOS-compatible design

/// Crate version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod executor; // Abstraction layer for different execution models
pub mod memory_neuron_array;
pub mod memory_stats_cache;
pub mod neuron_id_manager;
pub mod pattern_detector;
pub mod service;
pub mod stdp;
pub mod stdp_core; // Platform-agnostic STDP (no_std compatible)
                   // pub mod lifecycle_manager;  // DEPRECATED: Use AsyncPlasticityExecutor instead

// Re-export key types
pub use executor::{AsyncPlasticityExecutor, PlasticityExecutor};
// pub use lifecycle_manager::PlasticityLifecycleManager;  // DEPRECATED
pub use memory_neuron_array::{MemoryNeuronArray, MemoryNeuronLifecycleConfig, MemoryNeuronStats};
pub use memory_stats_cache::{
    create_memory_stats_cache, get_area_stats, get_stats_snapshot, init_memory_area,
    on_neuron_created, on_neuron_deleted, remove_memory_area, MemoryAreaStats, MemoryStatsCache,
};
pub use neuron_id_manager::{AllocationStats, NeuronIdManager, NeuronType};
pub use pattern_detector::{BatchPatternDetector, PatternConfig, PatternDetector, TemporalPattern};
pub use service::{PlasticityCommand, PlasticityConfig, PlasticityService};
pub use stdp::{compute_activity_factors, compute_timing_factors, STDPConfig};
