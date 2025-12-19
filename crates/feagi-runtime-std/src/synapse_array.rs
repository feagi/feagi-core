// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Standard synapse array implementation
//!
//! Uses `Vec` and `HashMap` for dynamic growth and fast lookups.

use ahash::AHashMap;
use feagi_synapse::{compute_synaptic_contribution, SynapseType};
use rayon::prelude::*;

/// Dynamic synapse array for desktop/server environments
pub struct SynapseArray {
    /// Current number of synapses
    pub count: usize,
    
    /// Source neuron IDs
    pub source_neurons: Vec<usize>,
    
    /// Target neuron IDs
    pub target_neurons: Vec<usize>,
    
    /// Synaptic weights (0-255)
    pub weights: Vec<u8>,
    
    /// Conductances (0-255)
    pub conductances: Vec<u8>,
    
    /// Synapse types (0=excitatory, 1=inhibitory)
    pub types: Vec<u8>,
    
    /// Source index for fast lookup
    pub source_index: AHashMap<usize, Vec<usize>>,
}

impl SynapseArray {
    /// Create a new synapse array with initial capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            count: 0,
            source_neurons: Vec::with_capacity(capacity),
            target_neurons: Vec::with_capacity(capacity),
            weights: Vec::with_capacity(capacity),
            conductances: Vec::with_capacity(capacity),
            types: Vec::with_capacity(capacity),
            source_index: AHashMap::new(),
        }
    }
    
    /// Add a synapse
    pub fn add_synapse(
        &mut self,
        source: usize,
        target: usize,
        weight: u8,
        conductance: u8,
        synapse_type: SynapseType,
    ) {
        let idx = self.count;
        
        self.source_neurons.push(source);
        self.target_neurons.push(target);
        self.weights.push(weight);
        self.conductances.push(conductance);
        self.types.push(synapse_type as u8);
        
        // Update index
        self.source_index
            .entry(source)
            .or_insert_with(Vec::new)
            .push(idx);
        
        self.count += 1;
    }
    
    /// Propagate activity from fired neurons in parallel
    ///
    /// Returns target neuron index → accumulated contribution
    pub fn propagate_parallel(&self, fired_neurons: &[usize]) -> AHashMap<usize, f32> {
        // Collect all synapse indices for fired neurons
        let synapse_indices: Vec<usize> = fired_neurons
            .par_iter()
            .filter_map(|&neuron_id| self.source_index.get(&neuron_id))
            .flatten()
            .copied()
            .collect();
        
        // Compute contributions in parallel (uses platform-agnostic function)
        let contributions: Vec<(usize, f32)> = synapse_indices
            .par_iter()
            .map(|&syn_idx| {
                let target = self.target_neurons[syn_idx];
                let synapse_type = if self.types[syn_idx] == 0 {
                    SynapseType::Excitatory
                } else {
                    SynapseType::Inhibitory
                };
                
                let contribution = compute_synaptic_contribution(
                    self.weights[syn_idx],
                    self.conductances[syn_idx],
                    synapse_type,
                );
                
                (target, contribution)
            })
            .collect();
        
        // Accumulate by target neuron
        let mut result = AHashMap::new();
        for (target, contribution) in contributions {
            *result.entry(target).or_insert(0.0) += contribution;
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_synapse() {
        let mut array = SynapseArray::new(10);
        array.add_synapse(0, 1, 255, 255, SynapseType::Excitatory);
        assert_eq!(array.count, 1);
    }
    
    #[test]
    fn test_propagate_parallel() {
        let mut array = SynapseArray::new(10);
        array.add_synapse(0, 1, 255, 255, SynapseType::Excitatory);
        array.add_synapse(0, 2, 128, 255, SynapseType::Excitatory);
        
        let fired = vec![0];
        let contributions = array.propagate_parallel(&fired);
        
        assert_eq!(contributions.len(), 2);
        assert!(contributions.contains_key(&1));
        assert!(contributions.contains_key(&2));
    }
}


