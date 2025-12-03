# feagi-connectome-serialization

Connectome serialization and deserialization for FEAGI runtime snapshots.

## Overview

Handles runtime brain state persistence:
- Binary serialization format
- LZ4 compression
- Fast load/save for inference
- Checksum validation

## Installation

```toml
[dependencies]
feagi-connectome-serialization = "2.0"
```

## Usage

```rust
use feagi_connectome_serialization::{save_connectome, load_connectome};

// Save trained brain
save_connectome(&snapshot, "brain.connectome")?;

// Load for inference
let snapshot = load_connectome("brain.connectome")?;
```

## Use Cases

- Inference engines loading pre-trained brains
- Checkpointing during training
- Model distribution

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.

