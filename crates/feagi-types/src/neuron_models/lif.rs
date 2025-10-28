/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # LIF (Leaky Integrate-and-Fire) Neuron Model
//!
//! ## Current Implementation
//! The LIF model is currently implemented in `../npu.rs` as the base `NeuronArray`.
//! This module provides a type alias for future migration to model-specific arrays.
//!
//! ## LIF Model Dynamics
//! ```text
//! Membrane Potential Update:
//!     V(t+1) = V(t) + I_syn - g_leak * (V(t) - V_rest)
//!
//! Where:
//!     I_syn = Σ (weight × psp) for all active synapses
//!     g_leak = leak_coefficient (0-1)
//!     V_rest = resting_potential
//! ```
//!
//! ## Future Migration
//! When implementing full multi-model architecture:
//! 1. Create dedicated `LIFNeuronArray` struct (similar to MemoryNeuronArray)
//! 2. Migrate LIF-specific parameters from base NeuronArray
//! 3. Update NPU to route to model-specific arrays
//!
//! See: `feagi-core/docs/MULTI_MODEL_NEURON_ARCHITECTURE.md` Section 3.1

use crate::npu::NeuronArray;

/// LIF Neuron Array (currently aliased to base NeuronArray)
///
/// **Phase 0**: This is a type alias for backward compatibility.
/// **Future**: Will become a dedicated struct with LIF-specific parameters.
pub type LIFNeuronArray = NeuronArray;

/// LIF Model-Specific Parameters (Future)
///
/// These parameters will be extracted from NeuronArray when implementing
/// dedicated model-specific arrays.
///
/// ```ignore
/// pub struct LIFNeuronArray {
///     // Common properties
///     pub membrane_potentials: Vec<f32>,
///     pub thresholds: Vec<f32>,
///     pub refractory_periods: Vec<u16>,
///     pub refractory_countdowns: Vec<u16>,
///     pub cortical_areas: Vec<u32>,
///     pub coordinates: Vec<u32>,
///     pub valid_mask: Vec<bool>,
///     
///     // LIF-specific parameters
///     pub leak_coefficients: Vec<f32>,
///     pub resting_potentials: Vec<f32>,
///     pub mp_charge_accumulation: Vec<bool>,
///     pub consecutive_fire_limits: Vec<u16>,
///     pub snooze_periods: Vec<u16>,
///     
///     // Mapping
///     pub global_neuron_ids: Vec<u32>,
/// }
/// ```
#[allow(dead_code)]
pub struct LIFModelParameters {
    /// Leak coefficient (0.0-1.0): percentage of potential lost per burst
    pub leak_coefficient: f32,
    /// Resting potential: baseline membrane potential
    pub resting_potential: f32,
    /// Membrane potential charge accumulation: persist input across bursts
    pub mp_charge_accumulation: bool,
    /// Consecutive fire limit: max fires before extended refractory
    pub consecutive_fire_limit: u16,
    /// Snooze period: additional refractory after consecutive fire limit
    pub snooze_period: u16,
}

impl Default for LIFModelParameters {
    fn default() -> Self {
        Self {
            leak_coefficient: 0.1,
            resting_potential: 0.0,
            mp_charge_accumulation: true,
            consecutive_fire_limit: 10,
            snooze_period: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lif_parameters_default() {
        let params = LIFModelParameters::default();
        assert_eq!(params.leak_coefficient, 0.1);
        assert_eq!(params.resting_potential, 0.0);
        assert!(params.mp_charge_accumulation);
        assert_eq!(params.consecutive_fire_limit, 10);
        assert_eq!(params.snooze_period, 5);
    }

    #[test]
    fn test_lif_array_alias() {
        // Ensure type alias works
        let array = LIFNeuronArray::new(100);
        assert_eq!(array.capacity, 100);
    }
}

