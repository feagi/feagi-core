/*!
Genome parsing and persistence module.

This module handles loading/saving FEAGI genomes (JSON format) and
translating them into runtime data structures.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod parser;
pub mod saver;

pub use parser::{GenomeParser, ParsedGenome};
pub use saver::GenomeSaver;




