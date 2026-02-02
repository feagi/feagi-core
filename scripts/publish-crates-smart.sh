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
# Crates.io enforces publish rate limits. Use a conservative default delay.
# You can override per-run with: DELAY_SECONDS=60 ./scripts/publish-crates-smart.sh
DELAY_SECONDS="${DELAY_SECONDS:-90}"

# ----------------------------------------------------------------------------
# Normalize CHANGED_CRATES input (supports both array and string formats)
#
# CI often passes:
#   CHANGED_CRATES="(feagi-a feagi-b ...)"
# While bash arrays are:
#   CHANGED_CRATES=(feagi-a feagi-b ...)
# ----------------------------------------------------------------------------
RAW_CHANGED_CRATES="${CHANGED_CRATES[*]}"
# Strip parentheses, convert commas to spaces, and normalize whitespace
CHANGED_CRATES_LIST="$(echo "${RAW_CHANGED_CRATES}" | tr -d '()' | tr ',' ' ' | xargs)"

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

# NOTE: This script must work on macOS default bash (3.2) and Ubuntu CI bash.
# Bash 3.2 does NOT support associative arrays (declare -A), so we use a
# portable mapping function instead.
crate_path_for() {
    case "$1" in
        feagi-observability) echo "crates/feagi-observability" ;;
        feagi-structures) echo "crates/feagi-structures" ;;
        feagi-config) echo "crates/feagi-config" ;;
        feagi-npu-neural) echo "crates/feagi-npu/neural" ;;
        feagi-npu-runtime) echo "crates/feagi-npu/runtime" ;;
        feagi-serialization) echo "crates/feagi-serialization" ;;
        feagi-state-manager) echo "crates/feagi-state-manager" ;;
        feagi-npu-burst-engine) echo "crates/feagi-npu/burst-engine" ;;
        feagi-npu-plasticity) echo "crates/feagi-npu/plasticity" ;;
        feagi-evolutionary) echo "crates/feagi-evolutionary" ;;
        feagi-brain-development) echo "crates/feagi-brain-development" ;;
        feagi-io) echo "crates/feagi-io" ;;
        feagi-sensorimotor) echo "crates/feagi-sensorimotor" ;;
        feagi-services) echo "crates/feagi-services" ;;
        feagi-api) echo "crates/feagi-api" ;;
        feagi-agent) echo "crates/feagi-agent" ;;
        feagi-hal) echo "crates/feagi-hal" ;;
        feagi) echo "." ;;
        *) return 1 ;;
    esac
}

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
    "feagi-sensorimotor"
    "feagi-services"
    "feagi-io"
    "feagi-agent"
    "feagi-api"
    "feagi-hal"
    "feagi"
)

# ============================================================================
# Check if crate is already published on crates.io
# ============================================================================

is_already_published() {
    local crate_name=$1
    local crate_path=$2
    
    # Get version from Cargo.toml
    cd "$crate_path" 2>/dev/null || return 1
    local version=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    cd "$WORKSPACE_ROOT" 2>/dev/null || cd - > /dev/null
    
    # Check if this exact version exists on crates.io
    if cargo search "$crate_name" --limit 1 2>/dev/null | grep -q "^$crate_name = \"$version\""; then
        echo "true"
    else
        echo "false"
    fi
}

# ============================================================================
# Check if crate should be published
# ============================================================================

should_publish_crate() {
    local crate_name=$1
    local crate_path=$2
    
    # CRITICAL: Check crates.io FIRST - if already published, always skip
    if [ "$(is_already_published "$crate_name" "$crate_path")" = "true" ]; then
        echo "skip_published"
        return
    fi

    # If the crate is NOT published yet, we MUST publish it even if it's not in
    # the changed list. Otherwise, dependents will fail with "no matching package".
    if [ -n "$CHANGED_CRATES_LIST" ]; then
        if [[ " ${CHANGED_CRATES_LIST} " != *" ${crate_name} "* ]]; then
            echo "publish_unpublished"
            return
        fi
    fi
    
    # If CHANGED_CRATES_LIST is empty, publish all unpublished crates
    if [ -z "$CHANGED_CRATES_LIST" ]; then
        echo "publish"
        return
    fi
    
    # Check if crate is in changed list
    if [[ " ${CHANGED_CRATES_LIST} " == *" ${crate_name} "* ]]; then
        echo "publish"
    else
        echo "skip_unchanged"
    fi
}

# ============================================================================
# Publish function
# ============================================================================

publish_crate() {
    local crate_name=$1
    local crate_path
    crate_path="$(crate_path_for "$crate_name")" || return 1
    
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
    
    # Package first to verify
    echo "   📦 Packaging..."
    # CI modifies Cargo.toml versions right before publishing, which makes the
    # working directory "dirty" by design. Packaging must allow this.
    if ! cargo package --allow-dirty --quiet 2>&1; then
        echo -e "   ${RED}❌ Failed to package $crate_name${NC}"
        cd "$WORKSPACE_ROOT" 2>/dev/null || cd - > /dev/null
        return 1
    fi
    
    # Publish
    if [ "$DRY_RUN" = "true" ]; then
        echo -e "   ${CYAN}🧪 Dry run: cargo publish --dry-run${NC}"
        cargo publish --dry-run --allow-dirty
    else
        echo "   🚀 Publishing to crates.io..."
        # With set -e, we must temporarily disable exit-on-error to capture output.
        set +e
        publish_output="$(cargo publish --allow-dirty --token "$CARGO_TOKEN" 2>&1)"
        publish_status=$?
        set -e

        if [ "$publish_status" -eq 0 ]; then
            echo -e "   ${GREEN}✅ Successfully published $crate_name v$version${NC}"

            # Delay for crates.io indexing (except for last crate)
            if [ "$crate_name" != "feagi" ]; then
                echo -e "   ${CYAN}⏳ Waiting ${DELAY_SECONDS}s for crates.io indexing...${NC}"
                sleep $DELAY_SECONDS
            fi
        else
            # If already published, treat as a skip (do NOT fail the run)
            if echo "$publish_output" | grep -qiE "already exists on crates\.io|already exists on crates\.io index|version .* already exists"; then
                echo -e "   ${YELLOW}⏭️  Skipping $crate_name v$version (already published on crates.io)${NC}"
                cd "$WORKSPACE_ROOT" 2>/dev/null || cd - > /dev/null
                return 0
            fi

            echo "$publish_output"
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
    crate_path="$(crate_path_for "$crate_name")" || continue
    
    if [ ! -f "$crate_path/Cargo.toml" ] && [ "$crate_path" != "." ]; then
        echo -e "${YELLOW}⚠️  Warning: $crate_path not found, skipping...${NC}"
        continue
    fi
    
    # Check if crate should be published
    publish_decision=$(should_publish_crate "$crate_name" "$crate_path")
    
    if [ "$publish_decision" = "skip_published" ]; then
        # Get version for display
        cd "$crate_path" 2>/dev/null || continue
        version=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
        cd "$WORKSPACE_ROOT" 2>/dev/null || cd - > /dev/null
        echo -e "${YELLOW}⏭️  Skipping $crate_name v$version (already published on crates.io)${NC}"
        SKIPPED_COUNT=$((SKIPPED_COUNT + 1))
        continue
    elif [ "$publish_decision" = "skip_unchanged" ]; then
        echo -e "${BLUE}⏭️  Skipping $crate_name (not in changed list)${NC}"
        SKIPPED_COUNT=$((SKIPPED_COUNT + 1))
        continue
    elif [ "$publish_decision" = "publish_unpublished" ]; then
        echo -e "${CYAN}📌 Publishing $crate_name (unpublished dependency)${NC}"
    fi
    
    # With `set -e`, a non-zero return would abort the script before we can
    # record failures. Temporarily disable errexit around the publish attempt.
    set +e
    publish_crate "$crate_name"
    publish_rc=$?
    set -e

    if [ "$publish_rc" -eq 0 ]; then
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

