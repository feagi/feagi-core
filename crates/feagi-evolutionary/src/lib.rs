// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
# FEAGI Evolution & Genome Management

Handles all **genotype** operations for FEAGI:

- Genome I/O (JSON ↔ Rust structs)
- Genome validation
- Evolution operators (mutation, crossover)
- Fitness evaluation
- Population management

## Architecture

This crate manages the **genetic blueprint** (genotype) of FEAGI brains.
The actual instantiated brain structure (phenotype) is handled by `feagi-bdu`.

## Separation of Concerns

```text
feagi-evo (Genotype)        feagi-bdu (Phenotype)
─────────────────────       ─────────────────────
│ Genome JSON I/O    │  →   │ Neuroembryogenesis │
│ Genome Validation  │      │ Connectome I/O      │
│ Evolution Ops      │      │ Synaptogenesis      │
│ Fitness Eval       │      │ NPU Integration     │
└────────────────────┘      └─────────────────────┘
```

## Modules

- `genome` - Genome I/O and validation
- `evolution` - Evolution operators (future)
- `fitness` - Fitness evaluation (future)
- `population` - Population management (future)

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

/// Crate version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Core modules
pub mod converter_flat;
pub mod converter_flat_full;
pub mod converter_hierarchical_to_flat;
pub mod cortical_type_parser;
pub mod genome;
pub mod random;
pub mod runtime;
pub mod storage;
pub mod templates;
pub mod types;
pub mod validator;

// Re-export commonly used types
pub use converter_flat::convert_flat_to_hierarchical;
pub use converter_flat_full::convert_flat_to_hierarchical_full;
pub use converter_hierarchical_to_flat::convert_hierarchical_to_flat;
pub use cortical_type_parser::{parse_cortical_type, validate_cortical_type};
pub use genome::parser::string_to_cortical_id;
pub use genome::{
    load_genome_from_file, load_genome_from_json, migrate_genome, peek_quantization_precision,
    save_genome_to_file, save_genome_to_json, GenomeParser, GenomeSaver, MigrationResult,
    ParsedGenome,
};
pub use runtime::{
    GenomeMetadata, GenomeSignatures, GenomeStats, Morphology, MorphologyParameters,
    MorphologyRegistry, MorphologyType, PatternElement, PhysiologyConfig, RuntimeGenome,
};
#[cfg(feature = "async-tokio")]
pub use storage::fs_storage::FileSystemStorage;
pub use storage::{GenomeStorage, StorageError};
pub use templates::{
    add_core_morphologies, create_death_area, create_genome_with_core_areas,
    create_genome_with_core_morphologies, create_minimal_genome, create_power_area,
    ensure_core_components, get_default_neural_properties, load_barebones_genome,
    load_essential_genome, load_test_genome, load_vision_genome, BAREBONES_GENOME_JSON,
    ESSENTIAL_GENOME_JSON, TEST_GENOME_JSON, VISION_GENOME_JSON,
};
pub use types::{EvoError, EvoResult};
pub use validator::{validate_genome, ValidationResult};
