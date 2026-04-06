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

use crate::traits::{Result, SynapseStorage};
use ahash::AHashMap;
use feagi_npu_neural::synapse::{compute_synaptic_contribution, SynapseType};
use rayon::prelude::*;
use std::format;
use std::vec::Vec;

/// Dynamic synapse array for desktop/server environments
pub struct SynapseArray {
    /// Current number of synapses
    pub count: usize,

    /// Source neuron IDs
    pub source_neurons: Vec<u32>,

    /// Target neuron IDs
    pub target_neurons: Vec<u32>,

    /// Synaptic weights (`f32`)
    pub weights: Vec<f32>,

    /// Postsynaptic potentials (`f32`)
    pub postsynaptic_potentials: Vec<f32>,

    /// Synapse types (0=excitatory, 1=inhibitory)
    pub types: Vec<u8>,

    /// Per-synapse packed edge flags (associative STDP, etc.)
    pub edge_flags: Vec<u8>,

    /// Per-synapse delay in whole bursts (`>= 1`).
    pub delay_bursts: Vec<u8>,

    /// Valid synapse mask
    pub valid_mask: Vec<bool>,

    /// Source index for fast lookup
    pub source_index: AHashMap<u32, Vec<usize>>,
}

impl SynapseArray {
    /// Create a new synapse array with initial capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            count: 0,
            source_neurons: Vec::with_capacity(capacity),
            target_neurons: Vec::with_capacity(capacity),
            weights: Vec::with_capacity(capacity),
            postsynaptic_potentials: Vec::with_capacity(capacity),
            types: Vec::with_capacity(capacity),
            edge_flags: Vec::with_capacity(capacity),
            delay_bursts: Vec::with_capacity(capacity),
            valid_mask: Vec::with_capacity(capacity),
            source_index: AHashMap::new(),
        }
    }

    /// Add a synapse (simplified for backward compatibility)
    pub fn add_synapse_simple(
        &mut self,
        source: u32,
        target: u32,
        weight: f32,
        psp: f32,
        synapse_type: SynapseType,
    ) {
        SynapseStorage::add_synapse(self, source, target, weight, psp, synapse_type as u8, 0, 1)
            .expect("Failed to add synapse");
    }

    /// Propagate activity from fired neurons in parallel
    ///
    /// Returns target neuron index → accumulated contribution
    pub fn propagate_parallel(&self, fired_neurons: &[u32]) -> AHashMap<u32, f32> {
        // Collect all synapse indices for fired neurons
        let synapse_indices: Vec<usize> = fired_neurons
            .par_iter()
            .filter_map(|&neuron_id| self.source_index.get(&neuron_id))
            .flatten()
            .copied()
            .collect();

        // Compute contributions in parallel (uses platform-agnostic function)
        let contributions: Vec<(u32, f32)> = synapse_indices
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
                    self.postsynaptic_potentials[syn_idx],
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

// Implement SynapseStorage trait for runtime abstraction
impl SynapseStorage for SynapseArray {
    // Read-only property accessors
    fn source_neurons(&self) -> &[u32] {
        &self.source_neurons[..self.count]
    }

    fn target_neurons(&self) -> &[u32] {
        &self.target_neurons[..self.count]
    }

    fn weights(&self) -> &[f32] {
        &self.weights[..self.count]
    }

    fn postsynaptic_potentials(&self) -> &[f32] {
        &self.postsynaptic_potentials[..self.count]
    }

    fn types(&self) -> &[u8] {
        &self.types[..self.count]
    }

    fn edge_flags(&self) -> &[u8] {
        &self.edge_flags[..self.count]
    }

    fn delay_bursts(&self) -> &[u8] {
        &self.delay_bursts[..self.count]
    }

    fn valid_mask(&self) -> &[bool] {
        &self.valid_mask[..self.count]
    }

    // Mutable property accessors
    fn weights_mut(&mut self) -> &mut [f32] {
        let count = self.count;
        &mut self.weights[..count]
    }

    fn postsynaptic_potentials_mut(&mut self) -> &mut [f32] {
        let count = self.count;
        &mut self.postsynaptic_potentials[..count]
    }

    fn valid_mask_mut(&mut self) -> &mut [bool] {
        let count = self.count;
        &mut self.valid_mask[..count]
    }

    // Metadata
    fn count(&self) -> usize {
        self.count
    }

    fn capacity(&self) -> usize {
        self.source_neurons.capacity()
    }

    // Synapse creation
    fn add_synapse(
        &mut self,
        source: u32,
        target: u32,
        weight: f32,
        psp: f32,
        synapse_type: u8,
        edge_flag: u8,
        delay_bursts: u8,
    ) -> Result<usize> {
        if delay_bursts < 1 {
            return Err(crate::traits::RuntimeError::InvalidParameters(
                "delay_bursts must be >= 1".to_string(),
            ));
        }
        let idx = self.count;

        self.source_neurons.push(source);
        self.target_neurons.push(target);
        self.weights.push(weight);
        self.postsynaptic_potentials.push(psp);
        self.types.push(synapse_type);
        self.edge_flags.push(edge_flag);
        self.delay_bursts.push(delay_bursts);
        self.valid_mask.push(true);

        // Update index
        self.source_index.entry(source).or_default().push(idx);

        self.count += 1;
        Ok(idx)
    }

    fn add_synapses_batch(
        &mut self,
        sources: &[u32],
        targets: &[u32],
        weights: &[f32],
        psps: &[f32],
        types: &[u8],
        edge_flags: Option<&[u8]>,
        delays: &[u8],
    ) -> crate::traits::Result<()> {
        let batch_size = sources.len();
        if let Some(flags) = edge_flags {
            if flags.len() != batch_size {
                return Err(crate::traits::RuntimeError::InvalidParameters(format!(
                    "edge_flags length {} != batch size {}",
                    flags.len(),
                    batch_size
                )));
            }
        }
        if delays.len() != batch_size {
            return Err(crate::traits::RuntimeError::InvalidParameters(format!(
                "delays length {} != batch size {}",
                delays.len(),
                batch_size
            )));
        }
        for i in 0..batch_size {
            let ef = edge_flags.map(|f| f[i]).unwrap_or(0);
            self.add_synapse(sources[i], targets[i], weights[i], psps[i], types[i], ef, delays[i])?;
        }
        Ok(())
    }

    fn remove_synapse(&mut self, idx: usize) -> crate::traits::Result<()> {
        if idx >= self.count {
            return Err(crate::traits::RuntimeError::InvalidParameters(format!(
                "Synapse index {} out of bounds (count: {})",
                idx, self.count
            )));
        }
        self.valid_mask[idx] = false;
        Ok(())
    }

    fn remove_synapses_from_sources(
        &mut self,
        source_neurons: &[u32],
    ) -> crate::traits::Result<usize> {
        let mut removed = 0;
        for idx in 0..self.count {
            if self.valid_mask[idx] && source_neurons.contains(&self.source_neurons[idx]) {
                self.valid_mask[idx] = false;
                removed += 1;
            }
        }
        Ok(removed)
    }

    fn remove_synapses_between(
        &mut self,
        source: u32,
        target: u32,
    ) -> crate::traits::Result<usize> {
        let mut removed = 0;
        for idx in 0..self.count {
            if self.valid_mask[idx]
                && self.source_neurons[idx] == source
                && self.target_neurons[idx] == target
            {
                self.valid_mask[idx] = false;
                removed += 1;
            }
        }
        Ok(removed)
    }

    fn update_weight(&mut self, idx: usize, new_weight: f32) -> crate::traits::Result<()> {
        if idx >= self.count {
            return Err(crate::traits::RuntimeError::InvalidParameters(format!(
                "Synapse index {} out of bounds (count: {})",
                idx, self.count
            )));
        }
        if !self.valid_mask[idx] {
            return Err(crate::traits::RuntimeError::InvalidParameters(format!(
                "Synapse {} is not valid",
                idx
            )));
        }
        self.weights[idx] = new_weight;
        Ok(())
    }

    fn valid_count(&self) -> usize {
        self.valid_mask[..self.count].iter().filter(|&&v| v).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_synapse() {
        let mut array = SynapseArray::new(10);
        array.add_synapse_simple(0, 1, 255.0, 255.0, SynapseType::Excitatory);
        assert_eq!(array.count, 1);
    }

    #[test]
    fn test_propagate_parallel() {
        let mut array = SynapseArray::new(10);
        array.add_synapse_simple(0, 1, 255.0, 255.0, SynapseType::Excitatory);
        array.add_synapse_simple(0, 2, 128.0, 255.0, SynapseType::Excitatory);

        let fired = std::vec![0];
        let contributions = array.propagate_parallel(&fired);

        assert_eq!(contributions.len(), 2);
        assert!(contributions.contains_key(&1));
        assert!(contributions.contains_key(&2));
    }
}
