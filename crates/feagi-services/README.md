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

### Artifact compatibility and migration

`brain_artifact::validate_and_migrate_brain_artifact` is the authoritative,
transport-independent compatibility boundary. It performs these stages
atomically on immutable source bytes:

1. Validate the binary container and connectome schema versions.
2. Migrate and validate the embedded genome through `feagi-evolutionary`.
3. Rewrite connectome cortical references using the genome migration's
   identifier map.
4. Reject references absent from the migrated genome.
5. Embed a `BrainArtifactManifest` with independent container, connectome,
   genome, producer, digest, and migration-history fields.
6. Serialize a new artifact and return a structured migration report.

Connectome-lite artifacts must include a genome. Full artifacts may omit it
because they carry complete structure; in that case the report explicitly says
genome compatibility was not evaluated. Future versions and migration-chain
gaps fail explicitly. The service never downgrades and has no permissive
fallback.

HTTP adapters expose `/v1/connectome/validate` and `/v1/connectome/migrate`.
Python services use the same implementation through
`feagi_rust_py_libs.connectome`, preventing validation drift between FEAGI and
Composer.

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.

