// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
NPU-Native Synaptogenesis - Zero-Copy Morphology Application

This module implements synaptogenesis that operates directly on the NPU,
eliminating the need for Python to pass neuron lists across the FFI boundary.

## Architecture

```text
Python: Call rust_apply_projector(npu, src_area_id, dst_area_id, params)
           ↓
Rust:   1. Query neurons from NPU (zero copy)
        2. Apply morphology rules (SIMD optimized)
        3. Create synapses directly in NPU
        4. Return synapse count
           ↓
Python: Receives u32 (synapse count only)
```

## Performance Impact

- **Eliminates:** 6+ seconds of FFI overhead per area pair
- **Eliminates:** Python list building and marshaling
- **Enables:** SIMD-optimized morphology application
- **Result:** ~50+ second improvement for typical genomes

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::connectivity::rules::patterns::Pattern3D;
use crate::connectivity::rules::{
    apply_vector_offset, match_patterns_batch, syn_block_connection, syn_expander, syn_projector,
};
use crate::types::BduResult;
use feagi_npu_neural::types::{NeuronId, SynapticConductance, SynapticWeight};
use feagi_npu_neural::SynapseType;
use std::sync::Arc;
use tracing::info;
// use feagi_npu_burst_engine::npu::RustNPU; // Now using DynamicNPU

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
    use crate::rng::get_rng;
    use rand::Rng;
    let mut rng = get_rng();

    // Query source neurons from NPU (zero copy - just iteration)
    let src_neurons = npu.get_neurons_in_cortical_area(src_area_id);
    if src_neurons.is_empty() {
        return Ok(0);
    }

    // Calculate dimensions by finding max coordinates in each area
    let src_dimensions = calculate_area_dimensions(npu, src_area_id);
    let dst_dimensions = calculate_area_dimensions(npu, dst_area_id);

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

/// Apply block connection morphology with batched processing (releases NPU lock between batches)
/// 
/// This version is optimized for large neuron counts (>100k) and releases the NPU lock
/// between batches to allow the burst loop to run, preventing 4-17 second blocking.
pub fn apply_block_connection_morphology_batched(
    npu: &Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
    src_area_id: u32,
    dst_area_id: u32,
    scaling_factor: u32,
    weight: u8,
    conductance: u8,
    synapse_attractivity: u8,
) -> BduResult<u32> {
    use crate::rng::get_rng;
    use rand::Rng;
    let mut rng = get_rng();
    
    const BATCH_SIZE: usize = 50_000; // Process 50k synapses per batch
    
    // Step 1: Query all neuron data (with lock)
    let (src_neurons, src_dimensions, dst_dimensions, dst_pos_map) = {
        let mut npu_lock = npu.lock().map_err(|e| {
            crate::types::BduError::Internal(format!("Failed to lock NPU: {}", e))
        })?;
        
        let src_neurons = npu_lock.get_neurons_in_cortical_area(src_area_id);
        if src_neurons.is_empty() {
            return Ok(0);
        }
        
        let src_dimensions = calculate_area_dimensions(&*npu_lock, src_area_id);
        let dst_dimensions = calculate_area_dimensions(&*npu_lock, dst_area_id);
        
        let mut dst_pos_map = std::collections::HashMap::new();
        for dst_nid in npu_lock.get_neurons_in_cortical_area(dst_area_id) {
            if let Some(coords) = npu_lock.get_neuron_coordinates(dst_nid) {
                dst_pos_map.insert(coords, dst_nid);
            }
        }
        
        // Collect source neuron positions (need to query while lock is held)
        let mut src_neuron_data = Vec::new();
        for src_nid in &src_neurons {
            if let Some(src_pos) = npu_lock.get_neuron_coordinates(*src_nid) {
                src_neuron_data.push((*src_nid, src_pos));
            }
        }
        
        (src_neuron_data, src_dimensions, dst_dimensions, dst_pos_map)
    };
    
    // Step 2: Pre-compute all synapse operations (NO LOCK - this is fast)
    let mut synapse_ops: Vec<(u32, u32)> = Vec::new();
    for (src_nid, src_pos) in src_neurons {
        let dst_pos = syn_block_connection(
            "",
            "",
            src_pos,
            src_dimensions,
            dst_dimensions,
            scaling_factor,
        )?;

        if let Some(&dst_nid) = dst_pos_map.get(&dst_pos) {
            if rng.gen_range(0..100) < synapse_attractivity {
                synapse_ops.push((src_nid, dst_nid));
            }
        }
    }
    
    if synapse_ops.is_empty() {
        return Ok(0);
    }
    
    let total_synapses = synapse_ops.len();
    if total_synapses > BATCH_SIZE {
        info!(
            target: "feagi-bdu",
            "Batching synapse creation: {} synapses in batches of {} (releasing NPU lock between batches)",
            total_synapses, BATCH_SIZE
        );
    }
    
    // Step 3: Create synapses in batches, releasing lock between batches
    let mut synapse_count = 0u32;
    for (batch_idx, batch) in synapse_ops.chunks(BATCH_SIZE).enumerate() {
        // Re-acquire NPU lock for this batch
        let mut npu_lock = npu.lock().map_err(|e| {
            crate::types::BduError::Internal(format!("Failed to lock NPU for batch {}: {}", batch_idx, e))
        })?;
        
        // Create synapses in this batch
        for &(src_nid, dst_nid) in batch {
            if npu_lock
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
        
        // Release lock (drop npu_lock) - burst loop can run now!
        drop(npu_lock);
        
        // Log progress for large batches
        if total_synapses > BATCH_SIZE && (batch_idx + 1) % 10 == 0 {
            info!(
                target: "feagi-bdu",
                "Synapse creation progress: {}/{} batches, {} synapses created",
                batch_idx + 1,
                (total_synapses + BATCH_SIZE - 1) / BATCH_SIZE,
                synapse_count
            );
        }
    }
    
    Ok(synapse_count)
}

/// Apply expander morphology directly on NPU
pub fn apply_expander_morphology(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
    weight: u8,
    conductance: u8,
    synapse_attractivity: u8,
) -> BduResult<u32> {
    use crate::rng::get_rng;
    use rand::Rng;
    let mut rng = get_rng();

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

        let dst_pos = syn_expander("", "", src_pos, src_dimensions, dst_dimensions)?;

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
                        SynapseType::Excitatory,
                    )
                    .is_ok()
            {
                synapse_count += 1;
            }
        }
    }

    Ok(synapse_count)
}

/// Apply block connection morphology directly on NPU
/// 
/// NOTE: This function holds the NPU lock for the entire duration.
/// For large neuron counts (>100k), consider using the batched version
/// that releases the lock between batches.
pub fn apply_block_connection_morphology(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
    scaling_factor: u32,
    weight: u8,
    conductance: u8,
    synapse_attractivity: u8,
) -> BduResult<u32> {
    use crate::rng::get_rng;
    use rand::Rng;
    let mut rng = get_rng();

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

        let dst_pos = syn_block_connection(
            "",
            "",
            src_pos,
            src_dimensions,
            dst_dimensions,
            scaling_factor,
        )?;

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
                        SynapseType::Excitatory,
                    )
                    .is_ok()
            {
                synapse_count += 1;
            }
        }
    }

    Ok(synapse_count)
}

/// Apply pattern matching morphology directly on NPU
pub fn apply_patterns_morphology(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
    patterns: Vec<(Pattern3D, Pattern3D)>,
    weight: u8,
    conductance: u8,
    synapse_attractivity: u8,
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
                            SynapseType::Excitatory,
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

/// Apply vector offset morphology directly on NPU
pub fn apply_vectors_morphology(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
    vectors: Vec<(i32, i32, i32)>,
    weight: u8,
    conductance: u8,
    synapse_attractivity: u8,
) -> BduResult<u32> {
    use crate::rng::get_rng;
    use rand::Rng;
    let mut rng = get_rng();

    if vectors.is_empty() {
        return Ok(0);
    }

    let src_neurons = npu.get_neurons_in_cortical_area(src_area_id);
    if src_neurons.is_empty() {
        return Ok(0);
    }

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

        // Apply all vectors
        for &vector in &vectors {
            if let Ok(dst_pos) = apply_vector_offset(src_pos, vector, 1.0, dst_dimensions) {
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

/// Calculate area dimensions by finding max coordinates
fn calculate_area_dimensions(
    npu: &feagi_npu_burst_engine::DynamicNPU,
    area_id: u32,
) -> (usize, usize, usize) {
    let mut max_x = 0;
    let mut max_y = 0;
    let mut max_z = 0;

    for nid in npu.get_neurons_in_cortical_area(area_id) {
        if let Some((x, y, z)) = npu.get_neuron_coordinates(nid) {
            max_x = max_x.max(x as usize);
            max_y = max_y.max(y as usize);
            max_z = max_z.max(z as usize);
        }
    }

    // Dimensions are max+1 (0-indexed coordinates)
    (max_x + 1, max_y + 1, max_z + 1)
}
