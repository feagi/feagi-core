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
//! ## Model Dynamics (Standardized Formula)
//! 
//! **CRITICAL**: This formula is implemented in:
//! - CPU backend: Uses this trait implementation
//! - GPU backend: Hardcoded in `shaders/synaptic_propagation_lif.wgsl`
//! 
//! ```text
//! Synaptic Contribution (per active synapse):
//!     contribution = sign × weight × psp
//!     
//!     Where:
//!     - sign = +1.0 (excitatory) or -1.0 (inhibitory)
//!     - weight = synaptic weight normalized [0, 1] (stored as u8 0-255)
//!     - psp = postsynaptic potential normalized [0, 1] (from source area's pstcr_)
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
//! 
//! **Example**: Source pstcr_=127 (0.5), weight=255 (1.0) → contribution=0.5
//!
//! ## GPU Shader Location
//!
//! The GPU implementation of this model is in:
//! - `backend/shaders/synaptic_propagation_lif.wgsl`
//! - `backend/shaders/neural_dynamics_lif.wgsl` (or `neural_dynamics_fcl.wgsl`)
//!
//! **IMPORTANT**: When modifying this model's formulas, update BOTH the trait implementation
//! and the GPU shaders to keep them synchronized.

use super::traits::{NeuronModel, ModelParameters};
use feagi_types::*;

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
        // Result range: -1.0 to +1.0
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
        //
        // Components:
        // - I_syn: Total synaptic input (sum of all contributions)
        // - g_leak: Leak coefficient (percentage of potential lost per burst)
        // - V_rest: Resting potential (baseline membrane potential)
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
///
/// These parameters define the behavior of a LIF neuron.
/// They can be stored per-neuron or per-cortical-area depending on requirements.
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
            leak_coefficient: 0.1,    // 10% leak per burst
            resting_potential: 0.0,   // Zero baseline
        }
    }
}

impl ModelParameters for LIFParameters {
    fn validate(&self) -> Result<()> {
        if self.leak_coefficient < 0.0 || self.leak_coefficient > 1.0 {
            return Err(Error::ComputationError(format!(
                "LIF: Leak coefficient must be in [0, 1], got {}",
                self.leak_coefficient
            )));
        }

        // Resting potential can be any value, no validation needed

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

        // Test with different weight
        let contrib_partial = model.compute_synaptic_contribution(
            0.8, // weight
            0.5, // psp
            SynapseType::Excitatory,
        );
        assert_eq!(contrib_partial, 0.4);
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
            0.5,  // current MP
            0.3,  // synaptic input
            &params,
            1.0,  // dt
        );
        // Expected: 0.5 + 0.3 - 0.1 * (0.5 - 0.0) = 0.5 + 0.3 - 0.05 = 0.75
        assert!((new_mp - 0.75).abs() < 1e-6);

        // Test with no input (pure leak)
        let new_mp = model.update_membrane_potential(
            1.0,  // current MP
            0.0,  // no input
            &params,
            1.0,  // dt
        );
        // Expected: 1.0 + 0.0 - 0.1 * (1.0 - 0.0) = 0.9
        assert!((new_mp - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_lif_firing_condition() {
        let model = LIFModel::new();

        // Should fire: MP above threshold, not refractory
        assert!(model.should_fire(1.5, 1.0, 0));

        // Should NOT fire: MP below threshold
        assert!(!model.should_fire(0.5, 1.0, 0));

        // Should NOT fire: In refractory period
        assert!(!model.should_fire(1.5, 1.0, 5));
    }

    #[test]
    fn test_lif_reset() {
        let model = LIFModel::new();
        let params = LIFParameters {
            leak_coefficient: 0.1,
            resting_potential: -0.5,
        };

        let reset_mp = model.reset_after_fire(&params);
        assert_eq!(reset_mp, -0.5);
    }

    #[test]
    fn test_lif_parameters_validation() {
        // Valid parameters
        let params_valid = LIFParameters {
            leak_coefficient: 0.5,
            resting_potential: 0.0,
        };
        assert!(params_valid.validate().is_ok());

        // Invalid: leak too high
        let params_invalid_high = LIFParameters {
            leak_coefficient: 1.5,
            resting_potential: 0.0,
        };
        assert!(params_invalid_high.validate().is_err());

        // Invalid: leak negative
        let params_invalid_neg = LIFParameters {
            leak_coefficient: -0.1,
            resting_potential: 0.0,
        };
        assert!(params_invalid_neg.validate().is_err());
    }

    #[test]
    fn test_lif_model_name() {
        let model = LIFModel::new();
        assert_eq!(model.model_name(), "Leaky Integrate-and-Fire (LIF)");
    }

    #[test]
    fn test_lif_default_parameters() {
        let params = LIFParameters::default();
        assert_eq!(params.leak_coefficient, 0.1);
        assert_eq!(params.resting_potential, 0.0);
        assert_eq!(LIFParameters::parameter_count(), 2);
    }
}

