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
//! ## Architecture
//!
//! - **CPU Backend**: Uses trait methods directly (dynamic dispatch)
//! - **GPU Backend**: Uses model-specific shader files (static compilation)
//!
//! This hybrid approach gives us:
//! - Single source of truth for formulas (in trait implementations)
//! - Optimal CPU performance (inlined trait methods)
//! - Optimal GPU performance (compiled shaders)
//!
//! ## Adding a New Neuron Model
//!
//! 1. Create `src/neuron_models/your_model.rs`
//! 2. Implement `NeuronModel` trait
//! 3. Create GPU shaders in `src/backend/shaders/synaptic_propagation_your_model.wgsl`
//! 4. Update `CPUBackend::new_your_model()` and `WGPUBackend::new_your_model()`

pub mod traits;
pub mod lif;

// Re-export core types
pub use traits::{NeuronModel, ModelParameters};
pub use lif::{LIFModel, LIFParameters};

