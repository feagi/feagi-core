// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Pattern matching morphology implementation.

Creates synapses based on pattern matching between source and destination areas.
*/

use crate::connectivity::core_morphologies::common::calculate_area_dimensions;
use crate::connectivity::rules::match_patterns_batch;
use crate::connectivity::rules::patterns::Pattern3D;
use crate::types::BduResult;
use feagi_npu_neural::types::{NeuronId, SynapticConductance, SynapticWeight};
use feagi_npu_neural::SynapseType;

/// Apply pattern matching morphology directly on NPU
#[allow(clippy::too_many_arguments)]
pub fn apply_patterns_morphology(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
    patterns: Vec<(Pattern3D, Pattern3D)>,
    weight: u8,
    conductance: u8,
    synapse_attractivity: u8,
    synapse_type: SynapseType,
) -> BduResult<u32> {
    use crate::rng::get_rng;
    use rand::Rng;
    let mut rng = get_rng();

    if patterns.is_empty() {
        return Ok(0);
    }

    let src_neurons = npu.get_neurons_in_cortical_area(src_area_id);
    if src_neurons.is_empty() {
        return Ok(0);
    }

    let src_dimensions = calculate_area_dimensions(npu, src_area_id);
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

        // Match patterns (Rust computation)
        let dst_positions =
            match_patterns_batch(src_pos, &patterns, src_dimensions, dst_dimensions);

        for dst_pos in dst_positions {
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
                            synapse_type,
                        )
                        .is_ok()
                {
                    synapse_count += 1;
                }
            }
        }
    }

    Ok(synapse_count)
}
