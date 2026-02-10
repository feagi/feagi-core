# FEAGI-Core Versioning Policy

**Status:** ACTIVE  
**Last Updated:** January 2025  
**Policy Type:** INDEPENDENT VERSIONING

---

## ⚠️ CRITICAL RULE: INDEPENDENT VERSIONING ONLY

**Each crate in the feagi-core workspace maintains its OWN version number.**

### ✅ CORRECT Implementation

Each crate's `Cargo.toml` MUST have an explicit version:

```toml
[package]
name = "feagi-npu-neural"
version = "0.0.1-beta.5"  # ← Explicit version number
edition.workspace = true
authors.workspace = true
```

### ❌ FORBIDDEN Implementation

**NEVER use workspace version inheritance:**

```toml
[package]
name = "feagi-npu-neural"
version.workspace = true  # ← FORBIDDEN! This creates synchronized versioning
```

---

## Why Independent Versioning?

1. **Granular Control**: Only bump versions for crates that actually changed
2. **Smaller Updates**: Users only download updated crates, not entire workspace
3. **Clear History**: Version numbers reflect actual changes to each crate
4. **Flexibility**: Different crates can evolve at different rates

---

## Version Bump Guidelines

### When to Bump Each Crate:

| Change Type | Action | Example |
|-------------|--------|---------|
| Bug fix in crate | Bump patch/beta | `0.0.1-beta.4` → `0.0.1-beta.5` |
| New feature in crate | Bump minor or beta | `0.0.1-beta.4` → `0.1.0-beta.1` |
| Breaking API change | Bump minor (pre-1.0) or major (post-1.0) | `0.1.0` → `0.2.0` |
| Documentation only | Optional: Bump patch | `0.0.1-beta.4` → `0.0.1-beta.5` |
| Dependency update | If it changes public API, bump | Check case-by-case |

### Version Cascading

If crate A changes and crate B depends on A:
- **Always bump A's version**
- **Only bump B if**:
  - B's public API changes
  - B's functionality changes
  - B needs to specify new minimum A version

---

## Publishing Workflow

### 1. Identify Changed Crates
```bash
# Show changed files since last release
git diff v0.0.1-beta.1..HEAD --name-only

# Identify affected crates
find crates -name "Cargo.toml" -exec dirname {} \;
```

### 2. Bump Versions Manually
Edit each changed crate's `Cargo.toml`:
```bash
vim crates/feagi-npu/neural/Cargo.toml
# Change: version = "0.0.1-beta.1"
# To:     version = "0.0.1-beta.4"
```

### 3. Update Dependencies
If crate B depends on changed crate A, update B's dependency:
```toml
# In crates/feagi-npu/burst-engine/Cargo.toml
[dependencies]
feagi-npu-neural = "0.0.1-beta.4"  # Update from beta.1
```

### 4. Publish in Dependency Order
```bash
# Follow PUBLISHING_ORDER.md
cargo publish -p feagi-npu-neural
sleep 30  # Wait for crates.io indexing
cargo publish -p feagi-npu-burst-engine
# ... continue in order
```

---

## Current Version Snapshot

As of last update, all crates are at:
```
feagi-agent:            0.0.1-beta.1
feagi-api:              0.0.1-beta.1
feagi-brain-development: 0.0.1-beta.1
feagi-config:           0.0.1-beta.1
feagi-evolutionary:     0.0.1-beta.1
feagi-hal:              0.0.1-beta.1
feagi-io:               0.0.1-beta.1
feagi-npu-burst-engine: 0.0.1-beta.1
feagi-npu-neural:       0.0.1-beta.1
feagi-npu-plasticity:   0.0.1-beta.1
feagi-npu-runtime:      0.0.1-beta.1
feagi-observability:    0.0.1-beta.1 (published)
feagi-sensorimotor:     0.0.1-beta.1
feagi-serialization:    0.0.1-beta.1
feagi-services:         0.0.1-beta.1
feagi-state-manager:    0.0.1-beta.1 (published)
feagi-structures:       0.0.1-beta.1
```

---

## Enforcement

### Pre-Commit Checks
Before committing:
```bash
# Verify no workspace version inheritance
grep -r "version.workspace = true" crates/*/Cargo.toml
# Should return nothing

# Verify all crates have explicit versions
find crates -name "Cargo.toml" -exec grep "^version = " {} \;
# Should show version for every crate
```

### Code Review Checklist
- [ ] No `version.workspace = true` in any crate
- [ ] Version bump justified in commit message
- [ ] Dependent crates updated if needed
- [ ] PUBLISHING_ORDER.md followed

---

## Migration from Synchronized Versioning

If accidentally using workspace versioning:

1. **Remove workspace version inheritance:**
   ```bash
   # For each crate
   sed -i 's/version.workspace = true/version = "0.0.1-beta.1"/' crates/*/Cargo.toml
   ```

2. **Verify:**
   ```bash
   cargo check --workspace
   ```

3. **Commit:**
   ```bash
   git add crates/*/Cargo.toml
   git commit -m "Fix: Switch to independent versioning per VERSIONING_POLICY.md"
   ```

---

## Questions?

**Q: Can I use workspace inheritance for other fields?**  
**A:** YES! Use `.workspace = true` for: `edition`, `authors`, `license`, `repository`, `homepage`. Just NOT for `version`.

**Q: What if I forget and use `version.workspace = true`?**  
**A:** Remove it immediately and set explicit version. This violates our policy.

**Q: How do I know which crates to publish?**  
**A:** Follow `PUBLISHING_ORDER.md` and only publish crates with version changes.

---

**This policy is MANDATORY for all FEAGI-core development.**

