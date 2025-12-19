# Publishing Strategy for feagi-core

## Structure Overview

`feagi-core` is a **workspace with a facade crate** that publishes both:
1. A **main facade crate** (`feagi`) - What users import
2. **Individual component crates** - Can be used independently

## What Gets Published to crates.io

### 1. Main Facade Crate

**Name**: `feagi`  
**Path**: Root of repository (`/Users/nadji/code/FEAGI-2.0/feagi-core/`)  
**Version**: `0.0.1`  
**Usage**: Primary import for most users

```toml
# Cargo.toml
[dependencies]
feagi = "0.0.1"  # ← Users import this
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
| `feagi-npu-neural` | 0.0.1 | Core neural types & algorithms | Library authors, embedded |
| `feagi-npu-runtime` | 0.0.1 | Runtime trait definitions | Platform implementers |
| `feagi-npu-runtime-std` | 0.0.1 | Desktop/server runtime | Standard applications |
| `feagi-npu-runtime-embedded` | 0.0.1 | Embedded runtime | ESP32, RTOS, no_std |
| `feagi-npu-burst-engine` | 0.0.1 | NPU execution engine | Inference-only apps |
| `feagi-npu-plasticity` | 0.0.1 | Synaptic learning (STDP) | Training, research |
| **Infrastructure** ||||
| `feagi-config` | 0.0.1 | Configuration loader | All applications |
| `feagi-state-manager` | 0.0.1 | Runtime state | Advanced integrations |
| `feagi-observability` | 0.0.1 | Logging & telemetry | Production deployments |
| `feagi-hal` | 0.0.1 | Platform HALs | Embedded platforms |
| **Algorithms** ||||
| `feagi-brain-development` | 0.0.1 | Neurogenesis | Training/development tools |
| `feagi-connectome-serialization` | 0.0.1 | Persistence | Model management tools |
| **I/O & Agent** ||||
| `feagi-io` | 0.0.1 | I/O layer | Agent bridges |
| `feagi-agent` | 0.0.1 | Client SDK | Agent developers |

**Advanced usage**:
```toml
# For inference-only application (minimal dependencies)
[dependencies]
feagi-npu-neural = "0.0.1"
feagi-npu-runtime-std = "0.0.1"
feagi-npu-burst-engine = "0.0.1"
feagi-connectome-serialization = "0.0.1"
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
cargo publish -p feagi-hal
cargo publish -p feagi-state-manager

# Phase 3: Data & Serialization
cargo publish -p feagi-data-serialization
cargo publish -p feagi-connectome-serialization

# Phase 4: Core algorithms
cargo publish -p feagi-npu-burst-engine
cargo publish -p feagi-npu-plasticity
cargo publish -p feagi-brain-development
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
feagi = "0.0.1"
```

```rust
use feagi::prelude::*;

let mut npu = RustNPU::new(100_000, 1_000_000, 20);
npu.process_burst()?;
```

### Selective Compilation: Feature Flags

```toml
[dependencies]
feagi = { version = "0.0.1", features = ["compute"], default-features = false }
```

```rust
// Only NPU + state (no I/O)
use feagi::burst_engine::RustNPU;
use feagi::state_manager::StateManager;
```

### Advanced: Direct Crate Dependencies

```toml
[dependencies]
feagi-burst-engine = "0.0.1"  # Direct dependency (bypass facade)
```

```rust
use feagi_burst_engine::RustNPU;
```

## Dependency Resolution

When users run:
```toml
[dependencies]
feagi = "0.0.1"
```

Cargo automatically resolves:
```
feagi 0.0.1
├── feagi-types 0.0.1
├── feagi-state-manager 0.0.1
├── feagi-burst-engine 0.0.1
├── feagi-brain-development 0.0.1
├── feagi-plasticity 0.0.1
├── feagi-connectome-serialization 0.0.1
├── feagi-io 0.0.1
└── feagi-agent 0.0.1
```

## Version Synchronization

**All crates MUST have synchronized versions**:
- Main crate: `0.0.1`
- All sub-crates: `0.0.1` (inherited from workspace)

**Update script**:
```bash
# Update all versions at once
cd /Users/nadji/code/FEAGI-2.0/feagi-core
# Update workspace version (most crates inherit from this)
sed -i 's/version = "0.0.1"/version = "0.0.2"/' Cargo.toml
# Update any crates with explicit versions
find crates -name Cargo.toml -exec sed -i 's/version = "0.0.1"/version = "0.0.2"/' {} \;
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
- Users get simple `feagi = "0.0.1"` import
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
- Selective dependencies: `feagi-burst-engine = "0.0.1"`

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
**Last Updated**: 2025-01-27  
**Version**: 0.0.1





