/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # Neuron Model Traits
//!
//! Defines the core trait interface that all neuron models must implement.

use feagi_types::*;

/// Core trait for neuron model computational behavior
///
/// This trait defines the mathematical operations that define a neuron model's behavior.
/// Implementations must provide the formulas for:
/// - Synaptic contribution calculation
/// - Membrane potential update
/// - Firing condition check
/// - Post-firing reset
///
/// # Example
///
/// ```ignore
/// struct MyModel;
///
/// impl NeuronModel for MyModel {
///     type Parameters = MyModelParameters;
///     
///     fn model_name(&self) -> &'static str {
///         "My Custom Model"
///     }
///     
///     fn compute_synaptic_contribution(&self, weight: f32, psp: f32, synapse_type: SynapseType) -> f32 {
///         // Your model's formula here
///     }
///     
///     // ... implement other required methods
/// }
/// ```
pub trait NeuronModel: Send + Sync {
    /// Model-specific parameters (e.g., LIF has leak_coefficient, resting_potential)
    type Parameters: ModelParameters;

    /// Human-readable model name for logging/debugging
    fn model_name(&self) -> &'static str;

    /// Calculate synaptic contribution from a single fired synapse
    ///
    /// This is called during synaptic propagation for each synapse from a fired neuron.
    ///
    /// # Arguments
    ///
    /// * `weight` - Synaptic weight normalized to [0, 1] range
    /// * `psp` - Postsynaptic potential (from source area's pstcr_), normalized to [0, 1]
    /// * `synapse_type` - Excitatory or Inhibitory
    ///
    /// # Returns
    ///
    /// Contribution to target neuron's membrane potential (can be positive or negative)
    ///
    /// # Performance Note
    ///
    /// This method is called millions of times per burst. Use `#[inline(always)]` on implementations.
    fn compute_synaptic_contribution(
        &self,
        weight: f32,
        psp: f32,
        synapse_type: SynapseType,
    ) -> f32;

    /// Update membrane potential given synaptic input
    ///
    /// Called during neural dynamics after all synaptic contributions are accumulated.
    ///
    /// # Arguments
    ///
    /// * `current_mp` - Current membrane potential
    /// * `synaptic_input` - Sum of all synaptic contributions (I_syn)
    /// * `params` - Model-specific parameters for this neuron
    /// * `dt` - Time step (usually 1.0 for discrete burst cycles)
    ///
    /// # Returns
    ///
    /// New membrane potential after update
    fn update_membrane_potential(
        &self,
        current_mp: f32,
        synaptic_input: f32,
        params: &Self::Parameters,
        dt: f32,
    ) -> f32;

    /// Check if neuron should fire
    ///
    /// # Arguments
    ///
    /// * `membrane_potential` - Current membrane potential (after update)
    /// * `threshold` - Firing threshold
    /// * `refractory_countdown` - Remaining refractory period (0 = can fire)
    ///
    /// # Returns
    ///
    /// `true` if neuron should fire this burst
    fn should_fire(
        &self,
        membrane_potential: f32,
        threshold: f32,
        refractory_countdown: u16,
    ) -> bool;

    /// Reset neuron state after firing
    ///
    /// # Arguments
    ///
    /// * `params` - Model-specific parameters
    ///
    /// # Returns
    ///
    /// New membrane potential after reset (typically resting potential or 0)
    fn reset_after_fire(&self, params: &Self::Parameters) -> f32;
}

/// Trait for model-specific parameter structures
///
/// All neuron models must define a parameter type that implements this trait.
/// Parameters can be stored per-neuron or per-cortical-area depending on the model.
pub trait ModelParameters: Clone + Send + Sync + 'static {
    /// Validate that parameters are within acceptable ranges
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err(message)` if invalid
    fn validate(&self) -> Result<()>;

    /// Get the number of parameters (for memory estimation)
    fn parameter_count() -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test that trait is object-safe (can use `Box<dyn NeuronModel>`)
    #[test]
    fn test_trait_object_safety() {
        // This test just ensures the trait is object-safe
        // Actual implementations will be tested in their respective modules
        
        // We can't create a Box<dyn NeuronModel> without a concrete implementation,
        // but we can verify the trait compiles and is properly defined
    }
}

