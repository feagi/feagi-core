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
use tracing::{info, warn, debug};

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
            config: ConnectomeConfig::default(),
            npu: None,
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
    pub fn new_for_testing_with_npu(npu: Arc<Mutex<feagi_burst_engine::RustNPU>>) -> Self {
        Self {
            cortical_areas: HashMap::new(),
            cortical_id_to_idx: HashMap::new(),
            cortical_idx_to_id: HashMap::new(),
            next_cortical_idx: 0,
            brain_regions: BrainRegionHierarchy::new(),
            config: ConnectomeConfig::default(),
            npu: Some(npu),
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
        
        // CRITICAL: Special handling for _power area - it MUST get cortical_idx=1
        // This is required for power injection auto-discovery in the burst engine
        if area.cortical_id == "_power" {
            area.cortical_idx = 1;
            // If cortical_idx=1 is already taken by another area, reassign that area
            if let Some(existing_id) = self.cortical_idx_to_id.get(&1).cloned() {
                if existing_id != "_power" {
                    // Reassign the existing area to next available index
                    let new_idx = self.next_cortical_idx;
                    self.next_cortical_idx += 1;
                    
                    // Update the existing area's index
                    if let Some(existing_area) = self.cortical_areas.get_mut(&existing_id) {
                        existing_area.cortical_idx = new_idx;
                    }
                    self.cortical_id_to_idx.insert(existing_id.clone(), new_idx);
                    self.cortical_idx_to_id.remove(&1);
                    self.cortical_idx_to_id.insert(new_idx, existing_id);
                }
            }
        } else {
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
        info!(target: "feagi-bdu","🔗 ConnectomeManager: NPU reference set");
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
        
        // CRITICAL: Register cortical area name in NPU for visualization/burst loop lookups
        npu_lock.register_cortical_area(*cortical_idx, cortical_id.to_string());
        
        info!(target: "feagi-bdu","Created {} neurons for area {} via NPU", neuron_count, cortical_id);
        
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
        cortical_id: &str,
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
            return Err(BduError::InvalidArea(format!("Cortical area {} not found", cortical_id)));
        }
        
        let cortical_idx = *self.cortical_id_to_idx.get(cortical_id)
            .ok_or_else(|| BduError::InvalidArea(format!("No index for {}", cortical_id)))?;
        
        // Get NPU
        let npu = self.npu.as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;
        
        let mut npu_lock = npu.lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;
        
        // Add neuron via NPU
        let neuron_id = npu_lock.add_neuron(
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
        ).map_err(|e| BduError::Internal(format!("Failed to add neuron: {}", e)))?;
        
        debug!(target: "feagi-bdu", "Created neuron {} in area {} at ({}, {}, {})", neuron_id.0, cortical_id, x, y, z);
        
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
        let npu = self.npu.as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;
        
        let mut npu_lock = npu.lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;
        
        let deleted = npu_lock.delete_neuron(neuron_id as u32);
        
        if deleted {
            debug!(target: "feagi-bdu","Deleted neuron {}", neuron_id);
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
                        debug!(target: "feagi-bdu","Morphology {} not yet implemented, skipping", morphology_id);
                        0
                    }
                };
                
                total_synapses += synapse_count;
                
                debug!(target: "feagi-bdu","Applied {} morphology: {} -> {} = {} synapses",
                    morphology_id, src_cortical_id, dst_cortical_id, synapse_count);
            }
        }
        
        info!(target: "feagi-bdu","Created {} synapses for area {} via NPU", total_synapses, src_cortical_id);
        
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
                // Check if neuron exists AND is valid (not deleted)
                npu_lock.is_neuron_valid(neuron_id as u32)
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
        
        info!(target: "feagi-bdu","🧬 Loading genome: {} (version {})", 
            parsed.genome_title, parsed.version);
        info!(target: "feagi-bdu","🧬   Cortical areas: {}", parsed.cortical_areas.len());
        info!(target: "feagi-bdu","🧬   Brain regions: {}", parsed.brain_regions.len());
        
        // Clear existing data
        self.cortical_areas.clear();
        self.cortical_id_to_idx.clear();
        self.cortical_idx_to_id.clear();
        self.next_cortical_idx = 0;
        self.brain_regions = feagi_types::BrainRegionHierarchy::new();
        
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
        genome: feagi_evo::RuntimeGenome,
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
        self.next_cortical_idx = 0;
        
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
    pub fn resize_for_genome(&mut self, genome: &feagi_evo::RuntimeGenome) -> BduResult<()> {
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
        let npu = self.npu.as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;
        
        let mut npu_lock = npu.lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;
        
        // Verify both neurons exist
        let source_exists = (source_neuron_id as u32) < npu_lock.get_neuron_count() as u32;
        let target_exists = (target_neuron_id as u32) < npu_lock.get_neuron_count() as u32;
        
        if !source_exists {
            return Err(BduError::InvalidNeuron(format!("Source neuron {} not found", source_neuron_id)));
        }
        if !target_exists {
            return Err(BduError::InvalidNeuron(format!("Target neuron {} not found", target_neuron_id)));
        }
        
        // Create synapse via NPU
        let syn_type = if synapse_type == 0 {
            feagi_types::SynapseType::Excitatory
        } else {
            feagi_types::SynapseType::Inhibitory
        };
        
        let synapse_idx = npu_lock.add_synapse(
            feagi_types::NeuronId(source_neuron_id as u32),
            feagi_types::NeuronId(target_neuron_id as u32),
            feagi_types::SynapticWeight(weight),
            feagi_types::SynapticConductance(conductance),
            syn_type,
        ).map_err(|e| BduError::Internal(format!("Failed to create synapse: {}", e)))?;
        
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
        let npu = self.npu.as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;
        
        let mut npu_lock = npu.lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;
        
        // Update synapse weight via NPU
        let updated = npu_lock.update_synapse_weight(
            feagi_types::NeuronId(source_neuron_id as u32),
            feagi_types::NeuronId(target_neuron_id as u32),
            feagi_types::SynapticWeight(new_weight),
        );
        
        if updated {
            debug!(target: "feagi-bdu","Updated synapse weight: {} -> {} = {}", source_neuron_id, target_neuron_id, new_weight);
            Ok(())
        } else {
            Err(BduError::InvalidSynapse(format!(
                "Synapse {} -> {} not found", 
                source_neuron_id, 
                target_neuron_id
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
        let npu = self.npu.as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;
        
        let mut npu_lock = npu.lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;
        
        // Remove synapse via NPU
        let removed = npu_lock.remove_synapse(
            feagi_types::NeuronId(source_neuron_id as u32),
            feagi_types::NeuronId(target_neuron_id as u32),
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
        cortical_id: &str,
        neurons: Vec<(u32, u32, u32, f32, f32, f32, i32, u16, f32, u16, u16, bool)>,
    ) -> BduResult<Vec<u64>> {
        // Get NPU
        let npu = self.npu.as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;
        
        let mut npu_lock = npu.lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;
        
        // Get cortical area to verify it exists and get its index
        let area = self.get_cortical_area(cortical_id)
            .ok_or_else(|| BduError::InvalidArea(format!("Cortical area {} not found", cortical_id)))?;
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
        
        for (x, y, z, threshold, leak, resting, ntype, refract, excit, consec_limit, snooze, mp_accum) in neurons {
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
        let (neurons_created, _indices) = npu_lock.add_neurons_batch(
            firing_thresholds,
            leak_coeffs,
            resting_potentials,
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
    pub fn delete_neurons_batch(
        &mut self,
        neuron_ids: Vec<u64>,
    ) -> BduResult<usize> {
        // Get NPU
        let npu = self.npu.as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;
        
        let mut npu_lock = npu.lock()
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
        let npu = self.npu.as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;
        
        let mut npu_lock = npu.lock()
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
                return Err(BduError::InvalidNeuron(format!("Neuron {} not found", neuron_id)));
            }
        }
        
        if let Some(leak) = leak_coefficient {
            if npu_lock.update_neuron_leak(neuron_id_u32, leak) {
                updated = true;
                debug!(target: "feagi-bdu","Updated neuron {} leak_coefficient = {}", neuron_id, leak);
            } else if !updated {
                return Err(BduError::InvalidNeuron(format!("Neuron {} not found", neuron_id)));
            }
        }
        
        if let Some(resting) = resting_potential {
            if npu_lock.update_neuron_resting_potential(neuron_id_u32, resting) {
                updated = true;
                debug!(target: "feagi-bdu","Updated neuron {} resting_potential = {}", neuron_id, resting);
            } else if !updated {
                return Err(BduError::InvalidNeuron(format!("Neuron {} not found", neuron_id)));
            }
        }
        
        if let Some(excit) = excitability {
            if npu_lock.update_neuron_excitability(neuron_id_u32, excit) {
                updated = true;
                debug!(target: "feagi-bdu","Updated neuron {} excitability = {}", neuron_id, excit);
            } else if !updated {
                return Err(BduError::InvalidNeuron(format!("Neuron {} not found", neuron_id)));
            }
        }
        
        if !updated {
            return Err(BduError::Internal("No properties provided for update".to_string()));
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
        let npu = self.npu.as_ref()
            .ok_or_else(|| BduError::Internal("NPU not connected".to_string()))?;
        
        let mut npu_lock = npu.lock()
            .map_err(|e| BduError::Internal(format!("Failed to lock NPU: {}", e)))?;
        
        // Update threshold via NPU
        if npu_lock.update_neuron_threshold(neuron_id as u32, new_threshold) {
            debug!(target: "feagi-bdu","Set neuron {} firing threshold = {}", neuron_id, new_threshold);
            Ok(())
        } else {
            Err(BduError::InvalidNeuron(format!("Neuron {} not found", neuron_id)))
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
    pub fn get_cortical_area_by_name(&self, name: &str) -> Option<feagi_types::CorticalArea> {
        self.cortical_areas.values()
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
        cortical_id: &str,
        new_dimensions: feagi_types::Dimensions,
    ) -> BduResult<()> {
        // Validate dimensions
        if new_dimensions.width == 0 || new_dimensions.height == 0 || new_dimensions.depth == 0 {
            return Err(BduError::InvalidArea(format!(
                "Invalid dimensions: {:?} (all must be > 0)",
                new_dimensions
            )));
        }
        
        // Get and update area
        let area = self.cortical_areas.get_mut(cortical_id)
            .ok_or_else(|| BduError::InvalidArea(format!("Cortical area {} not found", cortical_id)))?;
        
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
        let region = self.brain_regions.get_region(region_id)
            .ok_or_else(|| BduError::InvalidArea(format!("Brain region {} not found", region_id)))?;
        
        Ok(region.cortical_areas.iter().cloned().collect())
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
        let region = self.brain_regions.get_region_mut(region_id)
            .ok_or_else(|| BduError::InvalidArea(format!("Brain region {} not found", region_id)))?;
        
        if let Some(name) = new_name {
            region.name = name;
            debug!(target: "feagi-bdu","Updated brain region {} name", region_id);
        }
        
        if let Some(desc) = new_description {
            // BrainRegion doesn't have a description field in the struct, so we'll store it in properties
            region.properties.insert("description".to_string(), serde_json::json!(desc));
            debug!(target: "feagi-bdu","Updated brain region {} description", region_id);
        }
        
        info!(target: "feagi-bdu","Updated brain region {}", region_id);
        
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
        cortical_id: &str,
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
        
        npu_lock.get_neuron_id_at_coordinate(cortical_idx, x, y, z)
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
        
        Some(npu_lock.get_neuron_coordinates(neuron_id as u32))
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
    pub fn get_cortical_area_for_neuron(&self, neuron_id: u64) -> Option<String> {
        let npu = self.npu.as_ref()?;
        let npu_lock = npu.lock().ok()?;
        
        // Verify neuron exists
        let neuron_count = npu_lock.get_neuron_count();
        if (neuron_id as usize) >= neuron_count {
            return None;
        }
        
        let cortical_idx = npu_lock.get_neuron_cortical_area(neuron_id as u32);
        
        // Look up cortical_id from index
        self.cortical_areas.values()
            .find(|area| area.cortical_idx == cortical_idx)
            .map(|area| area.cortical_id.clone())
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
    pub fn get_neuron_properties(&self, neuron_id: u64) -> Option<std::collections::HashMap<String, serde_json::Value>> {
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
        let (x, y, z) = npu_lock.get_neuron_coordinates(neuron_id_u32);
        properties.insert("x".to_string(), serde_json::json!(x));
        properties.insert("y".to_string(), serde_json::json!(y));
        properties.insert("z".to_string(), serde_json::json!(z));
        
        // Get cortical area
        let cortical_idx = npu_lock.get_neuron_cortical_area(neuron_id_u32);
        properties.insert("cortical_area".to_string(), serde_json::json!(cortical_idx));
        
        // Get neuron state (returns: consecutive_fire_count, consecutive_fire_limit, snooze_period, membrane_potential, threshold, refractory_countdown)
        if let Some((consec_count, consec_limit, snooze, mp, threshold, refract_countdown)) = 
            npu_lock.get_neuron_state(feagi_types::NeuronId(neuron_id_u32)) {
            properties.insert("consecutive_fire_count".to_string(), serde_json::json!(consec_count));
            properties.insert("consecutive_fire_limit".to_string(), serde_json::json!(consec_limit));
            properties.insert("snooze_period".to_string(), serde_json::json!(snooze));
            properties.insert("membrane_potential".to_string(), serde_json::json!(mp));
            properties.insert("threshold".to_string(), serde_json::json!(threshold));
            properties.insert("refractory_countdown".to_string(), serde_json::json!(refract_countdown));
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
        if let Some(refract_period) = npu_lock.get_neuron_property_u16_by_index(idx, "refractory_period") {
            properties.insert("refractory_period".to_string(), serde_json::json!(refract_period));
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
    pub fn get_neuron_property(&self, neuron_id: u64, property_name: &str) -> Option<serde_json::Value> {
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
    pub fn get_all_cortical_ids(&self) -> Vec<String> {
        self.cortical_areas.keys().cloned().collect()
    }
    
    /// Get all cortical area indices
    ///
    /// # Returns
    ///
    /// Vector of all cortical area indices
    ///
    pub fn get_all_cortical_indices(&self) -> Vec<u32> {
        self.cortical_areas.values()
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
        self.cortical_areas.values()
            .map(|area| area.name.clone())
            .collect()
    }
    
    /// List all input (IPU/sensory) cortical areas
    ///
    /// # Returns
    ///
    /// Vector of IPU/sensory area IDs
    ///
    pub fn list_ipu_areas(&self) -> Vec<String> {
        self.cortical_areas.values()
            .filter(|area| area.area_type == feagi_types::AreaType::Sensory)
            .map(|area| area.cortical_id.clone())
            .collect()
    }
    
    /// List all output (OPU/motor) cortical areas
    ///
    /// # Returns
    ///
    /// Vector of OPU/motor area IDs
    ///
    pub fn list_opu_areas(&self) -> Vec<String> {
        self.cortical_areas.values()
            .filter(|area| area.area_type == feagi_types::AreaType::Motor)
            .map(|area| area.cortical_id.clone())
            .collect()
    }
    
    /// Get maximum dimensions across all cortical areas
    ///
    /// # Returns
    ///
    /// (max_width, max_height, max_depth)
    ///
    pub fn get_max_cortical_area_dimensions(&self) -> (usize, usize, usize) {
        self.cortical_areas.values()
            .fold((0, 0, 0), |(max_w, max_h, max_d), area| {
                (
                    max_w.max(area.dimensions.width),
                    max_h.max(area.dimensions.height),
                    max_d.max(area.dimensions.depth),
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
    pub fn get_cortical_area_properties(&self, cortical_id: &str) -> Option<std::collections::HashMap<String, serde_json::Value>> {
        let area = self.get_cortical_area(cortical_id)?;
        
        let mut properties = std::collections::HashMap::new();
        properties.insert("cortical_id".to_string(), serde_json::json!(area.cortical_id));
        properties.insert("cortical_idx".to_string(), serde_json::json!(area.cortical_idx));
        properties.insert("name".to_string(), serde_json::json!(area.name));
        properties.insert("area_type".to_string(), serde_json::json!(format!("{:?}", area.area_type)));
        properties.insert("dimensions".to_string(), serde_json::json!({
            "width": area.dimensions.width,
            "height": area.dimensions.height,
            "depth": area.dimensions.depth,
        }));
        properties.insert("position".to_string(), serde_json::json!(area.position));
        properties.insert("visible".to_string(), serde_json::json!(area.visible));
        properties.insert("sub_group".to_string(), serde_json::json!(area.sub_group));
        properties.insert("neurons_per_voxel".to_string(), serde_json::json!(area.neurons_per_voxel));
        properties.insert("postsynaptic_current".to_string(), serde_json::json!(area.postsynaptic_current));
        properties.insert("plasticity_constant".to_string(), serde_json::json!(area.plasticity_constant));
        properties.insert("degeneration".to_string(), serde_json::json!(area.degeneration));
        properties.insert("psp_uniform_distribution".to_string(), serde_json::json!(area.psp_uniform_distribution));
        properties.insert("firing_threshold_increment".to_string(), serde_json::json!(area.firing_threshold_increment));
        properties.insert("firing_threshold_limit".to_string(), serde_json::json!(area.firing_threshold_limit));
        properties.insert("consecutive_fire_count".to_string(), serde_json::json!(area.consecutive_fire_count));
        properties.insert("snooze_period".to_string(), serde_json::json!(area.snooze_period));
        properties.insert("refractory_period".to_string(), serde_json::json!(area.refractory_period));
        properties.insert("leak_coefficient".to_string(), serde_json::json!(area.leak_coefficient));
        properties.insert("leak_variability".to_string(), serde_json::json!(area.leak_variability));
        properties.insert("burst_engine_active".to_string(), serde_json::json!(area.burst_engine_active));
        
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
    pub fn get_all_cortical_area_properties(&self) -> Vec<std::collections::HashMap<String, serde_json::Value>> {
        self.cortical_areas.keys()
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
        self.brain_regions.get_all_region_ids()
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
        self.brain_regions.get_all_region_ids()
            .iter()
            .filter_map(|id| {
                self.brain_regions.get_region(id)
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
    pub fn get_brain_region_properties(&self, region_id: &str) -> Option<std::collections::HashMap<String, serde_json::Value>> {
        let region = self.brain_regions.get_region(region_id)?;
        
        let mut properties = std::collections::HashMap::new();
        properties.insert("region_id".to_string(), serde_json::json!(region.region_id));
        properties.insert("name".to_string(), serde_json::json!(region.name));
        properties.insert("region_type".to_string(), serde_json::json!(format!("{:?}", region.region_type)));
        properties.insert("cortical_areas".to_string(), serde_json::json!(region.cortical_areas.iter().collect::<Vec<_>>()));
        
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
    pub fn cortical_area_exists(&self, cortical_id: &str) -> bool {
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
    pub fn get_neurons_by_cortical_area(&self, cortical_id: &str) -> Vec<u64> {
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
    
    #[test]
    fn test_synapse_operations() {
        use std::sync::{Arc, Mutex};
        use feagi_burst_engine::npu::RustNPU;
        
        // Get ConnectomeManager singleton
        let manager_arc = ConnectomeManager::instance();
        
        // Create and attach NPU
        let npu = Arc::new(Mutex::new(RustNPU::new(100, 1000, 10)));
        {
            let mut manager = manager_arc.write();
            manager.set_npu(npu.clone());
        }
        
        let mut manager = manager_arc.write();
        
        // First create a cortical area to add neurons to
        let area = feagi_types::CorticalArea::new(
            "test01".to_string(),
            0, // cortical_idx
            "Test Area".to_string(),
            feagi_types::Dimensions { width: 10, height: 10, depth: 1 },
            (0, 0, 0), // position
            feagi_types::AreaType::Sensory,
        ).unwrap();
        manager.add_cortical_area(area).unwrap();
        
        // Create two neurons
        let neuron1_id = manager.add_neuron(
            "test01", 
            0, 0, 0, // coordinates
            100.0, // firing_threshold
            0.1, // leak_coefficient
            -60.0, // resting_potential
            0, // neuron_type
            2, // refractory_period
            1.0, // excitability
            5, // consecutive_fire_limit
            10, // snooze_length
            false, // mp_charge_accumulation
        ).unwrap();
        
        let neuron2_id = manager.add_neuron(
            "test01",
            1, 0, 0, // coordinates
            100.0,
            0.1,
            -60.0,
            0,
            2,
            1.0,
            5,
            10,
            false,
        ).unwrap();
        
        // Test create_synapse
        manager.create_synapse(
            neuron1_id,
            neuron2_id,
            128, // weight
            64, // conductance
            0,  // excitatory
        ).unwrap();
        
        // Test get_synapse
        let synapse_info = manager.get_synapse(neuron1_id, neuron2_id);
        assert!(synapse_info.is_some(), "Synapse not found");
        let (weight, conductance, syn_type) = synapse_info.unwrap();
        assert_eq!(weight, 128);
        assert_eq!(conductance, 64);
        assert_eq!(syn_type, 0); // excitatory
        
        // Test update_synapse_weight
        manager.update_synapse_weight(neuron1_id, neuron2_id, 200).unwrap();
        
        // Verify weight updated
        let synapse_info = manager.get_synapse(neuron1_id, neuron2_id);
        assert!(synapse_info.is_some());
        let (weight, _, _) = synapse_info.unwrap();
        assert_eq!(weight, 200);
        
        // Test remove_synapse
        let removed = manager.remove_synapse(neuron1_id, neuron2_id).unwrap();
        assert!(removed);
        
        // Verify synapse removed
        let synapse_info = manager.get_synapse(neuron1_id, neuron2_id);
        assert!(synapse_info.is_none());
        
        // Test remove non-existent synapse
        let removed = manager.remove_synapse(neuron1_id, neuron2_id).unwrap();
        assert!(!removed);
    }
}

