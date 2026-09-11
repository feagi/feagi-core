// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Shared helper functions for core morphologies.
*/

use crate::types::BduResult;

/// Calculate area dimensions by finding max coordinates
/// PERFORMANCE: This calls get_neurons_in_cortical_area which is expensive on cache miss
pub fn calculate_area_dimensions(
    npu: &feagi_npu_burst_engine::DynamicNPU,
    area_id: u32,
) -> (usize, usize, usize) {
    calculate_area_dimensions_without_scanning(npu, area_id).unwrap_or((0, 0, 0))
}

/// Calculate dimensions without scanning all neurons
///
/// CRITICAL: This MUST NOT call get_neurons_in_cortical_area.
/// Dimensions should be stored in ConnectomeManager when areas are created.
/// This is a temporary fallback that will error if dimensions aren't available.
#[allow(unused_variables)]
pub fn calculate_area_dimensions_without_scanning(
    npu: &feagi_npu_burst_engine::DynamicNPU,
    area_id: u32,
) -> BduResult<(usize, usize, usize)> {
    let neuron_ids = npu.get_neurons_in_cortical_area(area_id);
    if neuron_ids.is_empty() {
        return Err(crate::types::BduError::Internal(format!(
            "Area dimensions not available for area {} (no neurons present)",
            area_id
        )));
    }

    let (mut max_x, mut max_y, mut max_z) = (0u32, 0u32, 0u32);
    for neuron_id in neuron_ids {
        if let Some((x, y, z)) = npu.get_neuron_coordinates(neuron_id) {
            max_x = max_x.max(x);
            max_y = max_y.max(y);
            max_z = max_z.max(z);
        }
    }

    Ok((
        (max_x + 1) as usize,
        (max_y + 1) as usize,
        (max_z + 1) as usize,
    ))
}
