// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Vector offset morphology implementation.

Creates synapses based on vector offsets from source neurons.
*/

use crate::connectivity::core_morphologies::common::calculate_area_dimensions;
use crate::connectivity::rules::apply_vector_offset;
use crate::types::BduResult;
use feagi_npu_neural::types::{NeuronId, SynapticConductance, SynapticWeight};
use feagi_npu_neural::SynapseType;

/// Apply vector offset morphology directly on NPU
pub fn apply_vectors_morphology(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
    vectors: Vec<(i32, i32, i32)>,
    weight: u8,
    conductance: u8,
    synapse_attractivity: u8,
) -> BduResult<u32> {
    use crate::rng::get_rng;
    use rand::Rng;
    let mut rng = get_rng();

    if vectors.is_empty() {
        return Ok(0);
    }

    let src_neurons = npu.get_neurons_in_cortical_area(src_area_id);
    if src_neurons.is_empty() {
        return Ok(0);
    }

    let dst_dimensions = calculate_area_dimensions(npu, dst_area_id);

    let mut dst_pos_map = std::collections::HashMap::new();
    for dst_nid in npu.get_neurons_in_cortical_area(dst_area_id) {
        if let Some(coords) = npu.get_neuron_coordinates(dst_nid) {
            dst_pos_map.insert(coords, dst_nid);
        }
    }

    let mut synapse_count = 0u32;

    for src_nid in src_neurons {
        let Some(src_pos) = npu.get_neuron_coordinates(src_nid) else {
            continue;
        };

        // Apply all vectors
        for &vector in &vectors {
            if let Ok(dst_pos) = apply_vector_offset(src_pos, vector, 1.0, dst_dimensions) {
                // Note: Cannot collapse this if in Rust 2021 (let chains require Rust 2024)
                #[allow(clippy::collapsible_if)]
                if let Some(&dst_nid) = dst_pos_map.get(&dst_pos) {
                    if rng.gen_range(0..100) < synapse_attractivity
                        && npu
                            .add_synapse(
                                NeuronId(src_nid),
                                NeuronId(dst_nid),
                                SynapticWeight(weight),
                                SynapticConductance(conductance),
                                SynapseType::Excitatory,
                            )
                            .is_ok()
                    {
                        synapse_count += 1;
                    }
                }
            }
        }
    }

    Ok(synapse_count)
}

