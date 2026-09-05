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
use std::collections::HashSet;
#[cfg(feature = "std")]
use std::prelude::rust_2021::*; // Import Vec, String, etc. from std prelude

/// Memory-neuron global IDs occupy this range (same partition as plasticity).
#[cfg(feature = "std")]
pub const MEMORY_NEURON_ID_START: u32 = 50_000_000;
#[cfg(feature = "std")]
pub const MEMORY_NEURON_ID_MAX: u32 = 99_999_999;

/// True when `neuron_id` is in the memory-neuron ID partition.
#[cfg(feature = "std")]
pub fn is_memory_neuron_id(neuron_id: u32) -> bool {
    (MEMORY_NEURON_ID_START..=MEMORY_NEURON_ID_MAX).contains(&neuron_id)
}

/// Long-term memory neuron persisted with a connectome snapshot.
///
/// Short-term memory neurons are never written to this list.
#[cfg(feature = "std")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializableLongTermMemoryNeuron {
    pub neuron_id: u32,
    pub cortical_area_idx: u32,
    pub pattern_hash: Option<u64>,
    pub is_longterm_memory: bool,
    pub is_active: bool,
    pub lifespan_current: u32,
    pub lifespan_initial: u32,
    pub lifespan_growth_rate: f32,
    pub creation_burst: u64,
    pub last_activation_burst: u64,
    pub activation_count: u32,
}

/// Learned episodic replay frame for one long-term memory neuron.
///
/// These are stored memory traces used to drive twin areas on recall.
/// Fire-ledger history is never persisted.
#[cfg(feature = "std")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializableMemoryReplayFrame {
    pub offset: u32,
    pub upstream_area_idx: u32,
    pub coords: Vec<(u32, u32, u32)>,
    pub membrane_potentials: Option<Vec<f32>>,
}

/// Serializable version of NeuronArray
///
/// This captures all neuron data from the RustNPU in a format
/// that can be efficiently serialized.
#[cfg(feature = "std")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

    /// Firing threshold limits (f32). Omitted in older snapshots.
    #[serde(default)]
    pub threshold_limits: Vec<f32>,

    /// Consecutive fire counts (u16). Omitted in older snapshots.
    #[serde(default)]
    pub consecutive_fire_counts: Vec<u16>,

    /// Consecutive fire limits (u16). Omitted in older snapshots.
    #[serde(default)]
    pub consecutive_fire_limits: Vec<u16>,

    /// Snooze periods (u16). Omitted in older snapshots.
    #[serde(default)]
    pub snooze_periods: Vec<u16>,

    /// Membrane-potential charge accumulation flags. Omitted in older snapshots.
    #[serde(default)]
    pub mp_charge_accumulation: Vec<bool>,
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
            threshold_limits: std::vec::from_elem(0.0, capacity),
            consecutive_fire_counts: std::vec::from_elem(0, capacity),
            consecutive_fire_limits: std::vec::from_elem(0, capacity),
            snooze_periods: std::vec::from_elem(0, capacity),
            mp_charge_accumulation: std::vec::from_elem(false, capacity),
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

    /// Synaptic weights (`f32`)
    pub weights: Vec<f32>,

    /// Synaptic PSP values (`f32`)
    pub postsynaptic_potentials: Vec<f32>,

    /// Synapse types (u8: 0=excitatory, 1=inhibitory)
    pub types: Vec<u8>,

    /// Per-synapse delay in whole bursts (`>= 1`). Omitted in older snapshots defaults at load time.
    #[serde(default)]
    pub delay_bursts: Vec<u8>,

    /// Valid mask (bool)
    pub valid_mask: Vec<bool>,

    /// Source neuron index (for fast lookup)
    /// Maps source_neuron_id -> Vec<synapse_index>
    pub source_index: AHashMap<u32, Vec<usize>>,

    /// Packed synapse edge flags. Omitted in older snapshots.
    #[serde(default)]
    pub edge_flags: Vec<u8>,

    /// R-STDP eligibility traces. Omitted in older snapshots.
    #[serde(default)]
    pub eligibility_traces: Vec<f32>,
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
            postsynaptic_potentials: Vec::new(),
            types: Vec::new(),
            delay_bursts: Vec::new(),
            valid_mask: Vec::new(),
            source_index: AHashMap::new(),
            edge_flags: Vec::new(),
            eligibility_traces: Vec::new(),
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
            weights: std::vec::from_elem(0.0f32, capacity),
            postsynaptic_potentials: std::vec::from_elem(0.0f32, capacity),
            types: std::vec::from_elem(0, capacity),
            delay_bursts: std::vec::from_elem(1, capacity),
            valid_mask: std::vec::from_elem(false, capacity),
            source_index: AHashMap::new(),
            edge_flags: std::vec::from_elem(0, capacity),
            eligibility_traces: std::vec::from_elem(0.0, capacity),
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

/// Snapshot persistence mode.
///
/// - `Full`: full neuron + synapse state is persisted and imported directly.
/// - `Lite`: baseline structure is reconstructed from `genome_json`; snapshot carries
///   memory/plastic synapse overlays and long-term memory artifacts.
#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectomePersistMode {
    Full,
    Lite,
}

#[cfg(feature = "std")]
impl Default for ConnectomePersistMode {
    fn default() -> Self {
        Self::Full
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

    /// Snapshot persistence mode (`full` by default for backwards compatibility).
    #[serde(default)]
    pub persist_mode: ConnectomePersistMode,

    /// Flat genome JSON captured at export (areas, mappings, regions, morphologies, physiology).
    #[serde(default)]
    pub genome_json: Option<String>,

    /// Base64 cortical IDs of memory areas present at export.
    #[serde(default)]
    pub memory_area_ids: Vec<String>,

    /// Plastic mappings `(src_base64, dst_base64)` present at export.
    #[serde(default)]
    pub plastic_mappings: Vec<(String, String)>,

    /// Brain region IDs present at export.
    #[serde(default)]
    pub brain_region_ids: Vec<String>,

    /// Long-term memory neurons (STM is never persisted).
    #[serde(default)]
    pub long_term_memory_neurons: Vec<SerializableLongTermMemoryNeuron>,

    /// Replay frames keyed by long-term memory neuron id. STM frames are omitted.
    #[serde(default)]
    pub long_term_memory_replay_frames: Vec<(u32, Vec<SerializableMemoryReplayFrame>)>,
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
    pub fn is_lite_mode(&self) -> bool {
        self.persist_mode == ConnectomePersistMode::Lite
    }

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
        if self.is_lite_mode() {
            if self.genome_json.is_none() {
                return Err(
                    "Lite connectome snapshot requires embedded genome_json".to_string(),
                );
            }
            return self.validate_synapses_only();
        }

        self.validate_full_snapshot()
    }

    fn validate_full_snapshot(&self) -> Result<(), String> {
        // NPU export serializes used neurons/synapses (`count`), not the preallocated
        // NPU buffer (`capacity`). Field lengths must cover `count`.
        let n = self.neurons.count;
        if self.neurons.capacity < n {
            return Err(std::format!(
                "Neuron capacity {} is less than count {}",
                self.neurons.capacity,
                n
            ));
        }
        require_len(
            "neurons.membrane_potentials",
            self.neurons.membrane_potentials.len(),
            n,
        )?;
        require_len("neurons.thresholds", self.neurons.thresholds.len(), n)?;
        require_len(
            "neurons.leak_coefficients",
            self.neurons.leak_coefficients.len(),
            n,
        )?;
        require_len(
            "neurons.resting_potentials",
            self.neurons.resting_potentials.len(),
            n,
        )?;
        require_len("neurons.neuron_types", self.neurons.neuron_types.len(), n)?;
        require_len(
            "neurons.refractory_periods",
            self.neurons.refractory_periods.len(),
            n,
        )?;
        require_len(
            "neurons.excitabilities",
            self.neurons.excitabilities.len(),
            n,
        )?;
        require_len(
            "neurons.cortical_areas",
            self.neurons.cortical_areas.len(),
            n,
        )?;
        require_len("neurons.valid_mask", self.neurons.valid_mask.len(), n)?;
        require_len(
            "neurons.coordinates",
            self.neurons.coordinates.len(),
            n.saturating_mul(3),
        )?;
        require_optional_len(
            "neurons.threshold_limits",
            self.neurons.threshold_limits.len(),
            n,
        )?;
        require_optional_len(
            "neurons.consecutive_fire_counts",
            self.neurons.consecutive_fire_counts.len(),
            n,
        )?;
        require_optional_len(
            "neurons.consecutive_fire_limits",
            self.neurons.consecutive_fire_limits.len(),
            n,
        )?;
        require_optional_len(
            "neurons.snooze_periods",
            self.neurons.snooze_periods.len(),
            n,
        )?;
        require_optional_len(
            "neurons.mp_charge_accumulation",
            self.neurons.mp_charge_accumulation.len(),
            n,
        )?;

        let s = self.synapses.count;
        if self.synapses.capacity < s {
            return Err(std::format!(
                "Synapse capacity {} is less than count {}",
                self.synapses.capacity,
                s
            ));
        }
        require_len(
            "synapses.source_neurons",
            self.synapses.source_neurons.len(),
            s,
        )?;
        require_len(
            "synapses.target_neurons",
            self.synapses.target_neurons.len(),
            s,
        )?;
        require_len("synapses.weights", self.synapses.weights.len(), s)?;
        require_len(
            "synapses.postsynaptic_potentials",
            self.synapses.postsynaptic_potentials.len(),
            s,
        )?;
        require_len("synapses.types", self.synapses.types.len(), s)?;
        require_len("synapses.valid_mask", self.synapses.valid_mask.len(), s)?;
        require_optional_len("synapses.delay_bursts", self.synapses.delay_bursts.len(), s)?;
        require_optional_len("synapses.edge_flags", self.synapses.edge_flags.len(), s)?;
        require_optional_len(
            "synapses.eligibility_traces",
            self.synapses.eligibility_traces.len(),
            s,
        )?;

        // Check synapse references are valid. Regular endpoints must land in the
        // dense neuron array. Memory-range endpoints must be LTM neurons in this snapshot.
        let ltm_ids = self.long_term_memory_id_set();
        for i in 0..self.synapses.count {
            if !self.synapses.valid_mask[i] {
                continue;
            }

            let source = self.synapses.source_neurons[i];
            let target = self.synapses.target_neurons[i];

            if !synapse_endpoint_is_valid(source, self.neurons.count, &ltm_ids) {
                return Err(std::format!(
                    "Synapse {} has invalid source neuron: {}",
                    i,
                    source
                ));
            }

            if !synapse_endpoint_is_valid(target, self.neurons.count, &ltm_ids) {
                return Err(std::format!(
                    "Synapse {} has invalid target neuron: {}",
                    i,
                    target
                ));
            }
        }

        Ok(())
    }

    fn validate_synapses_only(&self) -> Result<(), String> {
        let s = self.synapses.count;
        if self.synapses.capacity < s {
            return Err(std::format!(
                "Synapse capacity {} is less than count {}",
                self.synapses.capacity,
                s
            ));
        }
        require_len(
            "synapses.source_neurons",
            self.synapses.source_neurons.len(),
            s,
        )?;
        require_len(
            "synapses.target_neurons",
            self.synapses.target_neurons.len(),
            s,
        )?;
        require_len("synapses.weights", self.synapses.weights.len(), s)?;
        require_len(
            "synapses.postsynaptic_potentials",
            self.synapses.postsynaptic_potentials.len(),
            s,
        )?;
        require_len("synapses.types", self.synapses.types.len(), s)?;
        require_len("synapses.valid_mask", self.synapses.valid_mask.len(), s)?;
        require_optional_len("synapses.delay_bursts", self.synapses.delay_bursts.len(), s)?;
        require_optional_len("synapses.edge_flags", self.synapses.edge_flags.len(), s)?;
        require_optional_len(
            "synapses.eligibility_traces",
            self.synapses.eligibility_traces.len(),
            s,
        )?;
        Ok(())
    }

    /// Get statistics about the connectome
    pub fn statistics(&self) -> ConnectomeStatistics {
        // Count active synapses
        let active_synapse_count = self.synapses.valid_mask[..self.synapses.count]
            .iter()
            .filter(|&&v| v)
            .count();

        let mut stats = ConnectomeStatistics {
            neuron_count: self.neurons.count,
            synapse_count: self.synapses.count,
            cortical_area_count: self.cortical_area_names.len(),
            active_synapse_count,
            ..Default::default()
        };

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

    /// Record architecture indexes so a connectome snapshot can be validated
    /// for memory areas, plastic mappings, and brain regions without re-parsing
    /// the full genome.
    pub fn set_architecture_indexes(
        &mut self,
        memory_area_ids: Vec<String>,
        plastic_mappings: Vec<(String, String)>,
        brain_region_ids: Vec<String>,
    ) {
        self.memory_area_ids = memory_area_ids;
        self.plastic_mappings = plastic_mappings;
        self.brain_region_ids = brain_region_ids;
    }

    /// Drop synapses that touch short-term memory neurons.
    ///
    /// Regular-to-regular synapses (plastic and associative weight changes included)
    /// are kept. Synapses to or from IDs in `ltm_ids` are kept. Fire-ledger contents
    /// are not part of this snapshot.
    pub fn retain_regular_and_long_term_memory_synapses(&mut self, ltm_ids: &HashSet<u32>) {
        let mut kept: Vec<usize> = Vec::new();
        for i in 0..self.synapses.count {
            if i < self.synapses.valid_mask.len() && !self.synapses.valid_mask[i] {
                continue;
            }
            let src = self.synapses.source_neurons[i];
            let dst = self.synapses.target_neurons[i];
            if synapse_allowed_for_persist(src, dst, ltm_ids) {
                kept.push(i);
            }
        }
        self.synapses = rebuild_synapses_from_indices(&self.synapses, &kept, |id| id);
        self.retain_long_term_memory_replay_frames(ltm_ids);
    }

    /// Keep replay frames only for long-term memory neuron ids.
    pub fn retain_long_term_memory_replay_frames(&mut self, ltm_ids: &HashSet<u32>) {
        self.long_term_memory_replay_frames =
            replay_frames_for_ltm_ids(&self.long_term_memory_replay_frames, ltm_ids);
    }

    fn long_term_memory_id_set(&self) -> HashSet<u32> {
        self.long_term_memory_neurons
            .iter()
            .filter(|n| n.is_longterm_memory && n.is_active)
            .map(|n| n.neuron_id)
            .collect()
    }

    /// Keep only neurons belonging to `cortical_idx` and synapses that touch them.
    ///
    /// Neuron IDs are remapped to a compact 0..n range. `genome_json` and architecture
    /// indexes are preserved so the area snapshot still carries genome context.
    pub fn filter_to_cortical_idx(&self, cortical_idx: u32) -> Self {
        let mut kept_old_ids: Vec<u32> = Vec::new();
        let mut old_to_new: AHashMap<u32, u32> = AHashMap::new();
        for i in 0..self.neurons.count {
            if i < self.neurons.valid_mask.len() && !self.neurons.valid_mask[i] {
                continue;
            }
            if i < self.neurons.cortical_areas.len()
                && self.neurons.cortical_areas[i] == cortical_idx
            {
                let new_id = kept_old_ids.len() as u32;
                old_to_new.insert(i as u32, new_id);
                kept_old_ids.push(i as u32);
            }
        }

        let n = kept_old_ids.len();
        let mut neurons = SerializableNeuronArray::new(n);
        neurons.count = n;
        for (new_i, old_i) in kept_old_ids.iter().enumerate() {
            let old = *old_i as usize;
            neurons.membrane_potentials[new_i] = self.neurons.membrane_potentials[old];
            neurons.thresholds[new_i] = self.neurons.thresholds[old];
            neurons.leak_coefficients[new_i] = self.neurons.leak_coefficients[old];
            neurons.resting_potentials[new_i] = self.neurons.resting_potentials[old];
            neurons.neuron_types[new_i] = self.neurons.neuron_types[old];
            neurons.refractory_periods[new_i] = self.neurons.refractory_periods[old];
            neurons.refractory_countdowns[new_i] = self.neurons.refractory_countdowns[old];
            neurons.excitabilities[new_i] = self.neurons.excitabilities[old];
            neurons.cortical_areas[new_i] = self.neurons.cortical_areas[old];
            neurons.coordinates[new_i * 3] = self.neurons.coordinates[old * 3];
            neurons.coordinates[new_i * 3 + 1] = self.neurons.coordinates[old * 3 + 1];
            neurons.coordinates[new_i * 3 + 2] = self.neurons.coordinates[old * 3 + 2];
            neurons.valid_mask[new_i] = true;
            if old < self.neurons.threshold_limits.len() {
                neurons.threshold_limits[new_i] = self.neurons.threshold_limits[old];
            }
            if old < self.neurons.consecutive_fire_counts.len() {
                neurons.consecutive_fire_counts[new_i] = self.neurons.consecutive_fire_counts[old];
            }
            if old < self.neurons.consecutive_fire_limits.len() {
                neurons.consecutive_fire_limits[new_i] = self.neurons.consecutive_fire_limits[old];
            }
            if old < self.neurons.snooze_periods.len() {
                neurons.snooze_periods[new_i] = self.neurons.snooze_periods[old];
            }
            if old < self.neurons.mp_charge_accumulation.len() {
                neurons.mp_charge_accumulation[new_i] = self.neurons.mp_charge_accumulation[old];
            }
        }

        let long_term_memory_neurons: Vec<SerializableLongTermMemoryNeuron> = self
            .long_term_memory_neurons
            .iter()
            .filter(|n| n.cortical_area_idx == cortical_idx && n.is_longterm_memory)
            .cloned()
            .collect();
        let ltm_ids: HashSet<u32> = long_term_memory_neurons
            .iter()
            .map(|n| n.neuron_id)
            .collect();

        let mut kept_syn: Vec<usize> = Vec::new();
        for i in 0..self.synapses.count {
            if i < self.synapses.valid_mask.len() && !self.synapses.valid_mask[i] {
                continue;
            }
            let src = self.synapses.source_neurons[i];
            let dst = self.synapses.target_neurons[i];
            let src_kept = old_to_new.contains_key(&src) || ltm_ids.contains(&src);
            let dst_kept = old_to_new.contains_key(&dst) || ltm_ids.contains(&dst);
            if src_kept && dst_kept {
                kept_syn.push(i);
            }
        }
        let synapses = rebuild_synapses_from_indices(&self.synapses, &kept_syn, |id| {
            old_to_new.get(&id).copied().unwrap_or(id)
        });

        Self {
            version: self.version,
            neurons,
            synapses,
            cortical_area_names: self
                .cortical_area_names
                .iter()
                .filter(|(idx, _)| **idx == cortical_idx)
                .map(|(k, v)| (*k, v.clone()))
                .collect(),
            burst_count: self.burst_count,
            power_amount: self.power_amount,
            fire_ledger_window: self.fire_ledger_window,
            metadata: self.metadata.clone(),
            persist_mode: self.persist_mode,
            genome_json: self.genome_json.clone(),
            memory_area_ids: self.memory_area_ids.clone(),
            plastic_mappings: self.plastic_mappings.clone(),
            brain_region_ids: self.brain_region_ids.clone(),
            long_term_memory_neurons,
            long_term_memory_replay_frames: replay_frames_for_ltm_ids(
                &self.long_term_memory_replay_frames,
                &ltm_ids,
            ),
        }
    }

    /// Replace neurons/synapses for `cortical_idx` with those from `area_snapshot`.
    ///
    /// Incoming neuron IDs are remapped into the next unused slots. Architecture
    /// fields (`genome_json`, indexes) stay on `self`.
    pub fn replace_cortical_idx(&self, cortical_idx: u32, area_snapshot: &Self) -> Self {
        let base = self.filter_out_cortical_idx(cortical_idx);
        let incoming = area_snapshot.filter_to_cortical_idx(cortical_idx);
        let id_offset = base.neurons.count as u32;
        let incoming_n = incoming.neurons.count;
        let new_n = base.neurons.count + incoming_n;

        let mut neurons = SerializableNeuronArray::new(new_n);
        neurons.count = new_n;
        copy_neuron_range(&base.neurons, 0, &mut neurons, 0, base.neurons.count);
        copy_neuron_range(
            &incoming.neurons,
            0,
            &mut neurons,
            base.neurons.count,
            incoming_n,
        );

        let incoming_s = incoming.synapses.count;
        let new_s = base.synapses.count + incoming_s;
        let mut synapses = SerializableSynapseArray::new(new_s);
        synapses.count = new_s;
        copy_synapse_range(&base.synapses, 0, &mut synapses, 0, base.synapses.count, 0);
        copy_synapse_range(
            &incoming.synapses,
            0,
            &mut synapses,
            base.synapses.count,
            incoming_s,
            id_offset,
        );
        let mut source_index = AHashMap::new();
        for i in 0..new_s {
            source_index
                .entry(synapses.source_neurons[i])
                .or_insert_with(Vec::new)
                .push(i);
        }
        synapses.source_index = source_index;

        let mut names = base.cortical_area_names.clone();
        for (idx, name) in incoming.cortical_area_names {
            names.insert(idx, name);
        }

        let mut long_term_memory_neurons = base.long_term_memory_neurons;
        long_term_memory_neurons.extend(incoming.long_term_memory_neurons);
        let mut long_term_memory_replay_frames = base.long_term_memory_replay_frames;
        long_term_memory_replay_frames.extend(incoming.long_term_memory_replay_frames);

        Self {
            version: self.version,
            neurons,
            synapses,
            cortical_area_names: names,
            burst_count: self.burst_count,
            power_amount: self.power_amount,
            fire_ledger_window: self.fire_ledger_window,
            metadata: self.metadata.clone(),
            persist_mode: self.persist_mode,
            genome_json: self.genome_json.clone(),
            memory_area_ids: self.memory_area_ids.clone(),
            plastic_mappings: self.plastic_mappings.clone(),
            brain_region_ids: self.brain_region_ids.clone(),
            long_term_memory_neurons,
            long_term_memory_replay_frames,
        }
    }

    fn filter_out_cortical_idx(&self, cortical_idx: u32) -> Self {
        let mut kept_old_ids: Vec<u32> = Vec::new();
        let mut old_to_new: AHashMap<u32, u32> = AHashMap::new();
        for i in 0..self.neurons.count {
            if i < self.neurons.cortical_areas.len()
                && self.neurons.cortical_areas[i] == cortical_idx
            {
                continue;
            }
            let new_id = kept_old_ids.len() as u32;
            old_to_new.insert(i as u32, new_id);
            kept_old_ids.push(i as u32);
        }
        let n = kept_old_ids.len();
        let mut neurons = SerializableNeuronArray::new(n);
        neurons.count = n;
        for (new_i, old_i) in kept_old_ids.iter().enumerate() {
            copy_neuron_range(&self.neurons, *old_i as usize, &mut neurons, new_i, 1);
        }

        let long_term_memory_neurons: Vec<SerializableLongTermMemoryNeuron> = self
            .long_term_memory_neurons
            .iter()
            .filter(|n| n.cortical_area_idx != cortical_idx)
            .cloned()
            .collect();
        let ltm_ids: HashSet<u32> = long_term_memory_neurons
            .iter()
            .map(|n| n.neuron_id)
            .collect();

        let mut kept_syn: Vec<usize> = Vec::new();
        for i in 0..self.synapses.count {
            let src = self.synapses.source_neurons[i];
            let dst = self.synapses.target_neurons[i];
            let src_kept = old_to_new.contains_key(&src) || ltm_ids.contains(&src);
            let dst_kept = old_to_new.contains_key(&dst) || ltm_ids.contains(&dst);
            if src_kept && dst_kept {
                kept_syn.push(i);
            }
        }
        let synapses = rebuild_synapses_from_indices(&self.synapses, &kept_syn, |id| {
            old_to_new.get(&id).copied().unwrap_or(id)
        });

        Self {
            version: self.version,
            neurons,
            synapses,
            cortical_area_names: self
                .cortical_area_names
                .iter()
                .filter(|(idx, _)| **idx != cortical_idx)
                .map(|(k, v)| (*k, v.clone()))
                .collect(),
            burst_count: self.burst_count,
            power_amount: self.power_amount,
            fire_ledger_window: self.fire_ledger_window,
            metadata: self.metadata.clone(),
            persist_mode: self.persist_mode,
            genome_json: self.genome_json.clone(),
            memory_area_ids: self.memory_area_ids.clone(),
            plastic_mappings: self.plastic_mappings.clone(),
            brain_region_ids: self.brain_region_ids.clone(),
            long_term_memory_neurons,
            long_term_memory_replay_frames: replay_frames_for_ltm_ids(
                &self.long_term_memory_replay_frames,
                &ltm_ids,
            ),
        }
    }

    /// Convert a full snapshot into a lite snapshot by retaining only memory/plastic
    /// synapses and dropping full neuron-array payload.
    pub fn to_lite_snapshot(mut self) -> Self {
        self.persist_mode = ConnectomePersistMode::Lite;
        self.neurons = SerializableNeuronArray::default();
        self.burst_count = 0;
        self.power_amount = 1.0;
        self
    }
}

fn require_len(name: &str, len: usize, min: usize) -> Result<(), String> {
    if len < min {
        return Err(std::format!(
            "{} length {} is less than count {}",
            name,
            len,
            min
        ));
    }
    Ok(())
}

fn require_optional_len(name: &str, len: usize, min: usize) -> Result<(), String> {
    if len == 0 {
        return Ok(());
    }
    require_len(name, len, min)
}

fn replay_frames_for_ltm_ids(
    frames: &[(u32, Vec<SerializableMemoryReplayFrame>)],
    ltm_ids: &HashSet<u32>,
) -> Vec<(u32, Vec<SerializableMemoryReplayFrame>)> {
    frames
        .iter()
        .filter(|(neuron_id, _)| ltm_ids.contains(neuron_id))
        .cloned()
        .collect()
}

fn synapse_allowed_for_persist(src: u32, dst: u32, ltm_ids: &HashSet<u32>) -> bool {
    let src_ok = !is_memory_neuron_id(src) || ltm_ids.contains(&src);
    let dst_ok = !is_memory_neuron_id(dst) || ltm_ids.contains(&dst);
    src_ok && dst_ok
}

fn synapse_endpoint_is_valid(id: u32, regular_count: usize, ltm_ids: &HashSet<u32>) -> bool {
    if is_memory_neuron_id(id) {
        ltm_ids.contains(&id)
    } else {
        (id as usize) < regular_count
    }
}

fn rebuild_synapses_from_indices<F>(
    src: &SerializableSynapseArray,
    kept: &[usize],
    remap: F,
) -> SerializableSynapseArray
where
    F: Fn(u32) -> u32,
{
    let s = kept.len();
    let mut synapses = SerializableSynapseArray::new(s);
    synapses.count = s;
    let mut source_index = AHashMap::new();
    for (new_i, old_i) in kept.iter().enumerate() {
        let old = *old_i;
        let mapped_src = remap(src.source_neurons[old]);
        let mapped_dst = remap(src.target_neurons[old]);
        synapses.source_neurons[new_i] = mapped_src;
        synapses.target_neurons[new_i] = mapped_dst;
        synapses.weights[new_i] = src.weights[old];
        synapses.postsynaptic_potentials[new_i] = src.postsynaptic_potentials[old];
        synapses.types[new_i] = src.types[old];
        synapses.valid_mask[new_i] = true;
        if old < src.delay_bursts.len() {
            synapses.delay_bursts[new_i] = src.delay_bursts[old];
        }
        if old < src.edge_flags.len() {
            synapses.edge_flags[new_i] = src.edge_flags[old];
        }
        if old < src.eligibility_traces.len() {
            synapses.eligibility_traces[new_i] = src.eligibility_traces[old];
        }
        source_index
            .entry(mapped_src)
            .or_insert_with(Vec::new)
            .push(new_i);
    }
    synapses.source_index = source_index;
    synapses
}

fn copy_neuron_range(
    src: &SerializableNeuronArray,
    src_start: usize,
    dst: &mut SerializableNeuronArray,
    dst_start: usize,
    count: usize,
) {
    for i in 0..count {
        let s = src_start + i;
        let d = dst_start + i;
        dst.membrane_potentials[d] = src.membrane_potentials[s];
        dst.thresholds[d] = src.thresholds[s];
        dst.leak_coefficients[d] = src.leak_coefficients[s];
        dst.resting_potentials[d] = src.resting_potentials[s];
        dst.neuron_types[d] = src.neuron_types[s];
        dst.refractory_periods[d] = src.refractory_periods[s];
        dst.refractory_countdowns[d] = src.refractory_countdowns[s];
        dst.excitabilities[d] = src.excitabilities[s];
        dst.cortical_areas[d] = src.cortical_areas[s];
        dst.coordinates[d * 3] = src.coordinates[s * 3];
        dst.coordinates[d * 3 + 1] = src.coordinates[s * 3 + 1];
        dst.coordinates[d * 3 + 2] = src.coordinates[s * 3 + 2];
        dst.valid_mask[d] = src.valid_mask[s];
        if s < src.threshold_limits.len() {
            dst.threshold_limits[d] = src.threshold_limits[s];
        }
        if s < src.consecutive_fire_counts.len() {
            dst.consecutive_fire_counts[d] = src.consecutive_fire_counts[s];
        }
        if s < src.consecutive_fire_limits.len() {
            dst.consecutive_fire_limits[d] = src.consecutive_fire_limits[s];
        }
        if s < src.snooze_periods.len() {
            dst.snooze_periods[d] = src.snooze_periods[s];
        }
        if s < src.mp_charge_accumulation.len() {
            dst.mp_charge_accumulation[d] = src.mp_charge_accumulation[s];
        }
    }
}

fn copy_synapse_range(
    src: &SerializableSynapseArray,
    src_start: usize,
    dst: &mut SerializableSynapseArray,
    dst_start: usize,
    count: usize,
    id_offset: u32,
) {
    for i in 0..count {
        let s = src_start + i;
        let d = dst_start + i;
        dst.source_neurons[d] = src.source_neurons[s] + id_offset;
        dst.target_neurons[d] = src.target_neurons[s] + id_offset;
        dst.weights[d] = src.weights[s];
        dst.postsynaptic_potentials[d] = src.postsynaptic_potentials[s];
        dst.types[d] = src.types[s];
        dst.valid_mask[d] = src.valid_mask[s];
        if s < src.delay_bursts.len() {
            dst.delay_bursts[d] = src.delay_bursts[s];
        }
        if s < src.edge_flags.len() {
            dst.edge_flags[d] = src.edge_flags[s];
        }
        if s < src.eligibility_traces.len() {
            dst.eligibility_traces[d] = src.eligibility_traces[s];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neuron_at(idx: u32, area: u32, potential: f32) -> (u32, u32, f32) {
        (idx, area, potential)
    }

    fn snapshot_with_two_areas() -> ConnectomeSnapshot {
        let mut neurons = SerializableNeuronArray::new(3);
        neurons.count = 3;
        neurons.cortical_areas = vec![1, 1, 2];
        neurons.membrane_potentials = vec![0.1, 0.2, 0.9];
        neurons.thresholds = vec![1.0, 1.0, 1.0];
        neurons.leak_coefficients = vec![0.0, 0.0, 0.0];
        neurons.resting_potentials = vec![0.0, 0.0, 0.0];
        neurons.neuron_types = vec![0, 0, 0];
        neurons.refractory_periods = vec![0, 0, 0];
        neurons.refractory_countdowns = vec![0, 0, 0];
        neurons.excitabilities = vec![1.0, 1.0, 1.0];
        neurons.coordinates = vec![0, 0, 0, 1, 0, 0, 0, 1, 0];
        neurons.valid_mask = vec![true, true, true];
        neurons.threshold_limits = vec![0.0, 0.0, 0.0];
        neurons.consecutive_fire_counts = vec![0, 0, 0];
        neurons.consecutive_fire_limits = vec![0, 0, 0];
        neurons.snooze_periods = vec![0, 0, 0];
        neurons.mp_charge_accumulation = vec![false, false, false];

        let mut synapses = SerializableSynapseArray::new(2);
        synapses.count = 2;
        synapses.source_neurons = vec![0, 2];
        synapses.target_neurons = vec![1, 2];
        synapses.weights = vec![0.5, 0.8];
        synapses.postsynaptic_potentials = vec![0.0, 0.0];
        synapses.types = vec![0, 0];
        synapses.delay_bursts = vec![1, 1];
        synapses.valid_mask = vec![true, true];
        synapses.edge_flags = vec![0, 0];
        synapses.eligibility_traces = vec![0.0, 0.0];
        let mut source_index = AHashMap::new();
        source_index.insert(0, vec![0]);
        source_index.insert(2, vec![1]);
        synapses.source_index = source_index;

        let mut names = AHashMap::new();
        names.insert(1, "area-a".to_string());
        names.insert(2, "area-b".to_string());

        ConnectomeSnapshot {
            version: 1,
            neurons,
            synapses,
            cortical_area_names: names,
            burst_count: 3,
            power_amount: 1.0,
            fire_ledger_window: 20,
            metadata: ConnectomeMetadata::default(),
            persist_mode: ConnectomePersistMode::Full,
            genome_json: Some("{\"version\":\"3.0\"}".to_string()),
            memory_area_ids: vec!["mmem".to_string()],
            plastic_mappings: vec![("a".to_string(), "b".to_string())],
            brain_region_ids: vec!["root".to_string()],
            long_term_memory_neurons: Vec::new(),
            long_term_memory_replay_frames: Vec::new(),
        }
    }

    #[test]
    fn filter_to_cortical_idx_keeps_architecture_and_area_synapses() {
        let snapshot = snapshot_with_two_areas();
        let filtered = snapshot.filter_to_cortical_idx(1);
        assert_eq!(filtered.neurons.count, 2);
        assert_eq!(filtered.synapses.count, 1);
        assert_eq!(filtered.synapses.source_neurons[0], 0);
        assert_eq!(filtered.synapses.target_neurons[0], 1);
        assert_eq!(
            filtered.genome_json.as_deref(),
            Some("{\"version\":\"3.0\"}")
        );
        assert_eq!(filtered.memory_area_ids, vec!["mmem".to_string()]);
        assert_eq!(filtered.cortical_area_names.len(), 1);
        let _ = neuron_at(0, 1, 0.1);
    }

    #[test]
    fn replace_cortical_idx_swaps_only_that_area() {
        let live = snapshot_with_two_areas();
        let incoming = live.filter_to_cortical_idx(2);
        let merged = live.replace_cortical_idx(2, &incoming);
        assert_eq!(merged.neurons.count, 3);
        assert_eq!(
            merged
                .neurons
                .cortical_areas
                .iter()
                .filter(|&&a| a == 2)
                .count(),
            1
        );
        assert_eq!(merged.brain_region_ids, vec!["root".to_string()]);
    }

    #[test]
    fn retain_regular_and_ltm_synapses_drops_stm_keeps_plastic_weights() {
        let mut snapshot = snapshot_with_two_areas();
        snapshot.synapses = SerializableSynapseArray::new(4);
        snapshot.synapses.count = 4;
        snapshot.synapses.source_neurons = vec![0, 1, 50_000_001, 50_000_002];
        snapshot.synapses.target_neurons = vec![1, 0, 1, 0];
        snapshot.synapses.weights = vec![0.11, 0.77, 0.33, 0.44];
        snapshot.synapses.postsynaptic_potentials = vec![0.0, 0.0, 0.0, 0.0];
        snapshot.synapses.types = vec![0, 0, 0, 0];
        snapshot.synapses.delay_bursts = vec![1, 1, 1, 1];
        snapshot.synapses.valid_mask = vec![true, true, true, true];
        snapshot.synapses.edge_flags = vec![1, 2, 0, 0];
        snapshot.synapses.eligibility_traces = vec![0.1, 0.9, 0.0, 0.0];
        snapshot.long_term_memory_neurons = vec![SerializableLongTermMemoryNeuron {
            neuron_id: 50_000_001,
            cortical_area_idx: 1,
            pattern_hash: Some(1),
            is_longterm_memory: true,
            is_active: true,
            lifespan_current: 100,
            lifespan_initial: 20,
            lifespan_growth_rate: 3.0,
            creation_burst: 1,
            last_activation_burst: 2,
            activation_count: 4,
        }];

        let ltm_ids = HashSet::from([50_000_001]);
        snapshot.retain_regular_and_long_term_memory_synapses(&ltm_ids);

        assert_eq!(snapshot.synapses.count, 3);
        assert_eq!(snapshot.synapses.source_neurons[..3], [0, 1, 50_000_001]);
        assert_eq!(snapshot.synapses.weights[..3], [0.11, 0.77, 0.33]);
        assert_eq!(snapshot.synapses.edge_flags[..3], [1, 2, 0]);
        assert_eq!(snapshot.synapses.eligibility_traces[..3], [0.1, 0.9, 0.0]);
        assert!(snapshot.validate().is_ok());
    }

    #[test]
    fn validate_accepts_used_length_below_npu_capacity() {
        let mut snapshot = snapshot_with_two_areas();
        snapshot.neurons.capacity = 10_000_000;
        snapshot.synapses.capacity = 5_000_000;
        assert!(snapshot.validate().is_ok());

        snapshot.neurons.membrane_potentials.truncate(1);
        let err = snapshot.validate().expect_err("short SoA must fail");
        assert!(err.contains("membrane_potentials"));
    }

    #[test]
    fn retain_replay_frames_keeps_ltm_drops_stm() {
        let mut snapshot = snapshot_with_two_areas();
        snapshot.long_term_memory_replay_frames = vec![
            (
                50_000_001,
                vec![SerializableMemoryReplayFrame {
                    offset: 0,
                    upstream_area_idx: 7,
                    coords: vec![(1, 2, 3)],
                    membrane_potentials: Some(vec![0.4]),
                }],
            ),
            (
                50_000_002,
                vec![SerializableMemoryReplayFrame {
                    offset: 1,
                    upstream_area_idx: 7,
                    coords: vec![(4, 5, 6)],
                    membrane_potentials: None,
                }],
            ),
        ];
        snapshot.retain_long_term_memory_replay_frames(&HashSet::from([50_000_001]));
        assert_eq!(snapshot.long_term_memory_replay_frames.len(), 1);
        assert_eq!(snapshot.long_term_memory_replay_frames[0].0, 50_000_001);
        assert_eq!(
            snapshot.long_term_memory_replay_frames[0].1[0].coords,
            vec![(1, 2, 3)]
        );
    }

    #[test]
    fn lite_snapshot_validation_requires_genome_json() {
        let mut snapshot = snapshot_with_two_areas().to_lite_snapshot();
        snapshot.genome_json = None;
        let err = snapshot.validate().expect_err("lite snapshot must require genome");
        assert!(err.contains("genome_json"));
    }

    #[test]
    fn lite_snapshot_allows_empty_payload_when_genome_present() {
        let snapshot = snapshot_with_two_areas().to_lite_snapshot();
        assert!(snapshot.validate().is_ok());
    }
}
