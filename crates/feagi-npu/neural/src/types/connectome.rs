// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 */

//! Connectome snapshot types
//!
//! These types represent the serializable state of a complete connectome.
//! They are platform-agnostic and can be used for file I/O, network transport, etc.

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use ahash::AHashMap;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use std::prelude::rust_2021::*;  // Import Vec, String, etc. from std prelude

/// Serializable version of NeuronArray
///
/// This captures all neuron data from the RustNPU in a format
/// that can be efficiently serialized.
#[cfg(feature = "std")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableNeuronArray {
    /// Number of valid neurons
    pub count: usize,

    /// Capacity (pre-allocated size)
    pub capacity: usize,

    /// Membrane potentials (f32)
    pub membrane_potentials: Vec<f32>,

    /// Firing thresholds (f32)
    pub thresholds: Vec<f32>,

    /// Leak coefficients (f32, 0-1 range for exponential decay)
    pub leak_coefficients: Vec<f32>,

    /// Resting potentials (f32)
    pub resting_potentials: Vec<f32>,

    /// Neuron types (i32)
    pub neuron_types: Vec<i32>,

    /// Refractory periods (u16)
    pub refractory_periods: Vec<u16>,

    /// Current refractory countdowns (u16)
    pub refractory_countdowns: Vec<u16>,

    /// Excitability multipliers (f32)
    pub excitabilities: Vec<f32>,

    /// Cortical area IDs (u32)
    pub cortical_areas: Vec<u32>,

    /// 3D coordinates (flat array: [x0, y0, z0, x1, y1, z1, ...])
    pub coordinates: Vec<u32>,

    /// Valid mask (bool)
    pub valid_mask: Vec<bool>,
}

#[cfg(feature = "std")]
impl Default for SerializableNeuronArray {
    fn default() -> Self {
        Self {
            count: 0,
            capacity: 0,
            membrane_potentials: Vec::new(),
            thresholds: Vec::new(),
            leak_coefficients: Vec::new(),
            resting_potentials: Vec::new(),
            neuron_types: Vec::new(),
            refractory_periods: Vec::new(),
            refractory_countdowns: Vec::new(),
            excitabilities: Vec::new(),
            cortical_areas: Vec::new(),
            coordinates: Vec::new(),
            valid_mask: Vec::new(),
        }
    }
}

#[cfg(feature = "std")]
impl SerializableNeuronArray {
    /// Create a new empty neuron array
    pub fn new(capacity: usize) -> Self {
        Self {
            count: 0,
            capacity,
            membrane_potentials: std::vec::from_elem(0.0, capacity),
            thresholds: std::vec::from_elem(0.0, capacity),
            leak_coefficients: std::vec::from_elem(0.0, capacity),
            resting_potentials: std::vec::from_elem(0.0, capacity),
            neuron_types: std::vec::from_elem(0, capacity),
            refractory_periods: std::vec::from_elem(0, capacity),
            refractory_countdowns: std::vec::from_elem(0, capacity),
            excitabilities: std::vec::from_elem(1.0, capacity),
            cortical_areas: std::vec::from_elem(0, capacity),
            coordinates: std::vec::from_elem(0, capacity * 3), // x, y, z for each neuron
            valid_mask: std::vec::from_elem(false, capacity),
        }
    }
}

/// Serializable version of SynapseArray
///
/// This captures all synapse data from the RustNPU.
#[cfg(feature = "std")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableSynapseArray {
    /// Number of valid synapses
    pub count: usize,

    /// Capacity (pre-allocated size)
    pub capacity: usize,

    /// Source neuron IDs (u32)
    pub source_neurons: Vec<u32>,

    /// Target neuron IDs (u32)
    pub target_neurons: Vec<u32>,

    /// Synaptic weights (u8, 0-255)
    pub weights: Vec<u8>,

    /// Synaptic conductances (u8, 0-255)
    pub conductances: Vec<u8>,

    /// Synapse types (u8: 0=excitatory, 1=inhibitory)
    pub types: Vec<u8>,

    /// Valid mask (bool)
    pub valid_mask: Vec<bool>,

    /// Source neuron index (for fast lookup)
    /// Maps source_neuron_id -> Vec<synapse_index>
    pub source_index: AHashMap<u32, Vec<usize>>,
}

#[cfg(feature = "std")]
impl Default for SerializableSynapseArray {
    fn default() -> Self {
        Self {
            count: 0,
            capacity: 0,
            source_neurons: Vec::new(),
            target_neurons: Vec::new(),
            weights: Vec::new(),
            conductances: Vec::new(),
            types: Vec::new(),
            valid_mask: Vec::new(),
            source_index: AHashMap::new(),
        }
    }
}

#[cfg(feature = "std")]
impl SerializableSynapseArray {
    /// Create a new empty synapse array
    pub fn new(capacity: usize) -> Self {
        Self {
            count: 0,
            capacity,
            source_neurons: std::vec::from_elem(0, capacity),
            target_neurons: std::vec::from_elem(0, capacity),
            weights: std::vec::from_elem(0, capacity),
            conductances: std::vec::from_elem(0, capacity),
            types: std::vec::from_elem(0, capacity),
            valid_mask: std::vec::from_elem(false, capacity),
            source_index: AHashMap::new(),
        }
    }
}

/// Connectome metadata (for tracking and debugging)
#[cfg(feature = "std")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectomeMetadata {
    /// When this connectome was saved
    pub timestamp: u64,

    /// Human-readable description
    pub description: String,

    /// Source (e.g., "genome: essential_genome.json", "checkpoint: burst_12345")
    pub source: String,

    /// Custom tags for organization
    pub tags: AHashMap<String, String>,
}

#[cfg(feature = "std")]
impl Default for ConnectomeMetadata {
    fn default() -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            description: String::new(),
            source: String::from("unknown"),
            tags: AHashMap::new(),
        }
    }
}

/// Complete connectome snapshot
///
/// This structure captures the entire state of a RustNPU, including:
/// - All neurons and their properties
/// - All synapses and their weights
/// - Cortical area metadata
/// - Runtime state (burst count, etc.)
#[cfg(feature = "std")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectomeSnapshot {
    /// Format version (for backward compatibility)
    pub version: u32,

    /// Neuron data
    pub neurons: SerializableNeuronArray,

    /// Synapse data
    pub synapses: SerializableSynapseArray,

    /// Cortical area ID to name mapping (for visualization)
    pub cortical_area_names: AHashMap<u32, String>,

    /// Burst count (runtime state)
    pub burst_count: u64,

    /// Power injection amount
    pub power_amount: f32,

    /// Fire ledger window size
    pub fire_ledger_window: usize,

    /// Metadata (optional, for debugging/tracking)
    pub metadata: ConnectomeMetadata,
}

/// Statistics about a connectome
#[cfg(feature = "std")]
#[derive(Debug, Clone, Default)]
pub struct ConnectomeStatistics {
    pub neuron_count: usize,
    pub synapse_count: usize,
    pub active_synapse_count: usize,
    pub cortical_area_count: usize,
    pub avg_synaptic_weight: f32,
}

#[cfg(feature = "std")]
impl std::fmt::Display for ConnectomeStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Neurons: {}, Synapses: {} ({} active), Cortical Areas: {}, Avg Weight: {:.2}",
            self.neuron_count,
            self.synapse_count,
            self.active_synapse_count,
            self.cortical_area_count,
            self.avg_synaptic_weight
        )
    }
}

#[cfg(feature = "std")]
impl ConnectomeSnapshot {
    /// Get human-readable summary of the connectome
    pub fn summary(&self) -> String {
        std::format!(
            "Connectome v{}: {} neurons, {} synapses, {} cortical areas (burst: {})",
            self.version,
            self.neurons.count,
            self.synapses.count,
            self.cortical_area_names.len(),
            self.burst_count
        )
    }

    /// Validate the connectome structure
    pub fn validate(&self) -> Result<(), String> {
        // Check neuron array consistency
        if self.neurons.membrane_potentials.len() != self.neurons.capacity {
            return Err(std::format!(
                "Neuron array size mismatch: membrane_potentials.len()={}, capacity={}",
                self.neurons.membrane_potentials.len(),
                self.neurons.capacity
            ));
        }

        // Check synapse array consistency
        if self.synapses.source_neurons.len() != self.synapses.capacity {
            return Err(std::format!(
                "Synapse array size mismatch: source_neurons.len()={}, capacity={}",
                self.synapses.source_neurons.len(),
                self.synapses.capacity
            ));
        }

        // Check synapse references are valid
        for i in 0..self.synapses.count {
            if !self.synapses.valid_mask[i] {
                continue;
            }

            let source = self.synapses.source_neurons[i] as usize;
            let target = self.synapses.target_neurons[i] as usize;

            if source >= self.neurons.count {
            return Err(std::format!(
                "Synapse {} has invalid source neuron: {}",
                i, source
            ));
            }

            if target >= self.neurons.count {
            return Err(std::format!(
                "Synapse {} has invalid target neuron: {}",
                i, target
            ));
            }
        }

        Ok(())
    }

    /// Get statistics about the connectome
    pub fn statistics(&self) -> ConnectomeStatistics {
        let mut stats = ConnectomeStatistics::default();

        stats.neuron_count = self.neurons.count;
        stats.synapse_count = self.synapses.count;
        stats.cortical_area_count = self.cortical_area_names.len();

        // Count active synapses
        stats.active_synapse_count = self.synapses.valid_mask[..self.synapses.count]
            .iter()
            .filter(|&&v| v)
            .count();

        // Calculate average synaptic weight
        let total_weight: u32 = self.synapses.weights[..self.synapses.count]
            .iter()
            .map(|&w| w as u32)
            .sum();
        stats.avg_synaptic_weight = if stats.active_synapse_count > 0 {
            total_weight as f32 / stats.active_synapse_count as f32
        } else {
            0.0
        };

        stats
    }
}

