// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Fixed-size neuron array for embedded systems
//!
//! Uses stack-allocated arrays for predictable memory usage.

use crate::traits::{NeuronStorage, Result, RuntimeError};
use feagi_npu_neural::types::NeuralValue;
use feagi_npu_neural::{is_refractory, update_neuron_lif};

#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

#[cfg(any(feature = "std", feature = "alloc"))]
use alloc::vec::Vec;

/// Fixed-size neuron array for embedded systems
///
/// All data is stack-allocated with compile-time size limits.
/// No heap allocations, perfect for `no_std` environments.
/// Generic over `T: NeuralValue` to support multiple quantization levels.
///
/// # Example
/// ```
/// use feagi_npu_runtime::embedded::NeuronArray;
///
/// // 100-neuron array on the stack (~5 KB for f32)
/// let mut neurons = NeuronArray::<f32, 100>::new();
/// neurons.add_neuron_simple(1.0, 0.1, 5, 1.0);
/// ```
pub struct NeuronArray<T: NeuralValue, const N: usize> {
    /// Current number of neurons
    pub count: usize,

    /// Membrane potentials (quantized to T)
    pub membrane_potentials: [T; N],

    /// Firing thresholds (quantized to T)
    pub thresholds: [T; N],

    /// Leak coefficients (kept as f32 for precision)
    pub leak_coefficients: [f32; N],

    /// Resting potentials
    pub resting_potentials: [T; N],

    /// Neuron types (0=excitatory, 1=inhibitory)
    pub neuron_types: [i32; N],

    /// Refractory periods
    pub refractory_periods: [u16; N],

    /// Refractory countdowns (state)
    pub refractory_countdowns: [u16; N],

    /// Excitability factors
    pub excitabilities: [f32; N],

    /// Consecutive fire counts
    pub consecutive_fire_counts: [u16; N],

    /// Consecutive fire limits
    pub consecutive_fire_limits: [u16; N],

    /// Snooze periods (extended refractory)
    pub snooze_periods: [u16; N],

    /// Membrane potential charge accumulation flags
    pub mp_charge_accumulation: [bool; N],

    /// Cortical area IDs
    pub cortical_areas: [u32; N],

    /// 3D coordinates (flat: [x0,y0,z0, x1,y1,z1, ...])
    pub coordinates: [u32; N], // Will need N*3, simplified for now

    /// Valid mask
    pub valid_mask: [bool; N],
}

impl<T: NeuralValue, const N: usize> NeuronArray<T, N> {
    /// Create a new fixed-size neuron array
    ///
    /// All arrays are zero-initialized on the stack.
    /// Note: Not const due to T::zero() trait method
    pub fn new() -> Self {
        Self {
            count: 0,
            membrane_potentials: [T::zero(); N],
            thresholds: [T::from_f32(1.0); N],
            leak_coefficients: [0.1; N],
            resting_potentials: [T::zero(); N],
            neuron_types: [0; N],
            refractory_periods: [0; N],
            refractory_countdowns: [0; N],
            excitabilities: [1.0; N],
            consecutive_fire_counts: [0; N],
            consecutive_fire_limits: [0; N],
            snooze_periods: [0; N],
            mp_charge_accumulation: [true; N],
            cortical_areas: [0; N],
            coordinates: [0; N],
            valid_mask: [false; N],
        }
    }

    /// Add a neuron (simplified for backward compatibility)
    ///
    /// Returns the neuron index, or None if array is full.
    pub fn add_neuron_simple(
        &mut self,
        threshold: T,
        leak: f32,
        refractory_period: u16,
        excitability: f32,
    ) -> Option<usize> {
        NeuronStorage::add_neuron(
            self,
            threshold,
            leak,
            T::zero(), // resting potential
            0,         // neuron type (excitatory)
            refractory_period,
            excitability,
            0,    // consecutive fire limit (unlimited)
            0,    // snooze period
            true, // mp_charge_accumulation
            0,    // cortical area
            0,
            0,
            0, // x, y, z coords
        )
        .ok()
    }

    /// Process burst (single-threaded, deterministic)
    ///
    /// Uses platform-agnostic core functions internally.
    /// Returns number of neurons that fired.
    ///
    /// # Arguments
    /// * `candidate_potentials` - Input currents for each neuron
    /// * `fired_mask` - Output: which neurons fired (caller-allocated)
    ///
    /// # Example
    /// ```
    /// let mut neurons = NeuronArray::<f32, 100>::new();
    /// let inputs = [1.5; 100];
    /// let mut fired = [false; 100];
    /// let count = neurons.process_burst(&inputs, &mut fired);
    /// ```
    pub fn process_burst(
        &mut self,
        candidate_potentials: &[T; N],
        fired_mask: &mut [bool; N],
    ) -> usize {
        let mut fired_count = 0;

        for idx in 0..self.count {
            fired_mask[idx] = false;

            if !self.valid_mask[idx] {
                continue;
            }

            // Check refractory (uses platform-agnostic function)
            if is_refractory(&mut self.refractory_countdowns[idx]) {
                continue;
            }

            // Update neuron (uses platform-agnostic function)
            let input = candidate_potentials[idx];
            let fired = update_neuron_lif(
                &mut self.membrane_potentials[idx],
                self.thresholds[idx],
                self.leak_coefficients[idx],
                T::zero(), // resting_potential
                input,
            );

            if fired {
                // Apply refractory
                self.refractory_countdowns[idx] = self.refractory_periods[idx];
                fired_mask[idx] = true;
                fired_count += 1;
            }
        }

        fired_count
    }

    /// Get memory footprint in bytes
    pub const fn memory_footprint() -> usize {
        core::mem::size_of::<Self>()
    }
}

// Implement NeuronStorage trait for runtime abstraction
impl<T: NeuralValue, const N: usize> NeuronStorage for NeuronArray<T, N> {
    type Value = T;

    fn membrane_potentials(&self) -> &[Self::Value] {
        &self.membrane_potentials[..self.count]
    }

    fn thresholds(&self) -> &[Self::Value] {
        &self.thresholds[..self.count]
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
        &self.coordinates[..self.count]
    }

    fn valid_mask(&self) -> &[bool] {
        &self.valid_mask[..self.count]
    }

    fn membrane_potentials_mut(&mut self) -> &mut [Self::Value] {
        let count = self.count;
        &mut self.membrane_potentials[..count]
    }

    fn thresholds_mut(&mut self) -> &mut [Self::Value] {
        let count = self.count;
        &mut self.thresholds[..count]
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

    fn count(&self) -> usize {
        self.count
    }

    fn capacity(&self) -> usize {
        N
    }

    fn add_neuron(
        &mut self,
        threshold: Self::Value,
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
        if self.count >= N {
            return Err(RuntimeError::CapacityExceeded {
                requested: self.count + 1,
                available: N,
            });
        }

        let idx = self.count;
        self.thresholds[idx] = threshold;
        self.leak_coefficients[idx] = leak;
        self.resting_potentials[idx] = resting;
        self.neuron_types[idx] = neuron_type;
        self.refractory_periods[idx] = refractory_period;
        self.excitabilities[idx] = excitability;
        self.consecutive_fire_limits[idx] = consecutive_fire_limit;
        self.snooze_periods[idx] = snooze_period;
        self.mp_charge_accumulation[idx] = mp_charge_accumulation;
        self.cortical_areas[idx] = cortical_area;
        self.coordinates[idx] = x; // Simplified: storing only x coordinate
        self.valid_mask[idx] = true;

        self.count += 1;
        Ok(idx)
    }

    fn add_neurons_batch(
        &mut self,
        thresholds: &[Self::Value],
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

        if self.count + n > N {
            return Err(RuntimeError::CapacityExceeded {
                requested: self.count + n,
                available: N,
            });
        }

        for i in 0..n {
            self.add_neuron(
                thresholds[i],
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

        Ok(())
    }

    fn get_neuron_at_coordinate(
        &self,
        cortical_area: u32,
        x: u32,
        _y: u32,
        _z: u32,
    ) -> Option<usize> {
        // Linear search through neurons (embedded systems typically have small neuron counts)
        for idx in 0..self.count {
            if self.valid_mask[idx]
                && self.cortical_areas[idx] == cortical_area
                && self.coordinates[idx] == x
            // Simplified: only checking x coordinate
            {
                return Some(idx);
            }
        }
        None
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn get_neurons_in_cortical_area(&self, cortical_area: u32) -> Vec<usize> {
        let mut result = Vec::new();
        for idx in 0..self.count {
            if self.valid_mask[idx] && self.cortical_areas[idx] == cortical_area {
                result.push(idx);
            }
        }
        result
    }

    fn get_neuron_count(&self, cortical_area: u32) -> usize {
        let mut count = 0;
        for idx in 0..self.count {
            if self.valid_mask[idx] && self.cortical_areas[idx] == cortical_area {
                count += 1;
            }
        }
        count
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
            // Simplified: only x coordinate stored, return (x, 0, 0)
            Some((self.coordinates[neuron_idx], 0, 0))
        } else {
            None
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn batch_coordinate_lookup(
        &self,
        cortical_area: u32,
        coords: &[(u32, u32, u32)],
    ) -> Vec<Option<usize>> {
        coords
            .iter()
            .map(|(x, _y, _z)| self.get_neuron_at_coordinate(cortical_area, *x, 0, 0))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_f32() {
        let array = NeuronArray::<f32, 10>::new();
        assert_eq!(array.count, 0);
    }

    #[test]
    fn test_add_neuron_f32() {
        let mut array = NeuronArray::<f32, 10>::new();
        let idx = array.add_neuron_simple(1.0, 0.1, 5, 1.0);
        assert_eq!(idx, Some(0));
        assert_eq!(array.count, 1);
    }

    #[test]
    fn test_array_full_f32() {
        let mut array = NeuronArray::<f32, 2>::new();
        assert!(array.add_neuron_simple(1.0, 0.1, 5, 1.0).is_some());
        assert!(array.add_neuron_simple(1.0, 0.1, 5, 1.0).is_some());
        assert!(array.add_neuron_simple(1.0, 0.1, 5, 1.0).is_none()); // Full
    }

    #[test]
    fn test_process_burst_f32() {
        let mut array = NeuronArray::<f32, 10>::new();
        array.add_neuron_simple(1.0, 0.1, 5, 1.0);
        array.add_neuron_simple(1.0, 0.1, 5, 1.0);

        let mut inputs = [0.0; 10];
        inputs[0] = 1.5; // Should fire
        inputs[1] = 0.5; // Should not fire

        let mut fired = [false; 10];
        let count = array.process_burst(&inputs, &mut fired);

        assert_eq!(count, 1);
        assert!(fired[0]);
        assert!(!fired[1]);
    }

    #[test]
    fn test_memory_footprint_f32() {
        let size = NeuronArray::<f32, 100>::memory_footprint();
        // ~100 neurons × 48 bytes = ~4.8 KB
        assert!(size < 10_000); // Should be under 10 KB
    }
}
