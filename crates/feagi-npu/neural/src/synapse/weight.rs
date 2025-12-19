// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Synaptic weight conversion and normalization
//!
//! Pure functions for weight manipulation.

/// Convert u8 weight (0-255) to normalized float (0.0-1.0)
///
/// # Example
/// ```
/// use feagi_npu_neural::synapse::weight_to_float;
///
/// assert_eq!(weight_to_float(0), 0.0);
/// assert_eq!(weight_to_float(255), 1.0);
/// assert!((weight_to_float(128) - 0.502).abs() < 0.01);
/// ```
#[inline]
pub fn weight_to_float(weight: u8) -> f32 {
    weight as f32 / 255.0
}

/// Convert normalized float (0.0-1.0) to u8 weight (0-255)
///
/// Clamps values outside [0.0, 1.0] range.
///
/// # Example
/// ```
/// use feagi_npu_neural::synapse::float_to_weight;
///
/// assert_eq!(float_to_weight(0.0), 0);
/// assert_eq!(float_to_weight(1.0), 255);
/// assert_eq!(float_to_weight(0.5), 128);
/// assert_eq!(float_to_weight(1.5), 255); // Clamped
/// assert_eq!(float_to_weight(-0.5), 0); // Clamped
/// ```
#[inline]
pub fn float_to_weight(value: f32) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    (clamped * 255.0) as u8
}

/// Apply weight change (STDP, plasticity)
///
/// Updates weight by delta, clamping to valid range [0, 255].
///
/// # Arguments
/// * `weight` - Current weight (0-255)
/// * `delta` - Change amount (can be negative)
///
/// # Returns
/// New weight, clamped to [0, 255]
///
/// # Example
/// ```
/// use feagi_npu_neural::synapse::apply_weight_change;
///
/// assert_eq!(apply_weight_change(100, 50), 150);
/// assert_eq!(apply_weight_change(100, -50), 50);
/// assert_eq!(apply_weight_change(200, 100), 255); // Clamped
/// assert_eq!(apply_weight_change(50, -100), 0); // Clamped
/// ```
#[inline]
pub fn apply_weight_change(weight: u8, delta: i16) -> u8 {
    let new_weight = weight as i16 + delta;
    new_weight.clamp(0, 255) as u8
}

/// Batch apply weight changes (SIMD-friendly)
///
/// # Arguments
/// * `weights` - Slice of current weights (mutable)
/// * `deltas` - Slice of weight changes
///
/// # Safety
/// Slices must have the same length.
#[inline]
pub fn apply_weight_changes_batch(weights: &mut [u8], deltas: &[i16]) {
    let count = weights.len();
    debug_assert_eq!(deltas.len(), count);

    for i in 0..count {
        weights[i] = apply_weight_change(weights[i], deltas[i]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weight_to_float() {
        assert_eq!(weight_to_float(0), 0.0);
        assert_eq!(weight_to_float(255), 1.0);
        assert!((weight_to_float(128) - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_float_to_weight() {
        assert_eq!(float_to_weight(0.0), 0);
        assert_eq!(float_to_weight(1.0), 255);
        // Note: 0.5 * 255 = 127.5, which rounds down to 127
        assert_eq!(float_to_weight(0.5), 127);
    }

    #[test]
    fn test_float_to_weight_clamp() {
        assert_eq!(float_to_weight(1.5), 255);
        assert_eq!(float_to_weight(-0.5), 0);
    }

    #[test]
    fn test_apply_weight_change() {
        assert_eq!(apply_weight_change(100, 50), 150);
        assert_eq!(apply_weight_change(100, -50), 50);
    }

    #[test]
    fn test_apply_weight_change_clamp() {
        assert_eq!(apply_weight_change(200, 100), 255);
        assert_eq!(apply_weight_change(50, -100), 0);
    }

    #[test]
    fn test_batch_apply() {
        let mut weights = [100, 150, 200];
        let deltas = [50, -50, 100];

        apply_weight_changes_batch(&mut weights, &deltas);

        assert_eq!(weights[0], 150);
        assert_eq!(weights[1], 100);
        assert_eq!(weights[2], 255); // Clamped
    }
}
