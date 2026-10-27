// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Memory Neuron Array - Structure of Arrays for memory neurons with lifecycle management
//!
//! High-performance SoA implementation optimized for:
//! - SIMD-friendly vectorized operations
//! - Rust/RTOS compatibility
//! - Thread-safe operations
//! - Efficient memory management with index reuse

use crate::neuron_id_manager::{AllocationStats, NeuronIdManager};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};

/// What a memory area does when it cannot get a slot for a new neuron.
///
/// A slot is missing when the area is at its `memory_neuron_cap` or the shared pool is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LtmEviction {
    /// Free a slot by evicting one neuron (lowest-lifespan short-term first, otherwise the
    /// least recently activated long-term neuron), then create.
    #[default]
    Lru,
    /// Never evict for this area: creation fails when no slot is free, and the area's own
    /// neurons are never chosen as victims for other areas.
    None,
}

impl LtmEviction {
    /// Genome / REST spelling: `"lru"` or `"none"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            LtmEviction::Lru => "lru",
            LtmEviction::None => "none",
        }
    }

    /// Parse the genome / REST spelling (case-insensitive). Unknown text is `None`.
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "lru" => Some(LtmEviction::Lru),
            "none" => Some(LtmEviction::None),
            _ => None,
        }
    }
}

/// Memory neuron lifecycle configuration
#[derive(Debug, Clone, Copy)]
pub struct MemoryNeuronLifecycleConfig {
    /// Initial lifespan in bursts
    pub initial_lifespan: u32,

    /// Lifespan growth per reactivation
    pub lifespan_growth_rate: f32,

    /// Lifespan threshold for long-term memory conversion
    pub longterm_threshold: u32,

    /// Maximum reactivations before forced LTM
    pub max_reactivations: u32,

    /// Most memory neurons (short-term plus long-term) this area may hold. `0` means no
    /// per-area cap; only the shared pool limits the area.
    ///
    /// Lowering the cap below the area's current neuron count does not evict anything
    /// immediately. Each later creation evicts one neuron (when `ltm_eviction` is `Lru`),
    /// so the count falls toward the cap only as new patterns arrive.
    pub memory_neuron_cap: u32,

    /// Behavior when the area cap or the shared pool leaves no free slot for a new neuron.
    pub ltm_eviction: LtmEviction,
}

impl Default for MemoryNeuronLifecycleConfig {
    fn default() -> Self {
        Self {
            initial_lifespan: 20,
            lifespan_growth_rate: 3.0,
            longterm_threshold: 100,
            max_reactivations: 1000,
            memory_neuron_cap: 0,
            ltm_eviction: LtmEviction::Lru,
        }
    }
}

/// Why a memory neuron left the array.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryNeuronRemovalReason {
    /// Short-term lifespan reached zero.
    Expired,
    /// Evicted to make room for a new neuron (area cap or full pool).
    Evicted,
}

/// A memory neuron that left the array and whose NPU-side footprint still has to be removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemovedMemoryNeuron {
    pub neuron_id: u32,
    pub area_idx: u32,
    pub was_longterm: bool,
    pub reason: MemoryNeuronRemovalReason,
    /// False for a record dropped during an LTM restore: it never entered the per-area
    /// stats, so removing it must not subtract from them.
    pub counted_in_stats: bool,
}

/// Per-area eviction policy the array applies when it picks a victim in that area.
#[derive(Debug, Clone, Copy)]
struct AreaEvictionPolicy {
    eviction: LtmEviction,
}

/// Snapshot of a single memory neuron for API / inspection (plasticity array only).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryNeuronDetail {
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
    /// Local occupancy hash used by `episodic_scan`. Independent of `pattern_hash`.
    #[serde(default)]
    pub spatial_signature: Option<u64>,
    /// Class channels bound from the partner class memory area.
    #[serde(default)]
    pub class_channels: Vec<u32>,
    /// Associative weight of each bound class channel. Absent weights are unset.
    #[serde(default)]
    pub class_channel_weights: Vec<(u32, f32)>,
}

/// Memory neuron array statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryNeuronStats {
    pub total_capacity: usize,
    pub active_neurons: usize,
    pub longterm_neurons: usize,
    pub dead_neurons: usize,
    pub reusable_indices: usize,
    pub memory_usage_bytes: usize,
    pub avg_lifespan: f64,
    pub avg_activation_count: f64,
}

/// High-performance Structure of Arrays for memory neurons
pub struct MemoryNeuronArray {
    capacity: usize,

    // Core neuron properties (SoA layout)
    neuron_ids: Vec<u32>,
    /// Global neuron id -> slot index, for active neurons only. Kept exact on every
    /// path that assigns or frees a slot.
    neuron_id_to_index: HashMap<u32, usize>,
    cortical_area_ids: Vec<u32>,
    is_active: Vec<bool>,

    // Lifecycle management
    lifespan_current: Vec<u32>,
    lifespan_initial: Vec<u32>,
    lifespan_growth_rate: Vec<f32>,
    is_longterm_memory: Vec<bool>,

    // Temporal tracking
    creation_burst: Vec<u64>,
    last_activation_burst: Vec<u64>,
    activation_count: Vec<u32>,

    // Pattern association (xxHash64 for fast, deterministic pattern identification).
    // Keyed by (cortical_area_id, hash): memory areas fed the same upstream pattern
    // compute the same hash but each owns an independent neuron.
    pattern_hash_to_index: HashMap<(u32, u64), usize>,
    index_to_pattern_hash: HashMap<usize, u64>,

    // Index management
    next_available_index: usize,
    reusable_indices: HashSet<usize>,

    // Area-specific tracking
    area_neuron_indices: HashMap<u32, HashSet<usize>>,

    // Scan sidecar: local occupancy hash and bound class channels.
    spatial_signature: HashMap<usize, u64>,
    class_channels: HashMap<usize, HashSet<u32>>,
    /// Associative weight for each bound class channel while reward training is on.
    class_channel_weights: HashMap<usize, HashMap<u32, f32>>,
    /// Decisions waiting for a later answer, and which pattern was already corrected.
    presentation_ledger: crate::classifier_reward::PresentationLedger,
    /// LTM-only index: (cortical_area_idx, spatial_signature) -> neuron indices.
    spatial_ltm_index: HashMap<(u32, u64), Vec<usize>>,

    // Neuron ID manager
    id_manager: NeuronIdManager,

    // ---- Eviction bookkeeping ----
    /// Eviction policy per area, set by the service when an area is registered.
    area_policies: HashMap<u32, AreaEvictionPolicy>,
    /// Per-area `memory_neuron_cap` known to the array; used only to trim an LTM restore.
    area_caps: HashMap<u32, u32>,
    /// Short-term neurons per area ordered by `(lifespan_current + age_epoch, slot)`.
    /// `age_epoch` rises by one per aging pass, so a neuron that ages in step keeps a
    /// constant key and only neurons skipped by a pass are re-keyed.
    stm_order: HashMap<u32, BTreeSet<(u64, usize)>>,
    /// Long-term neurons per area ordered by `(last_activation_burst, slot)`.
    ltm_order: HashMap<u32, BTreeSet<(u64, usize)>>,
    /// Key under which each slot sits in `stm_order` / `ltm_order`.
    order_key: Vec<u64>,
    age_epoch: u64,
    /// Active neurons across all areas (pool occupancy).
    active_count: usize,
    /// Active long-term neurons across all areas.
    ltm_count: usize,
    /// Neurons removed since the last drain, waiting for NPU-side cleanup.
    removed: Vec<RemovedMemoryNeuron>,
}

impl MemoryNeuronArray {
    /// Create a new memory neuron array
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            neuron_ids: vec![0; capacity],
            neuron_id_to_index: HashMap::new(),
            cortical_area_ids: vec![0; capacity],
            is_active: vec![false; capacity],
            lifespan_current: vec![0; capacity],
            lifespan_initial: vec![0; capacity],
            lifespan_growth_rate: vec![0.0; capacity],
            is_longterm_memory: vec![false; capacity],
            creation_burst: vec![0; capacity],
            last_activation_burst: vec![0; capacity],
            activation_count: vec![0; capacity],
            pattern_hash_to_index: HashMap::new(),
            index_to_pattern_hash: HashMap::new(),
            next_available_index: 0,
            reusable_indices: HashSet::new(),
            area_neuron_indices: HashMap::new(),
            spatial_signature: HashMap::new(),
            class_channels: HashMap::new(),
            class_channel_weights: HashMap::new(),
            presentation_ledger: crate::classifier_reward::PresentationLedger::default(),
            spatial_ltm_index: HashMap::new(),
            id_manager: NeuronIdManager::new(),
            area_policies: HashMap::new(),
            area_caps: HashMap::new(),
            stm_order: HashMap::new(),
            ltm_order: HashMap::new(),
            order_key: vec![0; capacity],
            age_epoch: 0,
            active_count: 0,
            ltm_count: 0,
            removed: Vec::new(),
        }
    }

    /// Set the eviction policy used when this array picks a victim inside `area_idx`.
    pub fn set_area_eviction(&mut self, area_idx: u32, eviction: LtmEviction) {
        self.area_policies
            .insert(area_idx, AreaEvictionPolicy { eviction });
    }

    /// Record an area's `memory_neuron_cap` (0 = none) so an LTM restore can honor it.
    /// Creation reads the cap from the config passed to [`Self::create_memory_neuron`].
    pub fn set_area_cap(&mut self, area_idx: u32, cap: u32) {
        self.area_caps.insert(area_idx, cap);
    }

    /// Forget every area's eviction policy and cap (registrations cleared).
    pub fn clear_all_area_policies(&mut self) {
        self.area_policies.clear();
        self.area_caps.clear();
    }

    /// Forget the eviction policy and cap for an area (area unregistered).
    pub fn clear_area_eviction(&mut self, area_idx: u32) {
        self.area_policies.remove(&area_idx);
        self.area_caps.remove(&area_idx);
    }

    fn area_eviction(&self, area_idx: u32) -> LtmEviction {
        self.area_policies
            .get(&area_idx)
            .map(|policy| policy.eviction)
            .unwrap_or_default()
    }

    /// Slots in the shared pool.
    pub fn pool_capacity(&self) -> usize {
        self.capacity
    }

    /// Active memory neurons across all areas.
    pub fn pool_in_use(&self) -> usize {
        self.active_count
    }

    /// Active long-term memory neurons across all areas.
    pub fn pool_ltm(&self) -> usize {
        self.ltm_count
    }

    /// Active neurons (short-term plus long-term) in one area.
    pub fn area_neuron_count(&self, area_idx: u32) -> usize {
        self.area_neuron_indices
            .get(&area_idx)
            .map_or(0, |indices| indices.len())
    }

    /// Take the neurons removed (expired or evicted) since the last call.
    ///
    /// The caller removes each neuron's NPU-side footprint (synapses, dynamic mapping,
    /// replay frames, activation records) and tells downstream consumers.
    pub fn drain_removed_neurons(&mut self) -> Vec<RemovedMemoryNeuron> {
        std::mem::take(&mut self.removed)
    }

    fn pool_full(&self) -> bool {
        self.reusable_indices.is_empty() && self.next_available_index >= self.capacity
    }

    /// Put an active slot into its area's ordered index (short-term or long-term).
    fn order_insert(&mut self, idx: usize) {
        let area = self.cortical_area_ids[idx];
        if self.is_longterm_memory[idx] {
            let key = self.last_activation_burst[idx];
            self.order_key[idx] = key;
            self.ltm_order.entry(area).or_default().insert((key, idx));
        } else {
            let key = u64::from(self.lifespan_current[idx]).saturating_add(self.age_epoch);
            self.order_key[idx] = key;
            self.stm_order.entry(area).or_default().insert((key, idx));
        }
    }

    /// Take a slot out of its ordered index. Call before changing the fields the key uses.
    fn order_remove(&mut self, idx: usize) {
        let area = self.cortical_area_ids[idx];
        let key = self.order_key[idx];
        let table = if self.is_longterm_memory[idx] {
            &mut self.ltm_order
        } else {
            &mut self.stm_order
        };
        if let Some(set) = table.get_mut(&area) {
            set.remove(&(key, idx));
            if set.is_empty() {
                table.remove(&area);
            }
        }
    }

    /// Mark an active short-term slot long-term and move it between indexes.
    fn convert_to_longterm(&mut self, idx: usize) {
        self.order_remove(idx);
        self.is_longterm_memory[idx] = true;
        self.ltm_count += 1;
        self.order_insert(idx);
        self.index_spatial_ltm(idx);
    }

    /// The lru-policy area holding the most neurons; ties go to the lowest area idx.
    /// `creator` uses `creator_policy` because the stored policy may lag its config.
    fn most_populated_lru_area(&self, creator: u32, creator_policy: LtmEviction) -> Option<u32> {
        let mut best: Option<(usize, u32)> = None;
        for (&area, indices) in &self.area_neuron_indices {
            let count = indices.len();
            if count == 0 {
                continue;
            }
            let policy = if area == creator {
                creator_policy
            } else {
                self.area_eviction(area)
            };
            if policy != LtmEviction::Lru {
                continue;
            }
            let better = match best {
                None => true,
                Some((best_count, best_area)) => {
                    count > best_count || (count == best_count && area < best_area)
                }
            };
            if better {
                best = Some((count, area));
            }
        }
        best.map(|(_, area)| area)
    }

    /// Victim inside one area: lowest-lifespan short-term first, otherwise the least
    /// recently activated long-term neuron. Neurons created or reactivated in
    /// `current_burst` are never chosen.
    fn pick_victim(&self, area: u32, current_burst: u64) -> Option<usize> {
        let evictable = |idx: usize| self.last_activation_burst[idx] != current_burst;
        if let Some(set) = self.stm_order.get(&area) {
            if let Some(&(_, idx)) = set.iter().find(|&&(_, idx)| evictable(idx)) {
                return Some(idx);
            }
        }
        self.ltm_order
            .get(&area)?
            .iter()
            .find(|&&(_, idx)| evictable(idx))
            .map(|&(_, idx)| idx)
    }

    /// Remove an active neuron to free its slot and record it for NPU-side cleanup.
    fn evict_neuron(&mut self, idx: usize) {
        let record = RemovedMemoryNeuron {
            neuron_id: self.neuron_ids[idx],
            area_idx: self.cortical_area_ids[idx],
            was_longterm: self.is_longterm_memory[idx],
            reason: MemoryNeuronRemovalReason::Evicted,
            counted_in_stats: true,
        };
        self.is_active[idx] = false;
        self.cleanup_dead_neuron_internal(idx);
        self.removed.push(record);
    }

    /// Create a new memory neuron.
    ///
    /// A pattern the area already holds reactivates its neuron. A new pattern needs a free
    /// slot. When the area is at `config.memory_neuron_cap` (cap > 0) the area's own
    /// neuron is evicted; when the shared pool is full the victim is the `Lru` area holding
    /// the most neurons (ties: lowest area idx; the creating area may be chosen). With
    /// `config.ltm_eviction == None`, or when no victim exists, nothing is evicted and the
    /// call returns `None`. Evicted neurons are recorded for [`Self::drain_removed_neurons`].
    pub fn create_memory_neuron(
        &mut self,
        pattern_hash: u64,
        cortical_area_id: u32,
        current_burst: u64,
        config: &MemoryNeuronLifecycleConfig,
    ) -> Option<usize> {
        // Check if pattern already exists in this area
        if let Some(&existing_idx) = self
            .pattern_hash_to_index
            .get(&(cortical_area_id, pattern_hash))
        {
            if self.is_active[existing_idx] {
                // Reactivate existing neuron instead
                return self.reactivate_memory_neuron_internal(existing_idx, current_burst);
            }
        }

        // Free a slot by eviction when the area is at its cap or the pool is full.
        let area_at_cap = config.memory_neuron_cap > 0
            && self.area_neuron_count(cortical_area_id) >= config.memory_neuron_cap as usize;
        if area_at_cap || self.pool_full() {
            if config.ltm_eviction == LtmEviction::None {
                return None;
            }
            let victim_area = if area_at_cap {
                cortical_area_id
            } else {
                self.most_populated_lru_area(cortical_area_id, config.ltm_eviction)?
            };
            let victim = self.pick_victim(victim_area, current_burst)?;
            self.evict_neuron(victim);
        }

        // Get neuron index (reuse or allocate new)
        let neuron_idx = self.get_available_index_internal()?;

        // Allocate global neuron ID. A failure returns the slot instead of leaking it.
        let Some(neuron_id) = self.id_manager.allocate_memory_neuron_id() else {
            self.reusable_indices.insert(neuron_idx);
            return None;
        };

        // Initialize neuron properties
        self.neuron_ids[neuron_idx] = neuron_id;
        self.neuron_id_to_index.insert(neuron_id, neuron_idx);
        self.cortical_area_ids[neuron_idx] = cortical_area_id;
        self.is_active[neuron_idx] = true;

        // Initialize lifecycle
        self.lifespan_current[neuron_idx] = config.initial_lifespan;
        self.lifespan_initial[neuron_idx] = config.initial_lifespan;
        self.lifespan_growth_rate[neuron_idx] = config.lifespan_growth_rate;
        self.is_longterm_memory[neuron_idx] = false;

        // Initialize temporal tracking
        self.creation_burst[neuron_idx] = current_burst;
        self.last_activation_burst[neuron_idx] = current_burst;
        self.activation_count[neuron_idx] = 1;

        // Register pattern association
        self.pattern_hash_to_index
            .insert((cortical_area_id, pattern_hash), neuron_idx);
        self.index_to_pattern_hash.insert(neuron_idx, pattern_hash);

        // Add to area tracking
        self.area_neuron_indices
            .entry(cortical_area_id)
            .or_default()
            .insert(neuron_idx);
        self.active_count += 1;
        self.order_insert(neuron_idx);

        Some(neuron_idx)
    }

    /// Reactivate an existing memory neuron
    pub fn reactivate_memory_neuron(&mut self, neuron_idx: usize, current_burst: u64) -> bool {
        self.reactivate_memory_neuron_internal(neuron_idx, current_burst)
            .is_some()
    }

    /// Internal reactivate that returns Option<usize> for compatibility
    fn reactivate_memory_neuron_internal(
        &mut self,
        neuron_idx: usize,
        current_burst: u64,
    ) -> Option<usize> {
        if !self.is_valid_index(neuron_idx) || !self.is_active[neuron_idx] {
            return None;
        }

        // Both ordered indexes key on fields this call changes.
        self.order_remove(neuron_idx);

        // Update activation tracking
        self.last_activation_burst[neuron_idx] = current_burst;
        self.activation_count[neuron_idx] += 1;

        // Grow lifespan if not long-term memory
        if !self.is_longterm_memory[neuron_idx] {
            let current_lifespan = self.lifespan_current[neuron_idx];
            let growth = self.lifespan_growth_rate[neuron_idx] as u32;
            self.lifespan_current[neuron_idx] = current_lifespan.saturating_add(growth);
        }

        self.order_insert(neuron_idx);
        Some(neuron_idx)
    }

    /// Age short-term memory neurons that were not created or reactivated in `current_burst`.
    ///
    /// A neuron matched this burst keeps its full lifespan, so `init_lifespan` is the
    /// number of later bursts it survives unseen and each repeat adds the growth rate.
    pub fn age_memory_neurons(&mut self, current_burst: u64) -> Vec<usize> {
        let n = self.next_available_index;
        if n == 0 {
            return Vec::new();
        }

        let mut died_indices = Vec::new();
        // Neurons matched this burst keep their lifespan while the epoch moves on.
        let mut skipped = Vec::new();

        // Age eligible neurons
        for i in 0..n {
            if self.is_active[i] && !self.is_longterm_memory[i] && self.lifespan_current[i] > 0 {
                if self.last_activation_burst[i] == current_burst {
                    skipped.push(i);
                    continue;
                }
                self.lifespan_current[i] -= 1;

                // Check if neuron died
                if self.lifespan_current[i] == 0 {
                    self.is_active[i] = false;
                    died_indices.push(i);
                }
            }
        }

        // Clean up dead neurons after iteration
        for &i in &died_indices {
            self.removed.push(RemovedMemoryNeuron {
                neuron_id: self.neuron_ids[i],
                area_idx: self.cortical_area_ids[i],
                was_longterm: false,
                reason: MemoryNeuronRemovalReason::Expired,
                counted_in_stats: true,
            });
            self.cleanup_dead_neuron_internal(i);
        }

        // Every neuron that aged kept its key (lifespan fell as the epoch rose). The
        // skipped ones did not age, so their key rises with the epoch.
        for &i in &skipped {
            self.order_remove(i);
        }
        self.age_epoch += 1;
        for &i in &skipped {
            self.order_insert(i);
        }

        died_indices
    }

    /// Promote an active short-term neuron to long-term memory by global id.
    ///
    /// Uses the same conversion as `check_longterm_conversion`, so ordering indexes and
    /// long-term counts stay consistent. Returns the slot index when promoted, `None` when the
    /// id is not an active short-term memory neuron (unknown, expired, placeholder, or already
    /// long-term).
    pub fn promote_to_longterm_by_id(&mut self, neuron_id: u32) -> Option<usize> {
        let idx = self.find_neuron_index_by_global_id(neuron_id)?;
        if !self.is_valid_index(idx) || !self.is_active[idx] || self.is_longterm_memory[idx] {
            return None;
        }
        self.convert_to_longterm(idx);
        Some(idx)
    }

    /// Check for neurons ready for long-term memory conversion
    pub fn check_longterm_conversion(&mut self, longterm_threshold: u32) -> Vec<usize> {
        let n = self.next_available_index;
        if n == 0 {
            return Vec::new();
        }

        let mut converted_indices = Vec::new();

        for i in 0..n {
            if self.is_active[i]
                && !self.is_longterm_memory[i]
                && self.lifespan_current[i] >= longterm_threshold
            {
                self.convert_to_longterm(i);
                converted_indices.push(i);
            }
        }

        converted_indices
    }

    /// Check for neurons ready for long-term conversion using per-area thresholds.
    ///
    /// Areas without an explicit lifecycle config use `default_threshold`.
    pub fn check_longterm_conversion_by_area(
        &mut self,
        lifecycle_configs: &HashMap<u32, MemoryNeuronLifecycleConfig>,
        default_threshold: u32,
    ) -> Vec<usize> {
        let n = self.next_available_index;
        if n == 0 {
            return Vec::new();
        }

        let area_indices: Vec<(u32, Vec<usize>)> = self
            .area_neuron_indices
            .iter()
            .map(|(&area, indices)| (area, indices.iter().copied().collect()))
            .collect();

        let mut converted_indices = Vec::new();
        for (area_id, indices) in area_indices {
            let threshold = lifecycle_configs
                .get(&area_id)
                .map(|config| config.longterm_threshold)
                .unwrap_or(default_threshold);

            for neuron_idx in indices {
                if self.is_active[neuron_idx]
                    && !self.is_longterm_memory[neuron_idx]
                    && self.lifespan_current[neuron_idx] >= threshold
                {
                    self.convert_to_longterm(neuron_idx);
                    converted_indices.push(neuron_idx);
                }
            }
        }

        converted_indices
    }

    /// Get all active neuron IDs for a cortical area
    pub fn get_active_neurons_by_area(&self, cortical_area_id: u32) -> Vec<u32> {
        if let Some(indices) = self.area_neuron_indices.get(&cortical_area_id) {
            indices
                .iter()
                .filter(|&&idx| self.is_valid_index(idx) && self.is_active[idx])
                .map(|&idx| self.neuron_ids[idx])
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Active short-term neurons (not marked LTM) in the area.
    pub fn count_short_term_in_area(&self, cortical_area_id: u32) -> usize {
        self.area_neuron_indices
            .get(&cortical_area_id)
            .map(|indices| {
                indices
                    .iter()
                    .filter(|&&idx| {
                        self.is_valid_index(idx)
                            && self.is_active[idx]
                            && !self.is_longterm_memory[idx]
                    })
                    .count()
            })
            .unwrap_or(0)
    }

    /// Active long-term memory neurons in the area.
    pub fn count_long_term_in_area(&self, cortical_area_id: u32) -> usize {
        self.area_neuron_indices
            .get(&cortical_area_id)
            .map(|indices| {
                indices
                    .iter()
                    .filter(|&&idx| {
                        self.is_valid_index(idx)
                            && self.is_active[idx]
                            && self.is_longterm_memory[idx]
                    })
                    .count()
            })
            .unwrap_or(0)
    }

    /// Stable-sorted active neuron IDs for the area, with pagination `(page_ids, total_count)`.
    pub fn paginated_neuron_ids_in_area(
        &self,
        cortical_area_id: u32,
        offset: usize,
        limit: usize,
    ) -> (Vec<u32>, usize) {
        let mut ids = self.get_active_neurons_by_area(cortical_area_id);
        ids.sort_unstable();
        let total = ids.len();
        let page = ids.into_iter().skip(offset).take(limit).collect();
        (page, total)
    }

    fn find_neuron_index_by_global_id(&self, neuron_id: u32) -> Option<usize> {
        self.neuron_id_to_index.get(&neuron_id).copied()
    }

    /// True when `neuron_id` is an active memory neuron. No allocation; equivalent to
    /// `get_memory_neuron_detail(neuron_id).is_some()`.
    pub fn is_active_memory_neuron(&self, neuron_id: u32) -> bool {
        match self.find_neuron_index_by_global_id(neuron_id) {
            Some(idx) => self.is_valid_index(idx) && self.is_active[idx],
            None => false,
        }
    }

    /// Full detail for an active memory neuron by global neuron id, if present.
    pub fn get_memory_neuron_detail(&self, neuron_id: u32) -> Option<MemoryNeuronDetail> {
        let idx = self.find_neuron_index_by_global_id(neuron_id)?;
        if !self.is_valid_index(idx) || !self.is_active[idx] {
            return None;
        }
        Some(MemoryNeuronDetail {
            neuron_id: self.neuron_ids[idx],
            cortical_area_idx: self.cortical_area_ids[idx],
            pattern_hash: self.index_to_pattern_hash.get(&idx).copied(),
            is_longterm_memory: self.is_longterm_memory[idx],
            is_active: self.is_active[idx],
            lifespan_current: self.lifespan_current[idx],
            lifespan_initial: self.lifespan_initial[idx],
            lifespan_growth_rate: self.lifespan_growth_rate[idx],
            creation_burst: self.creation_burst[idx],
            last_activation_burst: self.last_activation_burst[idx],
            activation_count: self.activation_count[idx],
            spatial_signature: self.spatial_signature.get(&idx).copied(),
            class_channels: self
                .class_channels
                .get(&idx)
                .map(|set| {
                    let mut channels: Vec<u32> = set.iter().copied().collect();
                    channels.sort_unstable();
                    channels
                })
                .unwrap_or_default(),
            class_channel_weights: self
                .class_channel_weights
                .get(&idx)
                .map(|weights| {
                    let mut pairs: Vec<(u32, f32)> =
                        weights.iter().map(|(&c, &w)| (c, w)).collect();
                    pairs.sort_unstable_by_key(|(channel, _)| *channel);
                    pairs
                })
                .unwrap_or_default(),
        })
    }

    /// Active long-term memory neurons only. Short-term memory is never included.
    pub fn export_long_term_memory_neurons(&self) -> Vec<MemoryNeuronDetail> {
        (0..self.next_available_index)
            .filter_map(|idx| {
                if !self.is_valid_index(idx)
                    || !self.is_active[idx]
                    || !self.is_longterm_memory[idx]
                {
                    return None;
                }
                self.get_memory_neuron_detail(self.neuron_ids[idx])
            })
            .collect()
    }

    /// Replace the array with the given long-term memory neurons, restoring original IDs.
    ///
    /// Short-term entries in `neurons` are ignored. Existing STM/LTM state is discarded.
    ///
    /// When there are more records than an area's `memory_neuron_cap` (see
    /// [`Self::set_area_cap`]) or than the pool holds, the most recently activated records
    /// are kept (ties: higher neuron id). Dropped records keep their ids reserved, so a
    /// later creation cannot reuse one and inherit the NPU synapses or mappings that were
    /// restored for it, and each is queued as an evicted removal so downstream consumers
    /// clean up what they hold for it. Records with an out-of-range or duplicate id are
    /// skipped. One summary line reports everything dropped or skipped, and the call
    /// returns `Ok` with the number restored.
    pub fn restore_long_term_memory_neurons(
        &mut self,
        neurons: &[MemoryNeuronDetail],
    ) -> Result<usize, String> {
        self.reset();
        let mut candidates: Vec<&MemoryNeuronDetail> = neurons
            .iter()
            .filter(|detail| detail.is_longterm_memory && detail.is_active)
            .collect();
        let offered = candidates.len();

        // Newest first, so truncation below keeps the most recently activated records.
        candidates.sort_by(|a, b| {
            b.last_activation_burst
                .cmp(&a.last_activation_burst)
                .then(b.neuron_id.cmp(&a.neuron_id))
        });

        // Out-of-range and duplicate ids cannot be restored. Drop them before ranking so
        // they do not take a pool slot from a good record; the newest duplicate wins.
        let mut skipped_invalid = 0usize;
        let mut seen_ids: HashSet<u32> = HashSet::new();
        candidates.retain(|detail| {
            let usable = NeuronIdManager::is_memory_neuron_id(detail.neuron_id)
                && seen_ids.insert(detail.neuron_id);
            if !usable {
                skipped_invalid += 1;
            }
            usable
        });

        let mut dropped: Vec<&MemoryNeuronDetail> = Vec::new();
        let mut dropped_by_area_cap = 0usize;
        let mut kept_per_area: HashMap<u32, usize> = HashMap::new();
        let mut kept: Vec<&MemoryNeuronDetail> = Vec::with_capacity(candidates.len());
        for detail in candidates {
            let cap = self
                .area_caps
                .get(&detail.cortical_area_idx)
                .copied()
                .unwrap_or(0) as usize;
            let in_area = kept_per_area.entry(detail.cortical_area_idx).or_insert(0);
            if cap > 0 && *in_area >= cap {
                dropped_by_area_cap += 1;
                dropped.push(detail);
                continue;
            }
            *in_area += 1;
            kept.push(detail);
        }
        let dropped_by_pool = kept.len().saturating_sub(self.capacity);
        dropped.extend(kept.split_off(self.capacity.min(kept.len())));

        // Hold every dropped id so it is never handed out again.
        for detail in dropped {
            if self.id_manager.reserve_memory_neuron_id(detail.neuron_id) {
                self.removed.push(RemovedMemoryNeuron {
                    neuron_id: detail.neuron_id,
                    area_idx: detail.cortical_area_idx,
                    was_longterm: true,
                    reason: MemoryNeuronRemovalReason::Evicted,
                    counted_in_stats: false,
                });
            } else {
                skipped_invalid += 1;
            }
        }

        // Restore oldest first so slot order follows activation order.
        let mut restored = 0;
        for detail in kept.into_iter().rev() {
            match self.restore_long_term_memory_neuron(detail) {
                Ok(_) => restored += 1,
                Err(_) => skipped_invalid += 1,
            }
        }
        let dropped_total = dropped_by_area_cap + dropped_by_pool;
        if dropped_total + skipped_invalid > 0 {
            tracing::warn!(
                target: "plasticity",
                "[PLASTICITY] LTM restore kept {} of {} records (most recently activated first): {} over an area memory_neuron_cap, {} over the pool capacity {}, {} skipped for an out-of-range or duplicate id",
                restored,
                offered,
                dropped_by_area_cap,
                dropped_by_pool,
                self.capacity,
                skipped_invalid
            );
        }
        Ok(restored)
    }

    fn restore_long_term_memory_neuron(
        &mut self,
        detail: &MemoryNeuronDetail,
    ) -> Result<usize, String> {
        if !NeuronIdManager::is_memory_neuron_id(detail.neuron_id) {
            return Err(format!(
                "Cannot restore LTM neuron {}: id is outside the memory range",
                detail.neuron_id
            ));
        }
        if !self.id_manager.reserve_memory_neuron_id(detail.neuron_id) {
            return Err(format!(
                "Cannot restore LTM neuron {}: id is already reserved",
                detail.neuron_id
            ));
        }
        let neuron_idx = self.get_available_index_internal().ok_or_else(|| {
            format!(
                "Cannot restore LTM neuron {}: memory array is at capacity {}",
                detail.neuron_id, self.capacity
            )
        })?;

        self.neuron_ids[neuron_idx] = detail.neuron_id;
        self.neuron_id_to_index.insert(detail.neuron_id, neuron_idx);
        self.cortical_area_ids[neuron_idx] = detail.cortical_area_idx;
        self.is_active[neuron_idx] = true;
        self.lifespan_current[neuron_idx] = detail.lifespan_current;
        self.lifespan_initial[neuron_idx] = detail.lifespan_initial;
        self.lifespan_growth_rate[neuron_idx] = detail.lifespan_growth_rate;
        self.is_longterm_memory[neuron_idx] = true;
        self.creation_burst[neuron_idx] = detail.creation_burst;
        self.last_activation_burst[neuron_idx] = detail.last_activation_burst;
        self.activation_count[neuron_idx] = detail.activation_count;

        if let Some(pattern_hash) = detail.pattern_hash {
            self.pattern_hash_to_index
                .insert((detail.cortical_area_idx, pattern_hash), neuron_idx);
            self.index_to_pattern_hash.insert(neuron_idx, pattern_hash);
        }
        if let Some(spatial_signature) = detail.spatial_signature {
            self.spatial_signature.insert(neuron_idx, spatial_signature);
        }
        if !detail.class_channels.is_empty() {
            self.class_channels
                .insert(neuron_idx, detail.class_channels.iter().copied().collect());
        }
        if !detail.class_channel_weights.is_empty() {
            self.class_channel_weights.insert(
                neuron_idx,
                detail.class_channel_weights.iter().copied().collect(),
            );
        }
        if detail.is_longterm_memory {
            self.index_spatial_ltm(neuron_idx);
        }

        self.area_neuron_indices
            .entry(detail.cortical_area_idx)
            .or_default()
            .insert(neuron_idx);
        self.active_count += 1;
        self.ltm_count += 1;
        self.order_insert(neuron_idx);

        Ok(neuron_idx)
    }

    /// Find the active neuron a memory area holds for a pattern hash
    pub fn find_neuron_by_pattern(
        &self,
        cortical_area_id: u32,
        pattern_hash: &u64,
    ) -> Option<usize> {
        self.pattern_hash_to_index
            .get(&(cortical_area_id, *pattern_hash))
            .copied()
            .filter(|&idx| self.is_valid_index(idx) && self.is_active[idx])
    }

    /// Get neuron ID at index
    pub fn get_neuron_id(&self, neuron_idx: usize) -> Option<u32> {
        if self.is_valid_index(neuron_idx) {
            Some(self.neuron_ids[neuron_idx])
        } else {
            None
        }
    }

    /// Activations since creation. One means the neuron was created on this call.
    pub fn get_activation_count(&self, neuron_idx: usize) -> Option<u32> {
        if self.is_valid_index(neuron_idx) && self.is_active[neuron_idx] {
            Some(self.activation_count[neuron_idx])
        } else {
            None
        }
    }

    /// Get cortical area ID at index
    pub fn get_cortical_area_id(&self, neuron_idx: usize) -> Option<u32> {
        if self.is_valid_index(neuron_idx) {
            Some(self.cortical_area_ids[neuron_idx])
        } else {
            None
        }
    }

    pub fn get_pattern_hash(&self, neuron_idx: usize) -> Option<u64> {
        self.index_to_pattern_hash.get(&neuron_idx).copied()
    }

    /// Store the scan occupancy hash. Indexed for LTM lookup only after conversion.
    pub fn set_spatial_signature(&mut self, neuron_idx: usize, spatial_signature: u64) {
        if !self.is_valid_index(neuron_idx) || !self.is_active[neuron_idx] {
            return;
        }
        if let Some(previous) = self.spatial_signature.insert(neuron_idx, spatial_signature) {
            if previous != spatial_signature {
                self.remove_spatial_ltm_index(neuron_idx, previous);
            }
        }
        if self.is_longterm_memory[neuron_idx] {
            self.index_spatial_ltm(neuron_idx);
        }
    }

    /// Bind a class channel onto a memory neuron (OR with existing channels).
    pub fn bind_class_channel(&mut self, neuron_idx: usize, class_channel: u32) {
        if !self.is_valid_index(neuron_idx) || !self.is_active[neuron_idx] {
            return;
        }
        self.class_channels
            .entry(neuron_idx)
            .or_default()
            .insert(class_channel);
    }

    /// Drop one class channel and its associative weight.
    pub fn unbind_class_channel(&mut self, neuron_idx: usize, class_channel: u32) {
        if let Some(channels) = self.class_channels.get_mut(&neuron_idx) {
            channels.remove(&class_channel);
            if channels.is_empty() {
                self.class_channels.remove(&neuron_idx);
            }
        }
        if let Some(weights) = self.class_channel_weights.get_mut(&neuron_idx) {
            weights.remove(&class_channel);
            if weights.is_empty() {
                self.class_channel_weights.remove(&neuron_idx);
            }
        }
    }

    /// Record the starting associative weight the first time reward training sees a channel.
    pub fn ensure_class_channel_weight(
        &mut self,
        neuron_idx: usize,
        class_channel: u32,
        initial: f32,
    ) {
        if initial <= 0.0 {
            return;
        }
        self.class_channel_weights
            .entry(neuron_idx)
            .or_default()
            .entry(class_channel)
            .or_insert(initial);
    }

    /// Move one class-channel associative weight. A result at or below zero unbinds the channel.
    ///
    /// A channel that has no stored weight starts from `initial` before `delta` is applied.
    pub fn apply_class_channel_delta(
        &mut self,
        neuron_idx: usize,
        class_channel: u32,
        delta: f32,
        max_weight: f32,
        initial: f32,
    ) -> f32 {
        let current = self
            .class_channel_weights
            .get(&neuron_idx)
            .and_then(|weights| weights.get(&class_channel).copied())
            .unwrap_or(initial);
        let mut next = current + delta;
        if next < 0.0 {
            next = 0.0;
        }
        if max_weight.is_finite() {
            next = next.min(max_weight);
        }
        if next <= 0.0 {
            self.unbind_class_channel(neuron_idx, class_channel);
            return 0.0;
        }
        self.bind_class_channel(neuron_idx, class_channel);
        self.class_channel_weights
            .entry(neuron_idx)
            .or_default()
            .insert(class_channel, next);
        next
    }

    pub fn presentation_ledger_mut(&mut self) -> &mut crate::classifier_reward::PresentationLedger {
        &mut self.presentation_ledger
    }

    pub fn class_channel_weight(&self, neuron_idx: usize, class_channel: u32) -> Option<f32> {
        self.class_channel_weights
            .get(&neuron_idx)
            .and_then(|weights| weights.get(&class_channel).copied())
    }

    pub fn get_class_channels(&self, neuron_idx: usize) -> Vec<u32> {
        self.class_channels
            .get(&neuron_idx)
            .map(|set| {
                let mut channels: Vec<u32> = set.iter().copied().collect();
                channels.sort_unstable();
                channels
            })
            .unwrap_or_default()
    }

    /// Active long-term memory neuron indices in an area.
    pub fn active_ltm_indices_in_area(&self, cortical_area_id: u32) -> Vec<usize> {
        self.area_neuron_indices
            .get(&cortical_area_id)
            .map(|indices| {
                indices
                    .iter()
                    .copied()
                    .filter(|&idx| {
                        self.is_valid_index(idx)
                            && self.is_active[idx]
                            && self.is_longterm_memory[idx]
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// LTM neurons in `cortical_area_id` whose spatial signature matches.
    pub fn find_ltm_by_spatial_signature(
        &self,
        cortical_area_id: u32,
        spatial_signature: u64,
    ) -> Vec<usize> {
        self.spatial_ltm_index
            .get(&(cortical_area_id, spatial_signature))
            .map(|indices| {
                indices
                    .iter()
                    .copied()
                    .filter(|&idx| {
                        self.is_valid_index(idx)
                            && self.is_active[idx]
                            && self.is_longterm_memory[idx]
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn index_spatial_ltm(&mut self, neuron_idx: usize) {
        let Some(&spatial_signature) = self.spatial_signature.get(&neuron_idx) else {
            return;
        };
        let area_id = self.cortical_area_ids[neuron_idx];
        let key = (area_id, spatial_signature);
        let entries = self.spatial_ltm_index.entry(key).or_default();
        if !entries.contains(&neuron_idx) {
            entries.push(neuron_idx);
        }
    }

    fn remove_spatial_ltm_index(&mut self, neuron_idx: usize, spatial_signature: u64) {
        let area_id = self.cortical_area_ids[neuron_idx];
        let key = (area_id, spatial_signature);
        if let Some(entries) = self.spatial_ltm_index.get_mut(&key) {
            entries.retain(|&idx| idx != neuron_idx);
            if entries.is_empty() {
                self.spatial_ltm_index.remove(&key);
            }
        }
    }

    fn clear_scan_sidecars(&mut self, neuron_idx: usize) {
        if let Some(spatial_signature) = self.spatial_signature.remove(&neuron_idx) {
            self.remove_spatial_ltm_index(neuron_idx, spatial_signature);
        }
        self.class_channels.remove(&neuron_idx);
        self.class_channel_weights.remove(&neuron_idx);
    }

    /// Get comprehensive statistics
    pub fn get_stats(&self) -> MemoryNeuronStats {
        let n = self.next_available_index;

        if n == 0 {
            return MemoryNeuronStats {
                total_capacity: self.capacity,
                ..Default::default()
            };
        }

        let active_count = self.is_active[..n].iter().filter(|&&x| x).count();
        let longterm_count = (0..n)
            .filter(|&i| self.is_active[i] && self.is_longterm_memory[i])
            .count();
        let dead_count = n - active_count;

        // Calculate averages for active neurons
        let (avg_lifespan, avg_activation_count) = if active_count > 0 {
            let total_lifespan: u32 = (0..n)
                .filter(|&i| self.is_active[i])
                .map(|i| self.lifespan_current[i])
                .sum();
            let total_activations: u32 = (0..n)
                .filter(|&i| self.is_active[i])
                .map(|i| self.activation_count[i])
                .sum();

            (
                total_lifespan as f64 / active_count as f64,
                total_activations as f64 / active_count as f64,
            )
        } else {
            (0.0, 0.0)
        };

        // Estimate memory usage
        let memory_usage = self.capacity * (
            std::mem::size_of::<u32>() * 4 +  // uint32 arrays
            std::mem::size_of::<f32>() +       // float32 array
            std::mem::size_of::<u64>() * 2 +   // uint64 arrays
            std::mem::size_of::<bool>() * 2    // bool arrays
        ) + self.pattern_hash_to_index.len() * (4 + 8 + 8)  // Pattern mappings ((u32, u64) + usize)
          + self.area_neuron_indices.len() * 64; // Area tracking overhead

        MemoryNeuronStats {
            total_capacity: self.capacity,
            active_neurons: active_count,
            longterm_neurons: longterm_count,
            dead_neurons: dead_count,
            reusable_indices: self.reusable_indices.len(),
            memory_usage_bytes: memory_usage,
            avg_lifespan,
            avg_activation_count,
        }
    }

    /// Get ID manager allocation statistics
    pub fn get_id_allocation_stats(&self) -> AllocationStats {
        self.id_manager.get_allocation_stats()
    }

    /// Get available index (reuse or allocate new)
    fn get_available_index_internal(&mut self) -> Option<usize> {
        // Try to reuse a dead neuron index first
        if let Some(&idx) = self.reusable_indices.iter().next() {
            self.reusable_indices.remove(&idx);
            return Some(idx);
        }

        // Allocate new index if capacity allows
        if self.next_available_index < self.capacity {
            let idx = self.next_available_index;
            self.next_available_index += 1;
            Some(idx)
        } else {
            None
        }
    }

    /// Clean up associations for a dead neuron
    fn cleanup_dead_neuron_internal(&mut self, neuron_idx: usize) {
        // Leave the ordered indexes while the flags and key they depend on are intact.
        self.order_remove(neuron_idx);
        if self.is_longterm_memory[neuron_idx] {
            self.ltm_count = self.ltm_count.saturating_sub(1);
            self.is_longterm_memory[neuron_idx] = false;
        }
        self.active_count = self.active_count.saturating_sub(1);

        // Deallocate global neuron ID
        let neuron_id = self.neuron_ids[neuron_idx];
        if self.neuron_id_to_index.get(&neuron_id) == Some(&neuron_idx) {
            self.neuron_id_to_index.remove(&neuron_id);
        }
        self.id_manager.deallocate_memory_neuron_id(neuron_id);

        let area_id = self.cortical_area_ids[neuron_idx];

        // Remove pattern association
        if let Some(pattern_hash) = self.index_to_pattern_hash.remove(&neuron_idx) {
            self.pattern_hash_to_index.remove(&(area_id, pattern_hash));
        }
        self.clear_scan_sidecars(neuron_idx);

        // Remove from area tracking
        if let Some(indices) = self.area_neuron_indices.get_mut(&area_id) {
            indices.remove(&neuron_idx);
        }

        // Add to reusable indices
        self.reusable_indices.insert(neuron_idx);
    }

    /// Check if neuron index is valid
    fn is_valid_index(&self, neuron_idx: usize) -> bool {
        neuron_idx < self.next_available_index
    }

    /// Reset array state (for testing)
    /// Deactivate all memory neurons in a specific cortical area
    pub fn reset_cortical_area(&mut self, cortical_area_id: u32) -> usize {
        let Some(neuron_indices) = self.area_neuron_indices.get(&cortical_area_id) else {
            return 0;
        };

        let indices_to_reset: Vec<usize> = neuron_indices.iter().copied().collect();
        let mut reset_count = 0;

        for neuron_idx in indices_to_reset {
            if !self.is_valid_index(neuron_idx) || !self.is_active[neuron_idx] {
                continue;
            }

            self.is_active[neuron_idx] = false;
            self.cleanup_dead_neuron_internal(neuron_idx);
            self.is_longterm_memory[neuron_idx] = false;
            self.lifespan_current[neuron_idx] = 0;
            self.lifespan_initial[neuron_idx] = 0;
            self.lifespan_growth_rate[neuron_idx] = 0.0;
            self.activation_count[neuron_idx] = 0;
            self.creation_burst[neuron_idx] = 0;
            self.last_activation_burst[neuron_idx] = 0;

            reset_count += 1;
        }

        // Remove area tracking
        self.area_neuron_indices.remove(&cortical_area_id);

        reset_count
    }

    pub fn reset(&mut self) {
        self.neuron_ids.fill(0);
        self.neuron_id_to_index.clear();
        self.cortical_area_ids.fill(0);
        self.is_active.fill(false);
        self.lifespan_current.fill(0);
        self.lifespan_initial.fill(0);
        self.lifespan_growth_rate.fill(0.0);
        self.is_longterm_memory.fill(false);
        self.creation_burst.fill(0);
        self.last_activation_burst.fill(0);
        self.activation_count.fill(0);

        self.pattern_hash_to_index.clear();
        self.index_to_pattern_hash.clear();
        self.area_neuron_indices.clear();
        self.spatial_signature.clear();
        self.class_channels.clear();
        self.class_channel_weights.clear();
        self.presentation_ledger = crate::classifier_reward::PresentationLedger::default();
        self.spatial_ltm_index.clear();

        self.next_available_index = 0;
        self.reusable_indices.clear();

        self.stm_order.clear();
        self.ltm_order.clear();
        self.order_key.fill(0);
        self.age_epoch = 0;
        self.active_count = 0;
        self.ltm_count = 0;
        self.removed.clear();

        self.id_manager.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_config_default() {
        let config = MemoryNeuronLifecycleConfig::default();
        assert_eq!(config.initial_lifespan, 20);
        assert_eq!(config.lifespan_growth_rate, 3.0);
        assert_eq!(config.longterm_threshold, 100);
        assert_eq!(config.max_reactivations, 1000);
    }

    #[test]
    fn test_create_memory_neuron() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig::default();

        let pattern_hash = 0x0101010101010101u64;
        let neuron_idx = array.create_memory_neuron(pattern_hash, 100, 0, &config);

        assert!(neuron_idx.is_some());
        let idx = neuron_idx.unwrap();
        assert!(array.is_active[idx]);
        assert_eq!(array.cortical_area_ids[idx], 100);
        assert_eq!(array.lifespan_current[idx], config.initial_lifespan);
        assert_eq!(array.activation_count[idx], 1);
        assert_eq!(array.creation_burst[idx], 0);
    }

    #[test]
    fn reward_consolidation_promotes_and_survives_expiry() {
        let mut array = MemoryNeuronArray::new(16);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 3,
            ..Default::default()
        };
        let promoted = array.create_memory_neuron(1, 100, 0, &config).unwrap();
        let control = array.create_memory_neuron(2, 100, 0, &config).unwrap();
        let promoted_id = array.neuron_ids[promoted];

        assert_eq!(array.promote_to_longterm_by_id(promoted_id), Some(promoted));
        assert!(array.is_longterm_memory[promoted]);
        // Already long-term, and a placeholder (unknown) id, are not promoted again.
        assert_eq!(array.promote_to_longterm_by_id(promoted_id), None);
        assert_eq!(array.promote_to_longterm_by_id(7), None);

        let mut died = Vec::new();
        for burst in 1..=10 {
            died.extend(array.age_memory_neurons(burst));
        }
        assert!(died.contains(&control), "unpromoted neuron expires");
        assert!(
            !died.contains(&promoted),
            "promoted neuron outlives init_lifespan"
        );
        assert!(array.is_active[promoted]);
    }

    #[test]
    fn test_create_duplicate_pattern() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig::default();

        let pattern_hash = 0x0101010101010101u64;
        let idx1 = array
            .create_memory_neuron(pattern_hash, 100, 0, &config)
            .unwrap();

        // Creating with same pattern should reactivate existing neuron
        let idx2 = array
            .create_memory_neuron(pattern_hash, 100, 1, &config)
            .unwrap();

        assert_eq!(idx1, idx2);
        assert_eq!(array.activation_count[idx1], 2); // Should have been reactivated
    }

    #[test]
    fn test_multiple_neurons() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig::default();

        let mut neurons = Vec::new();
        for i in 0..10 {
            let pattern_hash = i as u64;
            let idx = array
                .create_memory_neuron(pattern_hash, 100, 0, &config)
                .unwrap();
            neurons.push(idx);
        }

        assert_eq!(neurons.len(), 10);
        assert_eq!(array.next_available_index, 10);

        let stats = array.get_stats();
        assert_eq!(stats.active_neurons, 10);
    }

    #[test]
    fn test_reactivate_memory_neuron() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig::default();

        let pattern_hash = 0x0101010101010101u64;
        let idx = array
            .create_memory_neuron(pattern_hash, 100, 0, &config)
            .unwrap();

        let initial_count = array.activation_count[idx];
        let initial_lifespan = array.lifespan_current[idx];

        assert!(array.reactivate_memory_neuron(idx, 1));

        assert_eq!(array.activation_count[idx], initial_count + 1);
        assert_eq!(array.last_activation_burst[idx], 1);

        // Lifespan should have grown
        let expected_lifespan = initial_lifespan + config.lifespan_growth_rate as u32;
        assert_eq!(array.lifespan_current[idx], expected_lifespan);
    }

    #[test]
    fn test_reactivate_invalid_neuron() {
        let mut array = MemoryNeuronArray::new(1000);

        // Try to reactivate non-existent neuron
        assert!(!array.reactivate_memory_neuron(0, 1));
        assert!(!array.reactivate_memory_neuron(999, 1));
    }

    #[test]
    fn test_age_memory_neurons() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 2,
            ..Default::default()
        };

        let pattern_hash = 0x0101010101010101u64;
        let idx = array
            .create_memory_neuron(pattern_hash, 100, 0, &config)
            .unwrap();

        // Age once
        let died = array.age_memory_neurons(1);
        assert!(died.is_empty());
        assert_eq!(array.lifespan_current[idx], 1);

        // Age again - should die
        let died = array.age_memory_neurons(2);
        assert_eq!(died.len(), 1);
        assert_eq!(died[0], idx);
        assert!(!array.is_active[idx]);

        // Pattern should no longer be findable
        let found = array.find_neuron_by_pattern(100, &pattern_hash);
        assert!(found.is_none());
    }

    #[test]
    fn neuron_matched_this_burst_is_not_aged() {
        let mut array = MemoryNeuronArray::new(16);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 1,
            ..Default::default()
        };
        let idx = array.create_memory_neuron(7, 100, 5, &config).unwrap();

        assert!(array.age_memory_neurons(5).is_empty());
        assert_eq!(array.lifespan_current[idx], 1);

        assert_eq!(array.age_memory_neurons(6), vec![idx]);
    }

    /// Per-burst order used by the plasticity service: promote, match, then age.
    fn run_burst(
        array: &mut MemoryNeuronArray,
        configs: &HashMap<u32, MemoryNeuronLifecycleConfig>,
        burst: u64,
        pattern_seen: bool,
    ) -> Vec<usize> {
        let config = configs[&100];
        let converted = array.check_longterm_conversion_by_area(configs, config.longterm_threshold);
        if pattern_seen {
            array.create_memory_neuron(42, 100, burst, &config);
        }
        array.age_memory_neurons(burst);
        converted
    }

    #[test]
    fn lifespan_one_growth_one_reaches_ltm_when_pattern_repeats() {
        let mut array = MemoryNeuronArray::new(16);
        let configs = HashMap::from([(
            100u32,
            MemoryNeuronLifecycleConfig {
                initial_lifespan: 1,
                lifespan_growth_rate: 1.0,
                longterm_threshold: 3,
                max_reactivations: 1000,
                ..MemoryNeuronLifecycleConfig::default()
            },
        )]);

        for burst in 0..3 {
            assert!(run_burst(&mut array, &configs, burst, true).is_empty());
        }
        let converted = run_burst(&mut array, &configs, 3, true);
        assert_eq!(converted.len(), 1);
        assert!(array.is_longterm_memory[converted[0]]);
    }

    #[test]
    fn lifespan_one_pattern_seen_once_dies_after_one_unseen_burst() {
        let mut array = MemoryNeuronArray::new(16);
        let configs = HashMap::from([(
            100u32,
            MemoryNeuronLifecycleConfig {
                initial_lifespan: 1,
                lifespan_growth_rate: 1.0,
                longterm_threshold: 3,
                max_reactivations: 1000,
                ..MemoryNeuronLifecycleConfig::default()
            },
        )]);

        run_burst(&mut array, &configs, 0, true);
        assert!(array.find_neuron_by_pattern(100, &42).is_some());
        run_burst(&mut array, &configs, 1, false);
        assert!(array.find_neuron_by_pattern(100, &42).is_none());
    }

    #[test]
    fn test_age_multiple_neurons() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 5,
            ..Default::default()
        };

        let mut neurons = Vec::new();
        for i in 0..10 {
            let pattern_hash = i as u64;
            let idx = array
                .create_memory_neuron(pattern_hash, 100, 0, &config)
                .unwrap();
            neurons.push(idx);
        }

        // Age 5 times - all should die
        for burst in 1..=5 {
            let died = array.age_memory_neurons(burst);
            if burst < 5 {
                assert_eq!(died.len(), 0);
            } else {
                assert_eq!(died.len(), 10);
            }
        }

        let stats = array.get_stats();
        assert_eq!(stats.active_neurons, 0);
        assert_eq!(stats.dead_neurons, 10);
    }

    #[test]
    fn test_longterm_memory_no_aging() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 100,
            ..Default::default()
        };

        let pattern_hash = 0x0101010101010101u64;
        let idx = array
            .create_memory_neuron(pattern_hash, 100, 0, &config)
            .unwrap();

        // Convert to long-term memory
        let converted = array.check_longterm_conversion(100);
        assert_eq!(converted.len(), 1);
        assert!(array.is_longterm_memory[idx]);

        let initial_lifespan = array.lifespan_current[idx];

        // Age many times - should not affect long-term memory
        for burst in 1..=50 {
            array.age_memory_neurons(burst);
        }

        assert!(array.is_active[idx]);
        assert_eq!(array.lifespan_current[idx], initial_lifespan); // Should not change
    }

    #[test]
    fn test_longterm_conversion() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 100,
            ..Default::default()
        };

        let pattern_hash = 0x0101010101010101u64;
        let idx = array
            .create_memory_neuron(pattern_hash, 100, 0, &config)
            .unwrap();

        let converted = array.check_longterm_conversion(100);
        assert_eq!(converted.len(), 1);
        assert_eq!(converted[0], idx);
        assert!(array.is_longterm_memory[idx]);

        // Second check should not convert again
        let converted2 = array.check_longterm_conversion(100);
        assert_eq!(converted2.len(), 0);
    }

    #[test]
    fn test_longterm_conversion_threshold() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 50,
            ..Default::default()
        };

        let pattern_hash = 0x0101010101010101u64;
        let idx = array
            .create_memory_neuron(pattern_hash, 100, 0, &config)
            .unwrap();

        // Should not convert below threshold
        let converted = array.check_longterm_conversion(100);
        assert_eq!(converted.len(), 0);
        assert!(!array.is_longterm_memory[idx]);

        // Grow lifespan through reactivations
        for burst in 1..=20 {
            array.reactivate_memory_neuron(idx, burst);
        }

        // Now should convert
        let converted = array.check_longterm_conversion(100);
        assert_eq!(converted.len(), 1);
        assert!(array.is_longterm_memory[idx]);
    }

    #[test]
    fn test_find_neuron_by_pattern() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig::default();

        let pattern_hash = 0x0101010101010101u64;
        let idx = array
            .create_memory_neuron(pattern_hash, 100, 0, &config)
            .unwrap();

        let found = array.find_neuron_by_pattern(100, &pattern_hash);
        assert_eq!(found, Some(idx));

        // Different pattern should not be found
        let pattern_hash2 = 0x0202020202020202u64;
        let found2 = array.find_neuron_by_pattern(100, &pattern_hash2);
        assert_eq!(found2, None);
    }

    #[test]
    fn test_get_active_neurons_by_area() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig::default();

        // Create neurons in different areas
        for area in [100, 200] {
            for i in 0..5 {
                let pattern_hash = ((area as u64) << 32) | (i as u64);
                array.create_memory_neuron(pattern_hash, area, 0, &config);
            }
        }

        let area100_neurons = array.get_active_neurons_by_area(100);
        let area200_neurons = array.get_active_neurons_by_area(200);

        assert_eq!(area100_neurons.len(), 5);
        assert_eq!(area200_neurons.len(), 5);

        // Non-existent area
        let area999_neurons = array.get_active_neurons_by_area(999);
        assert_eq!(area999_neurons.len(), 0);
    }

    #[test]
    fn test_get_neuron_id() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig::default();

        let pattern_hash = 0x0101010101010101u64;
        let idx = array
            .create_memory_neuron(pattern_hash, 100, 0, &config)
            .unwrap();

        let neuron_id = array.get_neuron_id(idx);
        assert!(neuron_id.is_some());

        // Invalid index
        let invalid_id = array.get_neuron_id(999);
        assert!(invalid_id.is_none());
    }

    #[test]
    fn test_index_reuse() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 1,
            ..Default::default()
        };

        let pattern_hash = 0x0101010101010101u64;
        let idx1 = array
            .create_memory_neuron(pattern_hash, 100, 0, &config)
            .unwrap();
        assert_eq!(idx1, 0);

        // Let it die
        array.age_memory_neurons(1);
        assert!(!array.is_active[idx1]);

        // Create new neuron - should reuse index
        let pattern_hash2 = 0x0202020202020202u64;
        let idx2 = array
            .create_memory_neuron(pattern_hash2, 100, 2, &config)
            .unwrap();
        assert_eq!(idx2, 0); // Should reuse index 0
        assert_eq!(array.next_available_index, 1); // Should not have advanced
    }

    #[test]
    fn test_get_stats() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig::default();

        // Create some neurons
        for i in 0..10 {
            let pattern_hash = i as u64;
            array.create_memory_neuron(pattern_hash, 100, 0, &config);
        }

        let stats = array.get_stats();
        assert_eq!(stats.total_capacity, 1000);
        assert_eq!(stats.active_neurons, 10);
        assert_eq!(stats.longterm_neurons, 0);
        assert_eq!(stats.dead_neurons, 0);
        assert!(stats.avg_lifespan > 0.0);
        assert!(stats.avg_activation_count >= 1.0);
        assert!(stats.memory_usage_bytes > 0);
    }

    #[test]
    fn test_capacity_exhaustion() {
        let mut array = MemoryNeuronArray::new(5);
        let config = MemoryNeuronLifecycleConfig::default();

        // Create neurons up to capacity
        for i in 0..5 {
            let pattern_hash = i as u64;
            let idx = array.create_memory_neuron(pattern_hash, 100, 0, &config);
            assert!(idx.is_some());
        }

        // Try to create beyond capacity
        let pattern_hash = 0x6363636363636363u64;
        let idx = array.create_memory_neuron(pattern_hash, 100, 0, &config);
        assert!(idx.is_none());
    }

    #[test]
    fn test_reset_cortical_area() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig::default();

        // Create neurons in area 5
        let pattern1 = 0x0101010101010101u64;
        let pattern2 = 0x0202020202020202u64;
        array.create_memory_neuron(pattern1, 5, 0, &config);
        array.create_memory_neuron(pattern2, 5, 0, &config);

        // Create neurons in area 6
        let pattern3 = 0x0303030303030303u64;
        array.create_memory_neuron(pattern3, 6, 0, &config);

        // Verify initial state
        assert_eq!(array.get_active_neurons_by_area(5).len(), 2);
        assert_eq!(array.get_active_neurons_by_area(6).len(), 1);

        // Reset area 5
        let reset_count = array.reset_cortical_area(5);
        assert_eq!(reset_count, 2);

        // Verify area 5 is empty
        assert_eq!(array.get_active_neurons_by_area(5).len(), 0);

        // Verify area 6 is unchanged
        assert_eq!(array.get_active_neurons_by_area(6).len(), 1);

        // Verify pattern hashes are cleared for area 5
        assert!(!array.pattern_hash_to_index.contains_key(&(5, pattern1)));
        assert!(!array.pattern_hash_to_index.contains_key(&(5, pattern2)));

        // Verify area 6 pattern still exists
        assert!(array.pattern_hash_to_index.contains_key(&(6, pattern3)));
    }

    #[test]
    fn test_same_pattern_in_two_areas_gets_independent_neurons() {
        let mut array = MemoryNeuronArray::new(16);
        let config = MemoryNeuronLifecycleConfig::default();
        let hash = 0xABCDu64;

        let a = array.create_memory_neuron(hash, 100, 0, &config).unwrap();
        let b = array.create_memory_neuron(hash, 101, 0, &config).unwrap();
        assert_ne!(a, b, "each area owns its own neuron for a shared pattern");
        assert_ne!(array.get_neuron_id(a), array.get_neuron_id(b));
        assert_eq!(array.get_cortical_area_id(a), Some(100));
        assert_eq!(array.get_cortical_area_id(b), Some(101));
        assert_eq!(array.find_neuron_by_pattern(100, &hash), Some(a));
        assert_eq!(array.find_neuron_by_pattern(101, &hash), Some(b));
        assert_eq!(array.find_neuron_by_pattern(102, &hash), None);

        // Seeing the pattern again in area 100 reactivates only area 100's neuron.
        assert_eq!(array.create_memory_neuron(hash, 100, 1, &config), Some(a));
        assert_eq!(array.get_activation_count(a), Some(2));
        assert_eq!(array.get_activation_count(b), Some(1));

        // Resetting one area leaves the other's neuron and mapping intact.
        assert_eq!(array.reset_cortical_area(100), 1);
        assert_eq!(array.find_neuron_by_pattern(100, &hash), None);
        assert_eq!(array.find_neuron_by_pattern(101, &hash), Some(b));
    }

    #[test]
    fn test_reset() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig::default();

        // Create some neurons
        for i in 0..5 {
            let pattern_hash = i as u64;
            array.create_memory_neuron(pattern_hash, 100, 0, &config);
        }

        assert_eq!(array.next_available_index, 5);

        array.reset();

        assert_eq!(array.next_available_index, 0);
        let stats = array.get_stats();
        assert_eq!(stats.active_neurons, 0);
    }

    #[test]
    fn test_lifespan_growth_on_reactivation() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 10,
            lifespan_growth_rate: 5.0,
            longterm_threshold: 100,
            max_reactivations: 1000,
            ..MemoryNeuronLifecycleConfig::default()
        };

        let pattern_hash = 0x0101010101010101u64;
        let idx = array
            .create_memory_neuron(pattern_hash, 100, 0, &config)
            .unwrap();

        assert_eq!(array.lifespan_current[idx], 10);

        array.reactivate_memory_neuron(idx, 1);
        assert_eq!(array.lifespan_current[idx], 15);

        array.reactivate_memory_neuron(idx, 2);
        assert_eq!(array.lifespan_current[idx], 20);
    }

    #[test]
    fn test_st_ltm_counts_and_pagination_and_detail() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig::default();

        for i in 0..5 {
            array
                .create_memory_neuron(i as u64, 7, 0, &config)
                .expect("create");
        }
        assert_eq!(array.count_short_term_in_area(7), 5);
        assert_eq!(array.count_long_term_in_area(7), 0);

        array.is_longterm_memory[0] = true;
        assert_eq!(array.count_short_term_in_area(7), 4);
        assert_eq!(array.count_long_term_in_area(7), 1);

        let (page0, total) = array.paginated_neuron_ids_in_area(7, 0, 2);
        assert_eq!(total, 5);
        assert_eq!(page0.len(), 2);

        let nid = page0[0];
        let detail = array.get_memory_neuron_detail(nid).expect("detail");
        assert_eq!(detail.neuron_id, nid);
        assert_eq!(detail.cortical_area_idx, 7);
        assert!(detail.pattern_hash.is_some());
    }

    #[test]
    fn export_and_restore_preserves_only_long_term_memory() {
        let mut array = MemoryNeuronArray::new(1000);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 10,
            lifespan_growth_rate: 5.0,
            longterm_threshold: 20,
            max_reactivations: 1000,
            ..MemoryNeuronLifecycleConfig::default()
        };

        let stm_idx = array.create_memory_neuron(0x11, 3, 0, &config).unwrap();
        let ltm_idx = array.create_memory_neuron(0x22, 3, 0, &config).unwrap();
        array.lifespan_current[ltm_idx] = 25;
        let converted = array.check_longterm_conversion(20);
        assert_eq!(converted, vec![ltm_idx]);
        assert!(!array.is_longterm_memory[stm_idx]);

        let ltm_id = array.neuron_ids[ltm_idx];
        let exported = array.export_long_term_memory_neurons();
        assert_eq!(exported.len(), 1);
        assert_eq!(exported[0].neuron_id, ltm_id);
        assert!(exported[0].is_longterm_memory);

        let mut restored = MemoryNeuronArray::new(1000);
        let count = restored
            .restore_long_term_memory_neurons(&exported)
            .unwrap();
        assert_eq!(count, 1);
        let restored_ids: Vec<u32> = restored
            .export_long_term_memory_neurons()
            .iter()
            .map(|n| n.neuron_id)
            .collect();
        assert_eq!(restored_ids, vec![ltm_id]);
        assert_eq!(restored.get_active_neurons_by_area(3).len(), 1);

        let stm_only = vec![MemoryNeuronDetail {
            neuron_id: 50_000_099,
            cortical_area_idx: 3,
            pattern_hash: Some(0x33),
            is_longterm_memory: false,
            is_active: true,
            lifespan_current: 10,
            lifespan_initial: 10,
            lifespan_growth_rate: 5.0,
            creation_burst: 0,
            last_activation_burst: 0,
            activation_count: 1,
            spatial_signature: None,
            class_channels: Vec::new(),
            class_channel_weights: Vec::new(),
        }];
        let skipped = restored
            .restore_long_term_memory_neurons(&stm_only)
            .unwrap();
        assert_eq!(skipped, 0);
        assert!(restored.export_long_term_memory_neurons().is_empty());
    }

    #[test]
    fn spatial_signature_indexes_only_long_term_memory() {
        let mut array = MemoryNeuronArray::new(100);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 100,
            longterm_threshold: 100,
            ..Default::default()
        };
        let idx = array.create_memory_neuron(0x11, 3, 0, &config).unwrap();
        array.set_spatial_signature(idx, 0xABCD);
        assert!(array.find_ltm_by_spatial_signature(3, 0xABCD).is_empty());

        let converted = array.check_longterm_conversion(100);
        assert_eq!(converted, vec![idx]);
        assert_eq!(array.find_ltm_by_spatial_signature(3, 0xABCD), vec![idx]);

        array.bind_class_channel(idx, 2);
        array.bind_class_channel(idx, 2);
        assert_eq!(array.get_class_channels(idx), vec![2]);
    }

    // ---- id -> slot map consistency ----

    fn scan_find(array: &MemoryNeuronArray, id: u32) -> Option<usize> {
        (0..array.next_available_index).find(|&i| array.neuron_ids[i] == id && array.is_active[i])
    }

    /// For every id in `ids`, the map agrees with an active-only linear scan and
    /// `is_active_memory_neuron` matches `get_memory_neuron_detail(..).is_some()`.
    fn assert_map_consistent(array: &MemoryNeuronArray, ids: &[u32]) {
        for &id in ids {
            assert_eq!(
                array.find_neuron_index_by_global_id(id),
                scan_find(array, id),
                "id {id}"
            );
            assert_eq!(
                array.is_active_memory_neuron(id),
                array.get_memory_neuron_detail(id).is_some(),
                "id {id}"
            );
        }
        let active = array.is_active[..array.next_available_index]
            .iter()
            .filter(|&&a| a)
            .count();
        assert_eq!(array.neuron_id_to_index.len(), active);
    }

    fn id_of(array: &MemoryNeuronArray, idx: usize) -> u32 {
        array.neuron_ids[idx]
    }

    fn short_config(lifespan: u32) -> MemoryNeuronLifecycleConfig {
        MemoryNeuronLifecycleConfig {
            initial_lifespan: lifespan,
            lifespan_growth_rate: 1.0,
            longterm_threshold: 1000,
            max_reactivations: 1000,
            ..MemoryNeuronLifecycleConfig::default()
        }
    }

    #[test]
    fn id_map_tracks_create_expire_reuse_and_resets() {
        let mut array = MemoryNeuronArray::new(16);
        let cfg = short_config(1);
        let i0 = array.create_memory_neuron(1, 7, 1, &cfg).unwrap();
        let i1 = array.create_memory_neuron(2, 7, 1, &cfg).unwrap();
        let i2 = array.create_memory_neuron(3, 9, 1, &cfg).unwrap();
        let (a, b, c) = (id_of(&array, i0), id_of(&array, i1), id_of(&array, i2));
        assert_map_consistent(&array, &[a, b, c, 0, 9999]);
        assert!(array.is_active_memory_neuron(a));
        assert!(!array.is_active_memory_neuron(9999));

        // Expire all: map empties, ids are no longer active.
        array.age_memory_neurons(5);
        assert_map_consistent(&array, &[a, b, c]);
        assert!(!array.is_active_memory_neuron(a));

        // Reuse a freed slot with a new pattern: old id gone, new id found.
        let ni = array.create_memory_neuron(10, 7, 6, &cfg).unwrap();
        let n = id_of(&array, ni);
        assert!(ni < 3, "freed slot is reused");
        assert!(array.is_active_memory_neuron(n));
        assert_eq!(array.find_neuron_index_by_global_id(n), Some(ni));
        let old_in_slot = [a, b, c]
            .into_iter()
            .filter(|&x| x != n)
            .collect::<Vec<_>>();
        for id in &old_in_slot {
            assert!(!array.is_active_memory_neuron(*id));
        }
        assert_map_consistent(&array, &[a, b, c, n]);

        // Area reset.
        let j = array.create_memory_neuron(11, 9, 6, &cfg).unwrap();
        let m = id_of(&array, j);
        assert_map_consistent(&array, &[a, b, c, n, m]);
        array.reset_cortical_area(9);
        assert!(!array.is_active_memory_neuron(m));
        assert!(array.is_active_memory_neuron(n));
        assert_map_consistent(&array, &[a, b, c, n, m]);

        // Full reset.
        array.reset();
        assert!(array.neuron_id_to_index.is_empty());
        assert_map_consistent(&array, &[a, b, c, n, m]);
    }

    #[test]
    fn id_map_rebuilt_by_restore() {
        let mut array = MemoryNeuronArray::new(16);
        let cfg = short_config(10);
        let i0 = array.create_memory_neuron(1, 7, 1, &cfg).unwrap();
        let i1 = array.create_memory_neuron(2, 7, 1, &cfg).unwrap();
        let (a, b) = (id_of(&array, i0), id_of(&array, i1));
        array.is_longterm_memory[i0] = true;
        let exported = array.export_long_term_memory_neurons();
        assert_eq!(exported.len(), 1);

        let mut restored = MemoryNeuronArray::new(16);
        restored
            .restore_long_term_memory_neurons(&exported)
            .unwrap();
        assert!(restored.is_active_memory_neuron(a));
        assert!(!restored.is_active_memory_neuron(b));
        assert_map_consistent(&restored, &[a, b]);
    }

    // ---- Area cap, pool-full eviction, ordered indexes ----

    fn evict_cfg(
        lifespan: u32,
        threshold: u32,
        cap: u32,
        eviction: LtmEviction,
    ) -> MemoryNeuronLifecycleConfig {
        MemoryNeuronLifecycleConfig {
            initial_lifespan: lifespan,
            lifespan_growth_rate: 1.0,
            longterm_threshold: threshold,
            max_reactivations: 1000,
            memory_neuron_cap: cap,
            ltm_eviction: eviction,
        }
    }

    /// Everything the array keeps about `neuron_id` is gone.
    fn assert_fully_gone(array: &MemoryNeuronArray, neuron_id: u32, area: u32, pattern: u64) {
        assert!(!array.is_active_memory_neuron(neuron_id), "id map");
        assert!(array.get_memory_neuron_detail(neuron_id).is_none());
        assert!(
            array.find_neuron_by_pattern(area, &pattern).is_none(),
            "pattern map"
        );
        assert!(
            !array.get_active_neurons_by_area(area).contains(&neuron_id),
            "area index"
        );
        assert!(!array.neuron_id_to_index.contains_key(&neuron_id));
        assert!(!array.pattern_hash_to_index.contains_key(&(area, pattern)));
    }

    #[test]
    fn area_cap_evicts_least_recently_used_ltm() {
        let mut array = MemoryNeuronArray::new(64);
        // Lifespan equals the threshold, so a conversion pass makes every neuron LTM.
        let cfg = evict_cfg(5, 5, 3, LtmEviction::Lru);
        let mut ids = Vec::new();
        for (pattern, burst) in [(1u64, 1u64), (2, 2), (3, 3)] {
            let idx = array.create_memory_neuron(pattern, 7, burst, &cfg).unwrap();
            ids.push(array.neuron_ids[idx]);
        }
        assert_eq!(array.check_longterm_conversion(5).len(), 3);
        // Pattern 1 recurs, so pattern 2 is now the least recently used.
        let again = array.create_memory_neuron(1, 7, 4, &cfg).unwrap();
        assert_eq!(array.neuron_ids[again], ids[0]);
        assert!(array.drain_removed_neurons().is_empty());

        let new_idx = array.create_memory_neuron(4, 7, 5, &cfg).unwrap();
        assert_eq!(array.area_neuron_count(7), 3, "cap holds");
        assert_fully_gone(&array, ids[1], 7, 2);
        assert!(array.is_active_memory_neuron(ids[0]));
        assert!(array.is_active_memory_neuron(ids[2]));
        assert!(array.is_active_memory_neuron(array.neuron_ids[new_idx]));
        let removed = array.drain_removed_neurons();
        assert_eq!(
            removed,
            vec![RemovedMemoryNeuron {
                neuron_id: ids[1],
                area_idx: 7,
                was_longterm: true,
                reason: MemoryNeuronRemovalReason::Evicted,
                counted_in_stats: true,
            }]
        );
        assert_eq!(array.pool_in_use(), 3);
        assert_eq!(array.pool_ltm(), 2, "the new neuron is short-term");
    }

    #[test]
    fn area_cap_evicts_lowest_lifespan_short_term_before_any_long_term() {
        let mut array = MemoryNeuronArray::new(64);
        let cfg = evict_cfg(5, 6, 3, LtmEviction::Lru);
        let old_ltm_idx = array.create_memory_neuron(1, 7, 1, &cfg).unwrap();
        let old_ltm = array.neuron_ids[old_ltm_idx];
        array.convert_to_longterm(old_ltm_idx);
        let long_life = array.create_memory_neuron(2, 7, 2, &cfg).unwrap();
        let short_life = array.create_memory_neuron(3, 7, 2, &cfg).unwrap();
        let (long_id, short_id) = (array.neuron_ids[long_life], array.neuron_ids[short_life]);
        // Reactivate one STM neuron twice so its lifespan is higher than the other's.
        array.create_memory_neuron(2, 7, 3, &cfg).unwrap();
        array.create_memory_neuron(2, 7, 4, &cfg).unwrap();
        assert!(array.lifespan_current[long_life] > array.lifespan_current[short_life]);

        array.create_memory_neuron(9, 7, 5, &cfg).unwrap();
        assert_fully_gone(&array, short_id, 7, 3);
        assert!(
            array.is_active_memory_neuron(old_ltm),
            "LTM is spared while STM exists"
        );
        assert!(array.is_active_memory_neuron(long_id));
        let removed = array.drain_removed_neurons();
        assert_eq!(removed.len(), 1);
        assert!(!removed[0].was_longterm);
        assert_eq!(removed[0].reason, MemoryNeuronRemovalReason::Evicted);
    }

    #[test]
    fn pool_full_evicts_from_the_most_populated_lru_area() {
        let mut array = MemoryNeuronArray::new(4);
        let cfg = evict_cfg(5, 5, 0, LtmEviction::Lru);
        let mut first_area_ids = Vec::new();
        for (pattern, burst) in [(1u64, 1u64), (2, 2), (3, 3), (4, 4)] {
            let idx = array
                .create_memory_neuron(pattern, 10, burst, &cfg)
                .unwrap();
            first_area_ids.push(array.neuron_ids[idx]);
        }
        array.check_longterm_conversion(5);
        assert_eq!(array.pool_ltm(), 4);
        assert!(array.pool_full());

        // A second area succeeds by evicting the first area's least recently used LTM.
        let idx = array.create_memory_neuron(77, 20, 6, &cfg).unwrap();
        assert_eq!(array.cortical_area_ids[idx], 20);
        assert_fully_gone(&array, first_area_ids[0], 10, 1);
        assert_eq!(array.area_neuron_count(10), 3);
        assert_eq!(array.area_neuron_count(20), 1);
        assert_eq!(array.pool_in_use(), 4);
        let removed = array.drain_removed_neurons();
        assert_eq!(removed.len(), 1);
        assert_eq!((removed[0].area_idx, removed[0].was_longterm), (10, true));

        // Next time the first area is still the most populated, so it pays again.
        array.create_memory_neuron(78, 20, 7, &cfg).unwrap();
        assert_eq!(array.area_neuron_count(10), 2);
        assert_eq!(array.area_neuron_count(20), 2);
        // Equal counts: the lowest area idx is the victim area.
        array.create_memory_neuron(79, 20, 8, &cfg).unwrap();
        assert_eq!(array.area_neuron_count(10), 1);
        assert_eq!(array.area_neuron_count(20), 3);
    }

    #[test]
    fn pool_full_never_takes_from_a_none_area_and_none_creator_fails() {
        let mut array = MemoryNeuronArray::new(2);
        let lru = evict_cfg(5, 100, 0, LtmEviction::Lru);
        let keep = evict_cfg(5, 100, 0, LtmEviction::None);
        array.set_area_eviction(10, LtmEviction::None);
        array.create_memory_neuron(1, 10, 1, &keep).unwrap();
        array.create_memory_neuron(2, 10, 2, &keep).unwrap();

        // Another area cannot evict from the protected area.
        assert!(array.create_memory_neuron(3, 20, 3, &lru).is_none());
        assert_eq!(array.area_neuron_count(10), 2);
        assert!(array.drain_removed_neurons().is_empty());
        // The protected area itself fails too: none keeps today's behavior.
        assert!(array.create_memory_neuron(4, 10, 3, &keep).is_none());
        assert_eq!(array.pool_in_use(), 2);
    }

    #[test]
    fn area_cap_with_none_keeps_the_failure() {
        let mut array = MemoryNeuronArray::new(16);
        let cfg = evict_cfg(5, 100, 2, LtmEviction::None);
        assert!(array.create_memory_neuron(1, 7, 1, &cfg).is_some());
        assert!(array.create_memory_neuron(2, 7, 2, &cfg).is_some());
        assert!(array.create_memory_neuron(3, 7, 3, &cfg).is_none());
        assert_eq!(array.area_neuron_count(7), 2);
        // A recurring pattern still reactivates at the cap.
        assert!(array.create_memory_neuron(1, 7, 4, &cfg).is_some());
        // A different area has its own count.
        assert!(array.create_memory_neuron(3, 8, 3, &cfg).is_some());
    }

    #[test]
    fn neurons_touched_this_burst_are_never_victims() {
        let mut array = MemoryNeuronArray::new(16);
        let cfg = evict_cfg(5, 100, 1, LtmEviction::Lru);
        assert!(array.create_memory_neuron(1, 7, 3, &cfg).is_some());
        // Same burst, new pattern, cap 1: the only neuron was created this burst.
        assert!(array.create_memory_neuron(2, 7, 3, &cfg).is_none());
        assert!(array.create_memory_neuron(2, 7, 4, &cfg).is_some());
    }

    #[test]
    fn lowering_the_cap_evicts_one_neuron_per_creation() {
        let mut array = MemoryNeuronArray::new(16);
        let wide = evict_cfg(50, 1000, 0, LtmEviction::Lru);
        for pattern in 1..=5u64 {
            array
                .create_memory_neuron(pattern, 7, pattern, &wide)
                .unwrap();
        }
        let narrow = evict_cfg(50, 1000, 2, LtmEviction::Lru);
        assert_eq!(array.area_neuron_count(7), 5, "no mass eviction");
        array.create_memory_neuron(6, 7, 6, &narrow).unwrap();
        assert_eq!(array.area_neuron_count(7), 5, "one out, one in");
        array.create_memory_neuron(7, 7, 7, &narrow).unwrap();
        assert_eq!(array.area_neuron_count(7), 5);
        assert_eq!(array.drain_removed_neurons().len(), 2);
    }

    #[test]
    fn expiry_is_recorded_as_a_removal() {
        let mut array = MemoryNeuronArray::new(16);
        let cfg = evict_cfg(1, 100, 0, LtmEviction::Lru);
        let idx = array.create_memory_neuron(1, 7, 1, &cfg).unwrap();
        let id = array.neuron_ids[idx];
        array.age_memory_neurons(1);
        assert!(
            array.drain_removed_neurons().is_empty(),
            "matched this burst"
        );
        array.age_memory_neurons(2);
        assert_eq!(
            array.drain_removed_neurons(),
            vec![RemovedMemoryNeuron {
                neuron_id: id,
                area_idx: 7,
                was_longterm: false,
                reason: MemoryNeuronRemovalReason::Expired,
                counted_in_stats: true,
            }]
        );
        assert_eq!(array.pool_in_use(), 0);
    }

    #[test]
    fn id_allocation_failure_returns_the_slot() {
        let mut array = MemoryNeuronArray::new(4);
        // Reserving the last id of the range leaves nothing to allocate.
        assert!(array
            .id_manager
            .reserve_memory_neuron_id(crate::neuron_id_manager::MEMORY_NEURON_ID_MAX));
        let cfg = evict_cfg(5, 100, 0, LtmEviction::Lru);
        assert!(array.create_memory_neuron(1, 7, 1, &cfg).is_none());
        assert_eq!(
            array.reusable_indices.len(),
            1,
            "slot is returned, not leaked"
        );
        assert_eq!(array.pool_in_use(), 0);
        assert!(array.create_memory_neuron(2, 7, 2, &cfg).is_none());
        assert_eq!(array.reusable_indices.len(), 1);
    }

    #[test]
    fn restore_keeps_the_most_recent_records_when_over_capacity() {
        let detail = |id: u32, area: u32, last: u64| MemoryNeuronDetail {
            neuron_id: crate::neuron_id_manager::MEMORY_NEURON_ID_START + id,
            cortical_area_idx: area,
            pattern_hash: Some(u64::from(id)),
            is_longterm_memory: true,
            is_active: true,
            lifespan_current: 100,
            lifespan_initial: 20,
            lifespan_growth_rate: 3.0,
            creation_burst: 0,
            last_activation_burst: last,
            activation_count: 1,
            spatial_signature: None,
            class_channels: Vec::new(),
            class_channel_weights: Vec::new(),
        };
        // Pool of 3, five records: the three most recent survive and the call is Ok.
        let mut array = MemoryNeuronArray::new(3);
        let records: Vec<_> = (0..5).map(|i| detail(i, 7, u64::from(i) * 10)).collect();
        assert_eq!(array.restore_long_term_memory_neurons(&records), Ok(3));
        assert_eq!(array.pool_in_use(), 3);
        for kept in [2u32, 3, 4] {
            assert!(array
                .is_active_memory_neuron(crate::neuron_id_manager::MEMORY_NEURON_ID_START + kept));
        }
        for dropped in [0u32, 1] {
            assert!(!array.is_active_memory_neuron(
                crate::neuron_id_manager::MEMORY_NEURON_ID_START + dropped
            ));
        }

        // Area cap of 2 in a roomy pool: keep the two most recent of that area only.
        let mut array = MemoryNeuronArray::new(16);
        array.set_area_cap(7, 2);
        let mut records: Vec<_> = (0..4).map(|i| detail(i, 7, u64::from(i) * 10)).collect();
        records.push(detail(10, 8, 1));
        assert_eq!(array.restore_long_term_memory_neurons(&records), Ok(3));
        assert_eq!(array.area_neuron_count(7), 2);
        assert_eq!(array.area_neuron_count(8), 1);
        assert!(array.is_active_memory_neuron(crate::neuron_id_manager::MEMORY_NEURON_ID_START + 3));
        assert!(array.is_active_memory_neuron(crate::neuron_id_manager::MEMORY_NEURON_ID_START + 2));
        assert!(!array.is_active_memory_neuron(crate::neuron_id_manager::MEMORY_NEURON_ID_START));
    }

    #[test]
    fn restore_reserves_dropped_ids_queues_their_removal_and_skips_bad_records() {
        use crate::neuron_id_manager::{MEMORY_NEURON_ID_MAX, MEMORY_NEURON_ID_START as START};
        let detail = |id: u32, area: u32, last: u64| MemoryNeuronDetail {
            neuron_id: id,
            cortical_area_idx: area,
            pattern_hash: Some(u64::from(id)),
            is_longterm_memory: true,
            is_active: true,
            lifespan_current: 100,
            lifespan_initial: 20,
            lifespan_growth_rate: 3.0,
            creation_burst: 0,
            last_activation_burst: last,
            activation_count: 1,
            spatial_signature: None,
            class_channels: Vec::new(),
            class_channel_weights: Vec::new(),
        };
        // Pool of 2. The oldest record has the highest id, so it is dropped while its id is
        // above every kept id. A duplicate and an out-of-range record must not fail the call.
        let mut array = MemoryNeuronArray::new(2);
        let records = vec![
            detail(START + 1, 7, 30),
            detail(START + 2, 7, 20),
            detail(START + 50, 7, 10),
            detail(START + 2, 7, 5),
            detail(42, 7, 40),
        ];
        assert_eq!(array.restore_long_term_memory_neurons(&records), Ok(2));
        assert!(array.is_active_memory_neuron(START + 1));
        assert!(array.is_active_memory_neuron(START + 2));
        let removed = array.drain_removed_neurons();
        assert!(removed.iter().any(|r| r.neuron_id == START + 50
            && r.area_idx == 7
            && r.reason == MemoryNeuronRemovalReason::Evicted
            && !r.counted_in_stats));

        // Fresh ids start past every offered id, including the dropped one.
        let cfg = evict_cfg(5, 100, 0, LtmEviction::Lru);
        let idx = array.create_memory_neuron(9, 8, 50, &cfg).unwrap();
        assert!(
            array.neuron_ids[idx] > START + 50,
            "{}",
            array.neuron_ids[idx]
        );
        let _ = MEMORY_NEURON_ID_MAX;
    }

    /// Brute-force victim for one area, mirroring the documented rule.
    fn brute_victim(array: &MemoryNeuronArray, area: u32, burst: u64) -> Option<usize> {
        let live = |i: usize| {
            array.is_active[i]
                && array.cortical_area_ids[i] == area
                && array.last_activation_burst[i] != burst
        };
        (0..array.next_available_index)
            .filter(|&i| live(i) && !array.is_longterm_memory[i])
            .min_by_key(|&i| (array.lifespan_current[i], i))
            .or_else(|| {
                (0..array.next_available_index)
                    .filter(|&i| live(i) && array.is_longterm_memory[i])
                    .min_by_key(|&i| (array.last_activation_burst[i], i))
            })
    }

    #[test]
    fn ordered_indexes_match_a_brute_force_scan_under_random_use() {
        let mut array = MemoryNeuronArray::new(40);
        let mut state = 0x2545_F491_4F6C_DD1Du64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for burst in 1..400u64 {
            for _ in 0..(next() % 3) {
                let area = (next() % 3) as u32 + 1;
                let pattern = next() % 12;
                let cfg = evict_cfg(
                    (next() % 6) as u32 + 1,
                    8,
                    if area == 2 { 4 } else { 0 },
                    LtmEviction::Lru,
                );
                array.create_memory_neuron(pattern, area, burst, &cfg);
            }
            array.check_longterm_conversion(8);
            array.age_memory_neurons(burst);
            array.drain_removed_neurons();

            let active = array.is_active[..array.next_available_index]
                .iter()
                .filter(|&&a| a)
                .count();
            assert_eq!(array.pool_in_use(), active, "burst {burst}");
            let ltm = (0..array.next_available_index)
                .filter(|&i| array.is_active[i] && array.is_longterm_memory[i])
                .count();
            assert_eq!(array.pool_ltm(), ltm, "burst {burst}");
            for area in 1..=3u32 {
                for probe_burst in [burst, burst + 1] {
                    assert_eq!(
                        array.pick_victim(area, probe_burst),
                        brute_victim(&array, area, probe_burst),
                        "burst {burst} area {area} probe {probe_burst}"
                    );
                }
            }
        }
    }
}
