#!/bin/bash
# Copyright 2025 Neuraville Inc.
# SPDX-License-Identifier: Apache-2.0

# Multi-crate publishing script for feagi-core workspace
# Publishes crates in dependency order with delays for crates.io indexing

set -e  # Exit on error

CARGO_TOKEN="${CARGO_REGISTRY_TOKEN:-}"
DRY_RUN="${DRY_RUN:-false}"
DELAY_SECONDS=30  # Delay between publishes for crates.io indexing

if [ -z "$CARGO_TOKEN" ] && [ "$DRY_RUN" != "true" ]; then
    echo "❌ Error: CARGO_REGISTRY_TOKEN environment variable must be set"
    exit 1
fi

echo "🚀 Publishing feagi-core workspace crates to crates.io"
echo "📦 Dry run: $DRY_RUN"
echo ""

# Publication order - dependencies first
CRATES=(
    # Layer 1: Foundation (no internal dependencies)
    "crates/feagi-observability"
    
    # Layer 2: Core data structures
    "crates/feagi-data-structures"
    "crates/feagi-config"
    
    # Layer 3: Neural foundations
    "crates/feagi-npu/neural"
    
    # Layer 4: Runtime abstractions
    "crates/feagi-npu/runtime"
    
    # Layer 5: Serialization and state
    "crates/feagi-serialization"
    "crates/feagi-state-manager"
    
    # Layer 6: High-performance processing
    "crates/feagi-npu/burst-engine"
    "crates/feagi-npu/plasticity"
    
    # Layer 7: Evolutionary and development
    "crates/feagi-evolutionary"
    "crates/feagi-brain-development"
    
    # Layer 8: I/O Layer
    "crates/feagi-io"
    "crates/feagi-sensorimotor"
    
    # Layer 9: Services & API
    "crates/feagi-services"
    "crates/feagi-api"
    
    # Layer 10: Agent & Platform
    "crates/feagi-agent"
    "crates/feagi-hal"
    
    # Root workspace (meta-crate, publishes last)
    "."
)

publish_crate() {
    local crate_path=$1
    local crate_name=$(basename "$crate_path")
    
    # Get crate name from Cargo.toml
    if [ "$crate_path" = "." ]; then
        local actual_name=$(grep '^name = ' Cargo.toml | head -1 | sed 's/name = "\(.*\)"/\1/')
    else
        local actual_name=$(grep '^name = ' "$crate_path/Cargo.toml" | head -1 | sed 's/name = "\(.*\)"/\1/')
    fi
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📦 Publishing: $actual_name"
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
    if cargo search "$actual_name" --limit 1 | grep -q "^$actual_name = \"$version\""; then
        echo "⏭️  Skipping $actual_name v$version (already published)"
        cd - > /dev/null
        return 0
    fi
    
    # Package first to verify
    echo "   📦 Packaging..."
    if ! cargo package --quiet; then
        echo "❌ Failed to package $actual_name"
        cd - > /dev/null
        return 1
    fi
    
    # Publish
    if [ "$DRY_RUN" = "true" ]; then
        echo "   🧪 Dry run: cargo publish --dry-run"
        cargo publish --dry-run
    else
        echo "   🚀 Publishing to crates.io..."
        if cargo publish --token "$CARGO_TOKEN"; then
            echo "✅ Successfully published $actual_name v$version"
            
            # Delay for crates.io indexing (except for last crate)
            if [ "$crate_path" != "." ]; then
                echo "   ⏳ Waiting ${DELAY_SECONDS}s for crates.io indexing..."
                sleep $DELAY_SECONDS
            fi
        else
            echo "❌ Failed to publish $actual_name"
            cd - > /dev/null
            return 1
        fi
    fi
    
    # Return to workspace root
    if [ "$crate_path" != "." ]; then
        cd - > /dev/null
    fi
    
    return 0
}

# Main execution
echo "Starting publication process..."
echo ""

FAILED_CRATES=()
PUBLISHED_COUNT=0

for crate_path in "${CRATES[@]}"; do
    if [ ! -f "$crate_path/Cargo.toml" ] && [ "$crate_path" != "." ]; then
        echo "⚠️  Warning: $crate_path not found, skipping..."
        continue
    fi
    
    if publish_crate "$crate_path"; then
        ((PUBLISHED_COUNT++))
    else
        FAILED_CRATES+=("$crate_path")
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Publication Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Successfully published: $PUBLISHED_COUNT crates"

if [ ${#FAILED_CRATES[@]} -gt 0 ]; then
    echo "❌ Failed to publish: ${#FAILED_CRATES[@]} crates"
    echo ""
    echo "Failed crates:"
    for crate in "${FAILED_CRATES[@]}"; do
        echo "  - $crate"
    done
    echo ""
    exit 1
else
    echo ""
    echo "🎉 All crates published successfully!"
    exit 0
fi

