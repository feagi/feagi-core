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
let genome = load_genome("path/to/genome.json")?;
```

## Use Cases

- Development-time genome editing
- Brain configuration management
- NOT needed at runtime (use feagi-connectome-serialization instead)

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.

