// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # Neuron Model Architecture
//!
//! This module defines the trait-based neuron model system that allows FEAGI
//! to support multiple neuron types (LIF, Izhikevich, AdEx, etc.).
//!
//! ## Adding a New Neuron Model
//!
//! 1. Create `src/models/your_model.rs`
//! 2. Implement `NeuronModel` trait
//! 3. Add tests
//! 4. Export in `mod.rs`

pub mod traits;
pub mod lif;

// Re-export core types
pub use traits::{NeuronModel, ModelParameters};
pub use lif::{LIFModel, LIFParameters};

