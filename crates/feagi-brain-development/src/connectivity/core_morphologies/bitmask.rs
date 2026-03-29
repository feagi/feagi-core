// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Bitmask encoder/decoder morphology implementation.

Supports function-type morphologies:
- bitmask_encoder_x / bitmask_encoder_y / bitmask_encoder_z
- bitmask_decoder_x / bitmask_decoder_y / bitmask_decoder_z
*/

use crate::types::BduResult;
use feagi_npu_neural::types::{NeuronId, SynapticPsp, SynapticWeight};
use feagi_npu_neural::SynapseType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmaskAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmaskMode {
    Encoder,
    Decoder,
}

#[allow(clippy::too_many_arguments)]
pub fn apply_bitmask_morphology_with_dimensions(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
    src_dimensions: (usize, usize, usize),
    dst_dimensions: (usize, usize, usize),
    axis: BitmaskAxis,
    mode: BitmaskMode,
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

        let dst_positions =
            map_positions_for_bitmask(src_pos, src_dimensions, dst_dimensions, axis, mode);

        for dst_pos in dst_positions {
            // Note: Keep nested conditionals to maintain Rust 2021 compatibility.
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

fn map_positions_for_bitmask(
    src_pos: (u32, u32, u32),
    src_dimensions: (usize, usize, usize),
    dst_dimensions: (usize, usize, usize),
    axis: BitmaskAxis,
    mode: BitmaskMode,
) -> Vec<(u32, u32, u32)> {
    let src_axis_len = axis_len(src_dimensions, axis);
    let dst_axis_len = axis_len(dst_dimensions, axis);
    if src_axis_len == 0 || dst_axis_len == 0 {
        return Vec::new();
    }

    let clamped = (
        clamp_to_dim(src_pos.0, dst_dimensions.0),
        clamp_to_dim(src_pos.1, dst_dimensions.1),
        clamp_to_dim(src_pos.2, dst_dimensions.2),
    );

    match mode {
        BitmaskMode::Encoder => {
            let src_bit_index = axis_value(src_pos, axis) as usize;
            if src_bit_index >= src_axis_len {
                return Vec::new();
            }

            let mut out = Vec::new();
            for dst_axis_index in 0..dst_axis_len {
                if bit_is_set_msb(dst_axis_index as u32, src_bit_index, src_axis_len) {
                    out.push(compose_pos(axis, dst_axis_index as u32, clamped));
                }
            }
            out
        }
        BitmaskMode::Decoder => {
            let encoded_axis_value = axis_value(src_pos, axis);
            let mut out = Vec::new();
            for dst_bit_index in 0..dst_axis_len {
                if bit_is_set_msb(encoded_axis_value, dst_bit_index, dst_axis_len) {
                    out.push(compose_pos(axis, dst_bit_index as u32, clamped));
                }
            }
            out
        }
    }
}

#[inline]
fn axis_len(dim: (usize, usize, usize), axis: BitmaskAxis) -> usize {
    match axis {
        BitmaskAxis::X => dim.0,
        BitmaskAxis::Y => dim.1,
        BitmaskAxis::Z => dim.2,
    }
}

#[inline]
fn axis_value(pos: (u32, u32, u32), axis: BitmaskAxis) -> u32 {
    match axis {
        BitmaskAxis::X => pos.0,
        BitmaskAxis::Y => pos.1,
        BitmaskAxis::Z => pos.2,
    }
}

#[inline]
fn compose_pos(
    axis: BitmaskAxis,
    axis_value: u32,
    clamped_src_pos: (u32, u32, u32),
) -> (u32, u32, u32) {
    match axis {
        BitmaskAxis::X => (axis_value, clamped_src_pos.1, clamped_src_pos.2),
        BitmaskAxis::Y => (clamped_src_pos.0, axis_value, clamped_src_pos.2),
        BitmaskAxis::Z => (clamped_src_pos.0, clamped_src_pos.1, axis_value),
    }
}

#[inline]
fn clamp_to_dim(value: u32, dim_len: usize) -> u32 {
    if dim_len == 0 {
        return 0;
    }
    value.min((dim_len - 1) as u32)
}

#[inline]
fn bit_is_set_msb(value: u32, bit_index_from_msb: usize, bit_width: usize) -> bool {
    if bit_index_from_msb >= bit_width {
        return false;
    }
    let lsb_index = bit_width - 1 - bit_index_from_msb;
    if lsb_index >= u32::BITS as usize {
        return false;
    }
    (value & (1u32 << lsb_index)) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_x_uses_msb_convention() {
        // Source X index 1 with width 3 checks middle bit in 3-bit destination values.
        let out = map_positions_for_bitmask(
            (1, 2, 3),
            (3, 10, 10),
            (8, 10, 10),
            BitmaskAxis::X,
            BitmaskMode::Encoder,
        );

        assert_eq!(
            out,
            vec![(2, 2, 3), (3, 2, 3), (6, 2, 3), (7, 2, 3)],
            "Expected X positions where middle bit is set for 3-bit values"
        );
    }

    #[test]
    fn test_decoder_x_uses_msb_convention() {
        // Encoded value 5 is 0101 in width-4 bitspace => active dst bits at indices 1 and 3.
        let out = map_positions_for_bitmask(
            (5, 1, 1),
            (16, 10, 10),
            (4, 10, 10),
            BitmaskAxis::X,
            BitmaskMode::Decoder,
        );

        assert_eq!(out, vec![(1, 1, 1), (3, 1, 1)]);
    }
}
