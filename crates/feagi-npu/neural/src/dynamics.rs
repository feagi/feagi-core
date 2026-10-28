// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

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
//!
//! # Quantization Support (Phase 3)
//!
//! Algorithms are generic over `T: NeuralValue` to support multiple numeric
//! precisions (f32, f16, int8) configured via genome.

use crate::types::NeuralValue;

/// Fraction of membrane charge that survives `elapsed_bursts` bursts of leak.
///
/// # Formula
/// `retention = (1 - leak_coefficient) ^ elapsed_bursts`
///
/// This is the closed form of applying the per-burst recurrence
/// `V_t = V_{t-1} * (1 - g) + I_t` over a stretch of bursts in which the neuron received no
/// input, so the leak can be applied lazily when a neuron next receives input instead of on
/// every burst. `elapsed_bursts == 0` and `leak_coefficient == 0` return exactly 1, and
/// `elapsed_bursts == 1` skips the power calculation.
#[inline]
pub fn leak_retention(leak_coefficient: f32, elapsed_bursts: u64) -> f32 {
    if elapsed_bursts == 0 || leak_coefficient == 0.0 {
        return 1.0;
    }
    let base = 1.0 - leak_coefficient;
    if elapsed_bursts == 1 {
        return base;
    }
    // Exponentiation by squaring in f64 (no libm needed, works in no_std); one power per call.
    let mut result = 1.0_f64;
    let mut factor = base as f64;
    let mut n = elapsed_bursts;
    while n > 0 {
        if n & 1 == 1 {
            result *= factor;
        }
        n >>= 1;
        if n > 0 {
            factor *= factor;
        }
    }
    result as f32
}

/// Membrane charge after `elapsed_bursts` bursts of leak with no input.
///
/// `charge * (1 - leak_coefficient) ^ elapsed_bursts`. Pure: used both by the burst step
/// (before adding input) and by readers that report V between inputs.
#[inline]
pub fn decayed_charge(charge: f32, leak_coefficient: f32, elapsed_bursts: u64) -> f32 {
    charge * leak_retention(leak_coefficient, elapsed_bursts)
}

/// Update a single LIF (Leaky Integrate-and-Fire) neuron
///
/// Generic over numeric type T to support quantization.
///
/// This is the core neural dynamics algorithm extracted from the burst engine.
/// It is pure, deterministic, and platform-agnostic.
///
/// # Arguments
/// * `membrane_potential` - Membrane potential as of the neuron's last update (mutable)
/// * `threshold` - Firing threshold
/// * `leak_coefficient` - Leak rate (0.0 to 1.0, fraction of V lost per burst)
/// * `resting_potential` - Resting potential (not used in current implementation)
/// * `candidate_potential` - Input current from synapses
/// * `elapsed_bursts` - Bursts since the neuron's last update (`burst - last_update_burst`)
///
/// # Returns
/// `true` if neuron fired, `false` otherwise
///
/// # Algorithm (standard per-burst LIF, leak applied lazily)
/// Per burst: `V_t = V_{t-1} * (1 - g) + I_t`. A neuron is only touched when it receives
/// input at burst `t`, so the leak of the idle stretch is applied in one step:
/// 1. `V = V * (1 - g) ^ elapsed_bursts`
/// 2. `V = V + I`
/// 3. If `V >= threshold`: fire, reset to 0.0
/// 4. Otherwise keep `V` (no further leak: the next decay happens when the next
///    input arrives, or when a reader catches the value up)
///
/// # Example
/// ```
/// use feagi_npu_neural::update_neuron_lif;
///
/// let mut potential = 0.5f32;
/// let mut fraction = 0.0f32;
/// let fired = update_neuron_lif(&mut potential, &mut fraction, 1.0, 0.0, 0.1, 0.0, 0.6, 1);
/// assert!(fired); // 0.5 * 0.9 + 0.6 = 1.05 >= 1.0
/// ```
#[inline]
#[allow(clippy::too_many_arguments)] // Mirrors the batch layout; one scalar per column
pub fn update_neuron_lif<T: NeuralValue>(
    membrane_potential: &mut T,
    membrane_fraction: &mut f32,
    threshold: T,
    threshold_fraction: f32,
    leak_coefficient: f32, // Always f32 (small values don't quantize well)
    _resting_potential: T, // Not used in current LIF implementation
    candidate_potential: f32,
    elapsed_bursts: u64,
) -> bool {
    // Catch the stored charge up to this burst, then add the f32 synaptic current.
    // Quantizing the current first turns inhibition and sub-byte PSP into 0.
    let stored = membrane_potential.to_f32() + *membrane_fraction;
    let decayed = decayed_charge(stored, leak_coefficient, elapsed_bursts);
    let sum = if candidate_potential.is_finite() {
        decayed + candidate_potential
    } else {
        decayed
    };
    let (level, fraction) = T::split_charge(sum);
    *membrane_potential = level;
    *membrane_fraction = fraction;

    let charge = membrane_potential.to_f32() + *membrane_fraction;
    let threshold_charge = threshold.to_f32() + threshold_fraction;
    if charge >= threshold_charge {
        *membrane_potential = T::zero();
        *membrane_fraction = 0.0;
        return true;
    }
    false
}

/// Apply one burst of leak decay to a membrane potential
///
/// # Formula
/// `V_new = V_current * (1 - leak_coefficient)`
///
/// For `n` bursts use [`decayed_charge`] / [`leak_retention`]:
/// `V_new = V_current * (1 - leak_coefficient) ^ n`.
///
/// # Arguments
/// * `membrane_potential` - Current membrane potential (mutable)
/// * `leak_coefficient` - Leak rate (0.0-1.0, fraction lost per burst)
#[inline]
pub fn apply_leak<T: NeuralValue>(membrane_potential: &mut T, leak_coefficient: f32) {
    *membrane_potential = membrane_potential.mul_leak(leak_coefficient);
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
/// use feagi_npu_neural::should_fire;
///
/// let potential = 1.5f32;
/// let threshold = 1.0f32;
/// let excitability = 0.8;
/// let random = 0.5;
///
/// assert!(should_fire(potential, threshold, excitability, random));
/// ```
#[inline]
pub fn should_fire<T: NeuralValue>(
    membrane_potential: T,
    threshold: T,
    excitability: f32,
    random_value: f32,
) -> bool {
    if membrane_potential.lt(threshold) {
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
/// * `elapsed_bursts` - Bursts since each neuron's last update
/// * `fired_mask` - Output: which neurons fired (mutable)
///
/// # Safety
/// All slices must have the same length.
#[inline]
#[allow(clippy::too_many_arguments)] // One slice per neuron column (SIMD layout)
pub fn update_neurons_lif_batch<T: NeuralValue>(
    membrane_potentials: &mut [T],
    membrane_fractions: &mut [f32],
    thresholds: &[T],
    threshold_fractions: &[f32],
    leak_coefficients: &[f32], // Always f32 (small values)
    candidate_potentials: &[f32],
    elapsed_bursts: &[u64],
    fired_mask: &mut [bool],
) {
    let count = membrane_potentials.len();
    debug_assert_eq!(membrane_fractions.len(), count);
    debug_assert_eq!(thresholds.len(), count);
    debug_assert_eq!(threshold_fractions.len(), count);
    debug_assert_eq!(leak_coefficients.len(), count);
    debug_assert_eq!(candidate_potentials.len(), count);
    debug_assert_eq!(elapsed_bursts.len(), count);
    debug_assert_eq!(fired_mask.len(), count);

    for i in 0..count {
        fired_mask[i] = update_neuron_lif(
            &mut membrane_potentials[i],
            &mut membrane_fractions[i],
            thresholds[i],
            threshold_fractions[i],
            leak_coefficients[i],
            T::zero(),
            candidate_potentials[i],
            elapsed_bursts[i],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_fires_when_above_threshold() {
        let mut potential = 0.5;
        let mut fraction = 0.0;
        let fired = update_neuron_lif(&mut potential, &mut fraction, 1.0, 0.0, 0.1, 0.0, 0.6, 1);
        assert!(fired);
        assert_eq!(potential, 0.0); // Reset after firing
    }

    #[test]
    fn test_neuron_does_not_fire_below_threshold() {
        let mut potential = 0.5;
        let mut fraction = 0.0;
        let fired = update_neuron_lif(&mut potential, &mut fraction, 1.0, 0.0, 0.1, 0.0, 0.3, 1);
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
        let mut fractions = [0.0; 3];
        let threshold_fractions = [0.0; 3];
        let mut fired = [false; 3];

        update_neurons_lif_batch(
            &mut potentials,
            &mut fractions,
            &thresholds,
            &threshold_fractions,
            &leaks,
            &inputs,
            &[1; 3],
            &mut fired,
        );

        assert!(fired[0]);
        assert!(!fired[1]);
        assert!(fired[2]);
    }

    // INT8 membrane levels are absolute bytes, 0..=255.
    #[test]
    fn test_int8_neuron_fires_when_above_threshold() {
        use crate::types::INT8Value;

        let mut potential = INT8Value::zero();
        let mut fraction = 0.0;
        let threshold = INT8Value::from_f32(150.0);
        let input = 200.0;

        let fired = update_neuron_lif(
            &mut potential,
            &mut fraction,
            threshold,
            0.0,
            0.0,
            INT8Value::zero(),
            input,
            1,
        );

        assert!(fired, "200 >= 150 should fire");
        assert_eq!(potential.to_f32(), 0.0);
    }

    #[test]
    fn test_int8_neuron_does_not_fire_below_threshold() {
        use crate::types::INT8Value;

        let mut potential = INT8Value::zero();
        let mut fraction = 0.0;
        let threshold = INT8Value::from_f32(150.0);
        let input = 100.0;

        let fired = update_neuron_lif(
            &mut potential,
            &mut fraction,
            threshold,
            0.0,
            0.0,
            INT8Value::zero(),
            input,
            1,
        );

        assert!(!fired, "100 < 150 should not fire");
        assert_eq!(potential.to_f32(), 100.0);
    }

    #[test]
    fn test_int8_leak_application() {
        use crate::types::INT8Value;

        let mut potential = INT8Value::from_f32(200.0);
        apply_leak(&mut potential, 0.05);

        // 200 * 0.95 = 190, exact on the byte scale.
        assert_eq!(potential.to_f32(), 190.0);
    }

    #[test]
    fn test_int8_batch_update() {
        use crate::types::INT8Value;

        let mut potentials = [
            INT8Value::zero(),
            INT8Value::zero(),
            INT8Value::from_f32(40.0),
        ];
        let thresholds = [
            INT8Value::from_f32(150.0),
            INT8Value::from_f32(150.0),
            INT8Value::from_f32(150.0),
        ];
        let leaks = [0.0f32, 0.0, 0.0];
        let inputs = [200.0_f32, 100.0, 120.0];
        let mut fractions = [0.0; 3];
        let threshold_fractions = [0.0; 3];
        let mut fired = [false; 3];

        update_neurons_lif_batch(
            &mut potentials,
            &mut fractions,
            &thresholds,
            &threshold_fractions,
            &leaks,
            &inputs,
            &[1; 3],
            &mut fired,
        );

        assert!(fired[0], "200 >= 150 should fire");
        assert!(!fired[1], "100 < 150 should not fire");
        assert!(fired[2], "160 >= 150 should fire");
        assert_eq!(potentials[1].to_f32(), 100.0);
    }

    #[test]
    fn test_int8_sub_byte_psp_is_not_dropped() {
        use crate::types::INT8Value;

        let mut potential = INT8Value::zero();
        let mut fraction = 0.0;
        let (threshold, threshold_fraction) = INT8Value::split_charge(0.38);
        let fired = update_neuron_lif(
            &mut potential,
            &mut fraction,
            threshold,
            threshold_fraction,
            0.0,
            INT8Value::zero(),
            0.38,
            1,
        );
        assert!(fired, "a 0.38 PSP must meet a 0.38 threshold");
        assert_eq!(potential.to_f32(), 0.0);
        assert_eq!(fraction, 0.0);

        let mut held = INT8Value::zero();
        let mut held_fraction = 0.0;
        let held_fired = update_neuron_lif(
            &mut held,
            &mut held_fraction,
            threshold,
            threshold_fraction,
            0.0,
            INT8Value::zero(),
            0.2,
            1,
        );
        assert!(!held_fired, "0.2 must stay below a 0.38 threshold");
        assert_eq!(held.to_f32(), 0.0);
        assert!((held_fraction - 0.2).abs() < 1.0e-6);
    }

    #[test]
    fn test_int8_inhibition_reaches_the_membrane() {
        use crate::types::INT8Value;

        let mut potential = INT8Value::from_f32(100.0);
        let mut fraction = 0.0;
        let threshold = INT8Value::from_f32(150.0);
        let fired = update_neuron_lif(
            &mut potential,
            &mut fraction,
            threshold,
            0.0,
            0.0,
            INT8Value::zero(),
            -30.0,
            1,
        );
        assert!(!fired);
        assert_eq!(potential.to_f32(), 70.0);
    }

    #[test]
    fn test_leak_retention_matches_power() {
        for &g in &[0.0_f32, 0.02, 0.15, 0.5, 0.99] {
            for n in [0_u64, 1, 2, 3, 7, 50, 1000] {
                let expected = (1.0 - g as f64).powi(n as i32);
                let got = leak_retention(g, n) as f64;
                assert!(
                    (got - expected).abs() < 1e-6,
                    "g={g} n={n}: {got} vs {expected}"
                );
            }
        }
        assert_eq!(leak_retention(0.3, 0), 1.0);
        assert_eq!(leak_retention(0.0, 12345), 1.0);
        assert_eq!(leak_retention(0.3, 1), 0.7);
    }

    #[test]
    fn test_lazy_update_equals_per_burst_recurrence() {
        // Input 0.4 every 5th burst at g = 0.15, vs a brute-force per-burst simulation.
        let g = 0.15_f32;
        let (mut p, mut f) = (0.0_f32, 0.0_f32);
        let mut brute = 0.0_f32;
        let mut last = 0_u64;
        for t in 1..=60_u64 {
            let input = if t % 5 == 0 { 0.4 } else { 0.0 };
            brute = brute * (1.0 - g) + input;
            if t % 5 == 0 {
                update_neuron_lif(&mut p, &mut f, 100.0, 0.0, g, 0.0, input, t - last);
                last = t;
                assert!((p - brute).abs() < 1e-6, "t={t}: {p} vs {brute}");
            }
        }
    }
}
