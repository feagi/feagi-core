#!/usr/bin/env bash
# Copyright 2025 Neuraville Inc.
# SPDX-License-Identifier: Apache-2.0
#
# Bump the single umbrella version for feagi-core (unified versioning).
# Reads [workspace.package].version from root Cargo.toml, computes the next
# version (0.0.1-beta.N -> N+1, or X.Y.Z semver -> X.Y.(Z+1)), updates root
# and all path dependency version references, then outputs NEW_VERSION and
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
    in_ws && /^version[[:space:]]*=[[:space:]]*"/ { gsub(/^version[[:space:]]*=[[:space:]]*"|"$/, ""); print; exit }
' "$ROOT_CARGO")

if [ -z "$CURRENT" ]; then
    echo "ERROR: Could not read [workspace.package] version from $ROOT_CARGO" >&2
    exit 1
fi

# Bump rules:
#   0.0.1-beta.N -> 0.0.1-beta.(N+1)
#   X.Y.Z (semver) -> X.Y.(Z+1)
NEW=$(CURRENT="$CURRENT" python3 <<'PY'
import os
import re
import sys

v = os.environ["CURRENT"].strip()
m = re.match(r"^(.*-beta\.)([0-9]+)$", v)
if m:
    prefix, num = m.group(1), int(m.group(2))
    print(f"{prefix}{num + 1}")
    sys.exit(0)
m2 = re.match(r"^([0-9]+)\.([0-9]+)\.([0-9]+)$", v)
if m2:
    major, minor, patch = int(m2.group(1)), int(m2.group(2)), int(m2.group(3))
    print(f"{major}.{minor}.{patch + 1}")
    sys.exit(0)
raise SystemExit(f"Unsupported version format for bump: {v}")
PY
)

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
