/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # Neuron Model Trait (Future)
//!
//! This module defines the trait interface for neuron models.
//! Currently a placeholder for future multi-model architecture.
//!
//! ## Design Goals
//! - Model-agnostic NPU interface
//! - Compile-time type safety
//! - Zero-cost abstractions
//!
//! ## Future Implementation
//! When implementing multi-model support:
//! 1. Define `NeuronModel` trait with dynamics methods
//! 2. Implement for LIF, Izhikevich, AdEx, etc.
//! 3. Use trait objects or generics in NPU
//!
//! See: `feagi-core/docs/MULTI_MODEL_NEURON_ARCHITECTURE.md` Section 3

/// Neuron Model Trait (Placeholder)
///
/// **Phase 0**: This is a marker trait for future expansion.
/// **Future**: Will define interface for neuron dynamics.
///
/// ```ignore
/// pub trait NeuronModel: Send + Sync {
///     /// Model name (e.g., "LIF", "Izhikevich")
///     fn model_name(&self) -> &'static str;
///     
///     /// Process neural dynamics for a single neuron
///     fn process_dynamics(
///         &self,
///         neuron_idx: usize,
///         synaptic_input: f32,
///         burst_count: u64,
///     ) -> Option<u32>; // Returns global_id if fired
///     
///     /// Update membrane potential (synaptic input)
///     fn apply_synaptic_input(&mut self, neuron_idx: usize, contribution: f32);
///     
///     /// Apply leak/decay
///     fn apply_leak(&mut self, neuron_idx: usize);
///     
///     /// Check firing threshold
///     fn check_threshold(&self, neuron_idx: usize) -> bool;
///     
///     /// Handle firing event
///     fn fire(&mut self, neuron_idx: usize);
/// }
/// ```
pub trait NeuronModel: Send + Sync {
    /// Get model name
    fn model_name(&self) -> &'static str;
}

/// Placeholder implementation for current NeuronArray
impl NeuronModel for crate::npu::NeuronArray {
    fn model_name(&self) -> &'static str {
        "LIF"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_model_trait() {
        let array = crate::npu::NeuronArray::new(100);
        assert_eq!(array.model_name(), "LIF");
    }
}

