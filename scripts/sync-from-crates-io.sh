#!/usr/bin/env bash
# Copyright 2025 Neuraville Inc.
# SPDX-License-Identifier: Apache-2.0
#
# Sync workspace root and workspace.dependencies to match latest versions
# published on crates.io. Prevents packaging failures when the root points
# at older versions than dependents already published on crates.io.
#
# Run this at the start of a release (before smart-version-bump) so the
# baseline matches crates.io; then only changed crates get bumped.

set -e

WORKSPACE_ROOT="${WORKSPACE_ROOT:-$(pwd)}"
DRY_RUN="${DRY_RUN:-false}"

SYNC_VERSION_MAP="$(mktemp /tmp/sync-from-crates-io--temp.XXXXXX)"
export SYNC_VERSION_MAP
export WORKSPACE_ROOT
cleanup_temp() {
    rm -f "$SYNC_VERSION_MAP"
}
trap cleanup_temp EXIT

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
        *) return 1 ;;
    esac
}

# Same order as publish-crates-smart.sh (exclude feagi umbrella)
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
)

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}Syncing root and workspace.dependencies from crates.io...${NC}"
echo ""

ROOT_CARGO="$WORKSPACE_ROOT/Cargo.toml"
if [ ! -f "$ROOT_CARGO" ]; then
    echo -e "${RED}ERROR: $ROOT_CARGO not found${NC}"
    exit 1
fi

# Fetch highest semver version from crates.io API.
# NOTE: The API returns versions by publication date, NOT semver order.
# A lower version can be published after a higher one (version regression).
# We must sort by semver and pick the highest to avoid dependency conflicts.
get_latest_on_crates_io() {
    local name="$1"
    local url="https://crates.io/api/v1/crates/${name}"
    local json
    json=$(curl -sL --max-time 15 -H "User-Agent: feagi-release/1.0" "$url" 2>/dev/null || echo "{}")
    if [ -z "$json" ] || [ "$json" = "{}" ]; then
        echo ""
        return
    fi
    echo "$json" | python3 -c "
import re, sys, json
try:
    d = json.load(sys.stdin)
    vers = d.get('versions') or []
    candidates = [v.get('num', '') for v in vers if not v.get('yanked', False) and v.get('num')]
    if not candidates:
        sys.exit(0)
    def semver_key(s):
        parts = re.split(r'[-.]', s)
        return [int(p) if p.isdigit() else p for p in parts]
    candidates.sort(key=semver_key, reverse=True)
    print(candidates[0])
except Exception:
    pass
" 2>/dev/null || echo ""
}

updated=0
for crate in "${CRATE_ORDER[@]}"; do
    latest=$(get_latest_on_crates_io "$crate")
    if [ -z "$latest" ]; then
        echo -e "  ${crate}: ${YELLOW}(not on crates.io, skipping)${NC}"
        continue
    fi

    # Get local version for this crate
    path=$(crate_path_for "$crate" 2>/dev/null) || path=""
    local_version=""
    if [ -n "$path" ]; then
        crate_toml="$WORKSPACE_ROOT/$path/Cargo.toml"
        if [ -f "$crate_toml" ]; then
            local_version=$(grep '^version = "' "$crate_toml" | head -1 | sed 's/version = "\(.*\)"/\1/' | xargs)
        fi
    fi

    # Only upgrade: skip if local version is already >= crates.io
    if [ -n "$local_version" ]; then
        higher=$(printf '%s\n' "$local_version" "$latest" | sort -V | tail -1)
        if [ "$higher" = "$local_version" ] && [ "$local_version" != "$latest" ]; then
            echo -e "  ${crate}: ${CYAN}local ${local_version} > crates.io ${latest}, keeping local${NC}"
            continue
        fi
        if [ "$local_version" = "$latest" ]; then
            echo -e "  ${crate}: ${GREEN}${latest} (already matches)${NC}"
            echo "${crate}|${latest}" >> "$SYNC_VERSION_MAP"
            continue
        fi
    fi

    echo -e "  ${crate}: ${GREEN}${local_version:-?} -> ${latest}${NC}"
    echo "${crate}|${latest}" >> "$SYNC_VERSION_MAP"

    if [ "$DRY_RUN" = "true" ]; then
        continue
    fi

    # Update root Cargo.toml: [dependencies] and [workspace.dependencies]
    if grep -q "^${crate} = " "$ROOT_CARGO" 2>/dev/null; then
        sed "s/^${crate} = { version = \"=[^\"]*\"/${crate} = { version = \"=${latest}\"/" "$ROOT_CARGO" > "${ROOT_CARGO}.tmp"
        mv "${ROOT_CARGO}.tmp" "$ROOT_CARGO"
        updated=$((updated + 1))
    fi

    # Update crate's own version so workspace is aligned with crates.io
    # Only when crate has explicit version = "..." (not version.workspace = true)
    if [ -n "$path" ]; then
        crate_toml="$WORKSPACE_ROOT/$path/Cargo.toml"
        if [ -f "$crate_toml" ] && grep -q '^version = "' "$crate_toml" 2>/dev/null; then
            sed 's/^version = ".*"/version = "'"$latest"'"/' "$crate_toml" > "${crate_toml}.tmp"
            mv "${crate_toml}.tmp" "$crate_toml"
        fi
    fi
done

if [ "$DRY_RUN" != "true" ] && [ -s "$SYNC_VERSION_MAP" ]; then
    python3 - <<'PY'
import os
import re
from pathlib import Path

workspace_root = Path(os.environ.get("WORKSPACE_ROOT", ".")).resolve()
map_file = Path(os.environ["SYNC_VERSION_MAP"])
version_map = {}
for line in map_file.read_text(encoding="utf-8").splitlines():
    if "|" not in line:
        continue
    name, version = line.split("|", 1)
    if name and version:
        version_map[name.strip()] = version.strip()

if not version_map:
    raise SystemExit(0)

manifests = []
manifests.append(workspace_root / "Cargo.toml")
manifests.extend(workspace_root.glob("crates/*/Cargo.toml"))
manifests.extend(workspace_root.glob("crates/feagi-npu/*/Cargo.toml"))

for manifest in manifests:
    if not manifest.exists():
        continue
    text = manifest.read_text(encoding="utf-8")
    lines = text.splitlines()
    updated = False
    for idx, line in enumerate(lines):
        for name, version in version_map.items():
            if not line.lstrip().startswith(f"{name} "):
                continue
            if "path" not in line or "version" not in line or "{" not in line:
                continue
            match = re.match(rf'^(\s*{re.escape(name)}\s*=\s*\{{)(.*)(\}})(\s*#.*)?$', line)
            if not match:
                continue
            middle = match.group(2)
            middle = re.sub(r'\bversion\s*=\s*"[^"]*"', f'version = "={version}"', middle)
            comment = match.group(4) or ""
            lines[idx] = f"{match.group(1)}{middle}{match.group(3)}{comment}"
            updated = True
            break
    if updated:
        manifest.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
fi

echo ""
if [ "$updated" -gt 0 ]; then
    echo -e "${GREEN}Updated $updated dependency version(s) in $ROOT_CARGO to match crates.io${NC}"
else
    echo -e "${CYAN}No updates needed (or dry run)${NC}"
fi
echo ""
