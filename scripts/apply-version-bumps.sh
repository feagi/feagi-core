#!/usr/bin/env bash
# Copyright 2025 Neuraville Inc.
# SPDX-License-Identifier: Apache-2.0

# Apply Version Bumps to Cargo.toml files
#
# This script reads the output from smart-version-bump.sh and applies
# version updates to:
# 1. Individual crate Cargo.toml files
# 2. workspace.dependencies in root Cargo.toml
# 3. Workspace version if root crate changed

set -e

WORKSPACE_ROOT=$(pwd)
DRY_RUN="${DRY_RUN:-false}"

# ANSI colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}🔧 Applying Version Bumps${NC}"
echo ""

# ============================================================================
# Check for version data file
# ============================================================================

if [ -z "$VERSIONS_FILE" ]; then
    echo -e "${RED}ERROR: VERSIONS_FILE not set${NC}"
    echo "Run smart-version-bump.sh first and source its output"
    exit 1
fi

if [ ! -f "$VERSIONS_FILE" ]; then
    echo -e "${RED}ERROR: Version file not found: $VERSIONS_FILE${NC}"
    exit 1
fi

# Source the versions
source "$VERSIONS_FILE"

# ============================================================================
# Define crate paths (same as smart-version-bump.sh)
# ============================================================================

declare -A CRATE_PATHS=(
    ["feagi-observability"]="crates/feagi-observability"
    ["feagi-structures"]="crates/feagi-structures"
    ["feagi-config"]="crates/feagi-config"
    ["feagi-npu-neural"]="crates/feagi-npu/neural"
    ["feagi-npu-runtime"]="crates/feagi-npu/runtime"
    ["feagi-serialization"]="crates/feagi-serialization"
    ["feagi-state-manager"]="crates/feagi-state-manager"
    ["feagi-npu-burst-engine"]="crates/feagi-npu/burst-engine"
    ["feagi-npu-plasticity"]="crates/feagi-npu/plasticity"
    ["feagi-evolutionary"]="crates/feagi-evolutionary"
    ["feagi-brain-development"]="crates/feagi-brain-development"
    ["feagi-io"]="crates/feagi-io"
    ["feagi-sensorimotor"]="crates/feagi-sensorimotor"
    ["feagi-services"]="crates/feagi-services"
    ["feagi-api"]="crates/feagi-api"
    ["feagi-agent"]="crates/feagi-agent"
    ["feagi-hal"]="crates/feagi-hal"
    ["feagi"]="."
)

# ============================================================================
# Apply version updates
# ============================================================================

update_count=0

for crate_name in "${!CRATE_PATHS[@]}"; do
    safe_name=$(echo "$crate_name" | tr '-' '_' | tr '[:lower:]' '[:upper:]')
    var_name="NEW_VERSION_${safe_name}"
    new_version="${!var_name}"
    
    if [ -z "$new_version" ]; then
        # Crate unchanged, skip
        continue
    fi
    
    crate_path="${CRATE_PATHS[$crate_name]}"
    
    echo -e "${BLUE}📝 Updating $crate_name to $new_version${NC}"
    
    # Update crate's own Cargo.toml
    if [ "$crate_path" = "." ]; then
        cargo_toml="Cargo.toml"
    else
        cargo_toml="$crate_path/Cargo.toml"
    fi
    
    if [ ! -f "$cargo_toml" ]; then
        echo -e "${RED}ERROR: Cargo.toml not found: $cargo_toml${NC}"
        exit 1
    fi
    
    # Check if crate uses explicit version or workspace version
    if grep -q '^version\.workspace = true' "$cargo_toml" 2>/dev/null; then
        echo -e "  ${YELLOW}⚠${NC}  Uses workspace version - will update workspace.package"
        # Will be handled by workspace update below
    else
        # Update explicit version in crate
        if [ "$DRY_RUN" = "true" ]; then
            echo -e "  ${CYAN}[DRY RUN]${NC} Would update version in $cargo_toml"
        else
            sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" "$cargo_toml"
            rm -f "${cargo_toml}.bak"
            echo -e "  ${GREEN}✓${NC} Updated $cargo_toml"
        fi
    fi
    
    # Update workspace.dependencies reference in root Cargo.toml
    if [ "$crate_name" != "feagi" ]; then
        echo -e "  ${BLUE}→${NC} Updating workspace.dependencies reference"
        
        if [ "$DRY_RUN" = "true" ]; then
            echo -e "  ${CYAN}[DRY RUN]${NC} Would update $crate_name reference in Cargo.toml"
        else
            # Update version in workspace.dependencies section
            # Pattern: crate-name = { version = "...", path = "..." }
            sed -i.bak "s/\($crate_name = { version = \)\"[^\"]*\"/\1\"=$new_version\"/" Cargo.toml
            rm -f "Cargo.toml.bak"
            echo -e "  ${GREEN}✓${NC} Updated workspace.dependencies"
        fi
    fi
    
    update_count=$((update_count + 1))
    echo ""
done

# ============================================================================
# Synchronize workspace path dependency versions (prevents version mismatches)
# ============================================================================
#
# Why this exists:
# - In this workspace, many dependencies are declared as inline tables with BOTH `version = "..."`
#   and `path = "../some-crate"`.
# - During automated pre-release bumps, it is possible to end up with a transient mismatch where:
#     - a dependent crate requires `feagi-structures = "0.0.1-beta.X"`
#     - but the local path crate `feagi-structures` still has a different `[package].version`
#   which causes Cargo/Clippy to fail with:
#     "failed to select a version for the requirement ..."
#
# Deterministic rule:
# - For any inline-table dependency that has a `path = "../..."` pointing to a workspace crate
#   AND a `version = "..."`, we rewrite the version to match the *actual local crate version*
#   after bumps are applied.
#
echo -e "${BLUE}Synchronizing workspace path dependency versions...${NC}"
if [ "$DRY_RUN" = "true" ]; then
    echo -e "  ${CYAN}[DRY RUN]${NC} Would synchronize path dependency versions across manifests"
else
    python3 - <<'PY'
from __future__ import annotations

import re
from pathlib import Path


def read_workspace_version(root: Path) -> str:
    cargo = (root / "Cargo.toml").read_text(encoding="utf-8")
    m = re.search(r"(?ms)^\[workspace\.package\].*?^version\s*=\s*\"([^\"]+)\"", cargo)
    if not m:
        raise SystemExit("Failed to parse [workspace.package] version from Cargo.toml")
    return m.group(1)


def read_crate_name_and_version(manifest: Path, workspace_version: str) -> tuple[str | None, str | None]:
    text = manifest.read_text(encoding="utf-8")
    name_m = re.search(r"(?m)^name\s*=\s*\"([^\"]+)\"", text)
    name = name_m.group(1) if name_m else None

    if re.search(r"(?m)^version\.workspace\s*=\s*true\s*$", text):
        return name, workspace_version

    ver_m = re.search(r"(?m)^version\s*=\s*\"([^\"]+)\"", text)
    ver = ver_m.group(1) if ver_m else None
    return name, ver


def main() -> None:
    root = Path.cwd()
    workspace_version = read_workspace_version(root)

    # Collect all workspace crate versions by crate name.
    crate_versions: dict[str, str] = {}
    manifests: list[Path] = [root / "Cargo.toml"]
    manifests += sorted((root / "crates").glob("*/Cargo.toml"))
    manifests += sorted((root / "crates" / "feagi-npu").glob("*/Cargo.toml"))

    for mf in manifests:
        if not mf.exists():
            continue
        name, ver = read_crate_name_and_version(mf, workspace_version)
        if name and ver:
            crate_versions[name] = ver

    # Rewrite inline-table deps with path="../..." and version="...".
    dep_re = re.compile(
        r'^(?P<indent>\s*)(?P<dep>[A-Za-z0-9_-]+)\s*=\s*\{\s*(?P<body>[^}]*)\}\s*$'
    )
    version_re = re.compile(r'version\s*=\s*"(?P<ver>[^"]+)"')
    path_re = re.compile(r'path\s*=\s*"(?P<path>\.\./[^"]+)"')

    changed_files = 0
    changed_deps = 0

    for mf in manifests:
        text = mf.read_text(encoding="utf-8").splitlines(True)
        out: list[str] = []
        file_changed = False

        for line in text:
            m = dep_re.match(line.rstrip("\n"))
            if not m:
                out.append(line)
                continue

            dep = m.group("dep")
            body = m.group("body")
            if dep not in crate_versions:
                out.append(line)
                continue

            vm = version_re.search(body)
            pm = path_re.search(body)
            if not vm or not pm:
                out.append(line)
                continue

            current_req = vm.group("ver")
            actual = crate_versions[dep]

            # Normalize cargo req strings we might have written (e.g. "=0.0.1-beta.2")
            norm_req = current_req.lstrip("=").lstrip("^")
            if norm_req == actual:
                out.append(line)
                continue

            new_body = version_re.sub(f'version = "{actual}"', body, count=1)
            new_line = f'{m.group("indent")}{dep} = {{ {new_body.strip()} }}\n'
            out.append(new_line)
            file_changed = True
            changed_deps += 1

        if file_changed:
            mf.write_text("".join(out), encoding="utf-8")
            changed_files += 1

    print(f"Synchronized {changed_deps} dependency entries across {changed_files} file(s).")


if __name__ == "__main__":
    main()
PY
    echo -e "  ${GREEN}✓${NC} Synchronized workspace path dependency versions"
fi
echo ""

# ============================================================================
# Update workspace.package version if root crate changed
# ============================================================================

if [ -n "$NEW_VERSION_FEAGI" ]; then
    echo -e "${BLUE}📝 Updating workspace.package version to $NEW_VERSION_FEAGI${NC}"
    
    if [ "$DRY_RUN" = "true" ]; then
        echo -e "  ${CYAN}[DRY RUN]${NC} Would update [workspace.package] version"
    else
        # Update version in [workspace.package] section
        sed -i.bak '/^\[workspace\.package\]/,/^\[/ s/^version = ".*"/version = "'"$NEW_VERSION_FEAGI"'"/' Cargo.toml
        rm -f "Cargo.toml.bak"
        echo -e "  ${GREEN}✓${NC} Updated workspace.package version"
    fi
    echo ""
fi

# ============================================================================
# Summary
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ Version bump complete!${NC}"
echo ""
echo -e "Updated $update_count crate(s)"
echo ""

if [ "$DRY_RUN" = "true" ]; then
    echo -e "${YELLOW}NOTE: This was a dry run. No files were modified.${NC}"
    echo "Set DRY_RUN=false to apply changes."
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

exit 0

