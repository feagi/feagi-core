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

pub mod parser;
pub mod saver;
pub mod signatures;
pub mod converter;
pub mod loader;
pub mod runtime_saver;
pub mod migrator;

// Re-export main types
pub use parser::{GenomeParser, ParsedGenome};
pub use saver::GenomeSaver;
pub use signatures::generate_signatures;
pub use converter::to_runtime_genome;
pub use loader::{load_genome_from_file, load_genome_from_json, peek_quantization_precision};
pub use runtime_saver::{save_genome_to_file, save_genome_to_json};
pub use migrator::{migrate_genome, MigrationResult, map_old_id_to_new};
