# Independent Versioning Quick Reference

## One-Line Summary
**Each crate maintains its own version; only changed crates get bumped.**

---

## Key Concepts

| Concept | Description |
|---------|-------------|
| **Independent Versions** | Each crate has its own `X.Y.Z-beta.N` number |
| **Smart Detection** | Git diff detects which crates changed |
| **Automatic Propagation** | If A depends on B and B changes, A also bumps |
| **Exact Dependencies** | Use `=X.Y.Z-beta.N` for pre-1.0 stability |
| **Selective Publishing** | Only publish crates with new versions |

---

## Scripts

### `smart-version-bump.sh`
**Detects changes, computes new versions**
```bash
./scripts/smart-version-bump.sh
```

### `apply-version-bumps.sh`
**Applies versions to Cargo.toml files**
```bash
export VERSIONS_FILE=/tmp/versions-XXXXX
./scripts/apply-version-bumps.sh
```

### `publish-crates-smart.sh`
**Publishes only changed crates**
```bash
export CARGO_REGISTRY_TOKEN="token"
export CHANGED_CRATES="crate1 crate2 crate3"
./scripts/publish-crates-smart.sh
```

---

## CI/CD Workflow

**File:** `.github/workflows/staging-merge.yml`

**Trigger:** Merge to `staging`

**Process:**
1. Detect changes → `smart-version-bump.sh`
2. Apply versions → `apply-version-bumps.sh`
3. Publish → `publish-crates-smart.sh`
4. Commit version updates → staging branch
5. Create prerelease tag → `staging-YYYYMMDD-HHMMSS`

---

## Example Scenarios

### Scenario 1: Bug fix in `feagi-io`
```
Changed: feagi-io (0.0.1-beta.5 → 0.0.1-beta.6)
Propagated: feagi-api, feagi-agent, feagi (root)
Published: 4 crates
Time: ~2 min
```

### Scenario 2: Feature in `feagi-npu-neural`
```
Changed: feagi-npu-neural (0.0.1-beta.3 → 0.0.1-beta.4)
Propagated: 11+ dependent crates (see dependency graph)
Published: ~12 crates
Time: ~6 min
```

---

## Dependency Version Format

### Pre-1.0 (Current)
```toml
feagi-npu-neural = { version = "=0.0.1-beta.5", path = "..." }
```
**Exact version** - breaking changes expected between betas

### Post-1.0 (Future)
```toml
feagi-npu-neural = { version = "^1.0.0", path = "..." }
```
**Semver compatible** - patch/minor updates automatic

---

## Common Issues

### No crates detected as changed
- **Cause:** No git tags or uncommitted changes
- **Fix:** Commit changes, or set `LAST_TAG=xxx`

### Version already published
- **Cause:** Should not happen (scripts query crates.io)
- **Fix:** Manually increment in Cargo.toml

### Dependent crate not bumping
- **Cause:** Dependency graph not updated in script
- **Fix:** Update `DEPENDENCIES` array in `smart-version-bump.sh`

---

## Manual Testing

```bash
# 1. Detect and review
./scripts/smart-version-bump.sh

# 2. Dry run apply
export VERSIONS_FILE=/tmp/versions-XXXXX
DRY_RUN=true ./scripts/apply-version-bumps.sh

# 3. Apply for real
./scripts/apply-version-bumps.sh

# 4. Build to verify
cargo build --workspace --lib

# 5. Dry run publish
export CHANGED_CRATES="crate1 crate2"
DRY_RUN=true ./scripts/publish-crates-smart.sh

# 6. Publish for real
export CARGO_REGISTRY_TOKEN="token"
./scripts/publish-crates-smart.sh
```

---

## Full Documentation

See: `docs/INDEPENDENT_VERSIONING.md`


