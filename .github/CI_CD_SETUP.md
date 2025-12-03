# CI/CD Setup for feagi-core

**Date**: December 3, 2025  
**Project**: feagi-core (single-crate publication)

## Overview

GitHub Actions workflows configured for automated testing and publishing to crates.io.

## Workflows

### 1. **staging-pr.yml** - Staging PR Tests

**Trigger**: When PR is opened to `staging` branch

**Purpose**: Test beta versions before merge

**Checks**:
- Version format validation (must be `X.Y.Z-beta.N`)
- Version must be higher than current staging
- Version must be higher than main
- Beta number must increment properly
- All tests pass
- Code formatting
- Clippy linting

**Example**: `2.0.1-beta.1` → `2.0.1-beta.2`

### 2. **staging-merge.yml** - Staging Merge (Prerelease)

**Trigger**: When PR merges to `staging` branch

**Purpose**: Publish beta version to crates.io

**Actions**:
- Build release version
- Run tests
- Publish to crates.io (beta version)
- Create Git tag (e.g., `v2.0.1-beta.1`)
- Create GitHub Prerelease
- Update release notes

**Result**: Beta version available on crates.io for testing

### 3. **main-pr.yml** - Main PR Tests (Comprehensive)

**Trigger**: When PR is opened to `main` branch

**Purpose**: Strict validation before production release

**Checks**:
- Branch name format: `Pre-Main: X.Y.Z`
- Branch name version matches Cargo.toml
- Version is semantic only (no `-beta`)
- Version is higher than current main
- All tests pass (debug + release modes)
- Code formatting
- Clippy linting (zero warnings)
- Documentation builds
- Dry-run publish to crates.io

**Example Branch**: `Pre-Main: 2.0.1`

### 4. **main-merge.yml** - Main Merge (Production Release)

**Trigger**: When PR merges to `main` branch

**Purpose**: Publish production version to crates.io

**Actions**:
- Build release version
- Publish to crates.io (production version)
- Create Git tag (e.g., `v2.0.1`)
- Create GitHub Release with changelog
- Merge main back into staging (keep staging updated)
- Notify success

**Result**: Production version available on crates.io

## Workflow Summary

```
Developer Branch (2.0.1-beta.1)
    ↓ PR to staging
staging-pr.yml (tests)
    ↓ merge
staging-merge.yml (publish beta)
    ↓
Beta available on crates.io
    ↓
Pre-Main Branch (2.0.1)
    ↓ PR to main
main-pr.yml (comprehensive tests)
    ↓ merge
main-merge.yml (publish production)
    ↓
Production available on crates.io
    ↓ auto-merge
main → staging (keep staging updated)
```

## Version Requirements

### Staging Branch

**Format**: `X.Y.Z-beta.N`

**Examples**:
- `2.0.1-beta.1` (first beta of 2.0.1)
- `2.0.1-beta.2` (second beta of 2.0.1)
- `2.1.0-beta.1` (first beta of 2.1.0)

**Rules**:
- Must be higher than main version
- Beta number increments for same semantic version
- New semantic version can start at beta.1

### Main Branch

**Format**: `X.Y.Z` (semantic only)

**Examples**:
- `2.0.0`
- `2.0.1`
- `2.1.0`

**Rules**:
- Must be higher than previous main version
- No beta or other tags
- Branch name must match: `Pre-Main: X.Y.Z`

## Required GitHub Secrets

Set these in GitHub repository settings:

### `CARGO_PUSH_TOKEN`

Your crates.io API token for publishing.

**Get it**:
1. Log in to https://crates.io
2. Go to Account Settings → API Tokens
3. Create new token
4. Copy token

**Set it**:
1. GitHub repo → Settings → Secrets and variables → Actions
2. New repository secret
3. Name: `CARGO_PUSH_TOKEN`
4. Value: (paste your token)

### `GITHUB_TOKEN`

Automatically provided by GitHub Actions (no setup needed)

## Branch Protection Rules

### Main Branch

**Recommended settings**:
- ✅ Require pull request reviews (1 reviewer)
- ✅ Require status checks to pass before merging
  - `comprehensive-tests` (from main-pr.yml)
- ✅ Require branches to be up to date before merging
- ✅ Do not allow bypassing the above settings

### Staging Branch

**Recommended settings**:
- ✅ Require pull request reviews (1 reviewer)
- ✅ Require status checks to pass before merging
  - `test-and-version-check` (from staging-pr.yml)
- ✅ Allow force pushes (for rebasing)

## Usage Examples

### Release Beta Version

1. Create branch from staging: `git checkout staging && git checkout -b feature/my-feature`
2. Make changes
3. Update version in `Cargo.toml`: `2.0.1-beta.1` → `2.0.1-beta.2`
4. Commit and push
5. Open PR to `staging`
6. Workflow runs → tests → merge
7. Automatic publish to crates.io (beta)

### Release Production Version

1. Create branch from staging: `git checkout staging && git checkout -b Pre-Main: 2.0.1`
2. Update version in `Cargo.toml`: `2.0.1-beta.2` → `2.0.1`
3. Commit and push
4. Open PR to `main`
5. Workflow runs → comprehensive tests → merge
6. Automatic publish to crates.io (production)
7. Main auto-merges back to staging

## Manual Publishing (Fallback)

If workflows fail, publish manually:

```bash
cd /Users/nadji/code/FEAGI-2.0/feagi-core

# Test first
cargo publish --dry-run

# Publish
cargo publish
```

## Verification After Release

### Check crates.io

```bash
# Visit
https://crates.io/crates/feagi-core

# Or install
cargo install feagi-core --version 2.0.1
```

### Check docs.rs

```bash
# Visit
https://docs.rs/feagi-core/2.0.1
```

### Check GitHub Release

```bash
# Visit
https://github.com/feagi/feagi-core/releases/tag/v2.0.1
```

## Troubleshooting

### Workflow Fails: "CARGO_PUSH_TOKEN not set"

**Solution**: Add `CARGO_PUSH_TOKEN` secret in GitHub repo settings

### Workflow Fails: "Version already published"

**Solution**: Increment version number in Cargo.toml

### Workflow Fails: "Branch name format invalid"

**Solution**: For main PRs, branch name must be `Pre-Main: X.Y.Z`

### Workflow Fails: "Version check failed"

**Solution**: Ensure version is higher than current branch version

## Differences from feagi-data-processing

### What Changed

1. **Single Crate Publication**
   - Old: Published 3 separate crates in order
   - New: Publishes one `feagi-core` crate

2. **Simpler Publishing**
   - Old: Loop through crates array, publish each
   - New: Single `cargo publish` command

3. **Release Notes**
   - Updated crate name references
   - Updated installation examples
   - Added quick start example

4. **Repository URLs**
   - Updated to `github.com/feagi/feagi-core`
   - Updated crates.io links to `feagi-core`

### What Stayed the Same

- Branch strategy (staging → main)
- Version validation logic
- Test requirements
- Branch naming conventions
- Secret requirements

## Status

✅ **COMPLETE - Ready for Use**

All 4 workflows created and configured for feagi-core single-crate publication model.

---

**Created by**: AI Development Assistant  
**Date**: December 3, 2025  
**Status**: Production Ready

