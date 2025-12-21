# FEAGI-Core Independent Versioning System

**Last Updated:** December 2025  
**Status:** ✅ Implemented and Active

---

## Overview

FEAGI-Core uses **smart independent versioning** for its 18 workspace crates. Each crate maintains its own version number, and only crates with actual changes (or dependency updates) receive version bumps.

This approach provides:
- ✅ **Semantic clarity**: Version numbers reflect actual changes
- ✅ **Reduced noise**: No version pollution from unchanged crates
- ✅ **Better dependency management**: Users can selectively update
- ✅ **Clear audit trail**: Easy to see what changed between versions
- ✅ **Faster releases**: Only publish what changed

---

## How It Works

### 1. Change Detection

When a PR is merged to `staging`, the CI system:

1. **Detects changed files** since the last release tag
2. **Identifies affected crates** by analyzing which crate directories have changes
3. **Propagates changes** to dependent crates (if `feagi-npu-neural` changes, `feagi-npu-burst-engine` must also bump)
4. **Computes new version numbers** for each changed crate independently

### 2. Version Incrementing

Each crate maintains its **own beta counter**:

```
feagi-npu-neural:       0.0.1-beta.5
feagi-npu-burst-engine: 0.0.1-beta.3
feagi-io:               0.0.1-beta.8
feagi-api:              0.0.1-beta.2
```

**Beta version logic:**
- Query crates.io for highest published beta for that crate
- Increment by 1
- Example: If `feagi-io` is at `0.0.1-beta.7`, next version is `0.0.1-beta.8`

### 3. Dependency Updates

When a crate's version changes, all `workspace.dependencies` are updated with **exact version requirements**:

```toml
[workspace.dependencies]
feagi-npu-neural = { version = "=0.0.1-beta.5", path = "crates/feagi-npu/neural" }
feagi-npu-burst-engine = { version = "=0.0.1-beta.3", path = "crates/feagi-npu/burst-engine" }
```

**Why exact versions (`=X.Y.Z-beta.N`)?**
- ✅ **Pre-1.0 stability**: Breaking changes expected between betas
- ✅ **Explicit control**: No surprises from automatic updates
- ✅ **Reproducible builds**: Same `Cargo.lock` on all systems
- ✅ **CI automation**: Scripts handle updates automatically

**Post-1.0 strategy:**
After reaching stable 1.0, we'll switch to semver ranges (`^1.0.0`) for patch/minor compatibility.

---

## Architecture

### Scripts

#### `scripts/smart-version-bump.sh`
**Purpose:** Detect changes and compute new versions

**Algorithm:**
1. Get last release tag from git
2. Run `git diff` on each crate directory
3. Mark directly changed crates
4. Propagate changes to dependents (transitive)
5. Query crates.io for current published versions
6. Increment beta numbers independently per crate
7. Output version manifest

**Usage:**
```bash
# Detect changes and compute versions
./scripts/smart-version-bump.sh

# Use custom tag as baseline
LAST_TAG=v0.0.1-beta.2 ./scripts/smart-version-bump.sh

# Dry run
DRY_RUN=true ./scripts/smart-version-bump.sh
```

**Output:**
- Human-readable summary of changes
- Environment variable exports for automation
- List of changed crates

#### `scripts/apply-version-bumps.sh`
**Purpose:** Apply computed versions to Cargo.toml files

**Actions:**
1. Update individual crate `Cargo.toml` files (if explicit version)
2. Update `[workspace.package]` version (if root crate changed)
3. Update `[workspace.dependencies]` with exact versions

**Usage:**
```bash
# Source version data from smart-version-bump.sh
source $VERSIONS_FILE
./scripts/apply-version-bumps.sh

# Dry run
DRY_RUN=true ./scripts/apply-version-bumps.sh
```

#### `scripts/publish-crates-smart.sh`
**Purpose:** Publish only changed crates to crates.io

**Features:**
- Respects dependency order (publishes dependencies first)
- Skips unchanged crates
- 30-second delay between publishes for crates.io indexing
- Checks if version already published (idempotent)
- Validates packaging before publishing

**Usage:**
```bash
export CARGO_REGISTRY_TOKEN="your-token"
export CHANGED_CRATES="feagi-npu-neural feagi-npu-burst-engine feagi-api"
./scripts/publish-crates-smart.sh

# Dry run
DRY_RUN=true ./scripts/publish-crates-smart.sh
```

---

## CI/CD Integration

### Staging Branch Workflow

File: `.github/workflows/staging-merge.yml`

**Trigger:** PR merged to `staging`

**Steps:**
1. ✅ **Checkout code** (staging branch)
2. ✅ **Run tests** (all workspace tests)
3. ✅ **Build release** (verify compilation)
4. 🆕 **Smart version detection** (`smart-version-bump.sh`)
5. 🆕 **Apply version bumps** (`apply-version-bumps.sh`)
6. 🆕 **Publish changed crates** (`publish-crates-smart.sh`)
7. ✅ **Commit version updates** (back to staging)
8. ✅ **Create release tag** (timestamp-based)
9. ✅ **Create GitHub prerelease** (with changelog)

**Key Changes from Old System:**
- ❌ **Old:** Bump ALL crates to same version
- ✅ **New:** Bump ONLY changed crates independently
- ❌ **Old:** Publish all 19 crates every time (~15 minutes)
- ✅ **New:** Publish only changed crates (~2-5 minutes)
- ❌ **Old:** Version-based tags (`v0.0.1-beta.3`)
- ✅ **New:** Timestamp-based tags (`staging-20251221-143045`)

---

## Dependency Graph

Understanding the dependency graph is critical for propagating version changes:

```
Layer 1 (Foundation):
  └─ feagi-observability

Layer 2 (Core Data):
  ├─ feagi-data-structures → observability
  └─ feagi-config → observability

Layer 3 (Neural):
  └─ feagi-npu-neural → observability, data-structures

Layer 4 (Runtime):
  └─ feagi-npu-runtime → npu-neural

Layer 5 (Serialization):
  ├─ feagi-data-serialization → data-structures
  └─ feagi-state-manager → observability, data-structures, config

Layer 6 (Processing):
  ├─ feagi-npu-burst-engine → npu-neural, npu-runtime, data-serialization, state-manager
  └─ feagi-npu-plasticity → npu-neural

Layer 7 (Evolution):
  ├─ feagi-evolutionary → npu-neural, data-structures, observability
  └─ feagi-brain-development → npu-neural, burst-engine, evolutionary

Layer 8 (I/O):
  ├─ feagi-io → burst-engine, brain-development, services, data-serialization
  └─ feagi-sensorimotor → data-structures, data-serialization

Layer 9 (Services):
  ├─ feagi-services → state-manager, burst-engine, brain-development, evolutionary
  └─ feagi-api → services, io, evolutionary, brain-development, burst-engine

Layer 10 (Platform):
  ├─ feagi-agent → io, data-structures, data-serialization, observability
  └─ feagi-hal → npu-runtime, npu-neural, observability

Root:
  └─ feagi (meta-crate) → ALL above crates
```

**Propagation Example:**

If you change `feagi-npu-neural`, these crates MUST also bump:
- `feagi-npu-runtime` (direct dependency)
- `feagi-npu-burst-engine` (depends on npu-neural)
- `feagi-npu-plasticity` (depends on npu-neural)
- `feagi-evolutionary` (depends on npu-neural)
- `feagi-brain-development` (transitive via evolutionary + burst-engine)
- `feagi-services` (transitive via brain-development)
- `feagi-io` (transitive via services + burst-engine)
- `feagi-api` (transitive via services + io)
- `feagi-agent` (transitive via io)
- `feagi-hal` (depends on npu-neural)
- `feagi` (root meta-crate)

**Total: 12 crates** from a single change in foundational crate.

---

## Example Scenarios

### Scenario 1: Bug Fix in Leaf Crate

**Change:** Fix bug in `feagi-sensorimotor`

**Result:**
- `feagi-sensorimotor`: `0.0.1-beta.3` → `0.0.1-beta.4`
- `feagi` (root): `0.0.1-beta.2` → `0.0.1-beta.3` (references sensorimotor)
- **All other crates:** Unchanged

**Published:** 2 crates  
**Time:** ~1 minute

---

### Scenario 2: Change in Mid-Layer Crate

**Change:** Add feature to `feagi-io`

**Result:**
- `feagi-io`: `0.0.1-beta.5` → `0.0.1-beta.6`
- `feagi-api`: `0.0.1-beta.3` → `0.0.1-beta.4` (depends on io)
- `feagi-agent`: `0.0.1-beta.2` → `0.0.1-beta.3` (depends on io)
- `feagi`: `0.0.1-beta.2` → `0.0.1-beta.3`
- **All other crates:** Unchanged

**Published:** 4 crates  
**Time:** ~2 minutes

---

### Scenario 3: Breaking Change in Foundation

**Change:** Refactor `feagi-npu-neural`

**Result:**
- 12+ crates bump (see dependency graph above)
- **Published:** ~12 crates
- **Time:** ~6 minutes

---

## Manual Usage

### Local Testing

```bash
cd /path/to/feagi-core

# 1. Detect changes and compute versions
./scripts/smart-version-bump.sh

# Review the output, ensure it makes sense

# 2. Apply version bumps (dry run first)
export VERSIONS_FILE=/tmp/versions-XXXXX  # Use path from step 1
DRY_RUN=true ./scripts/apply-version-bumps.sh

# 3. Apply for real
./scripts/apply-version-bumps.sh

# 4. Verify Cargo.toml files
git diff

# 5. Test build
cargo build --workspace --lib

# 6. Publish (dry run first)
export CHANGED_CRATES="feagi-io feagi-api feagi feagi-agent"
DRY_RUN=true ./scripts/publish-crates-smart.sh

# 7. Publish for real
export CARGO_REGISTRY_TOKEN="your-token"
./scripts/publish-crates-smart.sh
```

---

## Troubleshooting

### Issue: Script says no crates changed, but I know X changed

**Cause:** No git tag baseline, or changes not committed

**Solution:**
```bash
# Check if tags exist
git tag

# Set explicit baseline tag
export LAST_TAG=staging-20251215-120000
./scripts/smart-version-bump.sh

# Ensure changes are committed
git status
git add .
git commit -m "feat: my changes"
```

---

### Issue: Crate version already published error

**Cause:** Version number already exists on crates.io (immutable)

**Solution:**
This should not happen with the smart system, as it queries crates.io for the highest version. If it does:
- Manually increment version in that crate's `Cargo.toml`
- Re-run the workflow

---

### Issue: Dependent crate not getting bumped

**Cause:** Dependency graph not updated in scripts

**Solution:**
1. Check `DEPENDENCIES` array in `smart-version-bump.sh`
2. Ensure the dependency relationship is listed
3. Update if missing

---

## Comparison: Old vs New System

| Aspect | Old System (Unified) | New System (Independent) |
|--------|---------------------|--------------------------|
| **Version Strategy** | All crates same version | Each crate independent |
| **Version Format** | `0.0.1-beta.3` (workspace-wide) | `0.0.1-beta.X` (per crate) |
| **Publishes per Release** | 19 crates every time | Only changed crates (2-12) |
| **Release Time** | ~15 minutes | ~2-6 minutes |
| **Version Pollution** | ❌ High (unchanged crates bumped) | ✅ Low (only changed crates) |
| **Semantic Clarity** | ❌ Poor (version ≠ changes) | ✅ Clear (version = changes) |
| **Dependency Updates** | ❌ Force all users to update all | ✅ Users update selectively |
| **Audit Trail** | ❌ Unclear what changed | ✅ Clear from version numbers |
| **CI Complexity** | Simple (brute force) | Moderate (smart detection) |

---

## Migration Path to 1.0

When FEAGI-Core reaches stable 1.0, we'll transition versioning strategy:

### Current (Pre-1.0): Exact Beta Versions
```toml
feagi-npu-neural = { version = "=0.0.1-beta.5", path = "..." }
```

### After 1.0: Semver Compatible Ranges
```toml
feagi-npu-neural = { version = "^1.0.0", path = "..." }
```

**Rationale:**
- Pre-1.0: Breaking changes expected between betas → exact versions
- Post-1.0: Semver guarantees compatibility → flexible ranges

**Migration Steps:**
1. Tag final `1.0.0` release
2. Update all `workspace.dependencies` to use `^1.0.0` ranges
3. Update scripts to use semantic versioning rules (major/minor/patch)
4. Document breaking change policy

---

## References

- **Scripts:** `scripts/smart-version-bump.sh`, `scripts/apply-version-bumps.sh`, `scripts/publish-crates-smart.sh`
- **CI Workflow:** `.github/workflows/staging-merge.yml`
- **Publication Order:** `PUBLISHING_ORDER.md`
- **Cargo Workspaces Guide:** https://doc.rust-lang.org/cargo/reference/workspaces.html
- **Crates.io Publishing:** https://doc.rust-lang.org/cargo/reference/publishing.html

---

## Feedback & Improvements

This system is new as of December 2025. If you encounter issues or have suggestions:

1. Open an issue in the repository
2. Tag with `versioning`, `ci-cd`
3. Provide example scenario and expected vs actual behavior

Potential future improvements:
- [ ] Automatic changelog generation per crate
- [ ] Version bump preview in PR checks
- [ ] Graphical dependency impact visualization
- [ ] Support for patch/minor/major increments (post-1.0)

