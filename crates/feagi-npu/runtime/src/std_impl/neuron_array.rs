// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Standard neuron array implementation
//!
//! Uses `Vec` for dynamic growth and Rayon for parallel processing.

use crate::traits::{NeuronStorage, Result};
use ahash::AHashMap;
use feagi_npu_neural::types::NeuralValue;
use feagi_npu_neural::{is_refractory, update_neuron_lif};
use rayon::prelude::*; // Faster hash map (already a dependency)
use std::sync::Mutex;
use std::vec::Vec;

/// Dynamic neuron array for desktop/server environments
///
/// Generic over `T: NeuralValue` to support multiple quantization levels
pub struct NeuronArray<T: NeuralValue> {
    /// Current number of neurons
    pub count: usize,

    /// Membrane potentials (quantized to T)
    pub membrane_potentials: Vec<T>,

    /// Firing thresholds (quantized to T) - minimum MP to fire
    pub thresholds: Vec<T>,

    /// Firing threshold limits (quantized to T) - maximum MP to fire (0 = no limit)
    pub threshold_limits: Vec<T>,

    /// Leak coefficients (kept as f32 for precision)
    pub leak_coefficients: Vec<f32>,

    /// Resting potentials
    pub resting_potentials: Vec<T>,

    /// Neuron types (0=excitatory, 1=inhibitory)
    pub neuron_types: Vec<i32>,

    /// Refractory periods
    pub refractory_periods: Vec<u16>,

    /// Refractory countdowns (state)
    pub refractory_countdowns: Vec<u16>,

    /// Excitability factors
    pub excitabilities: Vec<f32>,

    /// Consecutive fire counts
    pub consecutive_fire_counts: Vec<u16>,

    /// Consecutive fire limits
    pub consecutive_fire_limits: Vec<u16>,

    /// Snooze periods (extended refractory)
    pub snooze_periods: Vec<u16>,

    /// Membrane potential charge accumulation flags
    pub mp_charge_accumulation: Vec<bool>,

    /// Cortical area IDs
    pub cortical_areas: Vec<u32>,

    /// 3D coordinates (flat: [x0,y0,z0, x1,y1,z1, ...])
    pub coordinates: Vec<u32>,

    /// Valid mask
    pub valid_mask: Vec<bool>,

    /// Cached coordinate maps per cortical area
    /// Maps: cortical_area -> (x, y, z) -> neuron_index
    /// This cache eliminates O(n) scans on every coordinate lookup
    /// Cache is invalidated when neurons are added/removed for an area
    /// Uses Mutex for thread-safe interior mutability
    coord_map_cache: Mutex<AHashMap<u32, AHashMap<(u32, u32, u32), usize>>>,
}

impl<T: NeuralValue> NeuronArray<T> {
    /// Create a new neuron array with initial capacity
    pub fn new(capacity: usize) -> Self {
        let mut result = Self {
            count: 0,
            membrane_potentials: Vec::with_capacity(capacity),
            thresholds: Vec::with_capacity(capacity),
            threshold_limits: Vec::with_capacity(capacity),
            leak_coefficients: Vec::with_capacity(capacity),
            resting_potentials: Vec::with_capacity(capacity),
            neuron_types: Vec::with_capacity(capacity),
            refractory_periods: Vec::with_capacity(capacity),
            refractory_countdowns: Vec::with_capacity(capacity),
            excitabilities: Vec::with_capacity(capacity),
            consecutive_fire_counts: Vec::with_capacity(capacity),
            consecutive_fire_limits: Vec::with_capacity(capacity),
            snooze_periods: Vec::with_capacity(capacity),
            mp_charge_accumulation: Vec::with_capacity(capacity),
            cortical_areas: Vec::with_capacity(capacity),
            coordinates: Vec::with_capacity(capacity * 3), // x,y,z per neuron
            valid_mask: Vec::with_capacity(capacity),
            coord_map_cache: Mutex::new(AHashMap::new()),
        };
        // Resize to capacity with default values
        result.membrane_potentials.resize(capacity, T::zero());
        result.thresholds.resize(capacity, T::from_f32(1.0));
        result.threshold_limits.resize(capacity, T::max_value()); // MAX = no limit (SIMD-friendly encoding)
        result.leak_coefficients.resize(capacity, 0.1);
        result.resting_potentials.resize(capacity, T::zero());
        result.neuron_types.resize(capacity, 0);
        result.refractory_periods.resize(capacity, 0);
        result.refractory_countdowns.resize(capacity, 0);
        result.excitabilities.resize(capacity, 1.0);
        result.consecutive_fire_counts.resize(capacity, 0);
        result.consecutive_fire_limits.resize(capacity, u16::MAX); // MAX = no limit (SIMD-friendly encoding)
        result.snooze_periods.resize(capacity, 0);
        result.mp_charge_accumulation.resize(capacity, true);
        result.cortical_areas.resize(capacity, 0);
        result.coordinates.resize(capacity * 3, 0);
        result.valid_mask.resize(capacity, false);
        result
    }

    /// Add a neuron with simplified parameters (for backward compatibility)
    pub fn add_neuron_simple(
        &mut self,
        threshold: T,
        leak: f32,
        refractory_period: u16,
        excitability: f32,
    ) -> usize {
        // Call the full version with defaults
        NeuronStorage::add_neuron(
            self,
            threshold,
            T::max_value(), // threshold_limit (MAX = no limit, SIMD-friendly encoding)
            leak,
            T::zero(), // resting potential
            0,         // neuron type (excitatory)
            refractory_period,
            excitability,
            u16::MAX,  // consecutive fire limit (MAX = unlimited, SIMD-friendly encoding)
            0,         // snooze period
            true,      // mp_charge_accumulation
            0,         // cortical area
            0,
            0,
            0, // x, y, z coords
        )
        .expect("Failed to add neuron")
    }

    /// Process burst in parallel using Rayon
    ///
    /// Uses platform-agnostic core functions internally.
    ///
    /// NOTE: Due to Rust's borrowing rules, parallel processing computes
    /// results first, then applies mutations sequentially. For very small
    /// networks (<100 neurons), sequential may be faster.
    pub fn process_burst_parallel(
        &mut self,
        candidate_potentials: &[T],
        _burst_count: u64,
    ) -> Vec<usize> {
        // Phase 1: Compute in parallel (read-only)
        let results: Vec<_> = (0..self.count)
            .into_par_iter()
            .map(|idx| {
                if !self.valid_mask[idx] {
                    return (idx, false, T::zero());
                }

                let in_refractory = self.refractory_countdowns[idx] > 0;
                if in_refractory {
                    return (idx, false, T::zero());
                }

                // Simulate neuron update (read-only)
                let mut potential = self.membrane_potentials[idx];
                let input = candidate_potentials.get(idx).copied().unwrap_or(T::zero());
                let fired = update_neuron_lif(
                    &mut potential,
                    self.thresholds[idx],
                    self.leak_coefficients[idx],
                    T::zero(),
                    input,
                );

                (idx, fired, potential)
            })
            .collect();

        // Phase 2: Apply mutations sequentially
        let mut fired_indices = Vec::new();
        for (idx, fired, new_potential) in results {
            // Apply refractory countdown
            if self.refractory_countdowns[idx] > 0 {
                self.refractory_countdowns[idx] -= 1;
                continue;
            }

            // Apply potential update
            self.membrane_potentials[idx] = new_potential;

            if fired {
                self.refractory_countdowns[idx] = self.refractory_periods[idx];
                fired_indices.push(idx);
            }
        }

        fired_indices
    }

    /// Process burst sequentially (single-threaded)
    pub fn process_burst_sequential(
        &mut self,
        candidate_potentials: &[T],
        _burst_count: u64,
    ) -> Vec<usize> {
        let mut fired_indices = Vec::new();

        for idx in 0..self.count {
            if !self.valid_mask[idx] {
                continue;
            }

            if is_refractory(&mut self.refractory_countdowns[idx]) {
                continue;
            }

            let input = candidate_potentials.get(idx).copied().unwrap_or(T::zero());
            let fired = update_neuron_lif(
                &mut self.membrane_potentials[idx],
                self.thresholds[idx],
                self.leak_coefficients[idx],
                T::zero(),
                input,
            );

            if fired {
                self.refractory_countdowns[idx] = self.refractory_periods[idx];
                fired_indices.push(idx);
            }
        }

        fired_indices
    }
}

// Implement NeuronStorage trait for runtime abstraction
impl<T: NeuralValue> NeuronStorage for NeuronArray<T> {
    type Value = T;

    // Read-only property accessors
    fn membrane_potentials(&self) -> &[Self::Value] {
        &self.membrane_potentials[..self.count]
    }

    fn thresholds(&self) -> &[Self::Value] {
        &self.thresholds[..self.count]
    }

    fn threshold_limits(&self) -> &[Self::Value] {
        &self.threshold_limits[..self.count]
    }

    fn leak_coefficients(&self) -> &[f32] {
        &self.leak_coefficients[..self.count]
    }

    fn resting_potentials(&self) -> &[Self::Value] {
        &self.resting_potentials[..self.count]
    }

    fn neuron_types(&self) -> &[i32] {
        &self.neuron_types[..self.count]
    }

    fn refractory_periods(&self) -> &[u16] {
        &self.refractory_periods[..self.count]
    }

    fn refractory_countdowns(&self) -> &[u16] {
        &self.refractory_countdowns[..self.count]
    }

    fn excitabilities(&self) -> &[f32] {
        &self.excitabilities[..self.count]
    }

    fn consecutive_fire_counts(&self) -> &[u16] {
        &self.consecutive_fire_counts[..self.count]
    }

    fn consecutive_fire_limits(&self) -> &[u16] {
        &self.consecutive_fire_limits[..self.count]
    }

    fn snooze_periods(&self) -> &[u16] {
        &self.snooze_periods[..self.count]
    }

    fn mp_charge_accumulation(&self) -> &[bool] {
        &self.mp_charge_accumulation[..self.count]
    }

    fn cortical_areas(&self) -> &[u32] {
        &self.cortical_areas[..self.count]
    }

    fn coordinates(&self) -> &[u32] {
        &self.coordinates[..self.count * 3]
    }

    fn valid_mask(&self) -> &[bool] {
        &self.valid_mask[..self.count]
    }

    // Mutable property accessors
    fn membrane_potentials_mut(&mut self) -> &mut [Self::Value] {
        let count = self.count;
        &mut self.membrane_potentials[..count]
    }

    fn thresholds_mut(&mut self) -> &mut [Self::Value] {
        let count = self.count;
        &mut self.thresholds[..count]
    }

    fn threshold_limits_mut(&mut self) -> &mut [Self::Value] {
        let count = self.count;
        &mut self.threshold_limits[..count]
    }

    fn leak_coefficients_mut(&mut self) -> &mut [f32] {
        let count = self.count;
        &mut self.leak_coefficients[..count]
    }

    fn resting_potentials_mut(&mut self) -> &mut [Self::Value] {
        let count = self.count;
        &mut self.resting_potentials[..count]
    }

    fn neuron_types_mut(&mut self) -> &mut [i32] {
        let count = self.count;
        &mut self.neuron_types[..count]
    }

    fn refractory_periods_mut(&mut self) -> &mut [u16] {
        let count = self.count;
        &mut self.refractory_periods[..count]
    }

    fn refractory_countdowns_mut(&mut self) -> &mut [u16] {
        let count = self.count;
        &mut self.refractory_countdowns[..count]
    }

    fn excitabilities_mut(&mut self) -> &mut [f32] {
        let count = self.count;
        &mut self.excitabilities[..count]
    }

    fn consecutive_fire_counts_mut(&mut self) -> &mut [u16] {
        let count = self.count;
        &mut self.consecutive_fire_counts[..count]
    }

    fn consecutive_fire_limits_mut(&mut self) -> &mut [u16] {
        let count = self.count;
        &mut self.consecutive_fire_limits[..count]
    }

    fn snooze_periods_mut(&mut self) -> &mut [u16] {
        let count = self.count;
        &mut self.snooze_periods[..count]
    }

    fn mp_charge_accumulation_mut(&mut self) -> &mut [bool] {
        let count = self.count;
        &mut self.mp_charge_accumulation[..count]
    }

    fn valid_mask_mut(&mut self) -> &mut [bool] {
        let count = self.count;
        &mut self.valid_mask[..count]
    }

    // Metadata
    fn count(&self) -> usize {
        self.count
    }

    fn capacity(&self) -> usize {
        self.membrane_potentials.len()
    }

    // Neuron creation
    fn add_neuron(
        &mut self,
        threshold: Self::Value,
        threshold_limit: Self::Value,
        leak: f32,
        resting: Self::Value,
        neuron_type: i32,
        refractory_period: u16,
        excitability: f32,
        consecutive_fire_limit: u16,
        snooze_period: u16,
        mp_charge_accumulation: bool,
        cortical_area: u32,
        x: u32,
        y: u32,
        z: u32,
    ) -> Result<usize> {
        let idx = self.count;

        // Grow if needed
        if idx >= self.membrane_potentials.len() {
            self.membrane_potentials.push(T::zero());
            self.thresholds.push(threshold);
            self.threshold_limits.push(threshold_limit);
            self.leak_coefficients.push(leak);
            self.resting_potentials.push(resting);
            self.neuron_types.push(neuron_type);
            self.refractory_periods.push(refractory_period);
            self.refractory_countdowns.push(0);
            self.excitabilities.push(excitability);
            self.consecutive_fire_counts.push(0);
            self.consecutive_fire_limits.push(consecutive_fire_limit);
            self.snooze_periods.push(snooze_period);
            self.mp_charge_accumulation.push(mp_charge_accumulation);
            self.cortical_areas.push(cortical_area);
            self.coordinates.push(x);
            self.coordinates.push(y);
            self.coordinates.push(z);
            self.valid_mask.push(true);
        } else {
            self.thresholds[idx] = threshold;
            self.threshold_limits[idx] = threshold_limit;
            self.leak_coefficients[idx] = leak;
            self.resting_potentials[idx] = resting;
            self.neuron_types[idx] = neuron_type;
            self.refractory_periods[idx] = refractory_period;
            self.excitabilities[idx] = excitability;
            self.consecutive_fire_limits[idx] = consecutive_fire_limit;
            self.snooze_periods[idx] = snooze_period;
            self.mp_charge_accumulation[idx] = mp_charge_accumulation;
            self.cortical_areas[idx] = cortical_area;
            self.coordinates[idx * 3] = x;
            self.coordinates[idx * 3 + 1] = y;
            self.coordinates[idx * 3 + 2] = z;
            self.valid_mask[idx] = true;
        }

        self.count += 1;
        
        // Note: Cache invalidation is handled in add_neurons_batch for efficiency
        // For single neuron adds, we invalidate here (less common path)
        if let Ok(mut cache) = self.coord_map_cache.lock() {
            cache.remove(&cortical_area);
        }
        
        Ok(idx)
    }

    fn add_neurons_batch(
        &mut self,
        thresholds: &[Self::Value],
        threshold_limits: &[Self::Value],
        leak_coefficients: &[f32],
        resting_potentials: &[Self::Value],
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
    ) -> Result<()> {
        let n = thresholds.len();

        // Validate all slices are same length
        if threshold_limits.len() != n
            || leak_coefficients.len() != n
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
            return Err(crate::traits::RuntimeError::InvalidParameters(
                "Batch neuron creation: all slices must have same length".into(),
            ));
        }

        // Collect all affected cortical areas for cache invalidation
        let mut affected_areas = std::collections::HashSet::new();
        
        for i in 0..n {
            affected_areas.insert(cortical_areas[i]);
            self.add_neuron(
                thresholds[i],
                threshold_limits[i],
                leak_coefficients[i],
                resting_potentials[i],
                neuron_types[i],
                refractory_periods[i],
                excitabilities[i],
                consecutive_fire_limits[i],
                snooze_periods[i],
                mp_charge_accumulations[i],
                cortical_areas[i],
                x_coords[i],
                y_coords[i],
                z_coords[i],
            )?;
        }

        // Invalidate cache for all affected areas (more efficient than per-neuron invalidation)
        if let Ok(mut cache) = self.coord_map_cache.lock() {
            for area in affected_areas {
                cache.remove(&area);
            }
        }

        Ok(())
    }

    fn get_neuron_at_coordinate(
        &self,
        cortical_area: u32,
        x: u32,
        y: u32,
        z: u32,
    ) -> Option<usize> {
        // Linear search through coordinates
        (0..self.count).find(|&idx| {
            self.valid_mask[idx]
                && self.cortical_areas[idx] == cortical_area
                && self.coordinates[idx * 3] == x
                && self.coordinates[idx * 3 + 1] == y
                && self.coordinates[idx * 3 + 2] == z
        })
    }

    fn get_neurons_in_cortical_area(&self, cortical_area: u32) -> Vec<usize> {
        (0..self.count)
            .filter(|&idx| self.valid_mask[idx] && self.cortical_areas[idx] == cortical_area)
            .collect()
    }

    fn get_neuron_count(&self, cortical_area: u32) -> usize {
        (0..self.count)
            .filter(|&idx| self.valid_mask[idx] && self.cortical_areas[idx] == cortical_area)
            .count()
    }

    fn get_cortical_area(&self, neuron_idx: usize) -> Option<u32> {
        if neuron_idx < self.count && self.valid_mask[neuron_idx] {
            Some(self.cortical_areas[neuron_idx])
        } else {
            None
        }
    }

    fn get_coordinates(&self, neuron_idx: usize) -> Option<(u32, u32, u32)> {
        if neuron_idx < self.count && self.valid_mask[neuron_idx] {
            Some((
                self.coordinates[neuron_idx * 3],
                self.coordinates[neuron_idx * 3 + 1],
                self.coordinates[neuron_idx * 3 + 2],
            ))
        } else {
            None
        }
    }

    fn batch_coordinate_lookup(
        &self,
        cortical_area: u32,
        coords: &[(u32, u32, u32)],
    ) -> Vec<Option<usize>> {
        // OPTIMIZATION: Sequential hash map building with AHashMap + cache-friendly access
        // This changes complexity from O(n*m) to O(m + n) where n=coords, m=neurons in area
        // Sequential iteration has better cache locality than parallel for this workload
        // AHashMap provides 2-3x faster hashing than default SipHash

        // OPTIMIZATION: Use better capacity estimate to reduce hash map reallocations
        // Estimate: typically all or most neurons in area will be in the map, so use max of
        // coordinate count and a reasonable estimate based on total neurons
        // This avoids counting pass while still providing good capacity hint
        let capacity_estimate = coords.len().max(self.count / 4); // Assume area has at least 25% of neurons
        let mut coord_map: AHashMap<(u32, u32, u32), usize> =
            AHashMap::with_capacity(capacity_estimate);

        // Sequential iteration: better cache locality, compiler can auto-vectorize
        // Access patterns: valid_mask[idx], cortical_areas[idx], coordinates[idx*3..idx*3+3]
        // These are contiguous in memory, allowing SIMD auto-vectorization
        for idx in 0..self.count {
            // Fast path: early exit if not valid or wrong area (branch prediction friendly)
            if !self.valid_mask[idx] || self.cortical_areas[idx] != cortical_area {
                continue;
            }
            // Extract coordinates (cache-friendly: sequential access, SIMD-friendly pattern)
            let coord_base = idx * 3;
            let x = self.coordinates[coord_base];
            let y = self.coordinates[coord_base + 1];
            let z = self.coordinates[coord_base + 2];
            coord_map.insert((x, y, z), idx);
        }

        // Fast O(1) lookups for each coordinate
        coords
            .iter()
            .map(|&coord| coord_map.get(&coord).copied())
            .collect()
    }

    /// Optimized version that accepts separate slices to avoid tuple allocation
    fn batch_coordinate_lookup_from_slices(
        &self,
        cortical_area: u32,
        x_coords: &[u32],
        y_coords: &[u32],
        z_coords: &[u32],
    ) -> Vec<Option<usize>> {
        // Validate input lengths match
        if x_coords.len() != y_coords.len() || x_coords.len() != z_coords.len() {
            return vec![None; x_coords.len().max(y_coords.len()).max(z_coords.len())];
        }

        // CRITICAL PERFORMANCE: Use cached coordinate map if available
        // This eliminates O(n) scan through all neurons on every lookup
        let mut cache = self.coord_map_cache.lock().unwrap();
        
        // Check if cache exists for this cortical area
        if !cache.contains_key(&cortical_area) {
            // Cache miss - build the map and store it
            let capacity_estimate = self.count / 4; // Estimate based on total neurons
            let mut coord_map: AHashMap<(u32, u32, u32), usize> =
                AHashMap::with_capacity(capacity_estimate);

            // Build coordinate map from neuron storage (O(n) operation, but only once per area)
            for idx in 0..self.count {
                if !self.valid_mask[idx] || self.cortical_areas[idx] != cortical_area {
                    continue;
                }
                let coord_base = idx * 3;
                let x = self.coordinates[coord_base];
                let y = self.coordinates[coord_base + 1];
                let z = self.coordinates[coord_base + 2];
                coord_map.insert((x, y, z), idx);
            }
            
            // Store in cache for future lookups
            cache.insert(cortical_area, coord_map);
        }
        
        // Get reference to cached map and perform lookups while holding the lock
        // This is safe because we're already in a read lock context (single-threaded)
        let coord_map = cache.get(&cortical_area).unwrap();
        
        // Fast O(1) lookups using cached map
        x_coords
            .iter()
            .zip(y_coords.iter())
            .zip(z_coords.iter())
            .map(|((&x, &y), &z)| coord_map.get(&(x, y, z)).copied())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_neuron_f32() {
        let mut array = NeuronArray::<f32>::new(10);
        let idx = array.add_neuron_simple(1.0, 0.1, 5, 1.0);
        assert_eq!(idx, 0);
        assert_eq!(array.count, 1);
    }

    #[test]
    fn test_process_burst_sequential_f32() {
        let mut array = NeuronArray::<f32>::new(10);
        array.add_neuron_simple(1.0, 0.1, 5, 1.0);
        array.add_neuron_simple(1.0, 0.1, 5, 1.0);

        // High input - should fire
        let inputs = Vec::from([1.5, 0.5]); // First fires, second doesn't
        let fired = array.process_burst_sequential(&inputs, 0);

        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0], 0);
    }

    #[test]
    fn test_process_burst_parallel_f32() {
        let mut array = NeuronArray::<f32>::new(100);
        for _ in 0..100 {
            array.add_neuron_simple(1.0, 0.1, 5, 1.0);
        }

        let inputs = Vec::from([1.5; 100]); // All should fire
        let fired = array.process_burst_parallel(&inputs, 0);

        assert_eq!(fired.len(), 100);
    }
}
