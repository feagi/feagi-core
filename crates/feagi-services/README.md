# feagi-services

FEAGI service layer - Stable application boundary for transport adapters.

## Overview

Transport-agnostic business logic layer that can be used by:
- REST API (HTTP)
- ZMQ control
- Embedded I2C
- Any custom transport

Provides services for:
- Neuron operations
- Genome management
- Connectome manipulation
- System analytics
- Runtime control

## Installation

```toml
[dependencies]
feagi-services = "2.0"
```

## Usage

```rust
use feagi_services::{NeuronService, GenomeService};

// Implement for your transport
impl NeuronService for MyTransport {
    async fn create_neuron(&self, params: CreateNeuronParams) -> ServiceResult<NeuronInfo> {
        // Business logic here
    }
}
```

## Architecture

Services sit between transport adapters (HTTP/ZMQ/etc.) and domain logic (BDU/NPU/etc.), providing a stable interface that doesn't change when either layer changes.

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.

