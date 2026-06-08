//! `IRSample` v1 — the canonical intermediate representation a sample is normalized into.
//!
//! Adapters convert source formats into `IRSample`; encoder selectors then map an
//! `IRSample` into a FEAGI sensory payload. This decouples source-data complexity from
//! FEAGI runtime integration (design Section 5.3, Appendix B).

use serde::{Deserialize, Serialize};

use super::common::{DatasetVersionId, MetadataMap, Modality, OutputType, SampleId, Split};

/// Wire/format version of the `IRSample` contract.
pub const SCHEMA_VERSION: u32 = 1;

/// Optional spatial frame for dense/structured targets (Appendix B.4).
///
/// The authoritative coordinate transform lives inside the selected FEAGI coder's
/// properties; this record exposes a named frame plus opaque properties purely so
/// evaluation can align predictions to targets without re-implementing the transform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoordinateFrame {
    /// Identifier of the coordinate frame (e.g. image pixel frame, world frame).
    pub frame_id: String,
    /// Opaque frame properties mirrored from the coder configuration.
    pub properties: serde_json::Value,
}

/// The typed payload carried by a sample.
///
/// A typed union over modalities. Only the variants needed by current adapters are
/// modeled; new payload kinds are added additively as adapters land.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    /// One tabular row as an ordered vector of numeric features (e.g. IRIS).
    Tabular(Vec<f64>),
    /// A text document/sequence.
    Text(String),
    /// Raw bytes (e.g. an encoded image), interpreted per modality + adapter.
    Bytes(Vec<u8>),
}

/// The typed ground-truth target for a sample, selected by `OutputType`.
///
/// Replaces the legacy scalar-only `label`. Structured targets
/// (segmentation/pose/keypoints/bbox) are introduced additively alongside their FEAGI
/// coders and metric packs (Appendix B.6); this v1 models the classification/regression
/// subset exercised by the IRIS slice and tabular tasks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypedTarget {
    /// Single-label class target.
    Class {
        /// Zero-based class index.
        class_id: u32,
        /// Optional human-readable class label.
        label: Option<String>,
    },
    /// Multi-label class target (set of class indices).
    ClassSet(Vec<u32>),
    /// Scalar regression target.
    Scalar(f64),
    /// Vector regression target.
    Vector(Vec<f64>),
}

/// Canonical intermediate representation of a single dataset sample (design Section 5.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IRSample {
    /// Wire/format version; must equal [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Stable id of this sample within its dataset version.
    pub sample_id: SampleId,
    /// Dataset version this sample belongs to.
    pub dataset_version_id: DatasetVersionId,
    /// Split assignment.
    pub split: Split,
    /// Sample modality.
    pub modality: Modality,
    /// Typed payload.
    pub payload: Payload,
    /// Typed target, or `None` for unlabeled samples.
    pub target: Option<TypedTarget>,
    /// Structured-output task type for this sample.
    pub output_type: OutputType,
    /// Optional spatial frame for dense/structured targets (Appendix B.4).
    ///
    /// The authoritative spatial transform lives in the selected coder's properties; this
    /// is exposed only for evaluation alignment.
    pub coordinate_frame: Option<CoordinateFrame>,
    /// Optional acquisition timestamp, Unix epoch milliseconds.
    pub timestamp: Option<i64>,
    /// Free-form, deterministically ordered metadata.
    pub metadata: MetadataMap,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn iris_sample() -> IRSample {
        IRSample {
            schema_version: SCHEMA_VERSION,
            sample_id: SampleId("iris-0001".to_string()),
            dataset_version_id: DatasetVersionId("iris@1".to_string()),
            split: Split::Train,
            modality: Modality::Tabular,
            payload: Payload::Tabular(vec![5.1, 3.5, 1.4, 0.2]),
            target: Some(TypedTarget::Class {
                class_id: 0,
                label: Some("setosa".to_string()),
            }),
            output_type: OutputType::Class,
            coordinate_frame: None,
            timestamp: None,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn schema_version_is_pinned() {
        assert_eq!(iris_sample().schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn json_round_trip_preserves_sample() {
        let sample = iris_sample();
        let json = serde_json::to_string(&sample).expect("serialize");
        let restored: IRSample = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(sample, restored);
    }

    #[test]
    fn unlabeled_sample_has_no_target() {
        let mut sample = iris_sample();
        sample.target = None;
        let json = serde_json::to_string(&sample).expect("serialize");
        let restored: IRSample = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.target, None);
    }
}
