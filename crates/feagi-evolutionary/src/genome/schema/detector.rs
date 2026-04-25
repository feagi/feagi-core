// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Resolve the schema version of an arbitrary genome `Value`.
//!
//! The detector reads the integer `genome_schema_version` field if present,
//! otherwise back-fills from the legacy human-readable `version` string per a
//! closed table. Anything else is rejected with a structured error. Shape
//! sniffing is forbidden; see the module-level README.

use serde_json::Value;

use super::version::GenomeSchemaVersion;
use crate::types::{EvoError, EvoResult};

/// Resolve a genome's schema version from its JSON representation.
///
/// Resolution order:
/// 1. Top-level integer field `genome_schema_version` if present.
/// 2. Legacy `version` string back-filled per the closed table:
///    `"2.0"` -> 2, `"3.0"` -> 3.
/// 3. Anything else returns `EvoError::InvalidGenome`.
///
/// The closed table is intentional. New schema versions are introduced by
/// writing the integer field directly; they do not get a corresponding
/// legacy string.
pub fn detect_schema_version(genome: &Value) -> EvoResult<GenomeSchemaVersion> {
    if let Some(integer) = genome.get("genome_schema_version").and_then(|v| v.as_u64()) {
        let narrowed: u32 = integer.try_into().map_err(|_| {
            EvoError::InvalidGenome(format!(
                "genome_schema_version {} exceeds u32 range",
                integer
            ))
        })?;
        return Ok(GenomeSchemaVersion(narrowed));
    }

    let legacy = genome.get("version").and_then(|v| v.as_str());
    match legacy {
        Some("2.0") => Ok(GenomeSchemaVersion(2)),
        Some("3.0") => Ok(GenomeSchemaVersion(3)),
        Some(other) => Err(EvoError::InvalidGenome(format!(
            "Unsupported legacy genome version string '{}'. Expected '2.0' or '3.0', \
             or an explicit integer `genome_schema_version` field.",
            other
        ))),
        None => Err(EvoError::InvalidGenome(
            "Genome is missing both `genome_schema_version` (integer) and `version` (legacy \
             string)"
                .to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn integer_field_takes_priority_over_legacy_string() {
        let g = json!({ "genome_schema_version": 7, "version": "2.0" });
        assert_eq!(detect_schema_version(&g).unwrap(), GenomeSchemaVersion(7));
    }

    #[test]
    fn legacy_2_0_string_maps_to_two() {
        let g = json!({ "version": "2.0" });
        assert_eq!(detect_schema_version(&g).unwrap(), GenomeSchemaVersion(2));
    }

    #[test]
    fn legacy_3_0_string_maps_to_three() {
        let g = json!({ "version": "3.0" });
        assert_eq!(detect_schema_version(&g).unwrap(), GenomeSchemaVersion(3));
    }

    #[test]
    fn explicit_integer_alone_is_sufficient() {
        let g = json!({ "genome_schema_version": 3 });
        assert_eq!(detect_schema_version(&g).unwrap(), GenomeSchemaVersion(3));
    }

    #[test]
    fn unsupported_legacy_string_is_rejected() {
        let g = json!({ "version": "1.0" });
        assert!(matches!(
            detect_schema_version(&g),
            Err(EvoError::InvalidGenome(_))
        ));
    }

    #[test]
    fn legacy_v0_0_1_string_is_rejected() {
        // The networking.json file in g0/discovery_day_2024/MuJoCo carries
        // this shape but is not a genome. The detector must reject it
        // explicitly rather than guess.
        let g = json!({ "version": "v0.0.1" });
        assert!(matches!(
            detect_schema_version(&g),
            Err(EvoError::InvalidGenome(_))
        ));
    }

    #[test]
    fn missing_both_fields_is_rejected() {
        let g = json!({});
        assert!(matches!(
            detect_schema_version(&g),
            Err(EvoError::InvalidGenome(_))
        ));
    }

    #[test]
    fn integer_above_u32_is_rejected() {
        let g = json!({ "genome_schema_version": (u32::MAX as u64 + 1) });
        assert!(matches!(
            detect_schema_version(&g),
            Err(EvoError::InvalidGenome(_))
        ));
    }

    #[test]
    fn shape_sniffing_is_not_attempted() {
        // No version fields at all but with blueprint-shaped data should
        // still be rejected. The detector does not infer from shape.
        let g = json!({ "blueprint": {}, "brain_regions": {} });
        assert!(matches!(
            detect_schema_version(&g),
            Err(EvoError::InvalidGenome(_))
        ));
    }

    #[test]
    fn legacy_string_with_non_canonical_minor_is_rejected() {
        // "2.1" is the human-readable label some saved files carry, but the
        // back-fill table is closed: only "2.0" and "3.0" map.
        let g = json!({ "version": "2.1" });
        assert!(matches!(
            detect_schema_version(&g),
            Err(EvoError::InvalidGenome(_))
        ));
    }

    #[test]
    fn integer_field_of_wrong_type_falls_through_to_legacy() {
        // If the integer field is present but malformed (string, bool, null)
        // the detector ignores it and consults the legacy string. This keeps
        // the resolver tolerant of clearly-broken explicit fields without
        // shape sniffing.
        let g = json!({ "genome_schema_version": "three", "version": "3.0" });
        assert_eq!(detect_schema_version(&g).unwrap(), GenomeSchemaVersion(3));
    }
}
