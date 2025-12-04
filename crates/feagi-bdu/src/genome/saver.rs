// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Genome JSON saver.

Serializes FEAGI connectome data back to JSON genome format.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use serde_json::{json, Value};
use std::collections::HashMap;

use crate::models::CorticalArea;
use feagi_data_structures::genomic::BrainRegion;
use crate::types::{BduError, BduResult};

/// Genome saver
pub struct GenomeSaver;

impl GenomeSaver {
    /// Save connectome to genome JSON
    ///
    /// # Arguments
    ///
    /// * `cortical_areas` - Map of cortical areas
    /// * `brain_regions` - Map of brain regions
    /// * `genome_id` - Optional genome ID (generates default if None)
    /// * `genome_title` - Optional genome title
    ///
    /// # Returns
    ///
    /// JSON string of the genome
    ///
    pub fn save_to_json(
        cortical_areas: &HashMap<String, CorticalArea>,
        brain_regions: &HashMap<String, (BrainRegion, Option<String>)>,
        genome_id: Option<String>,
        genome_title: Option<String>,
    ) -> BduResult<String> {
        // Build blueprint section
        let mut blueprint = serde_json::Map::new();
        
        for (cortical_id, area) in cortical_areas {
            let mut area_data = serde_json::Map::new();
            
            // Required fields
            area_data.insert("cortical_name".to_string(), json!(area.name));
            area_data.insert("block_boundaries".to_string(), json!([
                area.dimensions.width,
                area.dimensions.height,
                area.dimensions.depth
            ]));
            area_data.insert("relative_coordinate".to_string(), json!([
                area.position.0,
                area.position.1,
                area.position.2
            ]));
            
            // Area type
            let cortical_type = match area.area_type {
                crate::models::cortical_area::AreaType::Sensory => "IPU",
                crate::models::cortical_area::AreaType::Motor => "OPU",
                crate::models::cortical_area::AreaType::Memory => "MEMORY",
                crate::models::cortical_area::AreaType::Custom => "CUSTOM",
            };
            area_data.insert("cortical_type".to_string(), json!(cortical_type));
            
            // Add all properties from the area's properties HashMap
            for (key, value) in &area.properties {
                area_data.insert(key.clone(), value.clone());
            }
            
            blueprint.insert(cortical_id.clone(), Value::Object(area_data));
        }
        
        // Build brain_regions section
        let mut regions_map = serde_json::Map::new();
        
        for (region_id, (region, parent_id)) in brain_regions {
            let mut region_data = serde_json::Map::new();
            
            region_data.insert("title".to_string(), json!(region.name));
            region_data.insert("parent_region_id".to_string(), 
                if let Some(ref parent) = parent_id {
                    json!(parent)
                } else {
                    Value::Null
                }
            );
            
            // Cortical areas in this region
            let areas: Vec<String> = region.cortical_areas.iter().cloned().collect();
            region_data.insert("areas".to_string(), json!(areas));
            
            // Child regions (empty for now - can be computed from hierarchy)
            region_data.insert("regions".to_string(), json!(Vec::<String>::new()));
            
            // Add all properties from the region's properties HashMap
            for (key, value) in &region.properties {
                region_data.insert(key.clone(), value.clone());
            }
            
            regions_map.insert(region_id.clone(), Value::Object(region_data));
        }
        
        // Build final genome structure
        let genome = json!({
            "genome_id": genome_id.unwrap_or_else(|| 
                format!("genome_{}", chrono::Utc::now().timestamp())
            ),
            "genome_title": genome_title.unwrap_or_else(|| "Exported Genome".to_string()),
            "version": "2.1",
            "blueprint": blueprint,
            "brain_regions": regions_map,
            "neuron_morphologies": {},
            "physiology": {}
        });
        
        // Serialize to pretty JSON
        serde_json::to_string_pretty(&genome)
            .map_err(|e| BduError::Internal(format!("Failed to serialize genome: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::cortical_area::AreaType;
    use feagi_data_structures::genomic::RegionType;
    use crate::types::Dimensions;
    
    #[test]
    fn test_save_minimal_genome() {
        let mut cortical_areas = HashMap::new();
        let mut brain_regions = HashMap::new();
        
        // Create a test cortical area
        let area = CorticalArea::new(
            "test01".to_string(),
            0,
            "Test Area".to_string(),
            Dimensions::new(10, 10, 10),
            (0, 0, 0),
            AreaType::Sensory,
        ).unwrap();
        
        cortical_areas.insert("test01".to_string(), area);
        
        // Create a test brain region
        let region = BrainRegion::new(
            "root".to_string(),
            "Root".to_string(),
            RegionType::Custom,
        ).unwrap();
        
        brain_regions.insert("root".to_string(), (region, None));
        
        // Save to JSON
        let json = GenomeSaver::save_to_json(
            &cortical_areas,
            &brain_regions,
            Some("test-001".to_string()),
            Some("Test Genome".to_string()),
        ).unwrap();
        
        // Verify it's valid JSON
        let parsed: Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["genome_id"], "test-001");
        assert_eq!(parsed["genome_title"], "Test Genome");
        assert_eq!(parsed["version"], "2.1");
        assert!(parsed["blueprint"].is_object());
        assert!(parsed["brain_regions"].is_object());
    }
    
    #[test]
    fn test_roundtrip() {
        use crate::genome::GenomeParser;
        
        // Create test data
        let mut cortical_areas = HashMap::new();
        let area = CorticalArea::new(
            "test01".to_string(),
            0,
            "Test Area".to_string(),
            Dimensions::new(10, 10, 10),
            (5, 5, 5),
            AreaType::Motor,
        ).unwrap();
        cortical_areas.insert("test01".to_string(), area);
        
        let mut brain_regions = HashMap::new();
        let region = BrainRegion::new(
            "root".to_string(),
            "Root Region".to_string(),
            RegionType::Custom,
        ).unwrap();
        brain_regions.insert("root".to_string(), (region, None));
        
        // Save to JSON
        let json = GenomeSaver::save_to_json(
            &cortical_areas,
            &brain_regions,
            Some("test-roundtrip".to_string()),
            Some("Roundtrip Test".to_string()),
        ).unwrap();
        
        // Parse it back
        let parsed = GenomeParser::parse(&json).unwrap();
        
        // Verify data integrity
        assert_eq!(parsed.genome_id, "test-roundtrip");
        assert_eq!(parsed.genome_title, "Roundtrip Test");
        assert_eq!(parsed.cortical_areas.len(), 1);
        assert_eq!(parsed.brain_regions.len(), 1);
        
        let area = &parsed.cortical_areas[0];
        assert_eq!(area.cortical_id, "test01");
        assert_eq!(area.name, "Test Area");
        assert_eq!(area.dimensions.width, 10);
        assert_eq!(area.position, (5, 5, 5));
    }
}
