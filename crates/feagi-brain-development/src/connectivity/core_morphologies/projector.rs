// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Projector morphology implementation.

Maps neurons from source to destination areas while maintaining spatial topology.
*/

use crate::connectivity::core_morphologies::common::calculate_area_dimensions;
use crate::connectivity::rules::syn_projector;
use crate::types::BduResult;
use feagi_npu_neural::types::{NeuronId, SynapticConductance, SynapticWeight};
use feagi_npu_neural::SynapseType;

/// Apply projector morphology directly on NPU
///
/// # Arguments
/// * `npu` - Mutable reference to NPU (for querying neurons and creating synapses)
/// * `src_area_id` - Source cortical area ID
/// * `dst_area_id` - Destination cortical area ID
/// * `transpose` - Optional axis transposition
/// * `project_last_layer_of` - Optional axis to project from last layer
/// * `weight` - Synapse weight (0-255)
/// * `conductance` - Synapse conductance
/// * `synapse_attractivity` - Probability (0-100) of creating synapse when match found
///
/// # Returns
/// Number of synapses created
#[allow(clippy::too_many_arguments)]
pub fn apply_projector_morphology(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
    transpose: Option<(usize, usize, usize)>,
    project_last_layer_of: Option<usize>,
    weight: u8,
    conductance: u8,
    synapse_attractivity: u8,
) -> BduResult<u32> {
    // Calculate dimensions by finding max coordinates in each area
    // NOTE: This is a fallback - callers should prefer passing dimensions directly
    let src_dimensions = calculate_area_dimensions(npu, src_area_id);
    let dst_dimensions = calculate_area_dimensions(npu, dst_area_id);
    apply_projector_morphology_with_dimensions(
        npu,
        src_area_id,
        dst_area_id,
        src_dimensions,
        dst_dimensions,
        transpose,
        project_last_layer_of,
        weight,
        conductance,
        synapse_attractivity,
    )
}

/// Apply projector morphology directly on NPU with explicit dimensions
///
/// # Arguments
/// * `npu` - Mutable reference to NPU (for querying neurons and creating synapses)
/// * `src_area_id` - Source cortical area ID
/// * `dst_area_id` - Destination cortical area ID
/// * `src_dimensions` - Source area dimensions (width, height, depth)
/// * `dst_dimensions` - Destination area dimensions (width, height, depth)
/// * `transpose` - Optional axis transposition
/// * `project_last_layer_of` - Optional axis to project from last layer
/// * `weight` - Synapse weight (0-255)
/// * `conductance` - Synapse conductance
/// * `synapse_attractivity` - Probability (0-100) of creating synapse when match found
///
/// # Returns
/// Number of synapses created
#[allow(clippy::too_many_arguments)]
pub fn apply_projector_morphology_with_dimensions(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
    src_dimensions: (usize, usize, usize),
    dst_dimensions: (usize, usize, usize),
    transpose: Option<(usize, usize, usize)>,
    project_last_layer_of: Option<usize>,
    weight: u8,
    conductance: u8,
    synapse_attractivity: u8,
) -> BduResult<u32> {
    use crate::rng::get_rng;
    use rand::Rng;
    let mut rng = get_rng();

    // Query source neurons from NPU (zero copy - just iteration)
    let src_neurons = npu.get_neurons_in_cortical_area(src_area_id);
    if src_neurons.is_empty() {
        return Ok(0);
    }

    // Build destination position-to-neuron map (O(N) once)
    let mut dst_pos_map = std::collections::HashMap::new();
    for dst_nid in npu.get_neurons_in_cortical_area(dst_area_id) {
        if let Some(coords) = npu.get_neuron_coordinates(dst_nid) {
            dst_pos_map.insert(coords, dst_nid);
        }
    }

    let mut synapse_count = 0u32;

    // Process each source neuron
    for src_nid in src_neurons {
        let Some(src_pos) = npu.get_neuron_coordinates(src_nid) else {
            continue; // Skip if neuron not found
        };

        // Apply projector morphology (Rust computation)
        let dst_positions = syn_projector(
            "",
            "",
            src_nid as u64,
            src_dimensions,
            dst_dimensions,
            src_pos,
            transpose,
            project_last_layer_of,
        )?;

        // Create synapses for matched positions
        for dst_pos in dst_positions {
            if let Some(&dst_nid) = dst_pos_map.get(&dst_pos) {
                // Apply synapse attractivity (stochastic filtering)
                if rng.gen_range(0..100) < synapse_attractivity {
                    // Create synapse directly in NPU
                    if npu
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

