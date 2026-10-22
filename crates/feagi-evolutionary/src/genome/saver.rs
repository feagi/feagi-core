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

use crate::types::{EvoError, EvoResult};
#[cfg(test)]
use feagi_structures::genomic::brain_regions::RegionID;
use feagi_structures::genomic::cortical_area::{CorticalArea, CorticalID};
#[cfg(test)]
use feagi_structures::genomic::cortical_area::{
    CorticalAreaDimensions, CorticalAreaType, IOCorticalAreaConfigurationFlag,
};
use feagi_structures::genomic::BrainRegion;

/// Genome saver
pub struct GenomeSaver;

impl GenomeSaver {
    /// Save connectome to genome JSON
    ///
    /// **DEPRECATED**: This method produces incomplete hierarchical format v2.1 without morphologies/physiology.
    /// Use `feagi_evolutionary::save_genome_to_json(RuntimeGenome)` instead, which produces complete flat format v3.0.
    ///
    /// This method is kept only for legacy tests. Production code MUST use the RuntimeGenome saver.
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
    /// JSON string of the genome (hierarchical v2.1, incomplete)
    ///
    #[deprecated(
        note = "Use feagi_evolutionary::save_genome_to_json(RuntimeGenome) instead. This produces incomplete v2.1 format."
    )]
    pub fn save_to_json(
        cortical_areas: &HashMap<CorticalID, CorticalArea>,
        brain_regions: &HashMap<String, (BrainRegion, Option<String>)>,
        genome_id: Option<String>,
        genome_title: Option<String>,
    ) -> EvoResult<String> {
        // Build blueprint section
        let mut blueprint = serde_json::Map::new();

        for (cortical_id, area) in cortical_areas {
            let cortical_id_str = cortical_id.as_base_64();
            let mut area_data = serde_json::Map::new();

            // Required fields
            area_data.insert("cortical_name".to_string(), json!(area.name));
            area_data.insert(
                "block_boundaries".to_string(),
                json!([
                    area.dimensions.width,
                    area.dimensions.height,
                    area.dimensions.depth
                ]),
            );
            area_data.insert(
                "relative_coordinate".to_string(),
                json!([area.position.x, area.position.y, area.position.z]),
            );

            // Area type (from properties)
            let cortical_type = area
                .properties
                .get("cortical_group")
                .and_then(|v| v.as_str())
                .unwrap_or("CUSTOM");
            area_data.insert("cortical_type".to_string(), json!(cortical_type));

            // Add all properties from the area's properties HashMap
            for (key, value) in &area.properties {
                area_data.insert(key.clone(), value.clone());
            }

            blueprint.insert(cortical_id_str, Value::Object(area_data));
        }

        // Build brain_regions section
        let mut regions_map = serde_json::Map::new();

        for (region_id, (region, parent_id)) in brain_regions {
            let mut region_data = serde_json::Map::new();

            region_data.insert("title".to_string(), json!(region.name));
            region_data.insert(
                "parent_region_id".to_string(),
                if let Some(ref parent) = parent_id {
                    json!(parent)
                } else {
                    Value::Null
                },
            );

            // Membership is saved only as cortical_areas. Legacy `areas` is a read-time input.
            let areas: Vec<String> = region
                .cortical_areas
                .iter()
                .map(|id| id.as_base_64())
                .collect();

            // Add all properties from HashMap. Skip membership keys so a stored
            // legacy `areas` property cannot be written beside cortical_areas.
            for (key, value) in &region.properties {
                if key == "areas" || key == "cortical_areas" {
                    continue;
                }
                region_data.insert(key.clone(), value.clone());
            }
            region_data.insert("cortical_areas".to_string(), json!(areas));

            // Note: regions (child_regions) are not stored in properties - will be empty for now
            region_data.insert("regions".to_string(), json!(Vec::<String>::new()));

            regions_map.insert(region_id.to_string(), Value::Object(region_data));
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
            .map_err(|e| EvoError::Internal(format!("Failed to serialize genome: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_structures::genomic::RegionType;

    #[test]
    fn test_save_minimal_genome() {
        let mut cortical_areas = HashMap::new();
        let mut brain_regions = HashMap::new();

        // Create a test cortical area (use valid core ID)
        use feagi_structures::genomic::cortical_area::CoreCorticalType;
        let cortical_id = CoreCorticalType::Power.to_cortical_id();
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test Area".to_string(),
            CorticalAreaDimensions::new(10, 10, 10).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
        )
        .unwrap();

        cortical_areas.insert(cortical_id, area);

        // Create a test brain region
        let region =
            BrainRegion::new(RegionID::new(), "Root".to_string(), RegionType::Undefined).unwrap();

        brain_regions.insert("root".to_string(), (region, None));

        // Save to JSON
        #[allow(deprecated)]
        let json = GenomeSaver::save_to_json(
            &cortical_areas,
            &brain_regions,
            Some("test-001".to_string()),
            Some("Test Genome".to_string()),
        )
        .unwrap();

        // Verify it's valid JSON
        let parsed: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["genome_id"], "test-001");
        assert_eq!(parsed["genome_title"], "Test Genome");
        assert_eq!(parsed["version"], "2.1");
        assert!(parsed["blueprint"].is_object());
        assert!(parsed["brain_regions"].is_object());
        let root = &parsed["brain_regions"]["root"];
        assert!(root.get("areas").is_none());
        assert!(root.get("cortical_areas").is_some());
    }

    #[test]
    fn save_writes_membership_only_as_cortical_areas() {
        use feagi_structures::genomic::cortical_area::CoreCorticalType;
        use serde_json::json;

        let cortical_id = CoreCorticalType::Power.to_cortical_id();
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test Area".to_string(),
            CorticalAreaDimensions::new(10, 10, 10).unwrap(),
            (0, 0, 0).into(),
            CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
        )
        .unwrap();
        let mut cortical_areas = HashMap::new();
        cortical_areas.insert(cortical_id, area);

        let mut region =
            BrainRegion::new(RegionID::new(), "Root".to_string(), RegionType::Undefined).unwrap();
        region.add_area(cortical_id);
        region.add_property("areas".to_string(), json!(["should-not-leak"]));
        region.add_property(
            "cortical_areas".to_string(),
            json!(["should-not-replace-membership"]),
        );
        region.add_property("description".to_string(), json!("kept"));
        let mut brain_regions = HashMap::new();
        brain_regions.insert("root".to_string(), (region, None));

        #[allow(deprecated)]
        let json_text = GenomeSaver::save_to_json(
            &cortical_areas,
            &brain_regions,
            Some("test-membership".to_string()),
            Some("Membership".to_string()),
        )
        .unwrap();
        let parsed: Value = serde_json::from_str(&json_text).unwrap();
        let root = &parsed["brain_regions"]["root"];
        assert!(root.get("areas").is_none());
        assert_eq!(root["description"], "kept");
        assert_eq!(root["cortical_areas"], json!([cortical_id.as_base_64()]));

        let loaded = crate::genome::GenomeParser::parse(&json_text).unwrap();
        let (loaded_region, _) = &loaded.brain_regions[0];
        assert!(loaded_region.contains_area(&cortical_id));
        assert_eq!(loaded_region.area_count(), 1);
    }

    #[test]
    fn test_roundtrip() {
        use crate::genome::GenomeParser;

        // Create test data (use valid core ID)
        use feagi_structures::genomic::cortical_area::CoreCorticalType;
        let mut cortical_areas = HashMap::new();
        let cortical_id = CoreCorticalType::Power.to_cortical_id();
        let area = CorticalArea::new(
            cortical_id,
            0,
            "Test Area".to_string(),
            CorticalAreaDimensions::new(10, 10, 10).unwrap(),
            (5, 5, 5).into(),
            CorticalAreaType::BrainOutput(IOCorticalAreaConfigurationFlag::Boolean),
        )
        .unwrap();
        cortical_areas.insert(cortical_id, area);

        let mut brain_regions = HashMap::new();
        let region = BrainRegion::new(
            RegionID::new(),
            "Root Region".to_string(),
            RegionType::Undefined,
        )
        .unwrap();
        brain_regions.insert("root".to_string(), (region, None));

        // Save to JSON
        #[allow(deprecated)]
        let json = GenomeSaver::save_to_json(
            &cortical_areas,
            &brain_regions,
            Some("test-roundtrip".to_string()),
            Some("Roundtrip Test".to_string()),
        )
        .unwrap();

        // Parse it back
        let parsed = GenomeParser::parse(&json).unwrap();

        // Verify data integrity
        assert_eq!(parsed.genome_id, "test-roundtrip");
        assert_eq!(parsed.genome_title, "Roundtrip Test");
        assert_eq!(parsed.cortical_areas.len(), 1);
        assert_eq!(parsed.brain_regions.len(), 1);

        let area = &parsed.cortical_areas[0];
        // cortical_id is now stored as CorticalID object after roundtrip
        let expected_power_id =
            feagi_structures::genomic::cortical_area::CoreCorticalType::Power.to_cortical_id();
        assert_eq!(area.cortical_id, expected_power_id);
        assert_eq!(area.name, "Test Area");
        assert_eq!(area.dimensions.width, 10);
        assert_eq!(area.position, (5, 5, 5).into());
    }
}
