# feagi-plasticity

Synaptic learning algorithms for FEAGI - STDP and memory formation.

## Overview

Implements biological learning mechanisms:
- STDP (Spike-Timing-Dependent Plasticity)
- Temporal pattern detection
- Memory neuron management
- Weight update rules

## Installation

```toml
[dependencies]
feagi-plasticity = "2.0"
```

## Usage

```rust
use feagi_plasticity::service::{PlasticityService, PlasticityConfig};

// Apply learning rules during or after burst cycles
```

## Use Cases

- Training systems
- Online learning
- NOT required for inference-only deployments

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.

