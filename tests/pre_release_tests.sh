#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

if [ -z "${BASH_VERSINFO:-}" ] || [ "${BASH_VERSINFO[0]}" -lt 4 ]; then
  echo "ERROR: bash 4+ is required to run this script."
  echo "Install via Homebrew: brew install bash"
  exit 1
fi

echo "Running feagi-core pre-release checks (no publish, no git writes)."

rustup component add rustfmt clippy

chmod +x scripts/smart-version-bump.sh
chmod +x scripts/apply-version-bumps.sh

VERSION_OUTPUT="$(./scripts/smart-version-bump.sh)"
echo "${VERSION_OUTPUT}"

VERSIONS_FILE="$(echo "${VERSION_OUTPUT}" | /usr/bin/grep "VERSIONS_FILE=" | /usr/bin/cut -d'=' -f2)"
if [ -z "${VERSIONS_FILE}" ]; then
  echo "ERROR: failed to detect VERSIONS_FILE from smart-version-bump output."
  exit 1
fi

export VERSIONS_FILE="${VERSIONS_FILE}"
./scripts/apply-version-bumps.sh

cargo fmt --all -- --check

echo "Verifying workspace path dependency version consistency..."
WORKSPACE_VERSION="$(
  /usr/bin/awk '
    BEGIN { in_ws=0 }
    /^\[workspace\.package\]/ { in_ws=1; next }
    /^\[/ { if (in_ws==1) exit }
    in_ws==1 && /^version = / {
      gsub(/version = /, "", $0);
      gsub(/"/, "", $0);
      print $0;
      exit
    }
  ' Cargo.toml
)"

if [ -z "${WORKSPACE_VERSION}" ]; then
  echo "ERROR: failed to detect [workspace.package] version from Cargo.toml."
  exit 1
fi

declare -A CRATE_VERSIONS

for manifest in Cargo.toml crates/*/Cargo.toml crates/feagi-npu/*/Cargo.toml; do
  [ -f "${manifest}" ] || continue
  name="$(/usr/bin/grep -E '^name = ' "${manifest}" | head -1 | /usr/bin/sed 's/name = "\(.*\)"/\1/')"
  if [ -z "${name}" ]; then
    continue
  fi

  if /usr/bin/grep -qE '^version\.workspace = true' "${manifest}"; then
    ver="${WORKSPACE_VERSION}"
  else
    ver="$(/usr/bin/grep -E '^version = ' "${manifest}" | head -1 | /usr/bin/sed 's/version = "\(.*\)"/\1/')"
  fi

  if [ -n "${ver}" ]; then
    CRATE_VERSIONS["${name}"]="${ver}"
  fi
done

mismatches=0

for manifest in Cargo.toml crates/*/Cargo.toml crates/feagi-npu/*/Cargo.toml; do
  [ -f "${manifest}" ] || continue

  while IFS= read -r line; do
    dep="$(echo "${line}" | /usr/bin/sed -n 's/^\s*\([A-Za-z0-9_-][A-Za-z0-9_-]*\)\s*=.*/\1/p')"
    req="$(echo "${line}" | /usr/bin/sed -n 's/.*version\s*=\s*"\([^"]\+\)".*/\1/p')"
    path="$(echo "${line}" | /usr/bin/sed -n 's/.*path\s*=\s*"\([^"]\+\)".*/\1/p')"

    [ -n "${dep}" ] || continue
    [ -n "${req}" ] || continue
    [ -n "${path}" ] || continue

    case "${path}" in
      ../*) ;;
      *) continue ;;
    esac

    req="${req#=}"
    req="${req#^}"

    actual="${CRATE_VERSIONS[${dep}]:-}"
    if [ -z "${actual}" ]; then
      continue
    fi

    if [ "${req}" != "${actual}" ]; then
      echo "ERROR: version mismatch in ${manifest}:"
      echo "  dependency: ${dep}"
      echo "  required:   ${req}"
      echo "  actual:     ${actual}"
      echo "  path:       ${path}"
      mismatches=$((mismatches + 1))
    fi
  done < <(/usr/bin/grep -E 'path\s*=\s*"\.\./[^\\"]+"' "${manifest}" || true)
done

if [ "${mismatches}" -gt 0 ]; then
  echo "ERROR: found ${mismatches} workspace path dependency mismatch(es)."
  exit 1
fi

echo "Workspace path dependency versions are consistent."

cargo clippy --workspace --lib --tests -- -D warnings
cargo test --workspace --lib --verbose
cargo test --workspace --lib --release --verbose
cargo build --release --lib --verbose

echo "Pre-release checks complete."
