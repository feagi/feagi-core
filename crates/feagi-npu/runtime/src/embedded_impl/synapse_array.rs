// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Fixed-size synapse array for embedded systems
//!
//! Uses stack-allocated arrays for predictable memory usage.

use crate::traits::{Result, RuntimeError, SynapseStorage};
use feagi_npu_neural::synapse::{compute_synaptic_contribution, SynapseType};

/// Fixed-size synapse array for embedded systems
///
/// All data is stack-allocated with compile-time size limits.
/// No heap allocations, perfect for `no_std` environments.
pub struct SynapseArray<const N: usize> {
    /// Current number of synapses
    pub count: usize,

    /// Source neuron IDs
    pub source_neurons: [u32; N],

    /// Target neuron IDs
    pub target_neurons: [u32; N],

    /// Synaptic weights (0-255)
    pub weights: [u8; N],

    /// Postsynaptic potentials (0-255)
    pub postsynaptic_potentials: [u8; N],

    /// Synapse types (0=excitatory, 1=inhibitory)
    pub types: [u8; N],

    /// Valid synapse mask
    pub valid_mask: [bool; N],
}

impl<const N: usize> SynapseArray<N> {
    /// Create a new fixed-size synapse array
    pub const fn new() -> Self {
        Self {
            count: 0,
            source_neurons: [0; N],
            target_neurons: [0; N],
            weights: [0; N],
            postsynaptic_potentials: [0; N],
            types: [0; N],
            valid_mask: [false; N],
        }
    }
}

impl<const N: usize> Default for SynapseArray<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> SynapseArray<N> {
    /// Add a synapse (simplified for backward compatibility)
    ///
    /// Returns true if successful, false if array is full.
    pub fn add_synapse_simple(
        &mut self,
        source: u32,
        target: u32,
        weight: u8,
        psp: u8,
        synapse_type: SynapseType,
    ) -> bool {
        SynapseStorage::add_synapse(self, source, target, weight, psp, synapse_type as u8).is_ok()
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
                self.postsynaptic_potentials[idx],
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

// Implement SynapseStorage trait
impl<const N: usize> SynapseStorage for SynapseArray<N> {
    fn source_neurons(&self) -> &[u32] {
        &self.source_neurons[..self.count]
    }

    fn target_neurons(&self) -> &[u32] {
        &self.target_neurons[..self.count]
    }

    fn weights(&self) -> &[u8] {
        &self.weights[..self.count]
    }

    fn postsynaptic_potentials(&self) -> &[u8] {
        &self.postsynaptic_potentials[..self.count]
    }

    fn types(&self) -> &[u8] {
        &self.types[..self.count]
    }

    fn valid_mask(&self) -> &[bool] {
        &self.valid_mask[..self.count]
    }

    fn weights_mut(&mut self) -> &mut [u8] {
        let count = self.count;
        &mut self.weights[..count]
    }

    fn postsynaptic_potentials_mut(&mut self) -> &mut [u8] {
        let count = self.count;
        &mut self.postsynaptic_potentials[..count]
    }

    fn valid_mask_mut(&mut self) -> &mut [bool] {
        let count = self.count;
        &mut self.valid_mask[..count]
    }

    fn count(&self) -> usize {
        self.count
    }

    fn capacity(&self) -> usize {
        N
    }

    fn add_synapse(
        &mut self,
        source: u32,
        target: u32,
        weight: u8,
        psp: u8,
        synapse_type: u8,
    ) -> Result<usize> {
        if self.count >= N {
            return Err(RuntimeError::CapacityExceeded {
                requested: self.count + 1,
                available: N,
            });
        }

        let idx = self.count;
        self.source_neurons[idx] = source;
        self.target_neurons[idx] = target;
        self.weights[idx] = weight;
        self.postsynaptic_potentials[idx] = psp;
        self.types[idx] = synapse_type;
        self.valid_mask[idx] = true;

        self.count += 1;
        Ok(idx)
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn add_synapses_batch(
        &mut self,
        sources: &[u32],
        targets: &[u32],
        weights: &[u8],
        psps: &[u8],
        types: &[u8],
    ) -> Result<()> {
        let n = sources.len();

        if self.count + n > N {
            return Err(RuntimeError::CapacityExceeded {
                requested: self.count + n,
                available: N,
            });
        }

        for i in 0..n {
            self.add_synapse(sources[i], targets[i], weights[i], psps[i], types[i])?;
        }

        Ok(())
    }

    fn remove_synapse(&mut self, idx: usize) -> Result<()> {
        if idx >= self.count {
            return Err(RuntimeError::CapacityExceeded {
                requested: idx,
                available: self.count,
            });
        }

        // Mark as invalid (don't shift array for performance)
        self.valid_mask[idx] = false;
        Ok(())
    }

    fn remove_synapses_from_sources(&mut self, source_neurons: &[u32]) -> Result<usize> {
        let mut removed = 0;
        for idx in 0..self.count {
            if self.valid_mask[idx] && source_neurons.contains(&self.source_neurons[idx]) {
                self.valid_mask[idx] = false;
                removed += 1;
            }
        }
        Ok(removed)
    }

    fn remove_synapses_between(&mut self, source: u32, target: u32) -> Result<usize> {
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

    fn update_weight(&mut self, idx: usize, new_weight: u8) -> Result<()> {
        if idx >= self.count || !self.valid_mask[idx] {
            return Err(RuntimeError::CapacityExceeded {
                requested: idx,
                available: self.count,
            });
        }

        self.weights[idx] = new_weight;
        Ok(())
    }

    fn valid_count(&self) -> usize {
        let mut count = 0;
        for idx in 0..self.count {
            if self.valid_mask[idx] {
                count += 1;
            }
        }
        count
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
        use feagi_npu_neural::SynapseType;
        let mut array = SynapseArray::<10>::new();
        assert!(array.add_synapse_simple(0, 1, 255, 255, SynapseType::Excitatory));
        assert_eq!(array.count, 1);
    }

    #[test]
    fn test_array_full() {
        use feagi_npu_neural::SynapseType;
        let mut array = SynapseArray::<2>::new();
        assert!(array.add_synapse_simple(0, 1, 255, 255, SynapseType::Excitatory));
        assert!(array
            .add_synapse(1, 2, 255, 255, SynapseType::Excitatory as u8)
            .is_ok());
        assert!(array
            .add_synapse(2, 3, 255, 255, SynapseType::Excitatory as u8)
            .is_err()); // Full
    }

    #[test]
    fn test_propagate() {
        let mut array = SynapseArray::<10>::new();
        let _ = array.add_synapse(0, 1, 255, 255, 0);
        let _ = array.add_synapse(0, 2, 128, 255, 0);

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
