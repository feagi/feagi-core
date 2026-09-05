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

## Connectome persistence

Full snapshots restore the serialized NPU state directly. Lite snapshots rebuild
the genome baseline and then apply memory and plasticity overlays.

During lite import, long-term-memory pattern hashes are recomputed from persisted
replay-frame cortical areas and voxel coordinates after neuroembryogenesis. This
keeps recall aligned with rebuilt runtime neuron IDs while leaving full import
and runtime pattern detection unchanged. Rehashing fails explicitly when a
replay coordinate is missing or resolves ambiguously.

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.

