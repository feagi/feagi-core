// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # CPU Backend
//!
//! SIMD-optimized CPU backend using existing neural_dynamics and synaptic_propagation modules.
//! This wraps the current high-performance Rust implementation.

use super::ComputeBackend;
use crate::neural_dynamics;
use feagi_npu_neural::models::{LIFModel, NeuronModel};
use feagi_npu_neural::types::*;
use feagi_npu_runtime::{NeuronStorage, SynapseStorage};

/// CPU backend with SIMD optimization (current implementation)
pub struct CPUBackend {
    /// Backend name for logging
    name: String,

    /// Neuron model for computational dynamics
    neuron_model: LIFModel,
}

impl CPUBackend {
    /// Create a new CPU backend with LIF neuron model
    pub fn new() -> Self {
        Self::new_lif()
    }

    /// Create a new CPU backend with LIF neuron model (explicit)
    pub fn new_lif() -> Self {
        Self {
            name: "CPU (SIMD) - LIF".to_string(),
            neuron_model: LIFModel::new(),
        }
    }
}

impl Default for CPUBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: NeuralValue, N: NeuronStorage<Value = T>, S: SynapseStorage> ComputeBackend<T, N, S>
    for CPUBackend
{
    fn backend_name(&self) -> &str {
        &self.name
    }

    fn process_synaptic_propagation(
        &mut self,
        fired_neurons: &[u32],
        synapse_storage: &S,
        fcl: &mut FireCandidateList,
    ) -> Result<usize> {
        // FCL-aware: Accumulate synaptic contributions into FCL
        let mut synapse_count = 0;

        // Iterate through all synapses (source_index not in trait)
        // TODO: Add source_index to SynapseStorage trait or build on-the-fly
        for syn_idx in 0..synapse_storage.count() {
            let source_id = synapse_storage.source_neurons()[syn_idx];
            if fired_neurons.contains(&source_id) {
                if !synapse_storage.valid_mask()[syn_idx] {
                    continue;
                }

                let target_id = synapse_storage.target_neurons()[syn_idx];
                // Canonical synaptic units: u8 (0..255) stored in synapse arrays.
                // We use direct cast to f32 (NO /255 normalization) to match the rest of FEAGI.
                let weight = synapse_storage.weights()[syn_idx] as f32;
                let psp = synapse_storage.postsynaptic_potentials()[syn_idx] as f32;
                let synapse_type = if synapse_storage.types()[syn_idx] == 0 {
                    SynapseType::Excitatory
                } else {
                    SynapseType::Inhibitory
                };

                // ✅ Use neuron model trait (LIF formula)
                // Result range: -65,025.0 to +65,025.0 (255 × 255) for excitatory/inhibitory.
                let contribution =
                    self.neuron_model
                        .compute_synaptic_contribution(weight, psp, synapse_type);

                // Accumulate into FCL
                fcl.add_candidate(NeuronId(target_id), contribution);
                synapse_count += 1;
            }
        }

        Ok(synapse_count)
    }

    fn process_neural_dynamics(
        &mut self,
        fcl: &FireCandidateList,
        neuron_storage: &mut N,
        burst_count: u64,
    ) -> Result<(Vec<u32>, usize, usize)> {
        // FCL-aware: Process only FCL neurons (existing neural_dynamics already supports this!)
        let result =
            neural_dynamics::process_neural_dynamics(fcl, None, neuron_storage, burst_count)?;

        // Extract neuron IDs from fire queue
        let fired_neurons: Vec<u32> = result
            .fire_queue
            .get_all_neuron_ids()
            .iter()
            .map(|n| n.0)
            .collect();

        Ok((
            fired_neurons,
            result.neurons_processed,
            result.neurons_in_refractory,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_backend_creation() {
        use feagi_npu_runtime::{StdNeuronArray, StdSynapseArray};
        let backend = CPUBackend::new();
        assert_eq!(
            <CPUBackend as ComputeBackend<f32, StdNeuronArray<f32>, StdSynapseArray>>::backend_name(
                &backend
            ),
            "CPU (SIMD) - LIF"
        );
    }

    #[test]
    fn test_cpu_backend_synaptic_propagation() {
        use feagi_npu_neural::synapse::{compute_synaptic_contribution, SynapseType as NeuralSynapseType};
        let mut backend = CPUBackend::new();

        // Create minimal test data
        let fired_neurons = vec![1];
        let mut synapse_storage = StdSynapseArray::new(4);
        synapse_storage.add_synapse_simple(1, 2, 2, 3, SynapseType::Excitatory); // 2×3 = 6
        let mut fcl = FireCandidateList::new();

        // Should not panic
        use feagi_npu_runtime::{StdNeuronArray, StdSynapseArray};
        let result = <CPUBackend as ComputeBackend<f32, StdNeuronArray<f32>, StdSynapseArray>>::process_synaptic_propagation(
            &mut backend, &fired_neurons, &synapse_storage, &mut fcl
        );

        assert!(result.is_ok());
        assert_eq!(fcl.get(NeuronId(2)), Some(6.0));
        assert_eq!(
            fcl.get(NeuronId(2)),
            Some(compute_synaptic_contribution(2, 3, NeuralSynapseType::Excitatory))
        );
    }
}
