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
use std::sync::{Arc, Mutex};

use feagi_types::{BrainRegion, BrainRegionHierarchy, CorticalArea};
use crate::types::{BduError, BduResult, NeuronId};

// NPU integration (optional dependency)
use feagi_burst_engine::RustNPU;

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
    cortical_areas: HashMap<String, CorticalArea>,
    
    /// Map of cortical_id -> cortical_idx (fast reverse lookup)
    cortical_id_to_idx: HashMap<String, u32>,
    
    /// Map of cortical_idx -> cortical_id (fast reverse lookup)
    cortical_idx_to_id: HashMap<u32, String>,
    
    /// Next available cortical index
    next_cortical_idx: u32,
    
    /// Brain region hierarchy
    brain_regions: BrainRegionHierarchy,
    
    /// Configuration
    config: ConnectomeConfig,
    
    /// Optional reference to the Rust NPU for neuron/synapse queries
    /// 
    /// This is set by the Python process manager after NPU initialization.
    /// All neuron/synapse data queries delegate to the NPU.
    npu: Option<Arc<Mutex<RustNPU>>>,
    
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
            next_cortical_idx: 0,
            brain_regions: BrainRegionHierarchy::new(),
            config: ConnectomeConfig::default(),
            npu: None,
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
    /// use feagi_bdu::ConnectomeManager;
    ///
    /// let manager = ConnectomeManager::instance();
    /// let read_lock = manager.read();
    /// let area_count = read_lock.get_cortical_area_count();
    /// ```
    ///
    pub fn instance() -> Arc<RwLock<Self>> {
        Arc::clone(&INSTANCE)
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
        
        // Assign cortical_idx if not set
        if area.cortical_idx == 0 {
            area.cortical_idx = self.next_cortical_idx;
            self.next_cortical_idx += 1;
        } else {
            // Check for index conflict
            if self.cortical_idx_to_id.contains_key(&area.cortical_idx) {
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
        
        let cortical_id = area.cortical_id.clone();
        let cortical_idx = area.cortical_idx;
        
        // Update lookup maps
        self.cortical_id_to_idx.insert(cortical_id.clone(), cortical_idx);
        self.cortical_idx_to_id.insert(cortical_idx, cortical_id.clone());
        
        // Store area
        self.cortical_areas.insert(cortical_id, area);
        
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
    pub fn remove_cortical_area(&mut self, cortical_id: &str) -> BduResult<()> {
        let area = self.cortical_areas.remove(cortical_id).ok_or_else(|| {
            BduError::InvalidArea(format!("Cortical area {} does not exist", cortical_id))
        })?;
        
        // Remove from lookup maps
        self.cortical_id_to_idx.remove(cortical_id);
        self.cortical_idx_to_id.remove(&area.cortical_idx);
        
        Ok(())
    }
    
    /// Get a cortical area by ID
    pub fn get_cortical_area(&self, cortical_id: &str) -> Option<&CorticalArea> {
        self.cortical_areas.get(cortical_id)
    }
    
    /// Get a mutable reference to a cortical area
    pub fn get_cortical_area_mut(&mut self, cortical_id: &str) -> Option<&mut CorticalArea> {
        self.cortical_areas.get_mut(cortical_id)
    }
    
    /// Get cortical index by ID
    pub fn get_cortical_idx(&self, cortical_id: &str) -> Option<u32> {
        self.cortical_id_to_idx.get(cortical_id).copied()
    }
    
    /// Get cortical ID by index
    pub fn get_cortical_id(&self, cortical_idx: u32) -> Option<&String> {
        self.cortical_idx_to_id.get(&cortical_idx)
    }
    
    /// Get all cortical area IDs
    pub fn get_cortical_area_ids(&self) -> Vec<&String> {
        self.cortical_areas.keys().collect()
    }
    
    /// Get the number of cortical areas
    pub fn get_cortical_area_count(&self) -> usize {
        self.cortical_areas.len()
    }
    
    /// Check if a cortical area exists
    pub fn has_cortical_area(&self, cortical_id: &str) -> bool {
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
    pub fn set_npu(&mut self, npu: Arc<Mutex<RustNPU>>) {
        self.npu = Some(npu);
        log::info!("🔗 ConnectomeManager: NPU reference set");
    }
    
    /// Check if NPU is connected
    pub fn has_npu(&self) -> bool {
        self.npu.is_some()
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
    pub fn create_neurons_for_area(&mut self, cortical_id: &str) -> BduResult<u32> {
        // Get cortical area
        let area = self.cortical_areas.get(cortical_id)
            .ok_or_else(|| BduError::InvalidArea(format!("Cortical area {} not found", cortical_id)))?
            .clone();
        
        // Get cortical index
        let cortical_idx = self.cortical_id_to_idx.get(cortical_id)
            .ok_or_else(|| BduError::InvalidArea(format!("No index for cortical area {}", cortical_id)))?;
        
        // Get NPU
        let npu = self.npu.as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;
        
        // Extract neural parameters from area properties
        let per_voxel_cnt = area.properties
            .get("per_voxel_neuron_cnt")
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as u32;
        
        let firing_threshold = area.properties
            .get("firing_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        
        let leak_coefficient = area.properties
            .get("leak_coefficient")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        
        let excitability = area.properties
            .get("neuron_excitability")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        
        let refractory_period = area.properties
            .get("refractory_period")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u16;
        
        let consecutive_fire_limit = area.properties
            .get("consecutive_fire_cnt_max")
            .and_then(|v| v.as_i64())
            .unwrap_or(100) as u16;
        
        let snooze_length = area.properties
            .get("snooze_length")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as u16;
        
        let mp_charge_accumulation = area.properties
            .get("mp_charge_accumulation")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // Call NPU to create neurons
        let mut npu_lock = npu.lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;
        
        let neuron_count = npu_lock.create_cortical_area_neurons(
            *cortical_idx,
            area.dimensions.width as u32,
            area.dimensions.height as u32,
            area.dimensions.depth as u32,
            per_voxel_cnt,
            firing_threshold,
            leak_coefficient,
            0.0, // resting_potential (LIF default)
            0, // neuron_type (excitatory)
            refractory_period,
            excitability,
            consecutive_fire_limit,
            snooze_length,
            mp_charge_accumulation,
        )
        .map_err(|e| BduError::Internal(format!("NPU neuron creation failed: {}", e)))?;
        
        log::info!("Created {} neurons for area {} via NPU", neuron_count, cortical_id);
        
        Ok(neuron_count)
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
    pub fn apply_cortical_mapping(&mut self, src_cortical_id: &str) -> BduResult<u32> {
        // Get source area
        let src_area = self.cortical_areas.get(src_cortical_id)
            .ok_or_else(|| BduError::InvalidArea(format!("Source area {} not found", src_cortical_id)))?
            .clone();
        
        // Get dstmap from area properties
        let dstmap = match src_area.properties.get("cortical_mapping_dst") {
            Some(serde_json::Value::Object(map)) if !map.is_empty() => map,
            _ => return Ok(0), // No mappings
        };
        
        let src_cortical_idx = *self.cortical_id_to_idx.get(src_cortical_id)
            .ok_or_else(|| BduError::InvalidArea(format!("No index for {}", src_cortical_id)))?;
        
        // Get NPU
        let npu = self.npu.as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;
        
        let mut total_synapses = 0u32;
        
        // Process each destination area
        for (dst_cortical_id, rules) in dstmap {
            let rules_array = match rules.as_array() {
                Some(arr) => arr,
                None => continue,
            };
            
            let dst_cortical_idx = match self.cortical_id_to_idx.get(dst_cortical_id) {
                Some(idx) => *idx,
                None => {
                    log::warn!("Destination area {} not found, skipping", dst_cortical_id);
                    continue;
                }
            };
            
            // Apply each morphology rule
            for rule in rules_array {
                let rule_obj = match rule.as_object() {
                    Some(obj) => obj,
                    None => continue,
                };
                
                let morphology_id = rule_obj.get("morphology_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                let weight = (rule_obj.get("postSynapticCurrent_multiplier")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0) * 255.0).min(255.0) as u8;
                
                let conductance = 255u8; // Default
                let synapse_attractivity = 100u8; // Default: always create
                
                // Call NPU synaptogenesis based on morphology type
                let mut npu_lock = npu.lock()
                    .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;
                
                let scalar = rule_obj.get("morphology_scalar")
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
                        log::debug!("Morphology {} not yet implemented, skipping", morphology_id);
                        0
                    }
                };
                
                total_synapses += synapse_count;
                
                log::debug!("Applied {} morphology: {} -> {} = {} synapses",
                    morphology_id, src_cortical_id, dst_cortical_id, synapse_count);
            }
        }
        
        log::info!("Created {} synapses for area {} via NPU", total_synapses, src_cortical_id);
        
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
    pub fn has_neuron(&self, neuron_id: NeuronId) -> bool {
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.lock() {
                let count = npu_lock.get_neuron_count();
                (neuron_id as u32) < count as u32
            } else {
                false
            }
        } else {
            false
        }
    }
    
    /// Get total number of active neurons
    ///
    /// # Returns
    ///
    /// The total number of neurons in the NPU, or 0 if NPU is not connected
    ///
    pub fn get_neuron_count(&self) -> usize {
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.lock() {
                npu_lock.get_neuron_count()
            } else {
                0
            }
        } else {
            0
        }
    }
    
    /// Get total number of synapses
    ///
    /// # Returns
    ///
    /// The total number of synapses in the NPU, or 0 if NPU is not connected
    ///
    pub fn get_synapse_count(&self) -> usize {
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.lock() {
                npu_lock.get_synapse_count()
            } else {
                0
            }
        } else {
            0
        }
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
    pub fn get_neuron_coordinates(&self, neuron_id: NeuronId) -> (u32, u32, u32) {
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.lock() {
                npu_lock.get_neuron_coordinates(neuron_id as u32)
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
    pub fn get_neuron_cortical_idx(&self, neuron_id: NeuronId) -> u32 {
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
    pub fn get_neurons_in_area(&self, cortical_id: &str) -> Vec<NeuronId> {
        // Get cortical_idx from cortical_id
        let cortical_idx = match self.cortical_id_to_idx.get(cortical_id) {
            Some(idx) => *idx,
            None => return Vec::new(),
        };
        
        if let Some(ref npu) = self.npu {
            if let Ok(npu_lock) = npu.lock() {
                // Convert Vec<u32> to Vec<u64>
                npu_lock.get_neurons_in_cortical_area(cortical_idx)
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
    pub fn get_outgoing_synapses(&self, source_neuron_id: NeuronId) -> Vec<(u32, u8, u8, u8)> {
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
    pub fn get_incoming_synapses(&self, target_neuron_id: NeuronId) -> Vec<(u32, u8, u8, u8)> {
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
    pub fn get_neuron_count_in_area(&self, cortical_id: &str) -> usize {
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
    pub fn is_area_populated(&self, cortical_id: &str) -> bool {
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
    pub fn get_synapse_count_in_area(&self, cortical_id: &str) -> usize {
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
    pub fn are_neurons_connected(&self, source_neuron_id: NeuronId, target_neuron_id: NeuronId) -> bool {
        let synapses = self.get_outgoing_synapses(source_neuron_id);
        synapses.iter().any(|(target, _, _, _)| *target == target_neuron_id as u32)
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
    pub fn get_connection_weight(&self, source_neuron_id: NeuronId, target_neuron_id: NeuronId) -> Option<u8> {
        let synapses = self.get_outgoing_synapses(source_neuron_id);
        synapses.iter()
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
    pub fn get_area_connectivity_stats(&self, cortical_id: &str) -> (usize, usize, f32) {
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
    /// The cortical area ID string, or None if neuron doesn't exist
    ///
    pub fn get_neuron_cortical_id(&self, neuron_id: NeuronId) -> Option<String> {
        let cortical_idx = self.get_neuron_cortical_idx(neuron_id);
        self.cortical_idx_to_id.get(&cortical_idx).cloned()
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
    pub fn get_neuron_density(&self, cortical_id: &str) -> f32 {
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
            
            stats.push((cortical_id.to_string(), neuron_count, synapse_count, density));
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
    /// use feagi_bdu::ConnectomeManager;
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
        let parsed = feagi_evo::GenomeParser::parse(json_str)?;
        
        log::info!("🧬 Loading genome: {} (version {})", 
            parsed.genome_title, parsed.version);
        log::info!("🧬   Cortical areas: {}", parsed.cortical_areas.len());
        log::info!("🧬   Brain regions: {}", parsed.brain_regions.len());
        
        // Clear existing data
        self.cortical_areas.clear();
        self.cortical_id_to_idx.clear();
        self.cortical_idx_to_id.clear();
        self.next_cortical_idx = 0;
        self.brain_regions = feagi_types::BrainRegionHierarchy::new();
        
        // Add cortical areas
        for area in parsed.cortical_areas {
            let cortical_idx = self.add_cortical_area(area)?;
            log::debug!("  ✅ Added cortical area {} (idx: {})", 
                self.cortical_idx_to_id.get(&cortical_idx).unwrap(), cortical_idx);
        }
        
        // Add brain regions (hierarchy)
        for (region, parent_id) in parsed.brain_regions {
            let region_id = region.region_id.clone();
            self.brain_regions.add_region(region, parent_id.clone())?;
            log::debug!("  ✅ Added brain region {} (parent: {:?})", 
                region_id, parent_id);
        }
        
        self.initialized = true;
        
        log::info!("🧬 ✅ Genome loaded successfully!");
        
        Ok(())
    }
    
    /// Save the connectome as a genome JSON
    ///
    /// # Arguments
    ///
    /// * `genome_id` - Optional custom genome ID (generates timestamp-based ID if None)
    /// * `genome_title` - Optional custom genome title
    ///
    /// # Returns
    ///
    /// JSON string representation of the genome
    ///
    pub fn save_genome_to_json(
        &self,
        genome_id: Option<String>,
        genome_title: Option<String>,
    ) -> BduResult<String> {
        // Build parent map from brain region hierarchy
        let mut brain_regions_with_parents = std::collections::HashMap::new();
        
        for region_id in self.brain_regions.get_all_region_ids() {
            if let Some(region) = self.brain_regions.get_region(region_id) {
                let parent_id = self.brain_regions.get_parent(region_id)
                    .map(|s| s.to_string());
                brain_regions_with_parents.insert(region_id.to_string(), (region.clone(), parent_id));
            }
        }
        
        // Generate and return JSON
        Ok(feagi_evo::GenomeSaver::save_to_json(
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
        use feagi_evo::load_genome_from_file;
        
        log::info!("Loading genome from: {:?}", genome_path.as_ref());
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
        genome: feagi_evo::RuntimeGenome,
    ) -> BduResult<crate::neuroembryogenesis::DevelopmentProgress> {
        // Prepare for new genome (clear existing state)
        self.prepare_for_new_genome()?;
        
        // Calculate and resize memory if needed
        self.resize_for_genome(&genome)?;
        
        // Run neuroembryogenesis
        let manager_arc = ConnectomeManager::instance();
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
        log::info!("Preparing for new genome (clearing existing state)");
        
        // Clear cortical areas
        self.cortical_areas.clear();
        self.cortical_id_to_idx.clear();
        self.cortical_idx_to_id.clear();
        self.next_cortical_idx = 0;
        
        // Clear brain regions
        self.brain_regions = BrainRegionHierarchy::new();
        
        // Reset NPU if present
        // TODO: Add reset() method to RustNPU
        // if let Some(ref npu) = self.npu {
        //     let mut npu_lock = npu.lock().unwrap();
        //     npu_lock.reset();
        // }
        
        log::info!("✅ Connectome cleared and ready for new genome");
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
    pub fn resize_for_genome(&mut self, genome: &feagi_evo::RuntimeGenome) -> BduResult<()> {
        // Calculate required capacity from genome stats
        let required_neurons = genome.stats.innate_neuron_count;
        let required_synapses = genome.stats.innate_synapse_count;
        
        log::info!(
            "Genome requires: {} neurons, {} synapses",
            required_neurons,
            required_synapses
        );
        
        // Calculate total voxels from all cortical areas
        let mut total_voxels = 0;
        for area in genome.cortical_areas.values() {
            total_voxels += area.dimensions.width * area.dimensions.height * area.dimensions.depth;
        }
        
        log::info!(
            "Genome has {} cortical areas with {} total voxels",
            genome.cortical_areas.len(),
            total_voxels
        );
        
        // TODO: Resize NPU if needed
        // For now, we assume NPU has sufficient capacity
        // In the future, we may want to dynamically resize the NPU based on genome requirements
        
        Ok(())
    }
}

// Manual Debug implementation (RustNPU doesn't implement Debug)
impl std::fmt::Debug for ConnectomeManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectomeManager")
            .field("cortical_areas", &self.cortical_areas.len())
            .field("next_cortical_idx", &self.next_cortical_idx)
            .field("brain_regions", &self.brain_regions)
            .field("npu", &if self.npu.is_some() { "Connected" } else { "Not connected" })
            .field("initialized", &self.initialized)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_types::{AreaType, Dimensions};
    
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
        
        let area = CorticalArea::new(
            "iav001".to_string(),
            0,
            "Visual Input".to_string(),
            Dimensions::new(128, 128, 20),
            (0, 0, 0),
            AreaType::Sensory,
        )
        .unwrap();
        
        let cortical_idx = manager.add_cortical_area(area).unwrap();
        
        assert_eq!(cortical_idx, 0);
        assert_eq!(manager.get_cortical_area_count(), 1);
        assert!(manager.has_cortical_area("iav001"));
        assert!(manager.is_initialized());
    }
    
    #[test]
    fn test_cortical_area_lookups() {
        ConnectomeManager::reset_for_testing();
        
        let instance = ConnectomeManager::instance();
        let mut manager = instance.write();
        
        let area = CorticalArea::new(
            "test01".to_string(),
            0,
            "Test Area".to_string(),
            Dimensions::new(10, 10, 10),
            (0, 0, 0),
            AreaType::Custom,
        )
        .unwrap();
        
        manager.add_cortical_area(area).unwrap();
        
        // ID -> idx lookup
        assert_eq!(manager.get_cortical_idx("test01"), Some(0));
        
        // idx -> ID lookup
        assert_eq!(manager.get_cortical_id(0), Some(&"test01".to_string()));
        
        // Get area
        let retrieved_area = manager.get_cortical_area("test01").unwrap();
        assert_eq!(retrieved_area.name, "Test Area");
    }
    
    #[test]
    fn test_remove_cortical_area() {
        ConnectomeManager::reset_for_testing();
        
        let instance = ConnectomeManager::instance();
        let mut manager = instance.write();
        
        let area = CorticalArea::new(
            "test02".to_string(),
            0,
            "Test".to_string(),
            Dimensions::new(10, 10, 10),
            (0, 0, 0),
            AreaType::Custom,
        )
        .unwrap();
        
        manager.add_cortical_area(area).unwrap();
        assert_eq!(manager.get_cortical_area_count(), 1);
        
        manager.remove_cortical_area("test02").unwrap();
        assert_eq!(manager.get_cortical_area_count(), 0);
        assert!(!manager.has_cortical_area("test02"));
    }
    
    #[test]
    fn test_duplicate_area_error() {
        ConnectomeManager::reset_for_testing();
        
        let instance = ConnectomeManager::instance();
        let mut manager = instance.write();
        
        let area1 = CorticalArea::new(
            "dup001".to_string(),
            0,
            "First".to_string(),
            Dimensions::new(10, 10, 10),
            (0, 0, 0),
            AreaType::Custom,
        )
        .unwrap();
        
        let area2 = CorticalArea::new(
            "dup001".to_string(), // Same ID
            1,
            "Second".to_string(),
            Dimensions::new(10, 10, 10),
            (0, 0, 0),
            AreaType::Custom,
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
        
        let root = BrainRegion::new(
            "root".to_string(),
            "Root".to_string(),
            feagi_types::RegionType::Custom,
        )
        .unwrap();
        
        manager.add_brain_region(root, None).unwrap();
        
        assert_eq!(manager.get_brain_region_ids().len(), 1);
        assert!(manager.get_brain_region("root").is_some());
    }
    
    #[test]
    fn test_genome_loading() {
        ConnectomeManager::reset_for_testing();
        
        let genome_json = r#"{
            "genome_id": "test-001",
            "genome_title": "Test Genome",
            "version": "2.1",
            "blueprint": {
                "test01": {
                    "cortical_name": "Test Area 1",
                    "block_boundaries": [10, 10, 10],
                    "relative_coordinate": [0, 0, 0],
                    "cortical_type": "IPU",
                    "firing_threshold": 50.0
                },
                "test02": {
                    "cortical_name": "Test Area 2",
                    "block_boundaries": [5, 5, 5],
                    "relative_coordinate": [10, 0, 0],
                    "cortical_type": "OPU"
                }
            },
            "brain_regions": {
                "root": {
                    "title": "Root Region",
                    "parent_region_id": null,
                    "areas": ["test01", "test02"]
                }
            }
        }"#;
        
        let instance = ConnectomeManager::instance();
        let mut manager = instance.write();
        
        // Load genome
        manager.load_genome_from_json(genome_json).unwrap();
        
        // Verify cortical areas loaded
        assert_eq!(manager.get_cortical_area_count(), 2);
        assert!(manager.has_cortical_area("test01"));
        assert!(manager.has_cortical_area("test02"));
        
        // Verify area details
        let area1 = manager.get_cortical_area("test01").unwrap();
        assert_eq!(area1.name, "Test Area 1");
        assert_eq!(area1.dimensions.width, 10);
        assert_eq!(area1.area_type, AreaType::Sensory);
        assert!(area1.properties.contains_key("firing_threshold"));
        
        let area2 = manager.get_cortical_area("test02").unwrap();
        assert_eq!(area2.name, "Test Area 2");
        assert_eq!(area2.dimensions.width, 5);
        assert_eq!(area2.area_type, AreaType::Motor);
        
        // Verify brain regions loaded
        assert_eq!(manager.get_brain_region_ids().len(), 1);
        let root_region = manager.get_brain_region("root").unwrap();
        assert_eq!(root_region.name, "Root Region");
        assert_eq!(root_region.cortical_areas.len(), 2);
        assert!(root_region.contains_area("test01"));
        assert!(root_region.contains_area("test02"));
        
        // Verify manager is initialized
        assert!(manager.is_initialized());
    }
}

