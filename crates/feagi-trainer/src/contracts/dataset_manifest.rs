//! `DatasetManifest` v1 — versioned, immutable metadata describing an imported dataset.
//!
//! The manifest is the dataset registry record (design Section 5.1). It reserves the
//! hosted-dataset-asset identifiers (`dataset_asset_id` + `dataset_version` + content hash)
//! so the same ids later resolve to a hosted asset with no contract change (ADR-012).

use serde::{Deserialize, Serialize};

use super::common::{
    ContentHash, DatasetAssetId, DatasetVersionId, MetadataMap, Modality, OutputType, Split,
    SplitId,
};

/// Wire/format version of the `DatasetManifest` contract.
pub const SCHEMA_VERSION: u32 = 1;

/// Definition of a single split within a dataset version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitDescriptor {
    /// Stable id for this split.
    pub id: SplitId,
    /// The split role (train/val/test/custom).
    pub split: Split,
    /// Number of samples assigned to this split.
    pub sample_count: u64,
}

/// Versioned, immutable metadata about an imported dataset (design Section 5.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatasetManifest {
    /// Wire/format version; must equal [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Immutable dataset version identity.
    pub dataset_version_id: DatasetVersionId,
    /// Stable asset id (resolves locally now, hosted asset later — ADR-012).
    pub dataset_asset_id: DatasetAssetId,
    /// Human-facing dataset version string (e.g. `1.0.0`).
    pub dataset_version: String,
    /// Source location the dataset was imported from.
    pub source_uri: String,
    /// Content hash binding this manifest to exact dataset bytes.
    pub content_hash: ContentHash,
    /// Hash of the dataset schema/structure (column layout, label space, etc.).
    pub schema_fingerprint: ContentHash,
    /// Declared modality.
    pub modality: Modality,
    /// Declared structured-output task type for the dataset.
    pub output_type: OutputType,
    /// Split definitions for this dataset version.
    pub splits: Vec<SplitDescriptor>,
    /// Free-form, deterministically ordered metadata.
    pub metadata: MetadataMap,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn iris_manifest() -> DatasetManifest {
        DatasetManifest {
            schema_version: SCHEMA_VERSION,
            dataset_version_id: DatasetVersionId("iris@1".to_string()),
            dataset_asset_id: DatasetAssetId("local:iris".to_string()),
            dataset_version: "1.0.0".to_string(),
            source_uri: "file:///datasets/iris.csv".to_string(),
            content_hash: ContentHash("sha256:abc".to_string()),
            schema_fingerprint: ContentHash("sha256:schema".to_string()),
            modality: Modality::Tabular,
            output_type: OutputType::Class,
            splits: vec![
                SplitDescriptor {
                    id: SplitId("train".to_string()),
                    split: Split::Train,
                    sample_count: 120,
                },
                SplitDescriptor {
                    id: SplitId("test".to_string()),
                    split: Split::Test,
                    sample_count: 30,
                },
            ],
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn schema_version_is_pinned() {
        assert_eq!(iris_manifest().schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn json_round_trip_preserves_manifest() {
        let manifest = iris_manifest();
        let json = serde_json::to_string(&manifest).expect("serialize");
        let restored: DatasetManifest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(manifest, restored);
    }
}
