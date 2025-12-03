# feagi-types

Core data structures for FEAGI neural processing framework.

## Overview

Foundational types used across the FEAGI ecosystem:
- `NeuronId`, `SynapseId`, `CorticalID`
- `Neuron`, `Synapse`, `CorticalArea`
- `NeuralValue` trait for quantization support (f32, f16, int8)

## Installation

```toml
[dependencies]
feagi-types = "2.0"
```

## Usage

```rust
use feagi_types::{NeuronId, Synapse, CorticalID};

let neuron_id = NeuronId(42);
let cortical_id = CorticalID::from_base64("iABCDEF").unwrap();
```

## Features

- Platform-agnostic (works with std and no_std)
- Zero-copy operations where possible
- Efficient serialization with serde

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.

