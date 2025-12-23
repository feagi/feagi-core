#!/bin/bash
# Copyright 2025 Neuraville Inc.
# SPDX-License-Identifier: Apache-2.0

# Smart Independent Publishing for feagi-core workspace
# 
# This script publishes ONLY crates that have version updates
# Skips unchanged crates to save time and avoid unnecessary publishes

set -e

CARGO_TOKEN="${CARGO_REGISTRY_TOKEN:-}"
DRY_RUN="${DRY_RUN:-false}"
DELAY_SECONDS=30
CHANGED_CRATES_LIST="${CHANGED_CRATES[@]}"

# ANSI colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

if [ -z "$CARGO_TOKEN" ] && [ "$DRY_RUN" != "true" ]; then
    echo -e "${RED}❌ Error: CARGO_REGISTRY_TOKEN environment variable must be set${NC}"
    exit 1
fi

echo -e "${CYAN}🚀 Publishing feagi-core workspace crates to crates.io${NC}"
echo -e "${BLUE}📦 Dry run: $DRY_RUN${NC}"
echo ""

# ============================================================================
# Define crate paths and publication order
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

# Publication order (dependencies first)
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
# Check if crate should be published
# ============================================================================

should_publish_crate() {
    local crate_name=$1
    
    # If CHANGED_CRATES_LIST is empty, publish all (fallback to old behavior)
    if [ -z "$CHANGED_CRATES_LIST" ]; then
        echo "true"
        return
    fi
    
    # Check if crate is in changed list
    if [[ " ${CHANGED_CRATES_LIST} " == *" ${crate_name} "* ]]; then
        echo "true"
    else
        echo "false"
    fi
}

# ============================================================================
# Publish function
# ============================================================================

publish_crate() {
    local crate_name=$1
    local crate_path="${CRATE_PATHS[$crate_name]}"
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${BLUE}📦 Publishing: $crate_name${NC}"
    echo "   Path: $crate_path"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Change to crate directory
    if [ "$crate_path" != "." ]; then
        cd "$crate_path"
    fi
    
    # Get version
    local version=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    echo "   Version: $version"
    
    # Check if already published
    if cargo search "$crate_name" --limit 1 2>/dev/null | grep -q "^$crate_name = \"$version\""; then
        echo -e "   ${YELLOW}⏭️  Skipping $crate_name v$version (already published)${NC}"
        cd "$WORKSPACE_ROOT" 2>/dev/null || cd - > /dev/null
        return 0
    fi
    
    # Package first to verify
    echo "   📦 Packaging..."
    if ! cargo package --quiet 2>&1; then
        echo -e "   ${RED}❌ Failed to package $crate_name${NC}"
        cd "$WORKSPACE_ROOT" 2>/dev/null || cd - > /dev/null
        return 1
    fi
    
    # Publish
    if [ "$DRY_RUN" = "true" ]; then
        echo -e "   ${CYAN}🧪 Dry run: cargo publish --dry-run${NC}"
        cargo publish --dry-run
    else
        echo "   🚀 Publishing to crates.io..."
        if cargo publish --token "$CARGO_TOKEN"; then
            echo -e "   ${GREEN}✅ Successfully published $crate_name v$version${NC}"
            
            # Delay for crates.io indexing (except for last crate)
            if [ "$crate_name" != "feagi" ]; then
                echo -e "   ${CYAN}⏳ Waiting ${DELAY_SECONDS}s for crates.io indexing...${NC}"
                sleep $DELAY_SECONDS
            fi
        else
            echo -e "   ${RED}❌ Failed to publish $crate_name${NC}"
            cd "$WORKSPACE_ROOT" 2>/dev/null || cd - > /dev/null
            return 1
        fi
    fi
    
    # Return to workspace root
    cd "$WORKSPACE_ROOT" 2>/dev/null || cd - > /dev/null
    
    return 0
}

# ============================================================================
# Main execution
# ============================================================================

WORKSPACE_ROOT=$(pwd)

echo -e "${BLUE}Starting publication process...${NC}"
echo ""

if [ -n "$CHANGED_CRATES_LIST" ]; then
    echo -e "${CYAN}📋 Smart publishing mode: Only publishing changed crates${NC}"
    echo -e "${GREEN}Changed crates: $CHANGED_CRATES_LIST${NC}"
    echo ""
else
    echo -e "${YELLOW}⚠️  No changed crates list provided - publishing ALL crates${NC}"
    echo ""
fi

FAILED_CRATES=()
PUBLISHED_COUNT=0
SKIPPED_COUNT=0

for crate_name in "${CRATE_ORDER[@]}"; do
    crate_path="${CRATE_PATHS[$crate_name]}"
    
    if [ ! -f "$crate_path/Cargo.toml" ] && [ "$crate_path" != "." ]; then
        echo -e "${YELLOW}⚠️  Warning: $crate_path not found, skipping...${NC}"
        continue
    fi
    
    # Check if crate should be published
    if [ "$(should_publish_crate "$crate_name")" = "false" ]; then
        echo -e "${BLUE}⏭️  Skipping $crate_name (unchanged)${NC}"
        SKIPPED_COUNT=$((SKIPPED_COUNT + 1))
        continue
    fi
    
    if publish_crate "$crate_name"; then
        PUBLISHED_COUNT=$((PUBLISHED_COUNT + 1))
    else
        FAILED_CRATES+=("$crate_name")
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${CYAN}📊 Publication Summary${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ Successfully published: $PUBLISHED_COUNT crates${NC}"
echo -e "${BLUE}⏭️  Skipped (unchanged): $SKIPPED_COUNT crates${NC}"

if [ ${#FAILED_CRATES[@]} -gt 0 ]; then
    echo -e "${RED}❌ Failed to publish: ${#FAILED_CRATES[@]} crates${NC}"
    echo ""
    echo "Failed crates:"
    for crate in "${FAILED_CRATES[@]}"; do
        echo "  - $crate"
    done
    echo ""
    exit 1
else
    echo ""
    echo -e "${GREEN}🎉 All crates published successfully!${NC}"
    exit 0
fi

