/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Standard neuron array implementation
//!
//! Uses `Vec` for dynamic growth and Rayon for parallel processing.

use feagi_neural::{update_neuron_lif, is_refractory};
use rayon::prelude::*;

/// Dynamic neuron array for desktop/server environments
pub struct NeuronArray {
    /// Current number of neurons
    pub count: usize,
    
    /// Membrane potentials
    pub membrane_potentials: Vec<f32>,
    
    /// Firing thresholds
    pub thresholds: Vec<f32>,
    
    /// Leak coefficients
    pub leak_coefficients: Vec<f32>,
    
    /// Refractory periods
    pub refractory_periods: Vec<u16>,
    
    /// Refractory countdowns (state)
    pub refractory_countdowns: Vec<u16>,
    
    /// Excitability factors
    pub excitabilities: Vec<f32>,
    
    /// Valid mask
    pub valid_mask: Vec<bool>,
}

impl NeuronArray {
    /// Create a new neuron array with initial capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            count: 0,
            membrane_potentials: vec![0.0; capacity],
            thresholds: vec![1.0; capacity],
            leak_coefficients: vec![0.1; capacity],
            refractory_periods: vec![0; capacity],
            refractory_countdowns: vec![0; capacity],
            excitabilities: vec![1.0; capacity],
            valid_mask: vec![false; capacity],
        }
    }
    
    /// Add a neuron
    pub fn add_neuron(
        &mut self,
        threshold: f32,
        leak: f32,
        refractory_period: u16,
        excitability: f32,
    ) -> usize {
        let idx = self.count;
        
        // Grow if needed
        if idx >= self.membrane_potentials.len() {
            self.membrane_potentials.push(0.0);
            self.thresholds.push(threshold);
            self.leak_coefficients.push(leak);
            self.refractory_periods.push(refractory_period);
            self.refractory_countdowns.push(0);
            self.excitabilities.push(excitability);
            self.valid_mask.push(true);
        } else {
            self.thresholds[idx] = threshold;
            self.leak_coefficients[idx] = leak;
            self.refractory_periods[idx] = refractory_period;
            self.excitabilities[idx] = excitability;
            self.valid_mask[idx] = true;
        }
        
        self.count += 1;
        idx
    }
    
    /// Process burst in parallel using Rayon
    ///
    /// Uses platform-agnostic core functions internally.
    /// 
    /// NOTE: Due to Rust's borrowing rules, parallel processing computes
    /// results first, then applies mutations sequentially. For very small
    /// networks (<100 neurons), sequential may be faster.
    pub fn process_burst_parallel(
        &mut self,
        candidate_potentials: &[f32],
        _burst_count: u64,
    ) -> Vec<usize> {
        // Phase 1: Compute in parallel (read-only)
        let results: Vec<_> = (0..self.count)
            .into_par_iter()
            .map(|idx| {
                if !self.valid_mask[idx] {
                    return (idx, false, 0.0);
                }
                
                let in_refractory = self.refractory_countdowns[idx] > 0;
                if in_refractory {
                    return (idx, false, 0.0);
                }
                
                // Simulate neuron update (read-only)
                let mut potential = self.membrane_potentials[idx];
                let input = candidate_potentials.get(idx).copied().unwrap_or(0.0);
                let fired = update_neuron_lif(
                    &mut potential,
                    self.thresholds[idx],
                    self.leak_coefficients[idx],
                    0.0,
                    input,
                );
                
                (idx, fired, potential)
            })
            .collect();
        
        // Phase 2: Apply mutations sequentially
        let mut fired_indices = Vec::new();
        for (idx, fired, new_potential) in results {
            // Apply refractory countdown
            if self.refractory_countdowns[idx] > 0 {
                self.refractory_countdowns[idx] -= 1;
                continue;
            }
            
            // Apply potential update
            self.membrane_potentials[idx] = new_potential;
            
            if fired {
                self.refractory_countdowns[idx] = self.refractory_periods[idx];
                fired_indices.push(idx);
            }
        }
        
        fired_indices
    }
    
    /// Process burst sequentially (single-threaded)
    pub fn process_burst_sequential(
        &mut self,
        candidate_potentials: &[f32],
        _burst_count: u64,
    ) -> Vec<usize> {
        let mut fired_indices = Vec::new();
        
        for idx in 0..self.count {
            if !self.valid_mask[idx] {
                continue;
            }
            
            if is_refractory(&mut self.refractory_countdowns[idx]) {
                continue;
            }
            
            let input = candidate_potentials.get(idx).copied().unwrap_or(0.0);
            let fired = update_neuron_lif(
                &mut self.membrane_potentials[idx],
                self.thresholds[idx],
                self.leak_coefficients[idx],
                0.0,
                input,
            );
            
            if fired {
                self.refractory_countdowns[idx] = self.refractory_periods[idx];
                fired_indices.push(idx);
            }
        }
        
        fired_indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_neuron() {
        let mut array = NeuronArray::new(10);
        let idx = array.add_neuron(1.0, 0.1, 5, 1.0);
        assert_eq!(idx, 0);
        assert_eq!(array.count, 1);
    }
    
    #[test]
    fn test_process_burst_sequential() {
        let mut array = NeuronArray::new(10);
        array.add_neuron(1.0, 0.1, 5, 1.0);
        array.add_neuron(1.0, 0.1, 5, 1.0);
        
        // High input - should fire
        let inputs = vec![1.5, 0.5]; // First fires, second doesn't
        let fired = array.process_burst_sequential(&inputs, 0);
        
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0], 0);
    }
    
    #[test]
    fn test_process_burst_parallel() {
        let mut array = NeuronArray::new(100);
        for _ in 0..100 {
            array.add_neuron(1.0, 0.1, 5, 1.0);
        }
        
        let inputs = vec![1.5; 100]; // All should fire
        let fired = array.process_burst_parallel(&inputs, 0);
        
        assert_eq!(fired.len(), 100);
    }
}

