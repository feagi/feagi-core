// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Runtime abstraction traits for cross-platform neural processing
//!
//! This module defines the core traits that enable FEAGI to run on different platforms:
//! - Desktop/Server (Vec-based, dynamic allocation)
//! - Embedded (fixed arrays, no_std)
//! - GPU (CUDA VRAM, GPU memory)
//! - WASM (WebAssembly.Memory, typed arrays)
//!
//! ## Design Philosophy
//!
//! - **Storage Abstraction**: Separate "what" from "how" (types vs storage)
//! - **Zero-Cost**: Traits compile to direct calls (no runtime overhead)
//! - **Platform-Agnostic**: Same burst engine code works everywhere
//! - **Type-Safe**: Compile-time guarantees for platform compatibility

use crate::traits::error::Result;
use crate::traits::NeuralValue;

#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

/// Runtime trait: Abstracts storage implementation and platform capabilities
///
/// This is the primary trait that platforms implement to provide neural storage.
/// Each runtime provides its own storage implementations (Vec, fixed arrays, GPU buffers).
///
/// # Example
///
/// ```ignore
/// // Desktop runtime with dynamic allocation
/// pub struct StdRuntime;
///
/// impl Runtime for StdRuntime {
///     type NeuronStorage<T: NeuralValue> = StdNeuronArray<T>;
///     type SynapseStorage = StdSynapseArray;
///
///     fn create_neuron_storage<T: NeuralValue>(&self, capacity: usize) -> Result<Self::NeuronStorage<T>> {
///         Ok(StdNeuronArray::new(capacity))
///     }
///
///     fn supports_parallel(&self) -> bool { true }
///     fn memory_limit(&self) -> Option<usize> { None }
/// }
/// ```
pub trait Runtime: Send + Sync {
    /// Neuron storage type (generic over value type T)
    type NeuronStorage<T>: NeuronStorage<Value = T>
    where
        T: NeuralValue;

    /// Synapse storage type
    type SynapseStorage: SynapseStorage;

    /// Create neuron storage with specified capacity
    fn create_neuron_storage<T: NeuralValue>(
        &self,
        capacity: usize,
    ) -> Result<Self::NeuronStorage<T>>;

    /// Create synapse storage with specified capacity
    fn create_synapse_storage(&self, capacity: usize) -> Result<Self::SynapseStorage>;

    /// Platform supports parallel processing (multi-threading, SIMD)
    fn supports_parallel(&self) -> bool;

    /// Platform supports SIMD vectorization
    fn supports_simd(&self) -> bool {
        true // Most platforms do
    }

    /// Platform memory limit (None = unlimited)
    fn memory_limit(&self) -> Option<usize>;

    /// Platform name for logging/debugging
    fn platform_name(&self) -> &'static str {
        "Generic Runtime"
    }
}

/// Neuron storage trait: Abstracts System-of-Arrays (SoA) for neurons
///
/// This trait provides access to neuron properties stored in a platform-specific way.
/// Implementations might use Vec (std), fixed arrays (embedded), or GPU buffers (CUDA).
///
/// # Design Notes
///
/// - Uses slice-based API for zero-copy access
/// - All properties return slices for efficient batch operations
/// - Mutations are explicit via `_mut()` methods
pub trait NeuronStorage: Send + Sync {
    /// Numeric type for membrane potentials (f32, INT8Value, etc.)
    type Value: NeuralValue;

    // === Neuron Properties (Read-Only) ===

    /// Membrane potentials slice
    fn membrane_potentials(&self) -> &[Self::Value];

    /// Firing thresholds slice (minimum MP to fire)
    fn thresholds(&self) -> &[Self::Value];

    /// Firing threshold limits slice (maximum MP to fire, 0 = no limit)
    fn threshold_limits(&self) -> &[Self::Value];

    /// Leak coefficients slice (0.0-1.0)
    fn leak_coefficients(&self) -> &[f32];

    /// Resting potentials slice
    fn resting_potentials(&self) -> &[Self::Value];

    /// Neuron types slice (0=excitatory, 1=inhibitory, etc.)
    fn neuron_types(&self) -> &[i32];

    /// Refractory periods slice (burst counts)
    fn refractory_periods(&self) -> &[u16];

    /// Refractory countdowns slice (current state)
    fn refractory_countdowns(&self) -> &[u16];

    /// Excitability factors slice (0.0-1.0)
    fn excitabilities(&self) -> &[f32];

    /// Consecutive fire counts slice
    fn consecutive_fire_counts(&self) -> &[u16];

    /// Consecutive fire limits slice (0 = unlimited)
    fn consecutive_fire_limits(&self) -> &[u16];

    /// Snooze periods slice (extended refractory)
    fn snooze_periods(&self) -> &[u16];

    /// Membrane potential charge accumulation flags
    fn mp_charge_accumulation(&self) -> &[bool];

    /// Cortical area IDs slice
    fn cortical_areas(&self) -> &[u32];

    /// 3D coordinates slice (flat array: [x0, y0, z0, x1, y1, z1, ...])
    fn coordinates(&self) -> &[u32];

    /// Valid neuron mask
    fn valid_mask(&self) -> &[bool];

    // === Neuron Properties (Mutable) ===

    /// Mutable membrane potentials slice
    fn membrane_potentials_mut(&mut self) -> &mut [Self::Value];

    /// Mutable firing thresholds slice
    fn thresholds_mut(&mut self) -> &mut [Self::Value];

    /// Mutable firing threshold limits slice
    fn threshold_limits_mut(&mut self) -> &mut [Self::Value];

    /// Mutable leak coefficients slice
    fn leak_coefficients_mut(&mut self) -> &mut [f32];

    /// Mutable resting potentials slice
    fn resting_potentials_mut(&mut self) -> &mut [Self::Value];

    /// Mutable neuron types slice
    fn neuron_types_mut(&mut self) -> &mut [i32];

    /// Mutable refractory periods slice
    fn refractory_periods_mut(&mut self) -> &mut [u16];

    /// Mutable refractory countdowns slice
    fn refractory_countdowns_mut(&mut self) -> &mut [u16];

    /// Mutable excitability factors slice
    fn excitabilities_mut(&mut self) -> &mut [f32];

    /// Mutable consecutive fire counts slice
    fn consecutive_fire_counts_mut(&mut self) -> &mut [u16];

    /// Mutable consecutive fire limits slice
    fn consecutive_fire_limits_mut(&mut self) -> &mut [u16];

    /// Mutable snooze periods slice
    fn snooze_periods_mut(&mut self) -> &mut [u16];

    /// Mutable membrane potential charge accumulation flags
    fn mp_charge_accumulation_mut(&mut self) -> &mut [bool];

    /// Mutable valid mask
    fn valid_mask_mut(&mut self) -> &mut [bool];

    // === Metadata ===

    /// Number of neurons currently stored
    fn count(&self) -> usize;

    /// Maximum capacity
    fn capacity(&self) -> usize;

    // === Neuron Creation ===

    /// Add a single neuron
    #[allow(clippy::too_many_arguments)] // Trait method - cannot refactor without breaking API
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
    ) -> Result<usize>;

    /// Batch add neurons (SIMD-optimized)
    ///
    /// Note: Return type uses Vec which requires either std or alloc feature.
    /// For no_std without alloc, implementations should use a fixed-size approach.
    #[allow(clippy::too_many_arguments)] // Trait method - cannot refactor without breaking API
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
    ) -> Result<()>; // Changed to Result<()> to avoid Vec requirement

    // === Query Methods ===

    /// Get neuron at specific 3D coordinate in a cortical area
    fn get_neuron_at_coordinate(&self, cortical_area: u32, x: u32, y: u32, z: u32)
        -> Option<usize>;

    /// Get all neuron indices in a cortical area
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn get_neurons_in_cortical_area(&self, cortical_area: u32) -> Vec<usize>;

    /// Get count of neurons in a cortical area
    fn get_neuron_count(&self, cortical_area: u32) -> usize;

    /// Get cortical area ID for a neuron
    fn get_cortical_area(&self, neuron_idx: usize) -> Option<u32>;

    /// Get 3D coordinates for a neuron
    fn get_coordinates(&self, neuron_idx: usize) -> Option<(u32, u32, u32)>;

    /// Batch lookup neurons by coordinates
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn batch_coordinate_lookup(
        &self,
        cortical_area: u32,
        coords: &[(u32, u32, u32)],
    ) -> Vec<Option<usize>>;
}

/// Synapse storage trait: Abstracts System-of-Arrays (SoA) for synapses
///
/// Provides access to synaptic connections stored in a platform-specific way.
pub trait SynapseStorage: Send + Sync {
    // === Synapse Properties (Read-Only) ===

    /// Source neuron IDs slice
    fn source_neurons(&self) -> &[u32];

    /// Target neuron IDs slice
    fn target_neurons(&self) -> &[u32];

    /// Synaptic weights slice (0-255, stored as u8)
    fn weights(&self) -> &[u8];

    /// Postsynaptic potentials slice (conductances, 0-255)
    fn postsynaptic_potentials(&self) -> &[u8];

    /// Synapse types slice (0=excitatory, 1=inhibitory)
    fn types(&self) -> &[u8];

    /// Valid synapse mask
    fn valid_mask(&self) -> &[bool];

    // === Synapse Properties (Mutable) ===

    /// Mutable weights slice
    fn weights_mut(&mut self) -> &mut [u8];

    /// Mutable postsynaptic potentials slice
    fn postsynaptic_potentials_mut(&mut self) -> &mut [u8];

    /// Mutable valid mask
    fn valid_mask_mut(&mut self) -> &mut [bool];

    // === Metadata ===

    /// Number of synapses currently stored
    fn count(&self) -> usize;

    /// Maximum capacity
    fn capacity(&self) -> usize;

    // === Synapse Creation ===

    /// Add a single synapse
    fn add_synapse(
        &mut self,
        source: u32,
        target: u32,
        weight: u8,
        psp: u8,
        synapse_type: u8,
    ) -> Result<usize>;

    // === Batch Operations ===

    /// Batch add synapses
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn add_synapses_batch(
        &mut self,
        sources: &[u32],
        targets: &[u32],
        weights: &[u8],
        psps: &[u8],
        types: &[u8],
    ) -> Result<()>;

    // === Synapse Removal ===

    /// Remove a single synapse by index
    fn remove_synapse(&mut self, idx: usize) -> Result<()>;

    /// Remove all synapses from specific source neurons
    fn remove_synapses_from_sources(&mut self, source_neurons: &[u32]) -> Result<usize>;

    /// Remove synapses between specific source and target
    fn remove_synapses_between(&mut self, source: u32, target: u32) -> Result<usize>;

    // === Synapse Updates ===

    /// Update weight of a synapse
    fn update_weight(&mut self, idx: usize, new_weight: u8) -> Result<()>;

    // === Query Methods ===

    /// Get count of valid (non-deleted) synapses
    fn valid_count(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    // These are trait contract tests to ensure the traits are well-defined

    // Test that traits have required bounds
    #[allow(dead_code)] // Compile-time assertions for trait bounds
    fn assert_runtime_bounds<R: Runtime>() {}

    #[allow(dead_code)] // Compile-time assertions for trait bounds
    fn assert_neuron_storage_bounds<T: NeuralValue, N: NeuronStorage<Value = T>>() {}

    #[allow(dead_code)] // Compile-time assertions for trait bounds
    fn assert_synapse_storage_bounds<S: SynapseStorage>() {}

    #[test]
    fn test_trait_bounds_compile() {
        // This test ensures the traits compile and have correct bounds
        // Actual implementations will be tested in runtime-specific crates
    }
}
