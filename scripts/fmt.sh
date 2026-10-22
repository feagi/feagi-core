#!/usr/bin/env bash
# Copyright 2025 Neuraville Inc.
# SPDX-License-Identifier: Apache-2.0
#
# Format all Rust code in feagi-core with rustfmt.
# Run this before committing to ensure CI passes (pre-merge runs cargo fmt --check).

set -e

cd "${0%/*}/.."
cargo fmt --all
