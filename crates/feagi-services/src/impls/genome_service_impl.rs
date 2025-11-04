/*!
Genome service implementation.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::traits::GenomeService;
use crate::types::*;
use async_trait::async_trait;
use feagi_bdu::ConnectomeManager;
use feagi_burst_engine::ParameterUpdateQueue;
use parking_lot::RwLock;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, warn, debug};
use feagi_bdu::neuroembryogenesis::Neuroembryogenesis;
use serde_json::Value;

use crate::genome::{ChangeType, CorticalChangeClassifier};

/// Default implementation of GenomeService
pub struct GenomeServiceImpl {
    connectome: Arc<RwLock<ConnectomeManager<f32>>>,
    parameter_queue: Option<ParameterUpdateQueue>,
    /// Currently loaded genome (source of truth for structural changes)
    /// This is updated when genome is loaded or when cortical areas are modified
    current_genome: Arc<RwLock<Option<feagi_evo::RuntimeGenome>>>,
}

impl GenomeServiceImpl {
    pub fn new(connectome: Arc<RwLock<ConnectomeManager<f32>>>) -> Self {
        Self { 
            connectome,
            parameter_queue: None,
            current_genome: Arc::new(RwLock::new(None)),
        }
    }
    
    pub fn new_with_parameter_queue(
        connectome: Arc<RwLock<ConnectomeManager<f32>>>,
        parameter_queue: ParameterUpdateQueue,
    ) -> Self {
        Self { 
            connectome,
            parameter_queue: Some(parameter_queue),
            current_genome: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait]
impl GenomeService for GenomeServiceImpl {
    async fn load_genome(&self, params: LoadGenomeParams) -> ServiceResult<GenomeInfo> {
        info!(target: "feagi-services", "Loading genome from JSON");
        
        // Parse genome using feagi-evo (this is CPU-bound, but relatively fast)
        let genome = feagi_evo::load_genome_from_json(&params.json_str)
            .map_err(|e| ServiceError::InvalidInput(format!("Failed to parse genome: {}", e)))?;
        
        // Extract simulation_timestep from genome physiology (will be returned in GenomeInfo)
        let simulation_timestep = genome.physiology.simulation_timestep;
        info!(target: "feagi-services", "Genome simulation_timestep: {} seconds", simulation_timestep);
        
        // Store genome for future updates (source of truth for structural changes)
        *self.current_genome.write() = Some(genome.clone());
        
        // Load into connectome via ConnectomeManager
        // This involves synaptogenesis which can be CPU-intensive, so run it on a blocking thread
        // CRITICAL: Add timeout to prevent hanging during shutdown
        // Note: spawn_blocking tasks cannot be cancelled, but timeout ensures we don't wait forever
        // CRITICAL FIX: Don't hold write lock during entire operation - let neuroembryogenesis manage locks
        // This prevents deadlock when neuroembryogenesis tries to acquire its own write locks
        let connectome_clone = self.connectome.clone();
        let blocking_handle = tokio::task::spawn_blocking(move || -> Result<feagi_bdu::neuroembryogenesis::DevelopmentProgress, ServiceError> {
            // Acquire write lock only for prepare/resize operations
            let genome_clone = genome;
            let (prepare_result, resize_result) = {
                let mut manager = connectome_clone.write();
                let prepare_result = manager.prepare_for_new_genome();
                let resize_result = prepare_result.as_ref().ok().map(|_| manager.resize_for_genome(&genome_clone));
                (prepare_result, resize_result)
            }; // Lock released here
            
            prepare_result.map_err(ServiceError::from)?;
            if let Some(resize_result) = resize_result {
                resize_result.map_err(ServiceError::from)?;
            }
            
            // Now call develop_from_genome without holding the lock
            // It will acquire its own locks internally
            let manager_arc = feagi_bdu::ConnectomeManager::<f32>::instance();
            let mut neuro = Neuroembryogenesis::new(manager_arc);
            neuro.develop_from_genome(&genome_clone)
                .map_err(|e| ServiceError::Backend(format!("Neuroembryogenesis failed: {}", e)))?;
            
            Ok(neuro.get_progress())
        });
        
        // Wait with timeout - if timeout expires, abort the blocking task
        let progress = match tokio::time::timeout(
            tokio::time::Duration::from_secs(300), // 5 minute timeout
            blocking_handle
        ).await {
            Ok(Ok(result)) => result?,
            Ok(Err(e)) => return Err(ServiceError::Backend(format!("Blocking task panicked: {}", e))),
            Err(_) => {
                // Timeout expired - abort the task (though it may continue running)
                warn!(target: "feagi-services", "Genome loading timed out after 5 minutes - aborting");
                return Err(ServiceError::Backend("Genome loading timed out after 5 minutes".to_string()));
            }
        };
        
        info!(
            target: "feagi-services",
            "Genome loaded: {} cortical areas, {} neurons, {} synapses created",
            progress.cortical_areas_created,
            progress.neurons_created,
            progress.synapses_created
        );
        
        // Return genome info with simulation_timestep
        let (cortical_area_count, brain_region_count) = {
            let manager = self.connectome.read();
            let cortical_area_count = manager.get_cortical_area_count();
            let brain_region_ids = manager.get_brain_region_ids();
            let brain_region_count = brain_region_ids.len();
            (cortical_area_count, brain_region_count)
        };
        
        Ok(GenomeInfo {
            genome_id: "current".to_string(),
            genome_title: "Current Genome".to_string(),
            version: "2.1".to_string(),
            cortical_area_count,
            brain_region_count,
            simulation_timestep,  // From genome physiology
        })
    }

    async fn save_genome(&self, params: SaveGenomeParams) -> ServiceResult<String> {
        info!(target: "feagi-services", "Saving genome to JSON");
        
        // Delegate to ConnectomeManager
        let json_str = self.connectome
            .read()
            .save_genome_to_json(
                params.genome_id,
                params.genome_title,
            )
            .map_err(ServiceError::from)?;
        
        Ok(json_str)
    }

    async fn get_genome_info(&self) -> ServiceResult<GenomeInfo> {
        debug!(target: "feagi-services", "Getting genome info");
        
        // CRITICAL: Minimize lock scope - drop lock immediately after reading values
        let (cortical_area_count, brain_region_count) = {
            let manager = self.connectome.read();
            let cortical_area_count = manager.get_cortical_area_count();
            let brain_region_ids = manager.get_brain_region_ids();
            let brain_region_count = brain_region_ids.len();
            info!(target: "feagi-services", "Reading genome info: {} cortical areas, {} brain regions", cortical_area_count, brain_region_count);
            info!(target: "feagi-services", "Brain region IDs: {:?}", brain_region_ids.iter().take(10).collect::<Vec<_>>());
            (cortical_area_count, brain_region_count)
        }; // Lock dropped here
        
        Ok(GenomeInfo {
            genome_id: "current".to_string(),  // TODO: Track genome_id in ConnectomeManager
            genome_title: "Current Genome".to_string(),
            version: "2.1".to_string(),
            cortical_area_count,
            brain_region_count,
            simulation_timestep: 0.025,  // Default value (TODO: Store in ConnectomeManager)
        })
    }

    async fn validate_genome(&self, json_str: String) -> ServiceResult<bool> {
        debug!(target: "feagi-services", "Validating genome JSON");
        
        // Parse genome
        let genome = feagi_evo::load_genome_from_json(&json_str)
            .map_err(|e| ServiceError::InvalidInput(format!("Failed to parse genome: {}", e)))?;
        
        // Validate genome structure
        let validation = feagi_evo::validate_genome(&genome);
        
        if !validation.errors.is_empty() {
            return Err(ServiceError::InvalidInput(format!(
                "Genome validation failed: {} errors, {} warnings. First error: {}",
                validation.errors.len(),
                validation.warnings.len(),
                validation.errors.first().unwrap_or(&"Unknown error".to_string())
            )));
        }
        
        Ok(true)
    }

    async fn reset_connectome(&self) -> ServiceResult<()> {
        info!(target: "feagi-services", "Resetting connectome");
        
        // Use ConnectomeManager's prepare_for_new_genome method
        self.connectome
            .write()
            .prepare_for_new_genome()
            .map_err(ServiceError::from)?;
        
        info!(target: "feagi-services", "Connectome reset complete");
        Ok(())
    }
    
    async fn update_cortical_area(
        &self,
        cortical_id: &str,
        changes: HashMap<String, Value>,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!(target: "feagi-services", "Updating cortical area: {} with {} changes", cortical_id, changes.len());
        
        // Verify cortical area exists
        {
            let manager = self.connectome.read();
            if !manager.has_cortical_area(cortical_id) {
                return Err(ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: cortical_id.to_string(),
                });
            }
        }
        
        // Classify changes for intelligent routing
        let change_type = CorticalChangeClassifier::classify_changes(&changes);
        CorticalChangeClassifier::log_classification_result(&changes, change_type);
        
        // Route based on change type
        match change_type {
            ChangeType::Parameter => {
                // Fast path: Direct neuron updates (~2-5ms, NO synapse rebuild)
                self.update_parameters_only(cortical_id, changes).await
            }
            ChangeType::Metadata => {
                // Fastest path: Metadata updates only (~1ms)
                self.update_metadata_only(cortical_id, changes).await
            }
            ChangeType::Structural => {
                // Structural path: Requires synapse rebuild (~100-200ms)
                self.update_with_localized_rebuild(cortical_id, changes).await
            }
            ChangeType::Hybrid => {
                // Hybrid path: Handle each type separately
                let separated = CorticalChangeClassifier::separate_changes_by_type(&changes);
                
                // Process in order: metadata first, then parameters, then structural
                if let Some(metadata_changes) = separated.get(&ChangeType::Metadata) {
                    if !metadata_changes.is_empty() {
                        self.update_metadata_only(cortical_id, metadata_changes.clone()).await?;
                    }
                }
                
                if let Some(param_changes) = separated.get(&ChangeType::Parameter) {
                    if !param_changes.is_empty() {
                        self.update_parameters_only(cortical_id, param_changes.clone()).await?;
                    }
                }
                
                if let Some(struct_changes) = separated.get(&ChangeType::Structural) {
                    if !struct_changes.is_empty() {
                        self.update_with_localized_rebuild(cortical_id, struct_changes.clone()).await?;
                    }
                }
                
                // Return updated info
                self.get_cortical_area_info(cortical_id).await
            }
        }
    }
}

impl GenomeServiceImpl {
    /// Fast path: Update only neuron parameters without synapse rebuild
    /// 
    /// Performance: ~1-2µs to queue (non-blocking), applied in next burst cycle
    async fn update_parameters_only(
        &self,
        cortical_id: &str,
        changes: HashMap<String, Value>,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!(target: "feagi-services", "[FAST-UPDATE] Parameter-only update for {}", cortical_id);
        
        // Get cortical index for NPU updates
        let cortical_idx = {
            let manager = self.connectome.read();
            manager.get_cortical_idx(cortical_id)
                .ok_or_else(|| ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: cortical_id.to_string(),
                })?
        };
        
        // Queue parameter updates for burst loop to consume (non-blocking!)
        if let Some(queue) = &self.parameter_queue {
            for (param_name, value) in &changes {
                // Only queue parameters that affect NPU neurons
                let classifier = CorticalChangeClassifier::parameter_changes();
                if classifier.contains(param_name.as_str()) {
                    queue.push(feagi_burst_engine::ParameterUpdate {
                        cortical_idx,
                        cortical_id: cortical_id.to_string(),
                        parameter_name: param_name.clone(),
                        value: value.clone(),
                    });
                    debug!(target: "feagi-services", "[PARAM-QUEUE] Queued {}={} for area {}", param_name, value, cortical_id);
                }
            }
            info!(target: "feagi-services", "[FAST-UPDATE] Queued parameter updates (will apply in next burst)");
        } else {
            warn!(target: "feagi-services", "Parameter queue not available - updates will not affect neurons");
        }
        
        // Update ConnectomeManager metadata for consistency
        {
            let mut manager = self.connectome.write();
            let area = manager.get_cortical_area_mut(cortical_id)
                .ok_or_else(|| ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: cortical_id.to_string(),
                })?;
            
            // Update BDU metadata fields
            for (key, value) in &changes {
                match key.as_str() {
                    "firing_threshold_limit" | "neuron_fire_threshold" => {
                        if let Some(v) = value.as_f64() {
                            area.firing_threshold_limit = v;
                        }
                    }
                    "leak_coefficient" | "neuron_leak_coefficient" => {
                        if let Some(v) = value.as_f64() {
                            area.leak_coefficient = v;
                        }
                    }
                    "refractory_period" | "neuron_refractory_period" => {
                        if let Some(v) = value.as_u64() {
                            area.refractory_period = v as u32;
                        }
                    }
                    "snooze_period" | "neuron_snooze_period" => {
                        if let Some(v) = value.as_u64() {
                            area.snooze_period = v as u32;
                        }
                    }
                    "consecutive_fire_count" | "neuron_consecutive_fire_count" => {
                        if let Some(v) = value.as_u64() {
                            area.consecutive_fire_count = v as u32;
                        }
                    }
                    "plasticity_constant" => {
                        if let Some(v) = value.as_f64() {
                            area.plasticity_constant = v;
                        }
                    }
                    "degeneration" => {
                        if let Some(v) = value.as_f64() {
                            area.degeneration = v;
                        }
                    }
                    "postsynaptic_current" => {
                        if let Some(v) = value.as_f64() {
                            area.postsynaptic_current = v;
                        }
                    }
                    "burst_engine_active" => {
                        if let Some(v) = value.as_bool() {
                            area.burst_engine_active = v;
                        }
                    }
                    _ => {}
                }
            }
        }
        
        info!(target: "feagi-services", "[FAST-UPDATE] Parameter update complete");
        
        // Return updated info
        self.get_cortical_area_info(cortical_id).await
    }
    
    /// Fastest path: Update only metadata without affecting neurons/synapses
    /// 
    /// Performance: ~1ms (metadata changes only)
    async fn update_metadata_only(
        &self,
        cortical_id: &str,
        changes: HashMap<String, Value>,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!(target: "feagi-services", "[METADATA-UPDATE] Metadata-only update for {}", cortical_id);
        
        // Update RuntimeGenome if available
        if let Some(genome) = self.current_genome.write().as_mut() {
            if let Some(area) = genome.cortical_areas.get_mut(cortical_id) {
                for (key, value) in &changes {
                    match key.as_str() {
                        "cortical_name" | "name" => {
                            if let Some(name) = value.as_str() {
                                area.name = name.to_string();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Update ConnectomeManager metadata
        {
            let mut manager = self.connectome.write();
            let area = manager.get_cortical_area_mut(cortical_id)
                .ok_or_else(|| ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: cortical_id.to_string(),
                })?;
            
            // Update metadata fields
            for (key, value) in &changes {
                match key.as_str() {
                    "cortical_name" | "name" => {
                        if let Some(name) = value.as_str() {
                            area.name = name.to_string();
                        }
                    }
                    "visible" => {
                        if let Some(v) = value.as_bool() {
                            area.visible = v;
                        }
                    }
                    _ => {}
                }
            }
        }
        
        info!(target: "feagi-services", "[METADATA-UPDATE] Metadata update complete");
        
        // Return updated info
        self.get_cortical_area_info(cortical_id).await
    }
    
    /// Structural rebuild: For dimension/density changes requiring synapse rebuild
    /// 
    /// Performance: ~100-200ms (localized to one area, not full brain)
    /// 
    /// CRITICAL: This requires:
    /// 1. Deleting all neurons in the area
    /// 2. Deleting all incoming/outgoing synapses (automatic via neuron deletion)
    /// 3. Recreating neurons with new dimensions/density
    /// 4. Rebuilding synapses via cortical mapping
    async fn update_with_localized_rebuild(
        &self,
        cortical_id: &str,
        changes: HashMap<String, Value>,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!(target: "feagi-services", "[STRUCTURAL-REBUILD] Localized rebuild for {}", cortical_id);
        
        // Must run on blocking thread due to heavy ConnectomeManager operations
        let connectome = Arc::clone(&self.connectome);
        let genome_store = Arc::clone(&self.current_genome);
        let cortical_id_owned = cortical_id.to_string();
        
        tokio::task::spawn_blocking(move || {
            Self::do_localized_rebuild(&cortical_id_owned, changes, connectome, genome_store)
        })
        .await
        .map_err(|e| ServiceError::Backend(format!("Rebuild task panicked: {}", e)))?
    }
    
    /// Perform localized rebuild (blocking operation)
    fn do_localized_rebuild(
        cortical_id: &str,
        changes: HashMap<String, Value>,
        connectome: Arc<RwLock<ConnectomeManager<f32>>>,
        genome_store: Arc<RwLock<Option<feagi_evo::RuntimeGenome>>>,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!("[STRUCTURAL-REBUILD] Starting localized rebuild for {}", cortical_id);
        
        // Step 1: Update RuntimeGenome dimensions/density
        let (old_dimensions, old_density, new_dimensions, new_density) = {
            let mut genome_guard = genome_store.write();
            let genome = genome_guard.as_mut()
                .ok_or_else(|| ServiceError::Backend("No genome loaded".to_string()))?;
            
            let area = genome.cortical_areas.get_mut(cortical_id)
                .ok_or_else(|| ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: cortical_id.to_string(),
                })?;
            
            let old_dims = area.dimensions;
            let old_dens = area.neurons_per_voxel;
            
            // Apply dimensional changes
            if let Some(dims) = changes.get("dimensions").or_else(|| changes.get("cortical_dimensions")) {
                if let Some(arr) = dims.as_array() {
                    if arr.len() >= 3 {
                        area.dimensions = feagi_types::Dimensions::new(
                            arr[0].as_u64().unwrap_or(1) as usize,
                            arr[1].as_u64().unwrap_or(1) as usize,
                            arr[2].as_u64().unwrap_or(1) as usize,
                        );
                    }
                }
            }
            
            // Apply density changes
            for density_param in ["neurons_per_voxel", "per_voxel_neuron_cnt", "neuron_density"] {
                if let Some(density) = changes.get(density_param).and_then(|v| v.as_u64()) {
                    area.neurons_per_voxel = density as u32;
                    area.properties.insert(
                        "per_voxel_neuron_cnt".to_string(),
                        serde_json::json!(density),
                    );
                    break;
                }
            }
            
            (old_dims, old_dens, area.dimensions, area.neurons_per_voxel)
        };
        
        info!("[STRUCTURAL-REBUILD] Dimension: {:?} -> {:?}", old_dimensions, new_dimensions);
        info!("[STRUCTURAL-REBUILD] Density: {} -> {} neurons/voxel", old_density, new_density);
        
        // Step 2: Delete all neurons in the cortical area
        let neurons_to_delete = {
            let manager = connectome.read();
            manager.get_neurons_in_area(cortical_id)
        };
        
        let deleted_count = if !neurons_to_delete.is_empty() {
            info!("[STRUCTURAL-REBUILD] Deleting {} existing neurons", neurons_to_delete.len());
            let mut manager = connectome.write();
            manager.delete_neurons_batch(neurons_to_delete)
                .map_err(|e| ServiceError::Backend(format!("Failed to delete neurons: {}", e)))?
        } else {
            0
        };
        
        info!("[STRUCTURAL-REBUILD] Deleted {} neurons", deleted_count);
        
        // Step 3: Update cortical area dimensions in ConnectomeManager
        {
            let mut manager = connectome.write();
            manager.resize_cortical_area(cortical_id, new_dimensions)
                .map_err(|e| ServiceError::Backend(format!("Failed to resize area: {}", e)))?;
            
            // Update neurons_per_voxel
            if let Some(area) = manager.get_cortical_area_mut(cortical_id) {
                area.neurons_per_voxel = new_density;
            }
        }
        
        // Step 4: Recreate neurons with new dimensions/density
        let neurons_created = {
            let mut manager = connectome.write();
            manager.create_neurons_for_area(cortical_id)
                .map_err(|e| ServiceError::Backend(format!("Failed to create neurons: {}", e)))?
        };
        
        info!("[STRUCTURAL-REBUILD] Created {} new neurons", neurons_created);
        
        // Step 5: Rebuild outgoing synapses (this area -> others)
        let outgoing_synapses = {
            let mut manager = connectome.write();
            manager.apply_cortical_mapping(cortical_id)
                .map_err(|e| ServiceError::Backend(format!("Failed to rebuild outgoing synapses: {}", e)))?
        };
        
        info!("[STRUCTURAL-REBUILD] Rebuilt {} outgoing synapses", outgoing_synapses);
        
        // Step 6: Rebuild incoming synapses (others -> this area)
        let incoming_synapses = {
            let genome_guard = genome_store.read();
            let genome = genome_guard.as_ref().unwrap();
            
            let mut total = 0u32;
            for (src_id, src_area) in &genome.cortical_areas {
                if src_id == cortical_id {
                    continue; // Skip self (already handled in outgoing)
                }
                
                // Check if this area maps to our target area
                if let Some(dstmap) = src_area.properties.get("cortical_mapping_dst") {
                    if let Some(obj) = dstmap.as_object() {
                        if obj.contains_key(cortical_id) {
                            // This area has mappings to our target - rebuild them
                            let mut manager = connectome.write();
                            let count = manager.apply_cortical_mapping(src_id)
                                .map_err(|e| ServiceError::Backend(format!("Failed to rebuild incoming synapses from {}: {}", src_id, e)))?;
                            total += count;
                            info!("[STRUCTURAL-REBUILD] Rebuilt {} incoming synapses from {}", count, src_id);
                        }
                    }
                }
            }
            
            total
        };
        
        info!("[STRUCTURAL-REBUILD] Rebuilt {} total incoming synapses", incoming_synapses);
        info!("[STRUCTURAL-REBUILD] ✅ Complete: {} neurons, {} outgoing, {} incoming synapses", 
              neurons_created, outgoing_synapses, incoming_synapses);
        
        // Return updated info
        Self::get_cortical_area_info_blocking(cortical_id, &connectome)
    }
    
    /// Helper to get cortical area info (blocking version for spawn_blocking contexts)
    fn get_cortical_area_info_blocking(
        cortical_id: &str,
        connectome: &Arc<RwLock<ConnectomeManager<f32>>>,
    ) -> ServiceResult<CorticalAreaInfo> {
        let manager = connectome.read();
        
        let area = manager.get_cortical_area(cortical_id)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;
        
        let cortical_idx = manager.get_cortical_idx(cortical_id)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;
        
        let neuron_count = manager.get_neuron_count_in_area(cortical_id);
        let synapse_count = manager.get_synapse_count_in_area(cortical_id);
        
        let cortical_group = match area.area_type {
            feagi_types::AreaType::Sensory => "IPU",
            feagi_types::AreaType::Motor => "OPU",
            feagi_types::AreaType::Memory => "MEMORY",
            feagi_types::AreaType::Custom => "CUSTOM",
        };
        
        Ok(CorticalAreaInfo {
            cortical_id: area.cortical_id.clone(),
            cortical_idx,
            name: area.name.clone(),
            dimensions: (area.dimensions.width, area.dimensions.height, area.dimensions.depth),
            position: area.position,
            area_type: format!("{:?}", area.area_type),
            cortical_group: cortical_group.to_string(),
            neuron_count,
            synapse_count,
            visible: area.visible,
            sub_group: area.sub_group.clone(),
            neurons_per_voxel: area.neurons_per_voxel,
            postsynaptic_current: area.postsynaptic_current,
            plasticity_constant: area.plasticity_constant,
            degeneration: area.degeneration,
            psp_uniform_distribution: area.psp_uniform_distribution,
            firing_threshold_increment: area.firing_threshold_increment,
            firing_threshold_limit: area.firing_threshold_limit,
            consecutive_fire_count: area.consecutive_fire_count,
            snooze_period: area.snooze_period,
            refractory_period: area.refractory_period,
            leak_coefficient: area.leak_coefficient,
            leak_variability: area.leak_variability,
            burst_engine_active: area.burst_engine_active,
            properties: HashMap::new(),
        })
    }
    
    /// Helper to get cortical area info
    async fn get_cortical_area_info(&self, cortical_id: &str) -> ServiceResult<CorticalAreaInfo> {
        let manager = self.connectome.read();
        
        let area = manager.get_cortical_area(cortical_id)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;
        
        let cortical_idx = manager.get_cortical_idx(cortical_id)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;
        
        let neuron_count = manager.get_neuron_count_in_area(cortical_id);
        let synapse_count = manager.get_synapse_count_in_area(cortical_id);
        
        // Derive cortical_group from area_type
        let cortical_group = match area.area_type {
            feagi_types::AreaType::Sensory => "IPU",
            feagi_types::AreaType::Motor => "OPU",
            feagi_types::AreaType::Memory => "MEMORY",
            feagi_types::AreaType::Custom => "CUSTOM",
        };
        
        Ok(CorticalAreaInfo {
            cortical_id: area.cortical_id.clone(),
            cortical_idx,
            name: area.name.clone(),
            dimensions: (area.dimensions.width, area.dimensions.height, area.dimensions.depth),
            position: area.position,
            area_type: format!("{:?}", area.area_type),
            cortical_group: cortical_group.to_string(),
            neuron_count,
            synapse_count,
            visible: area.visible,
            sub_group: area.sub_group.clone(),
            neurons_per_voxel: area.neurons_per_voxel,
            postsynaptic_current: area.postsynaptic_current,
            plasticity_constant: area.plasticity_constant,
            degeneration: area.degeneration,
            psp_uniform_distribution: area.psp_uniform_distribution,
            firing_threshold_increment: area.firing_threshold_increment,
            firing_threshold_limit: area.firing_threshold_limit,
            consecutive_fire_count: area.consecutive_fire_count,
            snooze_period: area.snooze_period,
            refractory_period: area.refractory_period,
            leak_coefficient: area.leak_coefficient,
            leak_variability: area.leak_variability,
            burst_engine_active: area.burst_engine_active,
            properties: HashMap::new(),
        })
    }
}

