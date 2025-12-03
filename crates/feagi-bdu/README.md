# feagi-bdu

Brain Development Unit - Neurogenesis and synaptogenesis for FEAGI.

## Overview

Handles structural development of neural networks:
- Cortical area creation
- Synaptogenesis (connectivity rule application)
- Morphology patterns (projector, expander, reducer, etc.)
- Spatial organization and hashing

## Installation

```toml
[dependencies]
feagi-bdu = "2.0"
```

## Usage

```rust
use feagi_bdu::connectivity::synaptogenesis::apply_synaptogenesis_rules;

// Create new cortical areas and connections during development
```

## Use Cases

- Training systems that need structural plasticity
- Development-time brain construction  
- NOT needed for inference-only deployments

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.

