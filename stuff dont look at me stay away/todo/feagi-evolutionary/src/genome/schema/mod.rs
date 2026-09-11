// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Genome schema versioning primitives.
//!
//! Owns:
//! - The `GenomeSchemaVersion` integer type and `MIN`/`CURRENT` constants
//!   (`version` submodule).
//! - The closed-table `detect_schema_version` resolver (`detector` submodule).
//!
//! Migration step files (`v2_to_v3.rs`, ...) and per-version validators live
//! in sibling modules and will be added in subsequent PRs. See
//! `feagi-core/docs/GENOME_SCHEMA_VERSIONING.md` for the implementation order
//! and `crates/feagi-evolutionary/src/genome/README.md` for the contributor
//! contract.

pub mod detector;
pub mod version;

pub use detector::detect_schema_version;
pub use version::{GenomeSchemaVersion, CURRENT_SCHEMA_VERSION, MIN_SCHEMA_VERSION};
