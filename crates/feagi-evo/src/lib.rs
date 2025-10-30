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

// Re-export commonly used types
pub use types::{EvoError, EvoResult};
pub use genome::{GenomeParser, GenomeSaver, ParsedGenome};


