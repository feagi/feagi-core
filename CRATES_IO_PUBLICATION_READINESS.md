# Crates.io Publication Readiness Review

**Date:** December 4, 2025  
**Project:** FEAGI (Framework for Evolutionary Artificial General Intelligence)  
**Repository:** https://github.com/feagi/feagi-core

---

## Executive Summary

**Status:** ⚠️ **Needs Fixes Before Publication**

The workspace structure is good, but several critical issues must be resolved before publishing to crates.io.

---

## Critical Issues (Must Fix)

### 1. Missing README Files ❌

The three merged crates from feagi-data-processing **lack README.md files**:

- `crates/feagi_data_structures/` - **NO README**
- `crates/feagi_data_serialization/` - **NO README**  
- `crates/feagi_connector_core/` - **NO README**

**Impact:** Crates.io requires a README or will use the description field only.  
**Action Required:** Create README.md files for each crate.

### 2. Inconsistent Repository URLs ❌

**Current Issues:**
- Workspace package: `https://github.com/Neuraville/FEAGI-2.0` ❌
- feagi-transports: `https://github.com/feagi/feagi` ❌
- feagi-state-manager: `https://github.com/Neuraville/FEAGI-2.0` ❌
- feagi-config: `https://github.com/Neuraville/FEAGI-2.0` ❌

**Should All Be:**
```toml
repository = "https://github.com/feagi/feagi-core"
```

**Action Required:** Standardize all repository URLs.

### 3. Compilation Errors ❌

The workspace currently has compilation errors in `feagi-bdu` due to API incompatibilities with the merged `feagi_data_structures` crate.

**Impact:** `cargo publish` will fail if the code doesn't compile.  
**Action Required:** Fix API compatibility issues (see FEAGI_EVO_COMPATIBILITY_FIXES.md).

---

## Recommendations (Best Practices)

### 4. Standardize Package Metadata ⚠️

**Current State:** Mixed inheritance patterns
- Some crates use `version.workspace = true`
- Some have explicit `version = "2.0.0"`
- Some have `authors.workspace = true`, some don't

**Recommendation:** Use workspace inheritance consistently for all shared metadata:

```toml
[package]
name = "feagi-xxx"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
description = "Specific description for this crate"
```

### 5. Add Keywords and Categories 📋

For better discoverability on crates.io, add:

```toml
keywords = ["feagi", "neuroscience", "ai", "neural-network", "brain"]
categories = ["science", "simulation"]
```

Suggested keywords by crate type:
- **Data crates:** `["feagi", "data-structures", "neuroscience", "serialization"]`
- **Algorithm crates:** `["feagi", "neural-network", "ai", "brain", "computation"]`
- **I/O crates:** `["feagi", "zmq", "networking", "agent-sdk"]`

### 6. Verify Documentation Links 📚

Ensure documentation field points to correct location:
```toml
documentation = "https://docs.rs/feagi"  # Workspace
documentation = "https://docs.rs/feagi-xxx"  # Individual crates
```

---

## Publication Strategy

### Publishing Order (Dependencies First)

Due to dependency relationships, crates must be published in this order:

#### Phase 1: Foundation (No Dependencies)
1. `feagi_data_structures`
2. `feagi-neural`
3. `feagi-runtime`

#### Phase 2: Infrastructure  
4. `feagi_data_serialization` (depends on feagi_data_structures)
5. `feagi-state-manager`
6. `feagi-runtime-std`
7. `feagi-runtime-embedded`
8. `feagi-observability`

#### Phase 3: Core Algorithms
9. `feagi_connector_core` (depends on data crates)
10. `feagi-burst-engine`
11. `feagi-bdu`
12. `feagi-plasticity`
13. `feagi-connectome-serialization`

#### Phase 4: Services & I/O
14. `feagi-evo`
15. `feagi-transports`
16. `feagi-io`
17. `feagi-agent`
18. `feagi-services`
19. `feagi-api`

#### Phase 5: Umbrella Crate
20. `feagi` (umbrella - depends on all published crates)

---

## Workspace Configuration Status

### ✅ Good Configuration

- **License:** Apache-2.0 (OSI approved, crates.io compatible)
- **Edition:** 2021 (current stable)
- **No publish = false:** All crates are eligible for publication
- **Workspace resolver:** "2" (correct for 2021 edition)
- **Version:** 2.0.0 (valid semantic version)

### Current Workspace Package Settings

```toml
[workspace.package]
version = "2.0.0"
edition = "2021"
authors = ["Neuraville Inc. <feagi@neuraville.com>"]
license = "Apache-2.0"
repository = "https://github.com/Neuraville/FEAGI-2.0"  # ❌ NEEDS FIX
homepage = "https://feagi.org"  # ✅ Good
documentation = "https://docs.rs/feagi"  # ✅ Good
```

---

## Checklist Before Publication

- [ ] Create README.md for feagi_data_structures
- [ ] Create README.md for feagi_data_serialization  
- [ ] Create README.md for feagi_connector_core
- [ ] Fix repository URLs (workspace + individual crates)
- [ ] Fix compilation errors in feagi-bdu
- [ ] Standardize package metadata inheritance
- [ ] Add keywords/categories to all crates
- [ ] Verify all crates compile: `cargo check --workspace`
- [ ] Verify all tests pass: `cargo test --workspace`
- [ ] Test umbrella crate features: `cargo test --features full`
- [ ] Dry run publish: `cargo publish --dry-run` for each crate
- [ ] Verify docs build: `cargo doc --no-deps --workspace`

---

## Umbrella Crate Strategy

The current `feagi` umbrella crate is well-designed:

✅ **Strengths:**
- Clear feature flags for selective compilation
- Optional dependencies for modular builds
- Platform target features (std, no_std, wasm)
- Good documentation structure

**Publishing Note:** The umbrella crate should be published LAST, after all dependencies are on crates.io.

---

## Naming Convention

**Current Pattern:** Mix of `feagi-xxx` (hyphenated) and `feagi_xxx` (underscored)

**Crates with underscores:**
- `feagi_data_structures` ✅ (library name convention)
- `feagi_data_serialization` ✅ (library name convention)
- `feagi_connector_core` ✅ (library name convention)

**Recommendation:** Keep current naming - underscores are acceptable for library crates.

---

## Estimated Publication Timeline

**Pre-publication work:** 1-2 days
- Create 3 README files
- Fix repository URLs  
- Fix compilation errors
- Standardize metadata

**Publication process:** 2-3 hours (sequential publishing of 20+ crates)

**Total:** ~2-3 days for complete publication

---

## Next Steps

1. **Immediate:** Create README files for merged crates
2. **High Priority:** Fix repository URLs
3. **Critical:** Resolve compilation errors
4. **Before Publish:** Run full test suite
5. **Publication Day:** Publish in dependency order

---

## Contact & Support

- **Project Homepage:** https://feagi.org
- **Repository:** https://github.com/feagi/feagi-core  
- **Documentation:** https://docs.rs/feagi
- **Crates.io:** https://crates.io/crates/feagi (after publication)

