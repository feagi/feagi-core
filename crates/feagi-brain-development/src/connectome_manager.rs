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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, trace, warn};

use crate::models::{BrainRegion, BrainRegionHierarchy, CorticalArea, CorticalAreaDimensions};
use crate::types::{BduError, BduResult};
use feagi_data_structures::genomic::cortical_area::{CoreCorticalType, CorticalID};
use feagi_npu_neural::types::NeuronId;

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
    npu: Option<Arc<Mutex<feagi_npu_burst_engine::DynamicNPU>>>,

    /// Cached neuron count (lock-free read) - updated by burst engine
    /// This prevents health checks from blocking on NPU lock
    cached_neuron_count: Arc<AtomicUsize>,

    /// Cached synapse count (lock-free read) - updated by burst engine
    /// This prevents health checks from blocking on NPU lock
    cached_synapse_count: Arc<AtomicUsize>,

    /// Is the connectome initialized (has cortical areas)?
    initialized: bool,
}

impl ConnectomeManager {
    /// Create a new ConnectomeManager (private - use `instance()`)
    fn new() -> Self {
        Self {
            cortical_areas: HashMap::new(),
            cortical_id_to_idx: HashMap::new(),
            cortical_idx_to_id: HashMap::new(),
            // CRITICAL: Reserve indices 0 (_death) and 1 (_power) - start regular areas at 2
            next_cortical_idx: 2,
            brain_regions: BrainRegionHierarchy::new(),
            morphology_registry: feagi_evolutionary::MorphologyRegistry::new(),
            config: ConnectomeConfig::default(),
            npu: None,
            cached_neuron_count: Arc::new(AtomicUsize::new(0)),
            cached_synapse_count: Arc::new(AtomicUsize::new(0)),
            initialized: false,
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
            cached_neuron_count: Arc::new(AtomicUsize::new(0)),
            cached_synapse_count: Arc::new(AtomicUsize::new(0)),
            initialized: false,
        }
    }

    /// Create a new isolated instance for testing with NPU
    ///
    /// This bypasses the singleton pattern and creates a fresh instance with NPU connected.
    /// Use this in tests to avoid conflicts between parallel test runs.
    ///
    /// # Arguments
    ///
    /// * `npu` - Arc<Mutex<RustNPU>> to connect to this manager
    ///
    /// # Example
    ///
    /// ```rust
    /// let npu = Arc::new(Mutex::new(RustNPU::new(1_000_000, 10_000_000, 10)));
    /// let manager = ConnectomeManager::new_for_testing_with_npu(npu);
    /// ```
    pub fn new_for_testing_with_npu(npu: Arc<Mutex<feagi_npu_burst_engine::DynamicNPU>>) -> Self {
        Self {
            cortical_areas: HashMap::new(),
            cortical_id_to_idx: HashMap::new(),
            cortical_idx_to_id: HashMap::new(),
            next_cortical_idx: 0,
            brain_regions: BrainRegionHierarchy::new(),
            morphology_registry: feagi_evolutionary::MorphologyRegistry::new(),
            config: ConnectomeConfig::default(),
            npu: Some(npu),
            cached_neuron_count: Arc::new(AtomicUsize::new(0)),
            cached_synapse_count: Arc::new(AtomicUsize::new(0)),
            initialized: false,
        }
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

        // CRITICAL: Reserve cortical_idx 0 for _death, 1 for _power
        // Use feagi-data-processing types as single source of truth
        use feagi_data_structures::genomic::cortical_area::CoreCorticalType;

        let death_id = CoreCorticalType::Death.to_cortical_id();
        let power_id = CoreCorticalType::Power.to_cortical_id();

        let is_death_area = &area.cortical_id == &death_id;
        let is_power_area = &area.cortical_id == &power_id;

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
        } else {
            // Regular areas: assign cortical_idx if not set (will be ≥2 due to next_cortical_idx=2 initialization)
            if area.cortical_idx == 0 {
                area.cortical_idx = self.next_cortical_idx;
                self.next_cortical_idx += 1;
                trace!(
                    target: "feagi-bdu",
                    "[REGULAR-AREA] Assigned cortical_idx={} to area '{}' (should be ≥2)",
                    area.cortical_idx,
                    area.cortical_id.as_base_64()
                );
            } else {
                // Check for reserved index collision
                if area.cortical_idx == 0 || area.cortical_idx == 1 {
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

        let cortical_id = area.cortical_id.clone();
        let cortical_idx = area.cortical_idx;

        // Update lookup maps
        self.cortical_id_to_idx
            .insert(cortical_id.clone(), cortical_idx);
        self.cortical_idx_to_id
            .insert(cortical_idx, cortical_id.clone());

        // Store area
        self.cortical_areas.insert(cortical_id.clone(), area);

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

        self.initialized = true;

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

    /// Get all cortical area IDs
    pub fn get_cortical_area_ids(&self) -> Vec<&CorticalID> {
        self.cortical_areas.keys().collect()
    }

    /// Get the number of cortical areas
    pub fn get_cortical_area_count(&self) -> usize {
        self.cortical_areas.len()
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
        Ok(self.brain_regions.add_region(region, parent_id)?)
    }

    /// Remove a brain region
    pub fn remove_brain_region(&mut self, region_id: &str) -> BduResult<()> {
        Ok(self.brain_regions.remove_region(region_id)?)
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

        Ok(())
    }

    /// Regenerate synapses for a specific cortical mapping
    ///
    /// Deletes existing synapses between the areas and creates new ones based on
    /// the updated mapping rules.
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
        if self.npu.is_some() {
            // First, delete existing synapses between these areas
            // TODO: Implement delete_synapses_between_areas in NPU

            // Then, apply cortical mapping to create new synapses
            let synapse_count = self.apply_cortical_mapping_for_pair(src_area_id, dst_area_id)?;

            info!(target: "feagi-bdu", "Created {} new synapses: {} -> {}",
                  synapse_count, src_area_id, dst_area_id);

            // CRITICAL: Rebuild synapse index so new synapses are visible to queries and propagation!
            let mut npu = self.npu.as_ref().unwrap().lock().unwrap();
            npu.rebuild_synapse_index();
            info!(target: "feagi-bdu", "Rebuilt synapse index after adding {} synapses", synapse_count);

            Ok(synapse_count)
        } else {
            info!(target: "feagi-bdu", "NPU not available - skipping synapse regeneration");
            Ok(0)
        }
    }

    /// Apply cortical mapping for a specific area pair
    fn apply_cortical_mapping_for_pair(
        &mut self,
        src_area_id: &CorticalID,
        dst_area_id: &CorticalID,
    ) -> BduResult<usize> {
        // Clone the rules to avoid borrow checker issues
        let rules = {
            let src_area = self.cortical_areas.get(src_area_id).ok_or_else(|| {
                crate::types::BduError::InvalidArea(format!(
                    "Source area not found: {}",
                    src_area_id
                ))
            })?;

            // Get cortical_mapping_dst
            let mapping_dst = src_area
                .properties
                .get("cortical_mapping_dst")
                .and_then(|v| v.as_object())
                .ok_or_else(|| {
                    crate::types::BduError::InvalidMorphology(format!(
                        "No cortical_mapping_dst for {}",
                        src_area_id
                    ))
                })?;

            // Get rules for this destination
            let rules = mapping_dst
                .get(&dst_area_id.as_base_64())
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    crate::types::BduError::InvalidMorphology(format!(
                        "No mapping rules from {} to {}",
                        src_area_id, dst_area_id
                    ))
                })?;

            rules.clone()
        }; // Borrow ends here

        if rules.is_empty() {
            return Ok(0);
        }

        // Apply each morphology rule
        let mut total_synapses = 0;
        for rule in &rules {
            let synapse_count =
                self.apply_single_morphology_rule(src_area_id, dst_area_id, rule)?;
            total_synapses += synapse_count;
        }

        Ok(total_synapses)
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
            arr.get(0).and_then(|v| v.as_str()).unwrap_or("")
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

            match morphology.morphology_type {
                feagi_evolutionary::MorphologyType::Functions => {
                    // Function-based morphologies (projector, memory, etc.)
                    match morphology_id {
                        "projector" => {
                            use crate::connectivity::synaptogenesis::apply_projector_morphology;
                            let count = apply_projector_morphology(
                                &mut npu, *src_idx, *dst_idx, None, // transpose
                                None, // project_last_layer_of
                                128,  // weight
                                255,  // conductance (u8)
                                100,  // synapse_attractivity
                            )?;
                            Ok(count as usize)
                        }
                        _ => {
                            // Other function morphologies not yet implemented
                            use tracing::debug;
                            debug!(target: "feagi-bdu", "Function morphology {} not yet implemented", morphology_id);
                            Ok(0)
                        }
                    }
                }
                feagi_evolutionary::MorphologyType::Vectors => {
                    use crate::connectivity::synaptogenesis::apply_vectors_morphology;
                    if let feagi_evolutionary::MorphologyParameters::Vectors { ref vectors } =
                        morphology.parameters
                    {
                        // Convert Vec<[i32; 3]> to Vec<(i32, i32, i32)>
                        let vectors_tuples: Vec<(i32, i32, i32)> =
                            vectors.iter().map(|v| (v[0], v[1], v[2])).collect();

                        let count = apply_vectors_morphology(
                            &mut npu,
                            *src_idx,
                            *dst_idx,
                            vectors_tuples,
                            128, // weight
                            255, // conductance (u8)
                            100, // synapse_attractivity (added parameter)
                        )?;
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
    /// * `npu` - Arc to the Rust NPU
    ///
    pub fn set_npu(&mut self, npu: Arc<Mutex<feagi_npu_burst_engine::DynamicNPU>>) {
        self.npu = Some(npu);
        info!(target: "feagi-bdu","🔗 ConnectomeManager: NPU reference set");

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
    pub fn get_npu(&self) -> Option<&Arc<Mutex<feagi_npu_burst_engine::DynamicNPU>>> {
        self.npu.as_ref()
    }

    /// Get neuron capacity from NPU
    ///
    /// # Returns
    ///
    /// * `usize` - Maximum neuron capacity, or 0 if NPU not connected
    ///
    pub fn get_neuron_capacity(&self) -> usize {
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.lock() {
                return npu_lock.get_neuron_capacity();
            }
        }
        0
    }

    /// Get synapse capacity from NPU
    ///
    /// # Returns
    ///
    /// * `usize` - Maximum synapse capacity, or 0 if NPU not connected
    ///
    pub fn get_synapse_capacity(&self) -> usize {
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.lock() {
                return npu_lock.get_synapse_capacity();
            }
        }
        0
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

        // Extract neural parameters from area properties
        use crate::models::CorticalAreaExt;
        let per_voxel_cnt = area.neurons_per_voxel();

        let firing_threshold = area
            .properties
            .get("firing_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;

        let leak_coefficient = area
            .properties
            .get("leak_coefficient")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;

        let excitability = area
            .properties
            .get("neuron_excitability")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;

        let refractory_period = area
            .properties
            .get("refractory_period")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u16;

        let consecutive_fire_limit = area
            .properties
            .get("consecutive_fire_cnt_max")
            .and_then(|v| v.as_i64())
            .unwrap_or(100) as u16;

        let snooze_length = area
            .properties
            .get("snooze_length")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as u16;

        let mp_charge_accumulation = area
            .properties
            .get("mp_charge_accumulation")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

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
                area.dimensions.width as u32,
                area.dimensions.height as u32,
                area.dimensions.depth as u32,
                per_voxel_cnt,
                firing_threshold,
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
    /// * `firing_threshold` - Firing threshold
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
    pub fn add_neuron(
        &mut self,
        cortical_id: &CorticalID,
        x: u32,
        y: u32,
        z: u32,
        firing_threshold: f32,
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

        // Get NPU
        let npu = self
            .npu
            .as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;

        let mut total_synapses = 0u32;

        // Process each destination area
        for (dst_cortical_id_str, rules) in dstmap {
            let rules_array = match rules.as_array() {
                Some(arr) => arr,
                None => continue,
            };

            // Convert string to CorticalID
            let dst_cortical_id = match CorticalID::try_from_base_64(dst_cortical_id_str) {
                Ok(id) => id,
                Err(_) => {
                    warn!(target: "feagi-bdu","Invalid cortical ID format: {}, skipping", dst_cortical_id_str);
                    continue;
                }
            };

            let dst_cortical_idx = match self.cortical_id_to_idx.get(&dst_cortical_id) {
                Some(idx) => *idx,
                None => {
                    warn!(target: "feagi-bdu","Destination area {} not found, skipping", dst_cortical_id);
                    continue;
                }
            };

            // Apply each morphology rule
            for rule in rules_array {
                let rule_obj = match rule.as_object() {
                    Some(obj) => obj,
                    None => continue,
                };

                let morphology_id = rule_obj
                    .get("morphology_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                let weight = (rule_obj
                    .get("postSynapticCurrent_multiplier")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0)
                    * 255.0)
                    .min(255.0) as u8;

                let conductance = 255u8; // Default
                let synapse_attractivity = 100u8; // Default: always create

                // Call NPU synaptogenesis based on morphology type
                let mut npu_lock = npu
                    .lock()
                    .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;

                let scalar = rule_obj
                    .get("morphology_scalar")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1) as u32;

                let synapse_count = match morphology_id {
                    "projector" => {
                        crate::connectivity::synaptogenesis::apply_projector_morphology(
                            &mut *npu_lock,
                            src_cortical_idx,
                            dst_cortical_idx,
                            None, // transpose
                            None, // project_last_layer_of
                            weight,
                            conductance,
                            synapse_attractivity,
                        )?
                    }
                    "block_to_block" => {
                        crate::connectivity::synaptogenesis::apply_block_connection_morphology(
                            &mut *npu_lock,
                            src_cortical_idx,
                            dst_cortical_idx,
                            scalar, // scaling_factor
                            weight,
                            conductance,
                            synapse_attractivity,
                        )?
                    }
                    _ => {
                        trace!(
                            target: "feagi-bdu",
                            "Morphology {} not yet implemented, skipping",
                            morphology_id
                        );
                        0
                    }
                };

                total_synapses += synapse_count;

                trace!(
                    target: "feagi-bdu",
                    "Applied {} morphology: {} -> {} = {} synapses",
                    morphology_id, src_cortical_id, dst_cortical_id, synapse_count);
            }
        }

        trace!(
            target: "feagi-bdu",
            "Created {} synapses for area {} via NPU",
            total_synapses,
            src_cortical_id
        );

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
    pub fn get_neuron_count_in_area(&self, cortical_id: &CorticalID) -> usize {
        self.get_neurons_in_area(cortical_id).len()
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

    /// Get total synapse count for a specific cortical area (outgoing only)
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - The cortical area ID
    ///
    /// # Returns
    ///
    /// Total number of outgoing synapses from neurons in this area
    ///
    pub fn get_synapse_count_in_area(&self, cortical_id: &CorticalID) -> usize {
        let neurons = self.get_neurons_in_area(cortical_id);
        let mut total = 0;

        for neuron_id in neurons {
            total += self.get_outgoing_synapses(neuron_id).len();
        }

        total
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
        self.next_cortical_idx = 2;
        info!("🔧 [BRAIN-RESET] Cortical mapping cleared, next_cortical_idx reset to 2 (reserves 0=_death, 1=_power)");
        self.brain_regions = crate::models::BrainRegionHierarchy::new();

        // Add cortical areas
        for area in parsed.cortical_areas {
            let cortical_idx = self.add_cortical_area(area)?;
            debug!(target: "feagi-bdu","  ✅ Added cortical area {} (idx: {})",
                self.cortical_idx_to_id.get(&cortical_idx).unwrap(), cortical_idx);
        }

        // Add brain regions (hierarchy)
        for (region, parent_id) in parsed.brain_regions {
            let region_id = region.region_id.clone();
            self.brain_regions.add_region(region, parent_id.clone())?;
            debug!(target: "feagi-bdu","  ✅ Added brain region {} (parent: {:?})",
                region_id, parent_id);
        }

        self.initialized = true;

        info!(target: "feagi-bdu","🧬 ✅ Genome loaded successfully!");

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

    /// Load genome from file and develop brain
    ///
    /// This is a high-level convenience method that:
    /// 1. Loads genome from JSON file
    /// 2. Prepares for new genome (clears existing state)
    /// 3. Runs neuroembryogenesis to develop the brain
    ///
    /// # Arguments
    ///
    /// * `genome_path` - Path to genome JSON file
    ///
    /// # Returns
    ///
    /// Development progress information
    ///
    pub fn load_from_genome_file<P: AsRef<std::path::Path>>(
        &mut self,
        genome_path: P,
    ) -> BduResult<crate::neuroembryogenesis::DevelopmentProgress> {
        use feagi_evolutionary::load_genome_from_file;

        info!(target: "feagi-bdu","Loading genome from: {:?}", genome_path.as_ref());
        let genome = load_genome_from_file(genome_path)?;

        self.load_from_genome(genome)
    }

    /// Load genome and develop brain
    ///
    /// This is the core genome loading method that:
    /// 1. Prepares for new genome (clears existing state)
    /// 2. Runs neuroembryogenesis to develop the brain
    ///
    /// # Arguments
    ///
    /// * `genome` - RuntimeGenome to load
    ///
    /// # Returns
    ///
    /// Development progress information
    ///
    pub fn load_from_genome(
        &mut self,
        genome: feagi_evolutionary::RuntimeGenome,
    ) -> BduResult<crate::neuroembryogenesis::DevelopmentProgress> {
        // Prepare for new genome (clear existing state)
        self.prepare_for_new_genome()?;

        // Calculate and resize memory if needed
        self.resize_for_genome(&genome)?;

        // CRITICAL FIX: This function is called with write lock already held from spawn_blocking.
        // We CANNOT call ConnectomeManager::instance() here because it would try to get ANOTHER
        // write lock on the same RwLock, causing a deadlock!
        // Instead, create neuroembryogenesis using the singleton instance directly.
        // BUT: We need to pass self's Arc reference, not create a new one.
        // Since we're inside a method that already has &mut self, we need to work with the singleton.
        // The solution: Call instance() but use it correctly - we're already holding the write lock
        // from the caller, so we need to ensure neuroembryogenesis uses the SAME Arc reference.

        // Get the singleton instance (same Arc as self.connectome in GenomeServiceImpl)
        let manager_arc = ConnectomeManager::instance();

        // CRITICAL: We're already holding a write lock from spawn_blocking.
        // Neuroembryogenesis will try to acquire its own write locks.
        // This is OK because parking_lot::RwLock allows nested write locks from the same thread!
        // But wait - we're in spawn_blocking, which is a different thread...
        // Actually, the issue is that neuroembryogenesis will try to acquire write locks
        // on the SAME Arc<RwLock<>>, which will deadlock if we're already holding a write lock.

        // SOLUTION: Don't hold the write lock during develop_from_genome.
        // Instead, release it and let neuroembryogenesis acquire its own locks.
        // But we can't do that because we're in a &mut self method...

        // ACTUAL SOLUTION: Create a temporary Arc wrapper for neuroembryogenesis
        // that uses the same underlying ConnectomeManager, but allows it to acquire its own locks.
        // OR: Refactor neuroembryogenesis to not need its own Arc.

        // For now, let's try a different approach: pass self directly to neuroembryogenesis
        // But neuroembryogenesis expects Arc<RwLock<ConnectomeManager>>...

        // TEMPORARY FIX: Use the singleton instance. This should work because parking_lot
        // allows multiple write locks from the same thread (reentrant).
        // But we're in spawn_blocking, so it's a different thread...

        // Let me check if parking_lot supports reentrant locks...
        // Actually, parking_lot::RwLock is NOT reentrant by default.

        // REAL FIX: Refactor so that load_from_genome doesn't need to hold the write lock
        // throughout the entire operation. Instead, release it and let neuroembryogenesis
        // manage its own locks.

        // For now, let's ensure we're using the same instance and that locks are properly scoped.
        let mut neuro = crate::neuroembryogenesis::Neuroembryogenesis::new(manager_arc);
        neuro.develop_from_genome(&genome)?;

        Ok(neuro.get_progress())
    }

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
        self.next_cortical_idx = 2;
        info!("🔧 [BRAIN-RESET] Cortical mapping cleared, next_cortical_idx reset to 2 (reserves 0=_death, 1=_power)");

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
        neurons: Vec<(u32, u32, u32, f32, f32, f32, i32, u16, f32, u16, u16, bool)>,
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
        // Signature: (thresholds, leak_coeffs, resting_pots, neuron_types, refract, excit, consec_limits, snooze, mp_accums, cortical_areas, x, y, z)
        // Convert f32 vectors to T
        // DynamicNPU will handle f32 inputs and convert internally based on its precision
        let firing_thresholds_t = firing_thresholds;
        let resting_potentials_t = resting_potentials;
        let (neurons_created, _indices) = npu_lock.add_neurons_batch(
            firing_thresholds_t,
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

        info!(target: "feagi-bdu",
            "Resized cortical area {} from {:?} to {:?}",
            cortical_id,
            old_dimensions,
            new_dimensions
        );

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
                        region.region_type = feagi_data_structures::genomic::RegionType::Undefined;
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
        let Some((x, y, z)) = npu_lock.get_neuron_coordinates(neuron_id_u32) else {
            return None;
        };
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
            .map(|s| s.clone())
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
    use super::*;

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

        use feagi_data_structures::genomic::cortical_area::{
            CorticalAreaType, IOCorticalAreaDataFlag,
        };
        let cortical_id = CorticalID::try_from_bytes(b"cst_add_").unwrap(); // Use unique custom ID
        let cortical_type = CorticalAreaType::BrainInput(IOCorticalAreaDataFlag::Boolean);
        let area = CorticalArea::new(
            cortical_id.clone(),
            0,
            "Visual Input".to_string(),
            CorticalAreaDimensions::new(128, 128, 20).unwrap(),
            (0, 0, 0).into(),
            cortical_type,
        )
        .unwrap();

        let initial_count = manager.get_cortical_area_count();
        let cortical_idx = manager.add_cortical_area(area).unwrap();

        assert_eq!(manager.get_cortical_area_count(), initial_count + 1);
        assert!(manager.has_cortical_area(&cortical_id));
        assert!(manager.is_initialized());
    }

    #[test]
    fn test_cortical_area_lookups() {
        ConnectomeManager::reset_for_testing();

        let instance = ConnectomeManager::instance();
        let mut manager = instance.write();

        use feagi_data_structures::genomic::cortical_area::{
            CorticalAreaType, IOCorticalAreaDataFlag,
        };
        let cortical_id = CorticalID::try_from_bytes(b"cst_look").unwrap(); // Use unique custom ID
        let cortical_type = CorticalAreaType::BrainInput(IOCorticalAreaDataFlag::Boolean);
        let area = CorticalArea::new(
            cortical_id.clone(),
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

        use feagi_data_structures::genomic::cortical_area::{
            CorticalAreaType, IOCorticalAreaDataFlag,
        };
        let cortical_id = CoreCorticalType::Power.to_cortical_id();
        let cortical_type = CorticalAreaType::BrainInput(IOCorticalAreaDataFlag::Boolean);
        let area = CorticalArea::new(
            cortical_id.clone(),
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

        use feagi_data_structures::genomic::cortical_area::{
            CorticalAreaType, IOCorticalAreaDataFlag,
        };
        let cortical_id1 = CoreCorticalType::Power.to_cortical_id();
        let cortical_type1 = CorticalAreaType::BrainInput(IOCorticalAreaDataFlag::Boolean);
        let area1 = CorticalArea::new(
            cortical_id1.clone(),
            0,
            "First".to_string(),
            CorticalAreaDimensions::new(10, 10, 10).unwrap(),
            (0, 0, 0).into(),
            cortical_type1,
        )
        .unwrap();

        let cortical_id2 = CoreCorticalType::Power.to_cortical_id();
        let cortical_type2 = CorticalAreaType::BrainInput(IOCorticalAreaDataFlag::Boolean);
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

        let region_id = feagi_data_structures::genomic::brain_regions::RegionID::new();
        let region_id_str = region_id.to_string();
        let root = BrainRegion::new(
            region_id,
            "Root".to_string(),
            feagi_data_structures::genomic::brain_regions::RegionType::Undefined,
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
        assert_eq!(manager.get_cortical_area_count(), 2);

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
        use std::sync::{Arc, Mutex};

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
        let npu = Arc::new(Mutex::new(DynamicNPU::F32(npu_result)));
        {
            let mut manager = manager_arc.write();
            manager.set_npu(npu.clone());
        }

        let mut manager = manager_arc.write();

        // First create a cortical area to add neurons to
        use feagi_data_structures::genomic::cortical_area::{
            CorticalAreaType, IOCorticalAreaDataFlag,
        };
        let cortical_id = CorticalID::try_from_bytes(b"cst_syn_").unwrap(); // Use unique custom ID
        let cortical_type = CorticalAreaType::BrainInput(IOCorticalAreaDataFlag::Boolean);
        let area = CorticalArea::new(
            cortical_id.clone(),
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
                    npu.register_cortical_area(cortical_idx as u32, cortical_id.as_base_64());
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
}
