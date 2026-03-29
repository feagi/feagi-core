// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Rotator-Z morphology implementation.

For each source neuron, creates candidate destination coordinates across all
destination z-layers, where each z-layer is an incremental XY rotation in
[-90, +90] degrees around the destination XY center.
*/

use crate::connectivity::rules::syn_rotator_z;
use crate::types::BduResult;
use feagi_npu_neural::types::{NeuronId, SynapticPsp, SynapticWeight};
use feagi_npu_neural::SynapseType;

#[allow(clippy::too_many_arguments)]
pub fn apply_rotator_z_morphology_with_dimensions(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
    src_dimensions: (usize, usize, usize),
    dst_dimensions: (usize, usize, usize),
    weight: f32,
    psp: f32,
    synapse_attractivity: u8,
    synapse_type: SynapseType,
) -> BduResult<u32> {
    use crate::rng::get_rng;
    use rand::Rng;

    let mut rng = get_rng();
    let src_neurons = npu.get_neurons_in_cortical_area(src_area_id);
    if src_neurons.is_empty() {
        return Ok(0);
    }
    if dst_dimensions.0 == 0 || dst_dimensions.1 == 0 || dst_dimensions.2 == 0 {
        return Ok(0);
    }

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

        let dst_positions = syn_rotator_z(src_pos, src_dimensions, dst_dimensions)?;

        for dst_pos in dst_positions {
            // Keep nested if statements for Rust 2021 compatibility.
            #[allow(clippy::collapsible_if)]
            if let Some(&dst_nid) = dst_pos_map.get(&dst_pos) {
                if rng.gen_range(0..100) < synapse_attractivity
                    && npu
                        .add_synapse(
                            NeuronId(src_nid),
                            NeuronId(dst_nid),
                            SynapticWeight(weight),
                            SynapticPsp(psp),
                            synapse_type,
                            0,
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
