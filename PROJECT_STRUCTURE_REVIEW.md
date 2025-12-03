# feagi-core Project Structure Review

**Date**: December 3, 2025  
**Purpose**: Verify workspace setup for multi-crate publication with "feagi" umbrella

---

## Current State Assessment

### ❌ ISSUE 1: Incorrect Umbrella Crate Name

**Current**:
```toml
[package]
name = "feagi-core"  # ← Wrong for umbrella
```

**Should be**:
```toml
[package]
name = "feagi"  # ← Umbrella crate (re-exports everything)
```

**Problem**: Users will do `cargo add feagi-core` instead of clean `cargo add feagi`

---

### ❌ ISSUE 2: Wrong Publishing Strategy

**Current state**:
- Umbrella (feagi-core): `publish = allowed` ✅
- All workspace members: `publish = false` ❌

**This creates**: Single crate publication (what we decided to AVOID)

**Should be** (for multi-crate ecosystem):
- Umbrella (feagi): `publish = true` ✅
- All workspace members: `publish = true` ✅ (REMOVE `publish = false`)

---

### ❌ ISSUE 3: Missing Individual Crate READMEs

**Current**: Only root README.md

**Need**: Each publishable crate needs its own `README.md`:
```
crates/feagi-types/README.md        ✅ EXISTS
crates/feagi-burst-engine/README.md ❌ MISSING  
crates/feagi-bdu/README.md          ❌ MISSING
...
```

**Impact**: Poor crates.io presentation for individual crates

---

### ✅ GOOD: Workspace Structure

```
feagi-core/
├── Cargo.toml           # Workspace + umbrella crate ✅
├── src/lib.rs           # Umbrella re-exports ✅
└── crates/              # Member crates ✅
    ├── feagi-types/     ✅
    ├── feagi-burst-engine/ ✅
    └── ...
```

**Correct pattern** per best practices doc

---

### ✅ GOOD: Feature Flags in Umbrella

```toml
[features]
default = ["std", "full"]
compute = ["burst-engine", "bdu", "plasticity"]
io = ["pns", "agent-sdk"]
```

**Allows selective compilation** ✅

---

### ✅ GOOD: Umbrella Re-exports

```rust
pub use feagi_types as types;
pub use feagi_burst_engine as burst_engine;
pub mod prelude { ... }
```

**Clean API** ✅

---

## Required Changes for Multi-Crate Publication

### Change 1: Rename Umbrella Crate

**File**: `Cargo.toml` (root)

```diff
- name = "feagi-core"
+ name = "feagi"
```

**File**: `README.md`

```diff
- # FEAGI Core
+ # FEAGI
- cargo add feagi-core
+ cargo add feagi
```

**Result**: Clean umbrella name

---

### Change 2: Remove `publish = false` from All Workspace Members

**Script**:
```bash
find crates -name Cargo.toml -exec sed -i '' '/^publish = false/d' {} \;
```

**Or manually** in each `crates/*/Cargo.toml`:
```diff
- publish = false
+ # (remove line)
```

**Result**: All crates can be published

---

### Change 3: Add README.md to Each Crate

**Template** for `crates/feagi-{name}/README.md`:
```markdown
# feagi-{name}

{Brief description}

## Installation

\`\`\`toml
[dependencies]
feagi-{name} = "2.0"
\`\`\`

## Usage

\`\`\`rust
use feagi_{name}::prelude::*;
\`\`\`

Part of the [FEAGI](https://github.com/feagi/feagi-core) ecosystem.
```

**Benefit**: Each crate looks professional on crates.io

---

### Change 4: Update Repository URL Pattern

**For consistency**, each crate's Cargo.toml should point to main repo:

```toml
repository = "https://github.com/feagi/feagi-core"  # All point to same repo
```

**Already correct** ✅

---

### Change 5: Separate Documentation

**Root README.md**: Overview of entire ecosystem  
**crates/*/README.md**: Specific crate usage

**Example**:

**Root README.md**:
```markdown
# FEAGI - Bio-Inspired Neural Framework

High-level overview...

## Crates

- **feagi-types**: Core data structures
- **feagi-burst-engine**: Neural processing
...

## Installation

Most users:
\`\`\`toml
feagi = "2.0"  # Everything
\`\`\`

Selective:
\`\`\`toml
feagi-burst-engine = "2.0"  # Just what you need
\`\`\`
```

**crates/feagi-burst-engine/README.md**:
```markdown
# feagi-burst-engine

Specific details about burst engine...
```

---

## Correct Multi-Crate Setup Checklist

- [ ] Rename umbrella to `feagi` (not `feagi-core`)
- [ ] Remove `publish = false` from all workspace members
- [ ] Add README.md to each crate (19 files)
- [ ] Update root README to describe ecosystem
- [ ] Verify all crates have proper metadata (description, keywords)
- [ ] Test: `cargo publish --dry-run` on each crate
- [ ] Update CI/CD to publish in dependency order

---

## Publishing Order (After Changes)

```bash
# Foundation (no dependencies)
cargo publish -p feagi-types
cargo publish -p feagi-config
cargo publish -p feagi-observability
cargo publish -p feagi-state-manager

# Core algorithms
cargo publish -p feagi-neural
cargo publish -p feagi-synapse
cargo publish -p feagi-connectome-serialization

# Runtime adapters
cargo publish -p feagi-runtime-std
cargo publish -p feagi-runtime-embedded
cargo publish -p feagi-embedded

# Processing engines
cargo publish -p feagi-burst-engine
cargo publish -p feagi-bdu
cargo publish -p feagi-plasticity
cargo publish -p feagi-evo

# I/O layer
cargo publish -p feagi-transports
cargo publish -p feagi-pns
cargo publish -p feagi-agent-sdk

# Service layer
cargo publish -p feagi-services
cargo publish -p feagi-api

# Finally, umbrella
cargo publish  # Publishes "feagi"
```

---

## Summary

**Current setup is ALMOST correct** for multi-crate publication, but has 3 critical issues:

1. ❌ Umbrella named `feagi-core` (should be `feagi`)
2. ❌ All members have `publish = false` (should publish ALL)
3. ❌ Missing individual crate READMEs

**Fix these 3 things** and you're ready for proper multi-crate ecosystem publication.

---

**Recommendation**: Fix these issues BEFORE restructuring crates. Get the publication mechanism right first, then optimize the crate boundaries.

