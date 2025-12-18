// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Genome service implementation.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::traits::GenomeService;
use crate::types::*;
use async_trait::async_trait;
use feagi_bdu::models::CorticalAreaExt;
use feagi_bdu::neuroembryogenesis::Neuroembryogenesis;
use feagi_bdu::ConnectomeManager;
use feagi_data_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalID,
};
use feagi_npu_burst_engine::ParameterUpdateQueue;
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, trace, warn};

use crate::genome::{ChangeType, CorticalChangeClassifier};

/// Default implementation of GenomeService
pub struct GenomeServiceImpl {
    connectome: Arc<RwLock<ConnectomeManager>>,
    parameter_queue: Option<ParameterUpdateQueue>,
    /// Currently loaded genome (source of truth for structural changes)
    /// This is updated when genome is loaded or when cortical areas are modified
    current_genome: Arc<RwLock<Option<feagi_evo::RuntimeGenome>>>,
    /// Counter tracking how many genomes have been loaded (increments on each load)
    genome_load_counter: Arc<RwLock<i32>>,
    /// Timestamp of when the current genome was loaded
    genome_load_timestamp: Arc<RwLock<Option<i64>>>,
}

impl GenomeServiceImpl {
    pub fn new(connectome: Arc<RwLock<ConnectomeManager>>) -> Self {
        Self {
            connectome,
            parameter_queue: None,
            current_genome: Arc::new(RwLock::new(None)),
            genome_load_counter: Arc::new(RwLock::new(0)),
            genome_load_timestamp: Arc::new(RwLock::new(None)),
        }
    }

    pub fn new_with_parameter_queue(
        connectome: Arc<RwLock<ConnectomeManager>>,
        parameter_queue: ParameterUpdateQueue,
    ) -> Self {
        Self {
            connectome,
            parameter_queue: Some(parameter_queue),
            current_genome: Arc::new(RwLock::new(None)),
            genome_load_counter: Arc::new(RwLock::new(0)),
            genome_load_timestamp: Arc::new(RwLock::new(None)),
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
        info!(target: "feagi-services", "Storing RuntimeGenome with {} cortical areas, {} morphologies",
            genome.cortical_areas.len(), genome.morphologies.iter().count());
        *self.current_genome.write() = Some(genome.clone());

        // Increment genome load counter and set timestamp
        let genome_num = {
            let mut counter = self.genome_load_counter.write();
            *counter += 1;
            *counter
        };

        let genome_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs() as i64);

        *self.genome_load_timestamp.write() = genome_timestamp;

        info!(target: "feagi-services", "Genome load #{}, timestamp: {:?}", genome_num, genome_timestamp);

        // Load into connectome via ConnectomeManager
        // This involves synaptogenesis which can be CPU-intensive, so run it on a blocking thread
        // CRITICAL: Add timeout to prevent hanging during shutdown
        // Note: spawn_blocking tasks cannot be cancelled, but timeout ensures we don't wait forever
        // CRITICAL FIX: Don't hold write lock during entire operation - let neuroembryogenesis manage locks
        // This prevents deadlock when neuroembryogenesis tries to acquire its own write locks
        let connectome_clone = self.connectome.clone();
        let blocking_handle = tokio::task::spawn_blocking(
            move || -> Result<feagi_bdu::neuroembryogenesis::DevelopmentProgress, ServiceError> {
                // Acquire write lock only for prepare/resize operations
                let mut genome_clone = genome;
                let (prepare_result, resize_result) = {
                    let mut manager = connectome_clone.write();
                    let prepare_result = manager.prepare_for_new_genome();
                    let resize_result = prepare_result
                        .as_ref()
                        .ok()
                        .map(|_| manager.resize_for_genome(&genome_clone));
                    (prepare_result, resize_result)
                }; // Lock released here

                prepare_result.map_err(ServiceError::from)?;
                if let Some(resize_result) = resize_result {
                    resize_result.map_err(ServiceError::from)?;
                }

                // Now call develop_from_genome without holding the lock
                // It will acquire its own locks internally
                let manager_arc = feagi_bdu::ConnectomeManager::instance();
                let mut neuro = Neuroembryogenesis::new(manager_arc.clone());
                neuro.develop_from_genome(&genome_clone).map_err(|e| {
                    ServiceError::Backend(format!("Neuroembryogenesis failed: {}", e))
                })?;

                // After neuroembryogenesis, update genome metadata with root_region_id
                let root_region_id = manager_arc.read().get_root_region_id();
                if let Some(root_id) = root_region_id {
                    genome_clone.metadata.brain_regions_root = Some(root_id);
                    info!(target: "feagi-services", "✅ Set genome brain_regions_root: {}", genome_clone.metadata.brain_regions_root.as_ref().unwrap());
                } else {
                    warn!(target: "feagi-services", "⚠️ No root region found after neuroembryogenesis");
                }

                Ok(neuro.get_progress())
            },
        );

        // Wait with timeout - if timeout expires, abort the blocking task
        let progress = match tokio::time::timeout(
            tokio::time::Duration::from_secs(300), // 5 minute timeout
            blocking_handle,
        )
        .await
        {
            Ok(Ok(result)) => result?,
            Ok(Err(e)) => {
                return Err(ServiceError::Backend(format!(
                    "Blocking task panicked: {}",
                    e
                )))
            }
            Err(_) => {
                // Timeout expired - abort the task (though it may continue running)
                warn!(target: "feagi-services", "Genome loading timed out after 5 minutes - aborting");
                return Err(ServiceError::Backend(
                    "Genome loading timed out after 5 minutes".to_string(),
                ));
            }
        };

        info!(
            target: "feagi-services",
            "Genome loaded: {} cortical areas, {} neurons, {} synapses created",
            progress.cortical_areas_created,
            progress.neurons_created,
            progress.synapses_created
        );

        // CRITICAL: Sync auto-generated brain regions back to RuntimeGenome
        // BDU may auto-generate brain regions if the genome didn't have any.
        // We need to sync these back to current_genome so they're included when saving.
        let brain_regions_from_bdu = {
            let manager = self.connectome.read();
            let hierarchy = manager.get_brain_region_hierarchy();
            hierarchy.get_all_regions()
        };

        if !brain_regions_from_bdu.is_empty() {
            let mut current_genome_guard = self.current_genome.write();
            if let Some(ref mut genome) = *current_genome_guard {
                // Only update if BDU has more regions (handles auto-generation case)
                if brain_regions_from_bdu.len() > genome.brain_regions.len() {
                    info!(
                        target: "feagi-services",
                        "Syncing {} auto-generated brain regions from BDU to RuntimeGenome",
                        brain_regions_from_bdu.len()
                    );
                    genome.brain_regions = brain_regions_from_bdu;
                }
            }
        }

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
            simulation_timestep,          // From genome physiology
            genome_num: Some(genome_num), // Actual load counter
            genome_timestamp,             // Timestamp when genome was loaded
        })
    }

    async fn save_genome(&self, params: SaveGenomeParams) -> ServiceResult<String> {
        info!(target: "feagi-services", "Saving genome to JSON");

        // Check if we have a RuntimeGenome stored (includes morphologies, physiology, etc.)
        let genome_opt = self.current_genome.read().clone();

        let mut genome = genome_opt.ok_or_else(|| {
            ServiceError::Internal(
                "No RuntimeGenome stored. Genome must be loaded via load_genome() before it can be saved.".to_string()
            )
        })?;

        info!(target: "feagi-services", "✅ RuntimeGenome loaded, exporting in flat format v3.0");

        // Update metadata if provided
        if let Some(id) = params.genome_id {
            genome.metadata.genome_id = id;
        }
        if let Some(title) = params.genome_title {
            genome.metadata.genome_title = title;
        }

        // Use the full RuntimeGenome saver (produces flat format v3.0)
        let json_str = feagi_evo::save_genome_to_json(&genome)
            .map_err(|e| ServiceError::Internal(format!("Failed to save genome: {}", e)))?;

        info!(target: "feagi-services", "✅ Genome exported successfully (flat format v3.0)");
        Ok(json_str)
    }

    async fn get_genome_info(&self) -> ServiceResult<GenomeInfo> {
        trace!(target: "feagi-services", "Getting genome info");

        // CRITICAL: Minimize lock scope - drop lock immediately after reading values
        let (cortical_area_count, brain_region_count) = {
            let manager = self.connectome.read();
            let cortical_area_count = manager.get_cortical_area_count();
            let brain_region_ids = manager.get_brain_region_ids();
            let brain_region_count = brain_region_ids.len();
            trace!(
                target: "feagi-services",
                "Reading genome info: {} cortical areas, {} brain regions",
                cortical_area_count,
                brain_region_count
            );
            trace!(
                target: "feagi-services",
                "Brain region IDs: {:?}",
                brain_region_ids.iter().take(10).collect::<Vec<_>>()
            );
            (cortical_area_count, brain_region_count)
        }; // Lock dropped here

        // Get simulation_timestep from stored genome if available
        let simulation_timestep = {
            let genome_opt = self.current_genome.read();
            genome_opt
                .as_ref()
                .map(|g| g.physiology.simulation_timestep)
                .unwrap_or(0.025) // Default if no genome loaded
        };

        // Get actual genome load counter and timestamp
        let genome_num = {
            let counter = self.genome_load_counter.read();
            if *counter > 0 {
                Some(*counter)
            } else {
                None // No genome loaded yet
            }
        };

        let genome_timestamp = *self.genome_load_timestamp.read();

        Ok(GenomeInfo {
            genome_id: "current".to_string(),
            genome_title: "Current Genome".to_string(),
            version: "2.1".to_string(),
            cortical_area_count,
            brain_region_count,
            simulation_timestep,
            genome_num,
            genome_timestamp,
        })
    }

    async fn validate_genome(&self, json_str: String) -> ServiceResult<bool> {
        trace!(target: "feagi-services", "Validating genome JSON");

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
                validation
                    .errors
                    .first()
                    .unwrap_or(&"Unknown error".to_string())
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

    async fn create_cortical_areas(
        &self,
        params: Vec<CreateCorticalAreaParams>,
    ) -> ServiceResult<Vec<CorticalAreaInfo>> {
        info!(target: "feagi-services", "Creating {} new cortical areas via GenomeService", params.len());

        // Step 1: Build CorticalArea structures
        let mut areas_to_add = Vec::new();
        for param in &params {
            // Convert String to CorticalID
            let cortical_id_typed = CorticalID::try_from_base_64(&param.cortical_id)
                .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;

            // Get cortical area type from the cortical ID
            let area_type = cortical_id_typed.as_cortical_type().map_err(|e| {
                ServiceError::InvalidInput(format!("Failed to determine cortical area type: {}", e))
            })?;

            // Create CorticalArea
            let mut area = CorticalArea::new(
                cortical_id_typed,
                0, // Auto-assigned by ConnectomeManager
                param.name.clone(),
                CorticalAreaDimensions::new(
                    param.dimensions.0 as u32,
                    param.dimensions.1 as u32,
                    param.dimensions.2 as u32,
                )?,
                param.position.into(), // Convert (i32, i32, i32) to GenomeCoordinate3D
                area_type,
            )?;

            // Apply all neural parameters
            if let Some(visible) = param.visible {
                area.add_property_mut("visible".to_string(), serde_json::json!(visible));
            }
            if let Some(sub_group) = &param.sub_group {
                area.add_property_mut("sub_group".to_string(), serde_json::json!(sub_group));
            }
            if let Some(neurons_per_voxel) = param.neurons_per_voxel {
                area.add_property_mut(
                    "neurons_per_voxel".to_string(),
                    serde_json::json!(neurons_per_voxel),
                );
            }
            if let Some(postsynaptic_current) = param.postsynaptic_current {
                area.add_property_mut(
                    "postsynaptic_current".to_string(),
                    serde_json::json!(postsynaptic_current),
                );
            }
            if let Some(plasticity_constant) = param.plasticity_constant {
                area.add_property_mut(
                    "plasticity_constant".to_string(),
                    serde_json::json!(plasticity_constant),
                );
            }
            if let Some(degeneration) = param.degeneration {
                area.add_property_mut("degeneration".to_string(), serde_json::json!(degeneration));
            }
            if let Some(psp_uniform_distribution) = param.psp_uniform_distribution {
                area.add_property_mut(
                    "psp_uniform_distribution".to_string(),
                    serde_json::json!(psp_uniform_distribution),
                );
            }
            if let Some(firing_threshold_increment) = param.firing_threshold_increment {
                area.add_property_mut(
                    "firing_threshold_increment".to_string(),
                    serde_json::json!(firing_threshold_increment),
                );
            }
            if let Some(firing_threshold_limit) = param.firing_threshold_limit {
                area.add_property_mut(
                    "firing_threshold_limit".to_string(),
                    serde_json::json!(firing_threshold_limit),
                );
            }
            if let Some(consecutive_fire_count) = param.consecutive_fire_count {
                area.add_property_mut(
                    "consecutive_fire_limit".to_string(),
                    serde_json::json!(consecutive_fire_count),
                );
            }
            if let Some(snooze_period) = param.snooze_period {
                area.add_property_mut(
                    "snooze_period".to_string(),
                    serde_json::json!(snooze_period),
                );
            }
            if let Some(refractory_period) = param.refractory_period {
                area.add_property_mut(
                    "refractory_period".to_string(),
                    serde_json::json!(refractory_period),
                );
            }
            if let Some(leak_coefficient) = param.leak_coefficient {
                area.add_property_mut(
                    "leak_coefficient".to_string(),
                    serde_json::json!(leak_coefficient),
                );
            }
            if let Some(leak_variability) = param.leak_variability {
                area.add_property_mut(
                    "leak_variability".to_string(),
                    serde_json::json!(leak_variability),
                );
            }
            if let Some(burst_engine_active) = param.burst_engine_active {
                area.add_property_mut(
                    "burst_engine_active".to_string(),
                    serde_json::json!(burst_engine_active),
                );
            }
            if let Some(properties) = &param.properties {
                area.properties = properties.clone();
            }

            areas_to_add.push(area);
        }

        // Step 2: Add to runtime genome (source of truth)
        {
            let mut genome_lock = self.current_genome.write();
            if let Some(ref mut genome) = *genome_lock {
                for area in &areas_to_add {
                    genome
                        .cortical_areas
                        .insert(area.cortical_id.clone(), area.clone());
                    info!(target: "feagi-services", "Added {} to runtime genome", area.cortical_id.as_base_64());
                }
            } else {
                return Err(ServiceError::Backend("No genome loaded".to_string()));
            }
        }

        // Step 3: Get genome for neuroembryogenesis context
        let genome_clone = {
            let genome_lock = self.current_genome.read();
            genome_lock
                .as_ref()
                .ok_or_else(|| ServiceError::Backend("No genome loaded".to_string()))?
                .clone()
        };

        // Step 4: Call neuroembryogenesis to create structures, neurons, and synapses
        let (neurons_created, synapses_created) = {
            let connectome_clone = self.connectome.clone();
            tokio::task::spawn_blocking(move || {
                let mut neuro = Neuroembryogenesis::new(connectome_clone);
                neuro.add_cortical_areas(areas_to_add.clone(), &genome_clone)
            })
            .await
            .map_err(|e| ServiceError::Backend(format!("Neuroembryogenesis task failed: {}", e)))?
            .map_err(|e| ServiceError::Backend(format!("Neuroembryogenesis failed: {}", e)))?
        };

        info!(target: "feagi-services",
              "✅ Created {} cortical areas: {} neurons, {} synapses",
              params.len(), neurons_created, synapses_created);

        // Step 5: Fetch and return area information
        let mut created_areas = Vec::new();
        for param in &params {
            match self.get_cortical_area_info(&param.cortical_id).await {
                Ok(area_info) => created_areas.push(area_info),
                Err(e) => {
                    warn!(target: "feagi-services", "Created area {} but failed to fetch info: {}", param.cortical_id, e);
                    return Err(ServiceError::Backend(format!(
                        "Created areas but failed to fetch info for {}: {}",
                        param.cortical_id, e
                    )));
                }
            }
        }

        Ok(created_areas)
    }

    async fn update_cortical_area(
        &self,
        cortical_id: &str,
        changes: HashMap<String, Value>,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!(target: "feagi-services", "Updating cortical area: {} with {} changes", cortical_id, changes.len());

        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;

        // Verify cortical area exists
        {
            let manager = self.connectome.read();
            if !manager.has_cortical_area(&cortical_id_typed) {
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
                self.update_with_localized_rebuild(cortical_id, changes)
                    .await
            }
            ChangeType::Hybrid => {
                // Hybrid path: Handle each type separately
                let separated = CorticalChangeClassifier::separate_changes_by_type(&changes);

                // Process in order: metadata first, then parameters, then structural
                if let Some(metadata_changes) = separated.get(&ChangeType::Metadata) {
                    if !metadata_changes.is_empty() {
                        self.update_metadata_only(cortical_id, metadata_changes.clone())
                            .await?;
                    }
                }

                if let Some(param_changes) = separated.get(&ChangeType::Parameter) {
                    if !param_changes.is_empty() {
                        self.update_parameters_only(cortical_id, param_changes.clone())
                            .await?;
                    }
                }

                if let Some(struct_changes) = separated.get(&ChangeType::Structural) {
                    if !struct_changes.is_empty() {
                        self.update_with_localized_rebuild(cortical_id, struct_changes.clone())
                            .await?;
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

        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;
        // Get cortical index for NPU updates
        let cortical_idx = {
            let manager = self.connectome.read();
            manager
                .get_cortical_idx(&cortical_id_typed)
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
                    queue.push(feagi_npu_burst_engine::ParameterUpdate {
                        cortical_idx,
                        cortical_id: cortical_id.to_string(),
                        parameter_name: param_name.clone(),
                        value: value.clone(),
                    });
                    trace!(
                        target: "feagi-services",
                        "[PARAM-QUEUE] Queued {}={} for area {}",
                        param_name,
                        value,
                        cortical_id
                    );
                }
            }
            info!(target: "feagi-services", "[FAST-UPDATE] Queued parameter updates (will apply in next burst)");
        } else {
            warn!(target: "feagi-services", "Parameter queue not available - updates will not affect neurons");
        }

        // Update ConnectomeManager metadata for consistency
        {
            let mut manager = self.connectome.write();
            let area = manager
                .get_cortical_area_mut(&cortical_id_typed)
                .ok_or_else(|| ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: cortical_id.to_string(),
                })?;

            // Update BDU metadata fields
            for (key, value) in &changes {
                match key.as_str() {
                    "firing_threshold_limit" | "neuron_fire_threshold" => {
                        if let Some(v) = value.as_f64() {
                            area.add_property_mut(
                                "firing_threshold_limit".to_string(),
                                serde_json::json!(v),
                            );
                        }
                    }
                    "leak_coefficient" | "neuron_leak_coefficient" => {
                        if let Some(v) = value.as_f64() {
                            area.add_property_mut(
                                "leak_coefficient".to_string(),
                                serde_json::json!(v),
                            );
                        }
                    }
                    "refractory_period" | "neuron_refractory_period" => {
                        if let Some(v) = value.as_u64() {
                            area.add_property_mut(
                                "refractory_period".to_string(),
                                serde_json::json!(v as u32),
                            );
                        }
                    }
                    "snooze_period" | "neuron_snooze_period" => {
                        if let Some(v) = value.as_u64() {
                            area.add_property_mut(
                                "snooze_period".to_string(),
                                serde_json::json!(v as u32),
                            );
                        }
                    }
                    "consecutive_fire_count" | "neuron_consecutive_fire_count" => {
                        if let Some(v) = value.as_u64() {
                            area.add_property_mut(
                                "consecutive_fire_limit".to_string(),
                                serde_json::json!(v as u32),
                            );
                        }
                    }
                    "plasticity_constant" => {
                        if let Some(v) = value.as_f64() {
                            area.add_property_mut(
                                "plasticity_constant".to_string(),
                                serde_json::json!(v),
                            );
                        }
                    }
                    "degeneration" => {
                        if let Some(v) = value.as_f64() {
                            area.add_property_mut("degeneration".to_string(), serde_json::json!(v));
                        }
                    }
                    "postsynaptic_current" => {
                        if let Some(v) = value.as_f64() {
                            area.add_property_mut(
                                "postsynaptic_current".to_string(),
                                serde_json::json!(v),
                            );
                        }
                    }
                    "burst_engine_active" => {
                        if let Some(v) = value.as_bool() {
                            area.add_property_mut(
                                "burst_engine_active".to_string(),
                                serde_json::json!(v),
                            );
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

        // Convert cortical_id to CorticalID
        let cortical_id_typed = feagi_evo::string_to_cortical_id(cortical_id).map_err(|e| {
            ServiceError::InvalidInput(format!("Invalid cortical ID '{}': {}", cortical_id, e))
        })?;

        // Update RuntimeGenome if available
        if let Some(genome) = self.current_genome.write().as_mut() {
            if let Some(area) = genome.cortical_areas.get_mut(&cortical_id_typed) {
                for (key, value) in &changes {
                    match key.as_str() {
                        "cortical_name" | "name" => {
                            if let Some(name) = value.as_str() {
                                area.name = name.to_string();
                            }
                        }
                        "coordinates_3d" | "coordinate_3d" | "coordinates" | "position" => {
                            // Parse coordinates - support both array [x, y, z] and object {"x": x, "y": y, "z": z}
                            if let Some(arr) = value.as_array() {
                                // Array format: [x, y, z]
                                if arr.len() >= 3 {
                                    let x = arr[0].as_i64().unwrap_or(0) as i32;
                                    let y = arr[1].as_i64().unwrap_or(0) as i32;
                                    let z = arr[2].as_i64().unwrap_or(0) as i32;
                                    area.position = (x, y, z).into();
                                    info!(target: "feagi-services", "[GENOME-UPDATE] Updated position (array format): ({}, {}, {})", x, y, z);
                                }
                            } else if let Some(obj) = value.as_object() {
                                // Object format: {"x": x, "y": y, "z": z}
                                let x = obj.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                let y = obj.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                let z = obj.get("z").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                area.position = (x, y, z).into();
                                info!(target: "feagi-services", "[GENOME-UPDATE] Updated position (object format): ({}, {}, {})", x, y, z);
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
            let area = manager
                .get_cortical_area_mut(&cortical_id_typed)
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
                            area.add_property_mut("visible".to_string(), serde_json::json!(v));
                        }
                    }
                    "coordinates_3d" | "coordinate_3d" | "coordinates" | "position" => {
                        // Parse coordinates - support both array [x, y, z] and object {"x": x, "y": y, "z": z}
                        if let Some(arr) = value.as_array() {
                            // Array format: [x, y, z]
                            if arr.len() >= 3 {
                                let x = arr[0].as_i64().unwrap_or(0) as i32;
                                let y = arr[1].as_i64().unwrap_or(0) as i32;
                                let z = arr[2].as_i64().unwrap_or(0) as i32;
                                area.position = (x, y, z).into();
                                info!(target: "feagi-services", "[CONNECTOME-UPDATE] Updated position (array format): ({}, {}, {})", x, y, z);
                            }
                        } else if let Some(obj) = value.as_object() {
                            // Object format: {"x": x, "y": y, "z": z}
                            let x = obj.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                            let y = obj.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                            let z = obj.get("z").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                            area.position = (x, y, z).into();
                            info!(target: "feagi-services", "[CONNECTOME-UPDATE] Updated position (object format): ({}, {}, {})", x, y, z);
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

        // Validate cortical ID format
        let _cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;

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
        connectome: Arc<RwLock<ConnectomeManager>>,
        genome_store: Arc<RwLock<Option<feagi_evo::RuntimeGenome>>>,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!(
            "[STRUCTURAL-REBUILD] Starting localized rebuild for {}",
            cortical_id
        );

        // Convert cortical_id to CorticalID
        let cortical_id_typed = feagi_evo::string_to_cortical_id(cortical_id).map_err(|e| {
            ServiceError::InvalidInput(format!("Invalid cortical ID '{}': {}", cortical_id, e))
        })?;

        // Step 1: Update RuntimeGenome dimensions/density
        let (old_dimensions, old_density, new_dimensions, new_density) = {
            let mut genome_guard = genome_store.write();
            let genome = genome_guard
                .as_mut()
                .ok_or_else(|| ServiceError::Backend("No genome loaded".to_string()))?;

            let area = genome
                .cortical_areas
                .get_mut(&cortical_id_typed)
                .ok_or_else(|| ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: cortical_id.to_string(),
                })?;

            let old_dims = area.dimensions;
            let old_dens = area.neurons_per_voxel();

            // Apply dimensional changes
            // CRITICAL: For IPU/OPU areas, cortical_dimensions_per_device must be multiplied by dev_count
            let is_per_device = changes.contains_key("cortical_dimensions_per_device");

            if let Some(dims) = changes
                .get("dimensions")
                .or_else(|| changes.get("cortical_dimensions"))
                .or_else(|| changes.get("cortical_dimensions_per_device"))
            {
                let (width, height, depth) = if let Some(arr) = dims.as_array() {
                    // Handle array format: [width, height, depth]
                    if arr.len() >= 3 {
                        (
                            arr[0].as_u64().unwrap_or(1) as usize,
                            arr[1].as_u64().unwrap_or(1) as usize,
                            arr[2].as_u64().unwrap_or(1) as usize,
                        )
                    } else {
                        (1, 1, 1)
                    }
                } else if let Some(obj) = dims.as_object() {
                    // Handle object format: {"x": width, "y": height, "z": depth}
                    (
                        obj.get("x").and_then(|v| v.as_u64()).unwrap_or(1) as usize,
                        obj.get("y").and_then(|v| v.as_u64()).unwrap_or(1) as usize,
                        obj.get("z").and_then(|v| v.as_u64()).unwrap_or(1) as usize,
                    )
                } else {
                    (1, 1, 1)
                };

                // If this is per-device dimensions, multiply depth by dev_count to get total dimensions
                let final_depth = if is_per_device {
                    // Get dev_count from changes or from existing area properties
                    let dev_count = changes
                        .get("dev_count")
                        .and_then(|v| v.as_u64())
                        .or_else(|| area.properties.get("dev_count").and_then(|v| v.as_u64()))
                        .unwrap_or(1) as usize;

                    info!("[STRUCTURAL-REBUILD] Per-device dimensions: [{}, {}, {}] × dev_count={} → total depth={}",
                          width, height, depth, dev_count, depth * dev_count);

                    depth * dev_count
                } else {
                    depth
                };

                area.dimensions =
                    CorticalAreaDimensions::new(width as u32, height as u32, final_depth as u32)?;
            }

            // Apply density changes
            // Update neurons_per_voxel from any of the legacy parameter names
            for density_param in [
                "neurons_per_voxel",
                "per_voxel_neuron_cnt",
                "neuron_density",
            ] {
                if let Some(density) = changes.get(density_param).and_then(|v| v.as_u64()) {
                    area.add_property_mut(
                        "neurons_per_voxel".to_string(),
                        serde_json::json!(density as u32),
                    );
                    break;
                }
            }

            // Store dev_count and cortical_dimensions_per_device in properties for IPU/OPU areas
            if let Some(dev_count) = changes.get("dev_count") {
                area.properties
                    .insert("dev_count".to_string(), dev_count.clone());
            }
            if let Some(per_device_dims) = changes.get("cortical_dimensions_per_device") {
                area.properties.insert(
                    "cortical_dimensions_per_device".to_string(),
                    per_device_dims.clone(),
                );
            }

            (
                old_dims,
                old_dens,
                area.dimensions,
                area.neurons_per_voxel(),
            )
        };

        info!(
            "[STRUCTURAL-REBUILD] Dimension: {:?} -> {:?}",
            old_dimensions, new_dimensions
        );
        info!(
            "[STRUCTURAL-REBUILD] Density: {} -> {} neurons/voxel",
            old_density, new_density
        );

        // Step 2: Delete all neurons in the cortical area
        let neurons_to_delete = {
            let manager = connectome.read();
            manager.get_neurons_in_area(&cortical_id_typed)
        };

        let deleted_count = if !neurons_to_delete.is_empty() {
            info!(
                "[STRUCTURAL-REBUILD] Deleting {} existing neurons",
                neurons_to_delete.len()
            );
            let mut manager = connectome.write();
            manager
                .delete_neurons_batch(neurons_to_delete)
                .map_err(|e| ServiceError::Backend(format!("Failed to delete neurons: {}", e)))?
        } else {
            0
        };

        info!("[STRUCTURAL-REBUILD] Deleted {} neurons", deleted_count);

        // Step 3: Update cortical area dimensions and properties in ConnectomeManager
        {
            let mut manager = connectome.write();
            manager
                .resize_cortical_area(&cortical_id_typed, new_dimensions)
                .map_err(|e| ServiceError::Backend(format!("Failed to resize area: {}", e)))?;

            // Update neurons_per_voxel and properties
            if let Some(area) = manager.get_cortical_area_mut(&cortical_id_typed) {
                area.add_property_mut(
                    "neurons_per_voxel".to_string(),
                    serde_json::json!(new_density),
                );

                // Store IPU/OPU properties
                if let Some(dev_count) = changes.get("dev_count") {
                    area.properties
                        .insert("dev_count".to_string(), dev_count.clone());
                }
                if let Some(per_device_dims) = changes.get("cortical_dimensions_per_device") {
                    area.properties.insert(
                        "cortical_dimensions_per_device".to_string(),
                        per_device_dims.clone(),
                    );
                }
            }
        }

        // Step 4: Recreate neurons with new dimensions/density
        let neurons_created = {
            let mut manager = connectome.write();
            manager
                .create_neurons_for_area(&cortical_id_typed)
                .map_err(|e| ServiceError::Backend(format!("Failed to create neurons: {}", e)))?
        };

        info!(
            "[STRUCTURAL-REBUILD] Created {} new neurons",
            neurons_created
        );

        // Step 5: Rebuild outgoing synapses (this area -> others)
        let outgoing_synapses = {
            let mut manager = connectome.write();
            manager
                .create_neurons_for_area(&cortical_id_typed)
                .map_err(|e| {
                    ServiceError::Backend(format!("Failed to rebuild outgoing synapses: {}", e))
                })?
        };

        info!(
            "[STRUCTURAL-REBUILD] Rebuilt {} outgoing synapses",
            outgoing_synapses
        );

        // Step 6: Rebuild incoming synapses (others -> this area)
        let incoming_synapses = {
            let genome_guard = genome_store.read();
            let genome = genome_guard.as_ref().unwrap();

            let mut total = 0u32;
            for (src_id, src_area) in &genome.cortical_areas {
                if src_id == &cortical_id_typed {
                    continue; // Skip self (already handled in outgoing)
                }

                // Check if this area maps to our target area
                if let Some(dstmap) = src_area.properties.get("cortical_mapping_dst") {
                    if let Some(obj) = dstmap.as_object() {
                        if obj.contains_key(cortical_id) {
                            // This area has mappings to our target - rebuild them
                            let mut manager = connectome.write();
                            let count = manager.apply_cortical_mapping(&src_id).map_err(|e| {
                                ServiceError::Backend(format!(
                                    "Failed to rebuild incoming synapses from {}: {}",
                                    src_id, e
                                ))
                            })?;
                            total += count;
                            info!(
                                "[STRUCTURAL-REBUILD] Rebuilt {} incoming synapses from {}",
                                count, src_id
                            );
                        }
                    }
                }
            }

            total
        };

        info!(
            "[STRUCTURAL-REBUILD] Rebuilt {} total incoming synapses",
            incoming_synapses
        );
        info!(
            "[STRUCTURAL-REBUILD] ✅ Complete: {} neurons, {} outgoing, {} incoming synapses",
            neurons_created, outgoing_synapses, incoming_synapses
        );

        // Return updated info
        Self::get_cortical_area_info_blocking(cortical_id, &connectome)
    }

    /// Helper to get cortical area info (blocking version for spawn_blocking contexts)
    fn get_cortical_area_info_blocking(
        cortical_id: &str,
        connectome: &Arc<RwLock<ConnectomeManager>>,
    ) -> ServiceResult<CorticalAreaInfo> {
        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;

        let manager = connectome.read();

        let area = manager
            .get_cortical_area(&cortical_id_typed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;

        let cortical_idx = manager
            .get_cortical_idx(&cortical_id_typed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;

        let neuron_count = manager.get_neuron_count_in_area(&cortical_id_typed);
        let synapse_count = manager.get_synapse_count_in_area(&cortical_id_typed);

        let cortical_group = area.get_cortical_group();

        Ok(CorticalAreaInfo {
            cortical_id: area.cortical_id.as_base_64(),
            cortical_id_s: area.cortical_id.to_string(), // Human-readable ASCII string
            cortical_idx,
            name: area.name.clone(),
            dimensions: (
                area.dimensions.width as usize,
                area.dimensions.height as usize,
                area.dimensions.depth as usize,
            ),
            position: area.position.into(),
            area_type: cortical_group
                .clone()
                .unwrap_or_else(|| "CUSTOM".to_string()),
            cortical_group: cortical_group.unwrap_or_else(|| "CUSTOM".to_string()),
            neuron_count,
            synapse_count,
            visible: area.visible(),
            sub_group: area.sub_group(),
            neurons_per_voxel: area.neurons_per_voxel(),
            postsynaptic_current: area.postsynaptic_current() as f64,
            plasticity_constant: area.plasticity_constant() as f64,
            degeneration: area.degeneration() as f64,
            psp_uniform_distribution: area.psp_uniform_distribution() != 0.0,
            firing_threshold_increment: area.firing_threshold_increment() as f64,
            firing_threshold_limit: area.firing_threshold_limit() as f64,
            consecutive_fire_count: area.consecutive_fire_count(),
            snooze_period: area.snooze_period() as u32,
            refractory_period: area.refractory_period() as u32,
            leak_coefficient: area.leak_coefficient() as f64,
            leak_variability: area.leak_variability() as f64,
            burst_engine_active: area.burst_engine_active(),
            properties: HashMap::new(),
            // IPU/OPU-specific fields (None for genome service - not decoded here)
            cortical_subtype: None,
            encoding_type: None,
            encoding_format: None,
            unit_id: None,
            group_id: None,
            parent_region_id: manager.get_parent_region_id_for_area(&cortical_id_typed),
        })
    }

    /// Helper to get cortical area info
    async fn get_cortical_area_info(&self, cortical_id: &str) -> ServiceResult<CorticalAreaInfo> {
        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;

        let manager = self.connectome.read();

        let area = manager
            .get_cortical_area(&cortical_id_typed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;

        let cortical_idx = manager
            .get_cortical_idx(&cortical_id_typed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })?;

        let neuron_count = manager.get_neuron_count_in_area(&cortical_id_typed);
        let synapse_count = manager.get_synapse_count_in_area(&cortical_id_typed);

        // Get cortical_group from the area (uses cortical_type_new if available)
        let cortical_group = area.get_cortical_group();

        Ok(CorticalAreaInfo {
            cortical_id: area.cortical_id.as_base_64(),
            cortical_id_s: area.cortical_id.to_string(), // Human-readable ASCII string
            cortical_idx,
            name: area.name.clone(),
            dimensions: (
                area.dimensions.width as usize,
                area.dimensions.height as usize,
                area.dimensions.depth as usize,
            ),
            position: area.position.into(),
            area_type: cortical_group
                .clone()
                .unwrap_or_else(|| "CUSTOM".to_string()),
            cortical_group: cortical_group.unwrap_or_else(|| "CUSTOM".to_string()),
            neuron_count,
            synapse_count,
            visible: area.visible(),
            sub_group: area.sub_group(),
            neurons_per_voxel: area.neurons_per_voxel(),
            postsynaptic_current: area.postsynaptic_current() as f64,
            plasticity_constant: area.plasticity_constant() as f64,
            degeneration: area.degeneration() as f64,
            psp_uniform_distribution: area.psp_uniform_distribution() != 0.0,
            firing_threshold_increment: area.firing_threshold_increment() as f64,
            firing_threshold_limit: area.firing_threshold_limit() as f64,
            consecutive_fire_count: area.consecutive_fire_count(),
            snooze_period: area.snooze_period() as u32,
            refractory_period: area.refractory_period() as u32,
            leak_coefficient: area.leak_coefficient() as f64,
            leak_variability: area.leak_variability() as f64,
            burst_engine_active: area.burst_engine_active(),
            properties: HashMap::new(),
            // IPU/OPU-specific fields (None for genome service - not decoded here)
            cortical_subtype: None,
            encoding_type: None,
            encoding_format: None,
            unit_id: None,
            group_id: None,
            parent_region_id: manager.get_parent_region_id_for_area(&cortical_id_typed),
        })
    }
}
