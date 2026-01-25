// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Genome migration utilities for converting old-format cortical IDs to new format.

This module provides tools to migrate genomes from v2.1 with non-compliant cortical IDs
(e.g., iic100, omot00, _power) to the new feagi-data-processing template-compliant format
(e.g., svi1____, mot0____, ___power).

CRITICAL: Uses CoreCorticalType and templates from feagi-data-processing as single source of truth.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::{EvoError, EvoResult};
use serde_json::Value;
use std::collections::HashMap;

fn is_legacy_io_shorthand(id: &str) -> bool {
    id.len() == 6 && (id.starts_with('i') || id.starts_with('o'))
}

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
    build_id_mapping(genome_json, &mut result)?;

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
    let blueprint = genome_json
        .get("blueprint")
        .and_then(|v| v.as_object())
        .ok_or_else(|| EvoError::InvalidGenome("Missing or invalid blueprint".to_string()))?;

    // Check if genome is in flat format (keys like "_____10c-iic000-cx-...")
    let is_flat = blueprint.keys().any(|k| k.starts_with("_____10c-"));

    // Collect unique cortical IDs found in the genome blueprint.
    use std::collections::{BTreeSet, HashSet};
    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut cortical_ids: BTreeSet<String> = BTreeSet::new(); // deterministic ordering

    if is_flat {
        for flat_key in blueprint.keys() {
            if let Some(cortical_id) = extract_cortical_id_from_flat_key(flat_key) {
                if seen_ids.insert(cortical_id.clone()) {
                    cortical_ids.insert(cortical_id);
                }
            }
        }
    } else {
        for old_id in blueprint.keys() {
            cortical_ids.insert(old_id.clone());
        }
    }

    // Collect already-used base64 cortical IDs to avoid collisions when allocating MiscData group IDs.
    let mut used_base64: HashSet<String> = HashSet::new();
    for id in cortical_ids.iter() {
        if feagi_structures::genomic::cortical_area::CorticalID::try_from_base_64(id).is_ok() {
            used_base64.insert(id.clone());
        }
    }

    // IDs that need stateful mapping (legacy IO shorthands without FDP bitmask metadata).
    let mut legacy_io_shorthands: Vec<String> = Vec::new();

    for id in cortical_ids.iter() {
        // Special-case: legacy base64 cortical IDs that are *syntactically valid* but represent
        // an old/unsupported vision family ("imis") that should be migrated to SegmentedVision ("isvi").
        //
        // We only apply this when we can deterministically infer the intended SegmentedVision tile
        // from the cortical area's name (vision_LL/LM/LR/ML/C/MR/TL/TM/TR). This avoids guessing.
        {
            use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
            use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
            use feagi_structures::genomic::cortical_area::CorticalID;
            use feagi_structures::genomic::SensoryCorticalUnit;

            let name_opt: Option<&str> = if is_flat {
                let name_key = format!("_____10c-{}-cx-__name-t", id);
                blueprint.get(&name_key).and_then(|v| v.as_str())
            } else {
                blueprint
                    .get(id)
                    .and_then(|v| v.as_object())
                    .and_then(|o| o.get("name"))
                    .and_then(|v| v.as_str())
            };

            if let (Ok(cid), Some(name)) = (CorticalID::try_from_base_64(id), name_opt) {
                if cid.extract_subtype().as_deref() == Some("mis") {
                    let tile_idx: Option<usize> = match name {
                        // Index convention (per project decision):
                        // - LL=0, LM=1, LR=2
                        // - ML=3, C=4, MR=5
                        // - TL=6, TM=7, TR=8
                        "vision_LL" => Some(0),
                        "vision_LM" => Some(1),
                        "vision_LR" => Some(2),
                        "vision_ML" => Some(3),
                        "vision_C" => Some(4),
                        "vision_MR" => Some(5),
                        "vision_TL" => Some(6),
                        "vision_TM" => Some(7),
                        "vision_TR" => Some(8),
                        _ => None,
                    };

                    if let Some(idx) = tile_idx {
                        let group_index: CorticalUnitIndex = 0.into();
                        let segmented =
                            SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                                FrameChangeHandling::Absolute,
                                group_index,
                            );
                        if idx < segmented.len() {
                            let new_id = segmented[idx].as_base_64();
                            if !used_base64.contains(&new_id) {
                                used_base64.insert(new_id.clone());
                                result.id_mapping.insert(id.clone(), new_id.clone());
                                result.cortical_ids_migrated += 1;
                                result.warnings.push(format!(
                                    "Legacy base64 vision cortical ID '{}' (subtype=mis, name='{}') migrated to SegmentedVision(tile_index={}, group=0) → '{}'",
                                    id, name, idx, new_id
                                ));
                                continue;
                            }

                            result.warnings.push(format!(
                                "Legacy base64 vision cortical ID '{}' (subtype=mis, name='{}') could not be migrated to SegmentedVision(tile_index={}, group=0) because target ID '{}' already exists in the genome",
                                id, name, idx, new_id
                            ));
                        }
                    }

                    // Option 2 (requested): legacy base64 vision-related IPU ("vision_ipu") should
                    // migrate to a supported MiscData IPU cortical ID with a unique group ID.
                    if name == "vision_ipu" {
                        // Allocate the smallest available MiscData IPU group deterministically.
                        for group_u16 in 0u16..=u8::MAX as u16 {
                            let group_u8 = group_u16 as u8;
                            let group_index: CorticalUnitIndex = group_u8.into();
                            let new_id = SensoryCorticalUnit::get_cortical_ids_array_for_misc_data_with_parameters(
                                FrameChangeHandling::Absolute,
                                group_index,
                            )[0]
                                .as_base_64();

                            if used_base64.contains(&new_id) {
                                continue;
                            }

                            used_base64.insert(new_id.clone());
                            result.id_mapping.insert(id.clone(), new_id.clone());
                            result.cortical_ids_migrated += 1;
                            result.warnings.push(format!(
                                "Legacy base64 vision cortical ID '{}' (subtype=mis, name='{}') migrated to MiscData IPU(group={}) → '{}'",
                                id, name, group_u8, new_id
                            ));
                            break;
                        }

                        // If we didn't insert a mapping, we ran out of group IDs.
                        if !result.id_mapping.contains_key(id) {
                            return Err(EvoError::InvalidGenome(
                                "Unable to allocate unique MiscData IPU group ID for legacy base64 vision cortical IDs".to_string(),
                            ));
                        }

                        continue;
                    }
                }
            }
        }

        if !needs_migration(id) {
            continue;
        }

        if let Some(new_id) = map_old_id_to_new(id) {
            tracing::debug!("🔄 [MIGRATION] '{}' → '{}'", id, new_id);
            used_base64.insert(new_id.clone());
            result.id_mapping.insert(id.clone(), new_id);
            result.cortical_ids_migrated += 1;
            continue;
        }

        if is_legacy_io_shorthand(id) {
            legacy_io_shorthands.push(id.clone());
            continue;
        }

        result.warnings.push(format!(
            "Cannot auto-migrate cortical ID: '{}' - no mapping defined",
            id
        ));
    }

    if !legacy_io_shorthands.is_empty() {
        apply_legacy_io_shorthand_migration(&legacy_io_shorthands, &mut used_base64, result)?;
    }

    Ok(())
}

fn apply_legacy_io_shorthand_migration(
    legacy_ids: &[String],
    used_base64: &mut std::collections::HashSet<String>,
    result: &mut MigrationResult,
) -> EvoResult<()> {
    use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
    use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};

    // Deterministic rule set (requested):
    // - Special-case known legacy segmented-vision shorthands iv00?? → SegmentedVision tile.
    // - Any other legacy IO shorthand starting with i/o → map to MiscData IPU/OPU.
    // - If multiple unknown shorthands exist, allocate distinct MiscData group IDs (CorticalUnitIndex)
    //   per-domain (IPU vs OPU). These are independent objects and should not share group counters.
    //   Skip any collisions with already-used base64 cortical IDs.
    let frame_handling = FrameChangeHandling::Absolute;

    let mut exceptions: Vec<String> = Vec::new();
    let mut next_group_ipu: u16 = 0;
    let mut next_group_opu: u16 = 0;

    for old_id in legacy_ids.iter() {
        if old_id.starts_with("iv00") && old_id.len() == 6 {
            // Legacy segmented vision shorthands use suffixes like:
            // - TL/TM/TR/ML/MR/BL/BM/BR and _C for center.
            //
            // Index convention (per project decision):
            // - BL=0, BM=1, BR=2
            // - ML=3, _C=4, MR=5
            // - TL=6, TM=7, TR=8
            let suffix = &old_id[4..6];
            let tile_idx: Option<usize> = match suffix {
                "_C" => Some(4),
                "BL" => Some(0),
                "BM" => Some(1),
                "BR" => Some(2),
                "ML" => Some(3),
                "MR" => Some(5),
                "TL" => Some(6),
                "TM" => Some(7),
                "TR" => Some(8),
                _ => None,
            };

            if let Some(idx) = tile_idx {
                let group_index: CorticalUnitIndex = 0.into();
                let segmented =
                    SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                        frame_handling,
                        group_index,
                    );
                if idx < segmented.len() {
                    let new_id = segmented[idx].as_base_64();
                    used_base64.insert(new_id.clone());
                    result.id_mapping.insert(old_id.clone(), new_id.clone());
                    result.cortical_ids_migrated += 1;

                    exceptions.push(format!(
                        "Legacy segmented-vision shorthand '{}' mapped to SegmentedVision(tile_index={}) (group=0) → '{}'",
                        old_id, idx, new_id
                    ));
                    continue;
                }
            }
        }

        let is_input = old_id.starts_with('i');
        let next_group = if is_input {
            &mut next_group_ipu
        } else {
            &mut next_group_opu
        };
        loop {
            if *next_group > u8::MAX as u16 {
                return Err(EvoError::InvalidGenome(
                    "Unable to allocate unique MiscData group ID for legacy IO shorthands"
                        .to_string(),
                ));
            }
            let group_u8 = *next_group as u8;
            let group_index: CorticalUnitIndex = group_u8.into();

            let new_id = if is_input {
                SensoryCorticalUnit::get_cortical_ids_array_for_misc_data_with_parameters(
                    frame_handling,
                    group_index,
                )[0]
                .as_base_64()
            } else {
                MotorCorticalUnit::get_cortical_ids_array_for_misc_data_with_parameters(
                    frame_handling,
                    group_index,
                )[0]
                .as_base_64()
            };

            *next_group += 1;

            if used_base64.contains(&new_id) {
                continue;
            }

            used_base64.insert(new_id.clone());
            result.id_mapping.insert(old_id.clone(), new_id.clone());
            result.cortical_ids_migrated += 1;

            exceptions.push(format!(
                "Legacy {} shorthand '{}' not recognized; mapped to {} MiscData (group={}) → '{}'",
                if is_input { "IPU" } else { "OPU" },
                old_id,
                if is_input { "IPU" } else { "OPU" },
                group_u8,
                new_id
            ));
            break;
        }
    }

    if !exceptions.is_empty() {
        tracing::warn!(
            target: "feagi-evo",
            "⚠️ [MIGRATION] Applied legacy IO shorthand migration rules ({}): {}",
            exceptions.len(),
            exceptions.join(" | ")
        );
        result.warnings.extend(exceptions);
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

    // Legacy IO shorthands (6-char ASCII) lacking FDP IO metadata (bytes 4-5).
    // Examples: iv00_C, i___id, o___id
    if is_legacy_io_shorthand(id) {
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
    use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
        FrameChangeHandling, PercentageNeuronPositioning,
    };
    use feagi_structures::genomic::SensoryCorticalUnit;

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
                    let group_index: CorticalUnitIndex = 0.into();
                    let cortical_ids =
                        SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                            frame_handling,
                            group_index,
                        );

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
    use feagi_structures::genomic::MotorCorticalUnit;
    if old_id.starts_with("omot") && old_id.len() >= 6 {
        if let Some(index_chars) = old_id.get(4..6) {
            if let Ok(unit_index) = index_chars.parse::<u8>() {
                // Priority: Absolute over Incremental, Linear over Fractional
                let frame_handling = FrameChangeHandling::Absolute;
                let positioning = PercentageNeuronPositioning::Linear;
                let group_index: CorticalUnitIndex = 0.into();
                let cortical_ids =
                    MotorCorticalUnit::get_cortical_ids_array_for_rotary_motor_with_parameters(
                        frame_handling,
                        positioning,
                        group_index,
                    );

                if unit_index == 0 && !cortical_ids.is_empty() {
                    let new_id = cortical_ids[0].as_base_64();
                    tracing::debug!(
                        "🔄 [MIGRATION] Converting old ID '{}' → '{}' (base64, Absolute+Linear)",
                        old_id,
                        new_id
                    );
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
                let group_index: CorticalUnitIndex = 0.into();
                let cortical_ids =
                    MotorCorticalUnit::get_cortical_ids_array_for_gaze_with_parameters(
                        frame_handling,
                        positioning,
                        group_index,
                    );

                if (unit_index as usize) < cortical_ids.len() {
                    let new_id = cortical_ids[unit_index as usize].as_base_64();
                    tracing::debug!(
                        "🔄 [MIGRATION] Converting old ID '{}' → '{}' (base64, Absolute+Linear)",
                        old_id,
                        new_id
                    );
                    return Some(new_id);
                }
            }
        }
    }

    // CORE: Use feagi-data-processing types as single source of truth
    use feagi_structures::genomic::cortical_area::CoreCorticalType;
    if old_id == "_power" {
        let new_id = CoreCorticalType::Power.to_cortical_id().as_base_64();
        tracing::debug!(
            "🔄 [MIGRATION] Converting old ID '{}' → '{}' (base64)",
            old_id,
            new_id
        );
        return Some(new_id);
    }
    // Legacy shorthand used by older FEAGI genomes: "___pwr" (6-char) refers to core Power.
    if old_id == "___pwr" {
        let new_id = CoreCorticalType::Power.to_cortical_id().as_base_64();
        tracing::debug!(
            "🔄 [MIGRATION] Converting old ID '{}' → '{}' (base64)",
            old_id,
            new_id
        );
        return Some(new_id);
    }
    if old_id == "_death" {
        let new_id = CoreCorticalType::Death.to_cortical_id().as_base_64();
        tracing::debug!(
            "🔄 [MIGRATION] Converting old ID '{}' → '{}' (base64)",
            old_id,
            new_id
        );
        return Some(new_id);
    }

    None
}

/// Migrate blueprint section (rename cortical area keys or flat keys)
fn migrate_blueprint(result: &mut MigrationResult) -> EvoResult<()> {
    let genome = result
        .genome
        .as_object_mut()
        .ok_or_else(|| EvoError::InvalidGenome("Genome is not an object".to_string()))?;

    let old_blueprint = genome
        .get("blueprint")
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
                    let new_key =
                        old_key.replace(&format!("-{}-", cortical_id), &format!("-{}-", new_id));
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
    let genome = result
        .genome
        .as_object_mut()
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
    let genome = result
        .genome
        .as_object_mut()
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
                                let new_dst_id =
                                    result.id_mapping.get(old_dst_id).unwrap_or(old_dst_id);
                                new_dstmap.insert(new_dst_id.clone(), mapping_rules.clone());
                            }

                            area_obj.insert(
                                "cortical_mapping_dst".to_string(),
                                Value::Object(new_dstmap),
                            );
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
        use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
        use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
        use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
        use feagi_structures::genomic::cortical_area::CoreCorticalType;
        use feagi_structures::genomic::MotorCorticalUnit;
        use feagi_structures::genomic::SensoryCorticalUnit;

        // IPU migrations - should return base64 IDs with Absolute frame handling
        let group_index: CorticalUnitIndex = 0.into();
        let frame_handling = FrameChangeHandling::Absolute;
        let expected_svi0 =
            SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                frame_handling,
                group_index,
            )[0]
            .as_base_64();
        let expected_svi1 =
            SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                frame_handling,
                group_index,
            )[1]
            .as_base_64();
        let expected_svi4 =
            SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                frame_handling,
                group_index,
            )[4]
            .as_base_64();
        let expected_svi8 =
            SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                frame_handling,
                group_index,
            )[8]
            .as_base_64();

        assert_eq!(map_old_id_to_new("iic000"), Some(expected_svi0));
        assert_eq!(map_old_id_to_new("iic100"), Some(expected_svi1));
        assert_eq!(map_old_id_to_new("iic400"), Some(expected_svi4));
        assert_eq!(map_old_id_to_new("iic800"), Some(expected_svi8));

        // OPU migrations - should return base64 IDs with Absolute + Linear
        let positioning = PercentageNeuronPositioning::Linear;
        let expected_mot0 =
            MotorCorticalUnit::get_cortical_ids_array_for_rotary_motor_with_parameters(
                frame_handling,
                positioning,
                group_index,
            )[0]
            .as_base_64();
        let expected_gaz0 = MotorCorticalUnit::get_cortical_ids_array_for_gaze_with_parameters(
            frame_handling,
            positioning,
            group_index,
        )[0]
        .as_base_64();

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
        assert_eq!(
            map_old_id_to_new(&CoreCorticalType::Power.to_cortical_id().as_base_64()),
            None
        );
    }

    #[test]
    fn test_needs_migration() {
        use feagi_structures::genomic::cortical_area::CoreCorticalType;

        // Should migrate
        assert!(needs_migration("iic000"));
        assert!(needs_migration("omot00"));
        assert!(needs_migration("_power"));

        // Should NOT migrate - use types from feagi-data-processing
        assert!(!needs_migration("svi0____"));
        assert!(!needs_migration("mot0____"));
        assert!(!needs_migration(
            &CoreCorticalType::Power.to_cortical_id().to_string()
        ));
        assert!(!needs_migration("custom01"));
    }

    #[test]
    fn test_migrate_simple_genome() {
        use feagi_structures::genomic::cortical_area::CoreCorticalType;

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
        // Check that the old IDs were mapped to new base64 IDs
        assert!(
            result.id_mapping.contains_key("iic000"),
            "iic000 should be migrated"
        );
        assert!(
            result.id_mapping.contains_key("_power"),
            "_power should be migrated"
        );
        assert_eq!(result.id_mapping.get("_power"), Some(&expected_power_id));

        // Check that blueprint was updated
        let new_blueprint = result
            .genome
            .get("blueprint")
            .and_then(|v| v.as_object())
            .expect("Blueprint missing");
        // Verify that the new IDs are in the blueprint and old ones are gone
        assert!(
            new_blueprint.contains_key(&expected_power_id),
            "Power ID should be in blueprint"
        );
        assert!(
            !new_blueprint.contains_key("iic000"),
            "Old iic000 should be removed"
        );
        assert!(
            !new_blueprint.contains_key("_power"),
            "Old _power should be removed"
        );

        // Check that brain_regions were updated
        let regions = result
            .genome
            .get("brain_regions")
            .and_then(|v| v.as_object())
            .expect("brain_regions missing");
        let root = regions
            .get("root")
            .and_then(|v| v.as_object())
            .expect("root region missing");
        let areas = root
            .get("areas")
            .and_then(|v| v.as_array())
            .expect("areas array missing");

        // Verify that areas contains the migrated IDs (not hardcoding expected format)
        let migrated_vision_id = result
            .id_mapping
            .get("iic000")
            .expect("iic000 should be mapped");
        assert_eq!(
            areas[0].as_str(),
            Some(migrated_vision_id.as_str()),
            "Vision ID should be migrated"
        );
        assert_eq!(
            areas[1].as_str(),
            Some(expected_power_id.as_str()),
            "Power ID should be migrated"
        );
    }

    #[test]
    fn test_migrate_legacy_io_shorthands_to_segmented_center_and_misc() {
        // Minimal flat-format genome blueprint containing legacy IO shorthands seen in older FEAGI:
        // - iv00_C: legacy central vision sensor shorthand (should map to SegmentedVision center)
        // - i___id: legacy IPU shorthand (unknown template) -> MiscData IPU
        // - o___id: legacy OPU shorthand (unknown template) -> MiscData OPU
        let genome = json!({
            "version": "2.0",
            "blueprint": {
                "_____10c-iv00_C-cx-__name-t": "Central vision sensor",
                "_____10c-i___id-cx-__name-t": "ID Trainer",
                "_____10c-o___id-cx-__name-t": "ID Recognition",
            },
            "brain_regions": null,
            "neuron_morphologies": {},
            "physiology": {}
        });

        let result = migrate_genome(&genome).unwrap();

        // iv00_C → SegmentedVision center (index 4), Absolute frame handling, group 0.
        use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
        use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
        use feagi_structures::genomic::SensoryCorticalUnit;
        let expected_center =
            SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                FrameChangeHandling::Absolute,
                CorticalUnitIndex::from(0u8),
            )[4]
            .as_base_64();

        assert_eq!(result.id_mapping.get("iv00_C").unwrap(), &expected_center);

        // Unknown shorthands → distinct MiscData group IDs (deterministic allocation).
        let i_mapped = result.id_mapping.get("i___id").expect("i___id mapped");
        let o_mapped = result.id_mapping.get("o___id").expect("o___id mapped");
        assert_ne!(i_mapped, o_mapped);

        // Ensure we generated an exceptions report.
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("Legacy") && w.contains("mapped")),
            "Expected migration warnings report for legacy IO shorthands"
        );
    }

    #[test]
    fn test_migrate_legacy_segmented_vision_tl_to_subunit_6() {
        // Validate project-specific mapping:
        // - iv00TL (legacy shorthand) → SegmentedVision tile_index 6 (TL) in group 0.
        let genome = json!({
            "version": "2.0",
            "blueprint": {
                "_____10c-iv00TL-cx-__name-t": "Vision Top Left",
            },
            "brain_regions": null,
            "neuron_morphologies": {},
            "physiology": {}
        });

        let result = migrate_genome(&genome).unwrap();

        use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
        use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;
        use feagi_structures::genomic::SensoryCorticalUnit;
        let expected =
            SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision_with_parameters(
                FrameChangeHandling::Absolute,
                CorticalUnitIndex::from(0u8),
            )[6]
            .as_base_64();

        assert_eq!(result.id_mapping.get("iv00TL").unwrap(), &expected);
    }
}
