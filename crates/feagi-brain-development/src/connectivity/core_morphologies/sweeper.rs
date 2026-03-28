// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Sweeper morphology implementation.

`sweeper` maps each source neuron to the next destination voxel in
X-major sweep order:
- +1 along X while available
- otherwise X resets to 0 and Y increments
- when Y is exhausted, Z increments and X,Y reset to 0
*/

use crate::types::{BduResult, Position};
use feagi_npu_neural::types::{NeuronId, SynapticPsp, SynapticWeight};
use feagi_npu_neural::SynapseType;

#[allow(clippy::too_many_arguments)]
pub fn apply_sweeper_morphology_with_dimensions(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
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

        let Some(dst_pos) = next_sweep_position(src_pos, dst_dimensions) else {
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
                    )
                    .is_ok()
            {
                synapse_count += 1;
            }
        }
    }

    Ok(synapse_count)
}

fn next_sweep_position(
    src_pos: Position,
    dst_dimensions: (usize, usize, usize),
) -> Option<Position> {
    let max_x = dst_dimensions.0 as u32;
    let max_y = dst_dimensions.1 as u32;
    let max_z = dst_dimensions.2 as u32;
    if max_x == 0 || max_y == 0 || max_z == 0 {
        return None;
    }

    // Clamp source position into destination bounds for deterministic behavior.
    let x = src_pos.0.min(max_x.saturating_sub(1));
    let y = src_pos.1.min(max_y.saturating_sub(1));
    let z = src_pos.2.min(max_z.saturating_sub(1));

    if x + 1 < max_x {
        Some((x + 1, y, z))
    } else if y + 1 < max_y {
        Some((0, y + 1, z))
    } else if z + 1 < max_z {
        Some((0, 0, z + 1))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sweeper_increments_x_first() {
        let next = next_sweep_position((1, 2, 3), (5, 6, 7));
        assert_eq!(next, Some((2, 2, 3)));
    }

    #[test]
    fn test_sweeper_rolls_to_next_y() {
        let next = next_sweep_position((4, 2, 3), (5, 6, 7));
        assert_eq!(next, Some((0, 3, 3)));
    }

    #[test]
    fn test_sweeper_rolls_to_next_z() {
        let next = next_sweep_position((4, 5, 3), (5, 6, 7));
        assert_eq!(next, Some((0, 0, 4)));
    }

    #[test]
    fn test_sweeper_ends_at_last_voxel() {
        let next = next_sweep_position((4, 5, 6), (5, 6, 7));
        assert_eq!(next, None);
    }
}
