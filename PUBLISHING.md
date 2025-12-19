# Publishing Strategy for feagi-core

## Structure Overview

`feagi-core` is a **workspace with a facade crate** that publishes both:
1. A **main facade crate** (`feagi`) - What users import
2. **Individual component crates** - Can be used independently

## What Gets Published to crates.io

### 1. Main Facade Crate

**Name**: `feagi`  
**Path**: Root of repository (`/Users/nadji/code/FEAGI-2.0/feagi-core/`)  
**Version**: `2.0.0`  
**Usage**: Primary import for most users

```toml
# Cargo.toml
[dependencies]
feagi = "2.0"  # ← Users import this
```

```rust
// Full FEAGI (default)
use feagi::prelude::*;

// Selective features
use feagi::burst_engine::RustNPU;
use feagi::state_manager::StateManager;
```

### 2. Individual Component Crates

These are **also published separately** for advanced use cases:

| Crate | Version | Description | Users Who Need It |
|-------|---------|-------------|------------------|
| **NPU Subsystem** ||||
| `feagi-npu-neural` | 2.0.0 | Core neural types & algorithms | Library authors, embedded |
| `feagi-npu-runtime` | 2.0.0 | Runtime trait definitions | Platform implementers |
| `feagi-npu-runtime-std` | 2.0.0 | Desktop/server runtime | Standard applications |
| `feagi-npu-runtime-embedded` | 2.0.0 | Embedded runtime | ESP32, RTOS, no_std |
| `feagi-npu-burst-engine` | 2.0.0 | NPU execution engine | Inference-only apps |
| `feagi-npu-plasticity` | 2.0.0 | Synaptic learning (STDP) | Training, research |
| **Infrastructure** ||||
| `feagi-config` | 2.0.0 | Configuration loader | All applications |
| `feagi-state-manager` | 2.0.0 | Runtime state | Advanced integrations |
| `feagi-observability` | 2.0.0 | Logging & telemetry | Production deployments |
| `feagi-embedded` | 2.0.0 | Platform HALs | Embedded platforms |
| **Algorithms** ||||
| `feagi-bdu` | 2.0.0 | Neurogenesis | Training/development tools |
| `feagi-connectome-serialization` | 2.0.0 | Persistence | Model management tools |
| **I/O & Agent** ||||
| `feagi-io` | 2.0.0 | I/O layer | Agent bridges |
| `feagi-agent` | 2.0.0 | Client SDK | Agent developers |

**Advanced usage**:
```toml
# For inference-only application (minimal dependencies)
[dependencies]
feagi-npu-neural = "2.0"
feagi-npu-runtime-std = "2.0"
feagi-npu-burst-engine = "2.0"
feagi-connectome-serialization = "2.0"
```

## Publishing Commands

### Publish All Crates (Correct Order)

```bash
cd /Users/nadji/code/FEAGI-2.0/feagi-core

# Phase 1: Foundation (no internal dependencies)
cargo publish -p feagi-npu-neural
cargo publish -p feagi-npu-runtime
cargo publish -p feagi-config
cargo publish -p feagi-observability
cargo publish -p feagi-data-structures

# Phase 2: Runtime implementations
cargo publish -p feagi-npu-runtime-std
cargo publish -p feagi-npu-runtime-embedded
cargo publish -p feagi-embedded
cargo publish -p feagi-state-manager

# Phase 3: Data & Serialization
cargo publish -p feagi-data-serialization
cargo publish -p feagi-connectome-serialization

# Phase 4: Core algorithms
cargo publish -p feagi-npu-burst-engine
cargo publish -p feagi-npu-plasticity
cargo publish -p feagi-bdu
cargo publish -p feagi-evolutionary

# Phase 5: I/O & Transport
cargo publish -p feagi-transports
cargo publish -p feagi-io
cargo publish -p feagi-connector-core

# Phase 6: Services & API
cargo publish -p feagi-services
cargo publish -p feagi-agent
cargo publish -p feagi-api

# Phase 7: Main facade crate (re-exports everything)
cargo publish
```

### Dry Run (Test Before Publishing)

```bash
cargo publish --dry-run -p feagi-npu-neural
cargo publish --dry-run -p feagi-npu-runtime
cargo publish --dry-run -p feagi-npu-runtime-std
cargo publish --dry-run -p feagi-npu-burst-engine
# ... etc
cargo publish --dry-run  # Main crate
```

## User Experience

### Most Common: Use Main Crate

```toml
[dependencies]
feagi = "2.0"
```

```rust
use feagi::prelude::*;

let mut npu = RustNPU::new(100_000, 1_000_000, 20);
npu.process_burst()?;
```

### Selective Compilation: Feature Flags

```toml
[dependencies]
feagi = { version = "2.0", features = ["compute"], default-features = false }
```

```rust
// Only NPU + state (no I/O)
use feagi::burst_engine::RustNPU;
use feagi::state_manager::StateManager;
```

### Advanced: Direct Crate Dependencies

```toml
[dependencies]
feagi-burst-engine = "2.0"  # Direct dependency (bypass facade)
```

```rust
use feagi_burst_engine::RustNPU;
```

## Dependency Resolution

When users run:
```toml
[dependencies]
feagi = "2.0"
```

Cargo automatically resolves:
```
feagi 2.0.0
├── feagi-types 2.0.0
├── feagi-state-manager 2.0.0
├── feagi-burst-engine 2.0.0
├── feagi-bdu 2.0.0
├── feagi-plasticity 2.0.0
├── feagi-connectome-serialization 2.0.0
├── feagi-io 2.0.0
└── feagi-agent 2.0.0
```

## Version Synchronization

**All crates MUST have synchronized versions**:
- Main crate: `2.0.0`
- All sub-crates: `2.0.0`

**Update script**:
```bash
# Update all versions at once
cd /Users/nadji/code/FEAGI-2.0/feagi-core
find crates -name Cargo.toml -exec sed -i 's/version = "2.0.0"/version = "2.1.0"/' {} \;
sed -i 's/version = "2.0.0"/version = "2.1.0"/' Cargo.toml
```

## Verification

### Check Structure
```bash
cd /Users/nadji/code/FEAGI-2.0/feagi-core

# Verify main crate
cargo check --lib

# Verify all workspace members
cargo check --workspace

# Check individual crates
cargo check -p feagi-types
cargo check -p feagi-state-manager
# ... etc
```

### Test Publishing (Local)
```bash
# Create local registry
mkdir -p ~/.cargo/local-registry

# Publish to local registry (dry run)
cargo publish --dry-run
```

## Comparison with Other Strategies

### ❌ Wrong: Single Publishable Crate
```
feagi-core/
└── src/
    └── lib.rs  # Everything in one crate
```
**Problem**: Can't selectively depend on components

### ❌ Wrong: Separate Repos
```
feagi-types/      (separate repo)
feagi-engine/     (separate repo)
...
```
**Problem**: Hard to maintain version sync

### ✅ Correct: Workspace + Facade
```
feagi-core/
├── src/lib.rs              # Main facade crate
├── Cargo.toml              # Workspace + facade manifest
└── crates/
    ├── feagi-types/        # Published separately
    ├── feagi-state-manager/ # Published separately
    └── ...
```
**Benefits**:
- Users get simple `feagi = "2.0"` import
- Advanced users can cherry-pick components
- Single repository for maintenance
- Synchronized versions

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Publish to crates.io

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Publish feagi-types
        run: cargo publish -p feagi-types --token ${{ secrets.CRATES_IO_TOKEN }}
      
      - name: Publish feagi-state-manager
        run: cargo publish -p feagi-state-manager --token ${{ secrets.CRATES_IO_TOKEN }}
      
      # ... publish other crates in order ...
      
      - name: Publish main feagi crate
        run: cargo publish --token ${{ secrets.CRATES_IO_TOKEN }}
```

## Documentation

### Main Crate Docs
```bash
cargo doc --open
```

### Individual Crate Docs
```bash
cargo doc -p feagi-burst-engine --open
```

### docs.rs
All crates will automatically have documentation at:
- https://docs.rs/feagi/
- https://docs.rs/feagi-types/
- https://docs.rs/feagi-burst-engine/
- etc.

## FAQs

### Q: Why not publish as a single crate?
**A**: We want to support both:
- Simple import: `use feagi::prelude::*;`
- Selective dependencies: `feagi-burst-engine = "2.0"`

### Q: Will users accidentally get duplicate dependencies?
**A**: No. Cargo deduplicates. Whether they use `feagi` or `feagi-burst-engine`, they get the same binary.

### Q: How do I update all versions at once?
**A**: Use the script in "Version Synchronization" section above.

### Q: Can I use git dependencies during development?
**A**: Yes:
```toml
[dependencies]
feagi = { git = "https://github.com/Neuraville/FEAGI-2.0", branch = "main" }
```

---

**Status**: ✅ Structure verified, ready for publishing  
**Last Updated**: 2025-10-28  
**Version**: 2.0.0





