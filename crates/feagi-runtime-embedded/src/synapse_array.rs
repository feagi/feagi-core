/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Fixed-size synapse array for embedded systems
//!
//! Uses stack-allocated arrays for predictable memory usage.

use feagi_synapse::{compute_synaptic_contribution, SynapseType};

/// Fixed-size synapse array for embedded systems
///
/// All data is stack-allocated with compile-time size limits.
/// No heap allocations, perfect for `no_std` environments.
pub struct SynapseArray<const N: usize> {
    /// Current number of synapses
    pub count: usize,
    
    /// Source neuron IDs
    pub source_neurons: [u16; N],
    
    /// Target neuron IDs
    pub target_neurons: [u16; N],
    
    /// Synaptic weights (0-255)
    pub weights: [u8; N],
    
    /// Conductances (0-255)
    pub conductances: [u8; N],
    
    /// Synapse types (0=excitatory, 1=inhibitory)
    pub types: [u8; N],
}

impl<const N: usize> SynapseArray<N> {
    /// Create a new fixed-size synapse array
    pub const fn new() -> Self {
        Self {
            count: 0,
            source_neurons: [0; N],
            target_neurons: [0; N],
            weights: [0; N],
            conductances: [0; N],
            types: [0; N],
        }
    }
    
    /// Add a synapse
    ///
    /// Returns true if successful, false if array is full.
    pub fn add_synapse(
        &mut self,
        source: u16,
        target: u16,
        weight: u8,
        conductance: u8,
        synapse_type: SynapseType,
    ) -> bool {
        if self.count >= N {
            return false; // Array full
        }
        
        let idx = self.count;
        self.source_neurons[idx] = source;
        self.target_neurons[idx] = target;
        self.weights[idx] = weight;
        self.conductances[idx] = conductance;
        self.types[idx] = synapse_type as u8;
        self.count += 1;
        
        true
    }
    
    /// Propagate activity from fired neurons (single-threaded)
    ///
    /// Uses platform-agnostic core functions internally.
    ///
    /// # Arguments
    /// * `fired_mask` - Which neurons fired (indexed by neuron ID)
    /// * `contributions` - Output: accumulated contributions per target neuron (caller-allocated)
    pub fn propagate<const MAX_NEURONS: usize>(
        &self,
        fired_mask: &[bool; MAX_NEURONS],
        contributions: &mut [f32; MAX_NEURONS],
    ) {
        // Zero contributions
        for contrib in contributions.iter_mut() {
            *contrib = 0.0;
        }
        
        // Process each synapse
        for idx in 0..self.count {
            let source = self.source_neurons[idx] as usize;
            
            // Skip if source didn't fire
            if source >= MAX_NEURONS || !fired_mask[source] {
                continue;
            }
            
            let target = self.target_neurons[idx] as usize;
            if target >= MAX_NEURONS {
                continue;
            }
            
            // Compute contribution (uses platform-agnostic function)
            let synapse_type = if self.types[idx] == 0 {
                SynapseType::Excitatory
            } else {
                SynapseType::Inhibitory
            };
            
            let contribution = compute_synaptic_contribution(
                self.weights[idx],
                self.conductances[idx],
                synapse_type,
            );
            
            contributions[target] += contribution;
        }
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
    fn test_new() {
        let array = SynapseArray::<10>::new();
        assert_eq!(array.count, 0);
    }
    
    #[test]
    fn test_add_synapse() {
        let mut array = SynapseArray::<10>::new();
        assert!(array.add_synapse(0, 1, 255, 255, SynapseType::Excitatory));
        assert_eq!(array.count, 1);
    }
    
    #[test]
    fn test_array_full() {
        let mut array = SynapseArray::<2>::new();
        assert!(array.add_synapse(0, 1, 255, 255, SynapseType::Excitatory));
        assert!(array.add_synapse(1, 2, 255, 255, SynapseType::Excitatory));
        assert!(!array.add_synapse(2, 3, 255, 255, SynapseType::Excitatory)); // Full
    }
    
    #[test]
    fn test_propagate() {
        let mut array = SynapseArray::<10>::new();
        array.add_synapse(0, 1, 255, 255, SynapseType::Excitatory);
        array.add_synapse(0, 2, 128, 255, SynapseType::Excitatory);
        
        let mut fired = [false; 10];
        fired[0] = true; // Neuron 0 fired
        
        let mut contributions = [0.0; 10];
        array.propagate(&fired, &mut contributions);
        
        assert!(contributions[1] > 0.0); // Excitatory contribution
        assert!(contributions[2] > 0.0); // Excitatory contribution
    }
    
    #[test]
    fn test_memory_footprint() {
        let size = SynapseArray::<1000>::memory_footprint();
        // ~1000 synapses × 8 bytes = ~8 KB
        assert!(size < 15_000); // Should be under 15 KB
    }
}

