//! `Scorecard` v1 — a portable, verifiable benchmark result bound to a connectome.
//!
//! A `Scorecard` is the Trainer's terminal evaluation artifact (ADR-012): a separate,
//! versioned record that *references* the brain under test (it never mutates a genome), so
//! one connectome can carry many scorecards — one per dataset/protocol — plus a history.
//!
//! It is generated **entirely locally and offline** by the open `feagi-trainer` crate from
//! the run inputs (the pinned connectome, the resolved `DatasetManifest`, the `RunSpec`
//! protocol) and the computed metrics; it performs no Composer/cloud I/O (ADR-006). The
//! `status`/`visibility` lifecycle fields live here so feagi-desktop + Composer can drive
//! publishing, Brain-Hub binding, and competitions/leaderboards *without changing the
//! Trainer* (ADR-012). Scorecards default to `status: self_reported`, `visibility: local`.
//!
//! Verification is by re-run: given the pinned `connectome_hash` + dataset version +
//! `evaluation_protocol_version`, any party can re-execute and confirm the metrics within
//! tolerance, at which point `status` becomes `verified` (ADR-003, ADR-012).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::common::{
    BackendKind, ConnectomeHash, ContentHash, DatasetAssetId, EvaluationProtocolVersion,
    GenomeVersionId, MetadataMap, PluginRef, QuantizationFingerprint, ScorecardId, SplitId,
};
use super::metric_stats::MetricStat;

/// Wire/format version of the `Scorecard` contract.
pub const SCHEMA_VERSION: u32 = 1;

/// Verification state of a scorecard's metrics (ADR-012).
///
/// A scorecard is `SelfReported` when produced by its author and becomes `Verified` only
/// when an independent re-run reproduces the metrics within tolerance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScorecardStatus {
    /// Produced locally by the run's author; not independently re-run.
    SelfReported,
    /// Reproduced by an independent re-run within tolerance.
    Verified,
}

/// Publication state of a scorecard (ADR-012).
///
/// Scorecards are generated and stored locally by default. Publishing is a distinct,
/// user-triggered action owned by feagi-desktop + Composer (with a public-genome
/// prerequisite); the Trainer never publishes and never sets this beyond `Local`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScorecardVisibility {
    /// Private to the local artifact store (default).
    Local,
    /// Published (set by desktop/Composer, bound to a public genome identity).
    Published,
}

/// Fingerprint of the execution environment that produced the metrics (ADR-003).
///
/// CPU is the deterministic verification baseline; GPU is recorded as a fingerprint only and
/// is not verification-grade for published scorecards. Crate/version fields anchor the
/// software identity used for the run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendFingerprint {
    /// Execution backend used for the run.
    pub backend: BackendKind,
    /// Human-readable backend descriptor (e.g. `x86_64-cpu`).
    pub descriptor: String,
    /// Quantization configuration fingerprint, if the brain under test is quantized.
    pub quantization: Option<QuantizationFingerprint>,
    /// `feagi-trainer` crate version that produced the scorecard.
    pub trainer_version: String,
    /// `feagi-core` version of the runtime under test.
    pub feagi_core_version: String,
}

/// A portable, verifiable benchmark result bound to a connectome (ADR-012).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scorecard {
    /// Wire/format version; must equal [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Identity of this scorecard record.
    pub scorecard_id: ScorecardId,
    /// Pinned, re-runnable connectome under test — the verification anchor (ADR-003).
    pub connectome_hash: ConnectomeHash,
    /// Optional lineage reference to the source genome version.
    pub genome_version_id: Option<GenomeVersionId>,
    /// Stable asset id of the dataset scored against (resolves locally now — ADR-012).
    pub dataset_asset_id: DatasetAssetId,
    /// Human-facing dataset version string the score is bound to.
    pub dataset_version: String,
    /// Content hash binding the score to exact dataset bytes/labels.
    pub dataset_content_hash: ContentHash,
    /// Evaluation protocol semantics version (episode/aggregation/threshold policy).
    pub evaluation_protocol_version: EvaluationProtocolVersion,
    /// Metric pack that computed the metrics.
    pub metric_pack: PluginRef,
    /// Split the score was computed over.
    pub split_id: SplitId,
    /// Execution-environment fingerprint that produced the metrics.
    pub backend_fingerprint: BackendFingerprint,
    /// Computed metric values (deterministically ordered). For a multi-seed run these are the
    /// per-metric means (the point estimates); for a single run they are that run's values.
    pub metrics: BTreeMap<String, f64>,
    /// Per-metric distribution across N-seed repeats (mean/stddev/confidence interval).
    ///
    /// Present only for multi-seed runs; omitted (and absent from the wire form) for single
    /// runs so the contract stays backward-compatible (additive evolution, ADR-006). The `n`
    /// and `confidence_level` inside each [`MetricStat`] record the repeat provenance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric_stats: Option<BTreeMap<String, MetricStat>>,
    /// Verification state of the metrics.
    pub status: ScorecardStatus,
    /// Publication state (Trainer emits `Local`; desktop/Composer may publish).
    pub visibility: ScorecardVisibility,
    /// Free-form, deterministically ordered metadata.
    pub metadata: MetadataMap,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iris_scorecard() -> Scorecard {
        let mut metrics = BTreeMap::new();
        metrics.insert("accuracy".to_string(), 0.9667);
        metrics.insert("macro_f1".to_string(), 0.9655);
        Scorecard {
            schema_version: SCHEMA_VERSION,
            scorecard_id: ScorecardId("sc-0001".to_string()),
            connectome_hash: ConnectomeHash("sha256:connectome".to_string()),
            genome_version_id: None,
            dataset_asset_id: DatasetAssetId("local:iris".to_string()),
            dataset_version: "1.0.0".to_string(),
            dataset_content_hash: ContentHash("sha256:abc".to_string()),
            evaluation_protocol_version: EvaluationProtocolVersion("clf-v1".to_string()),
            metric_pack: PluginRef {
                id: super::super::common::PluginId("classification".to_string()),
                version: "1.0.0".to_string(),
            },
            split_id: SplitId("test".to_string()),
            backend_fingerprint: BackendFingerprint {
                backend: BackendKind::Cpu,
                descriptor: "x86_64-cpu".to_string(),
                quantization: None,
                trainer_version: env!("CARGO_PKG_VERSION").to_string(),
                feagi_core_version: "0.0.12".to_string(),
            },
            metrics,
            metric_stats: None,
            status: ScorecardStatus::SelfReported,
            visibility: ScorecardVisibility::Local,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn schema_version_is_pinned() {
        assert_eq!(iris_scorecard().schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn defaults_are_local_and_self_reported() {
        let card = iris_scorecard();
        assert_eq!(card.status, ScorecardStatus::SelfReported);
        assert_eq!(card.visibility, ScorecardVisibility::Local);
    }

    #[test]
    fn json_round_trip_preserves_scorecard() {
        let card = iris_scorecard();
        let json = serde_json::to_string(&card).expect("serialize");
        let restored: Scorecard = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(card, restored);
    }

    #[test]
    fn metric_stats_is_omitted_when_absent() {
        // Backward compatibility: a single-run scorecard must not emit the new field, and an old
        // wire form lacking it must still deserialize (additive evolution, ADR-006).
        let card = iris_scorecard();
        let json = serde_json::to_string(&card).expect("serialize");
        assert!(!json.contains("metric_stats"));
        let restored: Scorecard = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.metric_stats, None);
    }

    #[test]
    fn metric_stats_round_trips_when_present() {
        let mut card = iris_scorecard();
        let mut stats = BTreeMap::new();
        stats.insert(
            "accuracy".to_string(),
            MetricStat {
                n: 5,
                mean: 0.9,
                stddev: 0.05,
                ci_low: 0.84,
                ci_high: 0.96,
                confidence_level: 0.95,
            },
        );
        card.metric_stats = Some(stats);
        let json = serde_json::to_string(&card).expect("serialize");
        assert!(json.contains("metric_stats"));
        let restored: Scorecard = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(card, restored);
    }
}
