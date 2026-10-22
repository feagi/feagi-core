#!/usr/bin/env bash
# Copyright 2025 Neuraville Inc.
# SPDX-License-Identifier: Apache-2.0
#
# Format all Rust code in feagi-core with the rustfmt from rust-toolchain.toml.
# Run this before committing to ensure CI passes (pre-merge runs cargo fmt --check).
# Refuses to run if the active compiler is not that channel. A rustup toolchain
# named "stable" is not current stable on every machine.

set -euo pipefail

cd "${0%/*}/.."

expected="$(
  awk '
    /^\[toolchain\]/ { in_tc = 1; next }
    /^\[/ { if (in_tc) exit }
    in_tc && /^channel[[:space:]]*=/ {
      gsub(/channel[[:space:]]*=[[:space:]]*"|"/, "")
      print
      exit
    }
  ' rust-toolchain.toml
)"

if [ -z "${expected}" ]; then
  echo "ERROR: rust-toolchain.toml has no [toolchain].channel" >&2
  exit 1
fi

active="$(rustc --version)"
case "${active}" in
  "rustc ${expected}."*) ;;
  *)
    echo "ERROR: feagi-core requires rustc ${expected}.x from rust-toolchain.toml." >&2
    echo "Active compiler: ${active}" >&2
    echo "Refusing to format. A different rustfmt will not match CI." >&2
    exit 1
    ;;
esac

echo "Formatting with ${active}"
rustfmt --version
cargo fmt --all
