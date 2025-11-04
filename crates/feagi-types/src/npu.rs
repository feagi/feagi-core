/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # NPU Core Data Structures
//!
//! Complete neuron and synapse arrays with SIMD-optimized operations.
//!
//! ## Design Philosophy
//! - **Structure-of-Arrays (SoA)**: Better cache locality and SIMD
//! - **Pre-allocated**: Fixed-size arrays for RTOS compatibility
//! - **Zero-copy**: Use slices and references where possible
//! - **Type-safe**: Strong typing for all neural properties

use crate::*;
use ahash::AHashMap;
use tracing::{debug, warn};

/// Complete neuron array with all properties
///
/// Uses Structure-of-Arrays for SIMD optimization.
/// Generic over `T: NeuralValue` to support multiple quantization levels (FP32, INT8, FP16).
///
/// # Type Parameters
/// - `T: NeuralValue`: The numeric type for membrane potentials, thresholds, and resting potentials.
///   - `f32`: 32-bit floating point (default, highest precision)
///   - `INT8Value`: 8-bit integer (memory efficient, 42% reduction)
///   - `f16`: 16-bit floating point (future, GPU-optimized)
///
/// # Memory Layout
/// - Membrane potentials, thresholds, resting potentials: `T` (quantized)
/// - Leak coefficients: `f32` (kept as f32 for precision - see QUANTIZATION_ISSUES_LOG.md #1)
/// - Excitabilities: `f32` (0.0-1.0 range, kept as f32)
#[derive(Debug, Clone)]
pub struct NeuronArray<T: NeuralValue> {
    /// Number of neurons allocated
    pub capacity: usize,

    /// Number of neurons actually used
    pub count: usize,

    /// Membrane potentials (mV or arbitrary units) - quantized to T
    pub membrane_potentials: Vec<T>,

    /// Firing thresholds - quantized to T
    pub thresholds: Vec<T>,

    /// Leak coefficients (0.0 to 1.0) - LIF leak toward resting potential
    /// Genome parameter: leak_c (with leak_v variability applied at neuron creation)
    /// Kept as f32 for precision (small values don't quantize well - see QUANTIZATION_ISSUES_LOG.md #1)
    pub leak_coefficients: Vec<f32>,

    /// Resting potentials - target potential for leak behavior - quantized to T
    pub resting_potentials: Vec<T>,

    /// Neuron types (0 = excitatory, 1 = inhibitory, etc.)
    pub neuron_types: Vec<i32>,

    /// Refractory periods (burst counts)
    pub refractory_periods: Vec<u16>,

    /// Current refractory countdown
    pub refractory_countdowns: Vec<u16>,

    /// Neuron excitability (0.0 to 1.0 for probabilistic firing)
    pub excitabilities: Vec<f32>,

    /// Consecutive fire counts (how many times neuron fired in a row)
    pub consecutive_fire_counts: Vec<u16>,

    /// Consecutive fire limits (max consecutive fires, 0 = unlimited)
    pub consecutive_fire_limits: Vec<u16>,

    /// Extended refractory periods (additive cooldown after consecutive fire limit)
    /// Gene name: snooze_length (kept for backward compatibility)
    /// Applied as: refractory_countdown = refractory_period + snooze_periods
    /// Note: Previously used separate snooze_countdowns, now unified with refractory_countdowns
    pub snooze_periods: Vec<u16>,

    /// Membrane potential accumulation flags (true = accumulate, false = reset to 0 each burst)
    /// Gene name: nx-mp_acc-b (mp_charge_accumulation in genome processor)
    /// - true: Neuron accumulates potential across bursts (integrator behavior)
    /// - false: Neuron resets to 0.0 at start of each burst (coincidence detector)
    pub mp_charge_accumulation: Vec<bool>,

    /// Cortical area ID for each neuron
    pub cortical_areas: Vec<u32>,

    // TODO: Future plasticity feature - Dynamic consecutive fire limit adjustment
    // Proposed: plasticity.adjust_consecutive_limit(neuron_id, delta)
    // Use cases:
    // - Increase limit for frequently firing neurons (allow more bursts)
    // - Decrease limit for overactive neurons (force cooldown sooner)
    // - Adaptive based on synaptic input patterns
    /// 3D coordinates (x, y, z) - flat array of [x0, y0, z0, x1, y1, z1, ...]
    pub coordinates: Vec<u32>,

    /// Valid neuron mask - true for initialized neurons
    pub valid_mask: Vec<bool>,

    /// ❌ REMOVED: neuron_id_to_index HashMap (was causing 4s slowdown!)
    /// Reality: neuron_id == array_index ALWAYS (sequential assignment)
    /// Coordinate lookup uses direct indexing: coordinates[neuron_id * 3]

    /// Reverse mapping: array index to neuron ID (kept for serialization)
    pub index_to_neuron_id: Vec<u32>,

    /// Spatial hash for coordinate-based neuron lookup
    /// Key = (cortical_area, x, y, z), Value = neuron_id
    /// This enables fast sensory injection by coordinates
    pub spatial_hash: AHashMap<(u32, u32, u32, u32), u32>,
}

impl<T: NeuralValue> NeuronArray<T> {
    /// Create a new neuron array with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            count: 0,
            membrane_potentials: vec![T::zero(); capacity],
            thresholds: vec![T::from_f32(1.0); capacity],
            leak_coefficients: vec![0.0; capacity], // 0 = no leak (common for power neurons)
            resting_potentials: vec![T::zero(); capacity],
            neuron_types: vec![0; capacity], // 0 = excitatory
            refractory_periods: vec![0; capacity],
            refractory_countdowns: vec![0; capacity],
            excitabilities: vec![1.0; capacity],
            consecutive_fire_counts: vec![0; capacity],
            consecutive_fire_limits: vec![0; capacity], // 0 = unlimited
            snooze_periods: vec![0; capacity],          // 0 = no extended refractory
            mp_charge_accumulation: vec![true; capacity], // Default: true (accumulate, backward compatible)
            cortical_areas: vec![0; capacity],
            coordinates: vec![0; capacity * 3],
            valid_mask: vec![false; capacity],
            index_to_neuron_id: vec![0; capacity],
            spatial_hash: AHashMap::new(),
        }
    }

    /// Add a neuron (returns neuron ID = index)
    ///
    /// Uses Leaky Integrate-and-Fire (LIF) model with genome parameters only.
    pub fn add_neuron(
        &mut self,
        threshold: T,  // Quantized threshold
        leak_coefficient: f32, // Genome: leak_c (with leak_v variability already applied) - kept as f32
        resting_potential: T,  // Target potential for leak - quantized
        neuron_type: i32,
        refractory_period: u16,
        excitability: f32,
        consecutive_fire_limit: u16,
        snooze_period: u16, // Genome: nx-snooze-f (rest period after consecutive fires)
        mp_charge_accumulation: bool, // Genome: nx-mp_acc-b (membrane potential accumulation)
        cortical_area: u32,
        x: u32,
        y: u32,
        z: u32,
    ) -> Result<NeuronId> {
        // ⚠️ WARNING: This is the SLOW single-neuron creation path!
        // Should only be called for individual neurons, NOT during bulk neurogenesis
        warn!("⚠️  [RUST-NPU] WARNING: add_neuron() called (SLOW path) - cortical_area={}, total_neurons={}", 
            cortical_area, self.count);

        if self.count >= self.capacity {
            return Err(FeagiError::MemoryAllocationError(
                "Neuron array capacity exceeded".to_string(),
            ));
        }

        let id = self.count;
        self.thresholds[id] = threshold;
        self.leak_coefficients[id] = leak_coefficient;
        self.resting_potentials[id] = resting_potential;
        self.neuron_types[id] = neuron_type;
        self.refractory_periods[id] = refractory_period;
        self.excitabilities[id] = excitability.clamp(0.0, 1.0);
        self.consecutive_fire_counts[id] = 0;
        self.consecutive_fire_limits[id] = consecutive_fire_limit;
        self.snooze_periods[id] = snooze_period; // From genome (nx-snooze-f), used as extended refractory
        self.mp_charge_accumulation[id] = mp_charge_accumulation; // From genome (nx-mp_acc-b)
        self.cortical_areas[id] = cortical_area;
        self.coordinates[id * 3] = x;
        self.coordinates[id * 3 + 1] = y;
        self.coordinates[id * 3 + 2] = z;
        self.valid_mask[id] = true;

        // neuron_id == index (sequential assignment)
        let neuron_id = id as u32;
        self.index_to_neuron_id[id] = neuron_id;

        // Register in spatial hash for coordinate-based lookups (sensory injection)
        let coord_key = (cortical_area, x, y, z);
        self.spatial_hash.insert(coord_key, neuron_id);

        self.count += 1;
        Ok(NeuronId(neuron_id))
    }

    /// Batch add neurons - SIMD-OPTIMIZED for bulk creation
    ///
    /// This is 100-1000x faster than calling add_neuron() in a loop
    /// Uses SIMD vectorization for array operations where possible.
    pub fn add_neurons_batch(
        &mut self,
        thresholds: &[T],  // Quantized thresholds
        leak_coefficients: &[f32],  // Kept as f32 for precision
        resting_potentials: &[T],  // Quantized resting potentials
        neuron_types: &[i32],
        refractory_periods: &[u16],
        excitabilities: &[f32],
        consecutive_fire_limits: &[u16],
        snooze_periods: &[u16],
        mp_charge_accumulations: &[bool],
        cortical_areas: &[u32],
        x_coords: &[u32],
        y_coords: &[u32],
        z_coords: &[u32],
    ) -> Result<Vec<NeuronId>> {
        let n = thresholds.len();

        // Capacity check
        if self.count + n > self.capacity {
            return Err(FeagiError::MemoryAllocationError(format!(
                "Cannot add {} neurons: would exceed capacity {} (current: {})",
                n, self.capacity, self.count
            )));
        }

        // Validate all arrays have same length
        if leak_coefficients.len() != n
            || resting_potentials.len() != n
            || neuron_types.len() != n
            || refractory_periods.len() != n
            || excitabilities.len() != n
            || consecutive_fire_limits.len() != n
            || snooze_periods.len() != n
            || mp_charge_accumulations.len() != n
            || cortical_areas.len() != n
            || x_coords.len() != n
            || y_coords.len() != n
            || z_coords.len() != n
        {
            return Err(FeagiError::ComputationError(
                "All input arrays must have the same length for batch neuron creation".to_string(),
            ));
        }

        let start_id = self.count;
        let mut neuron_ids = Vec::with_capacity(n);

        // SIMD-OPTIMIZED BULK ARRAY OPERATIONS
        // Use slice copy_from_slice for contiguous f32/i32/u16/u32 arrays (SIMD-optimized by LLVM)

        // Copy thresholds (SIMD-optimized memcpy)
        self.thresholds[start_id..start_id + n].copy_from_slice(thresholds);

        // Copy leak_coefficients (SIMD-optimized)
        self.leak_coefficients[start_id..start_id + n].copy_from_slice(leak_coefficients);

        // Copy resting_potentials (SIMD-optimized)
        self.resting_potentials[start_id..start_id + n].copy_from_slice(resting_potentials);

        // Copy neuron_types (SIMD-optimized)
        self.neuron_types[start_id..start_id + n].copy_from_slice(neuron_types);

        // Copy refractory_periods (SIMD-optimized)
        self.refractory_periods[start_id..start_id + n].copy_from_slice(refractory_periods);

        // Excitabilities need clamping - vectorized loop
        for i in 0..n {
            self.excitabilities[start_id + i] = excitabilities[i].clamp(0.0, 1.0);
        }

        // Copy consecutive_fire_limits (SIMD-optimized)
        self.consecutive_fire_limits[start_id..start_id + n]
            .copy_from_slice(consecutive_fire_limits);

        // Copy snooze_periods (SIMD-optimized)
        self.snooze_periods[start_id..start_id + n].copy_from_slice(snooze_periods);

        // Copy cortical_areas (SIMD-optimized)
        self.cortical_areas[start_id..start_id + n].copy_from_slice(cortical_areas);

        // Initialize consecutive_fire_counts to zero (SIMD-optimized memset)
        self.consecutive_fire_counts[start_id..start_id + n].fill(0);

        // Set valid_mask to true (SIMD-optimized)
        self.valid_mask[start_id..start_id + n].fill(true);

        // Copy mp_charge_accumulations and coordinates (requires element-wise due to data layout)
        use std::time::Instant;
        let coord_start = Instant::now();
        for i in 0..n {
            let idx = start_id + i;
            let neuron_id = idx as u32;

            self.mp_charge_accumulation[idx] = mp_charge_accumulations[i];

            // Coordinates (strided layout: [x0,y0,z0, x1,y1,z1, ...])
            self.coordinates[idx * 3] = x_coords[i];
            self.coordinates[idx * 3 + 1] = y_coords[i];
            self.coordinates[idx * 3 + 2] = z_coords[i];

            self.index_to_neuron_id[idx] = neuron_id;
            neuron_ids.push(NeuronId(neuron_id));
        }
        let coord_time = coord_start.elapsed();
        debug!("[COORD-LOOP] n={}, time={:?}", n, coord_time);

        // ✅ SPATIAL HASH ONLY (for coordinate→neuron_id lookups during sensory injection)
        // neuron_id_to_index HashMap eliminated - it was storing id→id and never read!
        let hash_start = Instant::now();
        self.spatial_hash.reserve(n);
        let reserve_time = hash_start.elapsed();

        let insert_start = Instant::now();
        for i in 0..n {
            let idx = start_id + i;
            let neuron_id = idx as u32;

            // Only spatial hash needed (for sensory injection by coordinates)
            let coord_key = (cortical_areas[i], x_coords[i], y_coords[i], z_coords[i]);
            self.spatial_hash.insert(coord_key, neuron_id);
        }
        let insert_time = insert_start.elapsed();

        debug!(
            n,
            reserve_ns = reserve_time.as_nanos(),
            inserts_ns = insert_time.as_nanos(),
            hash_size = self.spatial_hash.len(),
            "[SPATIAL-HASH] Coordinate hash updated"
        );

        self.count += n;
        Ok(neuron_ids)
    }

    /// Get neuron threshold (returns f32 for backward compatibility)
    #[inline(always)]
    pub fn get_threshold(&self, neuron_id: NeuronId) -> f32 {
        self.thresholds[neuron_id.0 as usize].to_f32()
    }

    /// Get neuron threshold as T (returns quantized value)
    #[inline(always)]
    pub fn get_threshold_quantized(&self, neuron_id: NeuronId) -> T {
        self.thresholds[neuron_id.0 as usize]
    }

    /// Get neuron membrane potential (returns f32 for backward compatibility)
    #[inline(always)]
    pub fn get_potential(&self, neuron_id: NeuronId) -> f32 {
        self.membrane_potentials[neuron_id.0 as usize].to_f32()
    }

    /// Get neuron membrane potential as T (returns quantized value)
    #[inline(always)]
    pub fn get_potential_quantized(&self, neuron_id: NeuronId) -> T {
        self.membrane_potentials[neuron_id.0 as usize]
    }

    /// Set neuron membrane potential (accepts T for type safety)
    #[inline(always)]
    pub fn set_potential(&mut self, neuron_id: NeuronId, potential: T) {
        self.membrane_potentials[neuron_id.0 as usize] = potential;
    }

    /// Set neuron membrane potential from f32 (convenience method)
    #[inline(always)]
    pub fn set_potential_f32(&mut self, neuron_id: NeuronId, potential: f32) {
        self.membrane_potentials[neuron_id.0 as usize] = T::from_f32(potential);
    }

    /// Accumulate to neuron membrane potential (accepts T for type safety)
    #[inline(always)]
    pub fn accumulate_potential(&mut self, neuron_id: NeuronId, delta: T) {
        let idx = neuron_id.0 as usize;
        self.membrane_potentials[idx] = self.membrane_potentials[idx].saturating_add(delta);
    }

    /// Accumulate to neuron membrane potential from f32 (convenience method)
    #[inline(always)]
    pub fn accumulate_potential_f32(&mut self, neuron_id: NeuronId, delta: f32) {
        self.accumulate_potential(neuron_id, T::from_f32(delta));
    }

    /// Get neuron cortical area
    #[inline(always)]
    pub fn get_cortical_area(&self, neuron_id: NeuronId) -> CorticalAreaId {
        CorticalAreaId(self.cortical_areas[neuron_id.0 as usize])
    }

    /// Get neuron coordinates
    #[inline(always)]
    pub fn get_coordinates(&self, neuron_id: NeuronId) -> (u32, u32, u32) {
        let idx = neuron_id.0 as usize * 3;
        (
            self.coordinates[idx],
            self.coordinates[idx + 1],
            self.coordinates[idx + 2],
        )
    }

    /// Get neuron ID by coordinates (spatial hash lookup for sensory injection)
    ///
    /// Returns None if no neuron exists at the given coordinates
    #[inline(always)]
    pub fn get_neuron_at_coordinate(
        &self,
        cortical_area: u32,
        x: u32,
        y: u32,
        z: u32,
    ) -> Option<NeuronId> {
        let coord_key = (cortical_area, x, y, z);
        self.spatial_hash.get(&coord_key).map(|&id| NeuronId(id))
    }

    /// Batch coordinate lookup for sensory injection (ZERO-COPY, no allocation)
    ///
    /// Converts (x,y,z) coordinates to neuron IDs using spatial hash.
    /// Silently skips invalid coordinates (returns only valid neurons).
    pub fn batch_coordinate_lookup(
        &self,
        cortical_area: u32,
        coordinates: &[(u32, u32, u32)],
    ) -> Vec<NeuronId> {
        coordinates
            .iter()
            .filter_map(|&(x, y, z)| {
                let coord_key = (cortical_area, x, y, z);
                self.spatial_hash.get(&coord_key).map(|&id| NeuronId(id))
            })
            .collect()
    }

    /// Get all neuron IDs in a specific cortical area
    ///
    /// PERFORMANCE: SIMD-accelerated parallel scan (auto-vectorized by LLVM)
    /// - Processes 8-16 neurons per instruction with AVX2/AVX-512
    /// - ~14 microseconds for 227k neurons (vs 4 seconds with Python loop!)
    /// - GPU-compatible algorithm (same for CUDA/Metal/Vulkan)
    /// - No data structure overhead, works with area expansion/deletion
    ///
    /// ARCHITECTURE: Linear scan is acceptable because:
    /// - SIMD makes it extremely fast (sub-millisecond)
    /// - Avoids synapse loss on area expansion
    /// - Works for both memory and non-memory neurons
    /// - Cache-friendly sequential access pattern
    #[inline]
    pub fn get_neurons_in_cortical_area(&self, cortical_idx: u32) -> Vec<u32> {
        use std::time::Instant;
        let start = Instant::now();

        // SIMD-accelerated filtering (LLVM auto-vectorizes this with AVX2/AVX-512)
        // Processes multiple neurons in parallel per instruction
        let result: Vec<u32> = (0..self.count)
            .filter(|&idx| {
                // Both checks can be vectorized: 16 comparisons per instruction
                self.valid_mask[idx] && self.cortical_areas[idx] == cortical_idx
            })
            .map(|idx| self.index_to_neuron_id[idx])
            .collect();

        let elapsed = start.elapsed();
        debug!(
            cortical_idx,
            found_neurons = result.len(),
            elapsed_ms = elapsed.as_millis(),
            total_scanned = self.count,
            "[SIMD-SCAN] Neuron area scan complete"
        );

        result
    }

    /// Get total number of active neurons
    #[inline(always)]
    pub fn get_neuron_count(&self) -> usize {
        self.count
    }
}

/// Type aliases for backward compatibility and convenience
///
/// These aliases provide explicit type annotations for common use cases.
/// Use them when you need to be explicit about quantization level.
pub type NeuronArrayF32 = NeuronArray<f32>;
pub type NeuronArrayINT8 = NeuronArray<crate::INT8Value>;
// Future: pub type NeuronArrayF16 = NeuronArray<f16>;

/// Complete synapse array with dynamic operations
///
/// Uses Structure-of-Arrays for SIMD optimization
#[derive(Debug, Clone)]
pub struct SynapseArray {
    /// Number of synapses allocated
    pub capacity: usize,

    /// Number of synapses actually used
    pub count: usize,

    /// Source neuron IDs
    pub source_neurons: Vec<u32>,

    /// Target neuron IDs
    pub target_neurons: Vec<u32>,

    /// Synaptic weights (0-255)
    pub weights: Vec<u8>,

    /// Postsynaptic potentials (0-255) - FEAGI term for synaptic strength
    /// Represents the source area's pstcr_ (postsynaptic current) value
    pub postsynaptic_potentials: Vec<u8>,

    /// Synapse types (0=excitatory, 1=inhibitory)
    pub types: Vec<u8>,

    /// Valid mask (for soft deletion)
    pub valid_mask: Vec<bool>,

    /// Source neuron index: neuron_id -> [synapse_indices]
    pub source_index: AHashMap<u32, Vec<usize>>,
}

impl SynapseArray {
    /// Create a new synapse array with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            count: 0,
            source_neurons: vec![0; capacity],
            target_neurons: vec![0; capacity],
            weights: vec![0; capacity],
            postsynaptic_potentials: vec![255; capacity],
            types: vec![0; capacity],
            valid_mask: vec![false; capacity],
            source_index: AHashMap::new(),
        }
    }

    /// Add a synapse (returns synapse index)
    pub fn add_synapse(
        &mut self,
        source: NeuronId,
        target: NeuronId,
        weight: SynapticWeight,
        conductance: SynapticConductance,
        synapse_type: SynapseType,
    ) -> Result<usize> {
        if self.count >= self.capacity {
            return Err(FeagiError::MemoryAllocationError(
                "Synapse array capacity exceeded".to_string(),
            ));
        }

        let idx = self.count;
        self.source_neurons[idx] = source.0;
        self.target_neurons[idx] = target.0;
        self.weights[idx] = weight.0;
        self.postsynaptic_potentials[idx] = conductance.0;
        self.types[idx] = match synapse_type {
            SynapseType::Excitatory => 0,
            SynapseType::Inhibitory => 1,
        };
        self.valid_mask[idx] = true;

        // Update source index
        self.source_index
            .entry(source.0)
            .or_insert_with(Vec::new)
            .push(idx);

        self.count += 1;
        Ok(idx)
    }

    /// SIMD-optimized batch synapse creation
    ///
    /// Creates multiple synapses in a single operation with minimal Python→Rust overhead.
    /// This method is 50-100x faster than calling add_synapse() in a Python loop.
    ///
    /// Performance advantages:
    /// - Single FFI boundary crossing (vs N crossings)
    /// - Contiguous memory writes (SoA optimization)
    /// - Batch capacity checking
    /// - Efficient source_index updates
    ///
    /// Args:
    ///     sources: Array of source neuron IDs
    ///     targets: Array of target neuron IDs  
    ///     weights: Array of synaptic weights
    ///     conductances: Array of conductances
    ///     synapse_types: Array of synapse types (0=excitatory, 1=inhibitory)
    ///
    /// Returns:
    ///     (successful_count, failed_indices) tuple
    ///     - successful_count: Number of synapses created
    ///     - failed_indices: Indices that failed (e.g., capacity exceeded)
    pub fn add_synapses_batch(
        &mut self,
        sources: &[u32],
        targets: &[u32],
        weights: &[u8],
        conductances: &[u8],
        synapse_types: &[u8],
    ) -> (usize, Vec<usize>) {
        let n = sources.len();

        // Validate all arrays have same length
        if targets.len() != n
            || weights.len() != n
            || conductances.len() != n
            || synapse_types.len() != n
        {
            // Return all indices as failed if array lengths don't match
            return (0, (0..n).collect());
        }

        let mut successful_count = 0;
        let mut failed_indices = Vec::new();

        // Pre-allocate space for source_index updates (avoid repeated allocations)
        let mut source_index_updates: Vec<(u32, usize)> = Vec::with_capacity(n);

        // Batch process all synapses
        for i in 0..n {
            // Check capacity before each insertion
            if self.count >= self.capacity {
                failed_indices.push(i);
                continue;
            }

            let idx = self.count;
            let source = sources[i];
            let target = targets[i];
            let weight = weights[i];
            let conductance = conductances[i];
            let synapse_type = synapse_types[i];

            // SoA writes: Excellent cache locality (sequential memory access)
            self.source_neurons[idx] = source;
            self.target_neurons[idx] = target;
            self.weights[idx] = weight;
            self.postsynaptic_potentials[idx] = conductance;
            self.types[idx] = synapse_type;
            self.valid_mask[idx] = true;

            // Collect source index updates (batch apply later)
            source_index_updates.push((source, idx));

            self.count += 1;
            successful_count += 1;
        }

        // Batch update source_index (reduces HashMap lookup overhead)
        for (source, idx) in source_index_updates {
            self.source_index
                .entry(source)
                .or_insert_with(Vec::new)
                .push(idx);
        }

        (successful_count, failed_indices)
    }

    /// Remove a synapse (soft delete - just marks as invalid)
    pub fn remove_synapse(&mut self, source: NeuronId, target: NeuronId) -> bool {
        if let Some(indices) = self.source_index.get(&source.0) {
            for &idx in indices {
                if self.valid_mask[idx]
                    && self.source_neurons[idx] == source.0
                    && self.target_neurons[idx] == target.0
                {
                    self.valid_mask[idx] = false;
                    return true;
                }
            }
        }
        false
    }

    /// SIMD-optimized batch removal: delete all synapses from specified sources
    ///
    /// This method uses the source_index for O(1) lookup and processes synapses
    /// in a cache-friendly manner for maximum performance.
    ///
    /// Returns: number of synapses deleted
    pub fn remove_synapses_from_sources(&mut self, sources: &[u32]) -> usize {
        let mut deleted = 0;

        // Use source_index for O(1) lookup per source (no full array scan)
        for &source in sources {
            if let Some(indices) = self.source_index.get(&source) {
                // Mark all synapses from this source as invalid
                // Process in chunks for better cache locality
                for &idx in indices {
                    if self.valid_mask[idx] {
                        self.valid_mask[idx] = false;
                        deleted += 1;
                    }
                }
            }
        }

        deleted
    }

    /// SIMD-optimized batch removal: delete synapses between source and target sets
    ///
    /// This method combines source_index lookup with bit-vector target filtering
    /// for maximum performance. Optimized for both few→many and many→many scenarios.
    ///
    /// Performance:
    /// - 1 source → 16K targets: ~50-100x faster than nested loops
    /// - 100 sources → 100 targets: ~20-50x faster
    ///
    /// Returns: number of synapses deleted
    pub fn remove_synapses_between(&mut self, sources: &[u32], targets: &[u32]) -> usize {
        if targets.is_empty() {
            return 0;
        }

        let mut deleted = 0;

        // Build bit vector for O(1) target membership testing
        // This is much faster than HashMap for repeated lookups
        let max_target = targets.iter().max().copied().unwrap_or(0) as usize;
        let bitvec_size = (max_target / 64) + 1;
        let mut target_bitvec = vec![0u64; bitvec_size];

        // Populate bit vector
        for &target in targets {
            let word_idx = target as usize / 64;
            let bit_idx = target as usize % 64;
            if word_idx < bitvec_size {
                target_bitvec[word_idx] |= 1u64 << bit_idx;
            }
        }

        // For each source, check its synapses against target bit vector
        for &source in sources {
            if let Some(indices) = self.source_index.get(&source) {
                // Process synapses from this source
                // The source_neurons and target_neurons arrays are contiguous (SoA)
                // which enables excellent cache performance
                for &idx in indices {
                    if !self.valid_mask[idx] {
                        continue; // Already deleted
                    }

                    let target = self.target_neurons[idx];
                    let word_idx = target as usize / 64;
                    let bit_idx = target as usize % 64;

                    // Bit vector lookup: O(1) with excellent cache locality
                    if word_idx < bitvec_size && (target_bitvec[word_idx] & (1u64 << bit_idx)) != 0
                    {
                        self.valid_mask[idx] = false;
                        deleted += 1;
                    }
                }
            }
        }

        deleted
    }

    /// Update synapse weight
    pub fn update_weight(
        &mut self,
        source: NeuronId,
        target: NeuronId,
        new_weight: SynapticWeight,
    ) -> bool {
        if let Some(indices) = self.source_index.get(&source.0) {
            for &idx in indices {
                if self.valid_mask[idx]
                    && self.source_neurons[idx] == source.0
                    && self.target_neurons[idx] == target.0
                {
                    self.weights[idx] = new_weight.0;
                    return true;
                }
            }
        }
        false
    }

    /// Get number of valid synapses
    pub fn valid_count(&self) -> usize {
        self.valid_mask.iter().filter(|&&v| v).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_array_creation() {
        let neurons = NeuronArray::<f32>::new(100);
        assert_eq!(neurons.capacity, 100);
        assert_eq!(neurons.count, 0);
    }

    #[test]
    fn test_add_neuron() {
        let mut neurons = NeuronArray::<f32>::new(100);
        let id = neurons
            .add_neuron(
                1.0,    // threshold (f32 literal)
                0.1,    // leak_coefficient
                0.0,    // resting_potential (f32 literal)
                0,      // neuron_type
                5,      // refractory_period
                1.0,    // excitability
                0,      // consecutive_fire_limit
                0,      // snooze_period
                true,   // mp_charge_accumulation
                1,      // cortical_area
                10, 5, 3, // x, y, z
            )
            .unwrap();

        assert_eq!(id.0, 0);
        assert_eq!(neurons.count, 1);
        assert_eq!(neurons.get_threshold(id), 1.0);
        assert_eq!(neurons.get_coordinates(id), (10, 5, 3));
    }

    #[test]
    fn test_synapse_array() {
        let mut synapses = SynapseArray::new(1000);

        let idx = synapses
            .add_synapse(
                NeuronId(1),
                NeuronId(2),
                SynapticWeight(128),
                SynapticConductance(255),
                SynapseType::Excitatory,
            )
            .unwrap();

        assert_eq!(idx, 0);
        assert_eq!(synapses.count, 1);
        assert_eq!(synapses.valid_count(), 1);

        // Test removal
        assert!(synapses.remove_synapse(NeuronId(1), NeuronId(2)));
        assert_eq!(synapses.valid_count(), 0);
    }
}
