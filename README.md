# FEAGI

Framework for Evolutionary Artificial General Intelligence - High-performance Rust libraries for bio-inspired neural computation.

## What is FEAGI?

FEAGI (Framework for Evolutionary Artificial General Intelligence) is a bio-inspired neural architecture that models brain structures and dynamics. FEAGI Core provides the foundational Rust libraries for building neural networks that learn and adapt like biological brains.

Unlike traditional neural networks, FEAGI:
- Models individual neurons with realistic dynamics (membrane potential, leak, refractory periods)
- Supports heterogeneous brain regions with distinct properties
- Enables structural plasticity (neurogenesis, synaptogenesis)
- Runs in real-time with spike-based computation
- Scales from microcontrollers to servers

## Key Features

- **Bio-Inspired Architecture**: Cortical areas, synaptic plasticity, and realistic neuron models
- **High Performance**: 50-100x faster than Python implementations through optimized Rust
- **Cross-Platform**: Runs on desktop, server, embedded (ESP32, Arduino, STM32), and cloud
- **GPU Acceleration**: Optional WGPU (cross-platform) and CUDA (NVIDIA) backends
- **No-Std Compatible**: Core algorithms work in resource-constrained environments
- **Modular Design**: Use individual crates or the complete framework
- **Python Bindings**: Integrate with existing Python workflows via PyO3

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
feagi = "0.0.1-beta.3"  # Umbrella crate (includes everything)
```

Or use individual building blocks:

```toml
[dependencies]
feagi-npu-burst-engine = "0.0.1-beta.3"  # Just the NPU
feagi-npu-neural = "0.0.1-beta.3"        # Just core types
```

Or umbrella with specific features:

```toml
[dependencies]
feagi = { version = "0.0.1-beta.3", features = ["gpu"] }
```

## Quick Start

### Create and Run a Neural Network

```rust
use feagi::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize neural processing unit
    let mut npu = RustNPU::new(100_000, 1_000_000, 20)?;
    
    // Load brain configuration
    npu.load_connectome("brain.json")?;
    
    // Process neural burst cycle
    npu.process_burst()?;
    
    Ok(())
}
```

### Embedded Target (no_std)

```rust
#![no_std]
use feagi_npu_neural::types::*;
use feagi_npu_runtime::traits::*;

// Configure for resource-constrained systems
let config = NeuronConfig::default();
// Process neural dynamics
```

### Python Integration

```python
# Python bindings available via PyO3
# Check feagi-python-sdk for Python integration
```

## Architecture

FEAGI Core is organized as a workspace of focused crates:

### Core Types and Data Structures
- **feagi-data-structures**: Fundamental data structures (neurons, synapses, cortical areas)
- **feagi-data-serialization**: Binary serialization formats (FBC - FEAGI Byte Container)
- **feagi-npu-neural**: Platform-agnostic neuron types and models (no_std compatible)
- **feagi-npu-runtime**: Runtime trait abstractions for std and embedded

### Neural Processing
- **feagi-npu-burst-engine**: High-performance burst cycle execution with GPU support
- **feagi-brain-development**: Neurogenesis and synaptogenesis
- **feagi-npu-plasticity**: Synaptic learning (STDP, memory consolidation)
- **feagi-evolutionary**: Genome I/O, evolution, and validation

### Infrastructure
- **feagi-state-manager**: Runtime state and agent registry
- **feagi-config**: TOML configuration loading and validation
- **feagi-observability**: Unified logging, telemetry, and profiling

### I/O and Integration
- **feagi-io**: I/O system (sensory input, motor output, transports)
- **feagi-sensorimotor**: Peripheral nervous system - data processing and encoding
- **feagi-agent**: Client library for agent integration
- **feagi-api**: REST API server with OpenAPI support
- **feagi-services**: High-level service compositions

### Platform Support
- **feagi-hal**: Hardware abstraction layer (ESP32, Arduino, STM32)

## Performance

FEAGI Core delivers significant performance improvements over interpreted implementations:

- **Synaptic Propagation**: 50-100x faster than Python/NumPy
- **Burst Frequency**: Supports 30Hz+ with millions of neurons
- **Memory Efficiency**: Minimal allocations, cache-friendly data structures
- **Parallel Processing**: Multi-threaded execution with Rayon
- **GPU Acceleration**: Optional WGPU or CUDA backends for massive parallelism

## Design Principles

### Biologically Plausible
- Individual neuron modeling with realistic parameters
- Spike-based computation (not rate-coded)
- Synaptic delays and conductances
- Structural and functional plasticity

### Cross-Platform from Day One
- Core algorithms are platform-agnostic (no_std compatible)
- Runtime adapters for different deployment targets
- Conditional compilation for embedded, desktop, and server
- No reliance on OS-specific features in core logic

### Performance Critical
- No allocations in hot paths (pre-allocated buffers)
- Cache-friendly data layouts (`#[repr(C)]`, AoS patterns)
- SIMD-friendly operations where applicable
- Optional GPU acceleration without compromising portability

### Type Safety
- Strong typing with newtypes (`NeuronId`, `SynapseId`)
- Compile-time guarantees over runtime checks
- Zero-cost abstractions throughout

## Feature Flags

### Umbrella Crate (feagi)

```toml
[features]
default = ["std", "full"]
std = [...]           # Standard library support
no_std = [...]        # Embedded/bare-metal
wasm = [...]          # WebAssembly target
full = ["compute", "io"]
compute = [...]       # Neural computation only
io = [...]            # I/O and networking
```

### Burst Engine

```toml
[features]
default = []
gpu = ["wgpu", ...]      # Cross-platform GPU (WGPU)
cuda = ["cudarc", ...]   # NVIDIA CUDA acceleration
all-gpu = ["gpu", "cuda"] # All GPU backends
```

## Building from Source

```bash
# Clone repository
git clone https://github.com/feagi/feagi-core
cd feagi-core

# Build the crate
cargo build --release

# Run tests
cargo test --workspace

# Build with GPU support
cargo build --release --features gpu

# Generate documentation
cargo doc --open
```

## Contributing

We welcome contributions! Whether you're fixing bugs, adding features, improving documentation, or optimizing performance, your help is appreciated.

### Getting Started

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes following our guidelines
4. Run tests and linting (`cargo test && cargo clippy`)
5. Submit a pull request

### Code Standards

All contributions must:
- Pass `cargo clippy` with zero warnings
- Pass `cargo test` (all tests)
- Include documentation for public APIs
- Follow Rust API guidelines
- Support cross-platform compilation where applicable

### Development Workflow

```bash
# Check compilation
cargo check --workspace

# Run tests
cargo test --workspace

# Lint code
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all

# Build release
cargo build --workspace --release
```

### Areas for Contribution

- **Performance optimization**: SIMD, GPU kernels, cache optimization
- **Platform support**: Additional embedded targets (Teensy, Nordic nRF, RISC-V)
- **Neural algorithms**: New plasticity rules, neuron models
- **Documentation**: Examples, tutorials, API documentation
- **Testing**: Edge cases, integration tests, benchmarks
- **Tools**: Visualization, debugging, profiling utilities

## Documentation

- **API Reference**: [docs.rs/feagi](https://docs.rs/feagi)
- **Architecture Guide**: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

Generate local documentation:

```bash
cargo doc --open
```

## Testing

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p feagi-burst-engine

# With features
cargo test -p feagi-burst-engine --features gpu

# Benchmarks
cargo bench -p feagi-burst-engine
```

## Platform Support

### Tested Platforms
- **Desktop**: Linux, macOS, Windows
- **Embedded**: ESP32 (WROOM, S3, C3)
- **Cloud**: Docker, Kubernetes

### Planned Support
- Arduino (Due, MKR, Nano 33 IoT)
- STM32 (F4, F7, H7 series)
- Teensy (4.0, 4.1)
- Nordic nRF (nRF52, nRF53)
- Raspberry Pi Pico (RP2040)

## Use Cases

- **Robotics**: Real-time control with adaptive learning
- **Edge AI**: On-device intelligence for IoT
- **Research**: Neuroscience modeling and experimentation
- **AGI Development**: Evolutionary and developmental AI systems
- **Embedded Intelligence**: Neural processing on microcontrollers

## Project Status

**Version**: 0.0.1-beta.3  
**Status**: Active development  
**Minimum Rust Version**: 1.75+  
**Versioning**: Independent per-crate (see [docs/INDEPENDENT_VERSIONING.md](docs/INDEPENDENT_VERSIONING.md))

FEAGI Core is under active development. The core APIs are stabilizing, but breaking changes may occur in minor releases.

## Community and Support

- **Discord**: [discord.gg/neuraville](https://discord.gg/PTVC8fyGN8)
- **Website**: [neuraville.com/feagi](https://neuraville.com/feagi)
- **Repository**: [github.com/feagi/feagi-core](https://github.com/feagi/feagi-core)
- **Issues**: [GitHub Issues](https://github.com/feagi/feagi-core/issues)
- **Email**: feagi@neuraville.com

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

Copyright 2025 Neuraville Inc.

## Citation

If you use FEAGI in your research, please cite:

```bibtex
@article{nadji2020brain,
  title={A brain-inspired framework for evolutionary artificial general intelligence},
  author={Nadji-Tehrani, Mohammad and Eslami, Ali},
  journal={IEEE transactions on neural networks and learning systems},
  volume={31},
  number={12},
  pages={5257--5271},
  year={2020},
  publisher={IEEE}
}
```

---

Built with Rust for performance, safety, and portability.
