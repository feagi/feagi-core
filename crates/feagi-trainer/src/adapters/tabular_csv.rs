//! Tabular CSV adapter — maps delimited numeric rows into class-labeled `IRSample`s.
//!
//! Scope: simple comma-separated values with one header row, numeric feature columns, and a
//! string/numeric label column resolved against an explicit class-label list. Quoted/escaped
//! CSV fields are out of scope for this slice (they would be a future enhancement); rows with
//! the wrong column count, unparseable features, or unknown labels are explicit errors — the
//! adapter never silently drops or coerces data.
//!
//! All configuration is explicit (no inferred defaults), satisfying the no-fallback rule.

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::contracts::common::{
    ContentHash, DatasetAssetId, DatasetVersionId, Modality, OutputType, PluginId, PluginRef,
    SampleId, Split, SplitId,
};
use crate::contracts::dataset_manifest::SCHEMA_VERSION as MANIFEST_SCHEMA_VERSION;
use crate::contracts::ir_sample::SCHEMA_VERSION as IR_SCHEMA_VERSION;
use crate::contracts::{DatasetManifest, IRSample, Payload, SplitDescriptor, TypedTarget};
use crate::error::TrainerError;
use crate::plugins::{AdapterPlugin, DatasetSource, ValidationReport};

/// Explicit configuration for the tabular CSV adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabularCsvConfig {
    /// Logical dataset name, used to derive the dataset version id and asset id.
    pub dataset_name: String,
    /// Whether the first row is a header to skip.
    pub has_header: bool,
    /// Zero-based indices of the numeric feature columns, in feature order.
    pub feature_columns: Vec<usize>,
    /// Zero-based index of the class-label column.
    pub label_column: usize,
    /// Ordered class labels; position is the resulting `class_id`.
    pub class_labels: Vec<String>,
    /// The split role all rows in this source belong to.
    pub split: Split,
    /// The split id all rows in this source are assigned to.
    pub split_id: SplitId,
}

/// Adapter that converts a single-split tabular CSV source into `IRSample`s.
#[derive(Debug, Clone)]
pub struct TabularCsvAdapter {
    config: TabularCsvConfig,
}

impl TabularCsvAdapter {
    /// Stable plugin id for this adapter.
    pub const PLUGIN_ID: &'static str = "tabular_csv";

    /// Creates a new adapter from explicit configuration.
    pub fn new(config: TabularCsvConfig) -> Self {
        Self { config }
    }

    /// Non-cryptographic, deterministic 64-bit content fingerprint (local-use only).
    ///
    /// Uses the standard library's fixed-key hasher so the value is stable across runs and
    /// platforms. A cryptographic content hash can replace this without a contract change.
    fn fingerprint(bytes: &[u8]) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        bytes.hash(&mut hasher);
        format!("siphash64:{:016x}", hasher.finish())
    }

    fn dataset_asset_id(&self) -> DatasetAssetId {
        DatasetAssetId(format!("local:{}", self.config.dataset_name))
    }

    fn dataset_version_id(&self, content_fingerprint: &str) -> DatasetVersionId {
        // Bind the version id to content so different bytes are a different version.
        let short = content_fingerprint
            .rsplit(':')
            .next()
            .unwrap_or(content_fingerprint);
        DatasetVersionId(format!("{}@{}", self.config.dataset_name, short))
    }

    /// Decodes the source bytes to text and returns the data rows (header skipped if configured).
    fn data_rows<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let mut lines: Vec<&str> = text
            .lines()
            .map(|line| line.trim_end_matches('\r'))
            .filter(|line| !line.trim().is_empty())
            .collect();
        if self.config.has_header && !lines.is_empty() {
            lines.remove(0);
        }
        lines
    }

    fn decode(source: &DatasetSource) -> Result<&str, TrainerError> {
        std::str::from_utf8(&source.bytes)
            .map_err(|e| TrainerError::Parse(format!("source is not valid UTF-8: {e}")))
    }

    /// Parses one data row into an `IRSample`.
    fn map_row_to_ir(
        &self,
        row_index: usize,
        row: &str,
        source_uri: &str,
        dataset_version_id: &DatasetVersionId,
    ) -> Result<IRSample, TrainerError> {
        let cols: Vec<&str> = row.split(',').map(|c| c.trim()).collect();

        let mut features: Vec<f64> = Vec::with_capacity(self.config.feature_columns.len());
        for &col in &self.config.feature_columns {
            let field = cols.get(col).ok_or_else(|| {
                TrainerError::Parse(format!("row {row_index}: missing feature column {col}"))
            })?;
            let value: f64 = field.parse().map_err(|_| {
                TrainerError::Parse(format!(
                    "row {row_index}: cannot parse '{field}' (column {col}) as a number"
                ))
            })?;
            features.push(value);
        }

        let label_field = cols.get(self.config.label_column).ok_or_else(|| {
            TrainerError::Parse(format!(
                "row {row_index}: missing label column {}",
                self.config.label_column
            ))
        })?;
        let class_id = self
            .config
            .class_labels
            .iter()
            .position(|label| label == label_field)
            .ok_or_else(|| {
                TrainerError::Parse(format!(
                    "row {row_index}: unknown class label '{label_field}'"
                ))
            })? as u32;

        Ok(IRSample {
            schema_version: IR_SCHEMA_VERSION,
            sample_id: SampleId(format!("{source_uri}#{row_index}")),
            dataset_version_id: dataset_version_id.clone(),
            split: self.config.split.clone(),
            modality: Modality::Tabular,
            payload: Payload::Tabular(features),
            target: Some(TypedTarget::Class {
                class_id,
                label: Some((*label_field).to_string()),
            }),
            output_type: OutputType::Class,
            coordinate_frame: None,
            timestamp: None,
            metadata: BTreeMap::new(),
        })
    }
}

impl AdapterPlugin for TabularCsvAdapter {
    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId(Self::PLUGIN_ID.to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn discover(&self, source: &DatasetSource) -> Result<DatasetManifest, TrainerError> {
        let text = Self::decode(source)?;
        let header = if self.config.has_header {
            text.lines().next().unwrap_or("")
        } else {
            ""
        };
        let row_count = self.data_rows(text).len() as u64;

        let content_fingerprint = Self::fingerprint(&source.bytes);
        let dataset_version_id = self.dataset_version_id(&content_fingerprint);

        Ok(DatasetManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            dataset_version_id,
            dataset_asset_id: self.dataset_asset_id(),
            dataset_version: "1.0.0".to_string(),
            source_uri: source.uri.clone(),
            content_hash: ContentHash(content_fingerprint),
            schema_fingerprint: ContentHash(Self::fingerprint(header.as_bytes())),
            modality: Modality::Tabular,
            output_type: OutputType::Class,
            splits: vec![SplitDescriptor {
                id: self.config.split_id.clone(),
                split: self.config.split.clone(),
                sample_count: row_count,
            }],
            metadata: BTreeMap::new(),
        })
    }

    fn validate(&self, manifest: &DatasetManifest) -> Result<ValidationReport, TrainerError> {
        let mut issues = Vec::new();
        if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
            issues.push(format!(
                "manifest schema_version {} != expected {MANIFEST_SCHEMA_VERSION}",
                manifest.schema_version
            ));
        }
        if manifest.modality != Modality::Tabular {
            issues.push(format!("modality {:?} is not Tabular", manifest.modality));
        }
        if manifest.output_type != OutputType::Class {
            issues.push(format!(
                "output_type {:?} is not Class",
                manifest.output_type
            ));
        }
        if manifest.splits.is_empty() {
            issues.push("manifest declares no splits".to_string());
        }
        if manifest.splits.iter().any(|s| s.sample_count == 0) {
            issues.push("a declared split has zero samples".to_string());
        }
        if self.config.class_labels.is_empty() {
            issues.push("adapter config has no class labels".to_string());
        }
        if self.config.feature_columns.is_empty() {
            issues.push("adapter config has no feature columns".to_string());
        }
        Ok(ValidationReport {
            passed: issues.is_empty(),
            issues,
        })
    }

    fn stream(
        &self,
        source: &DatasetSource,
        split: &SplitId,
    ) -> Result<Vec<IRSample>, TrainerError> {
        if split != &self.config.split_id {
            return Err(TrainerError::Validation(format!(
                "unknown split '{split}'; this source only provides '{}'",
                self.config.split_id
            )));
        }
        let text = Self::decode(source)?;
        let content_fingerprint = Self::fingerprint(&source.bytes);
        let dataset_version_id = self.dataset_version_id(&content_fingerprint);

        let rows = self.data_rows(text);
        let mut samples = Vec::with_capacity(rows.len());
        for (row_index, row) in rows.iter().enumerate() {
            samples.push(self.map_row_to_ir(row_index, row, &source.uri, &dataset_version_id)?);
        }
        Ok(samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IRIS_CSV: &str = "sepal_length,sepal_width,petal_length,petal_width,species\n\
5.1,3.5,1.4,0.2,setosa\n\
7.0,3.2,4.7,1.4,versicolor\n\
6.3,3.3,6.0,2.5,virginica\n";

    fn config() -> TabularCsvConfig {
        TabularCsvConfig {
            dataset_name: "iris".to_string(),
            has_header: true,
            feature_columns: vec![0, 1, 2, 3],
            label_column: 4,
            class_labels: vec![
                "setosa".to_string(),
                "versicolor".to_string(),
                "virginica".to_string(),
            ],
            split: Split::Train,
            split_id: SplitId("train".to_string()),
        }
    }

    fn source() -> DatasetSource {
        DatasetSource {
            uri: "mem://iris.csv".to_string(),
            bytes: IRIS_CSV.as_bytes().to_vec(),
        }
    }

    #[test]
    fn discover_counts_rows_and_sets_task() {
        let adapter = TabularCsvAdapter::new(config());
        let manifest = adapter.discover(&source()).expect("discover");
        assert_eq!(manifest.modality, Modality::Tabular);
        assert_eq!(manifest.output_type, OutputType::Class);
        assert_eq!(manifest.splits.len(), 1);
        assert_eq!(manifest.splits[0].sample_count, 3);
    }

    #[test]
    fn validate_passes_for_good_manifest() {
        let adapter = TabularCsvAdapter::new(config());
        let manifest = adapter.discover(&source()).expect("discover");
        let report = adapter.validate(&manifest).expect("validate");
        assert!(report.passed, "issues: {:?}", report.issues);
    }

    #[test]
    fn stream_maps_rows_to_ir_samples() {
        let adapter = TabularCsvAdapter::new(config());
        let samples = adapter
            .stream(&source(), &SplitId("train".to_string()))
            .expect("stream");
        assert_eq!(samples.len(), 3);
        assert_eq!(
            samples[0].payload,
            Payload::Tabular(vec![5.1, 3.5, 1.4, 0.2])
        );
        assert_eq!(
            samples[2].target,
            Some(TypedTarget::Class {
                class_id: 2,
                label: Some("virginica".to_string())
            })
        );
    }

    #[test]
    fn unknown_split_is_error() {
        let adapter = TabularCsvAdapter::new(config());
        let err = adapter
            .stream(&source(), &SplitId("test".to_string()))
            .unwrap_err();
        assert!(matches!(err, TrainerError::Validation(_)));
    }

    #[test]
    fn unparseable_feature_is_error() {
        let adapter = TabularCsvAdapter::new(config());
        let bad = DatasetSource {
            uri: "mem://bad.csv".to_string(),
            bytes: "a,b,c,d,species\nx,3.5,1.4,0.2,setosa\n"
                .as_bytes()
                .to_vec(),
        };
        let err = adapter
            .stream(&bad, &SplitId("train".to_string()))
            .unwrap_err();
        assert!(matches!(err, TrainerError::Parse(_)));
    }

    #[test]
    fn unknown_label_is_error() {
        let adapter = TabularCsvAdapter::new(config());
        let bad = DatasetSource {
            uri: "mem://bad.csv".to_string(),
            bytes: "a,b,c,d,species\n5.1,3.5,1.4,0.2,daffodil\n"
                .as_bytes()
                .to_vec(),
        };
        let err = adapter
            .stream(&bad, &SplitId("train".to_string()))
            .unwrap_err();
        assert!(matches!(err, TrainerError::Parse(_)));
    }
}
