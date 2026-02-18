#!/usr/bin/env bash
# Copyright 2025 Neuraville Inc.
# SPDX-License-Identifier: Apache-2.0
#
# Bump the single umbrella version for feagi-core (unified versioning).
# Reads [workspace.package].version from root Cargo.toml, increments the
# beta segment (e.g. 0.0.1-beta.18 -> 0.0.1-beta.19), updates root and
# all path dependency version references, then outputs NEW_VERSION and
# CHANGED_CRATES for the release workflow.

set -e

WORKSPACE_ROOT="${WORKSPACE_ROOT:-$(pwd)}"
DRY_RUN="${DRY_RUN:-false}"
ROOT_CARGO="$WORKSPACE_ROOT/Cargo.toml"

if [ ! -f "$ROOT_CARGO" ]; then
    echo "ERROR: $ROOT_CARGO not found" >&2
    exit 1
fi

# Read current version from [workspace.package]
CURRENT=$(awk '
    /^\[workspace\.package\]/ { in_ws = 1; next }
    /^\[/ { if (in_ws) exit }
    in_ws && /^version\s*=\s*"/ { gsub(/^version\s*=\s*"|"$/, ""); print; exit }
' "$ROOT_CARGO")

if [ -z "$CURRENT" ]; then
    echo "ERROR: Could not read [workspace.package] version from $ROOT_CARGO" >&2
    exit 1
fi

# Bump 0.0.1-beta.N -> 0.0.1-beta.(N+1)
NEW=$(python3 -c "
import re
v = '''$CURRENT'''
m = re.match(r'^(.*-beta\.)([0-9]+)$', v)
if m:
    prefix, num = m.group(1), int(m.group(2))
    print(f'{prefix}{num + 1}')
else:
    raise SystemExit(f'Unsupported version format for bump: {v}')
")

if [ -z "$NEW" ]; then
    echo "ERROR: Failed to compute next version from $CURRENT" >&2
    exit 1
fi

echo "Bumping umbrella version: $CURRENT -> $NEW"

if [ "$DRY_RUN" = "true" ]; then
    echo "[DRY RUN] Would update version to $NEW"
else
    # Update [workspace.package].version and root [package].version
    sed -i.bak "s/^version = \"$CURRENT\"/version = \"$NEW\"/" "$ROOT_CARGO"
    rm -f "${ROOT_CARGO}.bak"

    # Replace current version with new in all Cargo.toml path dependency refs
    for manifest in "$ROOT_CARGO" "$WORKSPACE_ROOT"/crates/*/Cargo.toml "$WORKSPACE_ROOT"/crates/feagi-npu/*/Cargo.toml; do
        [ -f "$manifest" ] || continue
        if grep -q "$CURRENT" "$manifest" 2>/dev/null; then
            sed -i.bak "s/$CURRENT/$NEW/g" "$manifest"
            rm -f "${manifest}.bak"
        fi
    done
fi

# All publishable crates (unified = all get the same version)
CHANGED_CRATES="feagi-observability feagi-structures feagi-config feagi-npu-neural feagi-npu-runtime feagi-serialization feagi-state-manager feagi-npu-burst-engine feagi-npu-plasticity feagi-evolutionary feagi-brain-development feagi-sensorimotor feagi-services feagi-io feagi-agent feagi-api feagi-hal feagi"

echo "NEW_VERSION=$NEW"
echo "CHANGED_CRATES=$CHANGED_CRATES"
