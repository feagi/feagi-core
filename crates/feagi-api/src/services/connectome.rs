// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Connectome service over the loaded genome and the running NPU.
//!
//! Structural detail (names, positions, physiology, brain regions, morphologies) comes from the
//! genome. Areas that exist in the engine are the live truth about what the brain currently holds,
//! including areas created through this API after the genome was loaded.

use crate::services::{npu_unavailable, OptionalNpu};
use async_trait::async_trait;
use feagi_genomic_context::brain_region::{BrainRegion, RegionID, RegionType};
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
        self.npu.as_deref().ok_or_else(|| npu_unavailable(operation))
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

    /// Describes one brain region, resolving the links the genome stores only in one direction.
    ///
    /// The genome records a region's parent under the `parent_region_id` property and nothing
    /// else, so the child list is recovered by asking which regions name this one as their parent.
    /// `all_regions` is therefore the whole map, not just the region being described.
    fn region_to_info(
        region_id: &str,
        region: &feagi_genomic_context::brain_region::BrainRegion,
        all_regions: &std::collections::HashMap<String, feagi_genomic_context::brain_region::BrainRegion>,
    ) -> BrainRegionInfo {
        fn parent_of(candidate: &feagi_genomic_context::brain_region::BrainRegion) -> Option<&str> {
            candidate.properties.get("parent_region_id").and_then(|value| value.as_str())
        }

        BrainRegionInfo {
            region_id: region_id.to_string(),
            name: region.name.clone(),
            region_type: region.region_type.to_string(),
            parent_id: parent_of(region).map(String::from),
            cortical_areas: region.cortical_areas.iter().map(|area_id| area_id.to_string()).collect(),
            child_regions: all_regions
                .iter()
                .filter(|(_, candidate)| parent_of(candidate) == Some(region_id))
                .map(|(child_id, _)| child_id.clone())
                .collect(),
            properties: region.properties.clone(),
        }
    }

    /// Names a morphology's type using the same spellings the genome format uses.
    ///
    /// Matched explicitly rather than derived from the serde representation so the wire vocabulary
    /// is visible here and cannot drift with a serde attribute change.
    fn morphology_type_name(morphology_type: &feagi_evolutionary::MorphologyType) -> &'static str {
        match morphology_type {
            feagi_evolutionary::MorphologyType::Vectors => "vectors",
            feagi_evolutionary::MorphologyType::Patterns => "patterns",
            feagi_evolutionary::MorphologyType::Functions => "functions",
            feagi_evolutionary::MorphologyType::Composite => "composite",
        }
    }

    /// Convert CorticalArea to CorticalAreaInfo
    fn area_to_info(&self, cortical_id: &CorticalID, area: &feagi_genomic_data::cortical_area_prev::CorticalArea) -> CorticalAreaInfo {
        // Extract physiology parameters from properties
        let leak_coefficient = area.properties.get("leak_coefficient").and_then(|v| v.as_f64()).unwrap_or(0.1);

        // Extract other properties
        let neurons_per_voxel = area
            .properties
            .get("neurons_per_voxel")
            .and_then(|v| v.as_u64())
            .map(|u| u as u32)
            .unwrap_or(1);

        let postsynaptic_current = area.properties.get("postsynaptic_current").and_then(|v| v.as_f64()).unwrap_or(0.1);

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

        let firing_threshold = area.properties.get("firing_threshold").and_then(|v| v.as_f64()).unwrap_or(1.0);
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
        let mp_driven_psp = area.properties.get("mp_driven_psp").and_then(|v| v.as_bool()).unwrap_or(false);
        let mp_charge_accumulation = area.properties.get("mp_charge_accumulation").and_then(|v| v.as_bool()).unwrap_or(false);
        let neuron_excitability = area.properties.get("neuron_excitability").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let init_lifespan = area
            .properties
            .get("init_lifespan")
            .and_then(|v| v.as_u64())
            .map(|u| u as u32)
            .unwrap_or(0);
        let lifespan_growth_rate = area.properties.get("lifespan_growth_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let longterm_mem_threshold = area
            .properties
            .get("longterm_mem_threshold")
            .and_then(|v| v.as_u64())
            .map(|u| u as u32)
            .unwrap_or(0);
        let temporal_depth = area.properties.get("temporal_depth").and_then(|v| v.as_u64()).map(|u| u as u32);
        let mp_learning_enabled = area.properties.get("mp_learning_enabled").and_then(|v| v.as_bool());

        let cid_bytes = cortical_id.as_bytes();
        let is_io_area_wasm = cid_bytes.len() == 8 && (cid_bytes.first().copied() == Some(b'i') || cid_bytes.first().copied() == Some(b'o'));
        let cortical_subtype_field = if is_io_area_wasm {
            String::from_utf8(cid_bytes[0..4].to_vec()).ok()
        } else {
            None
        };
        let wasm_subunit_id = if is_io_area_wasm { cid_bytes.get(6).copied() } else { None };
        let wasm_cortical_unit_index = if is_io_area_wasm { cid_bytes.get(7).copied() } else { None };

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
            visible: area.properties.get("visible").and_then(|v| v.as_bool()).unwrap_or(true),
            sub_group: area.properties.get("cortical_sub_group").and_then(|v| v.as_str()).map(String::from),
            neurons_per_voxel,
            postsynaptic_current,
            postsynaptic_current_max,
            plasticity_constant: area.properties.get("plasticity_constant").and_then(|v| v.as_f64()).unwrap_or(0.0),
            degeneration: area.properties.get("degeneration").and_then(|v| v.as_f64()).unwrap_or(0.0),
            psp_uniform_distribution: area.properties.get("psp_uniform_distribution").and_then(|v| v.as_bool()).unwrap_or(false),
            mp_driven_psp,
            firing_threshold,
            firing_threshold_increment,
            firing_threshold_limit: area.properties.get("firing_threshold_limit").and_then(|v| v.as_f64()).unwrap_or(1.0),
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
            leak_variability: area.properties.get("leak_variability").and_then(|v| v.as_f64()).unwrap_or(0.0),
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

/// Rewrites every reference to `old_id` in a mapping rule so it names `new_id`.
///
/// Mapping rules are stored as free-form JSON on cortical area and brain region properties, and a
/// morphology can be referenced three ways: as a `morphology_id` field, as the `mapper_morphology`
/// of a composite, or as the leading element of a rule tuple. All three are rewritten, and nested
/// values are followed, because rules are arbitrarily nested.
fn rename_morphology_references(value: &mut serde_json::Value, old_id: &str, new_id: &str, replaced: &mut usize) {
    match value {
        serde_json::Value::Object(object) => {
            for field in ["morphology_id", "mapper_morphology"] {
                if let Some(reference) = object.get_mut(field) {
                    if reference.as_str() == Some(old_id) {
                        *reference = serde_json::Value::String(new_id.to_string());
                        *replaced += 1;
                    }
                }
            }
            for child in object.values_mut() {
                rename_morphology_references(child, old_id, new_id, replaced);
            }
        }
        serde_json::Value::Array(array) => {
            // A rule tuple leads with the morphology it applies.
            if let Some(first) = array.first_mut() {
                if first.as_str() == Some(old_id) {
                    *first = serde_json::Value::String(new_id.to_string());
                    *replaced += 1;
                }
            }
            for child in array.iter_mut() {
                rename_morphology_references(child, old_id, new_id, replaced);
            }
        }
        _ => {}
    }
}

/// Realises a cortical area in the engine and describes the result.
///
/// Shared because the REST contract creates areas through two entry points: `ConnectomeService`
/// for direct creation and `GenomeService` for the custom-area route. Both must produce the same
/// area and the same description.
pub(crate) fn create_area_in_npu(npu: &dyn crate::services::NpuAccess, params: CreateCorticalAreaParams) -> ServiceResult<CorticalAreaInfo> {
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
    async fn create_cortical_area(&self, params: CreateCorticalAreaParams) -> ServiceResult<CorticalAreaInfo> {
        create_area_in_npu(self.npu("cortical area creation")?, params)
    }

    async fn update_cortical_area(&self, _cortical_id: &str, _params: UpdateCorticalAreaParams) -> ServiceResult<CorticalAreaInfo> {
        Err(ServiceError::NotImplemented("genome-backed service is read-only".to_string()))
    }

    async fn delete_cortical_area(&self, _cortical_id: &str) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented("genome-backed service is read-only".to_string()))
    }

    async fn get_cortical_area(&self, cortical_id: &str) -> ServiceResult<CorticalAreaInfo> {
        let cortical_id_parsed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|_| ServiceError::InvalidInput(format!("Invalid cortical_area ID format: {}", cortical_id)))?;
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
            return Ok(crate::services::with_genome_or_default(&self.genome, |g| {
                g.cortical_areas.iter().map(|(id, area)| self.area_to_info(id, area)).collect()
            }));
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
            Some(npu) => Ok(npu.cortical_areas().iter().map(|area| area.id.to_string()).collect()),
            None => Ok(crate::services::with_genome_or_default(&self.genome, |g| {
                g.cortical_areas.keys().map(|id| id.to_string()).collect::<Vec<_>>()
            })),
        }
    }

    async fn cortical_area_exists(&self, cortical_id: &str) -> ServiceResult<bool> {
        let cortical_id_parsed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|_| ServiceError::InvalidInput(format!("Invalid cortical_area ID format: {}", cortical_id)))?;
        Ok(crate::services::with_genome_or_default(&self.genome, |g| {
            g.cortical_areas.contains_key(&cortical_id_parsed)
        }))
    }

    async fn get_cortical_area_properties(&self, cortical_id: &str) -> ServiceResult<std::collections::HashMap<String, serde_json::Value>> {
        let cortical_id_parsed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|_| ServiceError::InvalidInput(format!("Invalid cortical_area ID format: {}", cortical_id)))?;
        crate::services::with_genome(&self.genome, |g| {
            g.cortical_areas.get(&cortical_id_parsed).map(|area| area.properties.clone())
        })?
        .ok_or_else(|| ServiceError::NotFound {
            resource: "cortical_area".to_string(),
            id: cortical_id.to_string(),
        })
    }

    async fn get_all_cortical_area_properties(&self) -> ServiceResult<Vec<std::collections::HashMap<String, serde_json::Value>>> {
        Ok(crate::services::with_genome_or_default(&self.genome, |g| {
            g.cortical_areas.values().map(|area| area.properties.clone()).collect()
        }))
    }

    async fn create_brain_region(&self, params: CreateBrainRegionParams) -> ServiceResult<BrainRegionInfo> {
        let region_id = RegionID::from_string(&params.region_id)
            .map_err(|e| ServiceError::InvalidInput(format!("invalid region ID '{}': {}", params.region_id, e)))?;

        // The genome carries a single region type, so anything else is a caller mistake rather
        // than a value to coerce.
        if !params.region_type.eq_ignore_ascii_case("undefined") {
            return Err(ServiceError::InvalidInput(format!(
                "unknown region type '{}'; the genome format defines only 'undefined'",
                params.region_type
            )));
        }

        let mut region = BrainRegion::new(region_id, params.name, RegionType::Undefined).map_err(|e| ServiceError::InvalidInput(e.to_string()))?;

        for (key, value) in params.properties.unwrap_or_default() {
            region.add_property(key, value);
        }
        // The parent link lives in the child's properties; this is the same representation the
        // genome parser writes and the saver reads back.
        if let Some(parent_id) = params.parent_id {
            region.add_property("parent_region_id".to_string(), serde_json::Value::String(parent_id));
        }

        crate::services::with_genome_mut(&self.genome, "brain region creation", |g| {
            if g.brain_regions.contains_key(&params.region_id) {
                return Err(ServiceError::AlreadyExists {
                    resource: "brain_region".to_string(),
                    id: params.region_id.clone(),
                });
            }
            if let Some(parent_id) = region.properties.get("parent_region_id").and_then(|value| value.as_str()) {
                if !g.brain_regions.contains_key(parent_id) {
                    return Err(ServiceError::NotFound {
                        resource: "brain_region".to_string(),
                        id: parent_id.to_string(),
                    });
                }
            }

            g.brain_regions.insert(params.region_id.clone(), region);
            let created = &g.brain_regions[&params.region_id];
            Ok(Self::region_to_info(&params.region_id, created, &g.brain_regions))
        })?
    }

    async fn delete_brain_region(&self, region_id: &str) -> ServiceResult<()> {
        // Deleting a region deletes the cortical areas beneath it, and the engine has no operation
        // for removing an area. Removing the region from the genome alone would leave those areas
        // running in the NPU while no longer described anywhere, so this reports the gap instead.
        let has_areas = crate::services::with_genome(&self.genome, |g| {
            g.brain_regions.get(region_id).map(|region| !region.cortical_areas.is_empty())
        })?
        .ok_or_else(|| ServiceError::NotFound {
            resource: "brain_region".to_string(),
            id: region_id.to_string(),
        })?;

        if has_areas {
            return Err(ServiceError::NotImplemented(
                "deleting a region that holds cortical areas requires removing those areas from \
                 the engine, which the current NPU cannot do"
                    .to_string(),
            ));
        }

        crate::services::with_genome_mut(&self.genome, "brain region deletion", |g| {
            // A region cannot be removed while others still point at it as their parent.
            let orphans: Vec<String> = g
                .brain_regions
                .iter()
                .filter(|(_, candidate)| candidate.properties.get("parent_region_id").and_then(|value| value.as_str()) == Some(region_id))
                .map(|(child_id, _)| child_id.clone())
                .collect();

            if !orphans.is_empty() {
                return Err(ServiceError::Conflict(format!(
                    "region '{}' still has child regions: {}",
                    region_id,
                    orphans.join(", ")
                )));
            }

            g.brain_regions.remove(region_id);
            Ok(())
        })?
    }

    async fn update_brain_region(
        &self,
        region_id: &str,
        properties: std::collections::HashMap<String, serde_json::Value>,
    ) -> ServiceResult<BrainRegionInfo> {
        crate::services::with_genome_mut(&self.genome, "brain region update", |g| {
            if !g.brain_regions.contains_key(region_id) {
                return Err(ServiceError::NotFound {
                    resource: "brain_region".to_string(),
                    id: region_id.to_string(),
                });
            }

            // Reparenting is applied through the same property the hierarchy is read from, so it
            // must name a region that exists.
            if let Some(parent_id) = properties.get("parent_region_id").and_then(|value| value.as_str()) {
                if !g.brain_regions.contains_key(parent_id) {
                    return Err(ServiceError::NotFound {
                        resource: "brain_region".to_string(),
                        id: parent_id.to_string(),
                    });
                }
                if parent_id == region_id {
                    return Err(ServiceError::InvalidInput("a region cannot be its own parent".to_string()));
                }
            }

            let region = g.brain_regions.get_mut(region_id).expect("presence checked above");
            for (key, value) in properties {
                // `name` is a field rather than a property, so it is applied where readers look.
                if key == "name" || key == "title" {
                    if let Some(name) = value.as_str() {
                        region.name = name.to_string();
                        continue;
                    }
                }
                region.add_property(key, value);
            }

            let updated = &g.brain_regions[region_id];
            Ok(Self::region_to_info(region_id, updated, &g.brain_regions))
        })?
    }

    async fn get_brain_region(&self, region_id: &str) -> ServiceResult<BrainRegionInfo> {
        crate::services::with_genome(&self.genome, |g| {
            g.brain_regions
                .get(region_id)
                .map(|region| Self::region_to_info(region_id, region, &g.brain_regions))
        })?
        .ok_or_else(|| ServiceError::NotFound {
            resource: "brain_region".to_string(),
            id: region_id.to_string(),
        })
    }

    async fn list_brain_regions(&self) -> ServiceResult<Vec<BrainRegionInfo>> {
        Ok(crate::services::with_genome_or_default(&self.genome, |g| {
            g.brain_regions
                .iter()
                .map(|(region_id, region)| Self::region_to_info(region_id, region, &g.brain_regions))
                .collect()
        }))
    }

    async fn get_brain_region_ids(&self) -> ServiceResult<Vec<String>> {
        Ok(crate::services::with_genome_or_default(&self.genome, |g| {
            g.brain_regions.keys().cloned().collect()
        }))
    }

    async fn brain_region_exists(&self, region_id: &str) -> ServiceResult<bool> {
        Ok(crate::services::with_genome_or_default(&self.genome, |g| {
            g.brain_regions.contains_key(region_id)
        }))
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

    async fn get_morphologies(&self) -> ServiceResult<std::collections::HashMap<String, MorphologyInfo>> {
        // Written against the guard directly rather than through the shared helpers: the per
        // morphology conversion can fail, so the closure returns a `Result` and there is no
        // meaningful default to fall back to. No genome loaded means no morphologies, which is
        // the same empty answer the other listings give.
        let guard = self.genome.read();
        let Some(g) = guard.as_ref() else {
            return Ok(std::collections::HashMap::new());
        };

        g.morphologies
            .morphology_ids()
            .into_iter()
            .map(|morphology_id| {
                // `morphology_ids` returned this key, so the registry holds it.
                let morphology = g
                    .morphologies
                    .get(&morphology_id)
                    .ok_or_else(|| ServiceError::Internal(format!("morphology registry listed '{}' but cannot return it", morphology_id)))?;
                let parameters = serde_json::to_value(&morphology.parameters).map_err(|err| {
                    ServiceError::Internal(format!(
                        "morphology '{}' has parameters that cannot be serialised: {}",
                        morphology_id, err
                    ))
                })?;
                Ok((
                    morphology_id,
                    MorphologyInfo {
                        morphology_type: Self::morphology_type_name(&morphology.morphology_type).to_string(),
                        class: morphology.class.clone(),
                        parameters,
                    },
                ))
            })
            .collect()
    }

    async fn create_morphology(&self, morphology_id: String, morphology: feagi_evolutionary::Morphology) -> ServiceResult<()> {
        if morphology_id.trim().is_empty() {
            return Err(ServiceError::InvalidInput("morphology_id must be non-empty".to_string()));
        }

        crate::services::with_genome_mut(&self.genome, "morphology creation", |g| {
            if g.morphologies.contains(&morphology_id) {
                return Err(ServiceError::AlreadyExists {
                    resource: "morphology".to_string(),
                    id: morphology_id.clone(),
                });
            }
            g.morphologies.add_morphology(morphology_id, morphology);
            Ok(())
        })?
    }

    async fn update_morphology(&self, _morphology_id: String, _morphology: feagi_evolutionary::Morphology) -> ServiceResult<()> {
        // Redefining a morphology has to regenerate the synapses of every mapping that uses it.
        // The current engine builds connectivity from quantized mapping entries and offers no
        // regeneration operation, so writing the new definition to the genome alone would leave
        // the running brain wired to the old one.
        Err(ServiceError::NotImplemented(
            "changing a morphology requires regenerating the synapses of the mappings that use \
             it, which the current NPU cannot do"
                .to_string(),
        ))
    }

    async fn delete_morphology(&self, morphology_id: &str) -> ServiceResult<()> {
        if morphology_id.trim().is_empty() {
            return Err(ServiceError::InvalidInput("morphology_id must be non-empty".to_string()));
        }

        crate::services::with_genome_mut(&self.genome, "morphology deletion", |g| {
            if g.morphologies.remove_morphology(morphology_id) {
                Ok(())
            } else {
                Err(ServiceError::NotFound {
                    resource: "morphology".to_string(),
                    id: morphology_id.to_string(),
                })
            }
        })?
    }

    async fn rename_morphology(&self, old_id: &str, new_id: &str) -> ServiceResult<()> {
        let old_id = old_id.trim();
        let new_id = new_id.trim();

        if old_id.is_empty() {
            return Err(ServiceError::InvalidInput("old_id must be non-empty".to_string()));
        }
        if new_id.is_empty() {
            return Err(ServiceError::InvalidInput("new_id must be non-empty".to_string()));
        }
        if old_id == new_id {
            return Err(ServiceError::InvalidInput("old_id and new_id must differ".to_string()));
        }

        crate::services::with_genome_mut(&self.genome, "morphology rename", |g| {
            let morphology = g.morphologies.get(old_id).cloned().ok_or_else(|| ServiceError::NotFound {
                resource: "morphology".to_string(),
                id: old_id.to_string(),
            })?;

            if morphology.class == "core" {
                return Err(ServiceError::InvalidInput(format!("core morphologies cannot be renamed: '{}'", old_id)));
            }
            if g.morphologies.contains(new_id) {
                return Err(ServiceError::AlreadyExists {
                    resource: "morphology".to_string(),
                    id: new_id.to_string(),
                });
            }

            g.morphologies.remove_morphology(old_id);
            g.morphologies.add_morphology(new_id.to_string(), morphology);

            // Mapping rules name their morphology by id, so a rename that stopped at the registry
            // would leave those rules pointing at something that no longer exists.
            let mut replaced = 0usize;
            for area in g.cortical_areas.values_mut() {
                for value in area.properties.values_mut() {
                    rename_morphology_references(value, old_id, new_id, &mut replaced);
                }
            }
            for region in g.brain_regions.values_mut() {
                for value in region.properties.values_mut() {
                    rename_morphology_references(value, old_id, new_id, &mut replaced);
                }
            }

            Ok(())
        })?
    }

    async fn update_cortical_mapping(
        &self,
        _src_area_id: String,
        _dst_area_id: String,
        _mapping_data: Vec<serde_json::Value>,
    ) -> ServiceResult<usize> {
        Err(ServiceError::NotImplemented("genome-backed service is read-only".to_string()))
    }

    async fn get_neuron_properties(&self, _neuron_id: u64) -> ServiceResult<HashMap<String, serde_json::Value>> {
        Err(ServiceError::NotImplemented(
            "neuron properties require live neural state, which the genome does not carry".to_string(),
        ))
    }

    async fn export_connectome(&self) -> ServiceResult<ConnectomeSnapshot> {
        Err(ServiceError::NotImplemented(
            "connectome export requires live neuron and synapse state".to_string(),
        ))
    }

    async fn import_connectome(&self, _snapshot: ConnectomeSnapshot) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented("genome-backed service is read-only".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renames_morphology_references_in_every_rule_shape() {
        // A morphology is referenced as a named field, as a composite's mapper, and as the head of
        // a rule tuple. A rename that missed any of these would leave a dangling reference.
        let mut rules = serde_json::json!({
            "cortical_mapping_dst": {
                "cdst0001": [
                    { "morphology_id": "old", "scalar": [1, 1, 1] },
                    { "morphology_id": "untouched", "scalar": [1, 1, 1] }
                ]
            },
            "composite": { "mapper_morphology": "old" },
            "tuple_rule": ["old", 2, 3]
        });

        let mut replaced = 0;
        rename_morphology_references(&mut rules, "old", "new", &mut replaced);

        assert_eq!(replaced, 3, "all three reference shapes must be rewritten");
        assert_eq!(rules["cortical_mapping_dst"]["cdst0001"][0]["morphology_id"], "new");
        assert_eq!(
            rules["cortical_mapping_dst"]["cdst0001"][1]["morphology_id"], "untouched",
            "unrelated morphologies must be left alone"
        );
        assert_eq!(rules["composite"]["mapper_morphology"], "new");
        assert_eq!(rules["tuple_rule"][0], "new");
        assert_eq!(rules["tuple_rule"][1], 2, "non-id tuple members are untouched");
    }

    #[test]
    fn reconstructs_child_regions_from_the_parent_link() {
        // The genome records only a child's parent, so the child list is derived. This checks both
        // directions agree and that an unrelated region is not swept in.
        let mut regions = std::collections::HashMap::new();
        let parent_key = "parent".to_string();

        let make = |name: &str, parent: Option<&str>| {
            let mut region = BrainRegion::new(RegionID::new(), name.to_string(), RegionType::Undefined).expect("region is valid");
            if let Some(parent) = parent {
                region.add_property("parent_region_id".to_string(), serde_json::Value::String(parent.to_string()));
            }
            region
        };

        regions.insert(parent_key.clone(), make("Parent", None));
        regions.insert("child".to_string(), make("Child", Some("parent")));
        regions.insert("stranger".to_string(), make("Stranger", None));

        let parent_info = GenomeConnectomeService::region_to_info(&parent_key, &regions[&parent_key], &regions);
        assert_eq!(parent_info.child_regions, vec!["child".to_string()]);
        assert_eq!(parent_info.parent_id, None);

        let child_info = GenomeConnectomeService::region_to_info("child", &regions["child"], &regions);
        assert_eq!(child_info.parent_id, Some(parent_key));
        assert!(child_info.child_regions.is_empty());
    }
}
