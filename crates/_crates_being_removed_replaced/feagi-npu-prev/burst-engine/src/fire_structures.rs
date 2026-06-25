// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Fire Queue data structures for NPU processing
//!
//! These structures represent neurons that fired in the current burst.
//! Used for synaptic propagation and archiving to Fire Ledger.

use ahash::AHashMap;
use feagi_npu_neural::types::NeuronId;

/// Default / dense LIF / associative-memory fires tagged as driven by dense or associative LIF
/// (not pattern-only injection).
pub const FIRE_KIND_STDP_ELIGIBLE: u8 = 0;
/// Memory neuron fired via episodic pattern path (`inject_memory_neuron_to_fcl` default).
/// These spikes are **triggers for associative STDP**: they are archived to the main STDP
/// fire ledger like other memory fires; the episodic-only ledger is a subset view for
/// pattern machinery that needs episodic-tagged activity alone.
pub const FIRE_KIND_EPISODIC_MEMORY: u8 = 1;

/// A single neuron that fired in the current burst
#[derive(Debug, Clone)]
pub struct FiringNeuron {
    pub neuron_id: NeuronId,
    pub membrane_potential: f32,
    pub cortical_idx: u32, // Changed from CorticalID - NPU works with indices only
    pub x: u32,
    pub y: u32,
    pub z: u32,
    /// [`FIRE_KIND_STDP_ELIGIBLE`] vs [`FIRE_KIND_EPISODIC_MEMORY`] for memory-neuron semantics.
    pub fire_kind: u8,
}

impl Default for FiringNeuron {
    fn default() -> Self {
        Self {
            neuron_id: NeuronId(0),
            membrane_potential: 0.0,
            cortical_idx: 0,
            x: 0,
            y: 0,
            z: 0,
            fire_kind: FIRE_KIND_STDP_ELIGIBLE,
        }
    }
}

/// Fire Queue - neurons that fired in the current burst
/// Organized by cortical area for efficient processing
#[derive(Debug, Clone)]
pub struct FireQueue {
    /// Firing neurons grouped by cortical area
    pub neurons_by_area: AHashMap<u32, Vec<FiringNeuron>>,

    /// Total number of neurons across all areas
    total_count: usize,

    /// Timestep this queue represents
    pub timestep: u64,
}

impl FireQueue {
    /// Create a new empty Fire Queue
    pub fn new() -> Self {
        Self {
            neurons_by_area: AHashMap::new(),
            total_count: 0,
            timestep: 0,
        }
    }

    /// Add a firing neuron to the queue
    pub fn add_neuron(&mut self, neuron: FiringNeuron) {
        self.neurons_by_area
            .entry(neuron.cortical_idx)
            .or_default()
            .push(neuron);
        self.total_count += 1;
    }

    /// Get total number of fired neurons across all areas
    pub fn total_neurons(&self) -> usize {
        self.total_count
    }

    /// Get neurons for a specific cortical area
    pub fn get_area_neurons(&self, cortical_idx: u32) -> Option<&Vec<FiringNeuron>> {
        self.neurons_by_area.get(&cortical_idx)
    }

    /// Clear the queue
    pub fn clear(&mut self) {
        self.neurons_by_area.clear();
        self.total_count = 0;
    }

    /// Set timestep
    pub fn set_timestep(&mut self, timestep: u64) {
        self.timestep = timestep;
    }

    /// Get all neuron IDs from all areas
    pub fn get_all_neuron_ids(&self) -> Vec<NeuronId> {
        let mut ids = Vec::with_capacity(self.total_count);
        for neurons in self.neurons_by_area.values() {
            ids.extend(neurons.iter().map(|n| n.neuron_id));
        }
        ids
    }

    /// Check if fire queue is empty
    pub fn is_empty(&self) -> bool {
        self.total_count == 0
    }

    /// Remove all neurons from a specific cortical area
    /// Returns the number of neurons removed
    pub fn remove_cortical_area(&mut self, cortical_idx: u32) -> usize {
        if let Some(neurons) = self.neurons_by_area.remove(&cortical_idx) {
            let count = neurons.len();
            self.total_count = self.total_count.saturating_sub(count);
            count
        } else {
            0
        }
    }

    /// Clone of this queue for **STDP** FireLedger: includes **all** fires, including
    /// [`FIRE_KIND_EPISODIC_MEMORY`] pattern-injection spikes, so associative STDP can match
    /// source/destination memory co-activation and create or update synapses.
    pub fn clone_for_stdp_fire_ledger(&self) -> Self {
        let mut out = FireQueue::new();
        out.set_timestep(self.timestep);
        for neurons in self.neurons_by_area.values() {
            for n in neurons {
                out.add_neuron(n.clone());
            }
        }
        out
    }

    /// Clone for **episodic memory** FireLedger: only episodic-tagged memory neuron fires.
    pub fn clone_for_episodic_memory_fire_ledger(&self, memory_neuron_id_start: u32) -> Self {
        let mut out = FireQueue::new();
        out.set_timestep(self.timestep);
        for neurons in self.neurons_by_area.values() {
            for n in neurons {
                let keep = n.neuron_id.0 >= memory_neuron_id_start
                    && n.fire_kind == FIRE_KIND_EPISODIC_MEMORY;
                if keep {
                    out.add_neuron(n.clone());
                }
            }
        }
        out
    }
}

impl Default for FireQueue {
    fn default() -> Self {
        Self::new()
    }
}
