// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
High-level genome loading API.

Provides convenient functions for loading genomes from files or JSON strings,
automatically parsing and converting to RuntimeGenome.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use super::migration::ChainResult;
use super::{converter::to_runtime_genome, GenomeParser};
use crate::{EvoResult, RuntimeGenome};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Load a genome from a JSON file.
pub fn load_genome_from_file<P: AsRef<Path>>(path: P) -> EvoResult<RuntimeGenome> {
    let json_str = fs::read_to_string(path)?;
    load_genome_from_json(&json_str)
}

/// Load a genome from a JSON file and return the chain report alongside
/// the runtime genome. See [`load_genome_with_report`] for semantics.
pub fn load_genome_with_report_from_file<P: AsRef<Path>>(path: P) -> EvoResult<(RuntimeGenome, ChainResult)> {
    let json_str = fs::read_to_string(path)?;
    load_genome_with_report(&json_str)
}

/// Peek at genome's quantization precision without full parsing
///
/// This is a lightweight function that extracts ONLY the quantization_precision
/// field from a genome file, allowing the system to create the appropriately-typed
/// NPU before loading the full genome.
///
/// # Returns
/// - `"fp32"` or `"f32"` -> f32 precision
/// - `"int8"` -> INT8 quantization
/// - `"fp16"` or `"f16"` -> f16 precision (future)
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

    let json_value: Value =
        serde_json::from_str(&json_str).map_err(|e| crate::types::EvoError::invalid_genome(format!("Failed to parse JSON: {e}")))?;

    let precision = json_value
        .get("genome_physiology")
        .and_then(|p| p.get("quantization_precision"))
        .and_then(|q| q.as_str())
        .unwrap_or("int8");

    Ok(precision.to_lowercase())
}

/// Load a genome from a JSON string.
///
/// Convenience wrapper over [`load_genome_with_report`] that discards the
/// chain report. Final-version validator errors are **not** treated as
/// hard failures by this function; they are logged at `warn` level and
/// the `RuntimeGenome` is returned anyway so the caller can proceed in
/// degraded mode. Callers that need to react to validator errors (show
/// a banner, reject the load, etc.) must use
/// [`load_genome_with_report`] and inspect `ChainResult.blocking_errors`.
///
/// Only hard I/O / JSON / parse failures propagate as `Err` here.
pub fn load_genome_from_json(json_str: &str) -> EvoResult<RuntimeGenome> {
    let (genome, _report) = load_genome_with_report(json_str)?;
    Ok(genome)
}

/// Load a genome from a JSON string and return the chain report alongside
/// the runtime genome.
///
/// Pipeline:
/// 1. Deserialize JSON.
/// 2. If the top-level shape is the flat (encoded-key) form, expand it to
///    the hierarchical form. Representation coercion, not schema
///    migration; runs before schema-version detection.
/// 3. Hand the hierarchical `Value` to the chain runner. The chain
///    detects the schema version, walks `vN -> vLatest`, applies each
///    registered migrator and normalizer, and runs the `vLatest`
///    validator. See `feagi-core/docs/GENOME_SCHEMA_VERSIONING.md`.
/// 4. Re-serialize the chain output, parse to `ParsedGenome`, and
///    convert to `RuntimeGenome`.
///
/// Validator semantics: the library **never** turns a validation error
/// into a load failure. It returns both the `RuntimeGenome` and the
/// `ChainResult` so the consumer can decide whether to (a) proceed and
/// notify the user, (b) drop problematic elements and run degraded (see
/// `genome_validity` in the FEAGI health endpoint), or (c) reject the
/// load at its own layer. Hard `Err` returns here are reserved for I/O,
/// JSON parse, or structural failures where there is no `RuntimeGenome`
/// to return at all.
pub fn load_genome_with_report(json_str: &str) -> EvoResult<(RuntimeGenome, ChainResult)> {
    let json_value: Value =
        serde_json::from_str(json_str).map_err(|e| crate::types::EvoError::invalid_genome(format!("Failed to parse JSON: {e}")))?;

    let hierarchical_json = if is_flat_format(&json_value) {
        crate::converter_flat_full::convert_flat_to_hierarchical_full(&json_value).map_err(|e| {
            tracing::error!(target: "feagi-evo", "convert_flat_to_hierarchical_full failed: {}", e);
            e
        })?
    } else {
        json_value
    };

    let (migrated_json, report) = run_default_chain(hierarchical_json)?;

    let migrated_json_str = serde_json::to_string(&migrated_json)
        .map_err(|e| crate::types::EvoError::invalid_genome(format!("Failed to serialize migrated genome: {e}")))?;

    let parsed = GenomeParser::parse(&migrated_json_str).map_err(|e| {
        tracing::error!(target: "feagi-evo", "GenomeParser::parse failed: {}", e);
        e
    })?;

    let runtime_genome = to_runtime_genome(parsed, &migrated_json_str).map_err(|e| {
        tracing::error!(target: "feagi-evo", "to_runtime_genome failed: {}", e);
        e
    })?;

    Ok((runtime_genome, report))
}

/// Run the default chain registry on `hierarchical_json` to bring it to
/// `CURRENT_SCHEMA_VERSION`. Logs migrators, normalizers, and advisory
/// warnings. Returns the migrated `Value` alongside the full
/// `ChainResult` so downstream callers can decide how to react to
/// blocking errors (never converted to `Err` here).
fn run_default_chain(mut hierarchical_json: Value) -> EvoResult<(Value, ChainResult)> {
    use crate::genome::default_chain_registry;
    use crate::genome::migration::ChainRunner;
    use crate::genome::schema::CURRENT_SCHEMA_VERSION;

    let registry = default_chain_registry();
    let runner = ChainRunner::new(&registry);

    let result = runner
        .run_to(&mut hierarchical_json, CURRENT_SCHEMA_VERSION)
        .map_err(|e| crate::types::EvoError::invalid_genome(format!("Genome chain failed: {e}")))?;

    if !result.migrators_applied.is_empty() {
        tracing::info!(
            target: "feagi-evo",
            "[GENOME-LOAD] Migrated v{} -> v{} via {:?}",
            result.from_version.as_u32(),
            result.to_version.as_u32(),
            result.migrators_applied
        );
        for diag in &result.per_step_diagnostics {
            for transform in &diag.transformations {
                tracing::debug!(
                    target: "feagi-evo",
                    "[GENOME-LOAD]   v{} -> v{}: {}",
                    diag.from_version.as_u32(),
                    diag.to_version.as_u32(),
                    transform
                );
            }
        }
    }

    if !result.normalizers_applied.is_empty() {
        for diag in &result.per_normalizer_diagnostics {
            if !diag.is_clean() {
                tracing::info!(
                    target: "feagi-evo",
                    "[GENOME-LOAD] Normalizer at v{} applied {} corrections",
                    diag.schema_version.as_u32(),
                    diag.transformations.len()
                );
                for transform in &diag.transformations {
                    tracing::debug!(target: "feagi-evo", "[GENOME-LOAD]   {}", transform);
                }
            }
        }
    }

    for warning in &result.advisory_warnings {
        tracing::debug!(target: "feagi-evo", "[GENOME-LOAD] advisory: {}", warning);
    }

    // Policy: validator errors are reports, not exceptions. Log at warn
    // so operators see them without the library deciding the recovery
    // path. Consumers inspect `ChainResult.blocking_errors` via
    // `load_genome_with_report` and route to the `genome_validity`
    // health-check flag or their own UI.
    if !result.is_blocking_clean() {
        tracing::warn!(
            target: "feagi-evo",
            "[GENOME-LOAD] Genome v{} has {} blocking validator error(s); returning genome anyway (consumer decides)",
            result.to_version.as_u32(),
            result.blocking_errors.len()
        );
        for err in &result.blocking_errors {
            tracing::warn!(target: "feagi-evo", "[GENOME-LOAD]   blocking: {}", err);
        }
    }

    Ok((hierarchical_json, result))
}

/// Check if genome is in flat format.
///
/// Flat format has blueprint keys like `_____10c-_power-cx-subgrp-t`
/// (encoded underscore/hyphen keys). Hierarchical format has blueprint
/// keys like `cortical_id` that map to objects.
fn is_flat_format(genome_value: &Value) -> bool {
    let blueprint = match genome_value.get("blueprint") {
        Some(bp) => bp,
        None => return false,
    };

    let blueprint_obj = match blueprint.as_object() {
        Some(obj) => obj,
        None => return false,
    };

    blueprint_obj
        .keys()
        .any(|key| key.starts_with("___") && key.contains('-') && key.len() > 20)
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

    #[test]
    fn load_with_report_surfaces_chain_metadata() {
        // v2.0 genome -> chain should run V2ToV3Migrator and stamp v3.
        let json = r#"{
            "genome_id": "t1",
            "genome_title": "t",
            "genome_description": "t",
            "version": "2.0",
            "blueprint": {},
            "brain_regions": {},
            "neuron_morphologies": {},
            "physiology": { "simulation_timestep": 0.025, "max_age": 1 },
            "stats": { "innate_cortical_area_count": 0, "innate_neuron_count": 0, "innate_synapse_count": 0 },
            "signatures": { "genome": "0", "blueprint": "0", "physiology": "0" },
            "timestamp": 0.0
        }"#;

        let (_genome, report) = load_genome_with_report(json).unwrap();
        assert_eq!(report.from_version.as_u32(), 2);
        assert_eq!(report.to_version.as_u32(), 3);
        assert_eq!(report.migrators_applied, vec!["v2_to_v3"]);
        // V3Validator is a placeholder today; clean report expected.
        assert!(report.is_blocking_clean());
    }

    #[test]
    fn load_never_hard_fails_on_validator_errors() {
        // This test pins the policy: the library does not convert
        // validator errors into Err. If V3Validator ever gains rules
        // that fail for this input, this test must keep passing because
        // the error is in `report.blocking_errors`, not `Err`.
        let json = r#"{
            "genome_id": "t2",
            "genome_title": "t",
            "genome_description": "t",
            "version": "3.0",
            "genome_schema_version": 3,
            "blueprint": {},
            "brain_regions": {},
            "neuron_morphologies": {},
            "physiology": { "simulation_timestep": 0.025, "max_age": 1 },
            "stats": { "innate_cortical_area_count": 0, "innate_neuron_count": 0, "innate_synapse_count": 0 },
            "signatures": { "genome": "0", "blueprint": "0", "physiology": "0" },
            "timestamp": 0.0
        }"#;
        let result = load_genome_with_report(json);
        assert!(result.is_ok(), "load_genome_with_report must not turn validator output into Err");
    }
}
