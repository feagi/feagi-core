//! `PredictionRecord` v1 — one decoded model output linked to its source sample.
//!
//! Prediction records are the persisted, per-sample evidence the evaluation engine consumes
//! (design Section 5.8/5.9). For FEAGI a "prediction" is the decoded motor/OPU output for an
//! encoded sample; this contract is transport/engine-agnostic so it is produced identically
//! by the remote and embedded runtime paths.

use serde::{Deserialize, Serialize};

use super::common::{MetadataMap, OutputType, RunId, SampleId};
use super::ir_sample::TypedTarget;

/// Wire/format version of the `PredictionRecord` contract.
pub const SCHEMA_VERSION: u32 = 1;

/// A typed prediction, selected by `OutputType`.
///
/// Mirrors `TypedTarget` but carries prediction-specific detail (e.g. per-class scores).
/// Structured predictions (segmentation/pose/keypoints/bbox) are added additively with
/// their decoders and metric packs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypedPrediction {
    /// Single-label class prediction with optional per-class scores.
    Class {
        /// Predicted class index.
        class_id: u32,
        /// Optional per-class scores aligned to class index (empty if unavailable).
        scores: Vec<f64>,
    },
    /// Multi-label class prediction.
    ClassSet(Vec<u32>),
    /// Scalar regression prediction.
    Scalar(f64),
    /// Vector regression prediction.
    Vector(Vec<f64>),
}

/// One prediction linked to the sample that produced it (design Section 5.8/5.9).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictionRecord {
    /// Wire/format version; must equal [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Run that produced this prediction.
    pub run_id: RunId,
    /// Source sample.
    pub sample_id: SampleId,
    /// Structured-output task type (must match the sample's `output_type`).
    pub output_type: OutputType,
    /// The decoded prediction.
    pub prediction: TypedPrediction,
    /// Ground-truth target for the sample, when known (mirrors the sample's target).
    pub target: Option<TypedTarget>,
    /// Optional decode timestamp, Unix epoch milliseconds.
    pub timestamp: Option<i64>,
    /// Free-form, deterministically ordered metadata.
    pub metadata: MetadataMap,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn record() -> PredictionRecord {
        PredictionRecord {
            schema_version: SCHEMA_VERSION,
            run_id: RunId("run-0001".to_string()),
            sample_id: SampleId("iris-0001".to_string()),
            output_type: OutputType::Class,
            prediction: TypedPrediction::Class {
                class_id: 0,
                scores: vec![0.8, 0.1, 0.1],
            },
            target: Some(TypedTarget::Class {
                class_id: 0,
                label: Some("setosa".to_string()),
            }),
            timestamp: None,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn schema_version_is_pinned() {
        assert_eq!(record().schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn json_round_trip_preserves_record() {
        let value = record();
        let json = serde_json::to_string(&value).expect("serialize");
        let restored: PredictionRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(value, restored);
    }
}
