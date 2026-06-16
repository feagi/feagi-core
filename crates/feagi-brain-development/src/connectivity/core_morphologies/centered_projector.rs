// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Centered projector morphology implementation.

`centered_projector` maps source voxels to destination voxels by aligning
area centers and preserving relative offsets:

- source center -> destination center
- surrounding source voxels -> corresponding destination offsets
- out-of-bounds destination coordinates are dropped
*/

use crate::types::{BduResult, Position};
use feagi_npu_neural::types::{NeuronId, SynapticPsp, SynapticWeight};
use feagi_npu_neural::SynapseType;

#[allow(clippy::too_many_arguments)]
pub fn apply_centered_projector_morphology_with_dimensions(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
    src_dimensions: (usize, usize, usize),
    dst_dimensions: (usize, usize, usize),
    weight: f32,
    psp: f32,
    synapse_attractivity: u8,
    synapse_type: SynapseType,
    delay_bursts: u8,
) -> BduResult<u32> {
    use crate::rng::get_rng;
    use rand::Rng;

    let mut rng = get_rng();
    let src_neurons = npu.get_neurons_in_cortical_area(src_area_id);
    if src_neurons.is_empty() {
        return Ok(0);
    }

    if src_dimensions.0 == 0 || src_dimensions.1 == 0 || src_dimensions.2 == 0 {
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
        let Some(dst_pos) = calculate_centered_projection(src_pos, src_dimensions, dst_dimensions)
        else {
            continue;
        };

        // Note: Keep nested if for Rust 2021 compatibility.
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
                        delay_bursts,
                    )
                    .is_ok()
            {
                synapse_count += 1;
            }
        }
    }

    Ok(synapse_count)
}

fn calculate_centered_projection(
    src_pos: Position,
    src_dimensions: (usize, usize, usize),
    dst_dimensions: (usize, usize, usize),
) -> Option<Position> {
    let src_dims = [
        src_dimensions.0 as i64,
        src_dimensions.1 as i64,
        src_dimensions.2 as i64,
    ];
    let dst_dims = [
        dst_dimensions.0 as i64,
        dst_dimensions.1 as i64,
        dst_dimensions.2 as i64,
    ];
    let src_coords = [src_pos.0 as i64, src_pos.1 as i64, src_pos.2 as i64];

    let mut mapped = [0i64; 3];
    for axis in 0..3 {
        if src_dims[axis] <= 0 || dst_dims[axis] <= 0 {
            return None;
        }
        if src_coords[axis] < 0 || src_coords[axis] >= src_dims[axis] {
            return None;
        }

        // For even sizes this chooses the lower-center voxel (e.g., 4 -> 1).
        let src_center = (src_dims[axis] - 1) / 2;
        let dst_center = (dst_dims[axis] - 1) / 2;
        let offset = src_coords[axis] - src_center;
        let dst_coord = dst_center + offset;
        if dst_coord < 0 || dst_coord >= dst_dims[axis] {
            return None;
        }
        mapped[axis] = dst_coord;
    }

    Some((mapped[0] as u32, mapped[1] as u32, mapped[2] as u32))
}

#[cfg(test)]
mod tests {
    use super::calculate_centered_projection;

    #[test]
    fn test_same_dimensions_identity_mapping() {
        let mapped = calculate_centered_projection((3, 2, 1), (8, 5, 3), (8, 5, 3));
        assert_eq!(mapped, Some((3, 2, 1)));
    }

    #[test]
    fn test_center_maps_to_center() {
        let mapped = calculate_centered_projection((2, 2, 0), (5, 5, 1), (9, 7, 1));
        assert_eq!(mapped, Some((4, 3, 0)));
    }

    #[test]
    fn test_source_larger_than_destination_drops_out_of_bounds() {
        let mapped = calculate_centered_projection((4, 4, 0), (5, 5, 1), (3, 3, 1));
        assert_eq!(mapped, None);
    }

    #[test]
    fn test_source_smaller_than_destination_keeps_relative_offset() {
        let mapped = calculate_centered_projection((0, 0, 0), (3, 3, 1), (7, 7, 1));
        assert_eq!(mapped, Some((2, 2, 0)));
    }

    #[test]
    fn test_out_of_bounds_source_position_returns_none() {
        let mapped = calculate_centered_projection((3, 0, 0), (3, 3, 1), (3, 3, 1));
        assert_eq!(mapped, None);
    }
}
