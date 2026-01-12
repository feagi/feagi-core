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
    // TODO: Get dimensions from ConnectomeManager.cortical_areas[area_id].dimensions
    // For now, return error - dimensions must be provided by caller
    // This ensures we NEVER call get_neurons_in_cortical_area during synaptogenesis
    Err(crate::types::BduError::Internal(
        format!("Area dimensions not available for area {}. Dimensions must be stored in ConnectomeManager when areas are created.", area_id)
    ))
}
