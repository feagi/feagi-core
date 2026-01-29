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
use feagi_evolutionary::{get_default_neural_properties, MemoryAreaProperties};
use feagi_brain_development::models::CorticalAreaExt;
use feagi_brain_development::neuroembryogenesis::Neuroembryogenesis;
use feagi_brain_development::ConnectomeManager;
use feagi_npu_burst_engine::{BurstLoopRunner, ParameterUpdateQueue};
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalSubUnitIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID,
    IOCorticalAreaConfigurationFlag,
};
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, trace, warn};

use crate::genome::{ChangeType, CorticalChangeClassifier};

fn frame_handling_label(frame: FrameChangeHandling) -> &'static str {
    match frame {
        FrameChangeHandling::Absolute => "Absolute",
        FrameChangeHandling::Incremental => "Incremental",
    }
}

fn positioning_label(positioning: PercentageNeuronPositioning) -> &'static str {
    match positioning {
        PercentageNeuronPositioning::Linear => "Linear",
        PercentageNeuronPositioning::Fractional => "Fractional",
    }
}

fn signage_label_from_flag(flag: &IOCorticalAreaConfigurationFlag) -> &'static str {
    match flag {
        IOCorticalAreaConfigurationFlag::SignedPercentage(..)
        | IOCorticalAreaConfigurationFlag::SignedPercentage2D(..)
        | IOCorticalAreaConfigurationFlag::SignedPercentage3D(..)
        | IOCorticalAreaConfigurationFlag::SignedPercentage4D(..) => "Percentage Signed",
        IOCorticalAreaConfigurationFlag::Percentage(..)
        | IOCorticalAreaConfigurationFlag::Percentage2D(..)
        | IOCorticalAreaConfigurationFlag::Percentage3D(..)
        | IOCorticalAreaConfigurationFlag::Percentage4D(..) => "Percentage Unsigned",
        IOCorticalAreaConfigurationFlag::CartesianPlane(..) => "Cartesian Plane",
        IOCorticalAreaConfigurationFlag::Misc(..) => "Misc",
        IOCorticalAreaConfigurationFlag::Boolean => "Boolean",
    }
}

/// Merge default template and memory properties into provided values.
/// Existing values always override defaults.
fn merge_memory_area_properties(
    base: HashMap<String, Value>,
    extra: Option<&HashMap<String, Value>>,
) -> HashMap<String, Value> {
    let mut defaults = get_default_neural_properties();
    let memory_defaults = MemoryAreaProperties::default();
    defaults.entry("cortical_group".to_string()).or_insert(Value::from("MEMORY"));
    defaults
        .entry("is_mem_type".to_string())
        .or_insert(Value::from(true));
    defaults
        .entry("temporal_depth".to_string())
        .or_insert(Value::from(memory_defaults.temporal_depth));
    defaults
        .entry("longterm_mem_threshold".to_string())
        .or_insert(Value::from(memory_defaults.longterm_threshold));
    defaults
        .entry("lifespan_growth_rate".to_string())
        .or_insert(Value::from(memory_defaults.lifespan_growth_rate));
    defaults
        .entry("init_lifespan".to_string())
        .or_insert(Value::from(memory_defaults.init_lifespan));

    defaults.extend(base);
    if let Some(extra_props) = extra {
        defaults.extend(extra_props.clone());
    }
    defaults
}

fn behavior_label_from_flag(flag: &IOCorticalAreaConfigurationFlag) -> &'static str {
    match flag {
        IOCorticalAreaConfigurationFlag::Boolean => "Not Applicable",
        IOCorticalAreaConfigurationFlag::CartesianPlane(frame)
        | IOCorticalAreaConfigurationFlag::Misc(frame)
        | IOCorticalAreaConfigurationFlag::Percentage(frame, _)
        | IOCorticalAreaConfigurationFlag::Percentage2D(frame, _)
        | IOCorticalAreaConfigurationFlag::Percentage3D(frame, _)
        | IOCorticalAreaConfigurationFlag::Percentage4D(frame, _)
        | IOCorticalAreaConfigurationFlag::SignedPercentage(frame, _)
        | IOCorticalAreaConfigurationFlag::SignedPercentage2D(frame, _)
        | IOCorticalAreaConfigurationFlag::SignedPercentage3D(frame, _)
        | IOCorticalAreaConfigurationFlag::SignedPercentage4D(frame, _) => {
            frame_handling_label(*frame)
        }
    }
}

fn coding_type_label_from_flag(flag: &IOCorticalAreaConfigurationFlag) -> &'static str {
    match flag {
        IOCorticalAreaConfigurationFlag::Percentage(_, positioning)
        | IOCorticalAreaConfigurationFlag::Percentage2D(_, positioning)
        | IOCorticalAreaConfigurationFlag::Percentage3D(_, positioning)
        | IOCorticalAreaConfigurationFlag::Percentage4D(_, positioning)
        | IOCorticalAreaConfigurationFlag::SignedPercentage(_, positioning)
        | IOCorticalAreaConfigurationFlag::SignedPercentage2D(_, positioning)
        | IOCorticalAreaConfigurationFlag::SignedPercentage3D(_, positioning)
        | IOCorticalAreaConfigurationFlag::SignedPercentage4D(_, positioning) => {
            positioning_label(*positioning)
        }
        IOCorticalAreaConfigurationFlag::CartesianPlane(..)
        | IOCorticalAreaConfigurationFlag::Misc(..)
        | IOCorticalAreaConfigurationFlag::Boolean => "Not Applicable",
    }
}

fn io_unit_reference_from_cortical_id(cortical_id: &CorticalID) -> Option<[u8; 3]> {
    let bytes = cortical_id.as_bytes();
    if bytes[0] != b'i' && bytes[0] != b'o' {
        return None;
    }
    Some([bytes[1], bytes[2], bytes[3]])
}

fn io_coding_options_for_unit(cortical_id: &CorticalID) -> Option<IOCodingOptions> {
    let unit_ref = io_unit_reference_from_cortical_id(cortical_id)?;
    let is_input = cortical_id.as_bytes()[0] == b'i';

    let (accepted_type, allowed_frames) = if is_input {
        let unit = SensoryCorticalUnit::list_all()
            .iter()
            .find(|u| u.get_cortical_id_unit_reference() == unit_ref)?;
        (
            unit.get_accepted_wrapped_io_data_type(),
            unit.get_allowed_frame_change_handling(),
        )
    } else {
        let unit = MotorCorticalUnit::list_all()
            .iter()
            .find(|u| u.get_cortical_id_unit_reference() == unit_ref)?;
        (
            unit.get_accepted_wrapped_io_data_type(),
            unit.get_allowed_frame_change_handling(),
        )
    };

    let mut signage_options = Vec::new();
    let mut behavior_options = Vec::new();
    let mut coding_type_options = Vec::new();

    let io_flag = match cortical_id.extract_io_data_flag() {
        Ok(flag) => flag,
        Err(err) => {
            warn!(
                target: "feagi-services",
                "[IO-CODING] {} failed to extract io_flag: {} (accepted_type={})",
                cortical_id,
                err,
                accepted_type
            );
            return None;
        }
    };
    signage_options.push(signage_label_from_flag(&io_flag).to_string());

    let supports_frame_handling = !matches!(io_flag, IOCorticalAreaConfigurationFlag::Boolean);
    if supports_frame_handling {
        if let Some(frames) = allowed_frames {
            for frame in frames {
                behavior_options.push(frame_handling_label(*frame).to_string());
            }
        } else {
            behavior_options.push("Absolute".to_string());
            behavior_options.push("Incremental".to_string());
        }
    } else {
        behavior_options.push("Not Applicable".to_string());
    }

    let supports_positioning = matches!(
        io_flag,
        IOCorticalAreaConfigurationFlag::Percentage(..)
            | IOCorticalAreaConfigurationFlag::Percentage2D(..)
            | IOCorticalAreaConfigurationFlag::Percentage3D(..)
            | IOCorticalAreaConfigurationFlag::Percentage4D(..)
            | IOCorticalAreaConfigurationFlag::SignedPercentage(..)
            | IOCorticalAreaConfigurationFlag::SignedPercentage2D(..)
            | IOCorticalAreaConfigurationFlag::SignedPercentage3D(..)
            | IOCorticalAreaConfigurationFlag::SignedPercentage4D(..)
    );
    if supports_positioning {
        coding_type_options.push("Linear".to_string());
        coding_type_options.push("Fractional".to_string());
    } else {
        coding_type_options.push("Not Applicable".to_string());
    }

    if signage_options.is_empty() {
        warn!(
            target: "feagi-services",
            "[IO-CODING] {} empty signage_options (accepted_type={}, io_flag={:?})",
            cortical_id,
            accepted_type,
            io_flag
        );
    }
    Some(IOCodingOptions {
        signage_options,
        behavior_options,
        coding_type_options,
    })
}

/// Default implementation of GenomeService
pub struct GenomeServiceImpl {
    connectome: Arc<RwLock<ConnectomeManager>>,
    parameter_queue: Option<ParameterUpdateQueue>,
    /// Currently loaded genome (source of truth for structural changes)
    /// This is updated when genome is loaded or when cortical areas are modified
    current_genome: Arc<RwLock<Option<feagi_evolutionary::RuntimeGenome>>>,
    /// Counter tracking how many genomes have been loaded (increments on each load)
    genome_load_counter: Arc<RwLock<i32>>,
    /// Timestamp of when the current genome was loaded
    genome_load_timestamp: Arc<RwLock<Option<i64>>>,
    /// Optional burst runner for refreshing cortical_id cache
    burst_runner: Option<Arc<RwLock<BurstLoopRunner>>>,
}

impl GenomeServiceImpl {
    pub fn new(connectome: Arc<RwLock<ConnectomeManager>>) -> Self {
        Self {
            connectome,
            parameter_queue: None,
            current_genome: Arc::new(RwLock::new(None)),
            genome_load_counter: Arc::new(RwLock::new(0)),
            genome_load_timestamp: Arc::new(RwLock::new(None)),
            burst_runner: None,
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
            burst_runner: None,
        }
    }

    /// Set the burst runner for cache refresh
    pub fn set_burst_runner(&mut self, burst_runner: Arc<RwLock<BurstLoopRunner>>) {
        self.burst_runner = Some(burst_runner);
    }

    /// Refresh cortical_id cache in burst runner
    fn refresh_burst_runner_cache(&self) {
        if let Some(ref burst_runner) = self.burst_runner {
            let manager = self.connectome.read();
            let mappings = manager.get_all_cortical_idx_to_id_mappings();
            let chunk_sizes = manager.get_all_visualization_granularities();
            let mapping_count = mappings.len();
            let burst_runner_write = burst_runner.write();
            burst_runner_write.refresh_cortical_id_mappings(mappings);
            burst_runner_write.refresh_visualization_granularities(chunk_sizes);
            info!(target: "feagi-services", "Refreshed burst runner cache with {} cortical areas", mapping_count);
        }
    }

    /// Get a reference to the current genome Arc
    /// This allows other services to share access to the genome for persistence
    pub fn get_current_genome_arc(&self) -> Arc<RwLock<Option<feagi_evolutionary::RuntimeGenome>>> {
        Arc::clone(&self.current_genome)
    }
}

#[async_trait]
impl GenomeService for GenomeServiceImpl {
    async fn load_genome(&self, params: LoadGenomeParams) -> ServiceResult<GenomeInfo> {
        info!(target: "feagi-services", "Loading genome from JSON");

        // Parse genome using feagi-evo (this is CPU-bound, but relatively fast)
        let genome = feagi_evolutionary::load_genome_from_json(&params.json_str)
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
            move || -> Result<feagi_brain_development::neuroembryogenesis::DevelopmentProgress, ServiceError> {
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
                let manager_arc = feagi_brain_development::ConnectomeManager::instance();
                let mut neuro = Neuroembryogenesis::new(manager_arc.clone());
                neuro.develop_from_genome(&genome_clone).map_err(|e| {
                    ServiceError::Backend(format!("Neuroembryogenesis failed: {}", e))
                })?;

                // Ensure core cortical areas exist after neuroembryogenesis
                // (they may have been added during corticogenesis, but we ensure they exist)
                {
                    let mut manager = manager_arc.write();
                    manager.ensure_core_cortical_areas().map_err(|e| {
                        ServiceError::Backend(format!("Failed to ensure core cortical areas: {}", e))
                    })?;
                }

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

        // Refresh burst runner cache after genome load
        self.refresh_burst_runner_cache();

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

        // Debug: Check all property values in RuntimeGenome before saving
        for (cortical_id, area) in &genome.cortical_areas {
            let area_id_str = cortical_id.as_base_64();
            info!(
                target: "feagi-services",
                "[GENOME-SAVE] Area {} has {} properties in RuntimeGenome",
                area_id_str,
                area.properties.len()
            );

            // Log key properties that should be saved
            let key_props = [
                "mp_driven_psp",
                "snooze_length",
                "consecutive_fire_cnt_max",
                "firing_threshold_increment_x",
                "firing_threshold_increment_y",
                "firing_threshold_increment_z",
                "firing_threshold",
                "leak_coefficient",
                "refractory_period",
                "neuron_excitability",
            ];

            for prop_name in &key_props {
                if let Some(prop_value) = area.properties.get(*prop_name) {
                    info!(
                        target: "feagi-services",
                        "[GENOME-SAVE] Area {} property {}={}",
                        area_id_str, prop_name, prop_value
                    );
                }
            }
        }

        // Update metadata if provided
        if let Some(id) = params.genome_id {
            genome.metadata.genome_id = id;
        }
        if let Some(title) = params.genome_title {
            genome.metadata.genome_title = title;
        }

        // Use the full RuntimeGenome saver (produces flat format v3.0)
        let json_str = feagi_evolutionary::save_genome_to_json(&genome)
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
        let genome = feagi_evolutionary::load_genome_from_json(&json_str)
            .map_err(|e| ServiceError::InvalidInput(format!("Failed to parse genome: {}", e)))?;

        // Validate genome structure
        let validation = feagi_evolutionary::validate_genome(&genome);

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
            if matches!(area_type, CorticalAreaType::Memory(_)) {
                let merged = merge_memory_area_properties(
                    area.properties.clone(),
                    param.properties.as_ref(),
                );
                area.properties = merged;
            } else if let Some(properties) = &param.properties {
                area.properties = properties.clone();
            }

            areas_to_add.push(area);
        }

        // Step 2: Add to runtime genome (source of truth)
        {
            let mut genome_lock = self.current_genome.write();
            if let Some(ref mut genome) = *genome_lock {
                for area in &areas_to_add {
                    genome.cortical_areas.insert(area.cortical_id, area.clone());
                    info!(target: "feagi-services", "Added {} to runtime genome", area.cortical_id.as_base_64());
                    if let Some(parent) = area
                        .properties
                        .get("parent_region_id")
                        .and_then(|v| v.as_str())
                    {
                        let region = genome.brain_regions.get_mut(parent).ok_or_else(|| {
                            ServiceError::InvalidInput(format!(
                                "Unknown parent_region_id '{}' for new cortical area {}",
                                parent,
                                area.cortical_id.as_base_64()
                            ))
                        })?;
                        region.cortical_areas.insert(area.cortical_id);
                    }
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

        // Refresh burst runner cache after creating areas
        self.refresh_burst_runner_cache();

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

        // Convert String to CorticalID (supports legacy core aliases)
        let cortical_id_typed = feagi_evolutionary::string_to_cortical_id(cortical_id)
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

        let mut changes = changes;
        let mut effective_cortical_id = cortical_id_typed;
        let mut effective_cortical_id_str = cortical_id.to_string();

        if changes.contains_key("group_id") {
            let new_id = self.apply_unit_index_update(
                &effective_cortical_id,
                &effective_cortical_id_str,
                &changes,
            )?;
            effective_cortical_id = new_id;
            effective_cortical_id_str = new_id.as_base_64();
            self.regenerate_mappings_for_area(&effective_cortical_id)?;
            changes.remove("group_id");
            if changes.is_empty() {
                return self.get_cortical_area_info(&effective_cortical_id_str).await;
            }
        }

        // Classify changes for intelligent routing
        let change_type = CorticalChangeClassifier::classify_changes(&changes);
        CorticalChangeClassifier::log_classification_result(&changes, change_type);

        // Route based on change type
        match change_type {
            ChangeType::Parameter => {
                // Fast path: Direct neuron updates (~2-5ms, NO synapse rebuild)
                self.update_parameters_only(&effective_cortical_id_str, changes)
                    .await
            }
            ChangeType::Metadata => {
                // Fastest path: Metadata updates only (~1ms)
                self.update_metadata_only(&effective_cortical_id_str, changes)
                    .await
            }
            ChangeType::Structural => {
                // Structural path: Requires synapse rebuild (~100-200ms)
                self.update_with_localized_rebuild(&effective_cortical_id_str, changes)
                    .await
            }
            ChangeType::Hybrid => {
                // Hybrid path: Handle each type separately
                let separated = CorticalChangeClassifier::separate_changes_by_type(&changes);

                // Process in order: metadata first, then parameters, then structural
                if let Some(metadata_changes) = separated.get(&ChangeType::Metadata) {
                    if !metadata_changes.is_empty() {
                        self.update_metadata_only(
                            &effective_cortical_id_str,
                            metadata_changes.clone(),
                        )
                            .await?;
                    }
                }

                if let Some(param_changes) = separated.get(&ChangeType::Parameter) {
                    if !param_changes.is_empty() {
                        self.update_parameters_only(
                            &effective_cortical_id_str,
                            param_changes.clone(),
                        )
                            .await?;
                    }
                }

                if let Some(struct_changes) = separated.get(&ChangeType::Structural) {
                    if !struct_changes.is_empty() {
                        self.update_with_localized_rebuild(
                            &effective_cortical_id_str,
                            struct_changes.clone(),
                        )
                            .await?;
                    }
                }

                // Return updated info
                self.get_cortical_area_info(&effective_cortical_id_str).await
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
            // Get base threshold for spatial gradient updates
            let base_threshold = {
                let manager = self.connectome.read();
                manager
                    .get_cortical_area(&cortical_id_typed)
                    .map(|area| area.firing_threshold())
            };

            for (param_name, value) in &changes {
                // Only queue parameters that affect NPU neurons
                let classifier = CorticalChangeClassifier::parameter_changes();
                if classifier.contains(param_name.as_str()) {
                    // Include base threshold for spatial gradient updates
                    let bt = if param_name == "neuron_fire_threshold_increment"
                        || param_name == "firing_threshold_increment"
                    {
                        base_threshold
                    } else {
                        None
                    };

                    queue.push(feagi_npu_burst_engine::ParameterUpdate {
                        cortical_idx,
                        cortical_id: cortical_id.to_string(),
                        parameter_name: param_name.clone(),
                        value: value.clone(),
                        dimensions: None, // Not needed anymore - neurons have stored positions
                        neurons_per_voxel: None,
                        base_threshold: bt,
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

        // Persist parameter-only updates into the live ConnectomeManager so API reads (and BV UI)
        // reflect the same values that are applied to the NPU.
        //
        // IMPORTANT: The parameter queue updates runtime neuron state; ConnectomeManager is the
        // source-of-truth for cortical-area *reported* properties.
        {
            let mut manager = self.connectome.write();
            if let Some(area) = manager.get_cortical_area_mut(&cortical_id_typed) {
                for (key, value) in &changes {
                    match key.as_str() {
                        // Thresholds
                        "firing_threshold" | "neuron_fire_threshold" => {
                            if let Some(v) = value.as_f64() {
                                area.properties
                                    .insert("firing_threshold".to_string(), serde_json::json!(v));
                            }
                        }
                        "firing_threshold_limit" | "neuron_firing_threshold_limit" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "firing_threshold_limit".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        // Spatial gradient increments
                        "firing_threshold_increment_x" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "firing_threshold_increment_x".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "firing_threshold_increment_y" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "firing_threshold_increment_y".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "firing_threshold_increment_z" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "firing_threshold_increment_z".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "firing_threshold_increment" | "neuron_fire_threshold_increment" => {
                            if let Some(arr) = value.as_array() {
                                if arr.len() == 3 {
                                    if let (Some(x), Some(y), Some(z)) =
                                        (arr[0].as_f64(), arr[1].as_f64(), arr[2].as_f64())
                                    {
                                        area.properties.insert(
                                            "firing_threshold_increment_x".to_string(),
                                            serde_json::json!(x),
                                        );
                                        area.properties.insert(
                                            "firing_threshold_increment_y".to_string(),
                                            serde_json::json!(y),
                                        );
                                        area.properties.insert(
                                            "firing_threshold_increment_z".to_string(),
                                            serde_json::json!(z),
                                        );
                                    }
                                }
                            } else if let Some(obj) = value.as_object() {
                                if let (Some(x), Some(y), Some(z)) = (
                                    obj.get("x").and_then(|v| v.as_f64()),
                                    obj.get("y").and_then(|v| v.as_f64()),
                                    obj.get("z").and_then(|v| v.as_f64()),
                                ) {
                                    area.properties.insert(
                                        "firing_threshold_increment_x".to_string(),
                                        serde_json::json!(x),
                                    );
                                    area.properties.insert(
                                        "firing_threshold_increment_y".to_string(),
                                        serde_json::json!(y),
                                    );
                                    area.properties.insert(
                                        "firing_threshold_increment_z".to_string(),
                                        serde_json::json!(z),
                                    );
                                }
                            }
                        }

                        // Timing/decay
                        "refractory_period" | "neuron_refractory_period" | "refrac" => {
                            if let Some(v) = value.as_u64() {
                                area.properties.insert(
                                    "refractory_period".to_string(),
                                    serde_json::json!(v as u32),
                                );
                            }
                        }
                        "leak_coefficient" | "neuron_leak_coefficient" | "leak" => {
                            if let Some(v) = value.as_f64() {
                                area.properties
                                    .insert("leak_coefficient".to_string(), serde_json::json!(v));
                            }
                        }

                        // Burst gating
                        "consecutive_fire_cnt_max"
                        | "neuron_consecutive_fire_count"
                        | "consecutive_fire_count" => {
                            if let Some(v) = value.as_u64() {
                                // ConnectomeManager getters expect `consecutive_fire_limit`.
                                area.properties.insert(
                                    "consecutive_fire_limit".to_string(),
                                    serde_json::json!(v as u32),
                                );
                            }
                        }
                        "snooze_length" | "neuron_snooze_period" | "snooze_period" => {
                            if let Some(v) = value.as_u64() {
                                // ConnectomeManager getters expect `snooze_period`.
                                area.properties.insert(
                                    "snooze_period".to_string(),
                                    serde_json::json!(v as u32),
                                );
                            }
                        }

                        // Excitability (BV uses percent UI but sends 0..=1 to the API)
                        "neuron_excitability" | "excitability" => {
                            if let Some(v) = value.as_f64() {
                                if (0.0..=1.0).contains(&v) {
                                    area.properties.insert(
                                        "neuron_excitability".to_string(),
                                        serde_json::json!(v),
                                    );
                                } else {
                                    warn!(
                                        target: "feagi-services",
                                        "[FAST-UPDATE] Ignoring neuron_excitability={} for area {} (expected 0..=1)",
                                        v,
                                        cortical_id
                                    );
                                }
                            }
                        }

                        // PSP + degeneration + plasticity
                        "postsynaptic_current" | "neuron_post_synaptic_potential" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "postsynaptic_current".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "postsynaptic_current_max" | "neuron_post_synaptic_potential_max" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "postsynaptic_current_max".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "degeneration" | "neuron_degeneracy_coefficient" => {
                            if let Some(v) = value.as_f64() {
                                area.properties
                                    .insert("degeneration".to_string(), serde_json::json!(v));
                            }
                        }
                        "plasticity_constant" | "neuron_plasticity_constant" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "plasticity_constant".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "psp_uniform_distribution" | "neuron_psp_uniform_distribution" => {
                            if let Some(v) = value.as_bool() {
                                area.properties.insert(
                                    "psp_uniform_distribution".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }

                        // Memory parameters (used by plasticity registration + API display)
                        "init_lifespan" | "neuron_init_lifespan" => {
                            if let Some(v) = value.as_u64() {
                                area.properties.insert(
                                    "init_lifespan".to_string(),
                                    serde_json::json!(v as u32),
                                );
                            }
                        }
                        "lifespan_growth_rate" | "neuron_lifespan_growth_rate" => {
                            // Accept integer and float representations.
                            if let Some(v) =
                                value.as_f64().or_else(|| value.as_u64().map(|u| u as f64))
                            {
                                area.properties.insert(
                                    "lifespan_growth_rate".to_string(),
                                    serde_json::json!(v as f32),
                                );
                            }
                        }
                        "longterm_mem_threshold" | "neuron_longterm_mem_threshold" => {
                            if let Some(v) = value.as_u64() {
                                area.properties.insert(
                                    "longterm_mem_threshold".to_string(),
                                    serde_json::json!(v as u32),
                                );
                            }
                        }
                        "temporal_depth" => {
                            if let Some(v) = value.as_u64() {
                                if v == 0 {
                                    warn!(
                                        target: "feagi-services",
                                        "[FAST-UPDATE] Ignoring temporal_depth=0 for area {} (temporal_depth must be >= 1)",
                                        cortical_id
                                    );
                                } else {
                                    area.properties.insert(
                                        "temporal_depth".to_string(),
                                        serde_json::json!(v as u32),
                                    );
                                }
                            }
                        }

                        // Membrane potential / runtime flags
                        "mp_charge_accumulation" | "neuron_mp_charge_accumulation" => {
                            if let Some(v) = value.as_bool() {
                                area.properties.insert(
                                    "mp_charge_accumulation".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "mp_driven_psp" | "neuron_mp_driven_psp" => {
                            if let Some(v) = value.as_bool() {
                                area.properties
                                    .insert("mp_driven_psp".to_string(), serde_json::json!(v));
                            }
                        }

                        // Burst engine
                        "burst_engine_active" => {
                            if let Some(v) = value.as_bool() {
                                area.properties.insert(
                                    "burst_engine_active".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "visualization_voxel_granularity" => {
                            // Only store if != 1x1x1 (default), delete if set to 1x1x1
                            // Handle both integer and float JSON values
                            if let Some(arr) = value.as_array() {
                                if arr.len() == 3 {
                                    // Try to parse as integers (u64) or floats (f64), then convert to u32
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
                                        let x_u32 = x as u32;
                                        let y_u32 = y as u32;
                                        let z_u32 = z as u32;

                                        // Default is 1x1x1 - only store if different
                                        if x_u32 == 1 && y_u32 == 1 && z_u32 == 1 {
                                            // Remove override (return to default)
                                            area.properties
                                                .remove("visualization_voxel_granularity");
                                        } else {
                                            // Store override (non-default value) as integer array
                                            area.properties.insert(
                                                "visualization_voxel_granularity".to_string(),
                                                serde_json::json!([x_u32, y_u32, z_u32]),
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        _ => {}
                    }
                }
            }
            manager.refresh_cortical_area_hashes(true, false);
        }

        // Update RuntimeGenome if available (CRITICAL for save/load persistence!)
        if let Some(genome) = self.current_genome.write().as_mut() {
            if let Some(area) = genome.cortical_areas.get_mut(&cortical_id_typed) {
                trace!(
                    target: "feagi-services",
                    "[GENOME-UPDATE] Updating RuntimeGenome for area {}",
                    cortical_id
                );
                for (key, value) in &changes {
                    match key.as_str() {
                        "neuron_fire_threshold" | "firing_threshold" => {
                            if let Some(v) = value.as_f64() {
                                area.properties
                                    .insert("firing_threshold".to_string(), serde_json::json!(v));
                            }
                        }
                        "firing_threshold_limit" | "neuron_firing_threshold_limit" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "firing_threshold_limit".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "firing_threshold_increment_x" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "firing_threshold_increment_x".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "firing_threshold_increment_y" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "firing_threshold_increment_y".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "firing_threshold_increment_z" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "firing_threshold_increment_z".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "leak_coefficient" | "neuron_leak_coefficient" => {
                            if let Some(v) = value.as_f64() {
                                area.properties
                                    .insert("leak_coefficient".to_string(), serde_json::json!(v));
                            }
                        }
                        "leak_variability" | "neuron_leak_variability" => {
                            if let Some(v) = value.as_f64() {
                                area.properties
                                    .insert("leak_variability".to_string(), serde_json::json!(v));
                            }
                        }
                        "refractory_period" | "neuron_refractory_period" => {
                            if let Some(v) = value.as_u64() {
                                area.properties.insert(
                                    "refractory_period".to_string(),
                                    serde_json::json!(v as u32),
                                );
                            }
                        }
                        "snooze_period" | "neuron_snooze_period" => {
                            if let Some(v) = value.as_u64() {
                                // Converter expects "snooze_length" not "snooze_period"
                                area.properties.insert(
                                    "snooze_length".to_string(),
                                    serde_json::json!(v as u32),
                                );
                            }
                        }
                        "consecutive_fire_count" | "neuron_consecutive_fire_count" => {
                            if let Some(v) = value.as_u64() {
                                // Converter expects "consecutive_fire_cnt_max" not "consecutive_fire_count"
                                area.properties.insert(
                                    "consecutive_fire_cnt_max".to_string(),
                                    serde_json::json!(v as u32),
                                );
                            }
                        }
                        "postsynaptic_current" | "neuron_post_synaptic_potential" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "postsynaptic_current".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "postsynaptic_current_max" | "neuron_post_synaptic_potential_max" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "postsynaptic_current_max".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "plasticity_constant" | "neuron_plasticity_constant" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "plasticity_constant".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "degeneration" | "neuron_degeneracy_coefficient" => {
                            if let Some(v) = value.as_f64() {
                                area.properties
                                    .insert("degeneration".to_string(), serde_json::json!(v));
                            }
                        }
                        "psp_uniform_distribution" | "neuron_psp_uniform_distribution" => {
                            if let Some(v) = value.as_bool() {
                                area.properties.insert(
                                    "psp_uniform_distribution".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "mp_driven_psp" | "neuron_mp_driven_psp" => {
                            if let Some(v) = value.as_bool() {
                                area.properties
                                    .insert("mp_driven_psp".to_string(), serde_json::json!(v));
                                info!(
                                    target: "feagi-services",
                                    "[GENOME-UPDATE] Updated mp_driven_psp={} in RuntimeGenome for area {}",
                                    v, cortical_id
                                );
                            } else {
                                warn!(
                                    target: "feagi-services",
                                    "[GENOME-UPDATE] Failed to update mp_driven_psp: value is not a bool (got {:?})",
                                    value
                                );
                            }
                        }
                        "mp_charge_accumulation" | "neuron_mp_charge_accumulation" => {
                            if let Some(v) = value.as_bool() {
                                area.properties.insert(
                                    "mp_charge_accumulation".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "neuron_excitability" => {
                            if let Some(v) = value.as_f64() {
                                area.properties.insert(
                                    "neuron_excitability".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "init_lifespan" | "neuron_init_lifespan" => {
                            if let Some(v) = value.as_u64() {
                                area.properties.insert(
                                    "init_lifespan".to_string(),
                                    serde_json::json!(v as u32),
                                );
                            }
                        }
                        "lifespan_growth_rate" | "neuron_lifespan_growth_rate" => {
                            if let Some(v) = value.as_u64() {
                                area.properties.insert(
                                    "lifespan_growth_rate".to_string(),
                                    serde_json::json!(v as u32),
                                );
                            }
                        }
                        "longterm_mem_threshold" | "neuron_longterm_mem_threshold" => {
                            if let Some(v) = value.as_u64() {
                                area.properties.insert(
                                    "longterm_mem_threshold".to_string(),
                                    serde_json::json!(v as u32),
                                );
                            }
                        }
                        "temporal_depth" => {
                            if let Some(v) = value.as_u64() {
                                if v == 0 {
                                    warn!(
                                        target: "feagi-services",
                                        "[GENOME-UPDATE] Ignoring temporal_depth=0 for area {} (temporal_depth must be >= 1)",
                                        cortical_id
                                    );
                                } else {
                                    area.properties.insert(
                                        "temporal_depth".to_string(),
                                        serde_json::json!(v as u32),
                                    );
                                }
                            }
                        }
                        "firing_threshold_increment" | "neuron_fire_threshold_increment" => {
                            // Converter expects separate x, y, z properties, not an array
                            if let Some(arr) = value.as_array() {
                                if arr.len() == 3 {
                                    if let (Some(x), Some(y), Some(z)) =
                                        (arr[0].as_f64(), arr[1].as_f64(), arr[2].as_f64())
                                    {
                                        area.properties.insert(
                                            "firing_threshold_increment_x".to_string(),
                                            serde_json::json!(x),
                                        );
                                        area.properties.insert(
                                            "firing_threshold_increment_y".to_string(),
                                            serde_json::json!(y),
                                        );
                                        area.properties.insert(
                                            "firing_threshold_increment_z".to_string(),
                                            serde_json::json!(z),
                                        );
                                    }
                                }
                            } else if let Some(obj) = value.as_object() {
                                // Convert {x, y, z} to separate properties
                                if let (Some(x), Some(y), Some(z)) = (
                                    obj.get("x").and_then(|v| v.as_f64()),
                                    obj.get("y").and_then(|v| v.as_f64()),
                                    obj.get("z").and_then(|v| v.as_f64()),
                                ) {
                                    area.properties.insert(
                                        "firing_threshold_increment_x".to_string(),
                                        serde_json::json!(x),
                                    );
                                    area.properties.insert(
                                        "firing_threshold_increment_y".to_string(),
                                        serde_json::json!(y),
                                    );
                                    area.properties.insert(
                                        "firing_threshold_increment_z".to_string(),
                                        serde_json::json!(z),
                                    );
                                }
                            }
                        }
                        "burst_engine_active" => {
                            if let Some(v) = value.as_bool() {
                                area.properties.insert(
                                    "burst_engine_active".to_string(),
                                    serde_json::json!(v),
                                );
                            }
                        }
                        "visualization_voxel_granularity" => {
                            // Only store if != 1x1x1 (default), delete if set to 1x1x1
                            // Handle both integer and float JSON values
                            if let Some(arr) = value.as_array() {
                                if arr.len() == 3 {
                                    // Try to parse as integers (u64) or floats (f64), then convert to u32
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
                                        let x_u32 = x as u32;
                                        let y_u32 = y as u32;
                                        let z_u32 = z as u32;

                                        // Default is 1x1x1 - only store if different
                                        if x_u32 == 1 && y_u32 == 1 && z_u32 == 1 {
                                            // Remove override (return to default)
                                            area.properties
                                                .remove("visualization_voxel_granularity");
                                        } else {
                                            // Store override (non-default value) as integer array
                                            area.properties.insert(
                                                "visualization_voxel_granularity".to_string(),
                                                serde_json::json!([x_u32, y_u32, z_u32]),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                warn!(
                    target: "feagi-services",
                    "[GENOME-UPDATE] WARNING: Cortical area {} not found in RuntimeGenome - property updates will not persist to saved genome!",
                    cortical_id
                );
            }
        } else {
            warn!(
                target: "feagi-services",
                "[GENOME-UPDATE] WARNING: No RuntimeGenome loaded - property updates will not persist to saved genome!"
            );
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
                    "neuron_fire_threshold" | "firing_threshold" => {
                        if let Some(v) = value.as_f64() {
                            area.add_property_mut(
                                "firing_threshold".to_string(),
                                serde_json::json!(v),
                            );
                        }
                    }
                    "firing_threshold_limit" | "neuron_firing_threshold_limit" => {
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
                    "leak_variability" | "neuron_leak_variability" => {
                        if let Some(v) = value.as_f64() {
                            area.add_property_mut(
                                "leak_variability".to_string(),
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
                    "plasticity_constant" | "neuron_plasticity_constant" => {
                        if let Some(v) = value.as_f64() {
                            area.add_property_mut(
                                "plasticity_constant".to_string(),
                                serde_json::json!(v),
                            );
                        }
                    }
                    "degeneration" | "neuron_degeneracy_coefficient" => {
                        if let Some(v) = value.as_f64() {
                            area.add_property_mut("degeneration".to_string(), serde_json::json!(v));
                        }
                    }
                    "postsynaptic_current" | "neuron_post_synaptic_potential" => {
                        if let Some(v) = value.as_f64() {
                            area.add_property_mut(
                                "postsynaptic_current".to_string(),
                                serde_json::json!(v),
                            );
                        }
                    }
                    "postsynaptic_current_max" | "neuron_post_synaptic_potential_max" => {
                        if let Some(v) = value.as_f64() {
                            area.add_property_mut(
                                "postsynaptic_current_max".to_string(),
                                serde_json::json!(v),
                            );
                        }
                    }
                    "psp_uniform_distribution" | "neuron_psp_uniform_distribution" => {
                        if let Some(v) = value.as_bool() {
                            area.add_property_mut(
                                "psp_uniform_distribution".to_string(),
                                serde_json::json!(v),
                            );
                        }
                    }
                    "mp_driven_psp" | "neuron_mp_driven_psp" => {
                        if let Some(v) = value.as_bool() {
                            area.add_property_mut(
                                "mp_driven_psp".to_string(),
                                serde_json::json!(v),
                            );
                            info!(
                                target: "feagi-services",
                                "[CONNECTOME-UPDATE] Updated mp_driven_psp={} in ConnectomeManager for area {}",
                                v, cortical_id
                            );
                        }
                    }
                    "mp_charge_accumulation" | "neuron_mp_charge_accumulation" => {
                        if let Some(v) = value.as_bool() {
                            area.add_property_mut(
                                "mp_charge_accumulation".to_string(),
                                serde_json::json!(v),
                            );
                        }
                    }
                    "excitability" | "neuron_excitability" => {
                        if let Some(v) = value.as_f64() {
                            area.add_property_mut("excitability".to_string(), serde_json::json!(v));
                        }
                    }
                    "init_lifespan" | "neuron_init_lifespan" => {
                        if let Some(v) = value.as_u64() {
                            area.add_property_mut(
                                "init_lifespan".to_string(),
                                serde_json::json!(v as u32),
                            );
                        }
                    }
                    "lifespan_growth_rate" | "neuron_lifespan_growth_rate" => {
                        if let Some(v) = value.as_u64() {
                            area.add_property_mut(
                                "lifespan_growth_rate".to_string(),
                                serde_json::json!(v as u32),
                            );
                        }
                    }
                    "longterm_mem_threshold" | "neuron_longterm_mem_threshold" => {
                        if let Some(v) = value.as_u64() {
                            area.add_property_mut(
                                "longterm_mem_threshold".to_string(),
                                serde_json::json!(v as u32),
                            );
                        }
                    }
                    "firing_threshold_increment" | "neuron_fire_threshold_increment" => {
                        // Expect either array [x, y, z] or dict {x, y, z}
                        if let Some(arr) = value.as_array() {
                            if arr.len() == 3 {
                                let x = arr[0].as_f64().unwrap_or(0.0);
                                let y = arr[1].as_f64().unwrap_or(0.0);
                                let z = arr[2].as_f64().unwrap_or(0.0);

                                // Store both array format and individual x/y/z properties
                                area.add_property_mut(
                                    "firing_threshold_increment".to_string(),
                                    serde_json::json!(arr),
                                );
                                area.add_property_mut(
                                    "firing_threshold_increment_x".to_string(),
                                    serde_json::json!(x),
                                );
                                area.add_property_mut(
                                    "firing_threshold_increment_y".to_string(),
                                    serde_json::json!(y),
                                );
                                area.add_property_mut(
                                    "firing_threshold_increment_z".to_string(),
                                    serde_json::json!(z),
                                );
                            }
                        } else if let Some(obj) = value.as_object() {
                            // Convert {x, y, z} to individual properties
                            let x = obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let y = obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let z = obj.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0);

                            area.add_property_mut(
                                "firing_threshold_increment".to_string(),
                                serde_json::json!([x, y, z]),
                            );
                            area.add_property_mut(
                                "firing_threshold_increment_x".to_string(),
                                serde_json::json!(x),
                            );
                            area.add_property_mut(
                                "firing_threshold_increment_y".to_string(),
                                serde_json::json!(y),
                            );
                            area.add_property_mut(
                                "firing_threshold_increment_z".to_string(),
                                serde_json::json!(z),
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
            manager.refresh_cortical_area_hashes(true, false);
        }

        // If memory-related parameters were updated, immediately apply them to the runtime
        // plasticity subsystem (and FireLedger tracking), otherwise changes only take effect after save+reload.
        //
        // BV behavior observed:
        // - Update via API updates RuntimeGenome + ConnectomeManager
        // - PlasticityService keeps running with the old temporal_depth/lifecycle config until genome reload
        //
        // This block re-registers the memory area configuration in the live executor.
        #[cfg(feature = "plasticity")]
        {
            let memory_param_changed = changes.keys().any(|k| {
                matches!(
                    k.as_str(),
                    "init_lifespan"
                        | "neuron_init_lifespan"
                        | "lifespan_growth_rate"
                        | "neuron_lifespan_growth_rate"
                        | "longterm_mem_threshold"
                        | "neuron_longterm_mem_threshold"
                        | "temporal_depth"
                )
            });

            if memory_param_changed {
                use feagi_evolutionary::extract_memory_properties;
                use feagi_npu_plasticity::{MemoryNeuronLifecycleConfig, PlasticityExecutor};

                let manager = self.connectome.read();
                if let Some(area) = manager.get_cortical_area(&cortical_id_typed) {
                    if let Some(mem_props) = extract_memory_properties(&area.properties) {
                        // Update FireLedger upstream tracking for this memory area (monotonic-increase).
                        // Note: FireLedger track_area is an *exact* setting; this uses max(existing, desired)
                        // to avoid shrinking windows that may be required elsewhere (e.g., other memory areas).
                        if let Some(npu_arc) = manager.get_npu().cloned() {
                            if let Ok(mut npu) = npu_arc.lock() {
                                let upstream_areas =
                                    manager.get_upstream_cortical_areas(&cortical_id_typed);
                                let existing_configs = npu.get_all_fire_ledger_configs();
                                let desired = mem_props.temporal_depth as usize;

                                for upstream_idx in upstream_areas.iter().copied() {
                                    let existing = existing_configs
                                        .iter()
                                        .find(|(idx, _)| *idx == upstream_idx)
                                        .map(|(_, w)| *w)
                                        .unwrap_or(0);
                                    let resolved = existing.max(desired);
                                    if resolved != existing {
                                        if let Err(e) =
                                            npu.configure_fire_ledger_window(upstream_idx, resolved)
                                        {
                                            warn!(
                                                target: "feagi-services",
                                                "[GENOME-UPDATE] Failed to configure FireLedger window for upstream idx={} (requested={}): {}",
                                                upstream_idx,
                                                resolved,
                                                e
                                            );
                                        }
                                    }
                                }
                            } else {
                                warn!(target: "feagi-services", "[GENOME-UPDATE] Failed to lock NPU for FireLedger update");
                            }
                        }

                        // Re-register memory area config in PlasticityExecutor so temporal_depth/lifecycle changes apply immediately.
                        if let Some(executor) = manager.get_plasticity_executor() {
                            if let Ok(exec) = executor.lock() {
                                let upstream_areas =
                                    manager.get_upstream_cortical_areas(&cortical_id_typed);
                                let lifecycle_config = MemoryNeuronLifecycleConfig {
                                    initial_lifespan: mem_props.init_lifespan,
                                    lifespan_growth_rate: mem_props.lifespan_growth_rate,
                                    longterm_threshold: mem_props.longterm_threshold,
                                    max_reactivations: 1000,
                                };

                                exec.register_memory_area(
                                    cortical_idx,
                                    cortical_id.to_string(),
                                    mem_props.temporal_depth,
                                    upstream_areas,
                                    Some(lifecycle_config),
                                );
                            } else {
                                warn!(target: "feagi-services", "[GENOME-UPDATE] Failed to lock PlasticityExecutor for memory-area update");
                            }
                        }
                    }
                }
            }
        }

        info!(target: "feagi-services", "[FAST-UPDATE] Parameter update complete");

        // Return updated info
        self.get_cortical_area_info(cortical_id).await
    }

    /// Update IO coding parameters and remap the cortical ID.
    ///
    /// This updates both RuntimeGenome and ConnectomeManager, preserving cortical_idx.
    fn apply_io_coding_update(
        &self,
        cortical_id: &CorticalID,
        cortical_id_str: &str,
        changes: &HashMap<String, Value>,
    ) -> ServiceResult<CorticalID> {
        let new_cortical_id_str = changes
            .get("new_cortical_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ServiceError::InvalidInput(
                    "new_cortical_id is required when updating IO coding parameters".to_string(),
                )
            })?;
        let _new_cortical_id = CorticalID::try_from_base_64(new_cortical_id_str).map_err(|e| {
            ServiceError::InvalidInput(format!(
                "Invalid new_cortical_id '{}': {}",
                new_cortical_id_str, e
            ))
        })?;

        let bytes = cortical_id.as_bytes();
        let is_input = bytes[0] == b'i';
        let is_output = bytes[0] == b'o';
        if !is_input && !is_output {
            return Err(ServiceError::InvalidInput(
                "IO coding updates only apply to IPU/OPU cortical areas".to_string(),
            ));
        }

        let current_flag = cortical_id.extract_io_data_flag().map_err(|e| {
            ServiceError::InvalidInput(format!(
                "Unable to decode IO configuration from cortical ID '{}': {}",
                cortical_id_str, e
            ))
        })?;

        let parse_frame = |raw: &str| -> Option<FrameChangeHandling> {
            let lower = raw.trim().to_ascii_lowercase();
            match lower.as_str() {
                "absolute" => Some(FrameChangeHandling::Absolute),
                "incremental" => Some(FrameChangeHandling::Incremental),
                _ => None,
            }
        };
        let parse_positioning = |raw: &str| -> Option<PercentageNeuronPositioning> {
            let lower = raw.trim().to_ascii_lowercase();
            match lower.as_str() {
                "linear" => Some(PercentageNeuronPositioning::Linear),
                "fractional" => Some(PercentageNeuronPositioning::Fractional),
                _ => None,
            }
        };
        let parse_signage = |raw: &str| -> Option<bool> {
            let lower = raw.trim().to_ascii_lowercase();
            if lower.contains("unsigned") {
                Some(false)
            } else if lower.contains("signed") {
                Some(true)
            } else {
                None
            }
        };

        let requested_signage = changes.get("coding_signage").and_then(|v| v.as_str());
        let requested_behavior = changes.get("coding_behavior").and_then(|v| v.as_str());
        let requested_type = changes.get("coding_type").and_then(|v| v.as_str());

        let new_flag = match current_flag {
            IOCorticalAreaConfigurationFlag::Percentage(frame, positioning) => {
                let signed = requested_signage.and_then(parse_signage).unwrap_or(false);
                let new_frame = requested_behavior.and_then(parse_frame).unwrap_or(frame);
                let new_positioning = requested_type
                    .and_then(parse_positioning)
                    .unwrap_or(positioning);
                if signed {
                    IOCorticalAreaConfigurationFlag::SignedPercentage(new_frame, new_positioning)
                } else {
                    IOCorticalAreaConfigurationFlag::Percentage(new_frame, new_positioning)
                }
            }
            IOCorticalAreaConfigurationFlag::Percentage2D(frame, positioning) => {
                let signed = requested_signage.and_then(parse_signage).unwrap_or(false);
                let new_frame = requested_behavior.and_then(parse_frame).unwrap_or(frame);
                let new_positioning = requested_type
                    .and_then(parse_positioning)
                    .unwrap_or(positioning);
                if signed {
                    IOCorticalAreaConfigurationFlag::SignedPercentage2D(new_frame, new_positioning)
                } else {
                    IOCorticalAreaConfigurationFlag::Percentage2D(new_frame, new_positioning)
                }
            }
            IOCorticalAreaConfigurationFlag::Percentage3D(frame, positioning) => {
                let signed = requested_signage.and_then(parse_signage).unwrap_or(false);
                let new_frame = requested_behavior.and_then(parse_frame).unwrap_or(frame);
                let new_positioning = requested_type
                    .and_then(parse_positioning)
                    .unwrap_or(positioning);
                if signed {
                    IOCorticalAreaConfigurationFlag::SignedPercentage3D(new_frame, new_positioning)
                } else {
                    IOCorticalAreaConfigurationFlag::Percentage3D(new_frame, new_positioning)
                }
            }
            IOCorticalAreaConfigurationFlag::Percentage4D(frame, positioning) => {
                let signed = requested_signage.and_then(parse_signage).unwrap_or(false);
                let new_frame = requested_behavior.and_then(parse_frame).unwrap_or(frame);
                let new_positioning = requested_type
                    .and_then(parse_positioning)
                    .unwrap_or(positioning);
                if signed {
                    IOCorticalAreaConfigurationFlag::SignedPercentage4D(new_frame, new_positioning)
                } else {
                    IOCorticalAreaConfigurationFlag::Percentage4D(new_frame, new_positioning)
                }
            }
            IOCorticalAreaConfigurationFlag::SignedPercentage(frame, positioning) => {
                let signed = requested_signage.and_then(parse_signage).unwrap_or(true);
                let new_frame = requested_behavior.and_then(parse_frame).unwrap_or(frame);
                let new_positioning = requested_type
                    .and_then(parse_positioning)
                    .unwrap_or(positioning);
                if signed {
                    IOCorticalAreaConfigurationFlag::SignedPercentage(new_frame, new_positioning)
                } else {
                    IOCorticalAreaConfigurationFlag::Percentage(new_frame, new_positioning)
                }
            }
            IOCorticalAreaConfigurationFlag::SignedPercentage2D(frame, positioning) => {
                let signed = requested_signage.and_then(parse_signage).unwrap_or(true);
                let new_frame = requested_behavior.and_then(parse_frame).unwrap_or(frame);
                let new_positioning = requested_type
                    .and_then(parse_positioning)
                    .unwrap_or(positioning);
                if signed {
                    IOCorticalAreaConfigurationFlag::SignedPercentage2D(new_frame, new_positioning)
                } else {
                    IOCorticalAreaConfigurationFlag::Percentage2D(new_frame, new_positioning)
                }
            }
            IOCorticalAreaConfigurationFlag::SignedPercentage3D(frame, positioning) => {
                let signed = requested_signage.and_then(parse_signage).unwrap_or(true);
                let new_frame = requested_behavior.and_then(parse_frame).unwrap_or(frame);
                let new_positioning = requested_type
                    .and_then(parse_positioning)
                    .unwrap_or(positioning);
                if signed {
                    IOCorticalAreaConfigurationFlag::SignedPercentage3D(new_frame, new_positioning)
                } else {
                    IOCorticalAreaConfigurationFlag::Percentage3D(new_frame, new_positioning)
                }
            }
            IOCorticalAreaConfigurationFlag::SignedPercentage4D(frame, positioning) => {
                let signed = requested_signage.and_then(parse_signage).unwrap_or(true);
                let new_frame = requested_behavior.and_then(parse_frame).unwrap_or(frame);
                let new_positioning = requested_type
                    .and_then(parse_positioning)
                    .unwrap_or(positioning);
                if signed {
                    IOCorticalAreaConfigurationFlag::SignedPercentage4D(new_frame, new_positioning)
                } else {
                    IOCorticalAreaConfigurationFlag::Percentage4D(new_frame, new_positioning)
                }
            }
            IOCorticalAreaConfigurationFlag::CartesianPlane(frame) => {
                if let Some(signage) = requested_signage {
                    if !signage.trim().eq_ignore_ascii_case("not applicable") {
                        return Err(ServiceError::InvalidInput(
                            "coding_signage not supported for CartesianPlane".to_string(),
                        ));
                    }
                }
                if let Some(coding_type) = requested_type {
                    if !coding_type.trim().eq_ignore_ascii_case("not applicable") {
                        return Err(ServiceError::InvalidInput(
                            "coding_type not supported for CartesianPlane".to_string(),
                        ));
                    }
                }
                let new_frame = requested_behavior.and_then(parse_frame).unwrap_or(frame);
                IOCorticalAreaConfigurationFlag::CartesianPlane(new_frame)
            }
            IOCorticalAreaConfigurationFlag::Misc(frame) => {
                if let Some(signage) = requested_signage {
                    if !signage.trim().eq_ignore_ascii_case("not applicable") {
                        return Err(ServiceError::InvalidInput(
                            "coding_signage not supported for Misc".to_string(),
                        ));
                    }
                }
                if let Some(coding_type) = requested_type {
                    if !coding_type.trim().eq_ignore_ascii_case("not applicable") {
                        return Err(ServiceError::InvalidInput(
                            "coding_type not supported for Misc".to_string(),
                        ));
                    }
                }
                let new_frame = requested_behavior.and_then(parse_frame).unwrap_or(frame);
                IOCorticalAreaConfigurationFlag::Misc(new_frame)
            }
            IOCorticalAreaConfigurationFlag::Boolean => {
                if let Some(signage) = requested_signage {
                    if !signage.trim().eq_ignore_ascii_case("boolean")
                        && !signage.trim().eq_ignore_ascii_case("not applicable")
                    {
                        return Err(ServiceError::InvalidInput(
                            "coding_signage not supported for Boolean".to_string(),
                        ));
                    }
                }
                if let Some(coding_type) = requested_type {
                    if !coding_type.trim().eq_ignore_ascii_case("not applicable") {
                        return Err(ServiceError::InvalidInput(
                            "coding_type not supported for Boolean".to_string(),
                        ));
                    }
                }
                if let Some(behavior) = requested_behavior {
                    if !behavior.trim().eq_ignore_ascii_case("not applicable") {
                        return Err(ServiceError::InvalidInput(
                            "coding_behavior not supported for Boolean".to_string(),
                        ));
                    }
                }
                IOCorticalAreaConfigurationFlag::Boolean
            }
        };

        let unit_identifier = [bytes[1], bytes[2], bytes[3]];
        let cortical_subunit_index = CorticalSubUnitIndex::from(bytes[6]);
        let cortical_unit_index = CorticalUnitIndex::from(bytes[7]);
        let computed_id = new_flag.as_io_cortical_id(
            is_input,
            unit_identifier,
            cortical_unit_index,
            cortical_subunit_index,
        );

        if computed_id.as_base_64() != new_cortical_id_str {
            return Err(ServiceError::InvalidInput(format!(
                "new_cortical_id '{}' does not match computed ID '{}'",
                new_cortical_id_str,
                computed_id.as_base_64()
            )));
        }
        info!(
            target: "feagi-services",
            "[IO-CODING] Remapping cortical ID {} -> {}",
            cortical_id.as_base_64(),
            computed_id.as_base_64()
        );

        let new_cortical_type = if is_input {
            CorticalAreaType::BrainInput(new_flag)
        } else {
            CorticalAreaType::BrainOutput(new_flag)
        };

        // Update RuntimeGenome if available
        if let Some(genome) = self.current_genome.write().as_mut() {
            let mut area = genome.cortical_areas.remove(cortical_id).ok_or_else(|| {
                ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: cortical_id.as_base_64(),
                }
            })?;
            area.cortical_id = computed_id;
            area.cortical_type = new_cortical_type;
            genome.cortical_areas.insert(computed_id, area);

            for region in genome.brain_regions.values_mut() {
                if region.cortical_areas.remove(cortical_id) {
                    region.cortical_areas.insert(computed_id);
                }
            }

            let old_id_str = cortical_id.as_base_64();
            let new_id_str = computed_id.as_base_64();
            for area in genome.cortical_areas.values_mut() {
                if let Some(mapping) = area
                    .properties
                    .get_mut("cortical_mapping_dst")
                    .and_then(|v| v.as_object_mut())
                {
                    if let Some(value) = mapping.remove(&old_id_str) {
                        mapping.insert(new_id_str.clone(), value);
                    }
                }
            }
        }

        // Update ConnectomeManager (runtime only; keep NPU registry unchanged)
        {
            let mut manager = self.connectome.write();
            manager.rename_cortical_area_id_with_options(
                cortical_id,
                computed_id,
                new_cortical_type,
                false,
            )?;
        }

        // Refresh burst runner cache to propagate updated cortical_id mapping.
        self.refresh_burst_runner_cache();

        Ok(computed_id)
    }

    /// Update the unit index (group_id) for an IO cortical area and remap the cortical ID.
    ///
    /// This updates both RuntimeGenome and ConnectomeManager, preserving cortical_idx.
    fn apply_unit_index_update(
        &self,
        cortical_id: &CorticalID,
        cortical_id_str: &str,
        changes: &HashMap<String, Value>,
    ) -> ServiceResult<CorticalID> {
        let new_group_id_value = changes
            .get("group_id")
            .ok_or_else(|| {
                ServiceError::InvalidInput("group_id is required for unit index update".to_string())
            })?;
        let new_group_id = if let Some(value) = new_group_id_value.as_u64() {
            value
                .try_into()
                .map_err(|_| ServiceError::InvalidInput("group_id out of range".to_string()))?
        } else if let Some(value) = new_group_id_value.as_f64() {
            if value.fract() != 0.0 {
                return Err(ServiceError::InvalidInput(
                    "group_id must be an integer".to_string(),
                ));
            }
            let as_u64 = value as u64;
            as_u64
                .try_into()
                .map_err(|_| ServiceError::InvalidInput("group_id out of range".to_string()))?
        } else if let Some(raw) = new_group_id_value.as_str() {
            raw.parse::<u8>().map_err(|_| {
                ServiceError::InvalidInput("group_id must be an integer".to_string())
            })?
        } else {
            return Err(ServiceError::InvalidInput(
                "group_id must be an integer".to_string(),
            ));
        };

        let bytes = cortical_id.as_bytes();
        let is_input = bytes[0] == b'i';
        let is_output = bytes[0] == b'o';
        if !is_input && !is_output {
            return Err(ServiceError::InvalidInput(
                "Unit index updates only apply to IPU/OPU cortical areas".to_string(),
            ));
        }

        let current_flag = cortical_id.extract_io_data_flag().map_err(|e| {
            ServiceError::InvalidInput(format!(
                "Unable to decode IO configuration from cortical ID '{}': {}",
                cortical_id_str, e
            ))
        })?;
        let unit_identifier = [bytes[1], bytes[2], bytes[3]];
        let cortical_subunit_index = CorticalSubUnitIndex::from(bytes[6]);
        let cortical_unit_index = CorticalUnitIndex::from(new_group_id);
        let computed_id = current_flag.as_io_cortical_id(
            is_input,
            unit_identifier,
            cortical_unit_index,
            cortical_subunit_index,
        );

        let new_cortical_type = if is_input {
            CorticalAreaType::BrainInput(current_flag)
        } else {
            CorticalAreaType::BrainOutput(current_flag)
        };

        if &computed_id == cortical_id {
            if let Some(genome) = self.current_genome.write().as_mut() {
                if let Some(area) = genome.cortical_areas.get_mut(cortical_id) {
                    area.properties
                        .insert("group_id".to_string(), serde_json::json!(new_group_id));
                }
            }
            let mut manager = self.connectome.write();
            if let Some(area) = manager.get_cortical_area_mut(cortical_id) {
                area.properties
                    .insert("group_id".to_string(), serde_json::json!(new_group_id));
            }
            return Ok(*cortical_id);
        }

        info!(
            target: "feagi-services",
            "[UNIT-INDEX] Remapping cortical ID {} -> {}",
            cortical_id.as_base_64(),
            computed_id.as_base_64()
        );

        // Update RuntimeGenome if available
        if let Some(genome) = self.current_genome.write().as_mut() {
            let mut area = genome.cortical_areas.remove(cortical_id).ok_or_else(|| {
                ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: cortical_id.as_base_64(),
                }
            })?;
            area.cortical_id = computed_id;
            area.cortical_type = new_cortical_type;
            area.properties
                .insert("group_id".to_string(), serde_json::json!(new_group_id));
            genome.cortical_areas.insert(computed_id, area);

            for region in genome.brain_regions.values_mut() {
                if region.cortical_areas.remove(cortical_id) {
                    region.cortical_areas.insert(computed_id);
                }
            }

            let old_id_str = cortical_id.as_base_64();
            let new_id_str = computed_id.as_base_64();
            for area in genome.cortical_areas.values_mut() {
                if let Some(mapping) = area
                    .properties
                    .get_mut("cortical_mapping_dst")
                    .and_then(|v| v.as_object_mut())
                {
                    if let Some(value) = mapping.remove(&old_id_str) {
                        mapping.insert(new_id_str.clone(), value);
                    }
                }
            }
        }

        // Update ConnectomeManager (runtime only; keep NPU registry unchanged)
        {
            let mut manager = self.connectome.write();
            manager.rename_cortical_area_id_with_options(
                cortical_id,
                computed_id,
                new_cortical_type,
                false,
            )?;
            if let Some(area) = manager.get_cortical_area_mut(&computed_id) {
                area.properties
                    .insert("group_id".to_string(), serde_json::json!(new_group_id));
            }
        }

        // Refresh burst runner cache to propagate updated cortical_id mapping.
        self.refresh_burst_runner_cache();

        Ok(computed_id)
    }

    /// Regenerate incoming/outgoing synapses for a cortical area without rebuilding neurons.
    fn regenerate_mappings_for_area(&self, cortical_id: &CorticalID) -> ServiceResult<()> {
        let mut manager = self.connectome.write();
        let cortical_id_str = cortical_id.as_base_64();

        let outgoing_targets = {
            let Some(area) = manager.get_cortical_area(cortical_id) else {
                return Err(ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: cortical_id_str.clone(),
                });
            };
            let mut targets = Vec::new();
            if let Some(mapping) = area
                .properties
                .get("cortical_mapping_dst")
                .and_then(|v| v.as_object())
            {
                for key in mapping.keys() {
                    match CorticalID::try_from_base_64(key) {
                        Ok(dst_id) => targets.push(dst_id),
                        Err(err) => warn!(
                            target: "feagi-services",
                            "Invalid cortical_mapping_dst ID '{}': {}",
                            key,
                            err
                        ),
                    }
                }
            }
            targets
        };

        let incoming_sources = {
            let mut sources = Vec::new();
            for src_id in manager.get_all_cortical_ids() {
                if src_id == *cortical_id {
                    continue;
                }
                let Some(src_area) = manager.get_cortical_area(&src_id) else {
                    continue;
                };
                if let Some(mapping) = src_area
                    .properties
                    .get("cortical_mapping_dst")
                    .and_then(|v| v.as_object())
                {
                    if mapping.contains_key(&cortical_id_str) {
                        sources.push(src_id);
                    }
                }
            }
            sources
        };

        for dst_id in outgoing_targets {
            if dst_id == *cortical_id {
                continue;
            }
            manager
                .regenerate_synapses_for_mapping(cortical_id, &dst_id)
                .map_err(|e| {
                    ServiceError::Backend(format!(
                        "Failed to regenerate outgoing synapses {} -> {}: {}",
                        cortical_id.as_base_64(),
                        dst_id.as_base_64(),
                        e
                    ))
                })?;
        }

        for src_id in incoming_sources {
            manager
                .regenerate_synapses_for_mapping(&src_id, cortical_id)
                .map_err(|e| {
                    ServiceError::Backend(format!(
                        "Failed to regenerate incoming synapses {} -> {}: {}",
                        src_id.as_base_64(),
                        cortical_id.as_base_64(),
                        e
                    ))
                })?;
        }

        Ok(())
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
        let needs_burst_cache_refresh = changes.contains_key("visualization_voxel_granularity");
        let mut properties_changed = false;
        let mut geometry_changed = false;
        for key in changes.keys() {
            match key.as_str() {
                "coordinates_3d" | "coordinate_3d" | "coordinates" | "position"
                | "coordinate_2d" | "coordinates_2d" => {
                    geometry_changed = true;
                }
                "cortical_name" | "name" | "visible" | "visualization_voxel_granularity" => {
                    properties_changed = true;
                }
                _ => {
                    properties_changed = true;
                }
            }
        }

        // Convert cortical_id to CorticalID
        let cortical_id_typed =
            feagi_evolutionary::string_to_cortical_id(cortical_id).map_err(|e| {
                ServiceError::InvalidInput(format!("Invalid cortical ID '{}': {}", cortical_id, e))
            })?;
        let mut effective_cortical_id = cortical_id_typed;
        let mut effective_cortical_id_str = cortical_id.to_string();

        let has_coding_update = changes.contains_key("coding_signage")
            || changes.contains_key("coding_behavior")
            || changes.contains_key("coding_type")
            || changes.contains_key("new_cortical_id");
        if has_coding_update {
            let new_id = self.apply_io_coding_update(&cortical_id_typed, cortical_id, &changes)?;
            effective_cortical_id = new_id;
            effective_cortical_id_str = new_id.as_base_64();
        }

        // Update RuntimeGenome if available
        if let Some(genome) = self.current_genome.write().as_mut() {
            if let Some(area) = genome.cortical_areas.get_mut(&effective_cortical_id) {
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
                        "coordinate_2d" | "coordinates_2d" => {
                            if let Some(arr) = value.as_array() {
                                if arr.len() >= 2 {
                                    let x = arr[0].as_i64().unwrap_or(0) as i32;
                                    let y = arr[1].as_i64().unwrap_or(0) as i32;
                                    area.properties.insert(
                                        "coordinate_2d".to_string(),
                                        serde_json::json!([x, y]),
                                    );
                                    info!(target: "feagi-services", "[GENOME-UPDATE] Updated coordinate_2d: ({}, {})", x, y);
                                } else {
                                    warn!(target: "feagi-services", "[GENOME-UPDATE] coordinate_2d array must have 2 elements, got {}", arr.len());
                                }
                            } else if let Some(obj) = value.as_object() {
                                let x = obj.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                let y = obj.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                area.properties
                                    .insert("coordinate_2d".to_string(), serde_json::json!([x, y]));
                                info!(target: "feagi-services", "[GENOME-UPDATE] Updated coordinate_2d (object format): ({}, {})", x, y);
                            } else {
                                warn!(target: "feagi-services", "[GENOME-UPDATE] coordinate_2d must be array or object, got: {:?}", value);
                            }
                        }
                        "visualization_voxel_granularity" => {
                            // Only store if != 1x1x1 (default), delete if set to 1x1x1
                            info!(target: "feagi-services", "[GENOME-UPDATE] Received visualization_voxel_granularity update: {:?}", value);
                            if let Some(arr) = value.as_array() {
                                if arr.len() == 3 {
                                    // Try to parse as integers (u64) or floats (f64), then convert to u32
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
                                        let x_u32 = x as u32;
                                        let y_u32 = y as u32;
                                        let z_u32 = z as u32;

                                        // Default is 1x1x1 - only store if different
                                        if x_u32 == 1 && y_u32 == 1 && z_u32 == 1 {
                                            // Remove override (return to default)
                                            area.properties
                                                .remove("visualization_voxel_granularity");
                                            info!(target: "feagi-services", "[GENOME-UPDATE] Removed visualization_voxel_granularity override (returned to default 1x1x1)");
                                        } else {
                                            // Store override (non-default value) as integer array
                                            area.properties.insert(
                                                "visualization_voxel_granularity".to_string(),
                                                serde_json::json!([x_u32, y_u32, z_u32]),
                                            );
                                            info!(target: "feagi-services", "[GENOME-UPDATE] Updated visualization_voxel_granularity: ({}, {}, {})", x_u32, y_u32, z_u32);
                                        }
                                    } else {
                                        warn!(target: "feagi-services", "[GENOME-UPDATE] Failed to parse visualization_voxel_granularity array values as integers");
                                    }
                                } else {
                                    warn!(target: "feagi-services", "[GENOME-UPDATE] visualization_voxel_granularity array must have 3 elements, got {}", arr.len());
                                }
                            } else {
                                warn!(target: "feagi-services", "[GENOME-UPDATE] visualization_voxel_granularity must be an array, got: {:?}", value);
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
                .get_cortical_area_mut(&effective_cortical_id)
                .ok_or_else(|| ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: effective_cortical_id_str.clone(),
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
                    "coordinate_2d" | "coordinates_2d" => {
                        if let Some(arr) = value.as_array() {
                            if arr.len() >= 2 {
                                let x = arr[0].as_i64().unwrap_or(0) as i32;
                                let y = arr[1].as_i64().unwrap_or(0) as i32;
                                area.properties
                                    .insert("coordinate_2d".to_string(), serde_json::json!([x, y]));
                                info!(target: "feagi-services", "[CONNECTOME-UPDATE] Updated coordinate_2d: ({}, {})", x, y);
                            } else {
                                warn!(target: "feagi-services", "[CONNECTOME-UPDATE] coordinate_2d array must have 2 elements, got {}", arr.len());
                            }
                        } else if let Some(obj) = value.as_object() {
                            let x = obj.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                            let y = obj.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                            area.properties
                                .insert("coordinate_2d".to_string(), serde_json::json!([x, y]));
                            info!(target: "feagi-services", "[CONNECTOME-UPDATE] Updated coordinate_2d (object format): ({}, {})", x, y);
                        } else {
                            warn!(target: "feagi-services", "[CONNECTOME-UPDATE] coordinate_2d must be array or object, got: {:?}", value);
                        }
                    }
                    "visualization_voxel_granularity" => {
                        // Only store if != 1x1x1 (default), delete if set to 1x1x1
                        info!(target: "feagi-services", "[CONNECTOME-UPDATE] Received visualization_voxel_granularity update: {:?}", value);
                        if let Some(arr) = value.as_array() {
                            if arr.len() == 3 {
                                // Try to parse as integers (u64) or floats (f64), then convert to u32
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
                                    let x_u32 = x as u32;
                                    let y_u32 = y as u32;
                                    let z_u32 = z as u32;

                                    // Default is 1x1x1 - only store if different
                                    if x_u32 == 1 && y_u32 == 1 && z_u32 == 1 {
                                        // Remove override (return to default)
                                        area.properties.remove("visualization_voxel_granularity");
                                        info!(target: "feagi-services", "[CONNECTOME-UPDATE] Removed visualization_voxel_granularity override (returned to default 1x1x1)");
                                    } else {
                                        // Store override (non-default value) as integer array
                                        area.properties.insert(
                                            "visualization_voxel_granularity".to_string(),
                                            serde_json::json!([x_u32, y_u32, z_u32]),
                                        );
                                        info!(target: "feagi-services", "[CONNECTOME-UPDATE] Updated visualization_voxel_granularity: ({}, {}, {})", x_u32, y_u32, z_u32);
                                    }
                                } else {
                                    warn!(target: "feagi-services", "[CONNECTOME-UPDATE] Failed to parse visualization_voxel_granularity array values as integers");
                                }
                            } else {
                                warn!(target: "feagi-services", "[CONNECTOME-UPDATE] visualization_voxel_granularity array must have 3 elements, got {}", arr.len());
                            }
                        } else {
                            warn!(target: "feagi-services", "[CONNECTOME-UPDATE] visualization_voxel_granularity must be an array, got: {:?}", value);
                        }
                    }
                    _ => {}
                }
            }
            manager.refresh_cortical_area_hashes(properties_changed, geometry_changed);
        }

        // Refresh burst runner cache so the NPU aggregation path immediately uses new granularity.
        if needs_burst_cache_refresh {
            self.refresh_burst_runner_cache();
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

        if changes.contains_key("group_id") && changes.len() == 1 {
            let new_id = self.apply_unit_index_update(&_cortical_id_typed, cortical_id, &changes)?;
            self.regenerate_mappings_for_area(&new_id)?;
            return self.get_cortical_area_info(&new_id.as_base_64()).await;
        }

        // Must run on blocking thread due to heavy ConnectomeManager operations
        let connectome = Arc::clone(&self.connectome);
        let genome_store = Arc::clone(&self.current_genome);
        let cortical_id_owned = cortical_id.to_string();
        let burst_runner_clone = self.burst_runner.clone();

        tokio::task::spawn_blocking(move || {
            Self::do_localized_rebuild(
                &cortical_id_owned,
                changes,
                connectome,
                genome_store,
                burst_runner_clone,
            )
        })
        .await
        .map_err(|e| ServiceError::Backend(format!("Rebuild task panicked: {}", e)))?
    }

    /// Perform localized rebuild (blocking operation)
    fn do_localized_rebuild(
        cortical_id: &str,
        changes: HashMap<String, Value>,
        connectome: Arc<RwLock<ConnectomeManager>>,
        genome_store: Arc<RwLock<Option<feagi_evolutionary::RuntimeGenome>>>,
        burst_runner: Option<Arc<RwLock<BurstLoopRunner>>>,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!(
            "[STRUCTURAL-REBUILD] Starting localized rebuild for {}",
            cortical_id
        );

        // Convert cortical_id to CorticalID
        let cortical_id_typed =
            feagi_evolutionary::string_to_cortical_id(cortical_id).map_err(|e| {
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

            // Update spatial gradient increment properties if changed
            // These require rebuild because thresholds must be recalculated based on position
            // Handle neuron_fire_threshold_increment as ARRAY [x, y, z] from BV
            if let Some(value) = changes.get("neuron_fire_threshold_increment") {
                if let Some(arr) = value.as_array() {
                    // Parse array [x, y, z] into separate properties
                    if arr.len() >= 3 {
                        let x = arr[0].as_f64().unwrap_or(0.0) as f32;
                        let y = arr[1].as_f64().unwrap_or(0.0) as f32;
                        let z = arr[2].as_f64().unwrap_or(0.0) as f32;

                        area.properties.insert(
                            "firing_threshold_increment_x".to_string(),
                            serde_json::json!(x),
                        );
                        area.properties.insert(
                            "firing_threshold_increment_y".to_string(),
                            serde_json::json!(y),
                        );
                        area.properties.insert(
                            "firing_threshold_increment_z".to_string(),
                            serde_json::json!(z),
                        );

                        info!(
                            "[STRUCTURAL-REBUILD] Updated firing_threshold_increment to [{}, {}, {}] for area {}",
                            x, y, z, cortical_id
                        );
                    }
                }
            }

            // Handle individual x/y/z properties if sent separately
            for increment_param in [
                "firing_threshold_increment_x",
                "firing_threshold_increment_y",
                "firing_threshold_increment_z",
            ] {
                if let Some(value) = changes.get(increment_param) {
                    area.properties
                        .insert(increment_param.to_string(), value.clone());
                    info!(
                        "[STRUCTURAL-REBUILD] Updated {} to {} for area {}",
                        increment_param, value, cortical_id
                    );
                }
            }

            // Update leak_variability if changed (also requires rebuild)
            for param in ["leak_variability", "neuron_leak_variability"] {
                if let Some(value) = changes.get(param) {
                    area.properties
                        .insert("leak_variability".to_string(), value.clone());
                    info!(
                        "[STRUCTURAL-REBUILD] Updated leak_variability to {} for area {}",
                        value, cortical_id
                    );
                    break;
                }
            }

            (
                old_dims,
                old_dens,
                area.dimensions,
                area.neurons_per_voxel(),
            )
        };

        let total_voxels = new_dimensions.width as usize
            * new_dimensions.height as usize
            * new_dimensions.depth as usize;
        let estimated_neurons = total_voxels * new_density as usize;

        info!(
            "[STRUCTURAL-REBUILD] Dimension: {:?} -> {:?}",
            old_dimensions, new_dimensions
        );
        info!(
            "[STRUCTURAL-REBUILD] Density: {} -> {} neurons/voxel",
            old_density, new_density
        );

        if estimated_neurons > 1_000_000 {
            warn!(
                "[STRUCTURAL-REBUILD] ⚠️ Large area resize: {} neurons estimated. This may take significant time and memory.",
                estimated_neurons
            );
        }

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

                // ✅ Sync spatial gradient increment properties to ConnectomeManager
                // This ensures BV reads back the updated values
                // Handle neuron_fire_threshold_increment as ARRAY [x, y, z] from BV
                if let Some(value) = changes.get("neuron_fire_threshold_increment") {
                    if let Some(arr) = value.as_array() {
                        if arr.len() >= 3 {
                            let x = arr[0].as_f64().unwrap_or(0.0) as f32;
                            let y = arr[1].as_f64().unwrap_or(0.0) as f32;
                            let z = arr[2].as_f64().unwrap_or(0.0) as f32;

                            area.properties.insert(
                                "firing_threshold_increment_x".to_string(),
                                serde_json::json!(x),
                            );
                            area.properties.insert(
                                "firing_threshold_increment_y".to_string(),
                                serde_json::json!(y),
                            );
                            area.properties.insert(
                                "firing_threshold_increment_z".to_string(),
                                serde_json::json!(z),
                            );
                        }
                    }
                }

                // Handle individual x/y/z properties if sent separately
                for increment_param in [
                    "firing_threshold_increment_x",
                    "firing_threshold_increment_y",
                    "firing_threshold_increment_z",
                ] {
                    if let Some(value) = changes.get(increment_param) {
                        area.properties
                            .insert(increment_param.to_string(), value.clone());
                    }
                }

                // ✅ Sync leak_variability to ConnectomeManager
                for param in ["leak_variability", "neuron_leak_variability"] {
                    if let Some(value) = changes.get(param) {
                        area.properties
                            .insert("leak_variability".to_string(), value.clone());
                        break;
                    }
                }
            }
            manager.refresh_cortical_area_hashes(true, true);
        }

        // Step 4: Recreate neurons with new dimensions/density
        // CRITICAL PERFORMANCE FIX: Extract data from connectome, release lock, then create neurons
        // This prevents blocking API requests during the multi-second neuron creation process
        let (cortical_idx, area_data) = {
            let manager = connectome.read();
            let area = manager
                .get_cortical_area(&cortical_id_typed)
                .ok_or_else(|| ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: cortical_id.to_string(),
                })?;
            let cortical_idx = manager
                .get_cortical_idx(&cortical_id_typed)
                .ok_or_else(|| ServiceError::Backend("Cortical index not found".to_string()))?;

            // Extract all data needed for neuron creation
            use feagi_brain_development::models::CorticalAreaExt;
            (
                cortical_idx,
                (
                    area.dimensions,
                    area.neurons_per_voxel(),
                    area.firing_threshold(),
                    area.firing_threshold_increment_x(),
                    area.firing_threshold_increment_y(),
                    area.firing_threshold_increment_z(),
                    area.firing_threshold_limit(),
                    area.leak_coefficient(),
                    area.neuron_excitability(),
                    area.refractory_period(),
                    area.consecutive_fire_count() as u16,
                    area.snooze_period(),
                    area.mp_charge_accumulation(),
                ),
            )
        };

        // Release connectome lock before creating neurons (NPU lock will be held, but connectome is free for API)
        let npu_arc_for_creation = {
            let manager = connectome.read();
            manager
                .get_npu()
                .ok_or_else(|| ServiceError::Backend("NPU not connected".to_string()))?
                .clone()
        };

        // CRITICAL PERFORMANCE: Event-based lock management
        // Lock is held until create_cortical_area_neurons() completes (100% done)
        // Timing varies by hardware/topology - we measure actual time, not estimate
        // NOTE: Connectome lock is already released, so API can query connectome data
        let total_neurons =
            area_data.0.width * area_data.0.height * area_data.0.depth * area_data.1;

        if total_neurons > 1_000_000 {
            info!(
                "[STRUCTURAL-REBUILD] Creating large area ({} neurons) - NPU lock held until creation completes",
                total_neurons
            );
        }

        let creation_start = std::time::Instant::now();

        // CRITICAL PERFORMANCE: For very large areas (>1M neurons), batch by depth layers AND within layers
        // to release NPU lock frequently and allow burst loop to run
        let neurons_created = if total_neurons > 1_000_000 {
            // Batch by depth layers (z-dimension) - each layer is processed separately
            // Additionally, batch within each layer if layer is large (>100k neurons)
            let mut total_created = 0u32;
            let depth = area_data.0.depth;
            let width = area_data.0.width;
            let height = area_data.0.height;
            let neurons_per_voxel = area_data.1;
            let neurons_per_layer = (width * height * neurons_per_voxel) as usize;
            const BATCH_SIZE: usize = 10_000; // Process neurons in batches of 10k within each layer (reduces lock hold time)

            // Determine if we need to batch within layers
            let needs_inner_batching = neurons_per_layer > 50_000; // Lower threshold for inner batching

            info!(
                "[STRUCTURAL-REBUILD] Batching neuron creation: {} layers × {} neurons/layer = {} total{}",
                depth, neurons_per_layer, total_neurons,
                if needs_inner_batching { " (with inner-layer batching)" } else { "" }
            );

            for z_layer in 0..depth {
                if needs_inner_batching {
                    // Large layer: batch within the layer by processing rows in chunks
                    let mut layer_created = 0u32;
                    let neurons_per_row = (width * neurons_per_voxel) as usize;
                    let rows_per_batch = (BATCH_SIZE / neurons_per_row).max(1);
                    let total_row_batches = (height as usize).div_ceil(rows_per_batch);

                    if z_layer == 0 {
                        info!(
                            "[STRUCTURAL-REBUILD] Inner-layer batching: {} rows/layer, {} rows/batch, ~{} batches/layer",
                            height, rows_per_batch, total_row_batches
                        );
                    }

                    for (batch_idx, row_start) in (0..height).step_by(rows_per_batch).enumerate() {
                        let row_end = (row_start + rows_per_batch as u32).min(height);
                        let rows_in_batch = row_end - row_start;
                        let neurons_in_batch = (rows_in_batch * width * neurons_per_voxel) as usize;

                        let batch_start = std::time::Instant::now();

                        // Acquire lock, create batch of rows, release lock
                        let batch_created = {
                            let mut npu_lock = npu_arc_for_creation.lock().map_err(|e| {
                                ServiceError::Backend(format!("Failed to lock NPU: {}", e))
                            })?;

                            // Create neurons for this batch of rows (height=rows_in_batch, y_offset=row_start, z_offset=z_layer)
                            npu_lock
                                .create_cortical_area_neurons_with_offsets(
                                    cortical_idx,
                                    width,
                                    rows_in_batch,
                                    1, // depth=1 (single z-layer)
                                    neurons_per_voxel,
                                    area_data.2,  // default_threshold
                                    area_data.3,  // threshold_increment_x
                                    area_data.4,  // threshold_increment_y
                                    area_data.5,  // threshold_increment_z
                                    area_data.6,  // default_threshold_limit
                                    area_data.7,  // default_leak_coefficient
                                    0.0,          // default_resting_potential
                                    0,            // default_neuron_type
                                    area_data.9,  // default_refractory_period
                                    area_data.8,  // default_excitability
                                    area_data.10, // default_consecutive_fire_limit
                                    area_data.11, // default_snooze_period
                                    area_data.12, // default_mp_charge_accumulation
                                    row_start, // y_offset: create neurons starting at this y-coordinate
                                    z_layer,   // z_offset: create neurons at this z-coordinate
                                )
                                .map_err(|e| {
                                    ServiceError::Backend(format!(
                                        "NPU neuron creation failed for layer {} rows {}-{}: {}",
                                        z_layer, row_start, row_end, e
                                    ))
                                })?
                        };

                        let batch_duration = batch_start.elapsed();
                        layer_created += batch_created;

                        // Log if batch took too long (helps identify if batching is working)
                        if batch_duration.as_millis() > 100
                            && (batch_idx == 0 || batch_idx == total_row_batches - 1)
                        {
                            tracing::debug!(
                                "[STRUCTURAL-REBUILD] Layer {} batch {}/{}: {} neurons in {:.1}ms (rows {}-{})",
                                z_layer, batch_idx + 1, total_row_batches, neurons_in_batch,
                                batch_duration.as_millis(), row_start, row_end
                            );
                        }
                    }

                    total_created += layer_created;
                } else {
                    // Small layer: create entire layer in one batch
                    let layer_created = {
                        let mut npu_lock = npu_arc_for_creation.lock().map_err(|e| {
                            ServiceError::Backend(format!("Failed to lock NPU: {}", e))
                        })?;

                        // Create neurons for this z-layer only (depth=1, z_offset=z_layer)
                        npu_lock
                            .create_cortical_area_neurons_with_z_offset(
                                cortical_idx,
                                width,
                                height,
                                1, // depth=1 (single layer)
                                neurons_per_voxel,
                                area_data.2,  // default_threshold
                                area_data.3,  // threshold_increment_x
                                area_data.4,  // threshold_increment_y
                                area_data.5,  // threshold_increment_z
                                area_data.6,  // default_threshold_limit
                                area_data.7,  // default_leak_coefficient
                                0.0,          // default_resting_potential
                                0,            // default_neuron_type
                                area_data.9,  // default_refractory_period
                                area_data.8,  // default_excitability
                                area_data.10, // default_consecutive_fire_limit
                                area_data.11, // default_snooze_period
                                area_data.12, // default_mp_charge_accumulation
                                z_layer,      // z_offset: create neurons at this z-coordinate
                            )
                            .map_err(|e| {
                                ServiceError::Backend(format!(
                                    "NPU neuron creation failed for layer {}: {}",
                                    z_layer, e
                                ))
                            })?
                    };

                    total_created += layer_created;
                }

                // Log progress every 5 layers or on last layer
                if (z_layer + 1) % 5 == 0 || z_layer == depth - 1 {
                    let progress = ((z_layer + 1) as f32 / depth as f32) * 100.0;
                    info!(
                        "[STRUCTURAL-REBUILD] Progress: {}/{} layers ({}%) - {} neurons created",
                        z_layer + 1,
                        depth,
                        progress as u32,
                        total_created
                    );
                }
            }

            total_created
        } else {
            // Small area: create all neurons in one batch (single lock acquisition)
            let mut npu_lock = npu_arc_for_creation
                .lock()
                .map_err(|e| ServiceError::Backend(format!("Failed to lock NPU: {}", e)))?;

            npu_lock
                .create_cortical_area_neurons(
                    cortical_idx,
                    area_data.0.width,
                    area_data.0.height,
                    area_data.0.depth,
                    area_data.1,
                    area_data.2,
                    area_data.3,
                    area_data.4,
                    area_data.5,
                    area_data.6,
                    area_data.7,
                    0.0,
                    0,
                    area_data.9,
                    area_data.8,
                    area_data.10,
                    area_data.11,
                    area_data.12,
                )
                .map_err(|e| ServiceError::Backend(format!("NPU neuron creation failed: {}", e)))?
        };

        let creation_duration = creation_start.elapsed();
        info!(
            "[STRUCTURAL-REBUILD] Created {} neurons in {:.2}s (NPU lock held until completion)",
            neurons_created,
            creation_duration.as_secs_f64()
        );

        if creation_duration.as_secs() > 1 {
            warn!(
                "[STRUCTURAL-REBUILD] ⚠️ Long creation time: {:.2}s - burst loop was blocked during this period",
                creation_duration.as_secs_f64()
            );
        }

        // Step 5: Rebuild outgoing synapses (this area -> others)
        let outgoing_targets = {
            let genome_guard = genome_store.read();
            let genome = genome_guard.as_ref().ok_or_else(|| {
                ServiceError::Backend("No genome loaded".to_string())
            })?;
            let area = genome.cortical_areas.get(&cortical_id_typed).ok_or_else(|| {
                ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: cortical_id.to_string(),
                }
            })?;
            let mut targets = Vec::new();
            if let Some(mapping) = area
                .properties
                .get("cortical_mapping_dst")
                .and_then(|v| v.as_object())
            {
                for key in mapping.keys() {
                    match CorticalID::try_from_base_64(key) {
                        Ok(dst_id) => targets.push(dst_id),
                        Err(err) => warn!(
                            target: "feagi-services",
                            "Invalid cortical_mapping_dst ID '{}': {}",
                            key,
                            err
                        ),
                    }
                }
            }
            targets
        };
        let outgoing_synapses = {
            let mut manager = connectome.write();
            let mut total = 0u32;
            for dst_id in outgoing_targets {
                if dst_id == cortical_id_typed {
                    continue;
                }
                let count = manager
                    .regenerate_synapses_for_mapping(&cortical_id_typed, &dst_id)
                    .map_err(|e| {
                        ServiceError::Backend(format!(
                            "Failed to rebuild outgoing synapses {} -> {}: {}",
                            cortical_id,
                            dst_id.as_base_64(),
                            e
                        ))
                    })?;
                total = total.saturating_add(count as u32);
            }
            total
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
                            let count = manager
                                .regenerate_synapses_for_mapping(src_id, &cortical_id_typed)
                                .map_err(|e| {
                                ServiceError::Backend(format!(
                                    "Failed to rebuild incoming synapses from {}: {}",
                                    src_id, e
                                ))
                            })?;
                            total = total.saturating_add(count as u32);
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

        // Step 7: Rebuild synapse index to ensure all new synapses are visible to propagation engine
        // This is critical for large areas (e.g., 2M+ neurons) to prevent system hangs
        // IMPORTANT: Release connectome lock before acquiring NPU lock to avoid blocking other operations
        let npu_arc = {
            let manager = connectome.read();
            manager
                .get_npu()
                .ok_or_else(|| ServiceError::Backend("NPU not connected".to_string()))?
                .clone()
        };

        // CRITICAL PERFORMANCE: Release lock between operations to allow burst loop to run
        // Event-based: Lock held only until each operation completes, then released

        // CRITICAL PERFORMANCE: Event-based lock management - release lock after each operation completes
        // This allows burst loop to run between operations, keeping system responsive

        info!(
            "[STRUCTURAL-REBUILD] Rebuilding synapse index for {} neurons...",
            neurons_created
        );
        let index_rebuild_start = std::time::Instant::now();
        {
            let mut npu_lock = npu_arc
                .lock()
                .map_err(|e| ServiceError::Backend(format!("Failed to lock NPU: {}", e)))?;

            npu_lock.rebuild_synapse_index();
            // Lock released here when scope ends (event-based: operation 100% complete)
        }
        let index_rebuild_duration = index_rebuild_start.elapsed();
        info!(
            "[STRUCTURAL-REBUILD] Synapse index rebuild complete in {:.2}s (NPU lock released)",
            index_rebuild_duration.as_secs_f64()
        );
        if index_rebuild_duration.as_millis() > 100 {
            warn!(
                "[STRUCTURAL-REBUILD] ⚠️ Slow synapse index rebuild: {:.2}s",
                index_rebuild_duration.as_secs_f64()
            );
        }

        // No power neuron cache rebuild needed - power neuron is always neuron ID 1 (deterministic)
        // Direct O(1) access in phase1_injection_with_synapses, no cache required!

        info!(
            "[STRUCTURAL-REBUILD] ✅ Complete: {} neurons, {} outgoing, {} incoming synapses",
            neurons_created, outgoing_synapses, incoming_synapses
        );

        // Refresh burst runner cache after structural rebuild (areas may have been resized)
        if let Some(ref burst_runner) = burst_runner {
            let manager = connectome.read();
            let mappings = manager.get_all_cortical_idx_to_id_mappings();
            let mapping_count = mappings.len();
            burst_runner.write().refresh_cortical_id_mappings(mappings);
            info!(target: "feagi-services", "Refreshed burst runner cache with {} cortical areas", mapping_count);
        }

        // CRITICAL PERFORMANCE: For large areas, skip expensive get_synapse_count_in_area
        // which iterates through all neurons (5.7M!) holding the NPU lock for hundreds of ms
        // Use known synapse counts from rebuild instead (we just calculated them)
        if neurons_created > 100_000 {
            info!(
                "[STRUCTURAL-REBUILD] Using known counts for large area ({} neurons) to avoid expensive NPU lock",
                neurons_created
            );
            // Get area info but use known counts (avoid expensive NPU iteration)
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

            // Use known counts from rebuild (no expensive NPU lock needed)
            let neuron_count = neurons_created as usize;
            let synapse_count = (outgoing_synapses + incoming_synapses) as usize;
            let cortical_group = area.get_cortical_group();
            let cortical_bytes = cortical_id_typed.as_bytes();
            let is_io_area = cortical_bytes[0] == b'i' || cortical_bytes[0] == b'o';
            let io_flag = if is_io_area {
                cortical_id_typed.extract_io_data_flag().ok()
            } else {
                None
            };
            let cortical_subtype = if is_io_area {
                String::from_utf8(cortical_bytes[0..4].to_vec()).ok()
            } else {
                None
            };
            let unit_id = if is_io_area {
                Some(cortical_bytes[6])
            } else {
                None
            };
            let group_id = if is_io_area {
                Some(cortical_bytes[7])
            } else {
                None
            };
            let coding_signage = io_flag
                .as_ref()
                .map(|flag| signage_label_from_flag(flag).to_string());
            let coding_behavior = io_flag
                .as_ref()
                .map(|flag| behavior_label_from_flag(flag).to_string());
            let coding_type = io_flag
                .as_ref()
                .map(|flag| coding_type_label_from_flag(flag).to_string());
            let coding_options = if is_io_area {
                io_coding_options_for_unit(&cortical_id_typed)
            } else {
                None
            };
            let encoding_type = coding_behavior.clone();
            let encoding_format = coding_type.clone();

            // Build full response using area properties (same as get_cortical_area_info_blocking)
            Ok(CorticalAreaInfo {
                cortical_id: area.cortical_id.as_base_64(),
                cortical_id_s: area.cortical_id.to_string(),
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
                cortical_group: cortical_group
                    .clone()
                    .unwrap_or_else(|| "CUSTOM".to_string()),
                cortical_type: {
                    use feagi_evolutionary::extract_memory_properties;
                    if extract_memory_properties(&area.properties).is_some() {
                        "memory".to_string()
                    } else if let Some(group) = &cortical_group {
                        match group.as_str() {
                            "IPU" => "sensory".to_string(),
                            "OPU" => "motor".to_string(),
                            "CORE" => "core".to_string(),
                            _ => "custom".to_string(),
                        }
                    } else {
                        "custom".to_string()
                    }
                },
                neuron_count,
                synapse_count,
                visible: area.visible(),
                sub_group: area.sub_group(),
                neurons_per_voxel: area.neurons_per_voxel(),
                postsynaptic_current: area.postsynaptic_current() as f64,
                postsynaptic_current_max: area.postsynaptic_current_max() as f64,
                plasticity_constant: area.plasticity_constant() as f64,
                degeneration: area.degeneration() as f64,
                psp_uniform_distribution: area.psp_uniform_distribution(),
                mp_driven_psp: area.mp_driven_psp(),
                firing_threshold: area.firing_threshold() as f64,
                firing_threshold_increment: [
                    area.firing_threshold_increment_x() as f64,
                    area.firing_threshold_increment_y() as f64,
                    area.firing_threshold_increment_z() as f64,
                ],
                firing_threshold_limit: area.firing_threshold_limit() as f64,
                consecutive_fire_count: area.consecutive_fire_count(),
                snooze_period: area.snooze_period() as u32,
                refractory_period: area.refractory_period() as u32,
                leak_coefficient: area.leak_coefficient() as f64,
                leak_variability: area.leak_variability() as f64,
                mp_charge_accumulation: area.mp_charge_accumulation(),
                neuron_excitability: area.neuron_excitability() as f64,
                burst_engine_active: area.burst_engine_active(),
                init_lifespan: area.init_lifespan(),
                lifespan_growth_rate: area.lifespan_growth_rate() as f64,
                longterm_mem_threshold: area.longterm_mem_threshold(),
                temporal_depth: {
                    use feagi_evolutionary::extract_memory_properties;
                    extract_memory_properties(&area.properties).map(|p| p.temporal_depth.max(1))
                },
                properties: HashMap::new(),
                cortical_subtype,
                encoding_type,
                encoding_format,
                unit_id,
                group_id,
                coding_signage,
                coding_behavior,
                coding_type,
                coding_options,
                parent_region_id: manager.get_parent_region_id_for_area(&cortical_id_typed),
                dev_count: area
                    .properties
                    .get("dev_count")
                    .and_then(|v| v.as_u64().map(|n| n as usize)),
                cortical_dimensions_per_device: area
                    .properties
                    .get("cortical_dimensions_per_device")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        if arr.len() == 3 {
                            Some((
                                arr[0].as_u64()? as usize,
                                arr[1].as_u64()? as usize,
                                arr[2].as_u64()? as usize,
                            ))
                        } else {
                            None
                        }
                    }),
                visualization_voxel_granularity: area
                    .properties
                    .get("visualization_voxel_granularity")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        if arr.len() == 3 {
                            Some((
                                arr[0].as_u64()? as u32,
                                arr[1].as_u64()? as u32,
                                arr[2].as_u64()? as u32,
                            ))
                        } else {
                            None
                        }
                    }),
            })
        } else {
            // For smaller areas, use the full info retrieval
            Self::get_cortical_area_info_blocking(cortical_id, &connectome)
        }
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
        let cortical_bytes = cortical_id_typed.as_bytes();
        let is_io_area = cortical_bytes[0] == b'i' || cortical_bytes[0] == b'o';
        let io_flag = if is_io_area {
            cortical_id_typed.extract_io_data_flag().ok()
        } else {
            None
        };
        let cortical_subtype = if is_io_area {
            String::from_utf8(cortical_bytes[0..4].to_vec()).ok()
        } else {
            None
        };
        let unit_id = if is_io_area {
            Some(cortical_bytes[6])
        } else {
            None
        };
        let group_id = if is_io_area {
            Some(cortical_bytes[7])
        } else {
            None
        };
        let coding_signage = io_flag
            .as_ref()
            .map(|flag| signage_label_from_flag(flag).to_string());
        let coding_behavior = io_flag
            .as_ref()
            .map(|flag| behavior_label_from_flag(flag).to_string());
        let coding_type = io_flag
            .as_ref()
            .map(|flag| coding_type_label_from_flag(flag).to_string());
        let coding_options = if is_io_area {
            io_coding_options_for_unit(&cortical_id_typed)
        } else {
            None
        };
        let encoding_type = coding_behavior.clone();
        let encoding_format = coding_type.clone();

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
            cortical_group: cortical_group
                .clone()
                .unwrap_or_else(|| "CUSTOM".to_string()),
            // Determine cortical_type based on properties
            cortical_type: {
                use feagi_evolutionary::extract_memory_properties;
                if extract_memory_properties(&area.properties).is_some() {
                    "memory".to_string()
                } else if let Some(group) = &cortical_group {
                    match group.as_str() {
                        "IPU" => "sensory".to_string(),
                        "OPU" => "motor".to_string(),
                        "CORE" => "core".to_string(),
                        _ => "custom".to_string(),
                    }
                } else {
                    "custom".to_string()
                }
            },
            neuron_count,
            synapse_count,
            visible: area.visible(),
            sub_group: area.sub_group(),
            neurons_per_voxel: area.neurons_per_voxel(),
            postsynaptic_current: area.postsynaptic_current() as f64,
            postsynaptic_current_max: area.postsynaptic_current_max() as f64,
            plasticity_constant: area.plasticity_constant() as f64,
            degeneration: area.degeneration() as f64,
            psp_uniform_distribution: area.psp_uniform_distribution(),
            mp_driven_psp: area.mp_driven_psp(),
            firing_threshold: area.firing_threshold() as f64,
            firing_threshold_increment: [
                area.firing_threshold_increment_x() as f64,
                area.firing_threshold_increment_y() as f64,
                area.firing_threshold_increment_z() as f64,
            ],
            firing_threshold_limit: area.firing_threshold_limit() as f64,
            consecutive_fire_count: area.consecutive_fire_count(),
            snooze_period: area.snooze_period() as u32,
            refractory_period: area.refractory_period() as u32,
            leak_coefficient: area.leak_coefficient() as f64,
            leak_variability: area.leak_variability() as f64,
            mp_charge_accumulation: area.mp_charge_accumulation(),
            neuron_excitability: area.neuron_excitability() as f64,
            burst_engine_active: area.burst_engine_active(),
            init_lifespan: area.init_lifespan(),
            lifespan_growth_rate: area.lifespan_growth_rate() as f64,
            longterm_mem_threshold: area.longterm_mem_threshold(),
            temporal_depth: {
                use feagi_evolutionary::extract_memory_properties;
                extract_memory_properties(&area.properties).map(|p| p.temporal_depth.max(1))
            },
            properties: HashMap::new(),
            cortical_subtype,
            encoding_type,
            encoding_format,
            unit_id,
            group_id,
            coding_signage,
            coding_behavior,
            coding_type,
            coding_options,
            parent_region_id: manager.get_parent_region_id_for_area(&cortical_id_typed),
            // Extract dev_count and cortical_dimensions_per_device from properties for IPU/OPU
            dev_count: area
                .properties
                .get("dev_count")
                .and_then(|v| v.as_u64().map(|n| n as usize)),
            cortical_dimensions_per_device: area
                .properties
                .get("cortical_dimensions_per_device")
                .and_then(|v| v.as_array())
                .and_then(|arr| {
                    if arr.len() == 3 {
                        Some((
                            arr[0].as_u64()? as usize,
                            arr[1].as_u64()? as usize,
                            arr[2].as_u64()? as usize,
                        ))
                    } else {
                        None
                    }
                }),
            visualization_voxel_granularity: area
                .properties
                .get("visualization_voxel_granularity")
                .and_then(|v| v.as_array())
                .and_then(|arr| {
                    if arr.len() == 3 {
                        Some((
                            arr[0].as_u64()? as u32,
                            arr[1].as_u64()? as u32,
                            arr[2].as_u64()? as u32,
                        ))
                    } else {
                        None
                    }
                }),
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

        // DIAGNOSTIC: Log the position, dimensions, and visibility of the area being queried
        tracing::info!(target: "feagi-services",
            "get_cortical_area_info: querying {} - position {:?}, dimensions {:?}, visible: {}",
            cortical_id, area.position, area.dimensions, area.visible()
        );

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
        let cortical_bytes = cortical_id_typed.as_bytes();
        let is_io_area = cortical_bytes[0] == b'i' || cortical_bytes[0] == b'o';
        let io_flag = if is_io_area {
            cortical_id_typed.extract_io_data_flag().ok()
        } else {
            None
        };
        let cortical_subtype = if is_io_area {
            String::from_utf8(cortical_bytes[0..4].to_vec()).ok()
        } else {
            None
        };
        let unit_id = if is_io_area {
            Some(cortical_bytes[6])
        } else {
            None
        };
        let group_id = if is_io_area {
            Some(cortical_bytes[7])
        } else {
            None
        };
        let coding_signage = io_flag
            .as_ref()
            .map(|flag| signage_label_from_flag(flag).to_string());
        let coding_behavior = io_flag
            .as_ref()
            .map(|flag| behavior_label_from_flag(flag).to_string());
        let coding_type = io_flag
            .as_ref()
            .map(|flag| coding_type_label_from_flag(flag).to_string());
        let coding_options = if is_io_area {
            io_coding_options_for_unit(&cortical_id_typed)
        } else {
            None
        };
        let encoding_type = coding_behavior.clone();
        let encoding_format = coding_type.clone();

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
            cortical_group: cortical_group
                .clone()
                .unwrap_or_else(|| "CUSTOM".to_string()),
            // Determine cortical_type based on properties
            cortical_type: {
                use feagi_evolutionary::extract_memory_properties;
                if extract_memory_properties(&area.properties).is_some() {
                    "memory".to_string()
                } else if let Some(group) = &cortical_group {
                    match group.as_str() {
                        "IPU" => "sensory".to_string(),
                        "OPU" => "motor".to_string(),
                        "CORE" => "core".to_string(),
                        _ => "custom".to_string(),
                    }
                } else {
                    "custom".to_string()
                }
            },
            neuron_count,
            synapse_count,
            visible: area.visible(),
            sub_group: area.sub_group(),
            neurons_per_voxel: area.neurons_per_voxel(),
            postsynaptic_current: area.postsynaptic_current() as f64,
            postsynaptic_current_max: area.postsynaptic_current_max() as f64,
            plasticity_constant: area.plasticity_constant() as f64,
            degeneration: area.degeneration() as f64,
            psp_uniform_distribution: area.psp_uniform_distribution(),
            mp_driven_psp: area.mp_driven_psp(),
            firing_threshold: area.firing_threshold() as f64,
            firing_threshold_increment: [
                area.firing_threshold_increment_x() as f64,
                area.firing_threshold_increment_y() as f64,
                area.firing_threshold_increment_z() as f64,
            ],
            firing_threshold_limit: area.firing_threshold_limit() as f64,
            consecutive_fire_count: area.consecutive_fire_count(),
            snooze_period: area.snooze_period() as u32,
            refractory_period: area.refractory_period() as u32,
            leak_coefficient: area.leak_coefficient() as f64,
            leak_variability: area.leak_variability() as f64,
            mp_charge_accumulation: area.mp_charge_accumulation(),
            neuron_excitability: area.neuron_excitability() as f64,
            burst_engine_active: area.burst_engine_active(),
            init_lifespan: area.init_lifespan(),
            lifespan_growth_rate: area.lifespan_growth_rate() as f64,
            longterm_mem_threshold: area.longterm_mem_threshold(),
            temporal_depth: {
                use feagi_evolutionary::extract_memory_properties;
                extract_memory_properties(&area.properties).map(|p| p.temporal_depth.max(1))
            },
            properties: HashMap::new(),
            cortical_subtype,
            encoding_type,
            encoding_format,
            unit_id,
            group_id,
            coding_signage,
            coding_behavior,
            coding_type,
            coding_options,
            parent_region_id: manager.get_parent_region_id_for_area(&cortical_id_typed),
            // Extract dev_count and cortical_dimensions_per_device from properties for IPU/OPU
            dev_count: area
                .properties
                .get("dev_count")
                .and_then(|v| v.as_u64().map(|n| n as usize)),
            cortical_dimensions_per_device: area
                .properties
                .get("cortical_dimensions_per_device")
                .and_then(|v| v.as_array())
                .and_then(|arr| {
                    if arr.len() == 3 {
                        Some((
                            arr[0].as_u64()? as usize,
                            arr[1].as_u64()? as usize,
                            arr[2].as_u64()? as usize,
                        ))
                    } else {
                        None
                    }
                }),
            visualization_voxel_granularity: area
                .properties
                .get("visualization_voxel_granularity")
                .and_then(|v| v.as_array())
                .and_then(|arr| {
                    if arr.len() == 3 {
                        Some((
                            arr[0].as_u64()? as u32,
                            arr[1].as_u64()? as u32,
                            arr[2].as_u64()? as u32,
                        ))
                    } else {
                        None
                    }
                }),
        })
    }
}
