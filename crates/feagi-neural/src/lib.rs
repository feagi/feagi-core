// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # FEAGI Neural Computation (Platform-Agnostic)
//!
//! ALL neural computation in one place:
//! - **Types**: Core type definitions (NeuronId, SynapseType, NeuralValue, etc.)
//! - **Synapse**: Synaptic contribution algorithms
//! - **Dynamics**: Membrane potential updates
//! - **Models**: Neuron models (LIF, Izhikevich, etc.)
//!
//! Merged from:
//! - feagi-types (Phase 2c)
//! - feagi-synapse (Phase 2a)
//! - feagi-burst-engine/neuron_models (Phase 2b)
//!
//! ## Target Platforms
//! - ✅ Desktop (Linux, macOS, Windows)
//! - ✅ Embedded (ESP32, ARM Cortex-M)
//! - ✅ RTOS (FreeRTOS, Zephyr)
//! - ✅ WASM (browser, Node.js)
//! - ✅ GPU (CUDA, WebGPU)

#![no_std]

#[cfg(feature = "std")]
extern crate std;

// Core type definitions (merged from feagi-types)
pub mod types;

// Neural dynamics algorithms
pub mod dynamics;
pub mod firing;
pub mod utils;

// Synaptic algorithms (merged from feagi-synapse)
pub mod synapse;

// Neuron models (moved from feagi-burst-engine)
pub mod models;

// Re-export everything for convenience
pub use dynamics::*;
pub use firing::*;
pub use utils::*;

// Re-export types
pub use types::{
    NeuronId, SynapseId,
    NeuralValue, INT8Value, INT8LeakCoefficient, Precision, QuantizationSpec,
    Synapse, SynapticWeight, SynapticConductance, SynapticContribution,
    Position,
    // Dimensions moved to feagi_data_structures::genomic::cortical_area::CorticalAreaDimensions
    FeagiError, Result, Error,
    FireCandidateList, FireQueue, FireLedger,
    // CorticalArea, BrainRegion, RegionType, BrainRegionHierarchy moved to feagi_data_structures
};

// Re-export synapse module
pub use synapse::{SynapseType, compute_synaptic_contribution, compute_synaptic_contributions_batch};

// Re-export neuron models
pub use models::{NeuronModel, ModelParameters, LIFModel, LIFParameters};


