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
pub mod parser;
pub mod region_export;
pub mod runtime_saver;
pub mod saver;
pub mod schema;
pub mod signatures;
pub mod validators;

// Re-export main types
pub use converter::to_runtime_genome;
pub use loader::{load_genome_from_file, load_genome_from_json, peek_quantization_precision};
pub use migration::{
    ChainRegistry, ChainResult, ChainRunner, MigrationError, MigrationStepDiagnostics, Migrator,
};
pub use migrator::{map_old_id_to_new, migrate_genome, MigrationResult};
pub use parser::{GenomeParser, ParsedGenome};
pub use region_export::subset_runtime_genome_for_region_branch;
pub use runtime_saver::{save_genome_to_file, save_genome_to_json};
pub use saver::GenomeSaver;
pub use schema::{
    detect_schema_version, GenomeSchemaVersion, CURRENT_SCHEMA_VERSION, MIN_SCHEMA_VERSION,
};
pub use signatures::generate_signatures;
pub use validators::{V3Validator, ValidationReport, Validator};
