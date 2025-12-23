// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM Connectome Service
//!
//! Extracts cortical area and brain region data from RuntimeGenome.

use async_trait::async_trait;
use feagi_evolutionary::RuntimeGenome;
use feagi_services::traits::connectome_service::ConnectomeService;
use feagi_services::types::errors::{ServiceError, ServiceResult};
use feagi_services::types::*;
use feagi_structures::genomic::cortical_area::CorticalID;
use std::collections::HashMap;
use std::sync::Arc;

/// WASM Connectome Service
///
/// Extracts data from RuntimeGenome to implement ConnectomeService trait.
/// Read-only operations only (no mutations).
pub struct WasmConnectomeService {
    /// Runtime genome (read-only)
    genome: Arc<RuntimeGenome>,
}

impl WasmConnectomeService {
    /// Create new WASM connectome service
    pub fn new(genome: Arc<RuntimeGenome>) -> Self {
        Self { genome }
    }

    /// Convert CorticalArea to CorticalAreaInfo
    fn area_to_info(
        &self,
        cortical_id: &CorticalID,
        area: &feagi_structures::genomic::cortical_area::CorticalArea,
    ) -> CorticalAreaInfo {
        use feagi_structures::genomic::cortical_area::CorticalArea;

        // Extract physiology parameters from properties
        let leak_coefficient = area
            .properties
            .get("leak_coefficient")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.1);

        // Extract other properties
        let neurons_per_voxel = area
            .properties
            .get("neurons_per_voxel")
            .and_then(|v| v.as_u64())
            .map(|u| u as u32)
            .unwrap_or(1);

        let postsynaptic_current = area
            .properties
            .get("postsynaptic_current")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.1);

        // Determine cortical group and area type from cortical_type
        // Extract area type string from properties or use default
        let area_type_str = area
            .properties
            .get("area_type")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "Custom".to_string());

        let cortical_group = match area_type_str.as_str() {
            "Sensory" | "IPU" => "IPU".to_string(),
            "Motor" | "OPU" => "OPU".to_string(),
            "Memory" => "MEMORY".to_string(),
            "Custom" => "CUSTOM".to_string(),
            _ => "CORE".to_string(),
        };

        CorticalAreaInfo {
            cortical_id: cortical_id.to_string(),
            cortical_id_s: cortical_id.to_string(), // TODO: Decode base64 if needed
            cortical_idx: area.cortical_idx,
            name: area.name.clone(),
            dimensions: (
                area.dimensions.width as usize,
                area.dimensions.height as usize,
                area.dimensions.depth as usize,
            ),
            position: (area.position.x, area.position.y, area.position.z),
            area_type: area_type_str,
            cortical_group,
            neuron_count: 0,  // TODO: Extract from NPU if available
            synapse_count: 0, // TODO: Extract from NPU if available
            visible: area
                .properties
                .get("visible")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            sub_group: area
                .properties
                .get("cortical_sub_group")
                .and_then(|v| v.as_str())
                .map(String::from),
            neurons_per_voxel,
            postsynaptic_current,
            plasticity_constant: area
                .properties
                .get("plasticity_constant")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            degeneration: area
                .properties
                .get("degeneration")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            psp_uniform_distribution: area
                .properties
                .get("psp_uniform_distribution")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            firing_threshold_increment: area
                .properties
                .get("firing_threshold_increment")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            firing_threshold_limit: area
                .properties
                .get("firing_threshold_limit")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0),
            consecutive_fire_count: area
                .properties
                .get("consecutive_fire_count")
                .and_then(|v| v.as_u64())
                .map(|u| u as u32)
                .unwrap_or(0),
            snooze_period: area
                .properties
                .get("snooze_period")
                .and_then(|v| v.as_u64())
                .map(|u| u as u32)
                .unwrap_or(0),
            refractory_period: area
                .properties
                .get("refractory_period")
                .and_then(|v| v.as_u64())
                .map(|u| u as u32)
                .unwrap_or(0),
            leak_coefficient,
            leak_variability: area
                .properties
                .get("leak_variability")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            burst_engine_active: true, // Always active in WASM
            properties: area.properties.clone(),
            cortical_subtype: None, // TODO: Extract from cortical_id if IPU/OPU
            encoding_type: None,    // TODO: Extract from cortical_id if IPU/OPU
            encoding_format: None,  // TODO: Extract from cortical_id if IPU/OPU
            unit_id: None,          // TODO: Extract from cortical_id if IPU/OPU
            group_id: None,         // TODO: Extract from cortical_id if IPU/OPU
            parent_region_id: None, // TODO: Find which brain region contains this area
        }
    }
}

#[async_trait]
impl ConnectomeService for WasmConnectomeService {
    async fn create_cortical_area(
        &self,
        _params: CreateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo> {
        Err(ServiceError::NotImplemented(
            "WASM mode is read-only".to_string(),
        ))
    }

    async fn update_cortical_area(
        &self,
        _cortical_id: &str,
        _params: UpdateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo> {
        Err(ServiceError::NotImplemented(
            "WASM mode is read-only".to_string(),
        ))
    }

    async fn delete_cortical_area(&self, _cortical_id: &str) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "WASM mode is read-only".to_string(),
        ))
    }

    async fn get_cortical_area(&self, cortical_id: &str) -> ServiceResult<CorticalAreaInfo> {
        let cortical_id_parsed = CorticalID::try_from_base_64(cortical_id).map_err(|_| {
            ServiceError::InvalidInput(format!("Invalid cortical ID format: {}", cortical_id))
        })?;
        let area = self
            .genome
            .cortical_areas
            .get(&cortical_id_parsed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "cortical_area".to_string(),
                id: cortical_id.to_string(),
            })?;

        Ok(self.area_to_info(&cortical_id_parsed, area))
    }

    async fn list_cortical_areas(&self) -> ServiceResult<Vec<CorticalAreaInfo>> {
        let areas: Vec<CorticalAreaInfo> = self
            .genome
            .cortical_areas
            .iter()
            .map(|(id, area)| self.area_to_info(id, area))
            .collect();

        Ok(areas)
    }

    async fn get_cortical_area_ids(&self) -> ServiceResult<Vec<String>> {
        Ok(self
            .genome
            .cortical_areas
            .keys()
            .map(|id| id.to_string())
            .collect::<Vec<_>>())
    }

    async fn cortical_area_exists(&self, cortical_id: &str) -> ServiceResult<bool> {
        let cortical_id_parsed = CorticalID::try_from_base_64(cortical_id).map_err(|_| {
            ServiceError::InvalidInput(format!("Invalid cortical ID format: {}", cortical_id))
        })?;
        Ok(self.genome.cortical_areas.contains_key(&cortical_id_parsed))
    }

    async fn get_cortical_area_properties(
        &self,
        cortical_id: &str,
    ) -> ServiceResult<std::collections::HashMap<String, serde_json::Value>> {
        let cortical_id_parsed = CorticalID::try_from_base_64(cortical_id).map_err(|_| {
            ServiceError::InvalidInput(format!("Invalid cortical ID format: {}", cortical_id))
        })?;
        let area = self
            .genome
            .cortical_areas
            .get(&cortical_id_parsed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "cortical_area".to_string(),
                id: cortical_id.to_string(),
            })?;

        Ok(area.properties.clone())
    }

    async fn get_all_cortical_area_properties(
        &self,
    ) -> ServiceResult<Vec<std::collections::HashMap<String, serde_json::Value>>> {
        Ok(self
            .genome
            .cortical_areas
            .values()
            .map(|area| area.properties.clone())
            .collect())
    }

    async fn create_brain_region(
        &self,
        _params: CreateBrainRegionParams,
    ) -> ServiceResult<BrainRegionInfo> {
        Err(ServiceError::NotImplemented(
            "WASM mode is read-only".to_string(),
        ))
    }

    async fn delete_brain_region(&self, _region_id: &str) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "WASM mode is read-only".to_string(),
        ))
    }

    async fn update_brain_region(
        &self,
        _region_id: &str,
        _properties: std::collections::HashMap<String, serde_json::Value>,
    ) -> ServiceResult<BrainRegionInfo> {
        Err(ServiceError::NotImplemented(
            "WASM mode is read-only".to_string(),
        ))
    }

    async fn get_brain_region(&self, region_id: &str) -> ServiceResult<BrainRegionInfo> {
        let _region =
            self.genome
                .brain_regions
                .get(region_id)
                .ok_or_else(|| ServiceError::NotFound {
                    resource: "brain_region".to_string(),
                    id: region_id.to_string(),
                })?;

        // Convert BrainRegion to BrainRegionInfo
        // TODO: Implement full conversion
        Err(ServiceError::NotImplemented(
            "Brain region conversion not yet implemented".to_string(),
        ))
    }

    async fn list_brain_regions(&self) -> ServiceResult<Vec<BrainRegionInfo>> {
        // TODO: Convert all brain regions to BrainRegionInfo
        Err(ServiceError::NotImplemented(
            "Brain region listing not yet implemented".to_string(),
        ))
    }

    async fn get_brain_region_ids(&self) -> ServiceResult<Vec<String>> {
        Ok(self.genome.brain_regions.keys().cloned().collect())
    }

    async fn brain_region_exists(&self, region_id: &str) -> ServiceResult<bool> {
        Ok(self.genome.brain_regions.contains_key(region_id))
    }

    async fn get_morphologies(
        &self,
    ) -> ServiceResult<std::collections::HashMap<String, MorphologyInfo>> {
        // TODO: Convert MorphologyRegistry to HashMap<String, MorphologyInfo>
        Err(ServiceError::NotImplemented(
            "Morphology extraction not yet implemented".to_string(),
        ))
    }

    async fn update_cortical_mapping(
        &self,
        _src_area_id: String,
        _dst_area_id: String,
        _mapping_data: Vec<serde_json::Value>,
    ) -> ServiceResult<usize> {
        Err(ServiceError::NotImplemented(
            "WASM mode is read-only".to_string(),
        ))
    }
}
