// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # FEAGI Neural Dynamics (Platform-Agnostic)
//!
//! Pure neural computation algorithms that work on any platform:
//! - Desktop (std)
//! - ESP32 (no_std)
//! - HPC clusters (std + MPI)
//! - GPU (WGPU/CUDA)
//!
//! ## Design Principles
//! - **No allocations**: All functions work on borrowed slices
//! - **No I/O**: Pure computation only
//! - **No platform dependencies**: Works with `no_std`
//! - **SIMD-friendly**: Data layouts optimized for vectorization
//!
//! ## Target Platforms
//! - ✅ Desktop (Linux, macOS, Windows)
//! - ✅ Embedded (ESP32, ARM Cortex-M)
//! - ✅ RTOS (FreeRTOS, Zephyr)
//! - ✅ WASM (browser, Node.js)

#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod dynamics;
pub mod firing;
pub mod utils;
pub mod synapse;
pub mod models;

pub use dynamics::*;
pub use firing::*;
pub use utils::*;

// Re-export synapse module for convenience
pub use synapse::{SynapseType, compute_synaptic_contribution, compute_synaptic_contributions_batch};

// Re-export neuron models for convenience
pub use models::{NeuronModel, ModelParameters, LIFModel, LIFParameters};


