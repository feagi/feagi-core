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

// Re-export main types
pub use parser::{GenomeParser, ParsedGenome};
pub use saver::GenomeSaver;
