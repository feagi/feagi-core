# FEAGI Core - Rust Libraries

**High-performance neural burst processing engine and core shared Rust libraries for FEAGI 2.0**

## Project Goals

- **Performance**: 50-100x speedup over Python (165ms → <3ms for synaptic propagation)
- **Scalability**: Support 30Hz burst frequency with 1.2M neuron firings
- **Architecture**: Incremental migration path from Python to Rust
- **Modularity**: Separate crates for selective compilation and IP protection

## Workspace Structure

```
feagi-core/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── feagi-types/              # Core types (NeuronId, Synapse, etc.)
│   ├── feagi-neural/             # Pure neural dynamics (no_std compatible)
│   ├── feagi-synapse/            # Pure synaptic algorithms (no_std compatible)
│   ├── feagi-burst-engine/       # High-performance burst processing
│   ├── feagi-state-manager/      # Runtime state management
│   ├── feagi-plasticity/         # Plasticity algorithms (separate IP)
│   ├── feagi-bdu/                # Brain development unit (neurogenesis)
│   ├── feagi-evo/                # Genome I/O and evolution
│   ├── feagi-config/             # Configuration management
│   ├── feagi-pns/                # Peripheral nervous system (I/O)
│   ├── feagi-agent-sdk/          # Agent client library
│   ├── feagi-api/                # REST API layer
│   ├── feagi-services/           # Service layer
│   ├── feagi-observability/      # Logging, telemetry, profiling
│   ├── feagi-transports/         # Transport abstractions
│   ├── feagi-embedded/           # Platform abstraction for embedded targets
│   ├── feagi-runtime-std/        # Desktop/server runtime (Vec, Rayon)
│   ├── feagi-runtime-embedded/   # ESP32/RTOS runtime (fixed arrays, no_std)
│   └── feagi-connectome-serialization/ # Connectome persistence
└── target/
    └── release/                   # Build artifacts
```

## Architecture Decisions

### 1. Workspace-based Crate Organization

- **feagi-types**: Shared core types (no dependencies)
- **feagi-neural/feagi-synapse**: Platform-agnostic core algorithms (no_std compatible)
- **feagi-burst-engine**: Depends only on types and core algorithms (fast, pure Rust)
- **feagi-plasticity**: Separate crate for IP protection
- **feagi-runtime-std/embedded**: Runtime adapters for different platforms
- **feagi-python**: PyO3 bindings (depends on all above)

### 2. IP Separation

The plasticity crate is intentionally separate:
- Different license option support (Apache-2.0 or proprietary)
- Can be compiled/distributed separately
- Trait-based plugin system for runtime integration

### 3. Platform Abstraction Strategy

- **Core algorithms** (`feagi-neural`, `feagi-synapse`): Platform-agnostic, no_std compatible
- **Runtime adapters**: Platform-specific implementations (std vs embedded)
- **Target platforms**: Desktop, server, embedded (ESP32, Arduino, STM32), RTOS

### 4. Incremental Migration Strategy

1. **Phase 1** (Current): Synaptic propagation (the bottleneck)
2. **Phase 2**: Full burst engine phases
3. **Phase 3**: Connectome management
4. **Phase 4**: Complete FEAGI core

## Performance Target vs Achievement

### Python Baseline (from profiling)
```
Phase 1 (Injection):  163.84 ms ( 88.7%)
  └─ Synaptic Propagation: 161.07 ms (100% of Phase 1)
     └─ Numpy Processing:  164.67 ms ( 91.7%)
```

### Rust Implementation
- **Target**: <3ms (50-100x speedup)
- **Status**: Requires benchmarking with production data

## Building

```bash
# Build entire workspace (all crates)
cargo build --workspace --release

# Build specific crate
cargo build --release -p feagi-burst-engine

# Run tests
cargo test --workspace

# Generate documentation
cargo doc --workspace --open

# Run with GPU acceleration (WGPU)
cargo build --release -p feagi-burst-engine --features gpu

# Run with CUDA acceleration (NVIDIA only)
cargo build --release -p feagi-burst-engine --features cuda
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
feagi = "2.0"
```

For selective compilation with feature flags:

```toml
[dependencies]
feagi = { version = "2.0", features = ["compute"], default-features = false }
```

For individual crates:

```toml
[dependencies]
feagi-burst-engine = "2.0"
feagi-types = "2.0"
```

## Usage Examples

### Basic Neural Processing

```rust
use feagi::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create burst engine with capacity for 100k neurons
    let mut npu = RustNPU::new(100_000, 1_000_000, 20)?;
    
    // Load connectome
    npu.load_connectome("path/to/connectome.json")?;
    
    // Process burst
    npu.process_burst()?;
    
    Ok(())
}
```

### Embedded Target (no_std)

```rust
#![no_std]
use feagi_neural::NeuronDynamics;
use feagi_runtime_embedded::EmbeddedRuntime;

// Configure for ESP32 with fixed-size arrays
let runtime = EmbeddedRuntime::new(1000, 5000);
let mut dynamics = NeuronDynamics::new(&runtime);
```

## Python Integration

FEAGI Core provides PyO3 bindings for seamless Python integration:

```python
import feagi_rust

# Create engine
engine = feagi_rust.SynapticPropagationEngine()

# Build synapse index (once during initialization)
engine.build_index(
    source_neurons,   # np.array[uint32]
    target_neurons,   # np.array[uint32]
    weights,          # np.array[uint8, 0-255]
    conductances,     # np.array[uint8, 0-255]
    types,            # np.array[uint8, 0=excitatory, 1=inhibitory]
    valid_mask        # np.array[bool]
)

# Set neuron mapping
engine.set_neuron_mapping({neuron_id: cortical_area_id})

# Compute propagation (HOT PATH - called every burst!)
fired_neurons = np.array([1, 2, 3], dtype=np.uint32)
result = engine.propagate(fired_neurons)
# Returns: {area_id: [(target_neuron, contribution), ...], ...}
```

## Design Principles

### RTOS Compatibility
- No allocations in hot paths (pre-allocate during init)
- No locks/mutexes in critical sections
- Rayon for data parallelism (CPU-bound tasks)
- Conditional compilation for embedded targets

### Cache-Friendly Data Structures
- `#[repr(C)]` for predictable memory layout
- Array-of-Structures (AoS) for synapse data
- Sequential memory access patterns
- SIMD-friendly vectorized operations

### Type Safety
- Strong types instead of primitives (`NeuronId(u32)` not `u32`)
- Compile-time guarantees (no runtime type checks)
- Zero-cost abstractions

### Cross-Platform Support
- **Desktop/Server**: Linux, macOS, Windows
- **Embedded**: ESP32, Arduino, STM32, Teensy, Nordic nRF
- **Industrial**: ABB, Fanuc, KUKA robots (future)
- **Cloud**: Docker, Kubernetes, AWS, GCP, Azure

## Feature Flags

### Main Crate Features

```toml
[features]
default = ["std", "full"]

# Platform targets
std = [...]        # Desktop/server support
no_std = [...]     # Embedded support
wasm = [...]       # WebAssembly support

# Component features
full = ["compute", "io"]
compute = ["burst-engine", "bdu", "plasticity", "serialization"]
io = ["pns", "agent-sdk"]

# Individual components
burst-engine = [...]
bdu = [...]
plasticity = [...]
# ... and more
```

### Burst Engine Features

```toml
[features]
default = []
gpu = ["wgpu", "pollster", "bytemuck"]     # Cross-platform GPU
cuda = ["cudarc", "half"]                   # NVIDIA CUDA
all-gpu = ["gpu", "cuda"]                   # All GPU backends
```

## Publishing to crates.io

This workspace publishes multiple crates:

**Main facade crate:**
- `feagi` - Primary import for most users

**Individual component crates:**
- `feagi-types` - Core data structures
- `feagi-burst-engine` - NPU execution
- `feagi-state-manager` - Runtime state
- `feagi-bdu` - Neurogenesis
- `feagi-plasticity` - Synaptic learning
- `feagi-connectome-serialization` - Persistence
- `feagi-pns` - I/O layer
- `feagi-agent-sdk` - Client SDK
- ... and more

See [PUBLISHING.md](PUBLISHING.md) for detailed publishing strategy.

## Documentation

- **API Documentation**: [docs.rs/feagi](https://docs.rs/feagi)
- **Architecture**: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- **Publishing Guide**: [PUBLISHING.md](PUBLISHING.md)
- **Module-specific**: Each crate contains its own README.md

Generate local documentation:

```bash
cargo doc --workspace --open
```

## Contributing

Follow FEAGI's coding guidelines in `/docs/coding_guidelines.md`.

All Rust code must:
- Pass `cargo clippy` (zero warnings)
- Pass `cargo test` (100% passing)
- Include inline documentation
- Follow Rust API guidelines
- Support cross-platform compilation

### Development Workflow

```bash
# Check code
cargo check --workspace

# Run tests
cargo test --workspace

# Run clippy linter
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all

# Build release
cargo build --workspace --release
```

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p feagi-burst-engine

# Run benchmarks
cargo bench -p feagi-burst-engine

# Run with specific features
cargo test -p feagi-burst-engine --features gpu
```

## License

- **Most crates**: Apache-2.0
- **feagi-plasticity**: Apache-2.0 (with optional proprietary licensing for commercial use)

See [LICENSE](LICENSE) for details.

## Project Status

**Version**: 2.0.0  
**Status**: Active development  
**Rust Version**: 1.75+

### Crates.io Readiness

**Ready for publication:**
- feagi-types
- feagi-neural
- feagi-synapse
- feagi-state-manager
- feagi-burst-engine
- feagi-config
- feagi-observability
- feagi-embedded
- feagi-runtime-std
- feagi-runtime-embedded

**Requires fixes before publication:**
- feagi-evo (test compilation issues)
- feagi-pns (test compilation issues)

**Note**: Libraries compile successfully and documentation generates correctly. Test failures do not block crates.io publication but should be addressed.

## Links

- **Repository**: https://github.com/Neuraville/FEAGI-2.0
- **Website**: https://feagi.org
- **Documentation**: https://docs.feagi.org
- **Discord**: https://discord.gg/feagi
- **Email**: feagi@neuraville.com

---

**Built by the FEAGI team using Rust**
