// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Tile morphology implementation.

Behavior:
- If source is larger than destination (by voxel count), destination dimensions
  define the tile unit and source is folded tile-wise into destination.
- Otherwise, source dimensions define the tile unit and source is replicated
  tile-wise into destination.
*/

use crate::types::{BduResult, Position};
use feagi_npu_neural::types::{NeuronId, SynapticPsp, SynapticWeight};
use feagi_npu_neural::SynapseType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TileMode {
    FoldIntoDestination,
    ReplicateIntoDestination,
}

#[allow(clippy::too_many_arguments)]
pub fn apply_tile_morphology_with_dimensions(
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

    if src_dimensions.0 == 0
        || src_dimensions.1 == 0
        || src_dimensions.2 == 0
        || dst_dimensions.0 == 0
        || dst_dimensions.1 == 0
        || dst_dimensions.2 == 0
    {
        return Ok(0);
    }

    let src_count = src_dimensions.0 * src_dimensions.1 * src_dimensions.2;
    let dst_count = dst_dimensions.0 * dst_dimensions.1 * dst_dimensions.2;
    let mode = if src_count > dst_count {
        TileMode::FoldIntoDestination
    } else {
        TileMode::ReplicateIntoDestination
    };

    let mut rng = get_rng();
    let src_neurons = npu.get_neurons_in_cortical_area(src_area_id);
    if src_neurons.is_empty() {
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
        let dst_positions = tile_destinations(src_pos, src_dimensions, dst_dimensions, mode);
        for dst_pos in dst_positions {
            // Keep nested conditions for Rust 2021 compatibility.
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

fn tile_destinations(
    src_pos: Position,
    src_dimensions: (usize, usize, usize),
    dst_dimensions: (usize, usize, usize),
    mode: TileMode,
) -> Vec<Position> {
    match mode {
        TileMode::FoldIntoDestination => {
            let dx = dst_dimensions.0 as u32;
            let dy = dst_dimensions.1 as u32;
            let dz = dst_dimensions.2 as u32;
            vec![(src_pos.0 % dx, src_pos.1 % dy, src_pos.2 % dz)]
        }
        TileMode::ReplicateIntoDestination => {
            let sx = src_dimensions.0 as u32;
            let sy = src_dimensions.1 as u32;
            let sz = src_dimensions.2 as u32;
            let dx = dst_dimensions.0 as u32;
            let dy = dst_dimensions.1 as u32;
            let dz = dst_dimensions.2 as u32;

            if src_pos.0 >= sx || src_pos.1 >= sy || src_pos.2 >= sz {
                return Vec::new();
            }

            let tiles_x = dx.div_ceil(sx);
            let tiles_y = dy.div_ceil(sy);
            let tiles_z = dz.div_ceil(sz);

            let mut out = Vec::new();
            for tx in 0..tiles_x {
                for ty in 0..tiles_y {
                    for tz in 0..tiles_z {
                        let x = tx * sx + src_pos.0;
                        let y = ty * sy + src_pos.1;
                        let z = tz * sz + src_pos.2;
                        if x < dx && y < dy && z < dz {
                            out.push((x, y, z));
                        }
                    }
                }
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_fold_uses_destination_tile_unit() {
        let out = tile_destinations(
            (3, 0, 0),
            (4, 1, 1),
            (2, 1, 1),
            TileMode::FoldIntoDestination,
        );
        assert_eq!(out, vec![(1, 0, 0)]);
    }

    #[test]
    fn test_tile_replicate_uses_source_tile_unit() {
        let out = tile_destinations(
            (1, 0, 0),
            (2, 1, 1),
            (5, 1, 1),
            TileMode::ReplicateIntoDestination,
        );
        assert_eq!(out, vec![(1, 0, 0), (3, 0, 0)]);
    }
}
