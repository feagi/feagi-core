// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
ConnectomeManager - Core brain connectivity manager.

This is the central orchestrator for the FEAGI connectome, managing:
- Cortical areas and their metadata
- Brain regions and hierarchy
- Neuron/synapse queries (delegates to NPU for actual data)
- Genome loading and persistence

## Architecture

The ConnectomeManager is a **metadata manager** that:
1. Stores cortical area/region definitions
2. Provides a high-level API for brain structure queries
3. Delegates neuron/synapse CRUD to the NPU (Structure of Arrays)

## Design Principles

- **Singleton**: One global instance per FEAGI process
- **Thread-safe**: Uses RwLock for concurrent reads
- **Performance**: Optimized for hot-path queries (area lookups)
- **NPU Delegation**: Neuron/synapse data lives in NPU, not here

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, trace, warn};
use xxhash_rust::xxh64::Xxh64;

type BrainRegionIoRegistry = HashMap<String, (Vec<String>, Vec<String>)>;

use crate::models::{BrainRegion, BrainRegionHierarchy, CorticalArea, CorticalAreaDimensions};
use crate::types::{BduError, BduResult};
use feagi_npu_neural::types::NeuronId;
use feagi_structures::genomic::cortical_area::CorticalID;

// State manager access for fatigue calculation
// Note: feagi-state-manager is always available when std is enabled (it's a default feature)
use feagi_state_manager::StateManager;

const DATA_HASH_SEED: u64 = 0;

// NPU integration (optional dependency)
// use feagi_npu_burst_engine::RustNPU; // Now using DynamicNPU

/// Global singleton instance of ConnectomeManager
static INSTANCE: Lazy<Arc<RwLock<ConnectomeManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(ConnectomeManager::new())));

/// Configuration for ConnectomeManager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectomeConfig {
    /// Maximum number of neurons (for NPU sizing)
    pub max_neurons: usize,

    /// Maximum number of synapses (for NPU sizing)
    pub max_synapses: usize,

    /// Backend type ("cpu", "cuda", "wgpu")
    pub backend: String,
}

impl Default for ConnectomeConfig {
    fn default() -> Self {
        Self {
            max_neurons: 10_000_000,
            max_synapses: 100_000_000,
            backend: "cpu".to_string(),
        }
    }
}

/// Central manager for the FEAGI connectome
///
/// ## Responsibilities
///
/// 1. **Cortical Area Management**: Add, remove, query cortical areas
/// 2. **Brain Region Management**: Hierarchical organization
/// 3. **Neuron/Synapse Queries**: High-level API (delegates to NPU)
/// 4. **Genome I/O**: Load/save brain structure
///
/// ## Data Storage
///
/// - **Cortical areas**: Stored in HashMap for O(1) lookup
/// - **Brain regions**: Stored in BrainRegionHierarchy
/// - **Neuron data**: Lives in NPU (not stored here)
/// - **Synapse data**: Lives in NPU (not stored here)
///
/// ## Thread Safety
///
/// Uses `RwLock` for concurrent reads with exclusive writes.
/// Multiple threads can read simultaneously, but writes block.
///
pub struct ConnectomeManager {
    /// Map of cortical_id -> CorticalArea metadata
    cortical_areas: HashMap<CorticalID, CorticalArea>,

    /// Map of cortical_id -> cortical_idx (fast reverse lookup)
    cortical_id_to_idx: HashMap<CorticalID, u32>,

    /// Map of cortical_idx -> cortical_id (fast reverse lookup)
    cortical_idx_to_id: HashMap<u32, CorticalID>,

    /// Next available cortical index
    next_cortical_idx: u32,

    /// Brain region hierarchy
    brain_regions: BrainRegionHierarchy,

    /// Morphology registry from loaded genome
    morphology_registry: feagi_evolutionary::MorphologyRegistry,

    /// Configuration
    config: ConnectomeConfig,

    /// Optional reference to the Rust NPU for neuron/synapse queries
    ///
    /// This is set by the Python process manager after NPU initialization.
    /// All neuron/synapse data queries delegate to the NPU.
    /// Wrapped in TracingMutex to automatically log all lock acquisitions
    npu: Option<Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>>,

    /// Plasticity executor reference (optional, only when plasticity feature is enabled)
    #[cfg(feature = "plasticity")]
    plasticity_executor:
        Option<Arc<std::sync::Mutex<feagi_npu_plasticity::AsyncPlasticityExecutor>>>,

    /// Cached neuron count (lock-free read) - updated by burst engine
    /// This prevents health checks from blocking on NPU lock
    cached_neuron_count: Arc<AtomicUsize>,

    /// Cached synapse count (lock-free read) - updated by burst engine
    /// This prevents health checks from blocking on NPU lock
    cached_synapse_count: Arc<AtomicUsize>,

    /// Per-area neuron count cache (lock-free reads) - updated when neurons are created/deleted
    /// This prevents health checks from blocking on NPU lock
    cached_neuron_counts_per_area: Arc<RwLock<HashMap<CorticalID, AtomicUsize>>>,

    /// Per-area synapse count cache (lock-free reads) - updated when synapses are created/deleted
    /// This prevents health checks from blocking on NPU lock
    cached_synapse_counts_per_area: Arc<RwLock<HashMap<CorticalID, AtomicUsize>>>,

    /// Is the connectome initialized (has cortical areas)?
    initialized: bool,

    /// Last fatigue index calculation time (for rate limiting)
    last_fatigue_calculation: Arc<Mutex<std::time::Instant>>,
}

/// Type alias for neuron batch data: (x, y, z, threshold, threshold_limit, leak, resting, neuron_type, refractory_period, excitability, consecutive_fire_limit, snooze_period, mp_charge_accumulation)
type NeuronData = (
    u32,
    u32,
    u32,
    f32,
    f32,
    f32,
    f32,
    i32,
    u16,
    f32,
    u16,
    u16,
    bool,
);

impl ConnectomeManager {
    /// Create a new ConnectomeManager (private - use `instance()`)
    fn new() -> Self {
        Self {
            cortical_areas: HashMap::new(),
            cortical_id_to_idx: HashMap::new(),
            cortical_idx_to_id: HashMap::new(),
            // CRITICAL: Reserve indices 0 (_death) and 1 (_power) - start regular areas at 2
            next_cortical_idx: 3, // Reserve 0=_death, 1=_power, 2=_fatigue
            brain_regions: BrainRegionHierarchy::new(),
            morphology_registry: feagi_evolutionary::MorphologyRegistry::new(),
            config: ConnectomeConfig::default(),
            npu: None,
            #[cfg(feature = "plasticity")]
            plasticity_executor: None,
            cached_neuron_count: Arc::new(AtomicUsize::new(0)),
            cached_synapse_count: Arc::new(AtomicUsize::new(0)),
            cached_neuron_counts_per_area: Arc::new(RwLock::new(HashMap::new())),
            cached_synapse_counts_per_area: Arc::new(RwLock::new(HashMap::new())),
            initialized: false,
            last_fatigue_calculation: Arc::new(Mutex::new(
                std::time::Instant::now() - std::time::Duration::from_secs(10),
            )), // Initialize to allow first calculation
        }
    }

    /// Get the global singleton instance
    ///
    /// # Returns
    ///
    /// Arc to the ConnectomeManager wrapped in RwLock
    ///
    /// # Example
    ///
    /// ```ignore
    /// use feagi_brain_development::ConnectomeManager;
    ///
    /// let manager = ConnectomeManager::instance();
    /// let read_lock = manager.read();
    /// let area_count = read_lock.get_cortical_area_count();
    /// ```
    ///
    pub fn instance() -> Arc<RwLock<ConnectomeManager>> {
        // Note: Singleton is always f32 for backward compatibility
        // New code should use ConnectomeManager::<T>::new_for_testing_with_npu() for custom types
        Arc::clone(&*INSTANCE)
    }

    /// Calculate optimal visualization voxel granularity for a cortical area
    ///
    /// This function determines the granularity for aggregated rendering based on:
    /// - Total voxel count (larger areas get larger chunks)
    /// - Aspect ratio (handles thin dimensions like 1024×900×3)
    /// - Target chunk count (~2k-10k chunks for manageable message size)
    ///
    /// # Arguments
    ///
    /// * `dimensions` - The cortical area dimensions (width, height, depth)
    ///
    /// # Returns
    ///
    /// Tuple of (chunk_x, chunk_y, chunk_z) that divides evenly into dimensions
    ///
    ///
    /// Create a new isolated instance for testing
    ///
    /// This bypasses the singleton pattern and creates a fresh instance.
    /// Use this in tests to avoid conflicts between parallel test runs.
    ///
    /// # Example
    ///
    /// ```rust
    /// let manager = ConnectomeManager::new_for_testing();
    /// // Use manager in isolated test
    /// ```
    pub fn new_for_testing() -> Self {
        Self {
            cortical_areas: HashMap::new(),
            cortical_id_to_idx: HashMap::new(),
            cortical_idx_to_id: HashMap::new(),
            next_cortical_idx: 0,
            brain_regions: BrainRegionHierarchy::new(),
            morphology_registry: feagi_evolutionary::MorphologyRegistry::new(),
            config: ConnectomeConfig::default(),
            npu: None,
            #[cfg(feature = "plasticity")]
            plasticity_executor: None,
            cached_neuron_count: Arc::new(AtomicUsize::new(0)),
            cached_synapse_count: Arc::new(AtomicUsize::new(0)),
            cached_neuron_counts_per_area: Arc::new(RwLock::new(HashMap::new())),
            cached_synapse_counts_per_area: Arc::new(RwLock::new(HashMap::new())),
            initialized: false,
            last_fatigue_calculation: Arc::new(Mutex::new(
                std::time::Instant::now() - std::time::Duration::from_secs(10),
            )),
        }
    }

    /// Create a new isolated instance for testing with NPU
    ///
    /// This bypasses the singleton pattern and creates a fresh instance with NPU connected.
    /// Use this in tests to avoid conflicts between parallel test runs.
    ///
    /// # Arguments
    ///
    /// * `npu` - Arc<TracingMutex<DynamicNPU>> to connect to this manager
    ///
    /// # Example
    ///
    /// ```rust
    /// let npu = Arc::new(TracingMutex::new(RustNPU::new(1_000_000, 10_000_000, 10), "NPU"));
    /// let manager = ConnectomeManager::new_for_testing_with_npu(npu);
    /// ```
    pub fn new_for_testing_with_npu(
        npu: Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
    ) -> Self {
        Self {
            cortical_areas: HashMap::new(),
            cortical_id_to_idx: HashMap::new(),
            cortical_idx_to_id: HashMap::new(),
            next_cortical_idx: 0,
            brain_regions: BrainRegionHierarchy::new(),
            morphology_registry: feagi_evolutionary::MorphologyRegistry::new(),
            config: ConnectomeConfig::default(),
            npu: Some(npu),
            #[cfg(feature = "plasticity")]
            plasticity_executor: None,
            cached_neuron_count: Arc::new(AtomicUsize::new(0)),
            cached_synapse_count: Arc::new(AtomicUsize::new(0)),
            cached_neuron_counts_per_area: Arc::new(RwLock::new(HashMap::new())),
            cached_synapse_counts_per_area: Arc::new(RwLock::new(HashMap::new())),
            initialized: false,
            last_fatigue_calculation: Arc::new(Mutex::new(
                std::time::Instant::now() - std::time::Duration::from_secs(10),
            )),
        }
    }

    /// Set up core morphologies in the registry (for testing only)
    ///
    /// This is a test helper to set up core morphologies (projector, block_to_block, etc.)
    /// in the morphology registry so that synaptogenesis tests can run.
    ///
    /// # Note
    ///
    /// This should only be called in tests. Morphologies are typically loaded from genome files.
    /// This method is public to allow integration tests to access it.
    pub fn setup_core_morphologies_for_testing(&mut self) {
        feagi_evolutionary::add_core_morphologies(&mut self.morphology_registry);
    }

    /// Reset the singleton (for testing only)
    ///
    /// # Safety
    ///
    /// This should only be called in tests to reset state between test runs.
    /// Calling this in production code will cause all references to the old
    /// instance to become stale.
    ///
    #[cfg(test)]
    pub fn reset_for_testing() {
        let mut instance = INSTANCE.write();
        *instance = Self::new();
    }

    // ======================================================================
    // Data Hashing (event-driven updates for health_check)
    // ======================================================================

    /// Update stored hashes for data types that have changed.
    fn update_state_hashes(
        &self,
        brain_regions: Option<u64>,
        cortical_areas: Option<u64>,
        brain_geometry: Option<u64>,
        morphologies: Option<u64>,
        cortical_mappings: Option<u64>,
    ) {
        if let Some(state_manager) = StateManager::instance().try_read() {
            if let Some(value) = brain_regions {
                state_manager.set_brain_regions_hash(value);
            }
            if let Some(value) = cortical_areas {
                state_manager.set_cortical_areas_hash(value);
            }
            if let Some(value) = brain_geometry {
                state_manager.set_brain_geometry_hash(value);
            }
            if let Some(value) = morphologies {
                state_manager.set_morphologies_hash(value);
            }
            if let Some(value) = cortical_mappings {
                state_manager.set_cortical_mappings_hash(value);
            }
        }
    }

    /// Refresh the brain regions hash (hierarchy, membership, and properties).
    fn refresh_brain_regions_hash(&self) {
        let hash = self.compute_brain_regions_hash();
        self.update_state_hashes(Some(hash), None, None, None, None);
    }

    /// Refresh the cortical areas hash (metadata and properties).
    fn refresh_cortical_areas_hash(&self) {
        let hash = self.compute_cortical_areas_hash();
        self.update_state_hashes(None, Some(hash), None, None, None);
    }

    /// Refresh the brain geometry hash (positions, dimensions, 2D coordinates).
    fn refresh_brain_geometry_hash(&self) {
        let hash = self.compute_brain_geometry_hash();
        self.update_state_hashes(None, None, Some(hash), None, None);
    }

    /// Refresh the morphologies hash.
    fn refresh_morphologies_hash(&self) {
        let hash = self.compute_morphologies_hash();
        self.update_state_hashes(None, None, None, Some(hash), None);
    }

    /// Refresh the cortical mappings hash.
    fn refresh_cortical_mappings_hash(&self) {
        let hash = self.compute_cortical_mappings_hash();
        self.update_state_hashes(None, None, None, None, Some(hash));
    }

    /// Refresh cortical area-related hashes based on the affected data.
    pub fn refresh_cortical_area_hashes(
        &self,
        properties_changed: bool,
        geometry_changed: bool,
    ) {
        let cortical_hash = if properties_changed {
            Some(self.compute_cortical_areas_hash())
        } else {
            None
        };
        let geometry_hash = if geometry_changed {
            Some(self.compute_brain_geometry_hash())
        } else {
            None
        };
        self.update_state_hashes(None, cortical_hash, geometry_hash, None, None);
    }

    /// Compute hash for brain regions (hierarchy, membership, and properties).
    fn compute_brain_regions_hash(&self) -> u64 {
        let mut hasher = Xxh64::new(DATA_HASH_SEED);
        let mut region_ids: Vec<String> = self
            .brain_regions
            .get_all_region_ids()
            .into_iter()
            .cloned()
            .collect();
        region_ids.sort();

        for region_id in region_ids {
            let Some(region) = self.brain_regions.get_region(&region_id) else {
                continue;
            };
            Self::hash_str(&mut hasher, &region_id);
            Self::hash_str(&mut hasher, &region.name);
            Self::hash_str(&mut hasher, &region.region_type.to_string());
            let parent_id = self.brain_regions.get_parent(&region_id);
            match parent_id {
                Some(parent) => Self::hash_str(&mut hasher, parent),
                None => Self::hash_str(&mut hasher, "null"),
            }

            let mut cortical_ids: Vec<String> = region
                .cortical_areas
                .iter()
                .map(|id| id.as_base_64())
                .collect();
            cortical_ids.sort();
            for cortical_id in cortical_ids {
                Self::hash_str(&mut hasher, &cortical_id);
            }

            Self::hash_properties_filtered(&mut hasher, &region.properties, &[]);
        }

        hasher.finish()
    }

    /// Compute hash for cortical areas and properties (excluding mappings).
    fn compute_cortical_areas_hash(&self) -> u64 {
        let mut hasher = Xxh64::new(DATA_HASH_SEED);
        let mut areas: Vec<&CorticalArea> = self.cortical_areas.values().collect();
        areas.sort_by_key(|area| area.cortical_id.as_base_64());

        for area in areas {
            let cortical_id = area.cortical_id.as_base_64();
            Self::hash_str(&mut hasher, &cortical_id);
            hasher.write_u32(area.cortical_idx);
            Self::hash_str(&mut hasher, &area.name);
            Self::hash_str(&mut hasher, &area.cortical_type.to_string());

            let excluded = ["cortical_mapping_dst", "upstream_cortical_areas"];
            Self::hash_properties_filtered(&mut hasher, &area.properties, &excluded);
        }

        hasher.finish()
    }

    /// Compute hash for brain geometry (area positions, dimensions, and 2D coordinates).
    fn compute_brain_geometry_hash(&self) -> u64 {
        let mut hasher = Xxh64::new(DATA_HASH_SEED);
        let mut areas: Vec<&CorticalArea> = self.cortical_areas.values().collect();
        areas.sort_by_key(|area| area.cortical_id.as_base_64());

        for area in areas {
            let cortical_id = area.cortical_id.as_base_64();
            Self::hash_str(&mut hasher, &cortical_id);

            Self::hash_i32(&mut hasher, area.position.x);
            Self::hash_i32(&mut hasher, area.position.y);
            Self::hash_i32(&mut hasher, area.position.z);

            Self::hash_u32(&mut hasher, area.dimensions.width);
            Self::hash_u32(&mut hasher, area.dimensions.height);
            Self::hash_u32(&mut hasher, area.dimensions.depth);

            let coord_2d = area
                .properties
                .get("coordinate_2d")
                .or_else(|| area.properties.get("coordinates_2d"));
            match coord_2d {
                Some(value) => Self::hash_json_value(&mut hasher, value),
                None => Self::hash_str(&mut hasher, "null"),
            }
        }

        hasher.finish()
    }

    /// Compute hash for morphologies.
    fn compute_morphologies_hash(&self) -> u64 {
        let mut hasher = Xxh64::new(DATA_HASH_SEED);
        let mut morphology_ids = self.morphology_registry.morphology_ids();
        morphology_ids.sort();

        for morphology_id in morphology_ids {
            if let Some(morphology) = self.morphology_registry.get(&morphology_id) {
                Self::hash_str(&mut hasher, &morphology_id);
                Self::hash_str(&mut hasher, &format!("{:?}", morphology.morphology_type));
                Self::hash_str(&mut hasher, &morphology.class);
                if let Ok(value) = serde_json::to_value(&morphology.parameters) {
                    Self::hash_json_value(&mut hasher, &value);
                }
            }
        }

        hasher.finish()
    }

    /// Compute hash for cortical mappings (cortical_mapping_dst).
    fn compute_cortical_mappings_hash(&self) -> u64 {
        let mut hasher = Xxh64::new(DATA_HASH_SEED);
        let mut areas: Vec<&CorticalArea> = self.cortical_areas.values().collect();
        areas.sort_by_key(|area| area.cortical_id.as_base_64());

        for area in areas {
            let cortical_id = area.cortical_id.as_base_64();
            Self::hash_str(&mut hasher, &cortical_id);
            if let Some(serde_json::Value::Object(map)) =
                area.properties.get("cortical_mapping_dst")
            {
                let mut dest_ids: Vec<&String> = map.keys().collect();
                dest_ids.sort();
                for dest_id in dest_ids {
                    Self::hash_str(&mut hasher, dest_id);
                    if let Some(value) = map.get(dest_id) {
                        Self::hash_json_value(&mut hasher, value);
                    }
                }
            } else {
                Self::hash_str(&mut hasher, "null");
            }
        }

        hasher.finish()
    }

    /// Hash a string with a separator to avoid concatenation collisions.
    fn hash_str(hasher: &mut Xxh64, value: &str) {
        hasher.write(value.as_bytes());
        hasher.write_u8(0);
    }

    /// Hash a signed 32-bit integer deterministically.
    fn hash_i32(hasher: &mut Xxh64, value: i32) {
        hasher.write(&value.to_le_bytes());
    }

    /// Hash an unsigned 32-bit integer deterministically.
    fn hash_u32(hasher: &mut Xxh64, value: u32) {
        hasher.write(&value.to_le_bytes());
    }

    /// Hash JSON values deterministically with sorted object keys.
    fn hash_json_value(hasher: &mut Xxh64, value: &serde_json::Value) {
        match value {
            serde_json::Value::Null => {
                hasher.write_u8(0);
            }
            serde_json::Value::Bool(val) => {
                hasher.write_u8(1);
                hasher.write_u8(*val as u8);
            }
            serde_json::Value::Number(num) => {
                hasher.write_u8(2);
                Self::hash_str(hasher, &num.to_string());
            }
            serde_json::Value::String(val) => {
                hasher.write_u8(3);
                Self::hash_str(hasher, val);
            }
            serde_json::Value::Array(items) => {
                hasher.write_u8(4);
                for item in items {
                    Self::hash_json_value(hasher, item);
                }
            }
            serde_json::Value::Object(map) => {
                hasher.write_u8(5);
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                for key in keys {
                    Self::hash_str(hasher, key);
                    if let Some(val) = map.get(key) {
                        Self::hash_json_value(hasher, val);
                    }
                }
            }
        }
    }

    /// Hash JSON properties deterministically, excluding specific keys.
    fn hash_properties_filtered(
        hasher: &mut Xxh64,
        properties: &HashMap<String, serde_json::Value>,
        excluded_keys: &[&str],
    ) {
        let mut keys: Vec<&String> = properties.keys().collect();
        keys.sort();
        for key in keys {
            if excluded_keys.contains(&key.as_str()) {
                continue;
            }
            Self::hash_str(hasher, key);
            if let Some(value) = properties.get(key) {
                Self::hash_json_value(hasher, value);
            }
        }
    }

    // ======================================================================
    // Cortical Area Management
    // ======================================================================

    /// Add a new cortical area
    ///
    /// # Arguments
    ///
    /// * `area` - The cortical area to add
    ///
    /// # Returns
    ///
    /// The assigned cortical index
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - An area with the same cortical_id already exists
    /// - The area's cortical_idx conflicts with an existing area
    ///
    pub fn add_cortical_area(&mut self, mut area: CorticalArea) -> BduResult<u32> {
        // Check if area already exists
        if self.cortical_areas.contains_key(&area.cortical_id) {
            return Err(BduError::InvalidArea(format!(
                "Cortical area {} already exists",
                area.cortical_id
            )));
        }

        // CRITICAL: Reserve cortical_idx 0 for _death, 1 for _power, 2 for _fatigue
        // Use feagi-data-processing types as single source of truth
        use feagi_structures::genomic::cortical_area::CoreCorticalType;

        let death_id = CoreCorticalType::Death.to_cortical_id();
        let power_id = CoreCorticalType::Power.to_cortical_id();
        let fatigue_id = CoreCorticalType::Fatigue.to_cortical_id();

        let is_death_area = area.cortical_id == death_id;
        let is_power_area = area.cortical_id == power_id;
        let is_fatigue_area = area.cortical_id == fatigue_id;

        if is_death_area {
            trace!(
                target: "feagi-bdu",
                "[CORE-AREA] Assigning RESERVED cortical_idx=0 to _death area (id={})",
                area.cortical_id
            );
            area.cortical_idx = 0;
        } else if is_power_area {
            trace!(
                target: "feagi-bdu",
                "[CORE-AREA] Assigning RESERVED cortical_idx=1 to _power area (id={})",
                area.cortical_id
            );
            area.cortical_idx = 1;
        } else if is_fatigue_area {
            trace!(
                target: "feagi-bdu",
                "[CORE-AREA] Assigning RESERVED cortical_idx=2 to _fatigue area (id={})",
                area.cortical_id
            );
            area.cortical_idx = 2;
        } else {
            // Regular areas: assign cortical_idx if not set (will be ≥3 due to next_cortical_idx=3 initialization)
            if area.cortical_idx == 0 {
                area.cortical_idx = self.next_cortical_idx;
                self.next_cortical_idx += 1;
                trace!(
                    target: "feagi-bdu",
                    "[REGULAR-AREA] Assigned cortical_idx={} to area '{}' (should be ≥3)",
                    area.cortical_idx,
                    area.cortical_id.as_base_64()
                );
            } else {
                // Check for reserved index collision
                if area.cortical_idx == 0 || area.cortical_idx == 1 || area.cortical_idx == 2 {
                    warn!(
                        "Regular area '{}' attempted to use RESERVED cortical_idx={}! Reassigning to next available.",
                        area.cortical_id, area.cortical_idx);
                    area.cortical_idx = self.next_cortical_idx;
                    self.next_cortical_idx += 1;
                    info!(
                        "   Reassigned '{}' to cortical_idx={}",
                        area.cortical_id, area.cortical_idx
                    );
                } else if self.cortical_idx_to_id.contains_key(&area.cortical_idx) {
                    return Err(BduError::InvalidArea(format!(
                        "Cortical index {} is already in use",
                        area.cortical_idx
                    )));
                }

                // Update next_cortical_idx if needed
                if area.cortical_idx >= self.next_cortical_idx {
                    self.next_cortical_idx = area.cortical_idx + 1;
                }
            }
        }

        let cortical_id = area.cortical_id;
        let cortical_idx = area.cortical_idx;

        // Update lookup maps
        self.cortical_id_to_idx.insert(cortical_id, cortical_idx);
        self.cortical_idx_to_id.insert(cortical_idx, cortical_id);

        // Initialize upstream_cortical_areas property (empty array for O(1) lookup)
        area.properties
            .insert("upstream_cortical_areas".to_string(), serde_json::json!([]));

        // Default visualization voxel granularity is 1x1x1 (assumed, not stored)
        // User overrides are stored in properties["visualization_voxel_granularity"] only if != 1x1x1

        // If the caller provided a parent brain region ID, persist the association in the
        // BrainRegionHierarchy membership set (this drives /v1/region/regions_members).
        //
        // IMPORTANT: This is separate from storing "parent_region_id" in the cortical area's
        // properties. BV may show that property even if the hierarchy isn't updated.
        let parent_region_id = area
            .properties
            .get("parent_region_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Store area
        self.cortical_areas.insert(cortical_id, area);

        // Update region membership (source of truth for region->areas listing)
        if let Some(region_id) = parent_region_id {
            let region = self
                .brain_regions
                .get_region_mut(&region_id)
                .ok_or_else(|| {
                    BduError::InvalidArea(format!(
                        "Unknown parent_region_id '{}' for cortical area {}",
                        region_id,
                        cortical_id.as_base_64()
                    ))
                })?;
            region.add_area(cortical_id);
        }

        // CRITICAL: Initialize per-area count caches to 0 (lock-free for readers)
        // This allows healthcheck endpoints to read counts without NPU lock
        {
            let mut neuron_cache = self.cached_neuron_counts_per_area.write();
            neuron_cache.insert(cortical_id, AtomicUsize::new(0));
            let mut synapse_cache = self.cached_synapse_counts_per_area.write();
            synapse_cache.insert(cortical_id, AtomicUsize::new(0));
        }

        // CRITICAL: Register cortical area in NPU during corticogenesis
        // This must happen BEFORE neurogenesis so neurons can look up their cortical IDs
        // Use base64 format for proper CorticalID conversion
        if let Some(ref npu) = self.npu {
            if let Ok(mut npu_lock) = npu.lock() {
                npu_lock.register_cortical_area(cortical_idx, cortical_id.as_base_64());
                trace!(
                    target: "feagi-bdu",
                    "Registered cortical area idx={} -> '{}' in NPU",
                    cortical_idx,
                    cortical_id.as_base_64()
                );
            }
        }

        // Synchronize cortical area flags with NPU (psp_uniform_distribution, mp_driven_psp, etc.)
        self.sync_cortical_area_flags_to_npu()?;

        self.initialized = true;

        self.refresh_cortical_area_hashes(true, true);
        self.refresh_brain_regions_hash();

        Ok(cortical_idx)
    }

    /// Remove a cortical area by ID
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - ID of the cortical area to remove
    ///
    /// # Returns
    ///
    /// `Ok(())` if removed, error if area doesn't exist
    ///
    /// # Note
    ///
    /// This does NOT remove neurons from the NPU - that must be done separately.
    ///
    pub fn remove_cortical_area(&mut self, cortical_id: &CorticalID) -> BduResult<()> {
        let area = self.cortical_areas.remove(cortical_id).ok_or_else(|| {
            BduError::InvalidArea(format!("Cortical area {} does not exist", cortical_id))
        })?;

        // Remove from lookup maps
        self.cortical_id_to_idx.remove(cortical_id);
        self.cortical_idx_to_id.remove(&area.cortical_idx);

        self.refresh_cortical_area_hashes(true, true);
        Ok(())
    }

    /// Get a cortical area by ID
    pub fn get_cortical_area(&self, cortical_id: &CorticalID) -> Option<&CorticalArea> {
        self.cortical_areas.get(cortical_id)
    }

    /// Get a mutable reference to a cortical area
    pub fn get_cortical_area_mut(&mut self, cortical_id: &CorticalID) -> Option<&mut CorticalArea> {
        self.cortical_areas.get_mut(cortical_id)
    }

    /// Get cortical index by ID
    pub fn get_cortical_idx(&self, cortical_id: &CorticalID) -> Option<u32> {
        self.cortical_id_to_idx.get(cortical_id).copied()
    }

    /// Find which brain region contains a cortical area
    ///
    /// This is used to populate `parent_region_id` in API responses for Brain Visualizer.
    /// Delegates to BrainRegionHierarchy for the actual search.
    ///
    /// # Arguments
    /// * `cortical_id` - Cortical area to search for
    ///
    /// # Returns
    /// * `Option<String>` - Parent region ID (UUID string) if found
    ///
    pub fn get_parent_region_id_for_area(&self, cortical_id: &CorticalID) -> Option<String> {
        self.brain_regions.find_region_containing_area(cortical_id)
    }

    /// Recompute brain-region `inputs`/`outputs` registries from current cortical mappings.
    ///
    /// This drives `/v1/region/regions_members` (via `BrainRegion.properties["inputs"/"outputs"]`).
    ///
    /// Semantics (matches Python `_auto_assign_region_io()` and BV expectations):
    /// - **outputs**: Any cortical area *in the region* that connects to an area outside the region
    /// - **inputs**: Any cortical area *in the region* that receives a connection from outside the region
    ///
    /// This function updates the hierarchy in-place and returns the computed base64 ID lists
    /// for downstream persistence into `RuntimeGenome`.
    pub fn recompute_brain_region_io_registry(&mut self) -> BduResult<BrainRegionIoRegistry> {
        use std::collections::HashSet;

        let region_ids: Vec<String> = self
            .brain_regions
            .get_all_region_ids()
            .into_iter()
            .cloned()
            .collect();

        let mut inputs_by_region: HashMap<String, HashSet<String>> = HashMap::new();
        let mut outputs_by_region: HashMap<String, HashSet<String>> = HashMap::new();

        // Initialize so regions with no IO still get cleared deterministically.
        for rid in &region_ids {
            inputs_by_region.insert(rid.clone(), HashSet::new());
            outputs_by_region.insert(rid.clone(), HashSet::new());
        }

        for (src_id, src_area) in &self.cortical_areas {
            let Some(dstmap) = src_area
                .properties
                .get("cortical_mapping_dst")
                .and_then(|v| v.as_object())
            else {
                continue;
            };

            let src_region_id = self.brain_regions.find_region_containing_area(src_id).ok_or_else(|| {
                BduError::InvalidArea(format!(
                    "Unable to recompute region IO: source cortical area {} is not assigned to any region",
                    src_id.as_base_64()
                ))
            })?;

            for dst_id_str in dstmap.keys() {
                let dst_id = CorticalID::try_from_base_64(dst_id_str).map_err(|e| {
                    BduError::InvalidArea(format!(
                        "Unable to recompute region IO: invalid destination cortical id '{}' in cortical_mapping_dst for {}: {}",
                        dst_id_str,
                        src_id.as_base_64(),
                        e
                    ))
                })?;

                let dst_region_id =
                    self.brain_regions.find_region_containing_area(&dst_id).ok_or_else(|| {
                        BduError::InvalidArea(format!(
                            "Unable to recompute region IO: destination cortical area {} is not assigned to any region",
                            dst_id.as_base_64()
                        ))
                    })?;

                if src_region_id == dst_region_id {
                    continue;
                }

                outputs_by_region
                    .entry(src_region_id.clone())
                    .or_default()
                    .insert(src_id.as_base_64());
                inputs_by_region
                    .entry(dst_region_id.clone())
                    .or_default()
                    .insert(dst_id.as_base_64());
            }
        }

        let mut computed: HashMap<String, (Vec<String>, Vec<String>)> = HashMap::new();
        for rid in region_ids {
            let mut inputs: Vec<String> = inputs_by_region
                .remove(&rid)
                .unwrap_or_default()
                .into_iter()
                .collect();
            let mut outputs: Vec<String> = outputs_by_region
                .remove(&rid)
                .unwrap_or_default()
                .into_iter()
                .collect();

            inputs.sort();
            outputs.sort();

            let region = self.brain_regions.get_region_mut(&rid).ok_or_else(|| {
                BduError::InvalidArea(format!(
                    "Unable to recompute region IO: region '{}' not found in hierarchy",
                    rid
                ))
            })?;

            if inputs.is_empty() {
                region.properties.remove("inputs");
            } else {
                region
                    .properties
                    .insert("inputs".to_string(), serde_json::json!(inputs.clone()));
            }

            if outputs.is_empty() {
                region.properties.remove("outputs");
            } else {
                region
                    .properties
                    .insert("outputs".to_string(), serde_json::json!(outputs.clone()));
            }

            computed.insert(rid, (inputs, outputs));
        }

        self.refresh_brain_regions_hash();

        Ok(computed)
    }

    /// Get the root brain region ID (region with no parent)
    ///
    /// # Returns
    /// * `Option<String>` - Root region ID (UUID string) if found
    ///
    pub fn get_root_region_id(&self) -> Option<String> {
        self.brain_regions.get_root_region_id()
    }

    /// Get cortical ID by index
    pub fn get_cortical_id(&self, cortical_idx: u32) -> Option<&CorticalID> {
        self.cortical_idx_to_id.get(&cortical_idx)
    }

    /// Get all cortical_idx -> cortical_id mappings (for burst loop caching)
    /// Returns a HashMap of cortical_idx -> cortical_id (base64 string)
    pub fn get_all_cortical_idx_to_id_mappings(&self) -> ahash::AHashMap<u32, String> {
        self.cortical_idx_to_id
            .iter()
            .map(|(idx, id)| (*idx, id.as_base_64()))
            .collect()
    }

    /// Get all cortical_idx -> visualization_voxel_granularity mappings
    ///
    /// Returns a map of cortical_idx to (granularity_x, granularity_y, granularity_z) for areas that have
    /// visualization voxel granularity configured.
    pub fn get_all_visualization_granularities(&self) -> ahash::AHashMap<u32, (u32, u32, u32)> {
        let mut granularities = ahash::AHashMap::new();
        for (cortical_id, area) in &self.cortical_areas {
            let cortical_idx = self
                .cortical_id_to_idx
                .get(cortical_id)
                .copied()
                .unwrap_or(0);

            // Extract visualization granularity overrides from properties.
            // Default is 1x1x1 (assumed, not stored) so we only include non-default overrides.
            if let Some(granularity_json) = area.properties.get("visualization_voxel_granularity") {
                if let Some(arr) = granularity_json.as_array() {
                    if arr.len() == 3 {
                        let x_opt = arr[0]
                            .as_u64()
                            .or_else(|| arr[0].as_f64().map(|f| f as u64));
                        let y_opt = arr[1]
                            .as_u64()
                            .or_else(|| arr[1].as_f64().map(|f| f as u64));
                        let z_opt = arr[2]
                            .as_u64()
                            .or_else(|| arr[2].as_f64().map(|f| f as u64));

                        if let (Some(x), Some(y), Some(z)) = (x_opt, y_opt, z_opt) {
                            let granularity = (x as u32, y as u32, z as u32);
                            // Only include overrides (non-default)
                            if granularity != (1, 1, 1) {
                                granularities.insert(cortical_idx, granularity);
                            }
                        }
                    }
                }
            }
        }
        granularities
    }

    /// Get all cortical area IDs
    pub fn get_cortical_area_ids(&self) -> Vec<&CorticalID> {
        self.cortical_areas.keys().collect()
    }

    /// Get the number of cortical areas
    pub fn get_cortical_area_count(&self) -> usize {
        self.cortical_areas.len()
    }

    /// Get all cortical areas that have synapses targeting the specified area (upstream/afferent areas)
    ///
    /// Reads from the `upstream_cortical_areas` property stored on the cortical area.
    /// This property is maintained by `add_upstream_area()` and `remove_upstream_area()`.
    ///
    /// # Arguments
    ///
    /// * `target_cortical_id` - The cortical area ID to find upstream connections for
    ///
    /// # Returns
    ///
    /// Vec of cortical_idx values for all upstream areas
    ///
    pub fn get_upstream_cortical_areas(&self, target_cortical_id: &CorticalID) -> Vec<u32> {
        if let Some(area) = self.cortical_areas.get(target_cortical_id) {
            if let Some(upstream_prop) = area.properties.get("upstream_cortical_areas") {
                if let Some(upstream_array) = upstream_prop.as_array() {
                    return upstream_array
                        .iter()
                        .filter_map(|v| v.as_u64().map(|n| n as u32))
                        .collect();
                }
            }

            // Property missing - data integrity issue
            warn!(target: "feagi-bdu",
                "Cortical area '{}' missing 'upstream_cortical_areas' property - treating as empty",
                target_cortical_id.as_base_64()
            );
        }

        Vec::new()
    }

    /// Add an upstream cortical area to a target area's upstream list
    ///
    /// This should be called when synapses are created from src_cortical_idx to target_cortical_id.
    ///
    /// # Arguments
    ///
    /// * `target_cortical_id` - The cortical area receiving connections
    /// * `src_cortical_idx` - The cortical index of the source area
    ///
    pub fn add_upstream_area(&mut self, target_cortical_id: &CorticalID, src_cortical_idx: u32) {
        if let Some(area) = self.cortical_areas.get_mut(target_cortical_id) {
            let upstream_array = area
                .properties
                .entry("upstream_cortical_areas".to_string())
                .or_insert_with(|| serde_json::json!([]));

            if let Some(arr) = upstream_array.as_array_mut() {
                let src_value = serde_json::json!(src_cortical_idx);
                if !arr.contains(&src_value) {
                    arr.push(src_value);
                    info!(target: "feagi-bdu",
                        "✓ Added upstream area idx={} to cortical area '{}'",
                        src_cortical_idx, target_cortical_id.as_base_64()
                    );
                }
            }
        }
    }

    /// Remove an upstream cortical area from a target area's upstream list
    ///
    /// This should be called when all synapses from src_cortical_idx to target_cortical_id are deleted.
    ///
    /// # Arguments
    ///
    /// * `target_cortical_id` - The cortical area that had connections
    /// * `src_cortical_idx` - The cortical index of the source area to remove
    ///
    pub fn remove_upstream_area(&mut self, target_cortical_id: &CorticalID, src_cortical_idx: u32) {
        if let Some(area) = self.cortical_areas.get_mut(target_cortical_id) {
            if let Some(upstream_prop) = area.properties.get_mut("upstream_cortical_areas") {
                if let Some(arr) = upstream_prop.as_array_mut() {
                    let src_value = serde_json::json!(src_cortical_idx);
                    if let Some(pos) = arr.iter().position(|v| v == &src_value) {
                        arr.remove(pos);
                        debug!(target: "feagi-bdu",
                            "Removed upstream area idx={} from cortical area '{}'",
                            src_cortical_idx, target_cortical_id.as_base_64()
                        );
                    }
                }
            }
        }
    }

    /// Check if a cortical area exists
    pub fn has_cortical_area(&self, cortical_id: &CorticalID) -> bool {
        self.cortical_areas.contains_key(cortical_id)
    }

    /// Check if the connectome is initialized (has areas)
    pub fn is_initialized(&self) -> bool {
        self.initialized && !self.cortical_areas.is_empty()
    }

    // ======================================================================
    // Brain Region Management
    // ======================================================================

    /// Add a brain region
    pub fn add_brain_region(
        &mut self,
        region: BrainRegion,
        parent_id: Option<String>,
    ) -> BduResult<()> {
        self.brain_regions.add_region(region, parent_id)?;
        self.refresh_brain_regions_hash();
        Ok(())
    }

    /// Remove a brain region
    pub fn remove_brain_region(&mut self, region_id: &str) -> BduResult<()> {
        self.brain_regions.remove_region(region_id)?;
        self.refresh_brain_regions_hash();
        Ok(())
    }

    /// Get a brain region by ID
    pub fn get_brain_region(&self, region_id: &str) -> Option<&BrainRegion> {
        self.brain_regions.get_region(region_id)
    }

    /// Get a mutable reference to a brain region
    pub fn get_brain_region_mut(&mut self, region_id: &str) -> Option<&mut BrainRegion> {
        self.brain_regions.get_region_mut(region_id)
    }

    /// Get all brain region IDs
    pub fn get_brain_region_ids(&self) -> Vec<&String> {
        self.brain_regions.get_all_region_ids()
    }

    /// Get the brain region hierarchy
    pub fn get_brain_region_hierarchy(&self) -> &BrainRegionHierarchy {
        &self.brain_regions
    }

    // ========================================================================
    // MORPHOLOGY ACCESS
    // ========================================================================

    /// Get all morphologies from the loaded genome
    pub fn get_morphologies(&self) -> &feagi_evolutionary::MorphologyRegistry {
        &self.morphology_registry
    }

    /// Get morphology count
    pub fn get_morphology_count(&self) -> usize {
        self.morphology_registry.count()
    }

    /// Insert or overwrite a morphology definition in the in-memory registry.
    ///
    /// NOTE: This updates the runtime registry used by mapping/synapse generation.
    /// Callers that also maintain a RuntimeGenome (source of truth) MUST update it too.
    pub fn upsert_morphology(
        &mut self,
        morphology_id: String,
        morphology: feagi_evolutionary::Morphology,
    ) {
        self.morphology_registry
            .add_morphology(morphology_id, morphology);
        self.refresh_morphologies_hash();
    }

    /// Remove a morphology definition from the in-memory registry.
    ///
    /// Returns true if the morphology existed and was removed.
    ///
    /// NOTE: Callers that also maintain a RuntimeGenome (source of truth) MUST update it too.
    pub fn remove_morphology(&mut self, morphology_id: &str) -> bool {
        let removed = self.morphology_registry.remove_morphology(morphology_id);
        if removed {
            self.refresh_morphologies_hash();
        }
        removed
    }

    // ========================================================================
    // CORTICAL MAPPING UPDATES
    // ========================================================================

    /// Update cortical mapping properties between two cortical areas
    ///
    /// Following Python's update_cortical_mapping_properties() logic:
    /// 1. Updates the source area's cortical_mapping_dst property
    /// 2. Triggers synapse regeneration for the affected connection
    ///
    /// # Arguments
    /// * `src_area_id` - Source cortical area ID
    /// * `dst_area_id` - Destination cortical area ID
    /// * `mapping_data` - List of connection specifications
    ///
    /// # Returns
    /// * `BduResult<()>` - Ok if successful, Err otherwise
    pub fn update_cortical_mapping(
        &mut self,
        src_area_id: &CorticalID,
        dst_area_id: &CorticalID,
        mapping_data: Vec<serde_json::Value>,
    ) -> BduResult<()> {
        use tracing::info;

        info!(target: "feagi-bdu", "Updating cortical mapping: {} -> {}", src_area_id, dst_area_id);

        // Get source area (must exist)
        let src_area = self.cortical_areas.get_mut(src_area_id).ok_or_else(|| {
            crate::types::BduError::InvalidArea(format!("Source area not found: {}", src_area_id))
        })?;

        // Get or create cortical_mapping_dst property
        let cortical_mapping_dst =
            if let Some(existing) = src_area.properties.get_mut("cortical_mapping_dst") {
                existing.as_object_mut().ok_or_else(|| {
                    crate::types::BduError::InvalidMorphology(
                        "cortical_mapping_dst is not an object".to_string(),
                    )
                })?
            } else {
                // Create new cortical_mapping_dst
                src_area
                    .properties
                    .insert("cortical_mapping_dst".to_string(), serde_json::json!({}));
                src_area
                    .properties
                    .get_mut("cortical_mapping_dst")
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
            };

        // Update or add the mapping for this destination
        if mapping_data.is_empty() {
            // Empty mapping_data = delete the connection
            cortical_mapping_dst.remove(&dst_area_id.as_base_64());
            info!(target: "feagi-bdu", "Removed mapping from {} to {}", src_area_id, dst_area_id);
        } else {
            cortical_mapping_dst.insert(
                dst_area_id.as_base_64(),
                serde_json::Value::Array(mapping_data.clone()),
            );
            info!(target: "feagi-bdu", "Updated mapping from {} to {} with {} connections",
                  src_area_id, dst_area_id, mapping_data.len());
        }

        self.refresh_cortical_mappings_hash();

        Ok(())
    }

    /// Regenerate synapses for a specific cortical mapping
    ///
    /// Creates new synapses based on mapping rules. Only removes existing synapses if
    /// a mapping already existed (update case), not for new mappings (allows multiple
    /// synapses between the same neurons).
    ///
    /// # Arguments
    /// * `src_area_id` - Source cortical area ID
    /// * `dst_area_id` - Destination cortical area ID
    ///
    /// # Returns
    /// * `BduResult<usize>` - Number of synapses created
    pub fn regenerate_synapses_for_mapping(
        &mut self,
        src_area_id: &CorticalID,
        dst_area_id: &CorticalID,
    ) -> BduResult<usize> {
        use tracing::info;

        info!(target: "feagi-bdu", "Regenerating synapses: {} -> {}", src_area_id, dst_area_id);

        // If NPU is available, regenerate synapses
        let Some(npu_arc) = self.npu.clone() else {
            info!(target: "feagi-bdu", "NPU not available - skipping synapse regeneration");
            return Ok(0);
        };

        // Mapping regeneration must be deterministic:
        // - On mapping deletion: prune all synapses from A→B, then attempt synaptogenesis (which yields 0).
        // - On rule removal/updates: prune all synapses from A→B, then re-run synaptogenesis using the *current*
        //   mapping rules. This guarantees stale synapses from removed rules do not persist, while preserving
        //   other A→B mappings by re-creating them from the remaining rules.
        //
        // NOTE: This pruning requires retrieving neuron IDs in each area. Today, that can be O(all_neurons)
        // via `get_neurons_in_cortical_area()`. This is the safest correctness-first behavior.

        let src_idx = *self.cortical_id_to_idx.get(src_area_id).ok_or_else(|| {
            BduError::InvalidArea(format!("No cortical idx for source area {}", src_area_id))
        })?;
        let dst_idx = *self.cortical_id_to_idx.get(dst_area_id).ok_or_else(|| {
            BduError::InvalidArea(format!(
                "No cortical idx for destination area {}",
                dst_area_id
            ))
        })?;

        // Prune all existing synapses from src_area→dst_area before (re)creating based on current rules.
        // This prevents stale synapses when rules are removed/edited.
        let mut pruned_synapse_count: usize = 0;
        use std::time::Instant;
        let start = Instant::now();

        // Get neuron lists (may be slow; see note above).
        //
        // IMPORTANT: Do not rely on per-area cached neuron counts here. Pruning must be correct even if
        // caches are stale (e.g., in tests or during partial initialization). If either side is empty,
        // pruning is a no-op anyway.
        let (sources, targets) = {
            let npu = npu_arc.lock().unwrap();
            let sources: Vec<NeuronId> = npu
                .get_neurons_in_cortical_area(src_idx)
                .into_iter()
                .map(NeuronId)
                .collect();
            let targets: Vec<NeuronId> = npu
                .get_neurons_in_cortical_area(dst_idx)
                .into_iter()
                .map(NeuronId)
                .collect();
            (sources, targets)
        };

        if !sources.is_empty() && !targets.is_empty() {
            let remove_start = Instant::now();
            pruned_synapse_count = {
                let mut npu = npu_arc.lock().unwrap();
                npu.remove_synapses_from_sources_to_targets(sources, targets)
            };
            let remove_time = remove_start.elapsed();
            let total_time = start.elapsed();

            info!(
                target: "feagi-bdu",
                "Pruned {} existing synapses for mapping {} -> {} (total={}ms, remove={}ms)",
                pruned_synapse_count,
                src_area_id,
                dst_area_id,
                total_time.as_millis(),
                remove_time.as_millis()
            );

            // Update StateManager synapse count (health_check endpoint)
            if pruned_synapse_count > 0 {
                let pruned_u32 = u32::try_from(pruned_synapse_count).map_err(|_| {
                    BduError::Internal(format!(
                        "Pruned synapse count overflow (usize -> u32): {}",
                        pruned_synapse_count
                    ))
                })?;
                if let Some(state_manager) = StateManager::instance().try_read() {
                    let core_state = state_manager.get_core_state();
                    core_state.subtract_synapse_count(pruned_u32);
                }

                // Best-effort: adjust per-area outgoing synapse count cache for the source area.
                // (Cache is used for lock-free health-check reads; correctness is eventually
                // consistent via periodic refresh of global count from NPU.)
                {
                    let mut cache = self.cached_synapse_counts_per_area.write();
                    let entry = cache
                        .entry(*src_area_id)
                        .or_insert_with(|| AtomicUsize::new(0));
                    let mut current = entry.load(Ordering::Relaxed);
                    loop {
                        let next = current.saturating_sub(pruned_synapse_count);
                        match entry.compare_exchange(
                            current,
                            next,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => break,
                            Err(v) => current = v,
                        }
                    }
                }
            }
        }

        // Apply cortical mapping rules to create synapses (may be 0 for memory areas).
        //
        // IMPORTANT:
        // - We already pruned A→B synapses above to ensure no stale synapses remain after a rule removal/update.
        // - `apply_cortical_mapping_for_pair()` returns the created synapse count but does not update
        //   StateManager/caches; we do that immediately after the call.
        let synapse_count = self.apply_cortical_mapping_for_pair(src_area_id, dst_area_id)?;

        // Update synapse count caches and StateManager based on synapses created.
        // NOTE: apply_cortical_mapping_for_pair() does not touch caches/StateManager.
        if synapse_count > 0 {
            let created_u32 = u32::try_from(synapse_count).map_err(|_| {
                BduError::Internal(format!(
                    "Created synapse count overflow (usize -> u32): {}",
                    synapse_count
                ))
            })?;

            // Update per-area outgoing synapse count cache (source area)
            {
                let mut cache = self.cached_synapse_counts_per_area.write();
                cache
                    .entry(*src_area_id)
                    .or_insert_with(|| AtomicUsize::new(0))
                    .fetch_add(synapse_count, Ordering::Relaxed);
            }

            // Update StateManager synapse count (health_check endpoint)
            if let Some(state_manager) = StateManager::instance().try_read() {
                let core_state = state_manager.get_core_state();
                core_state.add_synapse_count(created_u32);
            }
        }

        // Update upstream area tracking based on MAPPING existence, not synapse count
        // Memory areas have 0 synapses but still need upstream tracking for pattern detection
        let src_idx_for_upstream = src_idx;

        // Check if mapping exists by looking at cortical_mapping_dst property (after update)
        let has_mapping = self
            .cortical_areas
            .get(src_area_id)
            .and_then(|area| area.properties.get("cortical_mapping_dst"))
            .and_then(|v| v.as_object())
            .and_then(|map| map.get(&dst_area_id.as_base_64()))
            .is_some();

        info!(target: "feagi-bdu",
            "Mapping result: {} synapses, {} -> {} (mapping_exists={}, will {}update upstream)",
            synapse_count,
            src_area_id.as_base_64(),
            dst_area_id.as_base_64(),
            has_mapping,
            if has_mapping { "" } else { "NOT " }
        );

        if has_mapping {
            // Mapping exists - add to upstream tracking (for both memory and regular areas)
            self.add_upstream_area(dst_area_id, src_idx_for_upstream);

            // If destination is a memory area, register it with PlasticityExecutor (automatic)
            #[cfg(feature = "plasticity")]
            if let Some(ref executor) = self.plasticity_executor {
                use feagi_evolutionary::extract_memory_properties;

                if let Some(dst_area) = self.cortical_areas.get(dst_area_id) {
                    if let Some(mem_props) = extract_memory_properties(&dst_area.properties) {
                        let upstream_areas = self.get_upstream_cortical_areas(dst_area_id);

                        // Ensure FireLedger tracks the upstream areas with at least the required temporal depth.
                        // Dense, burst-aligned tracking is required for correct memory pattern hashing.
                        if let Some(ref npu_arc) = self.npu {
                            if let Ok(mut npu) = npu_arc.lock() {
                                let existing_configs = npu.get_all_fire_ledger_configs();
                                for &upstream_idx in &upstream_areas {
                                    let existing = existing_configs
                                        .iter()
                                        .find(|(idx, _)| *idx == upstream_idx)
                                        .map(|(_, w)| *w)
                                        .unwrap_or(0);

                                    let desired = mem_props.temporal_depth as usize;
                                    let resolved = existing.max(desired);
                                    if resolved != existing {
                                        if let Err(e) =
                                            npu.configure_fire_ledger_window(upstream_idx, resolved)
                                        {
                                            warn!(
                                                target: "feagi-bdu",
                                                "Failed to configure FireLedger window for upstream area idx={} (requested={}): {}",
                                                upstream_idx,
                                                resolved,
                                                e
                                            );
                                        }
                                    }
                                }
                            } else {
                                warn!(target: "feagi-bdu", "Failed to lock NPU for FireLedger configuration");
                            }
                        }

                        if let Ok(exec) = executor.lock() {
                            use feagi_npu_plasticity::{
                                MemoryNeuronLifecycleConfig, PlasticityExecutor,
                            };

                            let lifecycle_config = MemoryNeuronLifecycleConfig {
                                initial_lifespan: mem_props.init_lifespan,
                                lifespan_growth_rate: mem_props.lifespan_growth_rate,
                                longterm_threshold: mem_props.longterm_threshold,
                                max_reactivations: 1000,
                            };

                            exec.register_memory_area(
                                dst_area.cortical_idx,
                                dst_area_id.as_base_64(),
                                mem_props.temporal_depth,
                                upstream_areas.clone(),
                                Some(lifecycle_config),
                            );
                        } else {
                            warn!(target: "feagi-bdu", "Failed to lock PlasticityExecutor");
                        }
                    }
                } else {
                    warn!(target: "feagi-bdu", "Destination area {} not found in cortical_areas", dst_area_id.as_base_64());
                }
            } else {
                info!(target: "feagi-bdu", "PlasticityExecutor not available (feature disabled or not initialized)");
            }

            #[cfg(not(feature = "plasticity"))]
            {
                info!(target: "feagi-bdu", "Plasticity feature disabled at compile time");
            }
        } else {
            // Mapping deleted - remove from upstream tracking
            self.remove_upstream_area(dst_area_id, src_idx_for_upstream);

            // Ensure any STDP mapping parameters for this pair are removed when the mapping is gone.
            let mut npu = npu_arc.lock().unwrap();
            let _was_registered = npu.unregister_stdp_mapping(src_idx, dst_idx);
        }

        info!(
            target: "feagi-bdu",
            "Created {} new synapses: {} -> {}",
            synapse_count,
            src_area_id,
            dst_area_id
        );

        // CRITICAL: Rebuild synapse index so removals are reflected in propagation and query paths.
        // Many morphology paths rebuild the index after creation, but pruning requires an explicit rebuild.
        if pruned_synapse_count > 0 || synapse_count == 0 {
            let mut npu = npu_arc.lock().unwrap();
            npu.rebuild_synapse_index();
            info!(
                target: "feagi-bdu",
                "Rebuilt synapse index after regenerating {} -> {} (pruned={}, created={})",
                src_area_id,
                dst_area_id,
                pruned_synapse_count,
                synapse_count
            );
        } else {
            info!(
                target: "feagi-bdu",
                "Skipped synapse index rebuild for mapping {} -> {} (created={}, pruned=0; index rebuilt during synaptogenesis)",
                src_area_id,
                dst_area_id,
                synapse_count
            );
        }

        // Refresh the global synapse count cache from NPU (deterministic after prune/create).
        {
            let npu = npu_arc.lock().unwrap();
            let fresh_count = npu.get_synapse_count();
            self.cached_synapse_count
                .store(fresh_count, Ordering::Relaxed);
        }

        Ok(synapse_count)
    }

    /// Register STDP mapping parameters for a plastic rule
    fn register_stdp_mapping_for_rule(
        npu: &Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
        src_area_id: &CorticalID,
        dst_area_id: &CorticalID,
        src_cortical_idx: u32,
        dst_cortical_idx: u32,
        rule_obj: &serde_json::Map<String, serde_json::Value>,
    ) -> BduResult<()> {
        let plasticity_window = rule_obj
            .get("plasticity_window")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                BduError::Internal(format!(
                    "Missing plasticity_window in plastic mapping rule {} -> {}",
                    src_area_id, dst_area_id
                ))
            })? as usize;
        let plasticity_constant = rule_obj
            .get("plasticity_constant")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| {
                BduError::Internal(format!(
                    "Missing plasticity_constant in plastic mapping rule {} -> {}",
                    src_area_id, dst_area_id
                ))
            })?;
        let ltp_multiplier = rule_obj
            .get("ltp_multiplier")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| {
                BduError::Internal(format!(
                    "Missing ltp_multiplier in plastic mapping rule {} -> {}",
                    src_area_id, dst_area_id
                ))
            })?;
        let ltd_multiplier = rule_obj
            .get("ltd_multiplier")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| {
                BduError::Internal(format!(
                    "Missing ltd_multiplier in plastic mapping rule {} -> {}",
                    src_area_id, dst_area_id
                ))
            })?;

        let params = feagi_npu_burst_engine::npu::StdpMappingParams {
            plasticity_window,
            plasticity_constant,
            ltp_multiplier,
            ltd_multiplier,
        };

        let mut npu_lock = npu
            .lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;

        npu_lock
            .register_stdp_mapping(src_cortical_idx, dst_cortical_idx, params)
            .map_err(|e| {
                BduError::Internal(format!(
                    "Failed to register STDP mapping {} -> {}: {}",
                    src_area_id, dst_area_id, e
                ))
            })?;

        // FireLedger tracking for STDP (ensure A and B are tracked at least to plasticity_window)
        let existing_configs = npu_lock.get_all_fire_ledger_configs();
        for area_idx in [src_cortical_idx, dst_cortical_idx] {
            let existing = existing_configs
                .iter()
                .find(|(idx, _)| *idx == area_idx)
                .map(|(_, w)| *w)
                .unwrap_or(0);
            let resolved = existing.max(plasticity_window);
            if resolved != existing {
                npu_lock
                    .configure_fire_ledger_window(area_idx, resolved)
                    .map_err(|e| {
                        BduError::Internal(format!(
                            "Failed to configure FireLedger window for area idx={} (requested={}): {}",
                            area_idx, resolved, e
                        ))
                    })?;
            }
        }

        Ok(())
    }

    /// Apply cortical mapping for a specific area pair
    fn apply_cortical_mapping_for_pair(
        &mut self,
        src_area_id: &CorticalID,
        dst_area_id: &CorticalID,
    ) -> BduResult<usize> {
        // Clone the rules to avoid borrow checker issues.
        //
        // IMPORTANT: absence of mapping rules is a valid state (e.g. mapping deletion).
        // In that case, return Ok(0) rather than an error so API callers can treat
        // "deleted mapping" as success (and BV can update its cache/UI).
        let rules = {
            let src_area = self.cortical_areas.get(src_area_id).ok_or_else(|| {
                crate::types::BduError::InvalidArea(format!(
                    "Source area not found: {}",
                    src_area_id
                ))
            })?;

            let Some(mapping_dst) = src_area
                .properties
                .get("cortical_mapping_dst")
                .and_then(|v| v.as_object())
            else {
                return Ok(0);
            };

            let Some(rules) = mapping_dst
                .get(&dst_area_id.as_base_64())
                .and_then(|v| v.as_array())
            else {
                return Ok(0);
            };

            rules.clone()
        }; // Borrow ends here

        if rules.is_empty() {
            return Ok(0);
        }

        // Get indices for STDP handling
        let src_cortical_idx = *self.cortical_id_to_idx.get(src_area_id).ok_or_else(|| {
            crate::types::BduError::InvalidArea(format!("No index for {}", src_area_id))
        })?;
        let dst_cortical_idx = *self.cortical_id_to_idx.get(dst_area_id).ok_or_else(|| {
            crate::types::BduError::InvalidArea(format!("No index for {}", dst_area_id))
        })?;

        // Clone NPU Arc for STDP handling (Arc::clone is cheap - just increments ref count)
        let npu_arc = self
            .npu
            .as_ref()
            .ok_or_else(|| crate::types::BduError::Internal("NPU not connected".to_string()))?
            .clone();

        // Apply each morphology rule
        let mut total_synapses = 0;
        for rule in &rules {
            let rule_obj = match rule.as_object() {
                Some(obj) => obj,
                None => continue,
            };

            // Handle STDP/plasticity configuration if needed
            let plasticity_flag = rule_obj
                .get("plasticity_flag")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if plasticity_flag {
                Self::register_stdp_mapping_for_rule(
                    &npu_arc,
                    src_area_id,
                    dst_area_id,
                    src_cortical_idx,
                    dst_cortical_idx,
                    rule_obj,
                )?;
            }

            // Apply the morphology rule
            let synapse_count =
                self.apply_single_morphology_rule(src_area_id, dst_area_id, rule)?;
            total_synapses += synapse_count;
        }

        Ok(total_synapses)
    }

    /// Apply a function-type morphology (projector, memory, block_to_block, etc.)
    ///
    /// This helper consolidates all function-type morphology logic in one place.
    /// Function-type morphologies are code-driven and require code changes to add new ones.
    ///
    /// # Arguments
    /// * `morphology_id` - The morphology ID string (e.g., "projector", "block_to_block")
    /// * `rule` - The morphology rule JSON value
    /// * `npu_arc` - Arc to the NPU (for batched operations)
    /// * `npu` - Locked NPU reference
    /// * `src_area_id`, `dst_area_id` - Source and destination area IDs
    /// * `src_idx`, `dst_idx` - Source and destination area indices
    /// * `weight`, `conductance`, `synapse_attractivity` - Synapse parameters
    #[allow(clippy::too_many_arguments)]
    fn apply_function_morphology(
        &self,
        morphology_id: &str,
        rule: &serde_json::Value,
        npu_arc: &Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
        npu: &mut feagi_npu_burst_engine::DynamicNPU,
        src_area_id: &CorticalID,
        dst_area_id: &CorticalID,
        src_idx: u32,
        dst_idx: u32,
        weight: u8,
        conductance: u8,
        synapse_attractivity: u8,
        synapse_type: feagi_npu_neural::SynapseType,
    ) -> BduResult<usize> {
        match morphology_id {
            "projector" => {
                // Get dimensions from cortical areas (no neuron scanning!)
                let src_area = self.cortical_areas.get(src_area_id).ok_or_else(|| {
                    crate::types::BduError::InvalidArea(format!(
                        "Source area not found: {}",
                        src_area_id
                    ))
                })?;
                let dst_area = self.cortical_areas.get(dst_area_id).ok_or_else(|| {
                    crate::types::BduError::InvalidArea(format!(
                        "Destination area not found: {}",
                        dst_area_id
                    ))
                })?;

                let src_dimensions = (
                    src_area.dimensions.width as usize,
                    src_area.dimensions.height as usize,
                    src_area.dimensions.depth as usize,
                );
                let dst_dimensions = (
                    dst_area.dimensions.width as usize,
                    dst_area.dimensions.height as usize,
                    dst_area.dimensions.depth as usize,
                );

                use crate::connectivity::core_morphologies::apply_projector_morphology_with_dimensions;
                let count = apply_projector_morphology_with_dimensions(
                    npu,
                    src_idx,
                    dst_idx,
                    src_dimensions,
                    dst_dimensions,
                    None, // transpose
                    None, // project_last_layer_of
                    weight,
                    conductance,
                    synapse_attractivity,
                    synapse_type,
                )?;
                // Ensure the propagation engine sees the newly created synapses immediately
                npu.rebuild_synapse_index();
                Ok(count as usize)
            }
            "memory" => {
                // Memory morphology: No physical synapses created
                // Pattern detection and memory neuron creation handled by PlasticityService
                use tracing::trace;
                trace!(
                    target: "feagi-bdu",
                    "Memory morphology: {} -> {} (no physical synapses, plasticity-driven)",
                    src_idx, dst_idx
                );
                Ok(0)
            }
            "block_to_block" => {
                tracing::warn!(
                    target: "feagi-bdu",
                    "🔍 DEBUG apply_function_morphology: block_to_block case reached with src_idx={}, dst_idx={}",
                    src_idx, dst_idx
                );
                // Get dimensions from cortical areas (no neuron scanning!)
                let src_area = self.cortical_areas.get(src_area_id).ok_or_else(|| {
                    crate::types::BduError::InvalidArea(format!(
                        "Source area not found: {}",
                        src_area_id
                    ))
                })?;
                let dst_area = self.cortical_areas.get(dst_area_id).ok_or_else(|| {
                    crate::types::BduError::InvalidArea(format!(
                        "Destination area not found: {}",
                        dst_area_id
                    ))
                })?;

                let src_dimensions = (
                    src_area.dimensions.width as usize,
                    src_area.dimensions.height as usize,
                    src_area.dimensions.depth as usize,
                );
                let dst_dimensions = (
                    dst_area.dimensions.width as usize,
                    dst_area.dimensions.height as usize,
                    dst_area.dimensions.depth as usize,
                );

                // Extract scalar from rule (morphology_scalar)
                let scalar = if let Some(obj) = rule.as_object() {
                    // Object format: get from morphology_scalar array
                    if let Some(scalar_arr) =
                        obj.get("morphology_scalar").and_then(|v| v.as_array())
                    {
                        // Use first element as scalar (or default to 1)
                        scalar_arr.first().and_then(|v| v.as_i64()).unwrap_or(1) as u32
                    } else {
                        1 // @architecture:acceptable - default scalar
                    }
                } else if let Some(arr) = rule.as_array() {
                    // Array format: [morphology_id, scalar, multiplier, ...]
                    arr.get(1).and_then(|v| v.as_i64()).unwrap_or(1) as u32
                } else {
                    1 // @architecture:acceptable - default scalar
                };

                // CRITICAL: Do NOT call get_neurons_in_cortical_area to check neuron count!
                // Use dimensions to estimate: if area is large, use batched version
                let estimated_neurons = src_dimensions.0 * src_dimensions.1 * src_dimensions.2;
                let count = if estimated_neurons > 100_000 {
                    // Release lock and use batched version
                    let _ = npu;

                    crate::connectivity::synaptogenesis::apply_block_connection_morphology_batched(
                        npu_arc,
                        src_idx,
                        dst_idx,
                        src_dimensions,
                        dst_dimensions,
                        scalar, // scaling_factor
                        weight,
                        conductance,
                        synapse_attractivity,
                        synapse_type,
                    )? as usize
                } else {
                    // Small area: use regular version (faster for small counts)
                    tracing::warn!(
                        target: "feagi-bdu",
                        "🔍 DEBUG connectome_manager: Calling apply_block_connection_morphology with src_idx={}, dst_idx={}, src_dim={:?}, dst_dim={:?}",
                        src_idx, dst_idx, src_dimensions, dst_dimensions
                    );
                    let count =
                        crate::connectivity::synaptogenesis::apply_block_connection_morphology(
                            npu,
                            src_idx,
                            dst_idx,
                            src_dimensions,
                            dst_dimensions,
                            scalar, // scaling_factor
                            weight,
                            conductance,
                            synapse_attractivity,
                            synapse_type,
                        )? as usize;
                    tracing::warn!(
                        target: "feagi-bdu",
                        "🔍 DEBUG connectome_manager: apply_block_connection_morphology returned count={}",
                        count
                    );
                    // Rebuild synapse index while we still have the lock
                    if count > 0 {
                        npu.rebuild_synapse_index();
                    }
                    count
                };

                // Ensure the propagation engine sees the newly created synapses immediately (batched version only)
                if count > 0 && estimated_neurons > 100_000 {
                    let mut npu_lock = npu_arc.lock().unwrap();
                    npu_lock.rebuild_synapse_index();
                }

                Ok(count)
            }
            _ => {
                // Other function morphologies not yet implemented
                // NOTE: To add a new function-type morphology, add a case here
                use tracing::debug;
                debug!(target: "feagi-bdu", "Function morphology {} not yet implemented", morphology_id);
                Ok(0)
            }
        }
    }

    /// Apply a single morphology rule
    fn apply_single_morphology_rule(
        &mut self,
        src_area_id: &CorticalID,
        dst_area_id: &CorticalID,
        rule: &serde_json::Value,
    ) -> BduResult<usize> {
        // Extract morphology_id from rule (array or dict format)
        let morphology_id = if let Some(arr) = rule.as_array() {
            arr.first().and_then(|v| v.as_str()).unwrap_or("")
        } else if let Some(obj) = rule.as_object() {
            obj.get("morphology_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
        } else {
            return Ok(0);
        };

        if morphology_id.is_empty() {
            return Ok(0);
        }

        // Get morphology from registry
        let morphology = self.morphology_registry.get(morphology_id).ok_or_else(|| {
            crate::types::BduError::InvalidMorphology(format!(
                "Morphology not found: {}",
                morphology_id
            ))
        })?;

        // Convert area IDs to cortical indices (required by NPU functions)
        let src_idx = self.cortical_id_to_idx.get(src_area_id).ok_or_else(|| {
            crate::types::BduError::InvalidArea(format!(
                "Source area ID not found: {}",
                src_area_id
            ))
        })?;
        let dst_idx = self.cortical_id_to_idx.get(dst_area_id).ok_or_else(|| {
            crate::types::BduError::InvalidArea(format!(
                "Destination area ID not found: {}",
                dst_area_id
            ))
        })?;

        // Apply morphology based on type
        if let Some(ref npu_arc) = self.npu {
            let mut npu = npu_arc.lock().unwrap();

            // Get source area to access PSP property
            let src_area = self.cortical_areas.get(src_area_id).ok_or_else(|| {
                crate::types::BduError::InvalidArea(format!(
                    "Source area not found: {}",
                    src_area_id
                ))
            })?;

            // Extract weight from rule (postSynapticCurrent_multiplier)
            //
            // IMPORTANT:
            // - This value represents the synapse "weight" stored in the NPU (u8: 0..255).
            // - Do NOT scale by 255 here. A multiplier of 1.0 should remain weight=1 (not 255).
            let (weight, synapse_type) = {
                // Accept either integer or whole-number float inputs for compatibility with older clients/tests.
                let parse_i64 = |v: &serde_json::Value| -> Option<i64> {
                    if let Some(i) = v.as_i64() {
                        return Some(i);
                    }
                    let f = v.as_f64()?;
                    if f.fract() == 0.0 {
                        Some(f as i64)
                    } else {
                        None
                    }
                };

                let multiplier_i64: i64 = if let Some(obj) = rule.as_object() {
                    obj.get("postSynapticCurrent_multiplier")
                        .and_then(parse_i64)
                        .unwrap_or(1) // @architecture:acceptable - rule-level default multiplier
                } else if let Some(arr) = rule.as_array() {
                    // Array format: [morphology_id, scalar, multiplier, ...]
                    arr.get(2).and_then(parse_i64).unwrap_or(1) // @architecture:acceptable - rule-level default multiplier
                } else {
                    128 // @architecture:acceptable - emergency fallback for malformed rule
                };

                if multiplier_i64 < 0 {
                    let abs = if multiplier_i64 == i64::MIN {
                        i64::MAX
                    } else {
                        multiplier_i64.abs()
                    };
                    (
                        abs.clamp(0, 255) as u8,
                        feagi_npu_neural::SynapseType::Inhibitory,
                    )
                } else {
                    (
                        multiplier_i64.clamp(0, 255) as u8,
                        feagi_npu_neural::SynapseType::Excitatory,
                    )
                }
            };

            // Get PSP (conductance) from source cortical area.
            //
            // IMPORTANT:
            // - This value represents the synapse "conductance" stored in the NPU (u8: 0..255).
            // - Treat `postsynaptic_current` as an absolute value in 0..255 units.
            // - Do NOT scale by 255 here. A PSP of 1.0 should remain conductance=1 (not 255).
            let conductance = {
                use crate::models::cortical_area::CorticalAreaExt;
                let psp_f32 = src_area.postsynaptic_current();
                psp_f32.clamp(0.0, 255.0) as u8
            };

            // Extract synapse_attractivity from rule (probability 0-100)
            let synapse_attractivity = if let Some(obj) = rule.as_object() {
                obj.get("synapse_attractivity")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(100) as u8
            } else {
                100 // @architecture:acceptable - default to always create when not specified
            };

            match morphology.morphology_type {
                feagi_evolutionary::MorphologyType::Functions => {
                    tracing::warn!(
                        target: "feagi-bdu",
                        "🔍 DEBUG apply_single_morphology_rule: Functions type, morphology_id={}, calling apply_function_morphology",
                        morphology_id
                    );
                    // Function-based morphologies (projector, memory, block_to_block, etc.)
                    // Delegate to helper function to consolidate all function-type logic
                    self.apply_function_morphology(
                        morphology_id,
                        rule,
                        npu_arc,
                        &mut npu,
                        src_area_id,
                        dst_area_id,
                        *src_idx,
                        *dst_idx,
                        weight,
                        conductance,
                        synapse_attractivity,
                        synapse_type,
                    )
                }
                feagi_evolutionary::MorphologyType::Vectors => {
                    use crate::connectivity::synaptogenesis::apply_vectors_morphology_with_dimensions;

                    // Get dimensions from cortical areas (no neuron scanning!)
                    let dst_area = self.cortical_areas.get(dst_area_id).ok_or_else(|| {
                        crate::types::BduError::InvalidArea(format!(
                            "Destination area not found: {}",
                            dst_area_id
                        ))
                    })?;

                    let dst_dimensions = (
                        dst_area.dimensions.width as usize,
                        dst_area.dimensions.height as usize,
                        dst_area.dimensions.depth as usize,
                    );

                    if let feagi_evolutionary::MorphologyParameters::Vectors { ref vectors } =
                        morphology.parameters
                    {
                        // Convert Vec<[i32; 3]> to Vec<(i32, i32, i32)>
                        let vectors_tuples: Vec<(i32, i32, i32)> =
                            vectors.iter().map(|v| (v[0], v[1], v[2])).collect();

                        let count = apply_vectors_morphology_with_dimensions(
                            &mut npu,
                            *src_idx,
                            *dst_idx,
                            vectors_tuples,
                            dst_dimensions,
                            weight,               // From rule, not hardcoded
                            conductance,          // PSP from source area, NOT hardcoded!
                            synapse_attractivity, // From rule, not hardcoded
                            synapse_type,
                        )?;
                        // Ensure the propagation engine sees the newly created synapses immediately,
                        // and avoid a second outer NPU mutex acquisition later in the mapping update path.
                        npu.rebuild_synapse_index();
                        Ok(count as usize)
                    } else {
                        Ok(0)
                    }
                }
                feagi_evolutionary::MorphologyType::Patterns => {
                    use tracing::debug;
                    // Patterns morphology requires Pattern3D conversion - complex, skip for now
                    debug!(target: "feagi-bdu", "Pattern morphology {} - conversion not yet implemented", morphology_id);
                    Ok(0)
                }
                _ => {
                    use tracing::debug;
                    debug!(target: "feagi-bdu", "Morphology type {:?} not yet fully implemented", morphology.morphology_type);
                    Ok(0)
                }
            }
        } else {
            Ok(0) // NPU not available
        }
    }

    // ======================================================================
    // NPU Integration
    // ======================================================================

    /// Set the NPU reference for neuron/synapse queries
    ///
    /// This should be called once during FEAGI initialization after the NPU is created.
    ///
    /// # Arguments
    ///
    /// * `npu` - Arc to the Rust NPU (wrapped in TracingMutex for automatic lock tracing)
    ///
    pub fn set_npu(
        &mut self,
        npu: Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
    ) {
        self.npu = Some(Arc::clone(&npu));
        info!(target: "feagi-bdu","🔗 ConnectomeManager: NPU reference set");

        // CRITICAL: Update State Manager with capacity values (from config, never changes)
        // This ensures health check endpoint can read capacity without acquiring NPU lock
        #[cfg(not(feature = "wasm"))]
        {
            use feagi_state_manager::StateManager;
            if let Some(state_manager) = StateManager::instance().try_read() {
                let core_state = state_manager.get_core_state();
                // Capacity comes from config (set at initialization, never changes)
                core_state.set_neuron_capacity(self.config.max_neurons as u32);
                core_state.set_synapse_capacity(self.config.max_synapses as u32);
                info!(
                    target: "feagi-bdu",
                    "📊 Updated State Manager with capacity: {} neurons, {} synapses",
                    self.config.max_neurons, self.config.max_synapses
                );
            }
        }

        // CRITICAL: Backfill cortical area registrations into NPU.
        //
        // Cortical areas can be created/loaded before the NPU is attached (startup ordering).
        // Those areas won't be registered via `add_cortical_area()` (it registers only if NPU is present),
        // which causes visualization encoding to fall back to "area_{idx}" and subsequently drop the area
        // (base64 decode fails), making BV appear to "miss" firing activity for that cortical area.
        let existing_area_count = self.cortical_id_to_idx.len();
        if existing_area_count > 0 {
            match npu.lock() {
                Ok(mut npu_lock) => {
                    for (cortical_id, cortical_idx) in self.cortical_id_to_idx.iter() {
                        npu_lock.register_cortical_area(*cortical_idx, cortical_id.as_base_64());
                    }
                    info!(
                        target: "feagi-bdu",
                        "🔁 Backfilled {} cortical area registrations into NPU",
                        existing_area_count
                    );
                }
                Err(e) => {
                    warn!(
                        target: "feagi-bdu",
                        "⚠️ Failed to lock NPU for cortical area backfill registration: {}",
                        e
                    );
                }
            }
        }

        // Initialize cached stats immediately
        self.update_all_cached_stats();
        info!(target: "feagi-bdu","📊 Initialized cached stats: {} neurons, {} synapses",
            self.get_neuron_count(), self.get_synapse_count());
    }

    /// Check if NPU is connected
    pub fn has_npu(&self) -> bool {
        self.npu.is_some()
    }

    /// Get NPU reference (read-only access for queries)
    ///
    /// # Returns
    ///
    /// * `Option<&Arc<Mutex<RustNPU>>>` - Reference to NPU if connected
    ///
    pub fn get_npu(
        &self,
    ) -> Option<&Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>>
    {
        self.npu.as_ref()
    }

    /// Set the PlasticityExecutor reference (optional, only if plasticity feature enabled)
    /// The executor is passed as Arc<Mutex<dyn Any>> for feature-gating compatibility
    #[cfg(feature = "plasticity")]
    pub fn set_plasticity_executor(
        &mut self,
        executor: Arc<std::sync::Mutex<feagi_npu_plasticity::AsyncPlasticityExecutor>>,
    ) {
        self.plasticity_executor = Some(executor);
        info!(target: "feagi-bdu", "🔗 ConnectomeManager: PlasticityExecutor reference set");
    }

    /// Get the PlasticityExecutor reference (if plasticity feature enabled)
    #[cfg(feature = "plasticity")]
    pub fn get_plasticity_executor(
        &self,
    ) -> Option<&Arc<std::sync::Mutex<feagi_npu_plasticity::AsyncPlasticityExecutor>>> {
        self.plasticity_executor.as_ref()
    }

    /// Get neuron capacity from config (lock-free, never acquires NPU lock)
    ///
    /// # Returns
    ///
    /// * `usize` - Maximum neuron capacity from config (single source of truth)
    ///
    /// # Performance
    ///
    /// This is a lock-free read from config that never blocks, even during burst processing.
    /// Capacity values are set at NPU initialization and never change.
    ///
    pub fn get_neuron_capacity(&self) -> usize {
        // CRITICAL: Read from config, NOT NPU - capacity never changes and should not acquire locks
        self.config.max_neurons
    }

    /// Get synapse capacity from config (lock-free, never acquires NPU lock)
    ///
    /// # Returns
    ///
    /// * `usize` - Maximum synapse capacity from config (single source of truth)
    ///
    /// # Performance
    ///
    /// This is a lock-free read from config that never blocks, even during burst processing.
    /// Capacity values are set at NPU initialization and never change.
    ///
    pub fn get_synapse_capacity(&self) -> usize {
        // CRITICAL: Read from config, NOT NPU - capacity never changes and should not acquire locks
        self.config.max_synapses
    }

    /// Update fatigue index based on utilization of neuron and synapse arrays
    ///
    /// Calculates fatigue index as max(regular_neuron_util%, memory_neuron_util%, synapse_util%)
    /// Applies hysteresis: triggers at 85%, clears at 80%
    /// Rate limited to max once per 2 seconds to protect against rapid changes
    ///
    /// # Safety
    ///
    /// This method is completely non-blocking and safe to call during genome loading.
    /// If StateManager is unavailable or locked, it will skip the calculation gracefully.
    ///
    /// # Returns
    ///
    /// * `Option<u8>` - New fatigue index (0-100) if calculation was performed, None if rate limited or StateManager unavailable
    pub fn update_fatigue_index(&self) -> Option<u8> {
        // Rate limiting: max once per 2 seconds
        let mut last_calc = match self.last_fatigue_calculation.lock() {
            Ok(guard) => guard,
            Err(_) => return None, // Lock poisoned, skip calculation
        };

        let now = std::time::Instant::now();
        if now.duration_since(*last_calc).as_secs() < 2 {
            return None; // Rate limited
        }
        *last_calc = now;
        drop(last_calc);

        // Get regular neuron utilization
        let regular_neuron_count = self.get_neuron_count();
        let regular_neuron_capacity = self.get_neuron_capacity();
        let regular_neuron_util = if regular_neuron_capacity > 0 {
            ((regular_neuron_count as f64 / regular_neuron_capacity as f64) * 100.0).round() as u8
        } else {
            0
        };

        // Get memory neuron utilization from state manager
        // Use try_read() to avoid blocking during neurogenesis
        // If StateManager singleton initialization fails or is locked, skip calculation entirely
        let memory_neuron_util = match StateManager::instance().try_read() {
            Some(state_manager) => state_manager.get_core_state().get_memory_neuron_util(),
            None => {
                // StateManager is locked or not ready - skip fatigue calculation
                return None;
            }
        };

        // Get synapse utilization
        let synapse_count = self.get_synapse_count();
        let synapse_capacity = self.get_synapse_capacity();
        let synapse_util = if synapse_capacity > 0 {
            ((synapse_count as f64 / synapse_capacity as f64) * 100.0).round() as u8
        } else {
            0
        };

        // Calculate fatigue index as max of all utilizations
        let fatigue_index = regular_neuron_util
            .max(memory_neuron_util)
            .max(synapse_util);

        // Apply hysteresis: trigger at 85%, clear at 80%
        let current_fatigue_active = {
            // Try to read current state - if unavailable, assume false
            StateManager::instance()
                .try_read()
                .map(|m| m.get_core_state().is_fatigue_active())
                .unwrap_or(false)
        };

        let new_fatigue_active = if fatigue_index >= 85 {
            true
        } else if fatigue_index < 80 {
            false
        } else {
            current_fatigue_active // Keep current state in hysteresis zone
        };

        // Update state manager with all values
        // Use try_write() to avoid blocking during neurogenesis
        // If StateManager is unavailable, skip update (non-blocking)
        if let Some(state_manager) = StateManager::instance().try_write() {
            let core_state = state_manager.get_core_state();
            core_state.set_fatigue_index(fatigue_index);
            core_state.set_fatigue_active(new_fatigue_active);
            core_state.set_regular_neuron_util(regular_neuron_util);
            core_state.set_memory_neuron_util(memory_neuron_util);
            core_state.set_synapse_util(synapse_util);
        } else {
            // StateManager is locked or not ready - skip update (non-blocking)
            trace!(target: "feagi-bdu", "[FATIGUE] StateManager unavailable, skipping update");
        }

        // Update NPU's atomic boolean
        if let Some(ref npu) = self.npu {
            if let Ok(mut npu_lock) = npu.lock() {
                npu_lock.set_fatigue_active(new_fatigue_active);
            }
        }

        trace!(
            target: "feagi-bdu",
            "[FATIGUE] Index={}, Active={}, Regular={}%, Memory={}%, Synapse={}%",
            fatigue_index, new_fatigue_active, regular_neuron_util, memory_neuron_util, synapse_util
        );

        Some(fatigue_index)
    }

    // ======================================================================
    // Neuron/Synapse Creation Methods (Delegates to NPU)
    // ======================================================================

    /// Create neurons for a cortical area
    ///
    /// This delegates to the NPU's optimized batch creation function.
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Cortical area ID (6-character string)
    ///
    /// # Returns
    ///
    /// Number of neurons created
    ///
    pub fn create_neurons_for_area(&mut self, cortical_id: &CorticalID) -> BduResult<u32> {
        // Get cortical area
        let area = self
            .cortical_areas
            .get(cortical_id)
            .ok_or_else(|| {
                BduError::InvalidArea(format!("Cortical area {} not found", cortical_id))
            })?
            .clone();

        // Get cortical index
        let cortical_idx = self.cortical_id_to_idx.get(cortical_id).ok_or_else(|| {
            BduError::InvalidArea(format!("No index for cortical area {}", cortical_id))
        })?;

        // Get NPU
        let npu = self
            .npu
            .as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;

        // Extract neural parameters from area properties using CorticalAreaExt trait
        // This ensures consistent defaults across the codebase
        use crate::models::CorticalAreaExt;
        let per_voxel_cnt = area.neurons_per_voxel();
        let firing_threshold = area.firing_threshold();
        let firing_threshold_increment_x = area.firing_threshold_increment_x();
        let firing_threshold_increment_y = area.firing_threshold_increment_y();
        let firing_threshold_increment_z = area.firing_threshold_increment_z();
        // SIMD-friendly encoding: 0.0 means no limit, convert to MAX
        let firing_threshold_limit_raw = area.firing_threshold_limit();
        let firing_threshold_limit = if firing_threshold_limit_raw == 0.0 {
            f32::MAX // SIMD-friendly encoding: MAX = no limit
        } else {
            firing_threshold_limit_raw
        };

        // DEBUG: Log the increment values
        if firing_threshold_increment_x != 0.0
            || firing_threshold_increment_y != 0.0
            || firing_threshold_increment_z != 0.0
        {
            info!(
                target: "feagi-bdu",
                "🔍 [DEBUG] Area {}: firing_threshold_increment = [{}, {}, {}]",
                cortical_id.as_base_64(),
                firing_threshold_increment_x,
                firing_threshold_increment_y,
                firing_threshold_increment_z
            );
        } else {
            // Check if properties exist but are just 0
            if area.properties.contains_key("firing_threshold_increment_x")
                || area.properties.contains_key("firing_threshold_increment_y")
                || area.properties.contains_key("firing_threshold_increment_z")
            {
                info!(
                    target: "feagi-bdu",
                    "🔍 [DEBUG] Area {}: INCREMENT PROPERTIES FOUND: x={:?}, y={:?}, z={:?}",
                    cortical_id.as_base_64(),
                    area.properties.get("firing_threshold_increment_x"),
                    area.properties.get("firing_threshold_increment_y"),
                    area.properties.get("firing_threshold_increment_z")
                );
            }
        }

        let leak_coefficient = area.leak_coefficient();
        let excitability = area.neuron_excitability();
        let refractory_period = area.refractory_period();
        // SIMD-friendly encoding: 0 means no limit, convert to MAX
        let consecutive_fire_limit_raw = area.consecutive_fire_count() as u16;
        let consecutive_fire_limit = if consecutive_fire_limit_raw == 0 {
            u16::MAX // SIMD-friendly encoding: MAX = no limit
        } else {
            consecutive_fire_limit_raw
        };
        let snooze_length = area.snooze_period();
        let mp_charge_accumulation = area.mp_charge_accumulation();

        // Calculate expected neuron count for logging
        let voxels = area.dimensions.width as usize
            * area.dimensions.height as usize
            * area.dimensions.depth as usize;
        let expected_neurons = voxels * per_voxel_cnt as usize;

        trace!(
            target: "feagi-bdu",
            "Creating neurons for area {}: {}x{}x{} voxels × {} neurons/voxel = {} total neurons",
            cortical_id.as_base_64(),
            area.dimensions.width,
            area.dimensions.height,
            area.dimensions.depth,
            per_voxel_cnt,
            expected_neurons
        );

        // Call NPU to create neurons
        // NOTE: Cortical area should already be registered in NPU during corticogenesis
        let mut npu_lock = npu
            .lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;

        let neuron_count = npu_lock
            .create_cortical_area_neurons(
                *cortical_idx,
                area.dimensions.width,
                area.dimensions.height,
                area.dimensions.depth,
                per_voxel_cnt,
                firing_threshold,
                firing_threshold_increment_x,
                firing_threshold_increment_y,
                firing_threshold_increment_z,
                firing_threshold_limit,
                leak_coefficient,
                0.0, // resting_potential (LIF default)
                0,   // neuron_type (excitatory)
                refractory_period,
                excitability,
                consecutive_fire_limit,
                snooze_length,
                mp_charge_accumulation,
            )
            .map_err(|e| BduError::Internal(format!("NPU neuron creation failed: {}", e)))?;

        trace!(
            target: "feagi-bdu",
            "Created {} neurons for area {} via NPU",
            neuron_count,
            cortical_id.as_base_64()
        );

        // CRITICAL: Update per-area neuron count cache (lock-free for readers)
        // This allows healthcheck endpoints to read counts without NPU lock
        {
            let mut cache = self.cached_neuron_counts_per_area.write();
            cache
                .entry(*cortical_id)
                .or_insert_with(|| AtomicUsize::new(0))
                .store(neuron_count as usize, Ordering::Relaxed);
        }

        // Update total neuron count cache
        self.cached_neuron_count
            .fetch_add(neuron_count as usize, Ordering::Relaxed);

        // CRITICAL: Update StateManager neuron count (for health_check endpoint)
        if let Some(state_manager) = StateManager::instance().try_read() {
            let core_state = state_manager.get_core_state();
            core_state.add_neuron_count(neuron_count);
        }

        // Trigger fatigue index recalculation after neuron creation
        // NOTE: Disabled during genome loading to prevent blocking
        // Fatigue calculation will be enabled after genome loading completes
        // if neuron_count > 0 {
        //     let _ = self.update_fatigue_index();
        // }

        Ok(neuron_count)
    }

    /// Add a single neuron to a cortical area
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Cortical area ID
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `z` - Z coordinate
    /// * `firing_threshold` - Firing threshold (minimum MP to fire)
    /// * `firing_threshold_limit` - Firing threshold limit (maximum MP to fire, 0 = no limit)
    /// * `leak_coefficient` - Leak coefficient
    /// * `resting_potential` - Resting membrane potential
    /// * `neuron_type` - Neuron type (0=excitatory, 1=inhibitory)
    /// * `refractory_period` - Refractory period
    /// * `excitability` - Excitability multiplier
    /// * `consecutive_fire_limit` - Maximum consecutive fires
    /// * `snooze_length` - Snooze duration after consecutive fire limit
    /// * `mp_charge_accumulation` - Whether membrane potential accumulates
    ///
    /// # Returns
    ///
    /// The newly created neuron ID
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn add_neuron(
        &mut self,
        cortical_id: &CorticalID,
        x: u32,
        y: u32,
        z: u32,
        firing_threshold: f32,
        firing_threshold_limit: f32,
        leak_coefficient: f32,
        resting_potential: f32,
        neuron_type: u8,
        refractory_period: u16,
        excitability: f32,
        consecutive_fire_limit: u16,
        snooze_length: u16,
        mp_charge_accumulation: bool,
    ) -> BduResult<u64> {
        // Validate cortical area exists
        if !self.cortical_areas.contains_key(cortical_id) {
            return Err(BduError::InvalidArea(format!(
                "Cortical area {} not found",
                cortical_id
            )));
        }

        let cortical_idx = *self
            .cortical_id_to_idx
            .get(cortical_id)
            .ok_or_else(|| BduError::InvalidArea(format!("No index for {}", cortical_id)))?;

        // Get NPU
        let npu = self
            .npu
            .as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;

        let mut npu_lock = npu
            .lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;

        // Add neuron via NPU
        let neuron_id = npu_lock
            .add_neuron(
                firing_threshold,
                firing_threshold_limit,
                leak_coefficient,
                resting_potential,
                neuron_type as i32,
                refractory_period,
                excitability,
                consecutive_fire_limit,
                snooze_length,
                mp_charge_accumulation,
                cortical_idx,
                x,
                y,
                z,
            )
            .map_err(|e| BduError::Internal(format!("Failed to add neuron: {}", e)))?;

        trace!(
            target: "feagi-bdu",
            "Created neuron {} in area {} at ({}, {}, {})",
            neuron_id.0,
            cortical_id,
            x,
            y,
            z
        );

        // CRITICAL: Update StateManager neuron count (for health_check endpoint)
        if let Some(state_manager) = StateManager::instance().try_read() {
            let core_state = state_manager.get_core_state();
            core_state.add_neuron_count(1);
        }

        Ok(neuron_id.0 as u64)
    }

    /// Delete a neuron by ID
    ///
    /// # Arguments
    ///
    /// * `neuron_id` - Global neuron ID
    ///
    /// # Returns
    ///
    /// `true` if the neuron was deleted, `false` if it didn't exist
    ///
    pub fn delete_neuron(&mut self, neuron_id: u64) -> BduResult<bool> {
        // Get NPU
        let npu = self
            .npu
            .as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;

        let mut npu_lock = npu
            .lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;

        let deleted = npu_lock.delete_neuron(neuron_id as u32);

        if deleted {
            trace!(target: "feagi-bdu", "Deleted neuron {}", neuron_id);

            // CRITICAL: Update StateManager neuron count (for health_check endpoint)
            if let Some(state_manager) = StateManager::instance().try_read() {
                let core_state = state_manager.get_core_state();
                core_state.subtract_neuron_count(1);
            }

            // Trigger fatigue index recalculation after neuron deletion
            // NOTE: Disabled during genome loading to prevent blocking
            // let _ = self.update_fatigue_index();
        }

        Ok(deleted)
    }

    /// Apply cortical mapping rules (dstmap) to create synapses
    ///
    /// This parses the destination mapping rules from a source area and
    /// creates synapses using the NPU's synaptogenesis functions.
    ///
    /// # Arguments
    ///
    /// * `src_cortical_id` - Source cortical area ID
    ///
    /// # Returns
    ///
    /// Number of synapses created
    ///
    pub fn apply_cortical_mapping(&mut self, src_cortical_id: &CorticalID) -> BduResult<u32> {
        // Get source area
        let src_area = self
            .cortical_areas
            .get(src_cortical_id)
            .ok_or_else(|| {
                BduError::InvalidArea(format!("Source area {} not found", src_cortical_id))
            })?
            .clone();

        // Get dstmap from area properties
        let dstmap = match src_area.properties.get("cortical_mapping_dst") {
            Some(serde_json::Value::Object(map)) if !map.is_empty() => map,
            _ => return Ok(0), // No mappings
        };

        let src_cortical_idx = *self
            .cortical_id_to_idx
            .get(src_cortical_id)
            .ok_or_else(|| BduError::InvalidArea(format!("No index for {}", src_cortical_id)))?;

        let mut total_synapses = 0u32;
        let mut upstream_updates: Vec<(CorticalID, u32)> = Vec::new(); // Collect updates to apply later

        // Process each destination area using the unified path
        for (dst_cortical_id_str, _rules) in dstmap {
            // Convert string to CorticalID
            let dst_cortical_id = match CorticalID::try_from_base_64(dst_cortical_id_str) {
                Ok(id) => id,
                Err(_) => {
                    warn!(target: "feagi-bdu","Invalid cortical ID format: {}, skipping", dst_cortical_id_str);
                    continue;
                }
            };

            // Verify destination area exists
            if !self.cortical_id_to_idx.contains_key(&dst_cortical_id) {
                warn!(target: "feagi-bdu","Destination area {} not found, skipping", dst_cortical_id);
                continue;
            }

            // Apply cortical mapping for this pair (handles STDP and all morphology rules)
            let synapse_count =
                self.apply_cortical_mapping_for_pair(src_cortical_id, &dst_cortical_id)?;
            total_synapses += synapse_count as u32;

            // Queue upstream area update for ANY mapping (even if no synapses created)
            // This is critical for memory areas which have mappings but no physical synapses
            upstream_updates.push((dst_cortical_id, src_cortical_idx));
        }

        // Apply all upstream area updates now that NPU borrows are complete
        for (dst_id, src_idx) in upstream_updates {
            self.add_upstream_area(&dst_id, src_idx);
        }

        trace!(
            target: "feagi-bdu",
            "Created {} synapses for area {} via NPU",
            total_synapses,
            src_cortical_id
        );

        // CRITICAL: Update per-area synapse count cache (lock-free for readers)
        // This allows healthcheck endpoints to read counts without NPU lock
        if total_synapses > 0 {
            let mut cache = self.cached_synapse_counts_per_area.write();
            cache
                .entry(*src_cortical_id)
                .or_insert_with(|| AtomicUsize::new(0))
                .fetch_add(total_synapses as usize, Ordering::Relaxed);
        }

        // Update total synapse count cache
        self.cached_synapse_count
            .fetch_add(total_synapses as usize, Ordering::Relaxed);

        // CRITICAL: Update StateManager synapse count (for health_check endpoint)
        if total_synapses > 0 {
            if let Some(state_manager) = StateManager::instance().try_read() {
                let core_state = state_manager.get_core_state();
                core_state.add_synapse_count(total_synapses);
            }
        }

        Ok(total_synapses)
    }

    // ======================================================================
    // Neuron Query Methods (Delegates to NPU)
    // ======================================================================

    /// Check if a neuron exists
    ///
    /// # Arguments
    ///
    /// * `neuron_id` - The neuron ID to check
    ///
    /// # Returns
    ///
    /// `true` if the neuron exists in the NPU, `false` otherwise
    ///
    /// # Note
    ///
    /// Returns `false` if NPU is not connected
    ///
    pub fn has_neuron(&self, neuron_id: u64) -> bool {
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.lock() {
                // Check if neuron exists AND is valid (not deleted)
                npu_lock.is_neuron_valid(neuron_id as u32)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get total number of active neurons (lock-free cached read with opportunistic update)
    ///
    /// # Returns
    ///
    /// The total number of neurons (from cache)
    ///
    /// # Performance
    ///
    /// This is a lock-free atomic read that never blocks, even during burst processing.
    /// Opportunistically updates cache if NPU is available (non-blocking try_lock).
    ///
    pub fn get_neuron_count(&self) -> usize {
        // Opportunistically update cache if NPU is available (non-blocking)
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.try_lock() {
                let fresh_count = npu_lock.get_neuron_count();
                self.cached_neuron_count
                    .store(fresh_count, Ordering::Relaxed);
            }
            // If NPU is busy, just use cached value
        }

        // Always return cached value (never blocks)
        self.cached_neuron_count.load(Ordering::Relaxed)
    }

    /// Update the cached neuron count (explicit update)
    ///
    /// Use this if you want to force a cache update. Most callers should just
    /// use get_neuron_count() which updates opportunistically.
    ///
    pub fn update_cached_neuron_count(&self) {
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.try_lock() {
                let count = npu_lock.get_neuron_count();
                self.cached_neuron_count.store(count, Ordering::Relaxed);
            }
        }
    }

    /// Get total number of synapses (lock-free cached read with opportunistic update)
    ///
    /// # Returns
    ///
    /// The total number of synapses (from cache)
    ///
    /// # Performance
    ///
    /// This is a lock-free atomic read that never blocks, even during burst processing.
    /// Opportunistically updates cache if NPU is available (non-blocking try_lock).
    ///
    pub fn get_synapse_count(&self) -> usize {
        // Opportunistically update cache if NPU is available (non-blocking)
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.try_lock() {
                let fresh_count = npu_lock.get_synapse_count();
                self.cached_synapse_count
                    .store(fresh_count, Ordering::Relaxed);
            }
            // If NPU is busy, just use cached value
        }

        // Always return cached value (never blocks)
        self.cached_synapse_count.load(Ordering::Relaxed)
    }

    /// Update the cached synapse count (explicit update)
    ///
    /// Use this if you want to force a cache update. Most callers should just
    /// use get_synapse_count() which updates opportunistically.
    ///
    pub fn update_cached_synapse_count(&self) {
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.try_lock() {
                let count = npu_lock.get_synapse_count();
                self.cached_synapse_count.store(count, Ordering::Relaxed);
            }
        }
    }

    /// Update all cached stats (neuron and synapse counts)
    ///
    /// This is called automatically when NPU is connected and can be called
    /// explicitly if you want to force a cache refresh.
    ///
    pub fn update_all_cached_stats(&self) {
        self.update_cached_neuron_count();
        self.update_cached_synapse_count();
    }

    /// Get neuron coordinates (x, y, z)
    ///
    /// # Arguments
    ///
    /// * `neuron_id` - The neuron ID to query
    ///
    /// # Returns
    ///
    /// Coordinates as (x, y, z), or (0, 0, 0) if neuron doesn't exist or NPU not connected
    ///
    pub fn get_neuron_coordinates(&self, neuron_id: u64) -> (u32, u32, u32) {
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.lock() {
                npu_lock
                    .get_neuron_coordinates(neuron_id as u32)
                    .unwrap_or((0, 0, 0))
            } else {
                (0, 0, 0)
            }
        } else {
            (0, 0, 0)
        }
    }

    /// Get the cortical area index for a neuron
    ///
    /// # Arguments
    ///
    /// * `neuron_id` - The neuron ID to query
    ///
    /// # Returns
    ///
    /// Cortical area index, or 0 if neuron doesn't exist or NPU not connected
    ///
    pub fn get_neuron_cortical_idx(&self, neuron_id: u64) -> u32 {
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.lock() {
                npu_lock.get_neuron_cortical_area(neuron_id as u32)
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Get all neuron IDs in a specific cortical area
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - The cortical area ID (string)
    ///
    /// # Returns
    ///
    /// Vec of neuron IDs in the area, or empty vec if area doesn't exist or NPU not connected
    ///
    pub fn get_neurons_in_area(&self, cortical_id: &CorticalID) -> Vec<u64> {
        // Get cortical_idx from cortical_id
        let cortical_idx = match self.cortical_id_to_idx.get(cortical_id) {
            Some(idx) => *idx,
            None => return Vec::new(),
        };

        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.lock() {
                // Convert Vec<u32> to Vec<u64>
                npu_lock
                    .get_neurons_in_cortical_area(cortical_idx)
                    .into_iter()
                    .map(|id| id as u64)
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Get all outgoing synapses from a source neuron
    ///
    /// # Arguments
    ///
    /// * `source_neuron_id` - The source neuron ID
    ///
    /// # Returns
    ///
    /// Vec of (target_neuron_id, weight, conductance, synapse_type), or empty if NPU not connected
    ///
    pub fn get_outgoing_synapses(&self, source_neuron_id: u64) -> Vec<(u32, u8, u8, u8)> {
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.lock() {
                npu_lock.get_outgoing_synapses(source_neuron_id as u32)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Get all incoming synapses to a target neuron
    ///
    /// # Arguments
    ///
    /// * `target_neuron_id` - The target neuron ID
    ///
    /// # Returns
    ///
    /// Vec of (source_neuron_id, weight, conductance, synapse_type), or empty if NPU not connected
    ///
    pub fn get_incoming_synapses(&self, target_neuron_id: u64) -> Vec<(u32, u8, u8, u8)> {
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.lock() {
                npu_lock.get_incoming_synapses(target_neuron_id as u32)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Get neuron count for a specific cortical area
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - The cortical area ID (string)
    ///
    /// # Returns
    ///
    /// Number of neurons in the area, or 0 if area doesn't exist or NPU not connected
    ///
    /// Get neuron count for a specific cortical area (lock-free cached read)
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - The cortical area ID
    ///
    /// # Returns
    ///
    /// The number of neurons in the area (from cache, never blocks on NPU lock)
    ///
    /// # Performance
    ///
    /// This is a lock-free atomic read that never blocks, even during burst processing.
    /// Count is maintained in ConnectomeManager and updated when neurons are created/deleted.
    ///
    pub fn get_neuron_count_in_area(&self, cortical_id: &CorticalID) -> usize {
        // CRITICAL: Read from cache (lock-free) - never query NPU for healthcheck endpoints
        let cache = self.cached_neuron_counts_per_area.read();
        cache
            .get(cortical_id)
            .map(|count| count.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get all cortical areas that have neurons
    ///
    /// # Returns
    ///
    /// Vec of (cortical_id, neuron_count) for areas with at least one neuron
    ///
    pub fn get_populated_areas(&self) -> Vec<(String, usize)> {
        let mut result = Vec::new();

        for cortical_id in self.cortical_areas.keys() {
            let count = self.get_neuron_count_in_area(cortical_id);
            if count > 0 {
                result.push((cortical_id.to_string(), count));
            }
        }

        result
    }

    /// Check if a cortical area has any neurons
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - The cortical area ID
    ///
    /// # Returns
    ///
    /// `true` if the area has at least one neuron, `false` otherwise
    ///
    pub fn is_area_populated(&self, cortical_id: &CorticalID) -> bool {
        self.get_neuron_count_in_area(cortical_id) > 0
    }

    /// Get total synapse count for a specific cortical area (outgoing only) - lock-free cached read
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - The cortical area ID
    ///
    /// # Returns
    ///
    /// Total number of outgoing synapses from neurons in this area (from cache, never blocks on NPU lock)
    ///
    /// # Performance
    ///
    /// This is a lock-free atomic read that never blocks, even during burst processing.
    /// Count is maintained in ConnectomeManager and updated when synapses are created/deleted.
    ///
    pub fn get_synapse_count_in_area(&self, cortical_id: &CorticalID) -> usize {
        // CRITICAL: Read from cache (lock-free) - never query NPU for healthcheck endpoints
        let cache = self.cached_synapse_counts_per_area.read();
        cache
            .get(cortical_id)
            .map(|count| count.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Check if two neurons are connected (source → target)
    ///
    /// # Arguments
    ///
    /// * `source_neuron_id` - The source neuron ID
    /// * `target_neuron_id` - The target neuron ID
    ///
    /// # Returns
    ///
    /// `true` if there is a synapse from source to target, `false` otherwise
    ///
    pub fn are_neurons_connected(&self, source_neuron_id: u64, target_neuron_id: u64) -> bool {
        let synapses = self.get_outgoing_synapses(source_neuron_id);
        synapses
            .iter()
            .any(|(target, _, _, _)| *target == target_neuron_id as u32)
    }

    /// Get connection strength (weight) between two neurons
    ///
    /// # Arguments
    ///
    /// * `source_neuron_id` - The source neuron ID
    /// * `target_neuron_id` - The target neuron ID
    ///
    /// # Returns
    ///
    /// Synapse weight (0-255), or None if no connection exists
    ///
    pub fn get_connection_weight(
        &self,
        source_neuron_id: u64,
        target_neuron_id: u64,
    ) -> Option<u8> {
        let synapses = self.get_outgoing_synapses(source_neuron_id);
        synapses
            .iter()
            .find(|(target, _, _, _)| *target == target_neuron_id as u32)
            .map(|(_, weight, _, _)| *weight)
    }

    /// Get connectivity statistics for a cortical area
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - The cortical area ID
    ///
    /// # Returns
    ///
    /// (neuron_count, total_synapses, avg_synapses_per_neuron)
    ///
    pub fn get_area_connectivity_stats(&self, cortical_id: &CorticalID) -> (usize, usize, f32) {
        let neurons = self.get_neurons_in_area(cortical_id);
        let neuron_count = neurons.len();

        if neuron_count == 0 {
            return (0, 0, 0.0);
        }

        let mut total_synapses = 0;
        for neuron_id in neurons {
            total_synapses += self.get_outgoing_synapses(neuron_id).len();
        }

        let avg_synapses = total_synapses as f32 / neuron_count as f32;

        (neuron_count, total_synapses, avg_synapses)
    }

    /// Get the cortical area ID (string) for a neuron
    ///
    /// # Arguments
    ///
    /// * `neuron_id` - The neuron ID
    ///
    /// # Returns
    ///
    /// The cortical area ID, or None if neuron doesn't exist
    ///
    pub fn get_neuron_cortical_id(&self, neuron_id: u64) -> Option<CorticalID> {
        let cortical_idx = self.get_neuron_cortical_idx(neuron_id);
        self.cortical_idx_to_id.get(&cortical_idx).copied()
    }

    /// Get neuron density (neurons per voxel) for a cortical area
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - The cortical area ID
    ///
    /// # Returns
    ///
    /// Neuron density (neurons per voxel), or 0.0 if area doesn't exist
    ///
    pub fn get_neuron_density(&self, cortical_id: &CorticalID) -> f32 {
        let area = match self.cortical_areas.get(cortical_id) {
            Some(a) => a,
            None => return 0.0,
        };

        let neuron_count = self.get_neuron_count_in_area(cortical_id);
        let volume = area.dimensions.width * area.dimensions.height * area.dimensions.depth;

        if volume == 0 {
            return 0.0;
        }

        neuron_count as f32 / volume as f32
    }

    /// Get all cortical areas with connectivity statistics
    ///
    /// # Returns
    ///
    /// Vec of (cortical_id, neuron_count, synapse_count, density)
    ///
    pub fn get_all_area_stats(&self) -> Vec<(String, usize, usize, f32)> {
        let mut stats = Vec::new();

        for cortical_id in self.cortical_areas.keys() {
            let neuron_count = self.get_neuron_count_in_area(cortical_id);
            let synapse_count = self.get_synapse_count_in_area(cortical_id);
            let density = self.get_neuron_density(cortical_id);

            stats.push((
                cortical_id.to_string(),
                neuron_count,
                synapse_count,
                density,
            ));
        }

        stats
    }

    // ======================================================================
    // Configuration
    // ======================================================================

    /// Get the configuration
    pub fn get_config(&self) -> &ConnectomeConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: ConnectomeConfig) {
        self.config = config;
    }

    // ======================================================================
    // Genome I/O
    // ======================================================================

    /// Load a genome from JSON string
    ///
    /// This method:
    /// 1. Parses the genome JSON
    /// 2. Creates cortical areas from the blueprint
    /// 3. Reconstructs the brain region hierarchy
    /// 4. Stores neuron morphologies for later processing
    ///
    /// # Arguments
    ///
    /// * `json_str` - JSON string of the genome
    ///
    /// # Returns
    ///
    /// `Ok(())` if genome loaded successfully
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - JSON is malformed
    /// - Required fields are missing
    /// - Cortical areas have invalid data
    /// - Brain region hierarchy is invalid
    ///
    /// # Example
    ///
    /// ```ignore
    /// use feagi_brain_development::ConnectomeManager;
    ///
    /// let manager = ConnectomeManager::instance();
    /// let mut mgr = manager.write();
    ///
    /// let genome_json = r#"{ "version": "2.1", "blueprint": {...} }"#;
    /// mgr.load_genome_from_json(genome_json)?;
    /// ```
    ///
    pub fn load_genome_from_json(&mut self, json_str: &str) -> BduResult<()> {
        // Parse genome
        let parsed = feagi_evolutionary::GenomeParser::parse(json_str)?;

        info!(target: "feagi-bdu","🧬 Loading genome: {} (version {})",
            parsed.genome_title, parsed.version);
        info!(target: "feagi-bdu","🧬   Cortical areas: {}", parsed.cortical_areas.len());
        info!(target: "feagi-bdu","🧬   Brain regions: {}", parsed.brain_regions.len());

        // Clear existing data
        self.cortical_areas.clear();
        self.cortical_id_to_idx.clear();
        self.cortical_idx_to_id.clear();
        // CRITICAL: Reserve indices 0 (_death) and 1 (_power)
        self.next_cortical_idx = 3;
        info!("🔧 [BRAIN-RESET] Cortical mapping cleared, next_cortical_idx reset to 3 (reserves 0=_death, 1=_power, 2=_fatigue)");
        self.brain_regions = crate::models::BrainRegionHierarchy::new();

        // Add cortical areas
        for area in parsed.cortical_areas {
            let cortical_idx = self.add_cortical_area(area)?;
            debug!(target: "feagi-bdu","  ✅ Added cortical area {} (idx: {})",
                self.cortical_idx_to_id.get(&cortical_idx).unwrap(), cortical_idx);
        }

        // Ensure core cortical areas exist (death, power, fatigue)
        // These are required for brain operation and must always be present
        self.ensure_core_cortical_areas()?;

        // Add brain regions (hierarchy)
        for (region, parent_id) in parsed.brain_regions {
            let region_id = region.region_id;
            let parent_id_clone = parent_id.clone();
            self.brain_regions.add_region(region, parent_id_clone)?;
            debug!(target: "feagi-bdu","  ✅ Added brain region {} (parent: {:?})",
                region_id, parent_id);
        }

        self.initialized = true;

        info!(target: "feagi-bdu","🧬 ✅ Genome loaded successfully!");

        Ok(())
    }

    /// Ensure core cortical areas (_death, _power, _fatigue) exist
    ///
    /// Core areas are required for brain operation:
    /// - `_death` (cortical_idx=0): Manages neuron death and cleanup
    /// - `_power` (cortical_idx=1): Provides power injection for burst engine
    /// - `_fatigue` (cortical_idx=2): Monitors brain fatigue and triggers sleep mode
    ///
    /// If any core area is missing from the genome, it will be automatically created
    /// with default properties (1x1x1 dimensions, minimal configuration).
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all core areas exist or were successfully created
    /// * `Err(BduError)` if creation fails
    pub fn ensure_core_cortical_areas(&mut self) -> BduResult<()> {
        info!(target: "feagi-bdu", "🔧 [CORE-AREA] Ensuring core cortical areas exist...");

        use feagi_structures::genomic::cortical_area::{
            CoreCorticalType, CorticalArea, CorticalAreaDimensions, CorticalAreaType,
        };

        // Core areas are always 1x1x1 as per requirements
        let core_dimensions = CorticalAreaDimensions::new(1, 1, 1).map_err(|e| {
            BduError::Internal(format!("Failed to create core area dimensions: {}", e))
        })?;

        // Default position for core areas (origin)
        let core_position = (0, 0, 0).into();

        // Check and create _death (cortical_idx=0)
        let death_id = CoreCorticalType::Death.to_cortical_id();
        if !self.cortical_areas.contains_key(&death_id) {
            info!(target: "feagi-bdu", "🔧 [CORE-AREA] Creating missing _death area (cortical_idx=0)");
            let death_area = CorticalArea::new(
                death_id,
                0, // Will be overridden by add_cortical_area to 0
                "_death".to_string(),
                core_dimensions,
                core_position,
                CorticalAreaType::Core(CoreCorticalType::Death),
            )
            .map_err(|e| BduError::Internal(format!("Failed to create _death area: {}", e)))?;
            match self.add_cortical_area(death_area) {
                Ok(idx) => {
                    info!(target: "feagi-bdu", "  ✅ Created _death area with cortical_idx={}", idx);
                }
                Err(e) => {
                    error!(target: "feagi-bdu", "  ❌ Failed to add _death area: {}", e);
                    return Err(e);
                }
            }
        } else {
            info!(target: "feagi-bdu", "  ✓ _death area already exists");
        }

        // Check and create _power (cortical_idx=1)
        let power_id = CoreCorticalType::Power.to_cortical_id();
        if !self.cortical_areas.contains_key(&power_id) {
            info!(target: "feagi-bdu", "🔧 [CORE-AREA] Creating missing _power area (cortical_idx=1)");
            let power_area = CorticalArea::new(
                power_id,
                1, // Will be overridden by add_cortical_area to 1
                "_power".to_string(),
                core_dimensions,
                core_position,
                CorticalAreaType::Core(CoreCorticalType::Power),
            )
            .map_err(|e| BduError::Internal(format!("Failed to create _power area: {}", e)))?;
            match self.add_cortical_area(power_area) {
                Ok(idx) => {
                    info!(target: "feagi-bdu", "  ✅ Created _power area with cortical_idx={}", idx);
                }
                Err(e) => {
                    error!(target: "feagi-bdu", "  ❌ Failed to add _power area: {}", e);
                    return Err(e);
                }
            }
        } else {
            info!(target: "feagi-bdu", "  ✓ _power area already exists");
        }

        // Check and create _fatigue (cortical_idx=2)
        let fatigue_id = CoreCorticalType::Fatigue.to_cortical_id();
        if !self.cortical_areas.contains_key(&fatigue_id) {
            info!(target: "feagi-bdu", "🔧 [CORE-AREA] Creating missing _fatigue area (cortical_idx=2)");
            let fatigue_area = CorticalArea::new(
                fatigue_id,
                2, // Will be overridden by add_cortical_area to 2
                "_fatigue".to_string(),
                core_dimensions,
                core_position,
                CorticalAreaType::Core(CoreCorticalType::Fatigue),
            )
            .map_err(|e| BduError::Internal(format!("Failed to create _fatigue area: {}", e)))?;
            match self.add_cortical_area(fatigue_area) {
                Ok(idx) => {
                    info!(target: "feagi-bdu", "  ✅ Created _fatigue area with cortical_idx={}", idx);
                }
                Err(e) => {
                    error!(target: "feagi-bdu", "  ❌ Failed to add _fatigue area: {}", e);
                    return Err(e);
                }
            }
        } else {
            info!(target: "feagi-bdu", "  ✓ _fatigue area already exists");
        }

        info!(target: "feagi-bdu", "🔧 [CORE-AREA] Core area check complete");
        Ok(())
    }

    /// Save the connectome as a genome JSON
    ///
    /// **DEPRECATED**: This method produces incomplete hierarchical format v2.1 without morphologies/physiology.
    /// Use `GenomeService::save_genome()` instead, which produces complete flat format v3.0.
    ///
    /// This method is kept only for legacy tests. Production code MUST use GenomeService.
    ///
    /// # Arguments
    ///
    /// * `genome_id` - Optional custom genome ID (generates timestamp-based ID if None)
    /// * `genome_title` - Optional custom genome title
    ///
    /// # Returns
    ///
    /// JSON string representation of the genome (hierarchical v2.1, incomplete)
    ///
    #[deprecated(
        note = "Use GenomeService::save_genome() instead. This produces incomplete v2.1 format without morphologies/physiology."
    )]
    #[allow(deprecated)]
    pub fn save_genome_to_json(
        &self,
        genome_id: Option<String>,
        genome_title: Option<String>,
    ) -> BduResult<String> {
        // Build parent map from brain region hierarchy
        let mut brain_regions_with_parents = std::collections::HashMap::new();

        for region_id in self.brain_regions.get_all_region_ids() {
            if let Some(region) = self.brain_regions.get_region(region_id) {
                let parent_id = self
                    .brain_regions
                    .get_parent(region_id)
                    .map(|s| s.to_string());
                brain_regions_with_parents
                    .insert(region_id.to_string(), (region.clone(), parent_id));
            }
        }

        // Generate and return JSON
        Ok(feagi_evolutionary::GenomeSaver::save_to_json(
            &self.cortical_areas,
            &brain_regions_with_parents,
            genome_id,
            genome_title,
        )?)
    }

    // Load genome from file and develop brain
    //
    // This was a high-level convenience method that:
    // 1. Loads genome from JSON file
    // 2. Prepares for new genome (clears existing state)
    // 3. Runs neuroembryogenesis to develop the brain
    //
    // # Arguments
    //
    // * `genome_path` - Path to genome JSON file
    //
    // # Returns
    //
    // Development progress information
    //
    // NOTE: load_from_genome_file() and load_from_genome() have been REMOVED.
    // All genome loading must now go through GenomeService::load_genome() which:
    // - Stores RuntimeGenome for persistence
    // - Updates genome metadata
    // - Provides async/await support
    // - Includes timeout protection
    // - Ensures core cortical areas exist
    //
    // See: feagi-services/src/impls/genome_service_impl.rs::load_genome()

    /// Prepare for loading a new genome
    ///
    /// Clears all existing cortical areas, brain regions, and resets state.
    /// This is typically called before loading a new genome.
    ///
    pub fn prepare_for_new_genome(&mut self) -> BduResult<()> {
        info!(target: "feagi-bdu","Preparing for new genome (clearing existing state)");

        // Clear cortical areas
        self.cortical_areas.clear();
        self.cortical_id_to_idx.clear();
        self.cortical_idx_to_id.clear();
        // CRITICAL: Reserve indices 0 (_death) and 1 (_power)
        self.next_cortical_idx = 3;
        info!("🔧 [BRAIN-RESET] Cortical mapping cleared, next_cortical_idx reset to 3 (reserves 0=_death, 1=_power, 2=_fatigue)");

        // Clear brain regions
        self.brain_regions = BrainRegionHierarchy::new();

        // Reset NPU if present
        // TODO: Add reset() method to RustNPU
        // if let Some(ref npu) = self.npu {
        //     let mut npu_lock = npu.lock().unwrap();
        //     npu_lock.reset();
        // }

        info!(target: "feagi-bdu","✅ Connectome cleared and ready for new genome");
        Ok(())
    }

    /// Calculate and resize memory for a genome
    ///
    /// Analyzes the genome to determine memory requirements and
    /// prepares the NPU for the expected neuron/synapse counts.
    ///
    /// # Arguments
    ///
    /// * `genome` - Genome to analyze for memory requirements
    ///
    pub fn resize_for_genome(
        &mut self,
        genome: &feagi_evolutionary::RuntimeGenome,
    ) -> BduResult<()> {
        // Store morphologies from genome
        self.morphology_registry = genome.morphologies.clone();
        info!(target: "feagi-bdu", "Stored {} morphologies from genome", self.morphology_registry.count());

        // Calculate required capacity from genome stats
        let required_neurons = genome.stats.innate_neuron_count;
        let required_synapses = genome.stats.innate_synapse_count;

        info!(target: "feagi-bdu",
            "Genome requires: {} neurons, {} synapses",
            required_neurons,
            required_synapses
        );

        // Calculate total voxels from all cortical areas
        let mut total_voxels = 0;
        for area in genome.cortical_areas.values() {
            total_voxels += area.dimensions.width * area.dimensions.height * area.dimensions.depth;
        }

        info!(target: "feagi-bdu",
            "Genome has {} cortical areas with {} total voxels",
            genome.cortical_areas.len(),
            total_voxels
        );

        // TODO: Resize NPU if needed
        // For now, we assume NPU has sufficient capacity
        // In the future, we may want to dynamically resize the NPU based on genome requirements

        Ok(())
    }

    // ========================================================================
    // SYNAPSE OPERATIONS
    // ========================================================================

    /// Create a synapse between two neurons
    ///
    /// # Arguments
    ///
    /// * `source_neuron_id` - Source neuron ID
    /// * `target_neuron_id` - Target neuron ID
    /// * `weight` - Synapse weight (0-255)
    /// * `conductance` - Synapse conductance (0-255)
    /// * `synapse_type` - Synapse type (0=excitatory, 1=inhibitory)
    ///
    /// # Returns
    ///
    /// `Ok(())` if synapse created successfully
    ///
    pub fn create_synapse(
        &mut self,
        source_neuron_id: u64,
        target_neuron_id: u64,
        weight: u8,
        conductance: u8,
        synapse_type: u8,
    ) -> BduResult<()> {
        // Get NPU
        let npu = self
            .npu
            .as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;

        let mut npu_lock = npu
            .lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;

        // Verify both neurons exist
        let source_exists = (source_neuron_id as u32) < npu_lock.get_neuron_count() as u32;
        let target_exists = (target_neuron_id as u32) < npu_lock.get_neuron_count() as u32;

        if !source_exists {
            return Err(BduError::InvalidNeuron(format!(
                "Source neuron {} not found",
                source_neuron_id
            )));
        }
        if !target_exists {
            return Err(BduError::InvalidNeuron(format!(
                "Target neuron {} not found",
                target_neuron_id
            )));
        }

        // Create synapse via NPU
        let syn_type = if synapse_type == 0 {
            feagi_npu_neural::synapse::SynapseType::Excitatory
        } else {
            feagi_npu_neural::synapse::SynapseType::Inhibitory
        };

        let synapse_idx = npu_lock
            .add_synapse(
                NeuronId(source_neuron_id as u32),
                NeuronId(target_neuron_id as u32),
                feagi_npu_neural::types::SynapticWeight(weight),
                feagi_npu_neural::types::SynapticConductance(conductance),
                syn_type,
            )
            .map_err(|e| BduError::Internal(format!("Failed to create synapse: {}", e)))?;

        debug!(target: "feagi-bdu", "Created synapse: {} -> {} (weight: {}, conductance: {}, type: {}, idx: {})",
            source_neuron_id, target_neuron_id, weight, conductance, synapse_type, synapse_idx);

        // Trigger fatigue index recalculation after synapse creation
        // NOTE: Disabled during genome loading to prevent blocking
        // let _ = self.update_fatigue_index();

        Ok(())
    }

    /// Synchronize cortical area flags with NPU
    /// This should be called after adding/updating cortical areas
    fn sync_cortical_area_flags_to_npu(&mut self) -> BduResult<()> {
        if let Some(ref npu) = self.npu {
            if let Ok(mut npu_lock) = npu.lock() {
                // Build psp_uniform_distribution flags map
                let mut psp_uniform_flags = ahash::AHashMap::new();
                let mut mp_driven_psp_flags = ahash::AHashMap::new();

                for (cortical_id, area) in &self.cortical_areas {
                    // Get psp_uniform_distribution flag (default to false)
                    let psp_uniform = area
                        .get_property("psp_uniform_distribution")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    psp_uniform_flags.insert(*cortical_id, psp_uniform);

                    // Get mp_driven_psp flag (default to false)
                    let mp_driven_psp = area
                        .get_property("mp_driven_psp")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    mp_driven_psp_flags.insert(*cortical_id, mp_driven_psp);
                }

                // Update NPU with flags
                npu_lock.set_psp_uniform_distribution_flags(psp_uniform_flags);
                npu_lock.set_mp_driven_psp_flags(mp_driven_psp_flags);

                trace!(
                    target: "feagi-bdu",
                    "Synchronized cortical area flags to NPU ({} areas)",
                    self.cortical_areas.len()
                );
            }
        }

        Ok(())
    }

    /// Get synapse information between two neurons
    ///
    /// # Arguments
    ///
    /// * `source_neuron_id` - Source neuron ID
    /// * `target_neuron_id` - Target neuron ID
    ///
    /// # Returns
    ///
    /// `Some((weight, conductance, type))` if synapse exists, `None` otherwise
    ///
    pub fn get_synapse(
        &self,
        source_neuron_id: u64,
        target_neuron_id: u64,
    ) -> Option<(u8, u8, u8)> {
        // Get NPU
        let npu = self.npu.as_ref()?;
        let npu_lock = npu.lock().ok()?;

        // Use get_incoming_synapses and filter by source
        // (This does O(n) scan of synapse_array, but works even when propagation engine isn't updated)
        let incoming = npu_lock.get_incoming_synapses(target_neuron_id as u32);

        // Find the synapse from our specific source
        for (source_id, weight, conductance, synapse_type) in incoming {
            if source_id == source_neuron_id as u32 {
                return Some((weight, conductance, synapse_type));
            }
        }

        None
    }

    /// Update the weight of an existing synapse
    ///
    /// # Arguments
    ///
    /// * `source_neuron_id` - Source neuron ID
    /// * `target_neuron_id` - Target neuron ID
    /// * `new_weight` - New synapse weight (0-255)
    ///
    /// # Returns
    ///
    /// `Ok(())` if synapse updated, `Err` if synapse not found
    ///
    pub fn update_synapse_weight(
        &mut self,
        source_neuron_id: u64,
        target_neuron_id: u64,
        new_weight: u8,
    ) -> BduResult<()> {
        // Get NPU
        let npu = self
            .npu
            .as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;

        let mut npu_lock = npu
            .lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;

        // Update synapse weight via NPU
        let updated = npu_lock.update_synapse_weight(
            NeuronId(source_neuron_id as u32),
            NeuronId(target_neuron_id as u32),
            feagi_npu_neural::types::SynapticWeight(new_weight),
        );

        if updated {
            debug!(target: "feagi-bdu","Updated synapse weight: {} -> {} = {}", source_neuron_id, target_neuron_id, new_weight);
            Ok(())
        } else {
            Err(BduError::InvalidSynapse(format!(
                "Synapse {} -> {} not found",
                source_neuron_id, target_neuron_id
            )))
        }
    }

    /// Remove a synapse between two neurons
    ///
    /// # Arguments
    ///
    /// * `source_neuron_id` - Source neuron ID
    /// * `target_neuron_id` - Target neuron ID
    ///
    /// # Returns
    ///
    /// `Ok(true)` if synapse removed, `Ok(false)` if synapse didn't exist
    ///
    pub fn remove_synapse(
        &mut self,
        source_neuron_id: u64,
        target_neuron_id: u64,
    ) -> BduResult<bool> {
        // Get NPU
        let npu = self
            .npu
            .as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;

        let mut npu_lock = npu
            .lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;

        // Remove synapse via NPU
        let removed = npu_lock.remove_synapse(
            NeuronId(source_neuron_id as u32),
            NeuronId(target_neuron_id as u32),
        );

        if removed {
            debug!(target: "feagi-bdu","Removed synapse: {} -> {}", source_neuron_id, target_neuron_id);

            // CRITICAL: Update StateManager synapse count (for health_check endpoint)
            if let Some(state_manager) = StateManager::instance().try_read() {
                let core_state = state_manager.get_core_state();
                core_state.subtract_synapse_count(1);
            }
        }

        Ok(removed)
    }

    // ========================================================================
    // BATCH OPERATIONS
    // ========================================================================

    /// Batch create multiple neurons at once (SIMD-optimized)
    ///
    /// This is significantly faster than calling `add_neuron()` in a loop
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Target cortical area
    /// * `neurons` - Vector of neuron parameters (x, y, z, firing_threshold, leak, resting_potential, etc.)
    ///
    /// # Returns
    ///
    /// Vector of created neuron IDs
    ///
    pub fn batch_create_neurons(
        &mut self,
        cortical_id: &CorticalID,
        neurons: Vec<NeuronData>,
    ) -> BduResult<Vec<u64>> {
        // Get NPU
        let npu = self
            .npu
            .as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;

        let mut npu_lock = npu
            .lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;

        // Get cortical area to verify it exists and get its index
        let area = self.get_cortical_area(cortical_id).ok_or_else(|| {
            BduError::InvalidArea(format!("Cortical area {} not found", cortical_id))
        })?;
        let cortical_idx = area.cortical_idx;

        let count = neurons.len();

        // Extract parameters into separate vectors for batch operation
        let mut x_coords = Vec::with_capacity(count);
        let mut y_coords = Vec::with_capacity(count);
        let mut z_coords = Vec::with_capacity(count);
        let mut firing_thresholds = Vec::with_capacity(count);
        let mut threshold_limits = Vec::with_capacity(count);
        let mut leak_coeffs = Vec::with_capacity(count);
        let mut resting_potentials = Vec::with_capacity(count);
        let mut neuron_types = Vec::with_capacity(count);
        let mut refractory_periods = Vec::with_capacity(count);
        let mut excitabilities = Vec::with_capacity(count);
        let mut consec_fire_limits = Vec::with_capacity(count);
        let mut snooze_lengths = Vec::with_capacity(count);
        let mut mp_accums = Vec::with_capacity(count);
        let mut cortical_areas = Vec::with_capacity(count);

        for (
            x,
            y,
            z,
            threshold,
            threshold_limit,
            leak,
            resting,
            ntype,
            refract,
            excit,
            consec_limit,
            snooze,
            mp_accum,
        ) in neurons
        {
            x_coords.push(x);
            y_coords.push(y);
            z_coords.push(z);
            firing_thresholds.push(threshold);
            threshold_limits.push(threshold_limit);
            leak_coeffs.push(leak);
            resting_potentials.push(resting);
            neuron_types.push(ntype);
            refractory_periods.push(refract);
            excitabilities.push(excit);
            consec_fire_limits.push(consec_limit);
            snooze_lengths.push(snooze);
            mp_accums.push(mp_accum);
            cortical_areas.push(cortical_idx);
        }

        // Get the current neuron count - this will be the first ID of our batch
        let first_neuron_id = npu_lock.get_neuron_count() as u32;

        // Call NPU batch creation (SIMD-optimized)
        // Signature: (thresholds, threshold_limits, leak_coeffs, resting_pots, neuron_types, refract, excit, consec_limits, snooze, mp_accums, cortical_areas, x, y, z)
        // Convert f32 vectors to T
        // DynamicNPU will handle f32 inputs and convert internally based on its precision
        let firing_thresholds_t = firing_thresholds;
        let threshold_limits_t = threshold_limits;
        let resting_potentials_t = resting_potentials;
        let (neurons_created, _indices) = npu_lock.add_neurons_batch(
            firing_thresholds_t,
            threshold_limits_t,
            leak_coeffs,
            resting_potentials_t,
            neuron_types,
            refractory_periods,
            excitabilities,
            consec_fire_limits,
            snooze_lengths,
            mp_accums,
            cortical_areas,
            x_coords,
            y_coords,
            z_coords,
        );

        // Generate neuron IDs (they are sequential starting from first_neuron_id)
        let mut neuron_ids = Vec::with_capacity(count);
        for i in 0..neurons_created {
            neuron_ids.push((first_neuron_id + i) as u64);
        }

        info!(target: "feagi-bdu","Batch created {} neurons in cortical area {}", count, cortical_id);

        // CRITICAL: Update StateManager neuron count (for health_check endpoint)
        if let Some(state_manager) = StateManager::instance().try_read() {
            let core_state = state_manager.get_core_state();
            core_state.add_neuron_count(neurons_created);
        }

        Ok(neuron_ids)
    }

    /// Delete multiple neurons at once (batch operation)
    ///
    /// # Arguments
    ///
    /// * `neuron_ids` - Vector of neuron IDs to delete
    ///
    /// # Returns
    ///
    /// Number of neurons actually deleted
    ///
    pub fn delete_neurons_batch(&mut self, neuron_ids: Vec<u64>) -> BduResult<usize> {
        // Get NPU
        let npu = self
            .npu
            .as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;

        let mut npu_lock = npu
            .lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;

        let mut deleted_count = 0;

        // Delete each neuron
        // Note: Could be optimized with a batch delete method in NPU if needed
        for neuron_id in neuron_ids {
            if npu_lock.delete_neuron(neuron_id as u32) {
                deleted_count += 1;
            }
        }

        info!(target: "feagi-bdu","Batch deleted {} neurons", deleted_count);

        // CRITICAL: Update StateManager neuron count (for health_check endpoint)
        if deleted_count > 0 {
            if let Some(state_manager) = StateManager::instance().try_read() {
                let core_state = state_manager.get_core_state();
                core_state.subtract_neuron_count(deleted_count as u32);
            }
        }

        // Trigger fatigue index recalculation after batch neuron deletion
        // NOTE: Disabled during genome loading to prevent blocking
        // if deleted_count > 0 {
        //     let _ = self.update_fatigue_index();
        // }

        Ok(deleted_count)
    }

    // ========================================================================
    // NEURON UPDATE OPERATIONS
    // ========================================================================

    /// Update properties of an existing neuron
    ///
    /// # Arguments
    ///
    /// * `neuron_id` - Target neuron ID
    /// * `firing_threshold` - Optional new firing threshold
    /// * `leak_coefficient` - Optional new leak coefficient
    /// * `resting_potential` - Optional new resting potential
    /// * `excitability` - Optional new excitability
    ///
    /// # Returns
    ///
    /// `Ok(())` if neuron updated successfully
    ///
    pub fn update_neuron_properties(
        &mut self,
        neuron_id: u64,
        firing_threshold: Option<f32>,
        leak_coefficient: Option<f32>,
        resting_potential: Option<f32>,
        excitability: Option<f32>,
    ) -> BduResult<()> {
        // Get NPU
        let npu = self
            .npu
            .as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;

        let mut npu_lock = npu
            .lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;

        let neuron_id_u32 = neuron_id as u32;

        // Verify neuron exists by trying to update at least one property
        let mut updated = false;

        // Update properties if provided
        if let Some(threshold) = firing_threshold {
            if npu_lock.update_neuron_threshold(neuron_id_u32, threshold) {
                updated = true;
                debug!(target: "feagi-bdu","Updated neuron {} firing_threshold = {}", neuron_id, threshold);
            } else if !updated {
                return Err(BduError::InvalidNeuron(format!(
                    "Neuron {} not found",
                    neuron_id
                )));
            }
        }

        if let Some(leak) = leak_coefficient {
            if npu_lock.update_neuron_leak(neuron_id_u32, leak) {
                updated = true;
                debug!(target: "feagi-bdu","Updated neuron {} leak_coefficient = {}", neuron_id, leak);
            } else if !updated {
                return Err(BduError::InvalidNeuron(format!(
                    "Neuron {} not found",
                    neuron_id
                )));
            }
        }

        if let Some(resting) = resting_potential {
            if npu_lock.update_neuron_resting_potential(neuron_id_u32, resting) {
                updated = true;
                debug!(target: "feagi-bdu","Updated neuron {} resting_potential = {}", neuron_id, resting);
            } else if !updated {
                return Err(BduError::InvalidNeuron(format!(
                    "Neuron {} not found",
                    neuron_id
                )));
            }
        }

        if let Some(excit) = excitability {
            if npu_lock.update_neuron_excitability(neuron_id_u32, excit) {
                updated = true;
                debug!(target: "feagi-bdu","Updated neuron {} excitability = {}", neuron_id, excit);
            } else if !updated {
                return Err(BduError::InvalidNeuron(format!(
                    "Neuron {} not found",
                    neuron_id
                )));
            }
        }

        if !updated {
            return Err(BduError::Internal(
                "No properties provided for update".to_string(),
            ));
        }

        info!(target: "feagi-bdu","Updated properties for neuron {}", neuron_id);

        Ok(())
    }

    /// Update the firing threshold of a specific neuron
    ///
    /// # Arguments
    ///
    /// * `neuron_id` - Target neuron ID
    /// * `new_threshold` - New firing threshold value
    ///
    /// # Returns
    ///
    /// `Ok(())` if threshold updated successfully
    ///
    pub fn set_neuron_firing_threshold(
        &mut self,
        neuron_id: u64,
        new_threshold: f32,
    ) -> BduResult<()> {
        // Get NPU
        let npu = self
            .npu
            .as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;

        let mut npu_lock = npu
            .lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;

        // Update threshold via NPU
        if npu_lock.update_neuron_threshold(neuron_id as u32, new_threshold) {
            debug!(target: "feagi-bdu","Set neuron {} firing threshold = {}", neuron_id, new_threshold);
            Ok(())
        } else {
            Err(BduError::InvalidNeuron(format!(
                "Neuron {} not found",
                neuron_id
            )))
        }
    }

    // ========================================================================
    // AREA MANAGEMENT & QUERIES
    // ========================================================================

    /// Get cortical area by name (alternative to ID lookup)
    ///
    /// # Arguments
    ///
    /// * `name` - Human-readable area name
    ///
    /// # Returns
    ///
    /// `Some(CorticalArea)` if found, `None` otherwise
    ///
    pub fn get_cortical_area_by_name(&self, name: &str) -> Option<CorticalArea> {
        self.cortical_areas
            .values()
            .find(|area| area.name == name)
            .cloned()
    }

    /// Resize a cortical area (changes dimensions, may require neuron reallocation)
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Target cortical area ID
    /// * `new_dimensions` - New dimensions (width, height, depth)
    ///
    /// # Returns
    ///
    /// `Ok(())` if resized successfully
    ///
    /// # Note
    ///
    /// This does NOT automatically create/delete neurons. It only updates metadata.
    /// Caller must handle neuron population separately.
    ///
    pub fn resize_cortical_area(
        &mut self,
        cortical_id: &CorticalID,
        new_dimensions: CorticalAreaDimensions,
    ) -> BduResult<()> {
        // Validate dimensions
        if new_dimensions.width == 0 || new_dimensions.height == 0 || new_dimensions.depth == 0 {
            return Err(BduError::InvalidArea(format!(
                "Invalid dimensions: {:?} (all must be > 0)",
                new_dimensions
            )));
        }

        // Get and update area
        let area = self.cortical_areas.get_mut(cortical_id).ok_or_else(|| {
            BduError::InvalidArea(format!("Cortical area {} not found", cortical_id))
        })?;

        let old_dimensions = area.dimensions;
        area.dimensions = new_dimensions;

        // Note: Visualization voxel granularity is user-driven, not recalculated on resize
        // If user had set a custom value, it remains; otherwise defaults to 1x1x1

        info!(target: "feagi-bdu",
            "Resized cortical area {} from {:?} to {:?}",
            cortical_id,
            old_dimensions,
            new_dimensions
        );

        self.refresh_cortical_area_hashes(false, true);

        Ok(())
    }

    /// Get all cortical areas in a brain region
    ///
    /// # Arguments
    ///
    /// * `region_id` - Brain region ID
    ///
    /// # Returns
    ///
    /// Vector of cortical area IDs in the region
    ///
    pub fn get_areas_in_region(&self, region_id: &str) -> BduResult<Vec<String>> {
        let region = self.brain_regions.get_region(region_id).ok_or_else(|| {
            BduError::InvalidArea(format!("Brain region {} not found", region_id))
        })?;

        // Convert CorticalID to base64 strings
        Ok(region
            .cortical_areas
            .iter()
            .map(|id| id.as_base_64())
            .collect())
    }

    /// Update brain region properties
    ///
    /// # Arguments
    ///
    /// * `region_id` - Target region ID
    /// * `new_name` - Optional new name
    /// * `new_description` - Optional new description
    ///
    /// # Returns
    ///
    /// `Ok(())` if updated successfully
    ///
    pub fn update_brain_region(
        &mut self,
        region_id: &str,
        new_name: Option<String>,
        new_description: Option<String>,
    ) -> BduResult<()> {
        let region = self
            .brain_regions
            .get_region_mut(region_id)
            .ok_or_else(|| {
                BduError::InvalidArea(format!("Brain region {} not found", region_id))
            })?;

        if let Some(name) = new_name {
            region.name = name;
            debug!(target: "feagi-bdu","Updated brain region {} name", region_id);
        }

        if let Some(desc) = new_description {
            // BrainRegion doesn't have a description field in the struct, so we'll store it in properties
            region
                .properties
                .insert("description".to_string(), serde_json::json!(desc));
            debug!(target: "feagi-bdu","Updated brain region {} description", region_id);
        }

        info!(target: "feagi-bdu","Updated brain region {}", region_id);

        self.refresh_brain_regions_hash();

        Ok(())
    }

    /// Update brain region properties with generic property map
    ///
    /// Supports updating any brain region property including coordinates, title, description, etc.
    ///
    /// # Arguments
    ///
    /// * `region_id` - Target region ID
    /// * `properties` - Map of property names to new values
    ///
    /// # Returns
    ///
    /// `Ok(())` if updated successfully
    ///
    pub fn update_brain_region_properties(
        &mut self,
        region_id: &str,
        properties: std::collections::HashMap<String, serde_json::Value>,
    ) -> BduResult<()> {
        use tracing::{debug, info};

        let region = self
            .brain_regions
            .get_region_mut(region_id)
            .ok_or_else(|| {
                BduError::InvalidArea(format!("Brain region {} not found", region_id))
            })?;

        for (key, value) in properties {
            match key.as_str() {
                "title" | "name" => {
                    if let Some(name) = value.as_str() {
                        region.name = name.to_string();
                        debug!(target: "feagi-bdu", "Updated brain region {} name = {}", region_id, name);
                    }
                }
                "coordinate_3d" | "coordinates_3d" => {
                    region
                        .properties
                        .insert("coordinate_3d".to_string(), value.clone());
                    debug!(target: "feagi-bdu", "Updated brain region {} coordinate_3d = {:?}", region_id, value);
                }
                "coordinate_2d" | "coordinates_2d" => {
                    region
                        .properties
                        .insert("coordinate_2d".to_string(), value.clone());
                    debug!(target: "feagi-bdu", "Updated brain region {} coordinate_2d = {:?}", region_id, value);
                }
                "description" => {
                    region
                        .properties
                        .insert("description".to_string(), value.clone());
                    debug!(target: "feagi-bdu", "Updated brain region {} description", region_id);
                }
                "region_type" => {
                    if let Some(type_str) = value.as_str() {
                        // Note: RegionType is currently a placeholder (Undefined only)
                        // Specific region types will be added in the future
                        region.region_type = feagi_structures::genomic::RegionType::Undefined;
                        debug!(target: "feagi-bdu", "Updated brain region {} type = {}", region_id, type_str);
                    }
                }
                // Store any other properties in the properties map
                _ => {
                    region.properties.insert(key.clone(), value.clone());
                    debug!(target: "feagi-bdu", "Updated brain region {} property {} = {:?}", region_id, key, value);
                }
            }
        }

        info!(target: "feagi-bdu", "Updated brain region {} properties", region_id);

        Ok(())
    }

    // ========================================================================
    // NEURON QUERY METHODS (P6)
    // ========================================================================

    /// Get neuron by 3D coordinates within a cortical area
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Cortical area ID
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `z` - Z coordinate
    ///
    /// # Returns
    ///
    /// `Some(neuron_id)` if found, `None` otherwise
    ///
    pub fn get_neuron_by_coordinates(
        &self,
        cortical_id: &CorticalID,
        x: u32,
        y: u32,
        z: u32,
    ) -> Option<u64> {
        // Get cortical area to get its index
        let area = self.get_cortical_area(cortical_id)?;
        let cortical_idx = area.cortical_idx;

        // Query NPU via public method
        let npu = self.npu.as_ref()?;
        let npu_lock = npu.lock().ok()?;

        npu_lock
            .get_neuron_id_at_coordinate(cortical_idx, x, y, z)
            .map(|id| id as u64)
    }

    /// Get the position (coordinates) of a neuron
    ///
    /// # Arguments
    ///
    /// * `neuron_id` - Neuron ID
    ///
    /// # Returns
    ///
    /// `Some((x, y, z))` if found, `None` otherwise
    ///
    pub fn get_neuron_position(&self, neuron_id: u64) -> Option<(u32, u32, u32)> {
        let npu = self.npu.as_ref()?;
        let npu_lock = npu.lock().ok()?;

        // Verify neuron exists and get coordinates
        let neuron_count = npu_lock.get_neuron_count();
        if (neuron_id as usize) >= neuron_count {
            return None;
        }

        Some(
            npu_lock
                .get_neuron_coordinates(neuron_id as u32)
                .unwrap_or((0, 0, 0)),
        )
    }

    /// Get which cortical area contains a specific neuron
    ///
    /// # Arguments
    ///
    /// * `neuron_id` - Neuron ID
    ///
    /// # Returns
    ///
    /// `Some(cortical_id)` if found, `None` otherwise
    ///
    pub fn get_cortical_area_for_neuron(&self, neuron_id: u64) -> Option<CorticalID> {
        let npu = self.npu.as_ref()?;
        let npu_lock = npu.lock().ok()?;

        // Verify neuron exists
        let neuron_count = npu_lock.get_neuron_count();
        if (neuron_id as usize) >= neuron_count {
            return None;
        }

        let cortical_idx = npu_lock.get_neuron_cortical_area(neuron_id as u32);

        // Look up cortical_id from index
        self.cortical_areas
            .values()
            .find(|area| area.cortical_idx == cortical_idx)
            .map(|area| area.cortical_id)
    }

    /// Get all properties of a neuron
    ///
    /// # Arguments
    ///
    /// * `neuron_id` - Neuron ID
    ///
    /// # Returns
    ///
    /// `Some(properties)` if found, `None` otherwise
    ///
    pub fn get_neuron_properties(
        &self,
        neuron_id: u64,
    ) -> Option<std::collections::HashMap<String, serde_json::Value>> {
        let npu = self.npu.as_ref()?;
        let npu_lock = npu.lock().ok()?;

        let neuron_id_u32 = neuron_id as u32;
        let idx = neuron_id as usize;

        // Verify neuron exists
        let neuron_count = npu_lock.get_neuron_count();
        if idx >= neuron_count {
            return None;
        }

        let mut properties = std::collections::HashMap::new();

        // Basic info
        properties.insert("neuron_id".to_string(), serde_json::json!(neuron_id));

        // Get coordinates
        let (x, y, z) = npu_lock.get_neuron_coordinates(neuron_id_u32)?;
        properties.insert("x".to_string(), serde_json::json!(x));
        properties.insert("y".to_string(), serde_json::json!(y));
        properties.insert("z".to_string(), serde_json::json!(z));

        // Get cortical area
        let cortical_idx = npu_lock.get_neuron_cortical_area(neuron_id_u32);
        properties.insert("cortical_area".to_string(), serde_json::json!(cortical_idx));

        // Get neuron state (returns: consecutive_fire_count, consecutive_fire_limit, snooze_period, membrane_potential, threshold, refractory_countdown)
        if let Some((consec_count, consec_limit, snooze, mp, threshold, refract_countdown)) =
            npu_lock.get_neuron_state(NeuronId(neuron_id_u32))
        {
            properties.insert(
                "consecutive_fire_count".to_string(),
                serde_json::json!(consec_count),
            );
            properties.insert(
                "consecutive_fire_limit".to_string(),
                serde_json::json!(consec_limit),
            );
            properties.insert("snooze_period".to_string(), serde_json::json!(snooze));
            properties.insert("membrane_potential".to_string(), serde_json::json!(mp));
            properties.insert("threshold".to_string(), serde_json::json!(threshold));
            properties.insert(
                "refractory_countdown".to_string(),
                serde_json::json!(refract_countdown),
            );
        }

        // Get other properties via get_neuron_property_by_index
        if let Some(leak) = npu_lock.get_neuron_property_by_index(idx, "leak_coefficient") {
            properties.insert("leak_coefficient".to_string(), serde_json::json!(leak));
        }
        if let Some(resting) = npu_lock.get_neuron_property_by_index(idx, "resting_potential") {
            properties.insert("resting_potential".to_string(), serde_json::json!(resting));
        }
        if let Some(excit) = npu_lock.get_neuron_property_by_index(idx, "excitability") {
            properties.insert("excitability".to_string(), serde_json::json!(excit));
        }
        if let Some(threshold_limit) = npu_lock.get_neuron_property_by_index(idx, "threshold_limit")
        {
            properties.insert(
                "threshold_limit".to_string(),
                serde_json::json!(threshold_limit),
            );
        }

        // Get u16 properties
        if let Some(refract_period) =
            npu_lock.get_neuron_property_u16_by_index(idx, "refractory_period")
        {
            properties.insert(
                "refractory_period".to_string(),
                serde_json::json!(refract_period),
            );
        }

        Some(properties)
    }

    /// Get a specific property of a neuron
    ///
    /// # Arguments
    ///
    /// * `neuron_id` - Neuron ID
    /// * `property_name` - Name of the property to retrieve
    ///
    /// # Returns
    ///
    /// `Some(value)` if found, `None` otherwise
    ///
    pub fn get_neuron_property(
        &self,
        neuron_id: u64,
        property_name: &str,
    ) -> Option<serde_json::Value> {
        self.get_neuron_properties(neuron_id)?
            .get(property_name)
            .cloned()
    }

    // ========================================================================
    // CORTICAL AREA LIST/QUERY METHODS (P6)
    // ========================================================================

    /// Get all cortical area IDs
    ///
    /// # Returns
    ///
    /// Vector of all cortical area IDs
    ///
    pub fn get_all_cortical_ids(&self) -> Vec<CorticalID> {
        self.cortical_areas.keys().copied().collect()
    }

    /// Get all cortical area indices
    ///
    /// # Returns
    ///
    /// Vector of all cortical area indices
    ///
    pub fn get_all_cortical_indices(&self) -> Vec<u32> {
        self.cortical_areas
            .values()
            .map(|area| area.cortical_idx)
            .collect()
    }

    /// Get all cortical area names
    ///
    /// # Returns
    ///
    /// Vector of all cortical area names
    ///
    pub fn get_cortical_area_names(&self) -> Vec<String> {
        self.cortical_areas
            .values()
            .map(|area| area.name.clone())
            .collect()
    }

    /// List all input (IPU/sensory) cortical areas
    ///
    /// # Returns
    ///
    /// Vector of IPU/sensory area IDs
    ///
    pub fn list_ipu_areas(&self) -> Vec<CorticalID> {
        use crate::models::CorticalAreaExt;
        self.cortical_areas
            .values()
            .filter(|area| area.is_input_area())
            .map(|area| area.cortical_id)
            .collect()
    }

    /// List all output (OPU/motor) cortical areas
    ///
    /// # Returns
    ///
    /// Vector of OPU/motor area IDs
    ///
    pub fn list_opu_areas(&self) -> Vec<CorticalID> {
        use crate::models::CorticalAreaExt;
        self.cortical_areas
            .values()
            .filter(|area| area.is_output_area())
            .map(|area| area.cortical_id)
            .collect()
    }

    /// Get maximum dimensions across all cortical areas
    ///
    /// # Returns
    ///
    /// (max_width, max_height, max_depth)
    ///
    pub fn get_max_cortical_area_dimensions(&self) -> (usize, usize, usize) {
        self.cortical_areas
            .values()
            .fold((0, 0, 0), |(max_w, max_h, max_d), area| {
                (
                    max_w.max(area.dimensions.width as usize),
                    max_h.max(area.dimensions.height as usize),
                    max_d.max(area.dimensions.depth as usize),
                )
            })
    }

    /// Get all properties of a cortical area as a JSON-serializable map
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Cortical area ID
    ///
    /// # Returns
    ///
    /// `Some(properties)` if found, `None` otherwise
    ///
    pub fn get_cortical_area_properties(
        &self,
        cortical_id: &CorticalID,
    ) -> Option<std::collections::HashMap<String, serde_json::Value>> {
        let area = self.get_cortical_area(cortical_id)?;

        let mut properties = std::collections::HashMap::new();
        properties.insert(
            "cortical_id".to_string(),
            serde_json::json!(area.cortical_id),
        );
        properties.insert(
            "cortical_id_s".to_string(),
            serde_json::json!(area.cortical_id.to_string()),
        );
        properties.insert(
            "cortical_idx".to_string(),
            serde_json::json!(area.cortical_idx),
        );
        properties.insert("name".to_string(), serde_json::json!(area.name));
        use crate::models::CorticalAreaExt;
        properties.insert(
            "area_type".to_string(),
            serde_json::json!(area.get_cortical_group()),
        );
        properties.insert(
            "dimensions".to_string(),
            serde_json::json!({
                "width": area.dimensions.width,
                "height": area.dimensions.height,
                "depth": area.dimensions.depth,
            }),
        );
        properties.insert("position".to_string(), serde_json::json!(area.position));

        // Copy all properties from area.properties to the response
        for (key, value) in &area.properties {
            properties.insert(key.clone(), value.clone());
        }

        // Add custom properties
        properties.extend(area.properties.clone());

        Some(properties)
    }

    /// Get properties of all cortical areas
    ///
    /// # Returns
    ///
    /// Vector of property maps for all areas
    ///
    pub fn get_all_cortical_area_properties(
        &self,
    ) -> Vec<std::collections::HashMap<String, serde_json::Value>> {
        self.cortical_areas
            .keys()
            .filter_map(|id| self.get_cortical_area_properties(id))
            .collect()
    }

    // ========================================================================
    // BRAIN REGION QUERY METHODS (P6)
    // ========================================================================

    /// Get all brain region IDs
    ///
    /// # Returns
    ///
    /// Vector of all brain region IDs
    ///
    pub fn get_all_brain_region_ids(&self) -> Vec<String> {
        self.brain_regions
            .get_all_region_ids()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get all brain region names
    ///
    /// # Returns
    ///
    /// Vector of all brain region names
    ///
    pub fn get_brain_region_names(&self) -> Vec<String> {
        self.brain_regions
            .get_all_region_ids()
            .iter()
            .filter_map(|id| {
                self.brain_regions
                    .get_region(id)
                    .map(|region| region.name.clone())
            })
            .collect()
    }

    /// Get properties of a brain region
    ///
    /// # Arguments
    ///
    /// * `region_id` - Brain region ID
    ///
    /// # Returns
    ///
    /// `Some(properties)` if found, `None` otherwise
    ///
    pub fn get_brain_region_properties(
        &self,
        region_id: &str,
    ) -> Option<std::collections::HashMap<String, serde_json::Value>> {
        let region = self.brain_regions.get_region(region_id)?;

        let mut properties = std::collections::HashMap::new();
        properties.insert("region_id".to_string(), serde_json::json!(region.region_id));
        properties.insert("name".to_string(), serde_json::json!(region.name));
        properties.insert(
            "region_type".to_string(),
            serde_json::json!(format!("{:?}", region.region_type)),
        );
        properties.insert(
            "cortical_areas".to_string(),
            serde_json::json!(region.cortical_areas.iter().collect::<Vec<_>>()),
        );

        // Add custom properties
        properties.extend(region.properties.clone());

        Some(properties)
    }

    /// Check if a cortical area exists
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Cortical area ID to check
    ///
    /// # Returns
    ///
    /// `true` if area exists, `false` otherwise
    ///
    pub fn cortical_area_exists(&self, cortical_id: &CorticalID) -> bool {
        self.cortical_areas.contains_key(cortical_id)
    }

    /// Check if a brain region exists
    ///
    /// # Arguments
    ///
    /// * `region_id` - Brain region ID to check
    ///
    /// # Returns
    ///
    /// `true` if region exists, `false` otherwise
    ///
    pub fn brain_region_exists(&self, region_id: &str) -> bool {
        self.brain_regions.get_region(region_id).is_some()
    }

    /// Get the total number of brain regions
    ///
    /// # Returns
    ///
    /// Number of brain regions
    ///
    pub fn get_brain_region_count(&self) -> usize {
        self.brain_regions.region_count()
    }

    /// Get neurons by cortical area (alias for get_neurons_in_area for API compatibility)
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Cortical area ID
    ///
    /// # Returns
    ///
    /// Vector of neuron IDs in the area
    ///
    pub fn get_neurons_by_cortical_area(&self, cortical_id: &CorticalID) -> Vec<u64> {
        // This is an alias for get_neurons_in_area, which already exists
        // Keeping it for Python API compatibility
        // Note: The signature says Vec<NeuronId> but implementation returns Vec<u64>
        self.get_neurons_in_area(cortical_id)
    }
}

// Manual Debug implementation (RustNPU doesn't implement Debug)
impl std::fmt::Debug for ConnectomeManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectomeManager")
            .field("cortical_areas", &self.cortical_areas.len())
            .field("next_cortical_idx", &self.next_cortical_idx)
            .field("brain_regions", &self.brain_regions)
            .field(
                "npu",
                &if self.npu.is_some() {
                    "Connected"
                } else {
                    "Not connected"
                },
            )
            .field("initialized", &self.initialized)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_structures::genomic::cortical_area::CoreCorticalType;

    #[test]
    fn test_singleton_instance() {
        let instance1 = ConnectomeManager::instance();
        let instance2 = ConnectomeManager::instance();

        // Both should point to the same instance
        assert_eq!(Arc::strong_count(&instance1), Arc::strong_count(&instance2));
    }

    #[test]
    fn test_add_cortical_area() {
        ConnectomeManager::reset_for_testing();

        let instance = ConnectomeManager::instance();
        let mut manager = instance.write();

        use feagi_structures::genomic::cortical_area::{
            CorticalAreaType, IOCorticalAreaConfigurationFlag,
        };
        let cortical_id = CorticalID::try_from_bytes(b"cst_add_").unwrap(); // Use unique custom ID
        let cortical_type = CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean);
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Visual Input".to_string(),
            CorticalAreaDimensions::new(128, 128, 20).unwrap(),
            (0, 0, 0).into(),
            cortical_type,
        )
        .unwrap();

        let initial_count = manager.get_cortical_area_count();
        let _cortical_idx = manager.add_cortical_area(area).unwrap();

        assert_eq!(manager.get_cortical_area_count(), initial_count + 1);
        assert!(manager.has_cortical_area(&cortical_id));
        assert!(manager.is_initialized());
    }

    #[test]
    fn test_cortical_area_lookups() {
        ConnectomeManager::reset_for_testing();

        let instance = ConnectomeManager::instance();
        let mut manager = instance.write();

        use feagi_structures::genomic::cortical_area::{
            CorticalAreaType, IOCorticalAreaConfigurationFlag,
        };
        let cortical_id = CorticalID::try_from_bytes(b"cst_look").unwrap(); // Use unique custom ID
        let cortical_type = CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean);
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test Area".to_string(),
            CorticalAreaDimensions::new(10, 10, 10).unwrap(),
            (0, 0, 0).into(),
            cortical_type,
        )
        .unwrap();

        let cortical_idx = manager.add_cortical_area(area).unwrap();

        // ID -> idx lookup
        assert_eq!(manager.get_cortical_idx(&cortical_id), Some(cortical_idx));

        // idx -> ID lookup
        assert_eq!(manager.get_cortical_id(cortical_idx), Some(&cortical_id));

        // Get area
        let retrieved_area = manager.get_cortical_area(&cortical_id).unwrap();
        assert_eq!(retrieved_area.name, "Test Area");
    }

    #[test]
    fn test_remove_cortical_area() {
        ConnectomeManager::reset_for_testing();

        let instance = ConnectomeManager::instance();
        let mut manager = instance.write();

        use feagi_structures::genomic::cortical_area::{
            CorticalAreaType, IOCorticalAreaConfigurationFlag,
        };
        let cortical_id = CoreCorticalType::Power.to_cortical_id();

        // Remove area if it already exists from previous tests
        if manager.has_cortical_area(&cortical_id) {
            manager.remove_cortical_area(&cortical_id).unwrap();
        }

        let cortical_type = CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean);
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test".to_string(),
            CorticalAreaDimensions::new(10, 10, 10).unwrap(),
            (0, 0, 0).into(),
            cortical_type,
        )
        .unwrap();

        let initial_count = manager.get_cortical_area_count();
        manager.add_cortical_area(area).unwrap();
        assert_eq!(manager.get_cortical_area_count(), initial_count + 1);

        manager.remove_cortical_area(&cortical_id).unwrap();
        assert_eq!(manager.get_cortical_area_count(), initial_count);
        assert!(!manager.has_cortical_area(&cortical_id));
    }

    #[test]
    fn test_duplicate_area_error() {
        ConnectomeManager::reset_for_testing();

        let instance = ConnectomeManager::instance();
        let mut manager = instance.write();

        use feagi_structures::genomic::cortical_area::{
            CorticalAreaType, IOCorticalAreaConfigurationFlag,
        };
        let cortical_id1 = CoreCorticalType::Power.to_cortical_id();
        let cortical_type1 = CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean);
        let area1 = CorticalArea::new(
            cortical_id1,
            0,
            "First".to_string(),
            CorticalAreaDimensions::new(10, 10, 10).unwrap(),
            (0, 0, 0).into(),
            cortical_type1,
        )
        .unwrap();

        let cortical_id2 = CoreCorticalType::Power.to_cortical_id();
        let cortical_type2 = CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean);
        let area2 = CorticalArea::new(
            cortical_id2, // Same ID
            1,
            "Second".to_string(),
            CorticalAreaDimensions::new(10, 10, 10).unwrap(),
            (0, 0, 0).into(),
            cortical_type2,
        )
        .unwrap();

        manager.add_cortical_area(area1).unwrap();
        let result = manager.add_cortical_area(area2);

        assert!(result.is_err());
    }

    #[test]
    fn test_brain_region_management() {
        ConnectomeManager::reset_for_testing();

        let instance = ConnectomeManager::instance();
        let mut manager = instance.write();

        let region_id = feagi_structures::genomic::brain_regions::RegionID::new();
        let region_id_str = region_id.to_string();
        let root = BrainRegion::new(
            region_id,
            "Root".to_string(),
            feagi_structures::genomic::brain_regions::RegionType::Undefined,
        )
        .unwrap();

        manager.add_brain_region(root, None).unwrap();

        assert_eq!(manager.get_brain_region_ids().len(), 1);
        assert!(manager.get_brain_region(&region_id_str).is_some());
    }

    #[test]
    fn test_genome_loading() {
        ConnectomeManager::reset_for_testing();

        // Use valid custom cortical IDs
        let test01_id = CorticalID::try_from_bytes(b"cstgen01").unwrap();
        let test02_id = CorticalID::try_from_bytes(b"cstgen02").unwrap();

        let genome_json = format!(
            r#"{{
            "genome_id": "test-001",
            "genome_title": "Test Genome",
            "version": "2.1",
            "blueprint": {{
                "{}": {{
                    "cortical_name": "Test Area 1",
                    "block_boundaries": [10, 10, 10],
                    "relative_coordinate": [0, 0, 0],
                    "cortical_type": "IPU",
                    "firing_threshold": 50.0
                }},
                "{}": {{
                    "cortical_name": "Test Area 2",
                    "block_boundaries": [5, 5, 5],
                    "relative_coordinate": [10, 0, 0],
                    "cortical_type": "OPU"
                }}
            }},
            "brain_regions": {{
                "root": {{
                    "title": "Root Region",
                    "parent_region_id": null,
                    "areas": ["{}", "{}"]
                }}
            }}
        }}"#,
            test01_id.as_base_64(),
            test02_id.as_base_64(),
            test01_id.as_base_64(),
            test02_id.as_base_64()
        );

        let instance = ConnectomeManager::instance();
        let mut manager = instance.write();

        // Load genome
        manager.load_genome_from_json(&genome_json).unwrap();

        // Verify cortical areas loaded
        // ConnectomeManager may inject additional built-in cortical areas (core templates/morphologies)
        // in addition to the blueprint areas supplied by the genome. This test only needs to assert
        // that the requested blueprint areas were created and are queryable.
        assert!(
            manager.get_cortical_area_count() >= 2,
            "Expected at least the 2 blueprint cortical areas to be loaded"
        );

        assert!(manager.has_cortical_area(&test01_id));
        assert!(manager.has_cortical_area(&test02_id));

        // Verify area details
        let area1 = manager.get_cortical_area(&test01_id).unwrap();
        assert_eq!(area1.name, "Test Area 1");
        assert_eq!(area1.dimensions.width, 10);
        // Note: area_type is deprecated, cortical_type_new should be used
        assert!(area1.properties.contains_key("firing_threshold"));

        let area2 = manager.get_cortical_area(&test02_id).unwrap();
        assert_eq!(area2.name, "Test Area 2");
        assert_eq!(area2.dimensions.width, 5);
        // Note: area_type is deprecated, cortical_type_new should be used

        // Verify brain regions loaded
        let brain_region_ids = manager.get_brain_region_ids();
        assert_eq!(brain_region_ids.len(), 1);
        // Get the first (and only) brain region by its actual ID
        let root_region_id = &brain_region_ids[0];
        let root_region = manager.get_brain_region(root_region_id).unwrap();
        assert_eq!(root_region.name, "Root Region");
        assert_eq!(root_region.cortical_areas.len(), 2);
        // Check that the test areas are in the brain region
        assert!(root_region.contains_area(&test01_id));
        assert!(root_region.contains_area(&test02_id));

        // Verify manager is initialized
        assert!(manager.is_initialized());
    }

    #[test]
    fn test_synapse_operations() {
        use feagi_npu_burst_engine::npu::RustNPU;
        use feagi_npu_burst_engine::TracingMutex;
        use std::sync::Arc;

        // Get ConnectomeManager singleton
        let manager_arc = ConnectomeManager::instance();

        // Create and attach NPU
        use feagi_npu_burst_engine::backend::CPUBackend;
        use feagi_npu_burst_engine::DynamicNPU;
        use feagi_npu_runtime::StdRuntime;

        let runtime = StdRuntime;
        let backend = CPUBackend::new();
        let npu_result =
            RustNPU::new(runtime, backend, 100, 1000, 10).expect("Failed to create NPU");
        let npu = Arc::new(TracingMutex::new(DynamicNPU::F32(npu_result), "TestNPU"));
        {
            let mut manager = manager_arc.write();
            manager.set_npu(npu.clone());
        }

        let mut manager = manager_arc.write();

        // First create a cortical area to add neurons to
        use feagi_structures::genomic::cortical_area::{
            CorticalAreaType, IOCorticalAreaConfigurationFlag,
        };
        let cortical_id = CorticalID::try_from_bytes(b"cst_syn_").unwrap(); // Use unique custom ID
        let cortical_type = CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean);
        let area = CorticalArea::new(
            cortical_id,
            0, // cortical_idx
            "Test Area".to_string(),
            CorticalAreaDimensions::new(10, 10, 1).unwrap(),
            (0, 0, 0).into(), // position
            cortical_type,
        )
        .unwrap();
        let cortical_idx = manager.add_cortical_area(area).unwrap();

        // Register the cortical area with the NPU using the cortical ID's base64 representation
        if let Some(npu_arc) = manager.get_npu() {
            if let Ok(mut npu_guard) = npu_arc.try_lock() {
                if let DynamicNPU::F32(ref mut npu) = *npu_guard {
                    npu.register_cortical_area(cortical_idx, cortical_id.as_base_64());
                }
            }
        }

        // Create two neurons
        let neuron1_id = manager
            .add_neuron(
                &cortical_id,
                0,
                0,
                0,     // coordinates
                100.0, // firing_threshold
                0.0,   // firing_threshold_limit (0 = no limit)
                0.1,   // leak_coefficient
                -60.0, // resting_potential
                0,     // neuron_type
                2,     // refractory_period
                1.0,   // excitability
                5,     // consecutive_fire_limit
                10,    // snooze_length
                false, // mp_charge_accumulation
            )
            .unwrap();

        let neuron2_id = manager
            .add_neuron(
                &cortical_id,
                1,
                0,
                0, // coordinates
                100.0,
                f32::MAX, // firing_threshold_limit (MAX = no limit, SIMD-friendly encoding)
                0.1,
                -60.0,
                0,
                2,
                1.0,
                5,
                10,
                false,
            )
            .unwrap();

        // Test create_synapse (creation should succeed)
        manager
            .create_synapse(
                neuron1_id, neuron2_id, 128, // weight
                64,  // conductance
                0,   // excitatory
            )
            .unwrap();

        // Note: Synapse retrieval/update/removal tests require full NPU propagation engine initialization
        // which is beyond the scope of this unit test. The important part is that create_synapse succeeds.
        println!("✅ Synapse creation test passed");
    }

    #[test]
    fn test_apply_cortical_mapping_missing_rules_is_ok() {
        // This guards against a regression where deleting a mapping causes a 500 because
        // synapse regeneration treats "no mapping rules" as an error.
        let mut manager = ConnectomeManager::new_for_testing();

        use feagi_structures::genomic::cortical_area::{
            CorticalAreaType, IOCorticalAreaConfigurationFlag,
        };

        let src_id = CorticalID::try_from_bytes(b"map_src_").unwrap();
        let dst_id = CorticalID::try_from_bytes(b"map_dst_").unwrap();

        let src_area = CorticalArea::new(
            src_id,
            0,
            "src".to_string(),
            CorticalAreaDimensions::new(2, 2, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
        )
        .unwrap();

        let dst_area = CorticalArea::new(
            dst_id,
            1,
            "dst".to_string(),
            CorticalAreaDimensions::new(2, 2, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::BrainOutput(IOCorticalAreaConfigurationFlag::Boolean),
        )
        .unwrap();

        manager.add_cortical_area(src_area).unwrap();
        manager.add_cortical_area(dst_area).unwrap();

        // No cortical_mapping_dst property set -> should be Ok(0), not an error
        let count = manager
            .apply_cortical_mapping_for_pair(&src_id, &dst_id)
            .unwrap();
        assert_eq!(count, 0);

        // Now create then delete mapping; missing destination rules should still be Ok(0)
        manager
            .update_cortical_mapping(
                &src_id,
                &dst_id,
                vec![serde_json::json!({"morphology_id":"m1"})],
            )
            .unwrap();
        manager
            .update_cortical_mapping(&src_id, &dst_id, vec![])
            .unwrap();

        let count2 = manager
            .apply_cortical_mapping_for_pair(&src_id, &dst_id)
            .unwrap();
        assert_eq!(count2, 0);
    }

    #[test]
    fn test_mapping_deletion_prunes_synapses_between_areas() {
        use feagi_npu_burst_engine::backend::CPUBackend;
        use feagi_npu_burst_engine::RustNPU;
        use feagi_npu_burst_engine::TracingMutex;
        use feagi_npu_runtime::StdRuntime;
        use feagi_structures::genomic::cortical_area::{
            CorticalAreaDimensions, CorticalAreaType, IOCorticalAreaConfigurationFlag,
        };
        use std::sync::Arc;

        // Create NPU and manager (small capacities for a deterministic unit test)
        let runtime = StdRuntime;
        let backend = CPUBackend::new();
        let npu = RustNPU::new(runtime, backend, 10_000, 10_000, 10).expect("Failed to create NPU");
        let dyn_npu = Arc::new(TracingMutex::new(
            feagi_npu_burst_engine::DynamicNPU::F32(npu),
            "TestNPU",
        ));
        let mut manager = ConnectomeManager::new_for_testing_with_npu(dyn_npu.clone());

        // Create two cortical areas
        let src_id = CorticalID::try_from_bytes(b"cst_src_").unwrap();
        let dst_id = CorticalID::try_from_bytes(b"cst_dst_").unwrap();

        let src_area = CorticalArea::new(
            src_id,
            0,
            "src".to_string(),
            CorticalAreaDimensions::new(2, 2, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
        )
        .unwrap();
        let dst_area = CorticalArea::new(
            dst_id,
            1,
            "dst".to_string(),
            CorticalAreaDimensions::new(2, 2, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::BrainOutput(IOCorticalAreaConfigurationFlag::Boolean),
        )
        .unwrap();

        manager.add_cortical_area(src_area).unwrap();
        manager.add_cortical_area(dst_area).unwrap();

        // Add a couple neurons to each area
        let s0 = manager
            .add_neuron(&src_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();
        let s1 = manager
            .add_neuron(&src_id, 1, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();
        let t0 = manager
            .add_neuron(&dst_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();
        let t1 = manager
            .add_neuron(&dst_id, 1, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();

        // Create synapses that represent an established mapping between the two areas
        manager.create_synapse(s0, t0, 128, 200, 0).unwrap();
        manager.create_synapse(s1, t1, 128, 200, 0).unwrap();

        // Build index once before pruning
        {
            let mut npu = dyn_npu.lock().unwrap();
            npu.rebuild_synapse_index();
            assert_eq!(npu.get_synapse_count(), 2);
        }

        // Simulate mapping deletion and regeneration: should prune synapses and not re-add any
        manager
            .update_cortical_mapping(&src_id, &dst_id, vec![])
            .unwrap();
        let created = manager
            .regenerate_synapses_for_mapping(&src_id, &dst_id)
            .unwrap();
        assert_eq!(created, 0);

        // Verify synapses are gone (invalidated) and no outgoing synapses remain from the sources
        {
            let mut npu = dyn_npu.lock().unwrap();
            // Pruning invalidates synapses; rebuild the index so counts/outgoing queries reflect the current state.
            npu.rebuild_synapse_index();
            assert_eq!(npu.get_synapse_count(), 0);
            assert!(npu.get_outgoing_synapses(s0 as u32).is_empty());
            assert!(npu.get_outgoing_synapses(s1 as u32).is_empty());
        }
    }

    #[test]
    fn test_mapping_update_prunes_synapses_between_areas() {
        use feagi_npu_burst_engine::backend::CPUBackend;
        use feagi_npu_burst_engine::RustNPU;
        use feagi_npu_burst_engine::TracingMutex;
        use feagi_npu_runtime::StdRuntime;
        use feagi_structures::genomic::cortical_area::{
            CorticalAreaDimensions, CorticalAreaType, IOCorticalAreaConfigurationFlag,
        };
        use std::sync::Arc;

        // Create NPU and manager (small capacities for a deterministic unit test)
        let runtime = StdRuntime;
        let backend = CPUBackend::new();
        let npu = RustNPU::new(runtime, backend, 10_000, 10_000, 10).expect("Failed to create NPU");
        let dyn_npu = Arc::new(TracingMutex::new(
            feagi_npu_burst_engine::DynamicNPU::F32(npu),
            "TestNPU",
        ));
        let mut manager = ConnectomeManager::new_for_testing_with_npu(dyn_npu.clone());

        // Seed core morphologies so mapping regeneration can resolve function morphologies (e.g. "memory").
        feagi_evolutionary::templates::add_core_morphologies(&mut manager.morphology_registry);

        // Create two cortical areas
        // Use valid custom cortical IDs (the `cst...` namespace).
        let src_id = CorticalID::try_from_bytes(b"cstupds1").unwrap();
        let dst_id = CorticalID::try_from_bytes(b"cstupdt1").unwrap();

        let src_area = CorticalArea::new(
            src_id,
            0,
            "src".to_string(),
            CorticalAreaDimensions::new(2, 2, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
        )
        .unwrap();
        let dst_area = CorticalArea::new(
            dst_id,
            0,
            "dst".to_string(),
            CorticalAreaDimensions::new(2, 2, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::BrainOutput(IOCorticalAreaConfigurationFlag::Boolean),
        )
        .unwrap();

        manager.add_cortical_area(src_area).unwrap();
        manager.add_cortical_area(dst_area).unwrap();

        // Add a couple neurons to each area
        let s0 = manager
            .add_neuron(&src_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();
        let s1 = manager
            .add_neuron(&src_id, 1, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();
        let t0 = manager
            .add_neuron(&dst_id, 0, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();
        let t1 = manager
            .add_neuron(&dst_id, 1, 0, 0, 1.0, 0.0, 0.1, 0.0, 0, 1, 1.0, 3, 1, false)
            .unwrap();

        // Create synapses that represent an established mapping between the two areas
        manager.create_synapse(s0, t0, 128, 200, 0).unwrap();
        manager.create_synapse(s1, t1, 128, 200, 0).unwrap();

        // Build index once before pruning
        {
            let mut npu = dyn_npu.lock().unwrap();
            npu.rebuild_synapse_index();
            assert_eq!(npu.get_synapse_count(), 2);
        }

        // Update mapping rules (non-empty) and regenerate.
        // This should prune the existing A→B synapses before re-applying the mapping.
        //
        // Use "memory" morphology to avoid creating physical synapses; the key assertion is that
        // the pre-existing synapses were pruned on update.
        manager
            .update_cortical_mapping(
                &src_id,
                &dst_id,
                vec![serde_json::json!({
                    "morphology_id": "memory",
                    "morphology_scalar": [1],
                    "postSynapticCurrent_multiplier": 1,
                    "plasticity_flag": false,
                    "plasticity_constant": 0,
                    "ltp_multiplier": 0,
                    "ltd_multiplier": 0,
                    "plasticity_window": 0,
                })],
            )
            .unwrap();
        let created = manager
            .regenerate_synapses_for_mapping(&src_id, &dst_id)
            .unwrap();
        assert_eq!(created, 0);

        // Verify synapses are gone and no outgoing synapses remain from the sources
        {
            let mut npu = dyn_npu.lock().unwrap();
            // Pruning invalidates synapses; rebuild the index so counts/outgoing queries reflect the current state.
            npu.rebuild_synapse_index();
            assert_eq!(npu.get_synapse_count(), 0);
            assert!(npu.get_outgoing_synapses(s0 as u32).is_empty());
            assert!(npu.get_outgoing_synapses(s1 as u32).is_empty());
        }
    }

    #[test]
    fn test_upstream_area_tracking() {
        // Test that upstream_cortical_areas property is maintained correctly
        use crate::models::cortical_area::CorticalArea;
        use feagi_npu_burst_engine::backend::CPUBackend;
        use feagi_npu_burst_engine::TracingMutex;
        use feagi_npu_burst_engine::{DynamicNPU, RustNPU};
        use feagi_npu_runtime::StdRuntime;
        use feagi_structures::genomic::cortical_area::{
            CorticalAreaDimensions, CorticalAreaType, CorticalID,
        };

        // Create test manager with NPU
        let runtime = StdRuntime;
        let backend = CPUBackend::new();
        let npu = RustNPU::new(runtime, backend, 10_000, 10_000, 10).expect("Failed to create NPU");
        let dyn_npu = Arc::new(TracingMutex::new(DynamicNPU::F32(npu), "TestNPU"));
        let mut manager = ConnectomeManager::new_for_testing_with_npu(dyn_npu.clone());

        // Seed the morphology registry with core morphologies so mapping regeneration can run.
        // (new_for_testing_with_npu() intentionally starts empty.)
        feagi_evolutionary::templates::add_core_morphologies(&mut manager.morphology_registry);

        // Create source area
        let src_id = CorticalID::try_from_bytes(b"csrc0000").unwrap();
        let src_area = CorticalArea::new(
            src_id,
            0,
            "Source Area".to_string(),
            CorticalAreaDimensions::new(2, 2, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::Custom(
                feagi_structures::genomic::cortical_area::CustomCorticalType::LeakyIntegrateFire,
            ),
        )
        .unwrap();
        let src_idx = manager.add_cortical_area(src_area).unwrap();

        // Create destination area (memory area)
        let dst_id = CorticalID::try_from_bytes(b"cdst0000").unwrap();
        let dst_area = CorticalArea::new(
            dst_id,
            0,
            "Dest Area".to_string(),
            CorticalAreaDimensions::new(2, 2, 1).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::Custom(
                feagi_structures::genomic::cortical_area::CustomCorticalType::LeakyIntegrateFire,
            ),
        )
        .unwrap();
        manager.add_cortical_area(dst_area).unwrap();

        // Verify upstream_cortical_areas property was initialized to empty array
        {
            let dst_area = manager.get_cortical_area(&dst_id).unwrap();
            let upstream = dst_area.properties.get("upstream_cortical_areas").unwrap();
            assert!(
                upstream.as_array().unwrap().is_empty(),
                "Upstream areas should be empty initially"
            );
        }

        // Create a mapping from src to dst
        let mapping_data = vec![serde_json::json!({
            "morphology_id": "memory",
            "morphology_scalar": 1,
            "postSynapticCurrent_multiplier": 1.0,
        })];
        manager
            .update_cortical_mapping(&src_id, &dst_id, mapping_data)
            .unwrap();
        manager
            .regenerate_synapses_for_mapping(&src_id, &dst_id)
            .unwrap();

        // Verify src_idx was added to dst's upstream_cortical_areas
        {
            let upstream_areas = manager.get_upstream_cortical_areas(&dst_id);
            assert_eq!(upstream_areas.len(), 1, "Should have 1 upstream area");
            assert_eq!(
                upstream_areas[0], src_idx,
                "Upstream area should be src_idx"
            );
        }

        // Delete the mapping
        manager
            .update_cortical_mapping(&src_id, &dst_id, vec![])
            .unwrap();
        manager
            .regenerate_synapses_for_mapping(&src_id, &dst_id)
            .unwrap();

        // Verify src_idx was removed from dst's upstream_cortical_areas
        {
            let upstream_areas = manager.get_upstream_cortical_areas(&dst_id);
            assert_eq!(
                upstream_areas.len(),
                0,
                "Should have 0 upstream areas after deletion"
            );
        }
    }
}
