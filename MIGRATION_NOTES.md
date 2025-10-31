# Migration Notes for feagi-core

## Overview

This document tracks the migration plan for reorganizing FEAGI's crate structure according to the new architecture.

## Completed Migrations

### ✅ feagi-state-manager
- **Status**: Created (skeleton)
- **Location**: `feagi-core/crates/feagi-state-manager`
- **Date**: 2025-10-28
- **Notes**: New crate for runtime state management, replacing Python StateManager

## Pending Migrations

### 📋 feagi-pns → feagi-io

**Current Location**: `feagi-core/crates/feagi-pns`  
**Target Location**: `feagi-io/crates/feagi-pns` (new repository)  
**Reason**: PNS is I/O layer, should be separate from core neural computation

#### Migration Steps

1. **Create `feagi-io` repository**
   ```bash
   cd /Users/nadji/code/FEAGI-2.0
   mkdir feagi-io
   cd feagi-io
   cargo init --lib
   ```

2. **Move feagi-pns**
   ```bash
   git mv feagi-core/crates/feagi-pns feagi-io/crates/feagi-pns
   ```

3. **Update Dependencies**
   ```toml
   # feagi-io/Cargo.toml
   [workspace]
   members = ["crates/feagi-pns"]
   
   # feagi-io/crates/feagi-pns/Cargo.toml
   [dependencies]
   feagi-types = { git = "https://github.com/Neuraville/feagi-core", version = "2.0" }
   feagi-state-manager = { git = "https://github.com/Neuraville/feagi-core", version = "2.0" }
   feagi-burst-engine = { git = "https://github.com/Neuraville/feagi-core", version = "2.0" }
   ```

4. **Update Python Bindings**
   ```python
   # feagi-py/requirements.txt
   feagi-pns >= 2.0.0  # Now from feagi-io repo
   ```

5. **Update Documentation**
   - Update ARCHITECTURE.md
   - Update README.md references
   - Update import paths in examples

#### Affected Components
- ✅ **feagi-burst-engine**: Uses PNS for agent communication
- ✅ **feagi-inference-engine**: Uses PNS for ZMQ streams
- ✅ **feagi-py**: Python bindings reference PNS
- ✅ **feagi-bridge**: Subscribes to PNS visualization stream

#### Testing Checklist
- [ ] feagi-inference-engine still builds
- [ ] feagi-py bindings work
- [ ] feagi-bridge connects successfully
- [ ] All integration tests pass

---

### 📋 feagi-agent-sdk → feagi-io or feagi-connector

**Current Location**: `feagi-core/crates/feagi-agent-sdk`  
**Target Location**: TBD (either `feagi-io/crates/feagi-agent-sdk` or merge with `feagi-connector`)  
**Reason**: Agent SDK is for building agents (I/O layer), not core computation

#### Decision Required

**Option A**: Move to `feagi-io`
- Pro: Keeps all Rust I/O code together
- Pro: Clear separation from Python connector
- Con: Two agent SDKs (Rust + Python)

**Option B**: Merge with `feagi-connector`
- Pro: Unified agent SDK across languages
- Pro: Single source of truth for agent API
- Con: Mixing Rust + Python in one repo
- Con: feagi-connector is currently Python-only

**Recommendation**: **Option A** - Move to `feagi-io`
- Maintain clear language boundaries
- Rust SDK evolves independently of Python SDK
- Both SDKs can share the same protocol specification

#### Migration Steps (Option A)

1. **Move to feagi-io**
   ```bash
   git mv feagi-core/crates/feagi-agent-sdk feagi-io/crates/feagi-agent-sdk
   ```

2. **Update feagi-io workspace**
   ```toml
   [workspace]
   members = [
       "crates/feagi-pns",
       "crates/feagi-agent-sdk",
   ]
   ```

3. **Update Documentation**
   - Update examples to reference new location
   - Create migration guide for existing Rust agent developers

---

## Repository Structure After Migration

```
/Users/nadji/code/FEAGI-2.0/
│
├── feagi-data-processing/         # Foundation (separate repo)
│   └── Data structures, serialization
│
├── feagi-core/                    # Core neural computation (this repo)
│   ├── feagi-types/
│   ├── feagi-state-manager/       # ← NEW
│   ├── feagi-burst-engine/
│   ├── feagi-bdu/
│   ├── feagi-plasticity/
│   ├── feagi-connectome-serialization/
│   └── feagi-inference-engine/    # Application
│
├── feagi-io/                      # I/O layer (NEW repo)
│   ├── feagi-pns/                 # ← MOVED from feagi-core
│   └── feagi-agent-sdk/           # ← MOVED from feagi-core
│
├── feagi-py/                      # Python bindings (separate repo)
├── feagi-connector/               # Python agent SDK (separate repo)
├── feagi-bridge/                  # Bridge service (separate repo)
├── brain-visualizer/              # Visualization (separate repo)
└── ...
```

## Dependency Graph After Migration

```
feagi-data-processing (foundation)
        ↓
    feagi-types (feagi-core)
        ↓
    feagi-state-manager (feagi-core)
        ↓
┌───────────────────────────────────────────────┐
│  feagi-core (pure computation)                │
│   ├── feagi-burst-engine                      │
│   ├── feagi-bdu                               │
│   ├── feagi-plasticity                        │
│   └── feagi-connectome-serialization          │
└───────────────────────────────────────────────┘
        ↓
┌───────────────────────────────────────────────┐
│  feagi-io (I/O layer)                         │
│   ├── feagi-pns                               │
│   └── feagi-agent-sdk                         │
└───────────────────────────────────────────────┘
        ↓
    Applications
    ├── feagi-inference-engine (Rust)
    ├── feagi-py (Python bindings)
    ├── feagi-connector (Python SDK)
    └── brain-visualizer (Godot + Rust)
```

## Timeline

### Phase 1: Foundation (Week 1-2)
- ✅ Create feagi-state-manager skeleton
- ✅ Update feagi-core workspace organization
- ✅ Document architecture

### Phase 2: State Manager Implementation (Week 3-6)
- 🚧 Implement memory-mapped state
- 🚧 Implement agent registry
- 🚧 Implement cortical locks
- 🚧 Add Python bindings

### Phase 3: Repository Split (Week 7-8)
- 📋 Create feagi-io repository
- 📋 Move feagi-pns
- 📋 Move feagi-agent-sdk
- 📋 Update all cross-repo dependencies

### Phase 4: Testing & Documentation (Week 9-10)
- 📋 Integration testing across repositories
- 📋 Update all documentation
- 📋 Create migration guides
- 📋 Publish to crates.io

## Breaking Changes

### For Rust Developers

**Before**:
```rust
use feagi_core::feagi_pns::PNS;
```

**After**:
```rust
use feagi_io::feagi_pns::PNS;
```

**Migration**: Update `Cargo.toml` dependencies:
```toml
[dependencies]
# feagi-core = "2.0"  # Old
feagi-io = "2.0"      # New
```

### For Python Developers

**Before**:
```python
from feagi_rust import PyPNS
```

**After**:
```python
from feagi_rust import PyPNS  # No change - bindings updated internally
```

**Migration**: Update `requirements.txt`:
```
feagi-rust >= 2.0.0  # Rebuilt with new feagi-io dependency
```

## Questions & Decisions

### Q1: Should feagi-io be a monorepo or separate repos for each crate?
**Answer**: Monorepo (like feagi-core)
- Easier to maintain version synchronization
- Simpler dependency management
- Clear I/O layer boundary

### Q2: How to handle cross-repo dependencies during development?
**Answer**: Use git dependencies during development, crates.io after publishing
```toml
# Development
feagi-core = { git = "https://github.com/Neuraville/feagi-core", branch = "main" }

# Production
feagi-core = "2.0"
```

### Q3: Should we keep backward compatibility during migration?
**Answer**: No - clean break for 2.0 release
- Document breaking changes clearly
- Provide migration guides
- Ensure all examples are updated

## Rollback Plan

If migration causes critical issues:

1. **Revert repository split**
   ```bash
   git revert <migration-commit>
   ```

2. **Restore feagi-pns to feagi-core**
   ```bash
   git mv feagi-io/crates/feagi-pns feagi-core/crates/feagi-pns
   ```

3. **Update all documentation** to reflect rollback

4. **Communicate** to all developers via:
   - GitHub announcement
   - Discord/Slack notification
   - Email to contributors

---

**Last Updated**: 2025-10-28  
**Status**: Planning Phase  
**Next Action**: Implement feagi-state-manager core functionality




