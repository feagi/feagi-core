// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Fire structures for tracking neural activity
//!
//! Moved from feagi-types/src/fire_structures.rs (Phase 2c)

use super::ids::NeuronId;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Fire Candidate List (FCL) - neurons that might fire this burst
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct FireCandidateList {
    candidates: ahash::AHashMap<u32, f32>,
}

#[cfg(feature = "std")]
impl Default for FireCandidateList {
    fn default() -> Self {
        Self {
            candidates: ahash::AHashMap::with_capacity(100_000),
        }
    }
}

#[cfg(feature = "std")]
impl FireCandidateList {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn add_candidate(&mut self, neuron_id: NeuronId, potential: f32) {
        *self.candidates.entry(neuron_id.0).or_insert(0.0) += potential;
    }

    /// Add multiple candidates in batch (more efficient for large batches)
    /// Pre-aggregates contributions and reserves capacity to reduce reallocations
    pub fn add_candidates_batch(&mut self, candidates: &[(NeuronId, f32)]) {
        if candidates.is_empty() {
            return;
        }
        
        // Pre-allocate if needed (huge performance improvement for large batches)
        let estimated_unique = candidates.len() / 10; // Heuristic: ~10% unique neurons
        if estimated_unique > self.candidates.capacity() {
            self.candidates.reserve(estimated_unique);
        }
        
        // Direct insertion is actually fastest - batch method helps mainly with pre-allocation
        // Pre-aggregation doesn't help much because we still need to merge into main HashMap
        for &(neuron_id, potential) in candidates {
            *self.candidates.entry(neuron_id.0).or_insert(0.0) += potential;
        }
    }
    
    /// Reserve capacity for expected number of candidates (call before batch insertion)
    pub fn reserve(&mut self, capacity: usize) {
        self.candidates.reserve(capacity);
    }

    pub fn clear(&mut self) {
        self.candidates.clear();
    }

    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn get(&self, neuron_id: NeuronId) -> Option<f32> {
        self.candidates.get(&neuron_id.0).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (NeuronId, f32)> + '_ {
        self.candidates
            .iter()
            .map(|(&id, &pot)| (NeuronId(id), pot))
    }
}

/// Fire Queue (FQ) - neurons that actually fired this burst
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct FireQueue {
    neurons: Vec<NeuronId>,
}

#[cfg(feature = "std")]
impl Default for FireQueue {
    fn default() -> Self {
        Self {
            neurons: Vec::with_capacity(10_000),
        }
    }
}

#[cfg(feature = "std")]
impl FireQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, neuron_id: NeuronId) {
        self.neurons.push(neuron_id);
    }

    pub fn clear(&mut self) {
        self.neurons.clear();
    }

    pub fn len(&self) -> usize {
        self.neurons.len()
    }

    pub fn is_empty(&self) -> bool {
        self.neurons.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = NeuronId> + '_ {
        self.neurons.iter().copied()
    }

    pub fn as_slice(&self) -> &[NeuronId] {
        &self.neurons
    }
}

// Placeholder types for no_std (will be implemented with heapless or similar)
#[cfg(not(feature = "std"))]
pub struct FireCandidateList;

#[cfg(not(feature = "std"))]
pub struct FireQueue;
