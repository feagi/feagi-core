#!/bin/bash
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

