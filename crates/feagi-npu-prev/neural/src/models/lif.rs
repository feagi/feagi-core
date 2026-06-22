// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # LIF (Leaky Integrate-and-Fire) Neuron Model
//!
//! The LIF model is the default and simplest neuron model in FEAGI.
//!
//! ## Model Dynamics
//!
//! ```text
//! Synaptic Contribution (per active synapse):
//!     contribution = sign × weight × psp
//!
//!     Where:
//!     - sign = +1.0 (excitatory) or -1.0 (inhibitory)
//!     - weight = synaptic weight normalized [0, 1]
//!     - psp = postsynaptic potential normalized [0, 1]
//!
//!     Result range: -1.0 to +1.0
//!
//! Membrane Potential Update:
//!     I_syn = Σ contribution for all active synapses
//!     V(t+1) = V(t) + I_syn - g_leak × (V(t) - V_rest)
//!
//!     Where:
//!     - V = membrane potential
//!     - g_leak = leak_coefficient (0-1)
//!     - V_rest = resting_potential
//!
//! Firing Check:
//!     if refractory_countdown > 0:
//!         Skip (neuron in refractory period)
//!     else if V(t+1) ≥ threshold:
//!         FIRE and reset to V_rest
//! ```

use super::traits::{ModelParameters, NeuronModel};
use crate::synapse::SynapseType;

/// LIF (Leaky Integrate-and-Fire) neuron model
///
/// This is the default model used in FEAGI. It's simple, computationally efficient,
/// and captures basic neuronal dynamics.
#[derive(Debug, Clone, Copy)]
pub struct LIFModel;

impl LIFModel {
    /// Create a new LIF model instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for LIFModel {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuronModel for LIFModel {
    type Parameters = LIFParameters;

    fn model_name(&self) -> &'static str {
        "Leaky Integrate-and-Fire (LIF)"
    }

    #[inline(always)]
    fn compute_synaptic_contribution(
        &self,
        weight: f32,
        psp: f32,
        synapse_type: SynapseType,
    ) -> f32 {
        // LIF formula: contribution = sign × weight × psp
        // Canonical range (absolute-u8 contract): -65,025.0 to +65,025.0 (255 × 255)
        let sign = match synapse_type {
            SynapseType::Excitatory => 1.0,
            SynapseType::Inhibitory => -1.0,
        };
        sign * weight * psp
    }

    #[inline(always)]
    fn update_membrane_potential(
        &self,
        current_mp: f32,
        synaptic_input: f32,
        params: &LIFParameters,
        _dt: f32,
    ) -> f32 {
        // LIF membrane potential update:
        // V(t+1) = V(t) + I_syn - g_leak × (V(t) - V_rest)
        current_mp + synaptic_input
            - params.leak_coefficient * (current_mp - params.resting_potential)
    }

    #[inline(always)]
    fn should_fire(
        &self,
        membrane_potential: f32,
        threshold: f32,
        refractory_countdown: u16,
    ) -> bool {
        // LIF firing condition: V ≥ threshold AND not in refractory period
        refractory_countdown == 0 && membrane_potential >= threshold
    }

    #[inline(always)]
    fn reset_after_fire(&self, params: &LIFParameters) -> f32 {
        // LIF reset: Return to resting potential after firing
        params.resting_potential
    }
}

/// LIF model-specific parameters
#[derive(Debug, Clone, Copy)]
pub struct LIFParameters {
    /// Leak coefficient (0.0-1.0): percentage of (V - V_rest) lost per burst
    pub leak_coefficient: f32,

    /// Resting potential: baseline membrane potential when no input
    pub resting_potential: f32,
}

impl LIFParameters {
    /// Create new LIF parameters with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create LIF parameters with custom values
    pub fn with_values(leak_coefficient: f32, resting_potential: f32) -> Self {
        Self {
            leak_coefficient,
            resting_potential,
        }
    }
}

impl Default for LIFParameters {
    fn default() -> Self {
        Self {
            leak_coefficient: 0.1,  // 10% leak per burst
            resting_potential: 0.0, // Zero baseline
        }
    }
}

impl ModelParameters for LIFParameters {
    fn validate(&self) -> Result<(), &'static str> {
        if self.leak_coefficient < 0.0 || self.leak_coefficient > 1.0 {
            return Err("LIF: Leak coefficient must be in [0, 1]");
        }
        Ok(())
    }

    fn parameter_count() -> usize {
        2 // leak_coefficient + resting_potential
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lif_synaptic_contribution() {
        let model = LIFModel::new();

        // Test excitatory synapse
        let contrib_exc = model.compute_synaptic_contribution(
            1.0, // weight
            0.5, // psp
            SynapseType::Excitatory,
        );
        assert_eq!(contrib_exc, 0.5);

        // Test inhibitory synapse
        let contrib_inh = model.compute_synaptic_contribution(
            1.0, // weight
            0.5, // psp
            SynapseType::Inhibitory,
        );
        assert_eq!(contrib_inh, -0.5);
    }

    #[test]
    fn test_lif_membrane_potential_update() {
        let model = LIFModel::new();
        let params = LIFParameters {
            leak_coefficient: 0.1,
            resting_potential: 0.0,
        };

        // Test with positive synaptic input
        let new_mp = model.update_membrane_potential(
            0.5, // current MP
            0.3, // synaptic input
            &params, 1.0, // dt
        );
        // Expected: 0.5 + 0.3 - 0.1 * (0.5 - 0.0) = 0.75
        assert!((new_mp - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_lif_firing_condition() {
        let model = LIFModel::new();

        // Should fire
        assert!(model.should_fire(1.5, 1.0, 0));

        // Should NOT fire: below threshold
        assert!(!model.should_fire(0.5, 1.0, 0));

        // Should NOT fire: refractory
        assert!(!model.should_fire(1.5, 1.0, 5));
    }

    #[test]
    fn test_lif_parameters_validation() {
        let params_valid = LIFParameters {
            leak_coefficient: 0.5,
            resting_potential: 0.0,
        };
        assert!(params_valid.validate().is_ok());

        let params_invalid = LIFParameters {
            leak_coefficient: 1.5,
            resting_potential: 0.0,
        };
        assert!(params_invalid.validate().is_err());
    }
}
