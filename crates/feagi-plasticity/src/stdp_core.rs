/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Pure STDP computation (platform-agnostic, no_std compatible)
//!
//! This module contains the core STDP algorithms extracted for cross-platform use.
//! It works without allocations and can run on embedded systems.

#![allow(unused)]

/// STDP configuration parameters
#[derive(Debug, Clone, Copy)]
pub struct STDPConfig {
    /// Pre-synaptic time constant (τ_pre)
    pub tau_pre: f32,
    /// Post-synaptic time constant (τ_post)
    pub tau_post: f32,
    /// Potentiation learning rate (A+)
    pub a_plus: f32,
    /// Depression learning rate (A-)
    pub a_minus: f32,
}

impl Default for STDPConfig {
    fn default() -> Self {
        Self {
            tau_pre: 20.0,
            tau_post: 20.0,
            a_plus: 0.01,
            a_minus: 0.012,
        }
    }
}

/// Compute STDP weight change for a single synapse
///
/// Uses exponential STDP rule:
/// - Δw = A+ * exp(-Δt/τ_pre) if pre before post (potentiation)
/// - Δw = -A- * exp(Δt/τ_post) if post before pre (depression)
///
/// # Arguments
/// * `dt` - Spike timing difference (post_time - pre_time) in timesteps
/// * `config` - STDP configuration
///
/// # Returns
/// Weight change factor (positive for potentiation, negative for depression)
///
/// # Example
/// ```
/// use feagi_plasticity::stdp_core::{compute_stdp_weight_change, STDPConfig};
///
/// let config = STDPConfig::default();
/// 
/// // Pre before post (dt > 0) → potentiation
/// let delta_w = compute_stdp_weight_change(5, &config);
/// assert!(delta_w > 0.0);
///
/// // Post before pre (dt < 0) → depression
/// let delta_w = compute_stdp_weight_change(-5, &config);
/// assert!(delta_w < 0.0);
/// ```
#[inline]
pub fn compute_stdp_weight_change(dt: i32, config: &STDPConfig) -> f32 {
    if dt > 0 {
        // Pre fired before post → potentiation
        let dt_f = dt as f32;
        config.a_plus * (-dt_f / config.tau_pre.max(1e-6)).exp()
    } else if dt < 0 {
        // Post fired before pre → depression
        let dt_f = (-dt) as f32;
        -config.a_minus * (-dt_f / config.tau_post.max(1e-6)).exp()
    } else {
        // Same timestep → strong potentiation
        config.a_plus
    }
}

/// Apply STDP weight update to a synapse weight (u8 format)
///
/// # Arguments
/// * `current_weight` - Current synaptic weight (0-255)
/// * `dt` - Spike timing difference
/// * `config` - STDP configuration
///
/// # Returns
/// New weight, clamped to [0, 255]
///
/// # Example
/// ```
/// use feagi_plasticity::stdp_core::{update_weight_stdp, STDPConfig};
///
/// let config = STDPConfig::default();
/// let weight = 128;
///
/// // Pre before post → increase weight
/// let new_weight = update_weight_stdp(weight, 5, &config);
/// assert!(new_weight > weight);
/// ```
#[inline]
pub fn update_weight_stdp(current_weight: u8, dt: i32, config: &STDPConfig) -> u8 {
    let delta_w = compute_stdp_weight_change(dt, config);
    let new_weight = current_weight as f32 + delta_w * 255.0;
    new_weight.clamp(0.0, 255.0) as u8
}

/// Batch compute STDP weight changes (SIMD-friendly)
///
/// # Arguments
/// * `time_diffs` - Slice of timing differences
/// * `config` - STDP configuration
/// * `weight_changes` - Output slice (mutable)
///
/// # Safety
/// Slices must have the same length.
#[inline]
pub fn compute_stdp_batch(
    time_diffs: &[i32],
    config: &STDPConfig,
    weight_changes: &mut [f32],
) {
    let count = time_diffs.len();
    debug_assert_eq!(weight_changes.len(), count);

    for i in 0..count {
        weight_changes[i] = compute_stdp_weight_change(time_diffs[i], config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdp_potentiation() {
        let config = STDPConfig::default();
        let delta_w = compute_stdp_weight_change(5, &config);
        assert!(delta_w > 0.0);
        assert!(delta_w <= config.a_plus);
    }

    #[test]
    fn test_stdp_depression() {
        let config = STDPConfig::default();
        let delta_w = compute_stdp_weight_change(-5, &config);
        assert!(delta_w < 0.0);
        assert!(delta_w >= -config.a_minus);
    }

    #[test]
    fn test_stdp_same_time() {
        let config = STDPConfig::default();
        let delta_w = compute_stdp_weight_change(0, &config);
        assert_eq!(delta_w, config.a_plus);
    }

    #[test]
    fn test_stdp_exponential_decay() {
        let config = STDPConfig::default();
        let delta_w1 = compute_stdp_weight_change(1, &config);
        let delta_w2 = compute_stdp_weight_change(10, &config);
        assert!(delta_w1 > delta_w2); // Closer spikes have stronger effect
    }

    #[test]
    fn test_update_weight_potentiation() {
        let config = STDPConfig::default();
        let weight = 128;
        let new_weight = update_weight_stdp(weight, 5, &config);
        assert!(new_weight > weight);
    }

    #[test]
    fn test_update_weight_depression() {
        let config = STDPConfig::default();
        let weight = 128;
        let new_weight = update_weight_stdp(weight, -5, &config);
        assert!(new_weight < weight);
    }

    #[test]
    fn test_update_weight_clamp_high() {
        let config = STDPConfig {
            a_plus: 1.0, // Very strong potentiation
            ..Default::default()
        };
        let weight = 250;
        let new_weight = update_weight_stdp(weight, 1, &config);
        assert_eq!(new_weight, 255); // Clamped to max
    }

    #[test]
    fn test_update_weight_clamp_low() {
        let config = STDPConfig {
            a_minus: 1.0, // Very strong depression
            ..Default::default()
        };
        let weight = 5;
        let new_weight = update_weight_stdp(weight, -1, &config);
        assert_eq!(new_weight, 0); // Clamped to min
    }

    #[test]
    fn test_batch_computation() {
        let config = STDPConfig::default();
        let time_diffs = [5, -5, 0, 10];
        let mut changes = [0.0; 4];

        compute_stdp_batch(&time_diffs, &config, &mut changes);

        assert!(changes[0] > 0.0); // Potentiation
        assert!(changes[1] < 0.0); // Depression
        assert_eq!(changes[2], config.a_plus); // Same time
        assert!(changes[3] > 0.0); // Potentiation (weaker)
        assert!(changes[0] > changes[3]); // Closer spike stronger
    }
}


