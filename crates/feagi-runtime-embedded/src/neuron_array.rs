// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Fixed-size neuron array for embedded systems
//!
//! Uses stack-allocated arrays for predictable memory usage.

use feagi_neural::{update_neuron_lif, is_refractory};
use feagi_types::NeuralValue;

/// Fixed-size neuron array for embedded systems
///
/// All data is stack-allocated with compile-time size limits.
/// No heap allocations, perfect for `no_std` environments.
/// Generic over `T: NeuralValue` to support multiple quantization levels.
///
/// # Example
/// ```
/// use feagi_runtime_embedded::NeuronArray;
///
/// // 100-neuron array on the stack (~5 KB for f32)
/// let mut neurons = NeuronArray::<f32, 100>::new();
/// neurons.add_neuron(1.0, 0.1, 5, 1.0);
/// ```
pub struct NeuronArray<T: NeuralValue, const N: usize> {
    /// Current number of neurons
    pub count: usize,
    
    /// Membrane potentials (quantized to T)
    pub membrane_potentials: [T; N],
    
    /// Firing thresholds (quantized to T)
    pub thresholds: [T; N],
    
    /// Leak coefficients (kept as f32 for precision - see QUANTIZATION_ISSUES_LOG.md #1)
    pub leak_coefficients: [f32; N],
    
    /// Refractory periods
    pub refractory_periods: [u16; N],
    
    /// Refractory countdowns (state)
    pub refractory_countdowns: [u16; N],
    
    /// Excitability factors
    pub excitabilities: [f32; N],
    
    /// Valid mask
    pub valid_mask: [bool; N],
}

impl<T: NeuralValue, const N: usize> NeuronArray<T, N> {
    /// Create a new fixed-size neuron array
    ///
    /// All arrays are zero-initialized on the stack.
    /// Note: Not const due to T::zero() trait method
    pub fn new() -> Self {
        Self {
            count: 0,
            membrane_potentials: [T::zero(); N],
            thresholds: [T::from_f32(1.0); N],
            leak_coefficients: [0.1; N],
            refractory_periods: [0; N],
            refractory_countdowns: [0; N],
            excitabilities: [1.0; N],
            valid_mask: [false; N],
        }
    }
    
    /// Add a neuron
    ///
    /// Returns the neuron index, or None if array is full.
    pub fn add_neuron(
        &mut self,
        threshold: T,
        leak: f32,
        refractory_period: u16,
        excitability: f32,
    ) -> Option<usize> {
        if self.count >= N {
            return None; // Array full
        }
        
        let idx = self.count;
        self.thresholds[idx] = threshold;
        self.leak_coefficients[idx] = leak;
        self.refractory_periods[idx] = refractory_period;
        self.excitabilities[idx] = excitability;
        self.valid_mask[idx] = true;
        self.count += 1;
        
        Some(idx)
    }
    
    /// Process burst (single-threaded, deterministic)
    ///
    /// Uses platform-agnostic core functions internally.
    /// Returns number of neurons that fired.
    ///
    /// # Arguments
    /// * `candidate_potentials` - Input currents for each neuron
    /// * `fired_mask` - Output: which neurons fired (caller-allocated)
    ///
    /// # Example
    /// ```
    /// let mut neurons = NeuronArray::<f32, 100>::new();
    /// let inputs = [1.5; 100];
    /// let mut fired = [false; 100];
    /// let count = neurons.process_burst(&inputs, &mut fired);
    /// ```
    pub fn process_burst(
        &mut self,
        candidate_potentials: &[T; N],
        fired_mask: &mut [bool; N],
    ) -> usize {
        let mut fired_count = 0;
        
        for idx in 0..self.count {
            fired_mask[idx] = false;
            
            if !self.valid_mask[idx] {
                continue;
            }
            
            // Check refractory (uses platform-agnostic function)
            if is_refractory(&mut self.refractory_countdowns[idx]) {
                continue;
            }
            
            // Update neuron (uses platform-agnostic function)
            let input = candidate_potentials[idx];
            let fired = update_neuron_lif(
                &mut self.membrane_potentials[idx],
                self.thresholds[idx],
                self.leak_coefficients[idx],
                T::zero(), // resting_potential
                input,
            );
            
            if fired {
                // Apply refractory
                self.refractory_countdowns[idx] = self.refractory_periods[idx];
                fired_mask[idx] = true;
                fired_count += 1;
            }
        }
        
        fired_count
    }
    
    /// Get memory footprint in bytes
    pub const fn memory_footprint() -> usize {
        core::mem::size_of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_f32() {
        let array = NeuronArray::<f32, 10>::new();
        assert_eq!(array.count, 0);
    }
    
    #[test]
    fn test_add_neuron_f32() {
        let mut array = NeuronArray::<f32, 10>::new();
        let idx = array.add_neuron(1.0, 0.1, 5, 1.0);
        assert_eq!(idx, Some(0));
        assert_eq!(array.count, 1);
    }
    
    #[test]
    fn test_array_full_f32() {
        let mut array = NeuronArray::<f32, 2>::new();
        assert!(array.add_neuron(1.0, 0.1, 5, 1.0).is_some());
        assert!(array.add_neuron(1.0, 0.1, 5, 1.0).is_some());
        assert!(array.add_neuron(1.0, 0.1, 5, 1.0).is_none()); // Full
    }
    
    #[test]
    fn test_process_burst_f32() {
        let mut array = NeuronArray::<f32, 10>::new();
        array.add_neuron(1.0, 0.1, 5, 1.0);
        array.add_neuron(1.0, 0.1, 5, 1.0);
        
        let mut inputs = [0.0; 10];
        inputs[0] = 1.5; // Should fire
        inputs[1] = 0.5; // Should not fire
        
        let mut fired = [false; 10];
        let count = array.process_burst(&inputs, &mut fired);
        
        assert_eq!(count, 1);
        assert!(fired[0]);
        assert!(!fired[1]);
    }
    
    #[test]
    fn test_memory_footprint_f32() {
        let size = NeuronArray::<f32, 100>::memory_footprint();
        // ~100 neurons × 48 bytes = ~4.8 KB
        assert!(size < 10_000); // Should be under 10 KB
    }
}

