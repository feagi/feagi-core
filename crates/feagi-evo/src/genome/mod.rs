/*!
Genome I/O module.

Handles loading and saving FEAGI genome JSON files.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod parser;
pub mod saver;

// Re-export public types
pub use parser::{GenomeParser, ParsedGenome, RawGenome, RawCorticalArea, RawBrainRegion};
pub use saver::GenomeSaver;

