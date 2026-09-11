// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Latest-version genome validator.
//!
//! `V3Validator` is the **blocking** validator at `CURRENT_SCHEMA_VERSION`.
//! Step 2 of the schema-versioning rollout lands it as a placeholder that
//! returns a clean report. The real rules currently live in
//! `crate::validator::validate_genome` (which operates on `RuntimeGenome`)
//! and will be relocated here once the chain runner is wired into the
//! loader (step 4 of the plan in `docs/GENOME_SCHEMA_VERSIONING.md`).
//!
//! This placeholder is intentionally a no-op: the loader still uses the
//! existing `validate_genome`/`auto_fix_genome` path. Wiring up `V3Validator`
//! prematurely would either double-validate (subtly different rule sets) or
//! silently override existing behavior. Both are worse than a stub.

use serde_json::Value;

use super::{ValidationReport, Validator};
use crate::genome::schema::{GenomeSchemaVersion, CURRENT_SCHEMA_VERSION};

/// Validator for the latest schema version.
///
/// Currently a placeholder; see module docs.
#[derive(Debug, Default, Clone, Copy)]
pub struct V3Validator;

impl V3Validator {
    pub const fn new() -> Self {
        Self
    }
}

impl Validator for V3Validator {
    fn schema_version(&self) -> GenomeSchemaVersion {
        CURRENT_SCHEMA_VERSION
    }

    fn validate(&self, _genome: &Value) -> ValidationReport {
        ValidationReport::new(CURRENT_SCHEMA_VERSION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reports_current_schema_version() {
        let v = V3Validator::new();
        assert_eq!(v.schema_version(), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn placeholder_returns_clean_report() {
        // Until step 4 plugs in the real rules, V3Validator must accept
        // any input. This test pins that contract so a future change that
        // adds rules is forced to update both the rules and the wiring at
        // the same time.
        let v = V3Validator::new();
        let report = v.validate(&json!({ "anything": "goes" }));
        assert!(report.is_clean());
        assert_eq!(report.schema_version, Some(CURRENT_SCHEMA_VERSION));
    }
}
