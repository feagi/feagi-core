# feagi-burst-engine

High-performance neural processing unit (NPU) for FEAGI burst cycle execution.

## Overview

The burst engine processes neural activity in discrete time steps:
- Synaptic propagation
- Neural dynamics (membrane potential updates)
- Firing detection and management
- Optional GPU acceleration (WGPU, CUDA)

## Installation

```toml
[dependencies]
feagi-burst-engine = "2.0"

# With GPU support
feagi-burst-engine = { version = "2.0", features = ["gpu"] }

# With CUDA support (NVIDIA only)
feagi-burst-engine = { version = "2.0", features = ["cuda"] }
```

## Usage

```rust
use feagi_burst_engine::RustNPU;

let mut npu = RustNPU::new(100_000, 1_000_000, 20)?;
npu.load_connectome("brain.json")?;
npu.process_burst()?;
```

## Features

- `gpu` - Cross-platform GPU acceleration via WGPU
- `cuda` - NVIDIA CUDA acceleration
- `all-gpu` - Enable all GPU backends

## Performance

- 50-100x faster than Python implementations
- Supports 30Hz+ burst frequency
- Tested with millions of neurons

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.

