# feagi-runtime-std

Standard runtime adapter for FEAGI - Desktop and server deployments.

## Overview

Provides std-based implementations for:
- Dynamic neuron arrays (Vec-based)
- Dynamic synapse storage  
- Rayon parallelization
- Async I/O support

## Installation

```toml
[dependencies]
feagi-runtime-std = "2.0"
```

## Usage

```rust
use feagi_runtime_std::NeuronArrayStd;

// Used internally by feagi-burst-engine on std platforms
```

## Platform Support

- Linux, macOS, Windows
- Docker containers
- Cloud deployments
- Kubernetes

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.

