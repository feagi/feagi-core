/*!
Genome migration utilities for converting old-format cortical IDs to new format.

This module provides tools to migrate genomes from v2.1 with non-compliant cortical IDs
(e.g., iic100, omot00, _power) to the new feagi-data-processing template-compliant format
(e.g., svi1____, mot0____, ___power).

CRITICAL: Uses CoreCorticalType and templates from feagi-data-processing as single source of truth.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::collections::HashMap;
use serde_json::Value;
use crate::{EvoResult, EvoError};

/// Migration result containing the updated genome and statistics
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Migrated genome JSON
    pub genome: Value,
    /// Number of cortical IDs migrated
    pub cortical_ids_migrated: usize,
    /// Mapping from old ID to new ID
    pub id_mapping: HashMap<String, String>,
    /// Warnings encountered during migration
    pub warnings: Vec<String>,
}

/// Migrate a genome from old cortical ID format to new format
///
/// This function:
/// 1. Detects old-format cortical IDs (iic*, omot*, ogaz*, _power, etc.)
/// 2. Maps them to new template-compliant IDs using feagi-data-processing types
/// 3. Updates all references (blueprint, brain_regions, cortical_mapping_dst)
/// 4. Returns the migrated genome and migration statistics
///
/// # Arguments
/// * `genome_json` - Genome JSON Value to migrate
///
/// # Returns
/// * `MigrationResult` with migrated genome and statistics
pub fn migrate_genome(genome_json: &Value) -> EvoResult<MigrationResult> {
    let mut result = MigrationResult {
        genome: genome_json.clone(),
        cortical_ids_migrated: 0,
        id_mapping: HashMap::new(),
        warnings: Vec::new(),
    };
    
    // Step 1: Build ID mapping from old to new format
    build_id_mapping(&genome_json, &mut result)?;
    
    // Step 2: Migrate blueprint (cortical area definitions)
    migrate_blueprint(&mut result)?;
    
    // Step 3: Migrate brain_regions
    migrate_brain_regions(&mut result)?;
    
    // Step 4: Migrate cortical_mapping_dst references
    migrate_cortical_mappings(&mut result)?;
    
    Ok(result)
}

/// Build mapping from old cortical IDs to new template-compliant IDs
fn build_id_mapping(genome_json: &Value, result: &mut MigrationResult) -> EvoResult<()> {
    // Extract cortical IDs from blueprint
    let blueprint = genome_json.get("blueprint")
        .and_then(|v| v.as_object())
        .ok_or_else(|| EvoError::InvalidGenome("Missing or invalid blueprint".to_string()))?;
    
    // Check if genome is in flat format (keys like "_____10c-iic000-cx-...")
    let is_flat = blueprint.keys().any(|k| k.starts_with("_____10c-"));
    
    if is_flat {
        // Extract cortical IDs from flat keys
        use std::collections::HashSet;
        let mut seen_ids = HashSet::new();
        
        for flat_key in blueprint.keys() {
            if let Some(cortical_id) = extract_cortical_id_from_flat_key(flat_key) {
                if seen_ids.insert(cortical_id.clone()) {
                    // First time seeing this ID
                    if needs_migration(&cortical_id) {
                        if let Some(new_id) = map_old_id_to_new(&cortical_id) {
                            tracing::debug!("🔄 [MIGRATION] Flat format: '{}' → '{}'", cortical_id, new_id);
                            result.id_mapping.insert(cortical_id.clone(), new_id.clone());
                            result.cortical_ids_migrated += 1;
                        } else {
                            result.warnings.push(format!(
                                "Cannot auto-migrate cortical ID: '{}' - no mapping defined",
                                cortical_id
                            ));
                        }
                    }
                }
            }
        }
    } else {
        // Hierarchical format - direct cortical IDs
        for old_id in blueprint.keys() {
            if needs_migration(old_id) {
                if let Some(new_id) = map_old_id_to_new(old_id) {
                    tracing::debug!("🔄 [MIGRATION] Hierarchical format: '{}' → '{}'", old_id, new_id);
                    result.id_mapping.insert(old_id.clone(), new_id.clone());
                    result.cortical_ids_migrated += 1;
                } else {
                    result.warnings.push(format!(
                        "Cannot auto-migrate cortical ID: '{}' - no mapping defined",
                        old_id
                    ));
                }
            }
        }
    }
    
    Ok(())
}

/// Extract cortical ID from flat genome key
/// Example: "_____10c-iic000-cx-..." → "iic000"
fn extract_cortical_id_from_flat_key(key: &str) -> Option<String> {
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

/// Check if a cortical ID needs migration
fn needs_migration(id: &str) -> bool {
    // Old IPU formats
    if id.starts_with("iic") {
        return true;
    }
    
    // Old OPU formats
    if id.starts_with("omot") || id.starts_with("ogaz") {
        return true;
    }
    
    // Old CORE formats (not 8 bytes or not properly padded)
    if id.starts_with('_') && id.len() < 8 {
        return true;
    }
    
    false
}

/// Map old cortical ID to new template-compliant ID
///
/// Mapping rules:
/// - iic000 → Proper 8-byte SegmentedVision ID (index 0, Absolute frame handling, group 0)
/// - iic100 → Proper 8-byte SegmentedVision ID (index 1, Absolute frame handling, group 0)
/// - iic200 → Proper 8-byte SegmentedVision ID (index 2, Absolute frame handling, group 0)
/// - ... up to iic800 → Proper 8-byte SegmentedVision ID (index 8, Absolute frame handling, group 0)
/// - omot00 → Proper 8-byte Motor ID (index 0, Absolute frame handling, group 0)
/// - ogaz00 → Proper 8-byte Gaze ID (index 0, Absolute frame handling, group 0)
/// - _power → Proper 8-byte Core ID (CoreCorticalType::Power from feagi-data-processing)
/// - _death → Proper 8-byte Core ID (CoreCorticalType::Death from feagi-data-processing)
///
/// NOTE: Old format doesn't encode frame handling, so we default to Absolute.
/// This function is public so it can be used by string_to_cortical_id for individual ID conversions.
pub fn map_old_id_to_new(old_id: &str) -> Option<String> {
    use feagi_data_structures::genomic::cortical_area::descriptors::CorticalGroupIndex;
    use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::{FrameChangeHandling, PercentageNeuronPositioning};
    use feagi_data_structures::genomic::SensoryCorticalUnit;
    
    // IPU: iicXYZ → Proper 8-byte SegmentedVision ID
    if old_id.starts_with("iic") && old_id.len() >= 6 {
        // Extract index from iicX00 format (e.g., iic400 → index '4')
        if let Some(index_char) = old_id.chars().nth(3) {
            if index_char.is_ascii_digit() {
                let unit_index = index_char as u8 - b'0';
                if unit_index <= 8 {
                    // Generate proper 8-byte ID using SensoryCorticalUnit
                    // Priority: Absolute over Incremental (segmented vision doesn't use positioning)
                    let frame_handling = FrameChangeHandling::Absolute;
                    let group_index: CorticalGroupIndex = 0.into();
                    let cortical_ids = SensoryCorticalUnit::get_segmented_vision_cortical_ids_array(frame_handling, group_index);
                    
                    if (unit_index as usize) < cortical_ids.len() {
                        let new_id = cortical_ids[unit_index as usize].as_base_64();
                        tracing::debug!("🔄 [MIGRATION] Converting old ID '{}' → '{}' (base64, Absolute+Linear)", old_id, new_id);
                        return Some(new_id);
                    }
                }
            }
        }
    }
    
    // OPU: omot00 → Proper 8-byte Motor ID (Absolute + Linear, priority)
    use feagi_data_structures::genomic::MotorCorticalUnit;
    if old_id.starts_with("omot") && old_id.len() >= 6 {
        if let Some(index_chars) = old_id.get(4..6) {
            if let Ok(unit_index) = index_chars.parse::<u8>() {
                // Priority: Absolute over Incremental, Linear over Fractional
                let frame_handling = FrameChangeHandling::Absolute;
                let positioning = PercentageNeuronPositioning::Linear;
                let group_index: CorticalGroupIndex = 0.into();
                let cortical_ids = MotorCorticalUnit::get_rotary_motor_cortical_ids_array(frame_handling, positioning, group_index);
                
                if unit_index == 0 && !cortical_ids.is_empty() {
                    let new_id = cortical_ids[0].as_base_64();
                    tracing::debug!("🔄 [MIGRATION] Converting old ID '{}' → '{}' (base64, Absolute+Linear)", old_id, new_id);
                    return Some(new_id);
                }
            }
        }
    }
    
    // OPU: ogaz00 → Proper 8-byte Gaze ID (Absolute + Linear, priority)
    if old_id.starts_with("ogaz") && old_id.len() >= 6 {
        if let Some(index_chars) = old_id.get(4..6) {
            if let Ok(unit_index) = index_chars.parse::<u8>() {
                // Priority: Absolute over Incremental, Linear over Fractional
                let frame_handling = FrameChangeHandling::Absolute;
                let positioning = PercentageNeuronPositioning::Linear;
                let group_index: CorticalGroupIndex = 0.into();
                let cortical_ids = MotorCorticalUnit::get_gaze_control_cortical_ids_array(frame_handling, positioning, group_index);
                
                if (unit_index as usize) < cortical_ids.len() {
                    let new_id = cortical_ids[unit_index as usize].as_base_64();
                    tracing::debug!("🔄 [MIGRATION] Converting old ID '{}' → '{}' (base64, Absolute+Linear)", old_id, new_id);
                    return Some(new_id);
                }
            }
        }
    }
    
    // CORE: Use feagi-data-processing types as single source of truth
    use feagi_data_structures::genomic::cortical_area::CoreCorticalType;
    if old_id == "_power" {
        let new_id = CoreCorticalType::Power.to_cortical_id().as_base_64();
        tracing::debug!("🔄 [MIGRATION] Converting old ID '{}' → '{}' (base64)", old_id, new_id);
        return Some(new_id);
    }
    if old_id == "_death" {
        let new_id = CoreCorticalType::Death.to_cortical_id().as_base_64();
        tracing::debug!("🔄 [MIGRATION] Converting old ID '{}' → '{}' (base64)", old_id, new_id);
        return Some(new_id);
    }
    
    None
}

/// Migrate blueprint section (rename cortical area keys or flat keys)
fn migrate_blueprint(result: &mut MigrationResult) -> EvoResult<()> {
    let genome = result.genome.as_object_mut()
        .ok_or_else(|| EvoError::InvalidGenome("Genome is not an object".to_string()))?;
    
    let old_blueprint = genome.get("blueprint")
        .and_then(|v| v.as_object())
        .ok_or_else(|| EvoError::InvalidGenome("Missing or invalid blueprint".to_string()))?
        .clone();
    
    // Check if genome is in flat format
    let is_flat = old_blueprint.keys().any(|k| k.starts_with("_____10c-"));
    
    let mut new_blueprint = serde_json::Map::new();
    
    if is_flat {
        // Flat format: Update keys like "_____10c-iic000-cx-..." to "_____10c-svi0____-cx-..."
        for (old_key, value) in old_blueprint.iter() {
            if let Some(cortical_id) = extract_cortical_id_from_flat_key(old_key) {
                if let Some(new_id) = result.id_mapping.get(&cortical_id) {
                    // Replace cortical ID in flat key
                    let new_key = old_key.replace(&format!("-{}-", cortical_id), &format!("-{}-", new_id));
                    new_blueprint.insert(new_key, value.clone());
                } else {
                    new_blueprint.insert(old_key.clone(), value.clone());
                }
            } else {
                new_blueprint.insert(old_key.clone(), value.clone());
            }
        }
    } else {
        // Hierarchical format: Direct cortical IDs as keys
        for (old_id, area_data) in old_blueprint.iter() {
            let new_id = result.id_mapping.get(old_id).unwrap_or(old_id);
            new_blueprint.insert(new_id.clone(), area_data.clone());
        }
    }
    
    genome.insert("blueprint".to_string(), Value::Object(new_blueprint));
    
    Ok(())
}

/// Migrate brain_regions section (update cortical area references)
fn migrate_brain_regions(result: &mut MigrationResult) -> EvoResult<()> {
    let genome = result.genome.as_object_mut()
        .ok_or_else(|| EvoError::InvalidGenome("Genome is not an object".to_string()))?;
    
    if let Some(brain_regions_value) = genome.get_mut("brain_regions") {
        if let Some(brain_regions) = brain_regions_value.as_object_mut() {
            for region in brain_regions.values_mut() {
                if let Some(region_obj) = region.as_object_mut() {
                    // Migrate "areas" array
                    if let Some(areas_value) = region_obj.get_mut("areas") {
                        if let Some(areas) = areas_value.as_array_mut() {
                            for area_id in areas.iter_mut() {
                                if let Some(old_id) = area_id.as_str() {
                                    if let Some(new_id) = result.id_mapping.get(old_id) {
                                        *area_id = Value::String(new_id.clone());
                                    }
                                }
                            }
                        }
                    }
                    
                    // Migrate "inputs" array
                    if let Some(inputs_value) = region_obj.get_mut("inputs") {
                        if let Some(inputs) = inputs_value.as_array_mut() {
                            for input_id in inputs.iter_mut() {
                                if let Some(old_id) = input_id.as_str() {
                                    if let Some(new_id) = result.id_mapping.get(old_id) {
                                        *input_id = Value::String(new_id.clone());
                                    }
                                }
                            }
                        }
                    }
                    
                    // Migrate "outputs" array
                    if let Some(outputs_value) = region_obj.get_mut("outputs") {
                        if let Some(outputs) = outputs_value.as_array_mut() {
                            for output_id in outputs.iter_mut() {
                                if let Some(old_id) = output_id.as_str() {
                                    if let Some(new_id) = result.id_mapping.get(old_id) {
                                        *output_id = Value::String(new_id.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Migrate cortical_mapping_dst references in all cortical areas
fn migrate_cortical_mappings(result: &mut MigrationResult) -> EvoResult<()> {
    let genome = result.genome.as_object_mut()
        .ok_or_else(|| EvoError::InvalidGenome("Genome is not an object".to_string()))?;
    
    if let Some(blueprint_value) = genome.get_mut("blueprint") {
        if let Some(blueprint) = blueprint_value.as_object_mut() {
            for area_data in blueprint.values_mut() {
                if let Some(area_obj) = area_data.as_object_mut() {
                    // Migrate cortical_mapping_dst keys
                    if let Some(dstmap_value) = area_obj.get("cortical_mapping_dst") {
                        if let Some(old_dstmap) = dstmap_value.as_object() {
                            let mut new_dstmap = serde_json::Map::new();
                            
                            for (old_dst_id, mapping_rules) in old_dstmap.iter() {
                                let new_dst_id = result.id_mapping.get(old_dst_id).unwrap_or(old_dst_id);
                                new_dstmap.insert(new_dst_id.clone(), mapping_rules.clone());
                            }
                            
                            area_obj.insert("cortical_mapping_dst".to_string(), Value::Object(new_dstmap));
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_map_old_id_to_new() {
        use feagi_data_structures::genomic::cortical_area::CoreCorticalType;
        use feagi_data_structures::genomic::cortical_area::descriptors::CorticalGroupIndex;
        use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::FrameChangeHandling;
        use feagi_data_structures::genomic::SensoryCorticalUnit;
        use feagi_data_structures::genomic::MotorCorticalUnit;
        use feagi_data_structures::genomic::cortical_area::io_cortical_area_data_type::PercentageNeuronPositioning;
        
        // IPU migrations - should return base64 IDs with Absolute frame handling
        let group_index: CorticalGroupIndex = 0.into();
        let frame_handling = FrameChangeHandling::Absolute;
        let expected_svi0 = SensoryCorticalUnit::get_segmented_vision_cortical_ids_array(frame_handling, group_index)[0].as_base_64();
        let expected_svi1 = SensoryCorticalUnit::get_segmented_vision_cortical_ids_array(frame_handling, group_index)[1].as_base_64();
        let expected_svi4 = SensoryCorticalUnit::get_segmented_vision_cortical_ids_array(frame_handling, group_index)[4].as_base_64();
        let expected_svi8 = SensoryCorticalUnit::get_segmented_vision_cortical_ids_array(frame_handling, group_index)[8].as_base_64();
        
        assert_eq!(map_old_id_to_new("iic000"), Some(expected_svi0));
        assert_eq!(map_old_id_to_new("iic100"), Some(expected_svi1));
        assert_eq!(map_old_id_to_new("iic400"), Some(expected_svi4));
        assert_eq!(map_old_id_to_new("iic800"), Some(expected_svi8));
        
        // OPU migrations - should return base64 IDs with Absolute + Linear
        let positioning = PercentageNeuronPositioning::Linear;
        let expected_mot0 = MotorCorticalUnit::get_rotary_motor_cortical_ids_array(frame_handling, positioning, group_index)[0].as_base_64();
        let expected_gaz0 = MotorCorticalUnit::get_gaze_control_cortical_ids_array(frame_handling, positioning, group_index)[0].as_base_64();
        
        assert_eq!(map_old_id_to_new("omot00"), Some(expected_mot0));
        assert_eq!(map_old_id_to_new("ogaz00"), Some(expected_gaz0));
        
        // CORE migrations - use types from feagi-data-processing (single source of truth)
        assert_eq!(
            map_old_id_to_new("_power"), 
            Some(CoreCorticalType::Power.to_cortical_id().as_base_64())
        );
        assert_eq!(
            map_old_id_to_new("_death"), 
            Some(CoreCorticalType::Death.to_cortical_id().as_base_64())
        );
        
        // No migration needed for already-migrated IDs
        assert_eq!(map_old_id_to_new("svi0____"), None);
        assert_eq!(map_old_id_to_new(&CoreCorticalType::Power.to_cortical_id().as_base_64()), None);
    }
    
    #[test]
    fn test_needs_migration() {
        use feagi_data_structures::genomic::cortical_area::CoreCorticalType;
        
        // Should migrate
        assert!(needs_migration("iic000"));
        assert!(needs_migration("omot00"));
        assert!(needs_migration("_power"));
        
        // Should NOT migrate - use types from feagi-data-processing
        assert!(!needs_migration("svi0____"));
        assert!(!needs_migration("mot0____"));
        assert!(!needs_migration(&CoreCorticalType::Power.to_cortical_id().to_string()));
        assert!(!needs_migration("custom01"));
    }
    
    #[test]
    fn test_migrate_simple_genome() {
        use feagi_data_structures::genomic::cortical_area::CoreCorticalType;
        
        let genome = json!({
            "genome_id": "test",
            "version": "2.1",
            "blueprint": {
                "iic000": {
                    "cortical_name": "Vision 0",
                    "cortical_type": "IPU"
                },
                "_power": {
                    "cortical_name": "Power",
                    "cortical_type": "CORE"
                }
            },
            "brain_regions": {
                "root": {
                    "areas": ["iic000", "_power"],
                    "inputs": ["iic000"],
                    "outputs": []
                }
            }
        });
        
        let result = migrate_genome(&genome).expect("Migration failed");
        
        // Use types from feagi-data-processing (single source of truth)
        let expected_power_id = CoreCorticalType::Power.to_cortical_id().to_string();
        
        // Check that IDs were migrated
        assert_eq!(result.cortical_ids_migrated, 2);
        assert_eq!(result.id_mapping.get("iic000"), Some(&"svi0____".to_string()));
        assert_eq!(result.id_mapping.get("_power"), Some(&expected_power_id));
        
        // Check that blueprint was updated
        let new_blueprint = result.genome.get("blueprint")
            .and_then(|v| v.as_object())
            .expect("Blueprint missing");
        assert!(new_blueprint.contains_key("svi0____"));
        assert!(new_blueprint.contains_key(&expected_power_id));
        assert!(!new_blueprint.contains_key("iic000"));
        assert!(!new_blueprint.contains_key("_power"));
        
        // Check that brain_regions were updated
        let regions = result.genome.get("brain_regions")
            .and_then(|v| v.as_object())
            .expect("brain_regions missing");
        let root = regions.get("root")
            .and_then(|v| v.as_object())
            .expect("root region missing");
        let areas = root.get("areas")
            .and_then(|v| v.as_array())
            .expect("areas array missing");
        
        assert_eq!(areas[0].as_str(), Some("svi0____"));
        assert_eq!(areas[1].as_str(), Some(expected_power_id.as_str()));
    }
}

