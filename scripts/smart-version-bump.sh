#!/bin/bash
# Copyright 2025 Neuraville Inc.
# SPDX-License-Identifier: Apache-2.0

# Smart Independent Versioning System for feagi-core workspace
# 
# This script:
# 1. Detects which crates have changed since last release
# 2. Increments ONLY changed crates' versions (per-crate beta numbers)
# 3. Propagates version bumps to dependent crates
# 4. Updates workspace.dependencies with exact versions
# 5. Outputs a manifest of what will be published

set -e  # Exit on error

WORKSPACE_ROOT=$(pwd)
LAST_TAG="${LAST_TAG:-}"
DRY_RUN="${DRY_RUN:-false}"

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🔍 FEAGI Smart Independent Versioning System${NC}"
echo ""

# ============================================================================
# Define all crates in dependency order (same as publish-crates.sh)
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

# Dependency graph (key depends on values)
declare -A DEPENDENCIES=(
    ["feagi-observability"]=""
    ["feagi-structures"]="feagi-observability"
    ["feagi-config"]="feagi-observability"
    ["feagi-npu-neural"]="feagi-observability feagi-structures"
    ["feagi-npu-runtime"]="feagi-npu-neural"
    ["feagi-serialization"]="feagi-structures"
    ["feagi-state-manager"]="feagi-observability feagi-structures feagi-config"
    ["feagi-npu-burst-engine"]="feagi-npu-neural feagi-npu-runtime feagi-serialization feagi-structures feagi-state-manager"
    ["feagi-npu-plasticity"]="feagi-npu-neural"
    ["feagi-evolutionary"]="feagi-npu-neural feagi-structures feagi-observability"
    ["feagi-brain-development"]="feagi-npu-neural feagi-npu-burst-engine feagi-evolutionary feagi-structures feagi-observability"
    ["feagi-io"]="feagi-npu-burst-engine feagi-brain-development feagi-services feagi-npu-neural feagi-structures feagi-serialization"
    ["feagi-sensorimotor"]="feagi-structures feagi-serialization"
    ["feagi-services"]="feagi-state-manager feagi-npu-burst-engine feagi-brain-development feagi-evolutionary feagi-npu-neural feagi-observability"
    ["feagi-api"]="feagi-services feagi-io feagi-npu-neural feagi-evolutionary feagi-brain-development feagi-npu-burst-engine feagi-npu-runtime"
    ["feagi-agent"]="feagi-io feagi-structures feagi-serialization feagi-observability"
    ["feagi-hal"]="feagi-npu-runtime feagi-npu-neural feagi-observability feagi-structures"
    ["feagi"]="feagi-observability feagi-structures feagi-config feagi-npu-neural feagi-npu-runtime feagi-serialization feagi-state-manager feagi-npu-burst-engine feagi-npu-plasticity feagi-evolutionary feagi-brain-development feagi-io feagi-sensorimotor feagi-services feagi-api feagi-agent feagi-hal"
)

# Publication order (same as publish-crates.sh)
CRATE_ORDER=(
    "feagi-observability"
    "feagi-structures"
    "feagi-config"
    "feagi-npu-neural"
    "feagi-npu-runtime"
    "feagi-serialization"
    "feagi-state-manager"
    "feagi-npu-burst-engine"
    "feagi-npu-plasticity"
    "feagi-evolutionary"
    "feagi-brain-development"
    "feagi-io"
    "feagi-sensorimotor"
    "feagi-services"
    "feagi-api"
    "feagi-agent"
    "feagi-hal"
    "feagi"
)

# ============================================================================
# Helper Functions
# ============================================================================

get_crate_version() {
    local crate_name=$1
    local crate_path="${CRATE_PATHS[$crate_name]}"
    
    if [ "$crate_path" = "." ]; then
        grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
    else
        # Check if crate uses workspace version
        if grep -q '^version\.workspace = true' "$crate_path/Cargo.toml" 2>/dev/null; then
            # Get from workspace
            grep '^\[workspace\.package\]' -A 10 Cargo.toml | grep '^version = ' | head -1 | sed 's/version = "\(.*\)"/\1/'
        else
            # Get from crate's Cargo.toml
            grep '^version = ' "$crate_path/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/'
        fi
    fi
}

get_published_version() {
    local crate_name=$1
    # Query crates.io for latest published version
    local result=$(cargo search "$crate_name" --limit 1 2>/dev/null | grep "^$crate_name = " | sed 's/.*"\(.*\)".*/\1/' || echo "none")
    echo "$result"
}

increment_beta_version() {
    local current_version=$1
    local crate_name=$2
    
    # Parse version: X.Y.Z or X.Y.Z-beta.N
    if [[ $current_version =~ ^([0-9]+\.[0-9]+\.[0-9]+)(-beta\.([0-9]+))?$ ]]; then
        local base_version="${BASH_REMATCH[1]}"
        local beta_number="${BASH_REMATCH[3]:-0}"
        
        # Query crates.io for highest beta for this base version
        local highest_beta=0
        local search_results=$(cargo search "$crate_name" --limit 100 2>/dev/null || echo "")
        
        if [ -n "$search_results" ]; then
            # Extract all beta versions for this base version
            while IFS= read -r line; do
                if [[ $line =~ ^$crate_name\ =\ \"$base_version-beta\.([0-9]+)\" ]]; then
                    local found_beta="${BASH_REMATCH[1]}"
                    if [ "$found_beta" -gt "$highest_beta" ]; then
                        highest_beta=$found_beta
                    fi
                fi
            done <<< "$search_results"
        fi
        
        # Increment
        local new_beta=$((highest_beta + 1))
        echo "${base_version}-beta.${new_beta}"
    else
        echo -e "${RED}ERROR: Invalid version format: $current_version${NC}" >&2
        exit 1
    fi
}

has_crate_changed() {
    local crate_name=$1
    local crate_path="${CRATE_PATHS[$crate_name]}"
    
    # If no LAST_TAG specified, check against last git tag
    if [ -z "$LAST_TAG" ]; then
        LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    fi
    
    # If still no tag, assume everything changed (first release)
    if [ -z "$LAST_TAG" ]; then
        echo "true"
        return
    fi
    
    # Check if any files in crate directory changed since last tag
    local changed_files=$(git diff --name-only "$LAST_TAG" HEAD -- "$crate_path/" 2>/dev/null || echo "")
    
    if [ -n "$changed_files" ]; then
        echo "true"
    else
        echo "false"
    fi
}

get_dependent_crates() {
    local crate_name=$1
    local dependents=()
    
    # Find all crates that depend on this crate
    for other_crate in "${CRATE_ORDER[@]}"; do
        if [ "$other_crate" = "$crate_name" ]; then
            continue
        fi
        
        local deps="${DEPENDENCIES[$other_crate]}"
        if [[ " $deps " == *" $crate_name "* ]]; then
            dependents+=("$other_crate")
        fi
    done
    
    echo "${dependents[@]}"
}

# ============================================================================
# Main Logic
# ============================================================================

echo -e "${BLUE}📋 Step 1: Analyzing crate changes...${NC}"
echo ""

declare -A CHANGED_CRATES
declare -A CURRENT_VERSIONS
declare -A NEW_VERSIONS
declare -A CHANGE_REASONS

# First pass: Detect direct changes
for crate_name in "${CRATE_ORDER[@]}"; do
    current_version=$(get_crate_version "$crate_name")
    CURRENT_VERSIONS["$crate_name"]="$current_version"
    
    if [ "$(has_crate_changed "$crate_name")" = "true" ]; then
        CHANGED_CRATES["$crate_name"]="direct"
        CHANGE_REASONS["$crate_name"]="Direct code changes detected"
        echo -e "  ${YELLOW}📝${NC} $crate_name: Changed (current: $current_version)"
    fi
done

echo ""
echo -e "${BLUE}📋 Step 2: Propagating changes to dependent crates...${NC}"
echo ""

# Second pass: Propagate changes to dependents
changed_count=1
iteration=0
while [ $changed_count -gt 0 ]; do
    changed_count=0
    iteration=$((iteration + 1))
    
    for crate_name in "${CRATE_ORDER[@]}"; do
        # Skip if already marked as changed
        if [ -n "${CHANGED_CRATES[$crate_name]}" ]; then
            continue
        fi
        
        # Check if any dependencies changed
        local deps="${DEPENDENCIES[$crate_name]}"
        for dep in $deps; do
            if [ -n "${CHANGED_CRATES[$dep]}" ]; then
                CHANGED_CRATES["$crate_name"]="propagated"
                CHANGE_REASONS["$crate_name"]="Dependency '$dep' changed"
                echo -e "  ${CYAN}🔗${NC} $crate_name: Needs update (dependency: $dep)"
                changed_count=$((changed_count + 1))
                break
            fi
        done
    done
    
    if [ $iteration -gt 20 ]; then
        echo -e "${RED}ERROR: Circular dependency detected!${NC}"
        exit 1
    fi
done

# If nothing changed, we're done
if [ ${#CHANGED_CRATES[@]} -eq 0 ]; then
    echo -e "${GREEN}✅ No crates have changed. Nothing to version bump!${NC}"
    exit 0
fi

echo ""
echo -e "${BLUE}📋 Step 3: Computing new version numbers...${NC}"
echo ""

# Third pass: Compute new versions for changed crates
for crate_name in "${CRATE_ORDER[@]}"; do
    if [ -n "${CHANGED_CRATES[$crate_name]}" ]; then
        current_version="${CURRENT_VERSIONS[$crate_name]}"
        new_version=$(increment_beta_version "$current_version" "$crate_name")
        NEW_VERSIONS["$crate_name"]="$new_version"
        
        echo -e "  ${GREEN}📦${NC} $crate_name: $current_version → $new_version"
    fi
done

echo ""
echo -e "${BLUE}📋 Step 4: Generating version bump manifest...${NC}"
echo ""

# Generate summary
cat << EOF
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 VERSION BUMP SUMMARY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

EOF

echo -e "${GREEN}Crates to be bumped: ${#CHANGED_CRATES[@]}${NC}"
echo -e "${BLUE}Crates unchanged: $((${#CRATE_ORDER[@]} - ${#CHANGED_CRATES[@]}))${NC}"
echo ""

echo "Changed Crates:"
echo "───────────────"
for crate_name in "${CRATE_ORDER[@]}"; do
    if [ -n "${CHANGED_CRATES[$crate_name]}" ]; then
        current="${CURRENT_VERSIONS[$crate_name]}"
        new="${NEW_VERSIONS[$crate_name]}"
        reason="${CHANGE_REASONS[$crate_name]}"
        change_type="${CHANGED_CRATES[$crate_name]}"
        
        if [ "$change_type" = "direct" ]; then
            icon="📝"
        else
            icon="🔗"
        fi
        
        echo -e "  $icon $crate_name"
        echo -e "     Old: $current"
        echo -e "     New: ${GREEN}$new${NC}"
        echo -e "     Reason: $reason"
        echo ""
    fi
done

echo "Unchanged Crates:"
echo "─────────────────"
for crate_name in "${CRATE_ORDER[@]}"; do
    if [ -z "${CHANGED_CRATES[$crate_name]}" ]; then
        current="${CURRENT_VERSIONS[$crate_name]}"
        echo -e "  ✓ $crate_name (${current})"
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ============================================================================
# Export results for use by other scripts
# ============================================================================

# Export as environment variables that can be sourced
if [ "$DRY_RUN" != "true" ]; then
    # Create a temporary file with version updates
    VERSION_FILE=$(mktemp)
    echo "# Generated by smart-version-bump.sh" > "$VERSION_FILE"
    echo "# Source this file to get version information" >> "$VERSION_FILE"
    echo "" >> "$VERSION_FILE"
    
    for crate_name in "${CRATE_ORDER[@]}"; do
        if [ -n "${CHANGED_CRATES[$crate_name]}" ]; then
            safe_name=$(echo "$crate_name" | tr '-' '_' | tr '[:lower:]' '[:upper:]')
            echo "export NEW_VERSION_${safe_name}=\"${NEW_VERSIONS[$crate_name]}\"" >> "$VERSION_FILE"
        fi
    done
    
    echo ""
    echo -e "${CYAN}Version data exported to: $VERSION_FILE${NC}"
    echo "VERSIONS_FILE=$VERSION_FILE"
fi

# Export list of changed crates (for selective publishing)
echo ""
echo "CHANGED_CRATES=(${!CHANGED_CRATES[@]})"

exit 0

