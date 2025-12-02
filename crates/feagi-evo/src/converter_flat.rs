// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Convert flat genome format (2.0) to hierarchical format.

The flat format uses keys like "_____10c-AREA1-cx-dstmap-d" while the
hierarchical format uses nested JSON objects.

This is a complex conversion with many edge cases. For production use,
consider using the Python converter for legacy genomes.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::collections::HashSet;
use serde_json::{json, Value};
use crate::{EvoResult, EvoError};

/// Convert flat genome (2.0 format) to hierarchical format
///
/// **Note**: This is a simplified implementation. For production use with legacy genomes,
/// use the Python `genome_2_1_convertor` which handles all edge cases.
///
/// Supported conversions:
/// - Cortical area basic properties (name, dimensions, coordinates)
/// - Block boundaries → cortical_dimensions
/// - Relative coordinates → position
/// - Cortical type
///
/// **Not yet supported**:
/// - Complex cortical mappings (dstmap)
/// - All neural parameters
/// - 2D coordinates
/// - Memory-specific properties
///
pub fn convert_flat_to_hierarchical(flat_genome: &Value) -> EvoResult<Value> {
    // Extract flat blueprint section
    let flat_blueprint = if let Some(bp) = flat_genome.get("blueprint") {
        bp.as_object().ok_or_else(|| EvoError::InvalidGenome(
            "Flat genome blueprint must be an object".to_string()
        ))?
    } else {
        return Err(EvoError::InvalidGenome(
            "Flat genome missing blueprint section".to_string()
        ));
    };
    
    // Find all cortical areas
    let cortical_areas = extract_cortical_areas(flat_blueprint)?;
    
    // Build hierarchical blueprint
    let mut hierarchical_blueprint = serde_json::Map::new();
    
    for cortical_id in &cortical_areas {
        let area_data = build_hierarchical_area(cortical_id, flat_blueprint)?;
        hierarchical_blueprint.insert(cortical_id.clone(), area_data);
    }
    
    // Build complete hierarchical genome
    let mut hierarchical = serde_json::Map::new();
    hierarchical.insert("blueprint".to_string(), Value::Object(hierarchical_blueprint));
    
    // Copy other sections as-is
    if let Some(morphologies) = flat_genome.get("neuron_morphologies") {
        hierarchical.insert("neuron_morphologies".to_string(), morphologies.clone());
    }
    
    if let Some(physiology) = flat_genome.get("physiology") {
        hierarchical.insert("physiology".to_string(), physiology.clone());
    } else {
        hierarchical.insert("physiology".to_string(), json!({}));
    }
    
    if let Some(stats) = flat_genome.get("stats") {
        hierarchical.insert("stats".to_string(), stats.clone());
    }
    
    if let Some(signatures) = flat_genome.get("signatures") {
        hierarchical.insert("signatures".to_string(), signatures.clone());
    }
    
    // Copy metadata fields
    if let Some(id) = flat_genome.get("genome_id") {
        hierarchical.insert("genome_id".to_string(), id.clone());
    }
    if let Some(title) = flat_genome.get("genome_title") {
        hierarchical.insert("genome_title".to_string(), title.clone());
    }
    if let Some(version) = flat_genome.get("version") {
        hierarchical.insert("version".to_string(), version.clone());
    }
    if let Some(timestamp) = flat_genome.get("timestamp") {
        hierarchical.insert("timestamp".to_string(), timestamp.clone());
    }
    
    hierarchical.insert("brain_regions".to_string(), json!({}));
    
    Ok(Value::Object(hierarchical))
}

/// Extract all cortical area IDs from flat blueprint
fn extract_cortical_areas(flat_blueprint: &serde_json::Map<String, Value>) -> EvoResult<HashSet<String>> {
    let mut areas = HashSet::new();
    
    for key in flat_blueprint.keys() {
        if let Some(cortical_id) = parse_cortical_id(key) {
            areas.insert(cortical_id);
        }
    }
    
    Ok(areas)
}

/// Parse cortical ID from flat key
/// Format: "_____10c-AREA1-cx-property-type"
fn parse_cortical_id(key: &str) -> Option<String> {
    if !key.starts_with("_____10c-") {
        return None;
    }
    
    let parts: Vec<&str> = key.split('-').collect();
    if parts.len() >= 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// Build hierarchical area data from flat keys
fn build_hierarchical_area(
    cortical_id: &str,
    flat_blueprint: &serde_json::Map<String, Value>,
) -> EvoResult<Value> {
    let mut area = serde_json::Map::new();
    
    // Default values
    let mut dimensions = [1, 1, 1];
    let mut position = [0, 0, 0];
    let mut cortical_name = cortical_id.to_string();
    let mut cortical_type = "CUSTOM";
    
    // Scan for relevant keys
    for (key, value) in flat_blueprint.iter() {
        if let Some(area_id) = parse_cortical_id(key) {
            if area_id != cortical_id {
                continue;
            }
            
            // Parse property type
            if key.contains("-__name-t") {
                if let Some(s) = value.as_str() {
                    cortical_name = s.to_string();
                }
            } else if key.contains("-_group-t") {
                if let Some(s) = value.as_str() {
                    cortical_type = s;
                }
            } else if key.contains("-___bbx-i") {
                if let Some(n) = value.as_i64() {
                    dimensions[0] = n as usize;
                }
            } else if key.contains("-___bby-i") {
                if let Some(n) = value.as_i64() {
                    dimensions[1] = n as usize;
                }
            } else if key.contains("-___bbz-i") {
                if let Some(n) = value.as_i64() {
                    dimensions[2] = n as usize;
                }
            } else if key.contains("-rcordx-i") {
                if let Some(n) = value.as_i64() {
                    position[0] = n as i32;
                }
            } else if key.contains("-rcordy-i") {
                if let Some(n) = value.as_i64() {
                    position[1] = n as i32;
                }
            } else if key.contains("-rcordz-i") {
                if let Some(n) = value.as_i64() {
                    position[2] = n as i32;
                }
            }
        }
    }
    
    // Build hierarchical structure
    area.insert("cortical_name".to_string(), json!(cortical_name));
    area.insert("block_boundaries".to_string(), json!(dimensions));
    area.insert("relative_coordinate".to_string(), json!(position));
    area.insert("cortical_type".to_string(), json!(cortical_type));
    
    Ok(Value::Object(area))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_cortical_id() {
        assert_eq!(parse_cortical_id("_____10c-test01-cx-__name-t"), Some("test01".to_string()));
        assert_eq!(parse_cortical_id("_____10c-abc123-nx-fire_t-f"), Some("abc123".to_string()));
        assert_eq!(parse_cortical_id("invalid_key"), None);
    }
    
    #[test]
    fn test_convert_minimal_flat_genome() {
        let flat = json!({
            "genome_id": "test",
            "genome_title": "Test",
            "version": "2.0",
            "blueprint": {
                "_____10c-test01-cx-__name-t": "Test Area",
                "_____10c-test01-cx-___bbx-i": 10,
                "_____10c-test01-cx-___bby-i": 10,
                "_____10c-test01-cx-___bbz-i": 10,
                "_____10c-test01-cx-rcordx-i": 0,
                "_____10c-test01-cx-rcordy-i": 0,
                "_____10c-test01-cx-rcordz-i": 0,
                "_____10c-test01-cx-_group-t": "CUSTOM"
            },
            "neuron_morphologies": {},
            "physiology": {},
        });
        
        let result = convert_flat_to_hierarchical(&flat).unwrap();
        
        // Verify blueprint was converted
        let blueprint = result.get("blueprint").unwrap().as_object().unwrap();
        assert!(blueprint.contains_key("test01"));
        
        let area = blueprint.get("test01").unwrap().as_object().unwrap();
        assert_eq!(area.get("cortical_name").unwrap(), "Test Area");
        assert_eq!(area.get("cortical_type").unwrap(), "CUSTOM");
        
        let dims = area.get("block_boundaries").unwrap().as_array().unwrap();
        assert_eq!(dims.len(), 3);
        assert_eq!(dims[0], 10);
    }
}

