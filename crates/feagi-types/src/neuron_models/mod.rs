/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # Neuron Models Module
//!
//! This module contains model-specific neuron array implementations.
//! Currently only LIF (Leaky Integrate-and-Fire) is implemented.
//!
//! ## Architecture
//! - Each neuron model has its own dedicated array structure
//! - Zero memory waste: only store parameters that model uses
//! - Future: Izhikevich, AdEx, Hodgkin-Huxley models
//!
//! ## Current Status
//! - Phase 0: Structure established, single model (LIF)
//! - See: `feagi-core/docs/MULTI_MODEL_NEURON_ARCHITECTURE.md`

pub mod lif;
pub mod traits;

pub use lif::LIFNeuronArray;
pub use traits::NeuronModel;

