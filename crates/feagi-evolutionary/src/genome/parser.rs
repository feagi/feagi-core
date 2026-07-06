// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Genome JSON parser.

Parses FEAGI 2.1 genome JSON format into runtime data structures.

## Genome Structure (v2.1)

```json
{
  "genome_id": "...",
  "genome_title": "...",
  "version": "2.1",
  "blueprint": {
    "cortical_id": {
      "cortical_name": "...",
      "block_boundaries": [x, y, z],
      "relative_coordinate": [x, y, z],
      "cortical_type": "IPU/OPU/CUSTOM/CORE/MEMORY",
      ...
    }
  },
  "brain_regions": {
    "root": {
      "title": "...",
      "parent_region_id": null,
      "coordinate_3d": [x, y, z],
      "areas": ["cortical_id1", ...],
      "regions": ["child_region_id1", ...]
    }
  },
  "neuron_morphologies": { ... },
  "physiology": { ... }
}
```

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::warn;

use crate::types::{EvoError, EvoResult};
use feagi_genome_definitions::::RegionID;
use feagi_genome_definitions::::CorticalID;
use feagi_genome_definitions::::{
    CorticalArea, CorticalAreaDimensions as Dimensions,
};
use feagi_genome_definitions::descriptors::GenomeCoordinate3D;
use feagi_genome_definitions::::brain_region::BrainRegion;
use feagi_genome_definitions::::region_type::RegionType;

/// Parsed genome data ready for ConnectomeManager
#[derive(Debug, Clone)]
pub struct ParsedGenome {
    /// Genome metadata
    pub genome_id: String,
    pub genome_title: String,
    pub version: String,

    /// Cortical areas extracted from blueprint
    pub cortical_areas: Vec<CorticalArea>,

    /// Brain regions and hierarchy
    pub brain_regions: Vec<(BrainRegion, Option<String>)>, // (region, parent_id)

    /// Raw neuron morphologies (for later processing)
    pub neuron_morphologies: HashMap<String, Value>,

    /// Raw physiology data (for later processing)
    pub physiology: Option<Value>,
}

/// Raw genome JSON structure for deserialization
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawGenome {
    pub genome_id: Option<String>,
    pub genome_title: Option<String>,
    pub genome_description: Option<String>,
    pub version: String,
    /// Integer schema version. Optional on the wire so older genomes that
    /// pre-date this field still deserialize. The authoritative resolver
    /// is `crate::genome::schema::detect_schema_version` and consumers
    /// MUST go through it instead of branching on this field directly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genome_schema_version: Option<u32>,
    pub blueprint: HashMap<String, RawCorticalArea>,
    #[serde(default)]
    pub brain_regions: HashMap<String, RawBrainRegion>,
    #[serde(default)]
    pub neuron_morphologies: HashMap<String, Value>,
    #[serde(default)]
    pub physiology: Option<Value>,
    /// Root brain region ID (UUID string) - for O(1) root lookup
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brain_regions_root: Option<String>,
}

/// Raw cortical_area area from blueprint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawCorticalArea {
    pub cortical_name: Option<String>,
    pub block_boundaries: Option<Vec<u32>>,
    pub relative_coordinate: Option<Vec<i32>>,
    pub cortical_type: Option<String>,

    // Optional properties
    pub group_id: Option<String>,
    pub sub_group_id: Option<String>,
    pub per_voxel_neuron_cnt: Option<u32>,
    pub cortical_mapping_dst: Option<Value>,

    // Neural properties
    pub synapse_attractivity: Option<f32>,
    pub refractory_period: Option<u32>,
    pub firing_threshold: Option<f32>,
    pub firing_threshold_limit: Option<f32>,
    pub firing_threshold_increment_x: Option<f32>,
    pub firing_threshold_increment_y: Option<f32>,
    pub firing_threshold_increment_z: Option<f32>,
    pub leak_coefficient: Option<f32>,
    pub leak_variability: Option<f32>,
    pub neuron_excitability: Option<f32>,
    pub postsynaptic_current: Option<f32>,
    pub postsynaptic_current_max: Option<f32>,
    pub degeneration: Option<f32>,
    pub psp_uniform_distribution: Option<bool>,
    pub mp_charge_accumulation: Option<bool>,
    pub mp_driven_psp: Option<bool>,
    pub visualization: Option<bool>,
    pub burst_engine_activation: Option<bool>,
    #[serde(rename = "2d_coordinate")]
    pub coordinate_2d: Option<Vec<i32>>,

    // Memory properties
    pub is_mem_type: Option<bool>,
    pub longterm_mem_threshold: Option<u32>,
    pub lifespan_growth_rate: Option<f32>,
    pub init_lifespan: Option<u32>,
    pub temporal_depth: Option<u32>,
    pub mp_learning_enabled: Option<bool>,
    pub consecutive_fire_cnt_max: Option<u32>,
    pub snooze_length: Option<u32>,

    // Allow any other properties (future-proofing)
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// Raw brain region from genome
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawBrainRegion {
    #[serde(alias = "name")]
    pub title: Option<String>,
    pub description: Option<String>,
    pub parent_region_id: Option<String>,
    pub coordinate_2d: Option<Vec<i32>>,
    pub coordinate_3d: Option<Vec<i32>>,
    #[serde(alias = "cortical_areas")]
    pub areas: Option<Vec<String>>,
    pub regions: Option<Vec<String>>,
    pub inputs: Option<Vec<String>>,
    pub outputs: Option<Vec<String>>,
    /// Declared interface lists (persisted from RuntimeGenome / PUT region).
    pub designated_inputs: Option<Vec<String>>,
    pub designated_outputs: Option<Vec<String>>,
    pub signature: Option<String>,
    /// v3 `serde_json::to_value(BrainRegion)` nests `inputs` / `designated_*` under `properties`.
    pub properties: Option<HashMap<String, Value>>,
}

/// Convert cortical_mapping_dst keys from old format to base64
///
/// This ensures all destination cortical_area IDs in dstmap are stored in the new base64 format.
fn convert_dstmap_keys_to_base64(dstmap: &Value) -> Value {
    if let Some(dstmap_obj) = dstmap.as_object() {
        let mut converted = serde_json::Map::new();

        for (dest_id_str, mapping_value) in dstmap_obj {
            // Convert destination cortical_id to base64 format
            match string_to_cortical_id(dest_id_str) {
                Ok(dest_cortical_id) => {
                    converted.insert(dest_cortical_id.as_base_64(), mapping_value.clone());
                }
                Err(e) => {
                    // If conversion fails, keep original and log warning
                    tracing::warn!(
                        "Failed to convert dstmap key '{}' to base64: {}, keeping original",
                        dest_id_str,
                        e
                    );
                    converted.insert(dest_id_str.clone(), mapping_value.clone());
                }
            }
        }

        Value::Object(converted)
    } else {
        // Not an object, return as-is
        dstmap.clone()
    }
}

/// Convert a string cortical_id to CorticalID
/// Handles both old 6-char format and new base64 format
/// CRITICAL: Uses feagi-data-processing types as single source of truth for core areas
pub fn string_to_cortical_id(id_str: &str) -> EvoResult<CorticalID> {
    use feagi_genome_definitions::::CoreCorticalType;

    // Try base64 first (new format)
    if let Ok(cortical_id) = CorticalID::try_from_base_64(id_str) {
        let mut bytes = [0u8; CorticalID::CORTICAL_ID_LENGTH];
        cortical_id.write_id_to_bytes(&mut bytes);
        if bytes == *b"___power" {
            return Ok(CoreCorticalType::Power.to_cortical_id());
        }
        if bytes == *b"___death" {
            return Ok(CoreCorticalType::Death.to_cortical_id());
        }
        if bytes == *b"___fatig" {
            return Ok(CoreCorticalType::Fatigue.to_cortical_id());
        }
        if bytes == *b"___pain_" {
            return Ok(CoreCorticalType::Pain.to_cortical_id());
        }
        if bytes == *b"___pleas" {
            return Ok(CoreCorticalType::Pleasure.to_cortical_id());
        }
        if bytes == *b"___fear_" {
            return Ok(CoreCorticalType::Fear.to_cortical_id());
        }
        if bytes == *b"___hope_" {
            return Ok(CoreCorticalType::Hope.to_cortical_id());
        }
        return Ok(cortical_id);
    }

    // Handle legacy CORE area names (6-char format) - use proper types from feagi-data-processing
    if id_str == "_power" {
        return Ok(CoreCorticalType::Power.to_cortical_id());
    }
    // Legacy shorthand used by older FEAGI genomes: "___pwr" (6-char) refers to core Power.
    if id_str == "___pwr" {
        return Ok(CoreCorticalType::Power.to_cortical_id());
    }
    // Legacy 8-char core names used in some BV caches
    if id_str == "___power" {
        return Ok(CoreCorticalType::Power.to_cortical_id());
    }
    // 8-char padded form of ___pwr (from 6-char padding in legacy flat genomes)
    if id_str == "___pwr__" {
        return Ok(CoreCorticalType::Power.to_cortical_id());
    }
    if id_str == "___death" {
        return Ok(CoreCorticalType::Death.to_cortical_id());
    }
    if id_str == "___fatig" {
        return Ok(CoreCorticalType::Fatigue.to_cortical_id());
    }
    if id_str == "___pain_" {
        return Ok(CoreCorticalType::Pain.to_cortical_id());
    }
    if id_str == "___pleas" {
        return Ok(CoreCorticalType::Pleasure.to_cortical_id());
    }
    if id_str == "___fear_" {
        return Ok(CoreCorticalType::Fear.to_cortical_id());
    }
    if id_str == "___hope_" {
        return Ok(CoreCorticalType::Hope.to_cortical_id());
    }
    if id_str == "_death" {
        return Ok(CoreCorticalType::Death.to_cortical_id());
    }
    if id_str == "_fatigue" {
        return Ok(CoreCorticalType::Fatigue.to_cortical_id());
    }
    if id_str == "_pain" {
        return Ok(CoreCorticalType::Pain.to_cortical_id());
    }
    if id_str == "_pleasure" {
        return Ok(CoreCorticalType::Pleasure.to_cortical_id());
    }
    if id_str == "_fear" {
        return Ok(CoreCorticalType::Fear.to_cortical_id());
    }
    if id_str == "_hope" {
        return Ok(CoreCorticalType::Hope.to_cortical_id());
    }

    // For non-core areas, use CorticalID's legacy ASCII parser (6-char and 8-char)
    if id_str.len() == 6 || id_str.len() == 8 {
        CorticalID::try_from_legacy_ascii(id_str).map_err(|e| {
            EvoError::InvalidArea(format!("Failed to convert cortical_id '{}': {}", id_str, e))
        })
    } else {
        Err(EvoError::InvalidArea(format!(
            "Invalid cortical_id length: '{}' (expected 6 or 8 ASCII chars, or base64)",
            id_str
        )))
    }
}

/// Genome parser
pub struct GenomeParser;

impl GenomeParser {
    /// Normalize cortical_area ID list properties (inputs, outputs, designated_*) to base64 strings.
    fn normalize_brain_region_cortical_id_list_properties(region: &mut BrainRegion, keys: &[&str]) {
        for key in keys {
            let Some(val) = region.get_property(key) else {
                continue;
            };
            let Some(arr) = val.as_array() else {
                continue;
            };
            let mut out: Vec<String> = Vec::new();
            for item in arr {
                let Some(s) = item.as_str() else {
                    continue;
                };
                match string_to_cortical_id(s) {
                    Ok(cortical_id) => out.push(cortical_id.as_base_64()),
                    Err(e) => {
                        warn!(target: "feagi-evo",
                            "Failed to convert brain region '{}' entry '{}': {}. Skipping.",
                            key, s, e);
                    }
                }
            }
            if out.is_empty() {
                region.properties.remove(*key);
            } else {
                region.add_property((*key).to_string(), serde_json::json!(out));
            }
        }
    }

    /// Parse a genome JSON string into a ParsedGenome
    ///
    /// # Arguments
    ///
    /// * `json_str` - JSON string of the genome
    ///
    /// # Returns
    ///
    /// Parsed genome ready for loading into ConnectomeManager
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - JSON is malformed
    /// - Required fields are missing
    /// - Data types are invalid
    ///
    pub fn parse(json_str: &str) -> EvoResult<ParsedGenome> {
        // Deserialize raw genome
        let raw: RawGenome = serde_json::from_str(json_str)
            .map_err(|e| EvoError::InvalidGenome(format!("Failed to parse JSON: {}", e)))?;

        // Validate version - support 2.x and 3.x (3.0 is flat format with base64 IDs)
        if !raw.version.starts_with("2.") && !raw.version.starts_with("3.") && raw.version != "3" {
            return Err(EvoError::InvalidGenome(format!(
                "Unsupported genome version: {}. Expected 2.x or 3.x",
                raw.version
            )));
        }

        // Parse cortical_area areas from blueprint
        let cortical_areas = Self::parse_cortical_areas(&raw.blueprint)?;

        // Parse brain regions
        let brain_regions = Self::parse_brain_regions(&raw.brain_regions)?;

        Ok(ParsedGenome {
            genome_id: raw.genome_id.unwrap_or_else(|| "unknown".to_string()),
            genome_title: raw.genome_title.unwrap_or_else(|| "Untitled".to_string()),
            version: raw.version,
            cortical_areas,
            brain_regions,
            neuron_morphologies: raw.neuron_morphologies,
            physiology: raw.physiology,
        })
    }

    /// Parse cortical_area areas from blueprint
    fn parse_cortical_areas(
        blueprint: &HashMap<String, RawCorticalArea>,
    ) -> EvoResult<Vec<CorticalArea>> {
        let mut areas = Vec::with_capacity(blueprint.len());

        for (cortical_id_str, raw_area) in blueprint.iter() {
            // Skip empty IDs
            if cortical_id_str.is_empty() {
                warn!(target: "feagi-evo","Skipping empty cortical_id");
                continue;
            }

            // Convert string cortical_id to CorticalID (handles 6-char legacy and base64)
            let cortical_id = match string_to_cortical_id(cortical_id_str) {
                Ok(id) => id,
                Err(e) => {
                    warn!(target: "feagi-evo","Skipping invalid cortical_id '{}': {}", cortical_id_str, e);
                    continue;
                }
            };

            // Extract required fields
            let name = raw_area
                .cortical_name
                .clone()
                .unwrap_or_else(|| cortical_id_str.clone());

            let dimensions = if let Some(boundaries) = &raw_area.block_boundaries {
                if boundaries.len() != 3 {
                    return Err(EvoError::InvalidArea(format!(
                        "Invalid block_boundaries for {}: expected 3 values, got {}",
                        cortical_id_str,
                        boundaries.len()
                    )));
                }
                Dimensions::new(boundaries[0], boundaries[1], boundaries[2])
                    .map_err(|e| EvoError::InvalidArea(format!("Invalid dimensions: {}", e)))?
            } else {
                // Default to 1x1x1 if not specified (should not happen in valid genomes)
                warn!(target: "feagi-evo","Cortical area {} missing block_boundaries, defaulting to 1x1x1", cortical_id_str);
                Dimensions::new(1, 1, 1).map_err(|e| {
                    EvoError::InvalidArea(format!("Invalid default dimensions: {}", e))
                })?
            };

            let position = if let Some(coords) = &raw_area.relative_coordinate {
                if coords.len() != 3 {
                    return Err(EvoError::InvalidArea(format!(
                        "Invalid relative_coordinate for {}: expected 3 values, got {}",
                        cortical_id_str,
                        coords.len()
                    )));
                }
                GenomeCoordinate3D::new(coords[0], coords[1], coords[2])
            } else {
                // Default to origin if not specified
                warn!(target: "feagi-evo","Cortical area {} missing relative_coordinate, defaulting to (0,0,0)", cortical_id_str);
                GenomeCoordinate3D::new(0, 0, 0)
            };

            // Determine cortical_area type from cortical_id
            let cortical_type = cortical_id.as_cortical_type().map_err(|e| {
                EvoError::InvalidArea(format!(
                    "Failed to determine cortical_area type from ID {}: {}",
                    cortical_id_str, e
                ))
            })?;

            // Create cortical_area area with CorticalID object (zero-copy, type-safe)
            let mut area = CorticalArea::new(
                cortical_id,
                0, // cortical_idx will be assigned by ConnectomeManager
                name,
                dimensions,
                position,
                cortical_type,
            )?;

            // Store cortical_type as cortical_group for new type system
            if let Some(ref cortical_type_str) = raw_area.cortical_type {
                area.properties.insert(
                    "cortical_group".to_string(),
                    serde_json::json!(cortical_type_str),
                );
            }

            // Store all properties in the properties HashMap
            // Neural properties
            if let Some(v) = raw_area.synapse_attractivity {
                area.properties
                    .insert("synapse_attractivity".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.refractory_period {
                area.properties
                    .insert("refractory_period".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.firing_threshold {
                area.properties
                    .insert("firing_threshold".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.firing_threshold_limit {
                area.properties
                    .insert("firing_threshold_limit".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.firing_threshold_increment_x {
                area.properties.insert(
                    "firing_threshold_increment_x".to_string(),
                    serde_json::json!(v),
                );
            }
            if let Some(v) = raw_area.firing_threshold_increment_y {
                area.properties.insert(
                    "firing_threshold_increment_y".to_string(),
                    serde_json::json!(v),
                );
            }
            if let Some(v) = raw_area.firing_threshold_increment_z {
                area.properties.insert(
                    "firing_threshold_increment_z".to_string(),
                    serde_json::json!(v),
                );
            }
            if let Some(v) = raw_area.leak_coefficient {
                area.properties
                    .insert("leak_coefficient".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.leak_variability {
                area.properties
                    .insert("leak_variability".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.neuron_excitability {
                area.properties
                    .insert("neuron_excitability".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.postsynaptic_current {
                area.properties
                    .insert("postsynaptic_current".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.postsynaptic_current_max {
                area.properties
                    .insert("postsynaptic_current_max".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.degeneration {
                area.properties
                    .insert("degeneration".to_string(), serde_json::json!(v));
            }

            // Boolean properties
            if let Some(v) = raw_area.psp_uniform_distribution {
                area.properties
                    .insert("psp_uniform_distribution".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.mp_charge_accumulation {
                area.properties
                    .insert("mp_charge_accumulation".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.mp_driven_psp {
                area.properties
                    .insert("mp_driven_psp".to_string(), serde_json::json!(v));
                tracing::info!(
                    target: "feagi-evo",
                    "[GENOME-LOAD] Loaded mp_driven_psp={} for area {}",
                    v,
                    cortical_id_str
                );
            } else {
                tracing::debug!(
                    target: "feagi-evo",
                    "[GENOME-LOAD] mp_driven_psp not found in raw_area for {}, will use default=false",
                    cortical_id_str
                );
            }
            if let Some(v) = raw_area.visualization {
                area.properties
                    .insert("visualization".to_string(), serde_json::json!(v));
                // Also store as "visible" for compatibility with getters
                area.properties
                    .insert("visible".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.burst_engine_activation {
                area.properties
                    .insert("burst_engine_active".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.is_mem_type {
                area.properties
                    .insert("is_mem_type".to_string(), serde_json::json!(v));
            }

            // Memory properties
            if let Some(v) = raw_area.longterm_mem_threshold {
                area.properties
                    .insert("longterm_mem_threshold".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.lifespan_growth_rate {
                area.properties
                    .insert("lifespan_growth_rate".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.init_lifespan {
                area.properties
                    .insert("init_lifespan".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.temporal_depth {
                area.properties
                    .insert("temporal_depth".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.mp_learning_enabled {
                area.properties
                    .insert("mp_learning_enabled".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.consecutive_fire_cnt_max {
                area.properties
                    .insert("consecutive_fire_cnt_max".to_string(), serde_json::json!(v));
                // Also store as "consecutive_fire_limit" for getter compatibility
                area.properties
                    .insert("consecutive_fire_limit".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.snooze_length {
                area.properties
                    .insert("snooze_period".to_string(), serde_json::json!(v));
            }

            // Other properties
            if let Some(v) = &raw_area.group_id {
                area.properties
                    .insert("group_id".to_string(), serde_json::json!(v));
            }
            if let Some(v) = &raw_area.sub_group_id {
                area.properties
                    .insert("sub_group_id".to_string(), serde_json::json!(v));
            }
            // Store neurons_per_voxel in properties HashMap
            if let Some(v) = raw_area.per_voxel_neuron_cnt {
                area.properties
                    .insert("neurons_per_voxel".to_string(), serde_json::json!(v));
            }
            if let Some(v) = &raw_area.cortical_mapping_dst {
                // Convert dstmap keys from old format to base64
                let converted_dstmap = convert_dstmap_keys_to_base64(v);
                area.properties
                    .insert("cortical_mapping_dst".to_string(), converted_dstmap);
            }
            if let Some(v) = &raw_area.coordinate_2d {
                area.properties
                    .insert("2d_coordinate".to_string(), serde_json::json!(v));
            }

            // Store any other custom properties
            for (key, value) in &raw_area.other {
                area.properties.insert(key.clone(), value.clone());
            }

            // Note: cortical_type parsing disabled - CorticalArea is now a minimal data structure
            // CorticalAreaType information is stored in properties["cortical_group"] if needed

            areas.push(area);
        }

        Ok(areas)
    }

    /// Parse brain regions
    fn parse_brain_regions(
        raw_regions: &HashMap<String, RawBrainRegion>,
    ) -> EvoResult<Vec<(BrainRegion, Option<String>)>> {
        let mut regions = Vec::with_capacity(raw_regions.len());

        for (region_id_str, raw_region) in raw_regions.iter() {
            let title = raw_region
                .title
                .clone()
                .unwrap_or_else(|| region_id_str.clone());

            // Convert string region_id to RegionID (UUID)
            // For now, try to parse as UUID if it's already a UUID, otherwise generate new one
            let region_id = match RegionID::from_string(region_id_str) {
                Ok(id) => id,
                Err(_) => {
                    // If not a valid UUID, generate a new one
                    // This handles legacy string-based region IDs
                    RegionID::new()
                }
            };

            let region_type = RegionType::Undefined; // Default to Undefined

            let mut region = BrainRegion::new(region_id, title, region_type)?;

            // v3 RuntimeGenome sections nest IO under `properties`; merge before list fields.
            if let Some(props) = &raw_region.properties {
                for (k, v) in props {
                    region.add_property(k.clone(), v.clone());
                }
            }

            // Add cortical_area areas to region (using CorticalID directly)
            if let Some(areas) = &raw_region.areas {
                for area_id in areas {
                    // Convert area_id to CorticalID
                    match string_to_cortical_id(area_id) {
                        Ok(cortical_id) => {
                            region.add_area(cortical_id);
                        }
                        Err(e) => {
                            warn!(target: "feagi-evo",
                                "Failed to convert brain region area ID '{}' to CorticalID: {}. Skipping.",
                                area_id, e);
                        }
                    }
                }
            }

            // Store properties in HashMap
            if let Some(desc) = &raw_region.description {
                region.add_property("description".to_string(), serde_json::json!(desc));
            }
            if let Some(coord_2d) = &raw_region.coordinate_2d {
                region.add_property("coordinate_2d".to_string(), serde_json::json!(coord_2d));
            }
            if let Some(coord_3d) = &raw_region.coordinate_3d {
                region.add_property("coordinate_3d".to_string(), serde_json::json!(coord_3d));
            }
            // Store inputs/outputs as base64 strings
            if let Some(inputs) = &raw_region.inputs {
                let input_ids: Vec<String> = inputs
                    .iter()
                    .filter_map(|id| match string_to_cortical_id(id) {
                        Ok(cortical_id) => Some(cortical_id.as_base_64()),
                        Err(e) => {
                            warn!(target: "feagi-evo",
                                    "Failed to convert brain region input ID '{}': {}. Skipping.",
                                    id, e);
                            None
                        }
                    })
                    .collect();
                if !input_ids.is_empty() {
                    region.add_property("inputs".to_string(), serde_json::json!(input_ids));
                }
            }
            if let Some(outputs) = &raw_region.outputs {
                let output_ids: Vec<String> = outputs
                    .iter()
                    .filter_map(|id| match string_to_cortical_id(id) {
                        Ok(cortical_id) => Some(cortical_id.as_base_64()),
                        Err(e) => {
                            warn!(target: "feagi-evo",
                                    "Failed to convert brain region output ID '{}': {}. Skipping.",
                                    id, e);
                            None
                        }
                    })
                    .collect();
                if !output_ids.is_empty() {
                    region.add_property("outputs".to_string(), serde_json::json!(output_ids));
                }
            }
            if let Some(signature) = &raw_region.signature {
                region.add_property("signature".to_string(), serde_json::json!(signature));
            }

            if let Some(d) = &raw_region.designated_inputs {
                let ids: Vec<String> = d
                    .iter()
                    .filter_map(|id| match string_to_cortical_id(id) {
                        Ok(cortical_id) => Some(cortical_id.as_base_64()),
                        Err(e) => {
                            warn!(target: "feagi-evo",
                                "Failed to convert designated_inputs entry '{}': {}. Skipping.",
                                id, e);
                            None
                        }
                    })
                    .collect();
                if !ids.is_empty() {
                    region.add_property("designated_inputs".to_string(), serde_json::json!(ids));
                }
            }
            if let Some(d) = &raw_region.designated_outputs {
                let ids: Vec<String> = d
                    .iter()
                    .filter_map(|id| match string_to_cortical_id(id) {
                        Ok(cortical_id) => Some(cortical_id.as_base_64()),
                        Err(e) => {
                            warn!(target: "feagi-evo",
                                "Failed to convert designated_outputs entry '{}': {}. Skipping.",
                                id, e);
                            None
                        }
                    })
                    .collect();
                if !ids.is_empty() {
                    region.add_property("designated_outputs".to_string(), serde_json::json!(ids));
                }
            }

            Self::normalize_brain_region_cortical_id_list_properties(
                &mut region,
                &[
                    "inputs",
                    "outputs",
                    "designated_inputs",
                    "designated_outputs",
                ],
            );

            // Store parent_id for hierarchy construction
            let parent_id = raw_region.parent_region_id.clone();
            if let Some(ref parent_id_str) = parent_id {
                // Store as property for serialization
                region.add_property(
                    "parent_region_id".to_string(),
                    serde_json::json!(parent_id_str),
                );
            }

            regions.push((region, parent_id));
        }

        Ok(regions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_genome() {
        // Test backward compatibility: parsing v2.1 genome with old 6-byte cortical_area ID
        // Parser should convert old format to base64 for storage
        let json = r#"{
            "version": "2.1",
            "blueprint": {
                "_power": {
                    "cortical_name": "Test Area",
                    "block_boundaries": [10, 10, 10],
                    "relative_coordinate": [0, 0, 0],
                    "cortical_type": "CORE"
                }
            },
            "brain_regions": {
                "root": {
                    "title": "Root",
                    "parent_region_id": null,
                    "areas": ["_power"]
                }
            }
        }"#;

        let parsed = GenomeParser::parse(json).unwrap();

        assert_eq!(parsed.version, "2.1");
        assert_eq!(parsed.cortical_areas.len(), 1);
        // Input was "_power" (6 bytes), converted to "___power" (8 bytes, padded at start with underscores) then base64 encoded
        assert_eq!(
            parsed.cortical_areas[0].cortical_id.as_base_64(),
            "X19fcG93ZXI="
        );
        assert_eq!(parsed.cortical_areas[0].name, "Test Area");
        assert_eq!(parsed.brain_regions.len(), 1);

        // Phase 2: Verify cortical_type_new is populated
        // Note: cortical_type_new field removed - type is encoded in cortical_id
        assert!(parsed.cortical_areas[0]
            .cortical_id
            .as_cortical_type()
            .is_ok());
    }

    #[test]
    fn test_parse_multiple_areas() {
        // Test parsing multiple cortical_area areas with old format IDs
        let json = r#"{
            "version": "2.1",
            "blueprint": {
                "_power": {
                    "cortical_name": "Area 1",
                    "cortical_type": "CORE",
                    "block_boundaries": [5, 5, 5],
                    "relative_coordinate": [0, 0, 0]
                },
                "_death": {
                    "cortical_name": "Area 2",
                    "cortical_type": "CORE",
                    "block_boundaries": [10, 10, 10],
                    "relative_coordinate": [5, 0, 0]
                }
            }
        }"#;

        let parsed = GenomeParser::parse(json).unwrap();

        assert_eq!(parsed.cortical_areas.len(), 2);

        // Phase 2: Verify both areas have cortical_type_new populated
        for area in &parsed.cortical_areas {
            assert!(
                area.cortical_id.as_cortical_type().is_ok(),
                "Area {} should have cortical_type_new populated",
                area.cortical_id
            );
        }
    }

    #[test]
    fn test_string_to_cortical_id_legacy_power_shorthand() {
        // Older FEAGI genomes may encode the power core area as "___pwr" (6-char shorthand).
        // Migration must map this deterministically to the core Power cortical_area ID.
        use feagi_genome_definitions::::CoreCorticalType;
        let id = string_to_cortical_id("___pwr").unwrap();
        assert_eq!(
            id.as_base_64(),
            CoreCorticalType::Power.to_cortical_id().as_base_64()
        );
    }

    #[test]
    fn test_string_to_cortical_id_legacy_power_padded() {
        // 8-char padded form ___pwr__ (from 6-char padding in legacy flat genomes).
        use feagi_genome_definitions::::CoreCorticalType;
        let id = string_to_cortical_id("___pwr__").unwrap();
        assert_eq!(
            id.as_base_64(),
            CoreCorticalType::Power.to_cortical_id().as_base_64()
        );
    }

    #[test]
    fn test_parse_with_properties() {
        let json = r#"{
            "version": "2.1",
            "blueprint": {
                "mem001": {
                    "cortical_name": "Memory Area",
                    "block_boundaries": [8, 8, 8],
                    "relative_coordinate": [0, 0, 0],
                    "cortical_type": "MEMORY",
                    "is_mem_type": true,
                    "firing_threshold": 50.0,
                    "leak_coefficient": 0.9
                }
            }
        }"#;

        let parsed = GenomeParser::parse(json).unwrap();

        assert_eq!(parsed.cortical_areas.len(), 1);
        let area = &parsed.cortical_areas[0];

        // Old type system (deprecated)
        use feagi_genome_definitions::::CorticalAreaType;
        assert!(matches!(area.cortical_type, CorticalAreaType::Memory(_)));

        // Properties stored correctly
        assert!(area.properties.contains_key("is_mem_type"));
        assert!(area.properties.contains_key("firing_threshold"));
        assert!(area.properties.contains_key("cortical_group"));

        // NEW: cortical_type should be derivable from cortical_id (Phase 2)
        assert!(
            area.cortical_id.as_cortical_type().is_ok(),
            "cortical_id should be parseable to cortical_type"
        );
        if let Ok(cortical_type) = area.cortical_id.as_cortical_type() {
            use feagi_genome_definitions::::CorticalAreaType;
            assert!(
                matches!(cortical_type, CorticalAreaType::Memory(_)),
                "Should be classified as MEMORY type"
            );
        }
    }

    /// v3 save embeds IO lists under `properties`; loading must preserve designated_inputs for BV presets.
    #[test]
    fn test_parse_v3_brain_region_nested_properties_retains_designated_io() {
        let json = r#"{
            "version": "3.0",
            "blueprint": {
                "_power": {
                    "cortical_name": "Core",
                    "block_boundaries": [10, 10, 10],
                    "relative_coordinate": [0, 0, 0],
                    "cortical_type": "CORE"
                }
            },
            "brain_regions": {
                "550e8400-e29b-41d4-a716-446655440000": {
                    "name": "Sub",
                    "cortical_areas": ["_power"],
                    "properties": {
                        "designated_inputs": ["_power"],
                        "designated_outputs": []
                    }
                }
            }
        }"#;

        let parsed = GenomeParser::parse(json).unwrap();
        assert_eq!(parsed.brain_regions.len(), 1);
        let (region, _) = &parsed.brain_regions[0];
        let di = region
            .get_property("designated_inputs")
            .and_then(|v| v.as_array())
            .expect("designated_inputs");
        assert_eq!(di.len(), 1);
        assert_eq!(di[0].as_str().unwrap(), "X19fcG93ZXI=");
    }

    #[test]
    fn test_invalid_version() {
        let json = r#"{
            "version": "1.0",
            "blueprint": {}
        }"#;

        let result = GenomeParser::parse(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_json() {
        let json = r#"{ "version": "2.1", "blueprint": { malformed"#;

        let result = GenomeParser::parse(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_cortical_type_new_population() {
        // Test that cortical_type_new field is populated during parsing (Phase 2)
        // This tests that parsing works with valid cortical_area IDs and populates types correctly
        use feagi_genome_definitions::::CoreCorticalType;
        let power_id = CoreCorticalType::Power.to_cortical_id().as_base_64();
        let json = format!(
            r#"{{
            "version": "2.1",
            "blueprint": {{
                "cvision1": {{
                    "cortical_name": "Test Custom Vision",
                    "cortical_type": "CUSTOM",
                    "block_boundaries": [10, 10, 1],
                    "relative_coordinate": [0, 0, 0]
                }},
                "cmotor01": {{
                    "cortical_name": "Test Custom Motor",
                    "cortical_type": "CUSTOM",
                    "block_boundaries": [5, 5, 1],
                    "relative_coordinate": [0, 0, 0]
                }},
                "{}": {{
                    "cortical_name": "Test Core",
                    "cortical_type": "CORE",
                    "block_boundaries": [1, 1, 1],
                    "relative_coordinate": [0, 0, 0]
                }}
            }}
        }}"#,
            power_id
        );

        let parsed = GenomeParser::parse(&json).unwrap();
        assert_eq!(parsed.cortical_areas.len(), 3);

        // Verify all areas have cortical_type_new populated
        for area in &parsed.cortical_areas {
            assert!(
                area.cortical_id.as_cortical_type().is_ok(),
                "Area {} should have cortical_type_new populated",
                area.cortical_id
            );

            // Verify cortical_group property is also set
            assert!(
                area.properties.contains_key("cortical_group"),
                "Area {} should have cortical_group property",
                area.cortical_id
            );

            // Verify cortical_area group is consistent (avoid depending on feagi-brain-development)
            if let Some(prop_group) = area
                .properties
                .get("cortical_group")
                .and_then(|v| v.as_str())
            {
                assert!(
                    !prop_group.is_empty(),
                    "Area {} should have non-empty cortical_group property",
                    area.cortical_id.as_base_64()
                );
            }
        }
    }
}
