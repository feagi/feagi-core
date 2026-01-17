// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Connectome service implementation.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::traits::ConnectomeService;
use crate::types::*;
use async_trait::async_trait;
use feagi_brain_development::models::CorticalAreaExt;
use feagi_brain_development::ConnectomeManager;
use feagi_npu_burst_engine::BurstLoopRunner;
use feagi_structures::genomic::brain_regions::{BrainRegion, RegionID, RegionType};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::genomic::cortical_area::IOCorticalAreaConfigurationFlag;
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
use feagi_structures::genomic::cortical_area::{CorticalArea, CorticalAreaDimensions};
// Note: decode_cortical_id removed - use feagi_structures::CorticalID directly
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, trace, warn};

fn derive_friendly_cortical_name(cortical_id: &CorticalID) -> Option<String> {
    let bytes = cortical_id.as_bytes();
    let is_input = bytes[0] == b'i';
    let is_output = bytes[0] == b'o';
    if !is_input && !is_output {
        return None;
    }

    let unit_ref: [u8; 3] = [bytes[1], bytes[2], bytes[3]];
    let subunit_index = bytes[6];
    let unit_index = bytes[7];

    if is_input {
        for unit in SensoryCorticalUnit::list_all() {
            if unit.get_cortical_id_unit_reference() == unit_ref {
                let unit_name = unit.get_friendly_name();
                let has_subunits = unit.get_number_cortical_areas() > 1;
                let name = if has_subunits {
                    format!("{} Subunit {} Unit {}", unit_name, subunit_index, unit_index)
                } else {
                    format!("{} Unit {}", unit_name, unit_index)
                };
                return Some(name);
            }
        }
    } else {
        for unit in MotorCorticalUnit::list_all() {
            if unit.get_cortical_id_unit_reference() == unit_ref {
                let unit_name = unit.get_friendly_name();
                let has_subunits = unit.get_number_cortical_areas() > 1;
                let name = if matches!(unit, MotorCorticalUnit::Gaze) {
                    let subunit_name = match subunit_index {
                        0 => "Eccentricity",
                        1 => "Modulation",
                        _ => "Subunit",
                    };
                    format!("{} ({}) Unit {}", unit_name, subunit_name, unit_index)
                } else if has_subunits {
                    format!("{} Subunit {} Unit {}", unit_name, subunit_index, unit_index)
                } else {
                    format!("{} Unit {}", unit_name, unit_index)
                };
                return Some(name);
            }
        }
    }

    None
}

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
        | IOCorticalAreaConfigurationFlag::SignedPercentage4D(frame, _) => frame_handling_label(*frame),
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
        IOCorticalAreaConfigurationFlag::CartesianPlane(_)
        | IOCorticalAreaConfigurationFlag::Misc(_)
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

fn io_coding_options_for_unit(
    cortical_id: &CorticalID,
) -> Option<IOCodingOptions> {
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

/// Update a cortical area's `cortical_mapping_dst` property in-place.
///
/// - When `mapping_data` is empty: remove the destination entry, and if the
///   container becomes empty remove `cortical_mapping_dst` entirely.
/// - When `mapping_data` is non-empty: insert/overwrite the destination entry.
fn update_cortical_mapping_dst_in_properties(
    properties: &mut HashMap<String, serde_json::Value>,
    dst_area_id: &str,
    mapping_data: &[serde_json::Value],
) -> ServiceResult<()> {
    if mapping_data.is_empty() {
        let Some(existing) = properties.get_mut("cortical_mapping_dst") else {
            return Ok(());
        };
        let Some(mapping_dst) = existing.as_object_mut() else {
            return Err(ServiceError::Backend(
                "cortical_mapping_dst is not a JSON object".to_string(),
            ));
        };

        mapping_dst.remove(dst_area_id);
        if mapping_dst.is_empty() {
            properties.remove("cortical_mapping_dst");
        }
        return Ok(());
    }

    let entry = properties
        .entry("cortical_mapping_dst".to_string())
        .or_insert_with(|| serde_json::json!({}));

    let Some(mapping_dst) = entry.as_object_mut() else {
        return Err(ServiceError::Backend(
            "cortical_mapping_dst is not a JSON object".to_string(),
        ));
    };

    mapping_dst.insert(dst_area_id.to_string(), serde_json::json!(mapping_data));
    Ok(())
}

/// Default implementation of ConnectomeService
pub struct ConnectomeServiceImpl {
    connectome: Arc<RwLock<ConnectomeManager>>,
    /// Currently loaded genome (source of truth for genome persistence)
    /// Shared with GenomeServiceImpl to ensure cortical mappings are saved
    current_genome: Arc<RwLock<Option<feagi_evolutionary::RuntimeGenome>>>,
    /// Optional reference to RuntimeService for accessing NPU (for connectome I/O)
    #[cfg(feature = "connectome-io")]
    runtime_service: Arc<RwLock<Option<Arc<dyn crate::traits::RuntimeService + Send + Sync>>>>,
    /// Optional burst runner for refreshing cortical_id cache
    burst_runner: Option<Arc<RwLock<BurstLoopRunner>>>,
}

impl ConnectomeServiceImpl {
    pub fn new(
        connectome: Arc<RwLock<ConnectomeManager>>,
        current_genome: Arc<RwLock<Option<feagi_evolutionary::RuntimeGenome>>>,
    ) -> Self {
        Self {
            connectome,
            current_genome,
            #[cfg(feature = "connectome-io")]
            runtime_service: Arc::new(RwLock::new(None)),
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
            debug!(target: "feagi-services", "Refreshed burst runner cache with {} cortical areas", mapping_count);
        }
    }

    /// Set the runtime service (required for connectome export/import)
    ///
    /// This must be called after creating ConnectomeServiceImpl to enable
    /// connectome I/O operations.
    #[cfg(feature = "connectome-io")]
    pub fn set_runtime_service(
        &self,
        runtime_service: Arc<dyn crate::traits::RuntimeService + Send + Sync>,
    ) {
        *self.runtime_service.write() = Some(runtime_service);
        info!(target: "feagi-services", "RuntimeService connected to ConnectomeService for connectome I/O");
    }

    /// Convert RegionType enum to string
    fn region_type_to_string(region_type: &RegionType) -> String {
        match region_type {
            RegionType::Undefined => "Undefined".to_string(),
        }
    }

    /// Convert string to RegionType enum
    fn string_to_region_type(s: &str) -> Result<RegionType, ServiceError> {
        match s {
            "Undefined" | "Sensory" | "Motor" | "Memory" | "Custom" => Ok(RegionType::Undefined),
            _ => Err(ServiceError::InvalidInput(format!(
                "Invalid region type: {}",
                s
            ))),
        }
    }
}

#[async_trait]
impl ConnectomeService for ConnectomeServiceImpl {
    // ========================================================================
    // CORTICAL AREA OPERATIONS
    // ========================================================================

    async fn create_cortical_area(
        &self,
        params: CreateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!(target: "feagi-services","Creating cortical area: {}", params.cortical_id);

        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(&params.cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;

        // Get cortical area type from the cortical ID
        let area_type = cortical_id_typed.as_cortical_type().map_err(|e| {
            ServiceError::InvalidInput(format!("Failed to determine cortical area type: {}", e))
        })?;

        // Create CorticalArea
        let mut area = CorticalArea::new(
            cortical_id_typed,
            0, // Auto-assigned by ConnectomeManager
            params.name.clone(),
            CorticalAreaDimensions::new(
                params.dimensions.0 as u32,
                params.dimensions.1 as u32,
                params.dimensions.2 as u32,
            )?,
            params.position.into(), // Convert (i32, i32, i32) to GenomeCoordinate3D
            area_type,
        )?;

        // Set the cortical type
        // Note: cortical_type_new field removed - type is encoded in CorticalID

        // Apply all neural parameters from params
        if let Some(visible) = params.visible {
            area.add_property_mut("visible".to_string(), serde_json::json!(visible));
        }
        if let Some(sub_group) = params.sub_group {
            area.add_property_mut("sub_group".to_string(), serde_json::json!(sub_group));
        }
        if let Some(neurons_per_voxel) = params.neurons_per_voxel {
            area.add_property_mut(
                "neurons_per_voxel".to_string(),
                serde_json::json!(neurons_per_voxel),
            );
        }
        if let Some(postsynaptic_current) = params.postsynaptic_current {
            area.add_property_mut(
                "postsynaptic_current".to_string(),
                serde_json::json!(postsynaptic_current),
            );
        }
        if let Some(plasticity_constant) = params.plasticity_constant {
            area.add_property_mut(
                "plasticity_constant".to_string(),
                serde_json::json!(plasticity_constant),
            );
        }
        if let Some(degeneration) = params.degeneration {
            area.add_property_mut("degeneration".to_string(), serde_json::json!(degeneration));
        }
        if let Some(psp_uniform_distribution) = params.psp_uniform_distribution {
            area.add_property_mut(
                "psp_uniform_distribution".to_string(),
                serde_json::json!(psp_uniform_distribution),
            );
        }
        if let Some(firing_threshold_increment) = params.firing_threshold_increment {
            area.add_property_mut(
                "firing_threshold_increment".to_string(),
                serde_json::json!(firing_threshold_increment),
            );
        }
        if let Some(firing_threshold_limit) = params.firing_threshold_limit {
            area.add_property_mut(
                "firing_threshold_limit".to_string(),
                serde_json::json!(firing_threshold_limit),
            );
        }
        if let Some(consecutive_fire_count) = params.consecutive_fire_count {
            area.add_property_mut(
                "consecutive_fire_limit".to_string(),
                serde_json::json!(consecutive_fire_count),
            );
        }
        if let Some(snooze_period) = params.snooze_period {
            area.add_property_mut(
                "snooze_period".to_string(),
                serde_json::json!(snooze_period),
            );
        }
        if let Some(refractory_period) = params.refractory_period {
            area.add_property_mut(
                "refractory_period".to_string(),
                serde_json::json!(refractory_period),
            );
        }
        if let Some(leak_coefficient) = params.leak_coefficient {
            area.add_property_mut(
                "leak_coefficient".to_string(),
                serde_json::json!(leak_coefficient),
            );
        }
        if let Some(leak_variability) = params.leak_variability {
            area.add_property_mut(
                "leak_variability".to_string(),
                serde_json::json!(leak_variability),
            );
        }
        if let Some(burst_engine_active) = params.burst_engine_active {
            area.add_property_mut(
                "burst_engine_active".to_string(),
                serde_json::json!(burst_engine_active),
            );
        }

        // Extract parent_region_id before moving properties
        let parent_region_id = params
            .properties
            .as_ref()
            .and_then(|props| props.get("parent_region_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(properties) = params.properties {
            area.properties = properties;
        }

        // Add to connectome
        self.connectome
            .write()
            .add_cortical_area(area)
            .map_err(ServiceError::from)?;

        // Refresh burst runner cache after creating area
        self.refresh_burst_runner_cache();

        // CRITICAL: If parent_region_id is specified, add this cortical area
        // to the parent brain region's cortical_areas set so it persists in genome
        if let Some(region_id) = parent_region_id {
            let mut manager = self.connectome.write();
            if let Some(region) = manager.get_brain_region_mut(&region_id) {
                region.add_area(cortical_id_typed);
                info!(target: "feagi-services",
                    "Added cortical area {} to parent region {}",
                    params.cortical_id, region_id
                );
            } else {
                warn!(target: "feagi-services",
                    "Parent region {} not found for cortical area {}",
                    region_id, params.cortical_id
                );
            }
        }

        // Return info
        self.get_cortical_area(&params.cortical_id).await
    }

    async fn delete_cortical_area(&self, cortical_id: &str) -> ServiceResult<()> {
        info!(target: "feagi-services","Deleting cortical area: {}", cortical_id);

        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;

        // Remove from the live connectome, and also scrub from brain-region membership
        // so UI + region-based operations don't keep referencing a deleted area.
        //
        // Note: ConnectomeManager::remove_cortical_area currently does NOT remove the
        // ID from brain regions, so we do it explicitly here.
        {
            let mut manager = self.connectome.write();
            let region_ids: Vec<String> = manager
                .get_brain_region_ids()
                .into_iter()
                .cloned()
                .collect();
            for region_id in region_ids {
                if let Some(region) = manager.get_brain_region_mut(&region_id) {
                    region.remove_area(&cortical_id_typed);
                }
            }

            manager
                .remove_cortical_area(&cortical_id_typed)
                .map_err(ServiceError::from)?;
        }

        // CRITICAL: Persist deletion into RuntimeGenome (source of truth for save/export).
        if let Some(genome) = self.current_genome.write().as_mut() {
            let removed = genome.cortical_areas.remove(&cortical_id_typed).is_some();
            for region in genome.brain_regions.values_mut() {
                region.remove_area(&cortical_id_typed);
            }

            if removed {
                info!(
                    target: "feagi-services",
                    "[GENOME-UPDATE] Removed cortical area {} from RuntimeGenome",
                    cortical_id
                );
            } else {
                warn!(
                    target: "feagi-services",
                    "[GENOME-UPDATE] Cortical area {} not found in RuntimeGenome - deletion will not persist to saved genome",
                    cortical_id
                );
            }
        } else {
            warn!(
                target: "feagi-services",
                "[GENOME-UPDATE] No RuntimeGenome loaded - deletion will not persist to saved genome"
            );
        }

        // Refresh burst runner cache after deleting area
        self.refresh_burst_runner_cache();

        Ok(())
    }

    async fn update_cortical_area(
        &self,
        cortical_id: &str,
        _params: UpdateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo> {
        info!(target: "feagi-services","Updating cortical area: {}", cortical_id);

        // TODO: This should be routed through GenomeService for proper genome update
        // and change classification (PARAMETER vs STRUCTURAL vs METADATA)
        // Currently this is a stub - needs architecture alignment with Python implementation

        Err(ServiceError::NotImplemented(
            "Cortical area updates must go through GenomeService for proper genome synchronization"
                .to_string(),
        ))
    }

    async fn get_cortical_area(&self, cortical_id: &str) -> ServiceResult<CorticalAreaInfo> {
        trace!(target: "feagi-services", "Getting cortical area: {}", cortical_id);

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

        let neuron_count = manager.get_neuron_count_in_area(
            &CorticalID::try_from_base_64(cortical_id)
                .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?,
        );
        let synapse_count = manager.get_synapse_count_in_area(
            &CorticalID::try_from_base_64(cortical_id)
                .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?,
        );

        // Get cortical_group from the area (uses cortical_type_new if available)
        let cortical_group = area.get_cortical_group();

        // Note: decode_cortical_id removed - IPU/OPU metadata now in CorticalID
        let memory_props = {
            use feagi_evolutionary::extract_memory_properties;
            extract_memory_properties(&area.properties)
        };

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
        let unit_id = if is_io_area { Some(cortical_bytes[6]) } else { None };
        let group_id = if is_io_area { Some(cortical_bytes[7]) } else { None };
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
        if is_io_area {
            if let Some(opts) = &coding_options {
                info!(
                    target: "feagi-services",
                    "[IO-CODING] {} options signage={:?} behavior={:?} type={:?} io_flag={:?}",
                    cortical_id,
                    opts.signage_options,
                    opts.behavior_options,
                    opts.coding_type_options,
                    io_flag
                );
            } else {
                warn!(
                    target: "feagi-services",
                    "[IO-CODING] {} options missing (io_flag={:?})",
                    cortical_id,
                    io_flag
                );
            }
        }

        let name = if area.name.is_empty() || area.name == area.cortical_id.to_string() {
            derive_friendly_cortical_name(&area.cortical_id).unwrap_or_else(|| area.name.clone())
        } else {
            area.name.clone()
        };

        Ok(CorticalAreaInfo {
            cortical_id: cortical_id.to_string(),
            cortical_id_s: area.cortical_id.to_string(), // Human-readable ASCII string
            cortical_idx,
            name,
            dimensions: (
                area.dimensions.width as usize,
                area.dimensions.height as usize,
                area.dimensions.depth as usize,
            ),
            position: area.position.into(), // Convert GenomeCoordinate3D to (i32, i32, i32)
            area_type: cortical_group
                .clone()
                .unwrap_or_else(|| "CUSTOM".to_string()),
            cortical_group: cortical_group
                .clone()
                .unwrap_or_else(|| "CUSTOM".to_string()),
            // Determine cortical_type based on properties
            cortical_type: {
                if memory_props.is_some() {
                    "memory".to_string()
                } else if let Some(group) = &cortical_group {
                    match group.as_str() {
                        "IPU" => "sensory".to_string(),
                        "OPU" => "motor".to_string(),
                        "CORE" => "core".to_string(),
                        "MEMORY" => "memory".to_string(),
                        _ => "custom".to_string(),
                    }
                } else {
                    "custom".to_string()
                }
            },
            neuron_count,
            synapse_count,
            // All neural parameters come from the actual CorticalArea struct
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
            temporal_depth: memory_props.map(|p| p.temporal_depth.max(1)),
            properties: area.properties.clone(),
            // IPU/OPU-specific decoded fields (only populated for IPU/OPU areas)
            cortical_subtype,
            encoding_type: coding_behavior.clone(),
            encoding_format: coding_type.clone(),
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
            cortical_dimensions_per_device: {
                // Try to get from properties first
                let from_properties = area
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
                    });

                // If not in properties, compute from dimensions and dev_count for IPU/OPU areas
                if from_properties.is_none() {
                    if let Some(dev_count) = area
                        .properties
                        .get("dev_count")
                        .and_then(|v| v.as_u64().map(|n| n as usize))
                    {
                        if dev_count > 0 {
                            let total_width = area.dimensions.width as usize;
                            let height = area.dimensions.height as usize;
                            let depth = area.dimensions.depth as usize;
                            Some((total_width / dev_count, height, depth))
                        } else {
                            from_properties
                        }
                    } else {
                        from_properties
                    }
                } else {
                    from_properties
                }
            },
            visualization_voxel_granularity: {
                // Default is 1x1x1 if not in properties (user-driven, not stored)
                // Handle both integer and float JSON values
                area.properties
                    .get("visualization_voxel_granularity")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
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
                                Some((x as u32, y as u32, z as u32))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .or(Some((1, 1, 1))) // Default is 1x1x1
            },
        })
    }

    async fn list_cortical_areas(&self) -> ServiceResult<Vec<CorticalAreaInfo>> {
        trace!(target: "feagi-services", "Listing all cortical areas");

        let cortical_ids: Vec<String> = {
            let manager = self.connectome.read();
            manager
                .get_cortical_area_ids()
                .into_iter()
                .map(|id| id.as_base_64())
                .collect()
        };

        let mut areas = Vec::new();
        for cortical_id in cortical_ids {
            if let Ok(area_info) = self.get_cortical_area(&cortical_id).await {
                areas.push(area_info);
            }
        }

        Ok(areas)
    }

    async fn get_cortical_area_ids(&self) -> ServiceResult<Vec<String>> {
        debug!(target: "feagi-services","Getting cortical area IDs");

        // CRITICAL: Use try_read() instead of read() to avoid blocking forever
        // If write lock is held (e.g., during genome loading), return error instead of hanging
        let ids: Vec<String> = {
            let manager = match self.connectome.try_read() {
                Some(guard) => guard,
                None => {
                    warn!(target: "feagi-services", "⚠️ ConnectomeManager write lock is held - cannot read cortical area IDs");
                    return Err(ServiceError::Backend("ConnectomeManager is currently being modified (e.g., genome loading in progress). Please try again in a moment.".to_string()));
                }
            };

            let area_count = manager.get_cortical_area_count();
            let ids_refs = manager.get_cortical_area_ids();
            info!(target: "feagi-services", "Found {} cortical areas in ConnectomeManager", area_count);
            info!(target: "feagi-services", "Cortical area IDs (references): {:?}", ids_refs.iter().take(10).collect::<Vec<_>>());
            ids_refs.into_iter().map(|id| id.as_base_64()).collect()
        }; // Lock dropped here
        info!(target: "feagi-services", "Returning {} cortical area IDs: {:?}", ids.len(), ids.iter().take(10).collect::<Vec<_>>());
        Ok(ids)
    }

    async fn cortical_area_exists(&self, cortical_id: &str) -> ServiceResult<bool> {
        debug!(target: "feagi-services","Checking if cortical area exists: {}", cortical_id);

        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;

        Ok(self.connectome.read().has_cortical_area(&cortical_id_typed))
    }

    async fn get_cortical_area_properties(
        &self,
        cortical_id: &str,
    ) -> ServiceResult<std::collections::HashMap<String, serde_json::Value>> {
        debug!(target: "feagi-services","Getting cortical area properties: {}", cortical_id);

        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;

        let manager = self.connectome.read();
        manager
            .get_cortical_area_properties(&cortical_id_typed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            })
    }

    async fn get_all_cortical_area_properties(
        &self,
    ) -> ServiceResult<Vec<std::collections::HashMap<String, serde_json::Value>>> {
        debug!(target: "feagi-services","Getting all cortical area properties");

        let manager = self.connectome.read();
        Ok(manager.get_all_cortical_area_properties())
    }

    async fn get_neuron_properties(
        &self,
        neuron_id: u64,
    ) -> ServiceResult<HashMap<String, serde_json::Value>> {
        let manager = self.connectome.read();
        manager
            .get_neuron_properties(neuron_id)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "Neuron".to_string(),
                id: neuron_id.to_string(),
            })
    }

    // ========================================================================
    // BRAIN REGION OPERATIONS
    // ========================================================================

    async fn create_brain_region(
        &self,
        params: CreateBrainRegionParams,
    ) -> ServiceResult<BrainRegionInfo> {
        info!(target: "feagi-services","Creating brain region: {}", params.region_id);

        // Convert string to RegionType
        let region_type = Self::string_to_region_type(&params.region_type)?;

        // Create BrainRegion
        let mut region = BrainRegion::new(
            RegionID::from_string(&params.region_id)
                .map_err(|e| ServiceError::InvalidInput(format!("Invalid region ID: {}", e)))?,
            params.name.clone(),
            region_type,
        )
        .map_err(ServiceError::from)?;

        // Apply initial properties (persisted into ConnectomeManager and RuntimeGenome).
        if let Some(props) = params.properties.clone() {
            region = region.with_properties(props);
        }

        // Add to connectome
        self.connectome
            .write()
            .add_brain_region(region, params.parent_id.clone())
            .map_err(ServiceError::from)?;

        // Persist into RuntimeGenome (source of truth for genome save/export).
        //
        // NOTE: GenomeServiceImpl::create_cortical_areas requires that parent_region_id exists
        // in the RuntimeGenome brain_regions map. Without this, any subsequent cortical-area
        // creation that targets this region will fail.
        if let Some(genome) = self.current_genome.write().as_mut() {
            // Fetch the canonical region instance from ConnectomeManager to ensure any internal
            // normalization is reflected in the persisted copy.
            if let Some(created) = self
                .connectome
                .read()
                .get_brain_region(&params.region_id)
                .cloned()
            {
                genome
                    .brain_regions
                    .insert(params.region_id.clone(), created);
            }
        }

        // Return info
        self.get_brain_region(&params.region_id).await
    }

    async fn delete_brain_region(&self, region_id: &str) -> ServiceResult<()> {
        info!(target: "feagi-services","Deleting brain region: {}", region_id);

        self.connectome
            .write()
            .remove_brain_region(region_id)
            .map_err(ServiceError::from)?;

        Ok(())
    }

    async fn update_brain_region(
        &self,
        region_id: &str,
        properties: std::collections::HashMap<String, serde_json::Value>,
    ) -> ServiceResult<BrainRegionInfo> {
        info!(target: "feagi-services", "Updating brain region: {}", region_id);

        self.connectome
            .write()
            .update_brain_region_properties(region_id, properties)
            .map_err(ServiceError::from)?;

        // Return updated info
        self.get_brain_region(region_id).await
    }

    async fn get_brain_region(&self, region_id: &str) -> ServiceResult<BrainRegionInfo> {
        trace!(target: "feagi-services", "Getting brain region: {}", region_id);

        let manager = self.connectome.read();

        let region = manager
            .get_brain_region(region_id)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "BrainRegion".to_string(),
                id: region_id.to_string(),
            })?;

        let hierarchy = manager.get_brain_region_hierarchy();
        let parent_id = hierarchy.get_parent(region_id).map(|s| s.to_string());
        let child_regions: Vec<String> = hierarchy
            .get_children(region_id)
            .into_iter()
            .cloned()
            .collect();

        Ok(BrainRegionInfo {
            region_id: region_id.to_string(),
            name: region.name.clone(),
            region_type: Self::region_type_to_string(&region.region_type),
            parent_id,
            cortical_areas: region
                .cortical_areas
                .iter()
                .map(|id| id.as_base_64())
                .collect(), // Use base64 to match cortical area API
            child_regions,
            properties: region.properties.clone(),
        })
    }

    async fn list_brain_regions(&self) -> ServiceResult<Vec<BrainRegionInfo>> {
        trace!(target: "feagi-services", "Listing all brain regions");

        let region_ids: Vec<String> = {
            let manager = self.connectome.read();
            let ids = manager.get_brain_region_ids();
            trace!(target: "feagi-services", "Found {} brain region IDs from ConnectomeManager", ids.len());
            ids.into_iter().map(|s| s.to_string()).collect()
        };

        trace!(target: "feagi-services", "Processing {} regions...", region_ids.len());
        let mut regions = Vec::new();
        for region_id in region_ids {
            trace!(target: "feagi-services", "Getting region: {}", region_id);
            match self.get_brain_region(&region_id).await {
                Ok(region_info) => {
                    trace!(
                        target: "feagi-services",
                        "Got region: {} with {} areas",
                        region_info.name,
                        region_info.cortical_areas.len()
                    );
                    regions.push(region_info);
                }
                Err(e) => {
                    warn!(target: "feagi-services", "Failed to get region {}: {}", region_id, e);
                }
            }
        }

        trace!(target: "feagi-services", "Returning {} brain regions", regions.len());
        Ok(regions)
    }

    async fn get_brain_region_ids(&self) -> ServiceResult<Vec<String>> {
        debug!(target: "feagi-services","Getting brain region IDs");
        Ok(self
            .connectome
            .read()
            .get_brain_region_ids()
            .into_iter()
            .map(|s| s.to_string())
            .collect())
    }

    async fn brain_region_exists(&self, region_id: &str) -> ServiceResult<bool> {
        debug!(target: "feagi-services","Checking if brain region exists: {}", region_id);
        Ok(self.connectome.read().get_brain_region(region_id).is_some())
    }

    async fn get_morphologies(&self) -> ServiceResult<HashMap<String, MorphologyInfo>> {
        let manager = self.connectome.read();
        let registry = manager.get_morphologies();

        let mut result = HashMap::new();
        for (id, morphology) in registry.iter() {
            result.insert(
                id.clone(),
                MorphologyInfo {
                    morphology_type: format!("{:?}", morphology.morphology_type).to_lowercase(),
                    class: morphology.class.clone(),
                    parameters: serde_json::to_value(&morphology.parameters)
                        .unwrap_or(serde_json::json!({})),
                },
            );
        }

        trace!(target: "feagi-services", "Retrieved {} morphologies", result.len());
        Ok(result)
    }

    async fn create_morphology(
        &self,
        morphology_id: String,
        morphology: feagi_evolutionary::Morphology,
    ) -> ServiceResult<()> {
        if morphology_id.trim().is_empty() {
            return Err(ServiceError::InvalidInput(
                "morphology_id must be non-empty".to_string(),
            ));
        }

        // Require a loaded RuntimeGenome for persistence (source of truth).
        let mut genome_guard = self.current_genome.write();
        let Some(genome) = genome_guard.as_mut() else {
            return Err(ServiceError::InvalidState(
                "No RuntimeGenome loaded - cannot create morphology".to_string(),
            ));
        };

        if genome.morphologies.contains(&morphology_id) {
            return Err(ServiceError::AlreadyExists {
                resource: "morphology".to_string(),
                id: morphology_id,
            });
        }

        genome
            .morphologies
            .add_morphology(morphology_id.clone(), morphology.clone());

        // Keep ConnectomeManager registry in sync (used by mapping/synapse generation).
        self.connectome
            .write()
            .upsert_morphology(morphology_id, morphology);

        Ok(())
    }

    async fn update_morphology(
        &self,
        morphology_id: String,
        morphology: feagi_evolutionary::Morphology,
    ) -> ServiceResult<()> {
        if morphology_id.trim().is_empty() {
            return Err(ServiceError::InvalidInput(
                "morphology_id must be non-empty".to_string(),
            ));
        }

        let mut genome_guard = self.current_genome.write();
        let Some(genome) = genome_guard.as_mut() else {
            return Err(ServiceError::InvalidState(
                "No RuntimeGenome loaded - cannot update morphology".to_string(),
            ));
        };

        if !genome.morphologies.contains(&morphology_id) {
            return Err(ServiceError::NotFound {
                resource: "morphology".to_string(),
                id: morphology_id,
            });
        }

        genome
            .morphologies
            .add_morphology(morphology_id.clone(), morphology.clone());

        self.connectome
            .write()
            .upsert_morphology(morphology_id, morphology);

        Ok(())
    }

    async fn delete_morphology(&self, morphology_id: &str) -> ServiceResult<()> {
        if morphology_id.trim().is_empty() {
            return Err(ServiceError::InvalidInput(
                "morphology_id must be non-empty".to_string(),
            ));
        }

        let mut genome_guard = self.current_genome.write();
        let Some(genome) = genome_guard.as_mut() else {
            return Err(ServiceError::InvalidState(
                "No RuntimeGenome loaded - cannot delete morphology".to_string(),
            ));
        };

        if !genome.morphologies.remove_morphology(morphology_id) {
            return Err(ServiceError::NotFound {
                resource: "morphology".to_string(),
                id: morphology_id.to_string(),
            });
        }

        // Mirror deletion into the ConnectomeManager registry.
        self.connectome.write().remove_morphology(morphology_id);

        Ok(())
    }

    async fn update_cortical_mapping(
        &self,
        src_area_id: String,
        dst_area_id: String,
        mapping_data: Vec<serde_json::Value>,
    ) -> ServiceResult<usize> {
        info!(target: "feagi-services", "Updating cortical mapping: {} -> {} with {} connections",
              src_area_id, dst_area_id, mapping_data.len());

        // Convert String to CorticalID
        use feagi_structures::genomic::cortical_area::CorticalID;
        let src_id = CorticalID::try_from_base_64(&src_area_id).map_err(|e| {
            ServiceError::InvalidInput(format!("Invalid source cortical ID: {}", e))
        })?;
        let dst_id = CorticalID::try_from_base_64(&dst_area_id).map_err(|e| {
            ServiceError::InvalidInput(format!("Invalid destination cortical ID: {}", e))
        })?;

        // Update RuntimeGenome if available (CRITICAL for save/load persistence!)
        if let Some(genome) = self.current_genome.write().as_mut() {
            if let Some(src_area) = genome.cortical_areas.get_mut(&src_id) {
                update_cortical_mapping_dst_in_properties(
                    &mut src_area.properties,
                    &dst_area_id,
                    &mapping_data,
                )?;
                info!(
                    target: "feagi-services",
                    "[GENOME-UPDATE] Updated cortical_mapping_dst for {} -> {} (connections={})",
                    src_area_id,
                    dst_area_id,
                    mapping_data.len()
                );
            } else {
                warn!(target: "feagi-services", "[GENOME-UPDATE] Source area {} not found in RuntimeGenome", src_area_id);
            }
        } else {
            warn!(target: "feagi-services", "[GENOME-UPDATE] No RuntimeGenome loaded - mapping will not persist");
        }

        // Update the cortical_mapping_dst property in ConnectomeManager
        let region_io = {
            let mut manager = self.connectome.write();
            manager
                .update_cortical_mapping(&src_id, &dst_id, mapping_data.clone())
                .map_err(|e| ServiceError::Backend(format!("Failed to update mapping: {}", e)))?;

            // Regenerate synapses for this mapping
            let synapse_count = manager
                .regenerate_synapses_for_mapping(&src_id, &dst_id)
                .map_err(|e| {
                    ServiceError::Backend(format!("Failed to regenerate synapses: {}", e))
                })?;

            // Recompute region IO registries after mapping change (critical for BV region boundary behavior)
            let region_io = manager.recompute_brain_region_io_registry().map_err(|e| {
                ServiceError::Backend(format!("Failed to recompute region IO registry: {}", e))
            })?;

            info!(
                target: "feagi-services",
                "Cortical mapping updated: {} synapses created",
                synapse_count
            );

            (synapse_count, region_io)
        };

        // Persist updated region IO into RuntimeGenome so genome save/export stays consistent.
        if let Some(genome) = self.current_genome.write().as_mut() {
            for (region_id, (inputs, outputs)) in region_io.1 {
                if let Some(region) = genome.brain_regions.get_mut(&region_id) {
                    if inputs.is_empty() {
                        region.properties.remove("inputs");
                    } else {
                        region
                            .properties
                            .insert("inputs".to_string(), serde_json::json!(inputs));
                    }

                    if outputs.is_empty() {
                        region.properties.remove("outputs");
                    } else {
                        region
                            .properties
                            .insert("outputs".to_string(), serde_json::json!(outputs));
                    }
                } else {
                    warn!(
                        target: "feagi-services",
                        "Region '{}' not found in RuntimeGenome while persisting IO registry",
                        region_id
                    );
                }
            }
        }

        Ok(region_io.0)
    }

    // Note: unit tests for mapping persistence behavior are below in this module.

    // ========================================================================
    // CONNECTOME I/O OPERATIONS
    // ========================================================================

    #[cfg(feature = "connectome-io")]
    async fn export_connectome(
        &self,
    ) -> ServiceResult<feagi_npu_neural::types::connectome::ConnectomeSnapshot> {
        info!(target: "feagi-services", "Exporting connectome via service layer");

        // Get NPU from ConnectomeManager (which has reference to NPU)
        // Note: get_npu() returns Option<&Arc<...>>, so we need to clone the Arc
        // to use it outside the lock scope
        let npu_arc = {
            let connectome = self.connectome.read();
            let npu_opt = connectome.get_npu();
            npu_opt
                .ok_or_else(|| {
                    ServiceError::Backend("NPU not connected to ConnectomeManager".to_string())
                })?
                .clone()
        };

        // Export connectome from NPU
        // Note: export_connectome() is on RustNPU, but we have DynamicNPU
        // We need to handle both F32 and INT8 variants
        use tracing::debug;
        let lock_start = std::time::Instant::now();
        let thread_id = std::thread::current().id();
        debug!(
            "[NPU-LOCK] CONNECTOME-SERVICE: Thread {:?} attempting NPU lock for export_connectome at {:?}",
            thread_id, lock_start
        );
        let snapshot = {
            let npu_lock = npu_arc.lock().unwrap();
            let lock_acquired = std::time::Instant::now();
            let lock_wait = lock_acquired.duration_since(lock_start);
            debug!(
                "[NPU-LOCK] CONNECTOME-SERVICE: Thread {:?} acquired lock after {:.2}ms wait for export_connectome",
                thread_id,
                lock_wait.as_secs_f64() * 1000.0
            );
            match &*npu_lock {
                feagi_npu_burst_engine::DynamicNPU::F32(npu_f32) => npu_f32.export_connectome(),
                feagi_npu_burst_engine::DynamicNPU::INT8(npu_int8) => npu_int8.export_connectome(),
            }
        };
        let lock_released = std::time::Instant::now();
        let total_duration = lock_released.duration_since(lock_start);
        debug!(
            "[NPU-LOCK] CONNECTOME-SERVICE: Thread {:?} RELEASED NPU lock after export_connectome (total: {:.2}ms)",
            thread_id,
            total_duration.as_secs_f64() * 1000.0
        );

        info!(target: "feagi-services", "✅ Connectome exported: {} neurons, {} synapses",
            snapshot.neurons.count, snapshot.synapses.count);

        Ok(snapshot)
    }

    #[cfg(feature = "connectome-io")]
    async fn import_connectome(
        &self,
        snapshot: feagi_npu_neural::types::connectome::ConnectomeSnapshot,
    ) -> ServiceResult<()> {
        info!(target: "feagi-services", "Importing connectome via service layer: {} neurons, {} synapses",
            snapshot.neurons.count, snapshot.synapses.count);

        // NOTE: NPU.import_connectome_with_config() is a constructor that creates a NEW NPU.
        // This means importing requires replacing the entire NPU instance, which involves:
        // 1. Stopping the burst engine
        // 2. Creating a new NPU from the snapshot
        // 3. Replacing the NPU in ConnectomeManager and BurstLoopRunner
        // 4. Restarting the burst engine
        //
        // This is a complex operation that requires coordination across multiple components.
        // For now, we return NotImplemented and recommend using the NPU constructor directly
        // during application initialization, or implementing a higher-level "replace NPU" operation.

        warn!(target: "feagi-services", "⚠️ Connectome import via service layer not yet fully implemented");
        warn!(target: "feagi-services", "   NPU.import_connectome_with_config() creates a new NPU instance");
        warn!(target: "feagi-services", "   This requires stopping burst engine, replacing NPU, and restarting");
        warn!(target: "feagi-services", "   Recommendation: Use NPU.import_connectome_with_config() during initialization");

        Err(ServiceError::NotImplemented(
            "Connectome import via service layer requires NPU replacement coordination. Use NPU.import_connectome_with_config() during application initialization, or implement a 'replace NPU' operation that coordinates with BurstLoopRunner.".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::update_cortical_mapping_dst_in_properties;
    use crate::types::ServiceResult;
    use std::collections::HashMap;

    #[test]
    fn empty_mapping_deletes_destination_key_and_prunes_container() -> ServiceResult<()> {
        let mut props: HashMap<String, serde_json::Value> = HashMap::new();
        props.insert(
            "cortical_mapping_dst".to_string(),
            serde_json::json!({
                "dstA": [{"morphology_id": "m1"}],
                "dstB": []
            }),
        );

        update_cortical_mapping_dst_in_properties(&mut props, "dstA", &[])?;
        let dst = props
            .get("cortical_mapping_dst")
            .and_then(|v| v.as_object())
            .expect("cortical_mapping_dst should remain with dstB");
        assert!(!dst.contains_key("dstA"));
        assert!(dst.contains_key("dstB"));

        // Now remove last remaining destination, container should be removed entirely
        update_cortical_mapping_dst_in_properties(&mut props, "dstB", &[])?;
        assert!(!props.contains_key("cortical_mapping_dst"));
        Ok(())
    }

    #[test]
    fn non_empty_mapping_sets_destination_key() -> ServiceResult<()> {
        let mut props: HashMap<String, serde_json::Value> = HashMap::new();
        update_cortical_mapping_dst_in_properties(
            &mut props,
            "dstX",
            &[serde_json::json!({"morphology_id": "m1"})],
        )?;

        let dst = props
            .get("cortical_mapping_dst")
            .and_then(|v| v.as_object())
            .expect("cortical_mapping_dst should exist");
        let arr = dst
            .get("dstX")
            .and_then(|v| v.as_array())
            .expect("dstX should be an array");
        assert_eq!(arr.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn morphology_create_update_delete_roundtrip() -> ServiceResult<()> {
        use super::ConnectomeServiceImpl;
        use crate::traits::ConnectomeService;
        use parking_lot::RwLock;
        use std::sync::Arc;

        // Isolated connectome manager instance for this test.
        let connectome = Arc::new(RwLock::new(
            feagi_brain_development::ConnectomeManager::new_for_testing(),
        ));

        // Minimal RuntimeGenome (source of truth) for persistence.
        let genome = feagi_evolutionary::RuntimeGenome {
            metadata: feagi_evolutionary::GenomeMetadata {
                genome_id: "test".to_string(),
                genome_title: "test".to_string(),
                genome_description: "".to_string(),
                version: "2.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: None,
            },
            cortical_areas: HashMap::new(),
            brain_regions: HashMap::new(),
            morphologies: feagi_evolutionary::MorphologyRegistry::new(),
            physiology: feagi_evolutionary::PhysiologyConfig::default(),
            signatures: feagi_evolutionary::GenomeSignatures {
                genome: "0".to_string(),
                blueprint: "0".to_string(),
                physiology: "0".to_string(),
                morphologies: None,
            },
            stats: feagi_evolutionary::GenomeStats::default(),
        };
        let current_genome = Arc::new(RwLock::new(Some(genome)));

        let svc = ConnectomeServiceImpl::new(connectome.clone(), current_genome.clone());

        // Create
        let morph_id = "m_test_vectors".to_string();
        let morph = feagi_evolutionary::Morphology {
            morphology_type: feagi_evolutionary::MorphologyType::Vectors,
            parameters: feagi_evolutionary::MorphologyParameters::Vectors {
                vectors: vec![[1, 2, 3]],
            },
            class: "custom".to_string(),
        };
        svc.create_morphology(morph_id.clone(), morph).await?;

        // Verify both source-of-truth and connectome registry were updated
        {
            let genome_guard = current_genome.read();
            let genome = genome_guard.as_ref().expect("genome must exist");
            assert!(genome.morphologies.contains(&morph_id));
        }
        {
            let mgr = connectome.read();
            assert!(mgr.get_morphologies().contains(&morph_id));
        }

        // Update (overwrite vectors)
        let morph2 = feagi_evolutionary::Morphology {
            morphology_type: feagi_evolutionary::MorphologyType::Vectors,
            parameters: feagi_evolutionary::MorphologyParameters::Vectors {
                vectors: vec![[9, 9, 9]],
            },
            class: "custom".to_string(),
        };
        svc.update_morphology(morph_id.clone(), morph2).await?;
        {
            let mgr = connectome.read();
            let stored = mgr
                .get_morphologies()
                .get(&morph_id)
                .expect("morphology must exist");
            match &stored.parameters {
                feagi_evolutionary::MorphologyParameters::Vectors { vectors } => {
                    assert_eq!(vectors.as_slice(), &[[9, 9, 9]]);
                }
                other => panic!("unexpected parameters: {:?}", other),
            }
        }

        // Delete
        svc.delete_morphology(&morph_id).await?;
        {
            let genome_guard = current_genome.read();
            let genome = genome_guard.as_ref().expect("genome must exist");
            assert!(!genome.morphologies.contains(&morph_id));
        }
        {
            let mgr = connectome.read();
            assert!(!mgr.get_morphologies().contains(&morph_id));
        }

        Ok(())
    }

    #[tokio::test]
    async fn delete_cortical_area_persists_to_runtime_genome() -> ServiceResult<()> {
        use super::ConnectomeServiceImpl;
        use crate::traits::ConnectomeService;
        use feagi_structures::genomic::brain_regions::{BrainRegion, RegionID, RegionType};
        use feagi_structures::genomic::cortical_area::{
            CoreCorticalType, CorticalArea, CorticalAreaDimensions,
        };
        use feagi_structures::genomic::descriptors::GenomeCoordinate3D;
        use parking_lot::RwLock;
        use std::collections::HashMap;
        use std::sync::Arc;

        // Isolated connectome manager instance for this test.
        let connectome = Arc::new(RwLock::new(
            feagi_brain_development::ConnectomeManager::new_for_testing(),
        ));

        // Use a known-valid cortical ID/type pair to avoid ID encoding intricacies in this unit test.
        let cortical_id = CoreCorticalType::Power.to_cortical_id();

        let dims = CorticalAreaDimensions::new(1, 1, 1).expect("dimensions must be valid");
        let pos = GenomeCoordinate3D::new(0, 0, 0);
        let cortical_type = cortical_id
            .as_cortical_type()
            .expect("cortical type must be derivable from id");

        let area = CorticalArea::new(
            cortical_id,
            0, // Let ConnectomeManager assign a proper idx
            "test_area".to_string(),
            dims,
            pos,
            cortical_type,
        )
        .expect("area must be valid");

        // Create a region that contains the test area.
        let region_id = RegionID::new();
        let region_key = region_id.to_string();
        let region = BrainRegion::new(region_id, "root".to_string(), RegionType::Undefined)
            .expect("region must be valid")
            .with_areas([cortical_id]);

        // Seed RuntimeGenome with the area + region membership (this is what genome save/export uses).
        let genome = feagi_evolutionary::RuntimeGenome {
            metadata: feagi_evolutionary::GenomeMetadata {
                genome_id: "test".to_string(),
                genome_title: "test".to_string(),
                genome_description: "".to_string(),
                version: "3.0".to_string(),
                timestamp: 0.0,
                brain_regions_root: Some(region_key.clone()),
            },
            cortical_areas: HashMap::from([(cortical_id, area.clone())]),
            brain_regions: HashMap::from([(region_key.clone(), region.clone())]),
            morphologies: feagi_evolutionary::MorphologyRegistry::new(),
            physiology: feagi_evolutionary::PhysiologyConfig::default(),
            signatures: feagi_evolutionary::GenomeSignatures {
                genome: "0".to_string(),
                blueprint: "0".to_string(),
                physiology: "0".to_string(),
                morphologies: None,
            },
            stats: feagi_evolutionary::GenomeStats::default(),
        };
        let current_genome = Arc::new(RwLock::new(Some(genome)));

        // Seed ConnectomeManager with the same region + area (this is what BV and runtime uses).
        {
            let mut mgr = connectome.write();
            mgr.add_brain_region(region, None)
                .expect("brain region should be addable");
            mgr.add_cortical_area(area)
                .expect("cortical area should be addable");
        }

        let svc = ConnectomeServiceImpl::new(connectome.clone(), current_genome.clone());

        // Act: delete by base64 string.
        let cortical_id_base64 = cortical_id.as_base_64();
        svc.delete_cortical_area(&cortical_id_base64).await?;

        // Assert: RuntimeGenome no longer contains the area nor region membership.
        {
            let genome_guard = current_genome.read();
            let genome = genome_guard.as_ref().expect("genome must exist");
            assert!(!genome.cortical_areas.contains_key(&cortical_id));
            let region = genome
                .brain_regions
                .get(&region_key)
                .expect("region must exist in genome");
            assert!(!region.contains_area(&cortical_id));
        }

        Ok(())
    }
}
