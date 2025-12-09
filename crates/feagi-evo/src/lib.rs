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

// Core modules
pub mod genome;
pub mod types;
pub mod runtime;
pub mod validator;
pub mod converter_flat;
pub mod converter_flat_full;
pub mod converter_hierarchical_to_flat;
pub mod templates;
pub mod cortical_type_parser;
pub mod random;

// Re-export commonly used types
pub use types::{EvoError, EvoResult};
pub use genome::parser::string_to_cortical_id;
pub use genome::{
    GenomeParser, GenomeSaver, ParsedGenome,
    load_genome_from_file, load_genome_from_json,
    save_genome_to_file, save_genome_to_json,
    peek_quantization_precision,
    migrate_genome, MigrationResult,
};
pub use runtime::{
    RuntimeGenome, GenomeMetadata, MorphologyRegistry, Morphology, 
    MorphologyType, MorphologyParameters, PatternElement,
    PhysiologyConfig, GenomeSignatures, GenomeStats,
};
pub use validator::{validate_genome, ValidationResult};
pub use converter_flat::convert_flat_to_hierarchical;
pub use converter_flat_full::convert_flat_to_hierarchical_full;
pub use converter_hierarchical_to_flat::convert_hierarchical_to_flat;
pub use cortical_type_parser::{parse_cortical_type, validate_cortical_type};
pub use templates::{
    create_minimal_genome, 
    create_genome_with_core_areas,
    create_genome_with_core_morphologies, 
    add_core_morphologies,
    create_death_area,
    create_power_area,
    get_default_neural_properties,
    ensure_core_components,
    load_essential_genome,
    load_barebones_genome,
    load_test_genome,
    load_vision_genome,
    ESSENTIAL_GENOME_JSON,
    BAREBONES_GENOME_JSON,
    TEST_GENOME_JSON,
    VISION_GENOME_JSON,
};


