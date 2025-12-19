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

pub mod memory_neuron_array;
pub mod neuron_id_manager;
pub mod pattern_detector;
pub mod service;
pub mod stdp;
pub mod stdp_core; // Platform-agnostic STDP (no_std compatible)

// Re-export key types
pub use memory_neuron_array::{MemoryNeuronArray, MemoryNeuronLifecycleConfig, MemoryNeuronStats};
pub use neuron_id_manager::{AllocationStats, NeuronIdManager, NeuronType};
pub use pattern_detector::{BatchPatternDetector, PatternConfig, PatternDetector, TemporalPattern};
pub use service::{PlasticityCommand, PlasticityConfig, PlasticityService};
pub use stdp::{compute_activity_factors, compute_timing_factors, STDPConfig};
