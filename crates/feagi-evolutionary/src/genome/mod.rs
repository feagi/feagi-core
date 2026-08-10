// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Genome I/O and manipulation for FEAGI Evolution.

This module handles:
- Parsing genome JSON files (genotype)
- Saving genome state back to JSON
- Genome validation
- Genome transformation/mutation (future)

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod converter;
pub mod loader;
pub mod migration;
pub mod migrator;
pub mod normalizers;
pub mod parser;
pub mod region_export;
pub mod runtime_saver;
pub mod saver;
pub mod schema;
pub mod signatures;
pub mod validators;

// Re-export main types
pub use converter::to_runtime_genome;
pub use loader::{
    load_genome_from_file, load_genome_from_json, load_genome_with_report, load_genome_with_report_from_file, peek_quantization_precision,
};
pub use migration::{ChainRegistry, ChainResult, ChainRunner, MigrationError, MigrationStepDiagnostics, Migrator, V2ToV3Migrator};
pub use migrator::{map_old_id_to_new, migrate_genome, MigrationResult};
pub use normalizers::{NormalizationDiagnostics, Normalizer, V3Normalizer};
pub use parser::{GenomeParser, ParsedGenome};
pub use region_export::subset_runtime_genome_for_region_branch;
pub use runtime_saver::{save_genome_to_file, save_genome_to_json};
pub use saver::GenomeSaver;
pub use schema::{detect_schema_version, GenomeSchemaVersion, CURRENT_SCHEMA_VERSION, MIN_SCHEMA_VERSION};
pub use signatures::generate_signatures;
pub use validators::{V3Validator, ValidationReport, Validator};

/// Build the canonical chain registry: the **full** migrator chain, all
/// normalizers, and all validators known to this crate.
///
/// This is what `loader.rs` uses internally and what external consumers
/// (`nrs-composer` per decision #6) should import as their starting point.
/// Composers may then drop validators outside their retention window
/// before invoking the runner.
///
/// Adding a new schema version: register the new `vN -> vN+1` migrator
/// here, register its `vN+1` validator and (optionally) `vN+1` normalizer,
/// and bump `CURRENT_SCHEMA_VERSION` in `schema/version.rs`. See
/// `feagi-evolutionary/src/genome/README.md` for the full procedure.
pub fn default_chain_registry() -> ChainRegistry {
    let mut registry = ChainRegistry::new();

    registry
        .register_migrator(Box::new(V2ToV3Migrator::new()))
        .expect("V2ToV3Migrator is well-formed; this expect is the registry contract violation we want to crash on");

    registry
        .register_normalizer(Box::new(V3Normalizer::new()))
        .expect("V3Normalizer is well-formed; this expect is the registry contract violation we want to crash on");

    registry.register_validator(Box::new(V3Validator::new()));

    registry
}
