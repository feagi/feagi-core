// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
High-level genome loading API.

Provides convenient functions for loading genomes from files or JSON strings,
automatically parsing and converting to RuntimeGenome.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::path::Path;
use std::fs;
use serde_json::Value;
use crate::{EvoResult, RuntimeGenome};
use super::{GenomeParser, converter::to_runtime_genome};

/// Load a genome from a JSON file
pub fn load_genome_from_file<P: AsRef<Path>>(path: P) -> EvoResult<RuntimeGenome> {
    let json_str = fs::read_to_string(path)?;
    load_genome_from_json(&json_str)
}

/// Peek at genome's quantization precision without full parsing
///
/// This is a lightweight function that extracts ONLY the quantization_precision
/// field from a genome file, allowing the system to create the appropriately-typed
/// NPU before loading the full genome.
///
/// # Returns
/// - `"fp32"` or `"f32"` → f32 precision
/// - `"int8"` → INT8 quantization  
/// - `"fp16"` or `"f16"` → f16 precision (future)
/// - If field is missing or unparseable, returns `"int8"` (default)
///
/// # Example
/// ```rust,ignore
/// let precision = peek_quantization_precision("genome.json")?;
/// let npu = match precision.as_str() {
///     "fp32" | "f32" => DynamicNPUGeneric::F32(RustNPU::<f32>::new(...)?),
///     "int8" => DynamicNPUGeneric::INT8(RustNPU::<INT8Value>::new(...)?),
///     _ => DynamicNPUGeneric::INT8(RustNPU::<INT8Value>::new(...)?), // default
/// };
/// ```
pub fn peek_quantization_precision<P: AsRef<Path>>(path: P) -> EvoResult<String> {
    let json_str = fs::read_to_string(path)?;
    
    // Parse to generic JSON Value
    let json_value: Value = serde_json::from_str(&json_str)
        .map_err(|e| crate::types::EvoError::InvalidGenome(format!("Failed to parse JSON: {}", e)))?;
    
    // Try to extract quantization_precision from genome_physiology
    let precision = json_value
        .get("genome_physiology")
        .and_then(|p| p.get("quantization_precision"))
        .and_then(|q| q.as_str())
        .unwrap_or("int8"); // Default to INT8 if not found
    
    Ok(precision.to_lowercase())
}

/// Load a genome from a JSON string
pub fn load_genome_from_json(json_str: &str) -> EvoResult<RuntimeGenome> {
    // Parse JSON to Value first to check format
    let json_value: Value = serde_json::from_str(json_str)
        .map_err(|e| crate::types::EvoError::InvalidGenome(format!("Failed to parse JSON: {}", e)))?;
    
    // Check if genome is in flat format and convert if needed
    let hierarchical_json = if is_flat_format(&json_value) {
        // Convert flat format to hierarchical format
        crate::converter_flat_full::convert_flat_to_hierarchical_full(&json_value)?
    } else {
        json_value
    };
    
    // CRITICAL: Migrate old cortical ID formats to new feagi-data-processing compliant format
    // This converts old IDs like iic000 → svi0____, _power → ___power, etc.
    let migrated_json = match crate::genome::migrator::migrate_genome(&hierarchical_json) {
        Ok(migration_result) => {
            if migration_result.cortical_ids_migrated > 0 {
                tracing::info!(
                    "🔄 [GENOME-LOAD] Migrated {} cortical IDs from old format to new format",
                    migration_result.cortical_ids_migrated
                );
                // Log some example migrations for debugging
                for (old_id, new_id) in migration_result.id_mapping.iter().take(5) {
                    tracing::info!("🔄 [GENOME-LOAD]   Example: '{}' → '{}'", old_id, new_id);
                }
                if !migration_result.warnings.is_empty() {
                    for warning in &migration_result.warnings {
                        tracing::warn!("⚠️  [GENOME-LOAD] Migration warning: {}", warning);
                    }
                }
            } else {
                tracing::debug!("🔄 [GENOME-LOAD] No cortical IDs needed migration");
            }
            migration_result.genome
        }
        Err(e) => {
            tracing::warn!("⚠️  [GENOME-LOAD] Migration failed: {}, continuing without migration", e);
            hierarchical_json
        }
    };
    
    // Convert back to JSON string for parsing
    let hierarchical_json_str = serde_json::to_string(&migrated_json)
        .map_err(|e| crate::types::EvoError::InvalidGenome(format!("Failed to serialize converted genome: {}", e)))?;
    
    // Parse JSON to ParsedGenome
    let parsed = GenomeParser::parse(&hierarchical_json_str)?;
    
    // Convert to RuntimeGenome
    let mut runtime_genome = to_runtime_genome(parsed, &hierarchical_json_str)?;
    
    // CRITICAL: Auto-fix common issues before validation
    // This prevents genomes with 0 dimensions or 0 per_voxel_neuron_cnt from failing
    let fixes_applied = crate::validator::auto_fix_genome(&mut runtime_genome);
    if fixes_applied > 0 {
        tracing::info!("🔧 [GENOME-LOAD] Applied {} auto-fixes to genome", fixes_applied);
    }
    
    Ok(runtime_genome)
}

/// Check if genome is in flat format
/// Flat format has blueprint keys like "_____10c-_power-cx-subgrp-t" (with underscores)
/// Hierarchical format has blueprint keys like "cortical_id" that map to objects
fn is_flat_format(genome_value: &Value) -> bool {
    let blueprint = match genome_value.get("blueprint") {
        Some(bp) => bp,
        None => return false,
    };
    
    let blueprint_obj = match blueprint.as_object() {
        Some(obj) => obj,
        None => return false,
    };
    
    // Check if any keys look like flat format (contain multiple underscores and hyphens)
    // Flat format keys typically look like: "_____10c-_power-cx-subgrp-t"
    // Hierarchical format keys are simple IDs like: "cortical_id"
    blueprint_obj.keys().any(|key| {
        // Flat format keys typically have:
        // - Multiple underscores at start
        // - Hyphens separating parts
        // - Ending with a single letter suffix like "-t", "-i", "-f", "-b", "-d"
        key.starts_with("___") && key.contains('-') && key.len() > 20
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_minimal_genome() {
        let json = r#"{
            "genome_id": "test_genome",
            "genome_title": "Test Genome",
            "genome_description": "A test genome",
            "version": "2.0",
            "blueprint": {
                "_power": {
                    "cortical_name": "Test Area",
                    "block_boundaries": [10, 10, 10],
                    "relative_coordinate": [0, 0, 0],
                    "cortical_type": "INTERCONNECT"
                }
            },
            "brain_regions": {},
            "neuron_morphologies": {},
            "physiology": {
                "simulation_timestep": 0.025,
                "max_age": 10000000
            },
            "stats": {
                "innate_cortical_area_count": 1,
                "innate_neuron_count": 0,
                "innate_synapse_count": 0
            },
            "signatures": {
                "genome": "0000000000000000",
                "blueprint": "0000000000000000",
                "physiology": "0000000000000000"
            },
            "timestamp": 1234567890.0
        }"#;
        
        let genome = load_genome_from_json(json).unwrap();
        
        assert_eq!(genome.metadata.genome_id, "test_genome");
        assert_eq!(genome.metadata.version, "2.0");
        assert_eq!(genome.cortical_areas.len(), 1);
        assert_eq!(genome.physiology.simulation_timestep, 0.025);
    }
}

