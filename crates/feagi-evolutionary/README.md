# feagi-evolutionary

Evolution and genome management for FEAGI - genotype operations.

## Overview

Handles genome (brain definition) I/O and manipulation:
- Genome loading and saving
- Genotype validation
- Cortical area parsing
- Genome migration between versions

## Installation

```toml
[dependencies]
feagi-evolutionary = "2.0"
```

## Usage

```rust
use feagi_evolutionary::genome::{load_genome, save_genome};

// Load brain definition from JSON
let genome = load_genome("path/to/brain.genome")?;
```

## Genome evaluation records

`evaluation::GenomeEvaluation` (schema v1, ADR-016 in `docs/FEAGI_TRAINER_ADR_SET.md`) records
one evaluated individual: the content hash of the genome snapshot taken when the brain was
rebuilt at protocol start, the Trainer scorecard ids, a `ComparabilityKey`, validation-split
fitness, and genome-only lineage. It performs no I/O; hosts store it (Composer for
feagi-desktop).

- Two evaluations may be ranked only when their `ComparabilityKey`s are equal.
- `selectable_fitness()` is `Some` only for `FitnessOutcome::Scored`; runs without a
  validation phase (`NoFitnessSplit`) or with a skipped one (`Incomplete`) stay in history but
  are not fitness points or parents.
- `validate()` enforces the invariants: `sha256:<64 hex>` genome, parent, and run-config
  hashes; `n > 1` requires a confidence interval containing the value; manual and imported
  genomes are generation 0 with no parents; a mutation has one parent, a crossover two or
  more, and neither can be its own parent.

```rust
use feagi_evolutionary::{GenomeEvaluation, EvaluationError};

fn accept(record: &GenomeEvaluation) -> Result<Option<f64>, EvaluationError> {
    record.validate()?;
    Ok(record.selectable_fitness().map(|fitness| fitness.value))
}
```

## Use Cases

- Development-time genome editing
- Brain configuration management
- NOT needed at runtime (use feagi-connectome-serialization instead)

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.

