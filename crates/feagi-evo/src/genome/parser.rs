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
use feagi_types::{BrainRegion, CorticalArea, AreaType, RegionType, Dimensions};
use feagi_data_structures::genomic::cortical_area::CorticalID;

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
    pub blueprint: HashMap<String, RawCorticalArea>,
    #[serde(default)]
    pub brain_regions: HashMap<String, RawBrainRegion>,
    #[serde(default)]
    pub neuron_morphologies: HashMap<String, Value>,
    #[serde(default)]
    pub physiology: Option<Value>,
}

/// Raw cortical area from blueprint
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
    pub leak_coefficient: Option<f32>,
    pub neuron_excitability: Option<f32>,
    pub postsynaptic_current: Option<f32>,
    pub postsynaptic_current_max: Option<f32>,
    pub degeneration: Option<f32>,
    pub psp_uniform_distribution: Option<bool>,
    pub mp_charge_accumulation: Option<bool>,
    pub mp_driven_psp: Option<bool>,
    pub visualization: Option<bool>,
    #[serde(rename = "2d_coordinate")]
    pub coordinate_2d: Option<Vec<i32>>,
    
    // Memory properties
    pub is_mem_type: Option<bool>,
    pub longterm_mem_threshold: Option<u32>,
    pub lifespan_growth_rate: Option<f32>,
    pub init_lifespan: Option<u32>,
    pub temporal_depth: Option<u32>,
    pub consecutive_fire_cnt_max: Option<u32>,
    pub snooze_length: Option<u32>,
    
    // Allow any other properties (future-proofing)
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// Raw brain region from genome
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawBrainRegion {
    pub title: Option<String>,
    pub description: Option<String>,
    pub parent_region_id: Option<String>,
    pub coordinate_2d: Option<Vec<i32>>,
    pub coordinate_3d: Option<Vec<i32>>,
    pub areas: Option<Vec<String>>,
    pub regions: Option<Vec<String>>,
    pub inputs: Option<Vec<String>>,
    pub outputs: Option<Vec<String>>,
    pub signature: Option<String>,
}

/// Convert cortical_mapping_dst keys from old format to base64
///
/// This ensures all destination cortical IDs in dstmap are stored in the new base64 format.
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
                    tracing::warn!("Failed to convert dstmap key '{}' to base64: {}, keeping original", dest_id_str, e);
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
pub fn string_to_cortical_id(id_str: &str) -> EvoResult<CorticalID> {
    // Try base64 first (new format)
    if let Ok(cortical_id) = CorticalID::try_from_base_64(id_str) {
        return Ok(cortical_id);
    }
    
    // Fall back to ASCII (old 6-char format → pad to 8 bytes with underscores)
    if id_str.len() == 6 {
        let mut bytes = [b'_'; 8];  // Pad with underscores
        bytes[..6].copy_from_slice(id_str.as_bytes());
        
        CorticalID::try_from_bytes(&bytes)
            .map_err(|e| EvoError::InvalidArea(format!("Failed to convert cortical_id '{}': {}", id_str, e)))
    } else if id_str.len() == 8 {
        // Already 8 bytes - convert directly
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(id_str.as_bytes());
        
        CorticalID::try_from_bytes(&bytes)
            .map_err(|e| EvoError::InvalidArea(format!("Failed to convert cortical_id '{}': {}", id_str, e)))
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
        
        // Validate version
        if !raw.version.starts_with("2.") {
            return Err(EvoError::InvalidGenome(format!(
                "Unsupported genome version: {}. Expected 2.x",
                raw.version
            )));
        }
        
        // Parse cortical areas from blueprint
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
    
    /// Parse cortical areas from blueprint
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
            let name = raw_area.cortical_name.clone()
                .unwrap_or_else(|| cortical_id_str.clone());
            
            let dimensions = if let Some(boundaries) = &raw_area.block_boundaries {
                if boundaries.len() != 3 {
                    return Err(EvoError::InvalidArea(format!(
                        "Invalid block_boundaries for {}: expected 3 values, got {}",
                        cortical_id_str, boundaries.len()
                    )));
                }
                Dimensions::new(
                    boundaries[0] as usize, 
                    boundaries[1] as usize, 
                    boundaries[2] as usize
                )
            } else {
                // Default to 1x1x1 if not specified (should not happen in valid genomes)
                warn!(target: "feagi-evo","Cortical area {} missing block_boundaries, defaulting to 1x1x1", cortical_id_str);
                Dimensions::new(1, 1, 1)
            };
            
            let position = if let Some(coords) = &raw_area.relative_coordinate {
                if coords.len() != 3 {
                    return Err(EvoError::InvalidArea(format!(
                        "Invalid relative_coordinate for {}: expected 3 values, got {}",
                        cortical_id_str, coords.len()
                    )));
                }
                (coords[0], coords[1], coords[2])
            } else {
                // Default to origin if not specified
                warn!(target: "feagi-evo","Cortical area {} missing relative_coordinate, defaulting to (0,0,0)", cortical_id_str);
                (0, 0, 0)
            };
            
            // Parse area type
            let area_type = Self::parse_area_type(raw_area.cortical_type.as_deref())?;
            
            // Create cortical area with normalized base64 format
            let mut area = CorticalArea::new(
                cortical_id.as_base_64(),
                0, // cortical_idx will be assigned by ConnectomeManager
                name,
                dimensions,
                position,
                area_type,
            )?;
            
            // Store all properties in the properties HashMap
            // Neural properties
            if let Some(v) = raw_area.synapse_attractivity {
                area.properties.insert("synapse_attractivity".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.refractory_period {
                area.properties.insert("refractory_period".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.firing_threshold {
                area.properties.insert("firing_threshold".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.leak_coefficient {
                area.properties.insert("leak_coefficient".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.neuron_excitability {
                area.properties.insert("neuron_excitability".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.postsynaptic_current {
                area.properties.insert("postsynaptic_current".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.postsynaptic_current_max {
                area.properties.insert("postsynaptic_current_max".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.degeneration {
                area.properties.insert("degeneration".to_string(), serde_json::json!(v));
            }
            
            // Boolean properties
            if let Some(v) = raw_area.psp_uniform_distribution {
                area.properties.insert("psp_uniform_distribution".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.mp_charge_accumulation {
                area.properties.insert("mp_charge_accumulation".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.mp_driven_psp {
                area.properties.insert("mp_driven_psp".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.visualization {
                area.properties.insert("visualization".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.is_mem_type {
                area.properties.insert("is_mem_type".to_string(), serde_json::json!(v));
            }
            
            // Memory properties
            if let Some(v) = raw_area.longterm_mem_threshold {
                area.properties.insert("longterm_mem_threshold".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.lifespan_growth_rate {
                area.properties.insert("lifespan_growth_rate".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.init_lifespan {
                area.properties.insert("init_lifespan".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.temporal_depth {
                area.properties.insert("temporal_depth".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.consecutive_fire_cnt_max {
                area.properties.insert("consecutive_fire_cnt_max".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.snooze_length {
                area.properties.insert("snooze_length".to_string(), serde_json::json!(v));
            }
            
            // Other properties
            if let Some(v) = &raw_area.group_id {
                area.properties.insert("group_id".to_string(), serde_json::json!(v));
            }
            if let Some(v) = &raw_area.sub_group_id {
                area.properties.insert("sub_group_id".to_string(), serde_json::json!(v));
            }
            if let Some(v) = raw_area.per_voxel_neuron_cnt {
                area.properties.insert("per_voxel_neuron_cnt".to_string(), serde_json::json!(v));
            }
            if let Some(v) = &raw_area.cortical_mapping_dst {
                // Convert dstmap keys from old format to base64
                let converted_dstmap = convert_dstmap_keys_to_base64(v);
                area.properties.insert("cortical_mapping_dst".to_string(), converted_dstmap);
            }
            if let Some(v) = &raw_area.coordinate_2d {
                area.properties.insert("2d_coordinate".to_string(), serde_json::json!(v));
            }
            
            // Store any other custom properties
            for (key, value) in &raw_area.other {
                area.properties.insert(key.clone(), value.clone());
            }
            
            areas.push(area);
        }
        
        Ok(areas)
    }
    
    /// Parse brain regions
    fn parse_brain_regions(
        raw_regions: &HashMap<String, RawBrainRegion>,
    ) -> EvoResult<Vec<(BrainRegion, Option<String>)>> {
        let mut regions = Vec::with_capacity(raw_regions.len());
        
        for (region_id, raw_region) in raw_regions.iter() {
            let title = raw_region.title.clone()
                .unwrap_or_else(|| region_id.clone());
            
            let region_type = RegionType::Custom; // Default to Custom
            
            let mut region = BrainRegion::new(
                region_id.clone(),
                title,
                region_type,
            )?;
            
            // Add cortical areas to region
            if let Some(areas) = &raw_region.areas {
                for area_id in areas {
                    region.add_area(area_id.clone());
                }
            }
            
            // Store properties
            if let Some(desc) = &raw_region.description {
                region.properties.insert("description".to_string(), serde_json::json!(desc));
            }
            if let Some(coord_2d) = &raw_region.coordinate_2d {
                region.properties.insert("coordinate_2d".to_string(), serde_json::json!(coord_2d));
            }
            if let Some(coord_3d) = &raw_region.coordinate_3d {
                region.properties.insert("coordinate_3d".to_string(), serde_json::json!(coord_3d));
            }
            if let Some(inputs) = &raw_region.inputs {
                region.properties.insert("inputs".to_string(), serde_json::json!(inputs));
            }
            if let Some(outputs) = &raw_region.outputs {
                region.properties.insert("outputs".to_string(), serde_json::json!(outputs));
            }
            if let Some(signature) = &raw_region.signature {
                region.properties.insert("signature".to_string(), serde_json::json!(signature));
            }
            
            // Store parent_id for hierarchy construction
            let parent_id = raw_region.parent_region_id.clone();
            
            regions.push((region, parent_id));
        }
        
        Ok(regions)
    }
    
    /// Parse area type string to AreaType enum
    fn parse_area_type(type_str: Option<&str>) -> EvoResult<AreaType> {
        match type_str {
            Some("IPU") => Ok(AreaType::Sensory),
            Some("OPU") => Ok(AreaType::Motor),
            Some("MEMORY") => Ok(AreaType::Memory),
            Some("CORE") => Ok(AreaType::Custom), // CORE maps to Custom for now
            Some("CUSTOM") | None => Ok(AreaType::Custom),
            Some(other) => {
                warn!(target: "feagi-evo","Unknown cortical_type '{}', defaulting to Custom", other);
                Ok(AreaType::Custom)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_minimal_genome() {
        let json = r#"{
            "version": "2.1",
            "blueprint": {
                "_power": {
                    "cortical_name": "Test Area",
                    "block_boundaries": [10, 10, 10],
                    "relative_coordinate": [0, 0, 0],
                    "cortical_type": "IPU"
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
        // cortical_id is now stored in base64 format (underscore-padded)
        assert_eq!(parsed.cortical_areas[0].cortical_id, "X3Bvd2VyX18=");
        assert_eq!(parsed.cortical_areas[0].name, "Test Area");
        assert_eq!(parsed.brain_regions.len(), 1);
    }
    
    #[test]
    fn test_parse_multiple_areas() {
        let json = r#"{
            "version": "2.1",
            "blueprint": {
                "_power": {
                    "cortical_name": "Area 1",
                    "block_boundaries": [5, 5, 5],
                    "relative_coordinate": [0, 0, 0]
                },
                "_death": {
                    "cortical_name": "Area 2",
                    "block_boundaries": [10, 10, 10],
                    "relative_coordinate": [5, 0, 0]
                }
            }
        }"#;
        
        let parsed = GenomeParser::parse(json).unwrap();
        
        assert_eq!(parsed.cortical_areas.len(), 2);
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
        assert_eq!(area.area_type, AreaType::Memory);
        assert!(area.properties.contains_key("is_mem_type"));
        assert!(area.properties.contains_key("firing_threshold"));
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
}

