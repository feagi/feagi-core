// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Block connection morphology implementation.

Maps neurons using scaled block mapping between source and destination areas.
Supports both regular and batched versions for performance optimization.
*/

use crate::connectivity::rules::syn_block_connection;
use crate::types::BduResult;
use feagi_npu_neural::types::{NeuronId, SynapticConductance, SynapticWeight};
use feagi_npu_neural::SynapseType;
use std::sync::Arc;

/// Apply block connection morphology with batched processing (releases NPU lock between batches)
/// 
/// This version is optimized for large neuron counts (>100k) and releases the NPU lock
/// between batches to allow the burst loop to run, preventing 4-17 second blocking.
pub fn apply_block_connection_morphology_batched(
    npu: &Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
    src_area_id: u32,
    dst_area_id: u32,
    src_dimensions: (usize, usize, usize),
    dst_dimensions: (usize, usize, usize),
    scaling_factor: u32,
    weight: u8,
    conductance: u8,
    synapse_attractivity: u8,
) -> BduResult<u32> {
    use crate::rng::get_rng;
    use rand::Rng;
    use tracing::info;
    let mut rng = get_rng();
    
    const BATCH_SIZE: usize = 50_000; // Process 50k synapses per batch
    
    // CRITICAL: Do NOT call get_neurons_in_cortical_area - iterate through coordinate space instead
    // Step 1: Pre-compute all synapse operations by iterating coordinate space (NO LOCK)
    let mut synapse_ops: Vec<(u32, u32)> = Vec::new();
    
    // Iterate through coordinate space instead of neurons
    for x in 0..src_dimensions.0 {
        for y in 0..src_dimensions.1 {
            for z in 0..src_dimensions.2 {
                let src_pos = (x as u32, y as u32, z as u32);
                
                // Calculate destination coordinate using morphology
                let dst_pos = match syn_block_connection(
                    "",
                    "",
                    src_pos,
                    src_dimensions,
                    dst_dimensions,
                    scaling_factor,
                ) {
                    Ok(pos) => pos,
                    Err(_) => continue,
                };
                
                // Store operation (will look up neurons later in batches)
                synapse_ops.push((src_pos.0 << 16 | src_pos.1 << 8 | src_pos.2, dst_pos.0 << 16 | dst_pos.1 << 8 | dst_pos.2));
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
            "Batching synapse creation: {} coordinate pairs in batches of {} (releasing NPU lock between batches)",
            total_synapses, BATCH_SIZE
        );
    }
    
    // Step 2: Look up neurons and create synapses in batches, releasing lock between batches
    let mut synapse_count = 0u32;
    for (batch_idx, batch) in synapse_ops.chunks(BATCH_SIZE).enumerate() {
        // Re-acquire NPU lock for this batch
        let npu_lock = npu.lock().map_err(|e| {
            crate::types::BduError::Internal(format!("Failed to lock NPU for batch {}: {}", batch_idx, e))
        })?;
        
        // Decode coordinates and look up neurons
        let mut batch_synapses = Vec::new();
        for &(src_coord_encoded, dst_coord_encoded) in batch {
            let src_pos = (
                (src_coord_encoded >> 16) as u32,
                ((src_coord_encoded >> 8) & 0xFF) as u32,
                (src_coord_encoded & 0xFF) as u32,
            );
            let dst_pos = (
                (dst_coord_encoded >> 16) as u32,
                ((dst_coord_encoded >> 8) & 0xFF) as u32,
                (dst_coord_encoded & 0xFF) as u32,
            );
            
            // Look up neurons at coordinates
            if let Some(src_nid) = npu_lock.get_neuron_id_at_coordinate(src_area_id, src_pos.0, src_pos.1, src_pos.2) {
                if let Some(dst_nid) = npu_lock.get_neuron_id_at_coordinate(dst_area_id, dst_pos.0, dst_pos.1, dst_pos.2) {
                    if rng.gen_range(0..100) < synapse_attractivity {
                        batch_synapses.push((src_nid, dst_nid));
                    }
                }
            }
        }
        
        // Create synapses in this batch (need mutable lock)
        drop(npu_lock);
        let mut npu_lock = npu.lock().map_err(|e| {
            crate::types::BduError::Internal(format!("Failed to lock NPU for batch {}: {}", batch_idx, e))
        })?;
        
        for (src_nid, dst_nid) in batch_synapses {
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

/// Apply block connection morphology directly on NPU
/// 
/// NOTE: This function holds the NPU lock for the entire duration.
/// For large neuron counts (>100k), consider using the batched version
/// that releases the lock between batches.
pub fn apply_block_connection_morphology(
    npu: &mut feagi_npu_burst_engine::DynamicNPU,
    src_area_id: u32,
    dst_area_id: u32,
    src_dimensions: (usize, usize, usize),
    dst_dimensions: (usize, usize, usize),
    scaling_factor: u32,
    weight: u8,
    conductance: u8,
    synapse_attractivity: u8,
) -> BduResult<u32> {
    use crate::rng::get_rng;
    use rand::Rng;
    use tracing::warn;
    use std::time::Instant;
    let mut rng = get_rng();
    
    warn!(
        target: "feagi-bdu",
        "🔍 ENTRY: apply_block_connection_morphology called with src_area_id={}, dst_area_id={}, src_dim={:?}, dst_dim={:?}",
        src_area_id, dst_area_id, src_dimensions, dst_dimensions
    );
    
    // CRITICAL: Do NOT call get_neurons_in_cortical_area - it iterates through ALL 8M neurons!
    // 
    // PROBLEM: get_neuron_id_at_coordinate() does a linear search through all neurons for each lookup.
    // Iterating through coordinate space and calling it for each coordinate is O(coordinate_space * total_neurons),
    // which could be worse than scanning neurons once if coordinate space is large.
    //
    // SOLUTION: Use batch coordinate lookup if available, or fall back to scanning neurons once
    // and using the cached coordinate map. For now, we'll use a more efficient approach:
    // 1. Calculate all destination coordinates first (no neuron lookup needed)
    // 2. Use batch_get_neuron_ids_from_coordinates for both source and destination lookups
    //
    // However, we still need to know which source coordinates to check. For block_connection,
    // we iterate through the source coordinate space.
    
    let total_coords = src_dimensions.0 * src_dimensions.1 * src_dimensions.2;
    if total_coords > 1_000_000 {
        warn!(
            target: "feagi-bdu",
            "⚠️ Large coordinate space: {}x{}x{} = {} coordinates. Consider using batched lookup.",
            src_dimensions.0, src_dimensions.1, src_dimensions.2, total_coords
        );
    }
    
    let start = Instant::now();
    
    // OPTIMIZATION: For small coordinate spaces, pre-calculate all coordinate pairs
    // For large spaces, we could optimize further by only checking coordinates where neurons exist
    let total_source_coords = src_dimensions.0 * src_dimensions.1 * src_dimensions.2;
    
    // Collect all source coordinates we need to check
    let mut src_coords_to_check = Vec::with_capacity(total_source_coords);
    let mut expected_dst_coords = Vec::with_capacity(total_source_coords);
    
    for x in 0..src_dimensions.0 {
        for y in 0..src_dimensions.1 {
            for z in 0..src_dimensions.2 {
                let src_pos = (x as u32, y as u32, z as u32);
                src_coords_to_check.push(src_pos);
                
                // Calculate destination coordinate
                let dst_pos = match syn_block_connection(
                    "",
                    "",
                    src_pos,
                    src_dimensions,
                    dst_dimensions,
                    scaling_factor,
                ) {
                    Ok(pos) => pos,
                    Err(_) => {
                        expected_dst_coords.push(None);
                        continue;
                    }
                };
                expected_dst_coords.push(Some(dst_pos));
            }
        }
    }
    
    let calc_time = start.elapsed();
    if calc_time.as_millis() > 100 {
        warn!(
            target: "feagi-bdu",
            "⚠️ Slow coordinate calculation: {}ms for {} source coordinates ({}x{}x{})",
            calc_time.as_millis(),
            src_coords_to_check.len(),
            src_dimensions.0, src_dimensions.1, src_dimensions.2
        );
    }
    
    // CRITICAL: Use batch coordinate lookup which builds hashmap once (O(neurons_in_area))
    // then does O(1) lookups for each coordinate. This is MUCH faster than individual linear searches!
    let lookup_start = Instant::now();
    
    // DEBUG: Log what we're looking up
    warn!(
        target: "feagi-bdu",
        "🔍 DEBUG block_to_block: Looking up {} coordinates for area_id={}",
        src_coords_to_check.len(),
        src_area_id
    );
    if src_coords_to_check.len() <= 10 {
        warn!(
            target: "feagi-bdu",
            "  First few: {:?}",
            &src_coords_to_check[..src_coords_to_check.len().min(5)]
        );
    } else {
        warn!(
            target: "feagi-bdu",
            "  First: {:?}, last: {:?}",
            src_coords_to_check[0],
            src_coords_to_check[src_coords_to_check.len() - 1]
        );
    }
    
    let src_neuron_lookups = npu.batch_get_neuron_ids_from_coordinates_with_none(src_area_id, &src_coords_to_check);
    let lookup_time = lookup_start.elapsed();
    
    // DEBUG: Check how many neurons were actually found
    let found_count = src_neuron_lookups.iter().filter(|opt| opt.is_some()).count();
    warn!(
        target: "feagi-bdu",
        "🔍 DEBUG block_to_block: batch lookup found {} neurons out of {} coordinates",
        found_count,
        src_coords_to_check.len()
    );
    
    if found_count == 0 {
        // DEBUG: Try to verify neurons exist using the working method
        let neurons_in_area = npu.get_neurons_in_cortical_area(src_area_id);
        warn!(
            target: "feagi-bdu",
            "🔍 DEBUG block_to_block: batch lookup found 0 neurons, but get_neurons_in_cortical_area({}) found {} neurons",
            src_area_id,
            neurons_in_area.len()
        );
        
        // DEBUG: Check first few neurons' coordinates and area IDs
        if !neurons_in_area.is_empty() {
            let sample_size = neurons_in_area.len().min(5);
            let mut sample_coords = Vec::new();
            let mut sample_area_ids = Vec::new();
            for &nid in &neurons_in_area[..sample_size] {
                if let Some(coords) = npu.get_neuron_coordinates(nid) {
                    sample_coords.push(coords);
                }
                sample_area_ids.push(npu.get_neuron_cortical_area(nid));
            }
            warn!(
                target: "feagi-bdu",
                "🔍 DEBUG block_to_block: Sample neurons - area_ids: {:?}, coords: {:?}",
                sample_area_ids,
                sample_coords
            );
            
            // DEBUG: Check if any of our lookup coordinates match sample coordinates
            let matching_coords: Vec<_> = src_coords_to_check.iter()
                .filter(|&coord| sample_coords.contains(coord))
                .take(5)
                .collect();
            warn!(
                target: "feagi-bdu",
                "🔍 DEBUG block_to_block: Found {} matching coordinates between lookup and sample: {:?}",
                matching_coords.len(),
                matching_coords
            );
        }
    }
    
    // Match source neurons with their destination coordinates (preserving index mapping)
    let mut src_to_dst_map = Vec::new();
    let mut found_source_count = 0;
    
    for (idx, src_nid_opt) in src_neuron_lookups.iter().enumerate() {
        if let Some(src_nid) = src_nid_opt {
            found_source_count += 1;
            if let Some(dst_pos) = expected_dst_coords[idx] {
                src_to_dst_map.push((*src_nid, dst_pos));
            }
        }
    }
    
    if lookup_time.as_millis() > 100 || found_source_count == 0 {
        warn!(
            target: "feagi-bdu",
            "⚠️ Source batch lookup: {}ms for {} coordinates (found {} neurons, dimensions={}x{}x{})",
            lookup_time.as_millis(),
            src_coords_to_check.len(),
            found_source_count,
            src_dimensions.0, src_dimensions.1, src_dimensions.2
        );
    }
    
    if src_to_dst_map.is_empty() {
        warn!(
            target: "feagi-bdu",
            "⚠️ No source neurons found in coordinate space {}x{}x{}",
            src_dimensions.0, src_dimensions.1, src_dimensions.2
        );
        return Ok(0);
    }
    
    // Collect unique destination coordinates for batch lookup
    let dst_coords_to_check: Vec<_> = src_to_dst_map.iter().map(|(_, dst_pos)| *dst_pos).collect();
    
    // Batch lookup destination neurons (uses cached coordinate map - O(neurons_in_area) + O(coords))
    let dst_lookup_start = Instant::now();
    let dst_neuron_lookups = npu.batch_get_neuron_ids_from_coordinates_with_none(dst_area_id, &dst_coords_to_check);
    let dst_lookup_time = dst_lookup_start.elapsed();
    
    // Build destination coordinate -> neuron ID map
    let mut dst_coord_to_neuron = std::collections::HashMap::new();
    for (idx, dst_nid_opt) in dst_neuron_lookups.iter().enumerate() {
        if let Some(dst_nid) = dst_nid_opt {
            dst_coord_to_neuron.insert(dst_coords_to_check[idx], *dst_nid);
        }
    }
    
    // Create synapses for matched pairs
    let mut synapse_count = 0u32;
    let mut found_dest_count = 0;
    
    for (src_nid, dst_pos) in src_to_dst_map {
        if let Some(dst_nid) = dst_coord_to_neuron.get(&dst_pos) {
            found_dest_count += 1;
            if rng.gen_range(0..100) < synapse_attractivity
                && npu
                    .add_synapse(
                        src_nid,
                        *dst_nid,
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
    
    if dst_lookup_time.as_millis() > 100 || found_dest_count == 0 {
        warn!(
            target: "feagi-bdu",
            "⚠️ Destination batch lookup: {}ms for {} coordinates (found {} neurons, created {} synapses)",
            dst_lookup_time.as_millis(),
            dst_coords_to_check.len(),
            found_dest_count,
            synapse_count
        );
    }
    
    let total_time = start.elapsed();
    if total_time.as_millis() > 100 {
        warn!(
            target: "feagi-bdu",
            "⚠️ Slow block_connection synaptogenesis: {}ms total (calc={}ms, src_lookup={}ms, dst_lookup={}ms, synapses={})",
            total_time.as_millis(),
            calc_time.as_millis(),
            lookup_time.as_millis(),
            dst_lookup_time.as_millis(),
            synapse_count
        );
    }

    Ok(synapse_count)
}

