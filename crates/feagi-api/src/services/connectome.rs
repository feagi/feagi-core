// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Connectome service over the loaded genome and the running NPU.
//!
//! Structural detail (names, positions, physiology, brain regions, morphologies) comes from the
//! genome. Areas that exist in the engine are the live truth about what the brain currently holds,
//! including areas created through this API after the genome was loaded.

use crate::services::{npu_unavailable, OptionalNpu};
use async_trait::async_trait;
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_services::traits::connectome_service::ConnectomeService;
use feagi_services::types::errors::{ServiceError, ServiceResult};
use feagi_services::types::*;
use std::collections::HashMap;

/// Physiology values the genome reader applies when a genome omits them. Reused for areas created
/// through the API so both paths describe an unspecified area identically.
const DEFAULT_POSTSYNAPTIC_CURRENT: f64 = 0.1;
const DEFAULT_LEAK_COEFFICIENT: f64 = 0.1;
const DEFAULT_NEURON_EXCITABILITY: f64 = 1.0;

/// Parses a cortical id written either as 8 raw ASCII characters (`cust0001`) or as base64.
///
/// The raw form is tried first because it is what the REST API has always accepted.
fn parse_cortical_id(raw: &str) -> ServiceResult<CorticalID> {
    let bytes = raw.as_bytes();
    if bytes.len() == CorticalID::CORTICAL_ID_LENGTH {
        let mut fixed = [0u8; CorticalID::CORTICAL_ID_LENGTH];
        fixed.copy_from_slice(bytes);
        if let Ok(id) = CorticalID::try_from_bytes(&fixed) {
            return Ok(id);
        }
    }

    CorticalID::try_from_base_64(raw).map_err(|_| {
        ServiceError::InvalidInput(format!(
            "'{}' is neither 8 ASCII bytes starting with one of c/m/_/i/o, nor base64 encoding \
             such an ID",
            raw
        ))
    })
}

pub struct GenomeConnectomeService {
    /// Structural source for area metadata and the brain region tree.
    genome: crate::services::SharedGenome,
    /// The running engine, when one was injected.
    npu: OptionalNpu,
}

impl GenomeConnectomeService {
    /// Creates the service over the shared genome and an optional NPU handle.
    pub fn new(genome: crate::services::SharedGenome, npu: OptionalNpu) -> Self {
        Self { genome, npu }
    }

    /// The injected NPU, or an error naming the operation that needed it.
    fn npu(&self, operation: &str) -> ServiceResult<&dyn crate::services::NpuAccess> {
        self.npu
            .as_deref()
            .ok_or_else(|| npu_unavailable(operation))
    }

    /// Builds area info for an area the engine holds but the genome does not describe, which is
    /// the case for areas created through this API after the genome was loaded.
    ///
    /// Only the engine-known quantities are populated; the physiology fields carry the same
    /// creation-time values the caller supplied or the documented request defaults.
    fn npu_area_to_info(area: &crate::services::NpuCorticalArea) -> CorticalAreaInfo {
        let [x, y, z, density] = area.dimensions;
        CorticalAreaInfo {
            cortical_id: area.id.as_base_64(),
            cortical_id_s: std::str::from_utf8(area.id.as_bytes())
                .map(|s| s.to_string())
                .unwrap_or_else(|_| area.id.as_base_64()),
            cortical_idx: 0,
            name: std::str::from_utf8(area.id.as_bytes())
                .map(|s| s.to_string())
                .unwrap_or_else(|_| area.id.as_base_64()),
            dimensions: (x as usize, y as usize, z as usize),
            position: (0, 0, 0),
            area_type: "Custom".to_string(),
            cortical_group: "CUSTOM".to_string(),
            cortical_type: "Custom".to_string(),
            neuron_count: area.neuron_count as usize,
            synapse_count: 0,
            incoming_synapse_count: 0,
            outgoing_synapse_count: 0,
            visible: true,
            sub_group: None,
            neurons_per_voxel: density as u32,
            // Physiology matches the defaults the genome reader applies for unspecified fields,
            // so an area behaves the same whether it arrived by genome or by API.
            postsynaptic_current: DEFAULT_POSTSYNAPTIC_CURRENT,
            postsynaptic_current_max: DEFAULT_POSTSYNAPTIC_CURRENT,
            neuron_excitability: DEFAULT_NEURON_EXCITABILITY,
            leak_coefficient: DEFAULT_LEAK_COEFFICIENT,
            mp_charge_accumulation: true,
            burst_engine_active: true,
            ..CorticalAreaInfo::default()
        }
    }

    /// Convert CorticalArea to CorticalAreaInfo
    fn area_to_info(
        &self,
        cortical_id: &CorticalID,
        area: &feagi_genomic_data::cortical_area_prev::CorticalArea,
    ) -> CorticalAreaInfo {

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

        // Determine cortical_area group and area type from cortical_type
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
        let cortical_type = match cortical_group.as_str() {
            "IPU" => "sensory".to_string(),
            "OPU" => "motor".to_string(),
            "MEMORY" => "memory".to_string(),
            "CORE" => "core".to_string(),
            _ => "custom".to_string(),
        };

        let firing_threshold = area
            .properties
            .get("firing_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let firing_threshold_increment = [
            area.properties
                .get("firing_threshold_increment_x")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            area.properties
                .get("firing_threshold_increment_y")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            area.properties
                .get("firing_threshold_increment_z")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        ];
        let postsynaptic_current_max = area
            .properties
            .get("postsynaptic_current_max")
            .and_then(|v| v.as_f64())
            .unwrap_or(postsynaptic_current);
        let mp_driven_psp = area
            .properties
            .get("mp_driven_psp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let mp_charge_accumulation = area
            .properties
            .get("mp_charge_accumulation")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let neuron_excitability = area
            .properties
            .get("neuron_excitability")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let init_lifespan = area
            .properties
            .get("init_lifespan")
            .and_then(|v| v.as_u64())
            .map(|u| u as u32)
            .unwrap_or(0);
        let lifespan_growth_rate = area
            .properties
            .get("lifespan_growth_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let longterm_mem_threshold = area
            .properties
            .get("longterm_mem_threshold")
            .and_then(|v| v.as_u64())
            .map(|u| u as u32)
            .unwrap_or(0);
        let temporal_depth = area
            .properties
            .get("temporal_depth")
            .and_then(|v| v.as_u64())
            .map(|u| u as u32);
        let mp_learning_enabled = area
            .properties
            .get("mp_learning_enabled")
            .and_then(|v| v.as_bool());

        let cid_bytes = cortical_id.as_bytes();
        let is_io_area_wasm = cid_bytes.len() == 8
            && (cid_bytes.first().copied() == Some(b'i')
                || cid_bytes.first().copied() == Some(b'o'));
        let cortical_subtype_field = if is_io_area_wasm {
            String::from_utf8(cid_bytes[0..4].to_vec()).ok()
        } else {
            None
        };
        let wasm_subunit_id = if is_io_area_wasm {
            cid_bytes.get(6).copied()
        } else {
            None
        };
        let wasm_cortical_unit_index = if is_io_area_wasm {
            cid_bytes.get(7).copied()
        } else {
            None
        };

        CorticalAreaInfo {
            cortical_id: cortical_id.to_string(),
            cortical_id_s: cortical_id.to_string(), // TODO: Decode base64 if needed
            cortical_idx: area.cortical_idx,
            name: area.name.clone(),
            dimensions: (
                *area.dimensions.get_x().as_ref() as usize,
                *area.dimensions.get_y().as_ref() as usize,
                *area.dimensions.get_z().as_ref() as usize,
            ),
            position: (area.position.x(), area.position.y(), area.position.z()),
            area_type: area_type_str,
            cortical_group,
            cortical_type,
            neuron_count: 0,           // TODO: Extract from NPU if available
            synapse_count: 0,          // TODO: Extract from NPU if available
            incoming_synapse_count: 0, // TODO: Extract from NPU if available
            outgoing_synapse_count: 0, // TODO: Extract from NPU if available
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
            postsynaptic_current_max,
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
            mp_driven_psp,
            firing_threshold,
            firing_threshold_increment,
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
            mp_charge_accumulation,
            neuron_excitability,
            burst_engine_active: true, // Always active in WASM
            init_lifespan,
            lifespan_growth_rate,
            longterm_mem_threshold,
            temporal_depth,
            mp_learning_enabled,
            properties: area.properties.clone(),
            cortical_subtype: cortical_subtype_field,
            encoding_type: None,
            encoding_format: None,
            unit_id: wasm_cortical_unit_index,
            subunit_id: wasm_subunit_id,
            group_id: wasm_cortical_unit_index,
            coding_signage: None,
            coding_behavior: None,
            coding_type: None,
            coding_options: None,
            parent_region_id: None, // TODO: Find which brain region contains this area
            dev_count: None,
            cortical_dimensions_per_device: None,
            visualization_voxel_granularity: None,
        }
    }
}

/// Realises a cortical area in the engine and describes the result.
///
/// Shared because the REST contract creates areas through two entry points: `ConnectomeService`
/// for direct creation and `GenomeService` for the custom-area route. Both must produce the same
/// area and the same description.
pub(crate) fn create_area_in_npu(
    npu: &dyn crate::services::NpuAccess,
    params: CreateCorticalAreaParams,
) -> ServiceResult<CorticalAreaInfo> {
    let cortical_id = parse_cortical_id(&params.cortical_id)?;
    let (x, y, z) = params.dimensions;
    let density = params.neurons_per_voxel.unwrap_or(1) as u64;

    let created = npu
        .add_cortical_area(cortical_id, x as u64, y as u64, z as u64, density)
        .map_err(ServiceError::InvalidInput)?;

    // Report what the caller asked for, over the identity and neuron count the engine realised.
    let mut info = GenomeConnectomeService::npu_area_to_info(&created);
    info.name = params.name;
    info.position = params.position;
    info.area_type = params.area_type;
    info.visible = params.visible.unwrap_or(true);
    info.sub_group = params.sub_group;
    if let Some(v) = params.postsynaptic_current {
        info.postsynaptic_current = v;
    }
    if let Some(v) = params.plasticity_constant {
        info.plasticity_constant = v;
    }
    if let Some(v) = params.degeneration {
        info.degeneration = v;
    }
    if let Some(v) = params.psp_uniform_distribution {
        info.psp_uniform_distribution = v;
    }
    if let Some(v) = params.firing_threshold_limit {
        info.firing_threshold_limit = v;
    }
    if let Some(v) = params.consecutive_fire_count {
        info.consecutive_fire_count = v;
    }
    if let Some(v) = params.snooze_period {
        info.snooze_period = v;
    }
    if let Some(v) = params.refractory_period {
        info.refractory_period = v;
    }
    if let Some(v) = params.leak_coefficient {
        info.leak_coefficient = v;
    }
    if let Some(v) = params.leak_variability {
        info.leak_variability = v;
    }
    if let Some(v) = params.burst_engine_active {
        info.burst_engine_active = v;
    }
    if let Some(properties) = params.properties {
        info.properties = properties;
    }

    Ok(info)
}

#[async_trait]
impl ConnectomeService for GenomeConnectomeService {
    async fn create_cortical_area(
        &self,
        params: CreateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo> {
        create_area_in_npu(self.npu("cortical area creation")?, params)
    }

    async fn update_cortical_area(
        &self,
        _cortical_id: &str,
        _params: UpdateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo> {
        Err(ServiceError::NotImplemented(
            "genome-backed service is read-only".to_string(),
        ))
    }

    async fn delete_cortical_area(&self, _cortical_id: &str) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "genome-backed service is read-only".to_string(),
        ))
    }

    async fn get_cortical_area(&self, cortical_id: &str) -> ServiceResult<CorticalAreaInfo> {
        let cortical_id_parsed = CorticalID::try_from_base_64(cortical_id).map_err(|_| {
            ServiceError::InvalidInput(format!("Invalid cortical_area ID format: {}", cortical_id))
        })?;
        crate::services::with_genome(&self.genome, |g| {
            g.cortical_areas
                .get(&cortical_id_parsed)
                .map(|area| self.area_to_info(&cortical_id_parsed, area))
        })?
        .ok_or_else(|| ServiceError::NotFound {
            resource: "cortical_area".to_string(),
            id: cortical_id.to_string(),
        })
    }

    async fn list_cortical_areas(&self) -> ServiceResult<Vec<CorticalAreaInfo>> {
        // With an engine attached, the areas it holds are the live truth: the genome may have been
        // superseded by areas created through this API. The genome still supplies the metadata for
        // any area it describes.
        let Some(npu) = self.npu.as_deref() else {
            return crate::services::with_genome(&self.genome, |g| {
                g.cortical_areas
                    .iter()
                    .map(|(id, area)| self.area_to_info(id, area))
                    .collect()
            });
        };

        let genome = self.genome.read();
        Ok(npu
            .cortical_areas()
            .iter()
            .map(|live| {
                genome
                    .as_ref()
                    .and_then(|g| g.cortical_areas.get(&live.id))
                    .map(|described| self.area_to_info(&live.id, described))
                    .unwrap_or_else(|| Self::npu_area_to_info(live))
            })
            .collect())
    }

    async fn get_cortical_area_ids(&self) -> ServiceResult<Vec<String>> {
        // Same authority rule as `list_cortical_areas`: the engine knows what exists.
        match self.npu.as_deref() {
            Some(npu) => Ok(npu
                .cortical_areas()
                .iter()
                .map(|area| area.id.to_string())
                .collect()),
            None => crate::services::with_genome(&self.genome, |g| {
                g.cortical_areas
                    .keys()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
            }),
        }
    }

    async fn cortical_area_exists(&self, cortical_id: &str) -> ServiceResult<bool> {
        let cortical_id_parsed = CorticalID::try_from_base_64(cortical_id).map_err(|_| {
            ServiceError::InvalidInput(format!("Invalid cortical_area ID format: {}", cortical_id))
        })?;
        crate::services::with_genome(&self.genome, |g| {
            g.cortical_areas.contains_key(&cortical_id_parsed)
        })
    }

    async fn get_cortical_area_properties(
        &self,
        cortical_id: &str,
    ) -> ServiceResult<std::collections::HashMap<String, serde_json::Value>> {
        let cortical_id_parsed = CorticalID::try_from_base_64(cortical_id).map_err(|_| {
            ServiceError::InvalidInput(format!("Invalid cortical_area ID format: {}", cortical_id))
        })?;
        crate::services::with_genome(&self.genome, |g| {
            g.cortical_areas
                .get(&cortical_id_parsed)
                .map(|area| area.properties.clone())
        })?
        .ok_or_else(|| ServiceError::NotFound {
            resource: "cortical_area".to_string(),
            id: cortical_id.to_string(),
        })
    }

    async fn get_all_cortical_area_properties(
        &self,
    ) -> ServiceResult<Vec<std::collections::HashMap<String, serde_json::Value>>> {
        crate::services::with_genome(&self.genome, |g| {
            g.cortical_areas
                .values()
                .map(|area| area.properties.clone())
                .collect()
        })
    }

    async fn create_brain_region(
        &self,
        _params: CreateBrainRegionParams,
    ) -> ServiceResult<BrainRegionInfo> {
        Err(ServiceError::NotImplemented(
            "genome-backed service is read-only".to_string(),
        ))
    }

    async fn delete_brain_region(&self, _region_id: &str) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "genome-backed service is read-only".to_string(),
        ))
    }

    async fn update_brain_region(
        &self,
        _region_id: &str,
        _properties: std::collections::HashMap<String, serde_json::Value>,
    ) -> ServiceResult<BrainRegionInfo> {
        Err(ServiceError::NotImplemented(
            "genome-backed service is read-only".to_string(),
        ))
    }

    async fn get_brain_region(&self, region_id: &str) -> ServiceResult<BrainRegionInfo> {
        if !crate::services::with_genome(&self.genome, |g| g.brain_regions.contains_key(region_id))?
        {
            return Err(ServiceError::NotFound {
                resource: "brain_region".to_string(),
                id: region_id.to_string(),
            });
        }

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
        crate::services::with_genome(&self.genome, |g| {
            g.brain_regions.keys().cloned().collect()
        })
    }

    async fn brain_region_exists(&self, region_id: &str) -> ServiceResult<bool> {
        crate::services::with_genome(&self.genome, |g| g.brain_regions.contains_key(region_id))
    }

    async fn get_root_region_id(&self) -> ServiceResult<Option<String>> {
        // The genome parser records each region's parent under the `parent_region_id` property;
        // the root is the one that has none. This matches `BrainRegionHierarchy::get_root_region_id`,
        // which selects the region absent from its parent map.
        crate::services::with_genome(&self.genome, |g| {
            g.brain_regions
                .iter()
                .find(|(_, region)| {
                    !matches!(
                        region.properties.get("parent_region_id"),
                        Some(v) if !v.is_null()
                    )
                })
                .map(|(region_id, _)| region_id.clone())
        })
    }

    async fn get_morphologies(
        &self,
    ) -> ServiceResult<std::collections::HashMap<String, MorphologyInfo>> {
        // TODO: Convert MorphologyRegistry to HashMap<String, MorphologyInfo>
        Err(ServiceError::NotImplemented(
            "Morphology extraction not yet implemented".to_string(),
        ))
    }

    async fn create_morphology(
        &self,
        _morphology_id: String,
        _morphology: feagi_evolutionary::Morphology,
    ) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "genome-backed service is read-only".to_string(),
        ))
    }

    async fn update_morphology(
        &self,
        _morphology_id: String,
        _morphology: feagi_evolutionary::Morphology,
    ) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "genome-backed service is read-only".to_string(),
        ))
    }

    async fn delete_morphology(&self, _morphology_id: &str) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "genome-backed service is read-only".to_string(),
        ))
    }

    async fn rename_morphology(&self, _old_id: &str, _new_id: &str) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "genome-backed service is read-only".to_string(),
        ))
    }

    async fn update_cortical_mapping(
        &self,
        _src_area_id: String,
        _dst_area_id: String,
        _mapping_data: Vec<serde_json::Value>,
    ) -> ServiceResult<usize> {
        Err(ServiceError::NotImplemented(
            "genome-backed service is read-only".to_string(),
        ))
    }

    async fn get_neuron_properties(
        &self,
        _neuron_id: u64,
    ) -> ServiceResult<HashMap<String, serde_json::Value>> {
        Err(ServiceError::NotImplemented(
            "neuron properties require live neural state, which the genome does not carry"
                .to_string(),
        ))
    }

    async fn export_connectome(&self) -> ServiceResult<ConnectomeSnapshot> {
        Err(ServiceError::NotImplemented(
            "connectome export requires live neuron and synapse state".to_string(),
        ))
    }

    async fn import_connectome(&self, _snapshot: ConnectomeSnapshot) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "genome-backed service is read-only".to_string(),
        ))
    }
}
