//! `EvaluationSpec` v1 — declares how a run's predictions are scored.
//!
//! The evaluation spec names the metric pack, pins the evaluation-protocol semantics, the
//! split scored, and the tolerance used for local self-verification (re-run-matches). It is
//! a determinant of comparability: two runs with different `evaluation_protocol_version`s
//! are not comparable (Appendix D.3).

use serde::{Deserialize, Serialize};

use super::common::{EvaluationProtocolVersion, MetadataMap, PluginRef, SplitId};

/// Wire/format version of the `EvaluationSpec` contract.
pub const SCHEMA_VERSION: u32 = 1;

/// Declarative evaluation configuration for a run (design Section 5.8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationSpec {
    /// Wire/format version; must equal [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Metric pack used to score predictions.
    pub metric_pack: PluginRef,
    /// Versioned evaluation-protocol semantics (comparability key).
    pub evaluation_protocol_version: EvaluationProtocolVersion,
    /// Split the metrics are computed on.
    pub split_id: SplitId,
    /// Absolute tolerance for local self-verification: a re-run is considered a match when
    /// every metric is within this tolerance of the original.
    pub verification_tolerance: f64,
    /// Free-form, deterministically ordered metadata.
    pub metadata: MetadataMap,
}

#[cfg(test)]
mod tests {
    use super::super::common::PluginId;
    use super::*;
    use std::collections::BTreeMap;

    fn spec() -> EvaluationSpec {
        EvaluationSpec {
            schema_version: SCHEMA_VERSION,
            metric_pack: PluginRef {
                id: PluginId("classification".to_string()),
                version: "1.0.0".to_string(),
            },
            evaluation_protocol_version: EvaluationProtocolVersion("clf-v1".to_string()),
            split_id: SplitId("test".to_string()),
            verification_tolerance: 1e-6,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn schema_version_is_pinned() {
        assert_eq!(spec().schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn json_round_trip_preserves_spec() {
        let value = spec();
        let json = serde_json::to_string(&value).expect("serialize");
        let restored: EvaluationSpec = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(value, restored);
    }
}
