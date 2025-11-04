/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Utility functions for neural processing
//!
//! Platform-agnostic helpers.

/// Fast PCG hash for deterministic pseudo-random number generation
///
/// Based on PCG family of PRNGs: https://www.pcg-random.org/
/// Provides fast, high-quality pseudo-random numbers without mutable state.
///
/// # Example
/// ```
/// use feagi_neural::pcg_hash;
///
/// let hash1 = pcg_hash(42);
/// let hash2 = pcg_hash(42);
/// assert_eq!(hash1, hash2); // Deterministic
/// ```
#[inline(always)]
pub fn pcg_hash(input: u32) -> u32 {
    let state = input.wrapping_mul(747796405).wrapping_add(2891336453);
    let word = ((state >> ((state >> 28) + 4)) ^ state).wrapping_mul(277803737);
    (word >> 22) ^ word
}

/// Convert PCG hash to floating point in range [0, 1)
///
/// # Example
/// ```
/// use feagi_neural::pcg_hash_to_float;
///
/// let random = pcg_hash_to_float(42);
/// assert!(random >= 0.0 && random < 1.0);
/// ```
#[inline(always)]
pub fn pcg_hash_to_float(input: u32) -> f32 {
    (pcg_hash(input) as f32) / 4294967296.0
}

/// Generate pseudo-random value for excitability check
///
/// Combines neuron_id AND burst_count to ensure different random values each burst.
///
/// # Arguments
/// * `neuron_id` - Unique neuron identifier
/// * `burst_count` - Current burst number (timestep)
///
/// # Returns
/// Random value in [0.0, 1.0)
///
/// # Example
/// ```
/// use feagi_neural::excitability_random;
///
/// let random1 = excitability_random(42, 100);
/// let random2 = excitability_random(42, 101);
/// assert_ne!(random1, random2); // Different each burst
/// ```
#[inline(always)]
pub fn excitability_random(neuron_id: u32, burst_count: u64) -> f32 {
    // Combine neuron_id and burst_count for different values each burst
    let seed = neuron_id
        .wrapping_mul(2654435761)
        .wrapping_add((burst_count as u32).wrapping_mul(1597334677));
    pcg_hash_to_float(seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcg_hash_deterministic() {
        let hash1 = pcg_hash(42);
        let hash2 = pcg_hash(42);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_pcg_hash_different() {
        let hash1 = pcg_hash(42);
        let hash2 = pcg_hash(43);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_pcg_hash_to_float_range() {
        for i in 0..100 {
            let random = pcg_hash_to_float(i);
            assert!(random >= 0.0);
            assert!(random < 1.0);
        }
    }

    #[test]
    fn test_excitability_random_changes_per_burst() {
        let neuron_id = 42;
        let random1 = excitability_random(neuron_id, 100);
        let random2 = excitability_random(neuron_id, 101);
        assert_ne!(random1, random2);
    }

    #[test]
    fn test_excitability_random_different_neurons() {
        let burst_count = 100;
        let random1 = excitability_random(1, burst_count);
        let random2 = excitability_random(2, burst_count);
        assert_ne!(random1, random2);
    }
}


