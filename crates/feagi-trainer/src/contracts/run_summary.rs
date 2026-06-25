//! `RunSummary` v1 — the terminal status + headline metrics of a run.
//!
//! Summarizes a run's lifecycle outcome and aggregate metrics, and links to the generated
//! `Scorecard` (design Section 5.7/5.9). Per-sample detail lives in `PredictionRecord`s.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::common::{MetadataMap, RunId, ScorecardId};

/// Wire/format version of the `RunSummary` contract.
pub const SCHEMA_VERSION: u32 = 1;

/// Run lifecycle state (design Section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// Created but not yet validated.
    Created,
    /// Undergoing pre-run validation (binding resolution, compatibility checks).
    Validating,
    /// Executing.
    Running,
    /// Finished successfully.
    Completed,
    /// Terminated with an error.
    Failed,
}

/// Terminal summary of a run (design Section 5.7/5.9).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunSummary {
    /// Wire/format version; must equal [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Run this summary describes.
    pub run_id: RunId,
    /// Terminal lifecycle status.
    pub status: RunStatus,
    /// Total samples planned for the run.
    pub total_samples: u64,
    /// Samples actually evaluated.
    pub evaluated_samples: u64,
    /// Aggregate metric values (deterministically ordered).
    pub metrics: BTreeMap<String, f64>,
    /// Run start time, Unix epoch milliseconds.
    pub started_at: Option<i64>,
    /// Run completion time, Unix epoch milliseconds.
    pub completed_at: Option<i64>,
    /// Generated scorecard, if any.
    pub scorecard_id: Option<ScorecardId>,
    /// Free-form, deterministically ordered metadata.
    pub metadata: MetadataMap,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary() -> RunSummary {
        let mut metrics = BTreeMap::new();
        metrics.insert("accuracy".to_string(), 0.9667);
        RunSummary {
            schema_version: SCHEMA_VERSION,
            run_id: RunId("run-0001".to_string()),
            status: RunStatus::Completed,
            total_samples: 30,
            evaluated_samples: 30,
            metrics,
            started_at: Some(1_000),
            completed_at: Some(2_000),
            scorecard_id: Some(ScorecardId("sc-0001".to_string())),
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn schema_version_is_pinned() {
        assert_eq!(summary().schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn json_round_trip_preserves_summary() {
        let value = summary();
        let json = serde_json::to_string(&value).expect("serialize");
        let restored: RunSummary = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(value, restored);
    }
}
