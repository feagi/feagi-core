// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Genome schema version primitives.
//!
//! See `crates/feagi-evolutionary/src/genome/README.md` and
//! `feagi-core/docs/GENOME_SCHEMA_VERSIONING.md` for the design rationale,
//! invariants, and the procedure for adding a new schema version.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Integer schema version of a genome.
///
/// Serializes as a bare integer on the wire so it can be queried directly
/// from MongoDB and so diffs are obvious. The legacy human-readable
/// `version` string field on a genome is unrelated and MUST NOT be used
/// to drive code dispatch.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GenomeSchemaVersion(pub u32);

impl GenomeSchemaVersion {
    pub const fn new(version: u32) -> Self {
        Self(version)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl fmt::Display for GenomeSchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// Lowest schema version recognized by this crate.
///
/// The integer space starts at 2. There is no `v1`: the project never
/// persisted a genome at that integer in any production database or in
/// the offline `g0/` corpus. The chain registry is contiguous starting
/// at this constant.
pub const MIN_SCHEMA_VERSION: GenomeSchemaVersion = GenomeSchemaVersion(2);

/// Latest schema version. New genomes are produced at this version, and
/// `Validator(CURRENT_SCHEMA_VERSION)` is the only blocking validator.
pub const CURRENT_SCHEMA_VERSION: GenomeSchemaVersion = GenomeSchemaVersion(3);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_follows_integer_value() {
        assert!(GenomeSchemaVersion(2) < GenomeSchemaVersion(3));
        assert!(MIN_SCHEMA_VERSION <= CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn current_is_at_or_above_min() {
        assert!(CURRENT_SCHEMA_VERSION >= MIN_SCHEMA_VERSION);
    }

    #[test]
    fn min_is_two() {
        assert_eq!(MIN_SCHEMA_VERSION.as_u32(), 2);
    }

    #[test]
    fn current_is_three() {
        assert_eq!(CURRENT_SCHEMA_VERSION.as_u32(), 3);
    }

    #[test]
    fn serializes_as_bare_integer() {
        let v = GenomeSchemaVersion(3);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "3");
    }

    #[test]
    fn deserializes_from_bare_integer() {
        let v: GenomeSchemaVersion = serde_json::from_str("3").unwrap();
        assert_eq!(v, GenomeSchemaVersion(3));
    }

    #[test]
    fn round_trips_through_serde() {
        let v = GenomeSchemaVersion(42);
        let json = serde_json::to_string(&v).unwrap();
        let back: GenomeSchemaVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn display_uses_v_prefix() {
        assert_eq!(format!("{}", GenomeSchemaVersion(3)), "v3");
    }

    #[test]
    fn const_constructor_matches_field_constructor() {
        const VIA_CONST: GenomeSchemaVersion = GenomeSchemaVersion::new(5);
        assert_eq!(VIA_CONST, GenomeSchemaVersion(5));
        assert_eq!(VIA_CONST.as_u32(), 5);
    }
}
