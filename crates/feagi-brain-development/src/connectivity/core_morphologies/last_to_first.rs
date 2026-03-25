// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Last-to-first morphology implementation.

Connects only the highest source voxel (max x, max y, max z)
to destination voxel (0, 0, 0).
*/

use crate::types::BduResult;
use feagi_npu_neural::types::{NeuronId, SynapticPsp, SynapticWeight};
use feagi_npu_neural::SynapseType;

#[allow(clippy::too_many_arguments)]
pub fn apply_last_to_first_morphology_with_dimensions(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
    src_dimensions: (usize, usize, usize),
    dst_dimensions: (usize, usize, usize),
    weight: u8,
    psp: u8,
    synapse_attractivity: u8,
    synapse_type: SynapseType,
) -> BduResult<u32> {
    use crate::rng::get_rng;
    use rand::Rng;

    let mut rng = get_rng();

    if src_dimensions.0 == 0
        || src_dimensions.1 == 0
        || src_dimensions.2 == 0
        || dst_dimensions.0 == 0
        || dst_dimensions.1 == 0
        || dst_dimensions.2 == 0
    {
        return Ok(0);
    }

    let src_last = (
        (src_dimensions.0 - 1) as u32,
        (src_dimensions.1 - 1) as u32,
        (src_dimensions.2 - 1) as u32,
    );
    let dst_first = (0u32, 0u32, 0u32);

    let mut dst_first_nid: Option<u32> = None;
    for dst_nid in npu.get_neurons_in_cortical_area(dst_area_id) {
        if npu.get_neuron_coordinates(dst_nid) == Some(dst_first) {
            dst_first_nid = Some(dst_nid);
            break;
        }
    }
    let Some(dst_nid) = dst_first_nid else {
        return Ok(0);
    };

    let mut synapse_count = 0u32;
    for src_nid in npu.get_neurons_in_cortical_area(src_area_id) {
        if npu.get_neuron_coordinates(src_nid) != Some(src_last) {
            continue;
        }
        if rng.gen_range(0..100) < synapse_attractivity
            && npu
                .add_synapse(
                    NeuronId(src_nid),
                    NeuronId(dst_nid),
                    SynapticWeight(weight),
                    SynapticPsp(psp),
                    synapse_type,
                )
                .is_ok()
        {
            synapse_count += 1;
        }
    }

    Ok(synapse_count)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_last_voxel_is_max_index() {
        let src_dimensions = (5usize, 4usize, 3usize);
        let src_last = (
            (src_dimensions.0 - 1) as u32,
            (src_dimensions.1 - 1) as u32,
            (src_dimensions.2 - 1) as u32,
        );
        assert_eq!(src_last, (4, 3, 2));
    }
}
