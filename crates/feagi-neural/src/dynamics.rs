/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Neural dynamics algorithms (LIF, Izhikevich, AdEx)
//!
//! Pure functions for computing membrane potential updates.
//! Platform-agnostic, works with `no_std`.

/// Update a single LIF (Leaky Integrate-and-Fire) neuron
///
/// This is the core neural dynamics algorithm extracted from the burst engine.
/// It is pure, deterministic, and platform-agnostic.
///
/// # Arguments
/// * `membrane_potential` - Current membrane potential (mutable)
/// * `threshold` - Firing threshold
/// * `leak_coefficient` - Leak rate (0.0 to 1.0, percentage lost per step)
/// * `resting_potential` - Resting potential (not used in current implementation)
/// * `candidate_potential` - Input current from synapses
///
/// # Returns
/// `true` if neuron fired, `false` otherwise
///
/// # Algorithm
/// 1. Add candidate potential (input)
/// 2. Check if above threshold
/// 3. If fired: reset to 0.0
/// 4. If not fired: apply leak
///
/// # Example
/// ```
/// use feagi_neural::update_neuron_lif;
///
/// let mut potential = 0.5;
/// let threshold = 1.0;
/// let leak = 0.1;
/// let input = 0.6;
///
/// let fired = update_neuron_lif(&mut potential, threshold, leak, 0.0, input);
/// assert!(fired); // 0.5 + 0.6 = 1.1 > 1.0
/// assert_eq!(potential, 0.0); // Reset after firing
/// ```
#[inline]
pub fn update_neuron_lif(
    membrane_potential: &mut f32,
    threshold: f32,
    leak_coefficient: f32,
    _resting_potential: f32, // Not used in current LIF implementation
    candidate_potential: f32,
) -> bool {
    // Add input
    *membrane_potential += candidate_potential;

    // Check threshold
    if *membrane_potential >= threshold {
        // Fire and reset
        *membrane_potential = 0.0;
        return true;
    }

    // Apply leak (lose percentage of potential)
    if leak_coefficient > 0.0 {
        *membrane_potential *= 1.0 - leak_coefficient;
    }

    false
}

/// Apply leak decay to membrane potential
///
/// Implements exponential decay toward resting potential.
///
/// # Formula
/// `V_new = V_current * (1.0 - leak_coefficient)`
///
/// # Arguments
/// * `membrane_potential` - Current membrane potential (mutable)
/// * `leak_coefficient` - Leak rate (0.0 = no leak, 1.0 = instant reset)
#[inline]
pub fn apply_leak(membrane_potential: &mut f32, leak_coefficient: f32) {
    if leak_coefficient > 0.0 {
        *membrane_potential *= 1.0 - leak_coefficient;
    }
}

/// Check if neuron should fire based on threshold and excitability
///
/// # Arguments
/// * `membrane_potential` - Current membrane potential
/// * `threshold` - Firing threshold
/// * `excitability` - Probability of firing (0.0 to 1.0)
/// * `random_value` - Random value in [0.0, 1.0) for probabilistic firing
///
/// # Returns
/// `true` if neuron should fire
///
/// # Example
/// ```
/// use feagi_neural::should_fire;
///
/// let potential = 1.5;
/// let threshold = 1.0;
/// let excitability = 0.8; // 80% chance
/// let random = 0.5; // < 0.8, so should fire
///
/// assert!(should_fire(potential, threshold, excitability, random));
/// ```
#[inline]
pub fn should_fire(
    membrane_potential: f32,
    threshold: f32,
    excitability: f32,
    random_value: f32,
) -> bool {
    if membrane_potential < threshold {
        return false;
    }

    // Fast path: always fire if excitability >= 0.999
    if excitability >= 0.999 {
        return true;
    }

    // Fast path: never fire if excitability <= 0.0
    if excitability <= 0.0 {
        return false;
    }

    // Probabilistic firing
    random_value < excitability
}

/// Batch update multiple LIF neurons (SIMD-friendly layout)
///
/// Processes multiple neurons at once. Data layout is optimized for SIMD.
///
/// # Arguments
/// * `membrane_potentials` - Slice of membrane potentials (mutable)
/// * `thresholds` - Slice of firing thresholds
/// * `leak_coefficients` - Slice of leak coefficients
/// * `candidate_potentials` - Slice of input currents
/// * `fired_mask` - Output: which neurons fired (mutable)
///
/// # Safety
/// All slices must have the same length.
#[inline]
pub fn update_neurons_lif_batch(
    membrane_potentials: &mut [f32],
    thresholds: &[f32],
    leak_coefficients: &[f32],
    candidate_potentials: &[f32],
    fired_mask: &mut [bool],
) {
    let count = membrane_potentials.len();
    debug_assert_eq!(thresholds.len(), count);
    debug_assert_eq!(leak_coefficients.len(), count);
    debug_assert_eq!(candidate_potentials.len(), count);
    debug_assert_eq!(fired_mask.len(), count);

    for i in 0..count {
        fired_mask[i] = update_neuron_lif(
            &mut membrane_potentials[i],
            thresholds[i],
            leak_coefficients[i],
            0.0, // resting_potential
            candidate_potentials[i],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_fires_when_above_threshold() {
        let mut potential = 0.5;
        let fired = update_neuron_lif(&mut potential, 1.0, 0.1, 0.0, 0.6);
        assert!(fired);
        assert_eq!(potential, 0.0); // Reset after firing
    }

    #[test]
    fn test_neuron_does_not_fire_below_threshold() {
        let mut potential = 0.5;
        let fired = update_neuron_lif(&mut potential, 1.0, 0.1, 0.0, 0.3);
        assert!(!fired);
        assert!(potential > 0.0); // Potential accumulated (with leak)
        assert!(potential < 0.8); // Leak applied
    }

    #[test]
    fn test_leak_decay() {
        let mut potential = 1.0;
        apply_leak(&mut potential, 0.5);
        assert_eq!(potential, 0.5); // 50% leak
    }

    #[test]
    fn test_should_fire_above_threshold() {
        assert!(should_fire(1.5, 1.0, 1.0, 0.5));
    }

    #[test]
    fn test_should_not_fire_below_threshold() {
        assert!(!should_fire(0.5, 1.0, 1.0, 0.5));
    }

    #[test]
    fn test_probabilistic_firing() {
        // With random=0.5 and excitability=0.8, should fire
        assert!(should_fire(1.5, 1.0, 0.8, 0.5));
        
        // With random=0.9 and excitability=0.8, should not fire
        assert!(!should_fire(1.5, 1.0, 0.8, 0.9));
    }

    #[test]
    fn test_batch_update() {
        let mut potentials = [0.5, 0.5, 0.5];
        let thresholds = [1.0, 1.0, 1.0];
        let leaks = [0.1, 0.1, 0.1];
        let inputs = [0.6, 0.3, 0.6]; // First and third should fire
        let mut fired = [false; 3];

        update_neurons_lif_batch(&mut potentials, &thresholds, &leaks, &inputs, &mut fired);

        assert!(fired[0]);
        assert!(!fired[1]);
        assert!(fired[2]);
    }
}

