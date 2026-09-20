//! Explicit configuration for annotated time-series ingest (ECG and other analog streams).
//!
//! Every field is required by the caller. The adapter never infers window size, class labels,
//! channel names, or edge policy.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::adapters::class_keep::validate_class_keep_percents;
use crate::contracts::common::{Split, SplitId};

/// On-disk source the adapter reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeSeriesSourceKind {
    /// Canonical analog package (`manifest.json` + `streams/` + `events.jsonl`).
    Package,
    /// WFDB record directory (`.hea` / `.dat` / annotation files).
    Wfdb,
}

/// How a window that does not fit inside an episode is handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncompleteWindowPolicy {
    /// Fail the ingest.
    Error,
    /// Skip that event. Must be chosen explicitly; it is not a silent default.
    Exclude,
}

/// How annotation symbols missing from `class_labels` are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownLabelPolicy {
    /// Fail the ingest.
    Error,
    /// Skip that event. Must be chosen explicitly.
    Exclude,
}

/// How analog samples are scaled before encoding.
///
/// `None` leaves physical units unchanged (the ECG trainer path). Min-max modes exist only
/// when a coder requires `[0, 1]` (population Z bins).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeSeriesNormalize {
    /// Pass physical-unit samples through. Non-finite values are an error.
    None,
    /// Independent min-max of each window. Constant windows are an error (no range).
    MinMaxPerWindow,
    /// Min-max of one entire episode. Used when a `[0, 1]` coder needs a full-record range.
    MinMaxPerEpisode,
}

/// How analog events are presented to the encoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TimeSeriesPresentation {
    /// One IRSample per event; resampled features sent as a single sensory frame.
    #[default]
    Snapshot,
    /// One IRSample per event; each raw window sample is one burst. Class held on Misc B.
    StreamTrain,
    /// One IRSample per episode; continuous A stream, B silent, score at each hold end.
    StreamInfer,
}

/// One WFDB header signal index mapped to a portable stream id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WfdbChannelMap {
    /// Zero-based signal index in the `.hea` file.
    pub source_index: usize,
    /// Portable stream id written into the corpus / IR metadata.
    pub stream_id: String,
}

/// Event-centered window cut out of an analog episode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSeriesWindowConfig {
    /// Samples before the event sample (inclusive span is `pre + 1 + post`).
    pub pre_samples: u64,
    /// Samples after the event sample.
    pub post_samples: u64,
    /// Streams included in the window, in encoder channel order.
    pub stream_ids: Vec<String>,
    /// Number of resampled points *per stream* fed to the encoder.
    pub feature_count: u32,
}

/// Adapter configuration for WFDB or a canonical analog package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSeriesPackageConfig {
    /// Logical dataset name used in asset / version ids.
    pub dataset_name: String,
    /// Which on-disk layout `source.uri` points at.
    pub source_kind: TimeSeriesSourceKind,
    /// Event-window cut used to emit classification `IRSample`s.
    pub window: TimeSeriesWindowConfig,
    /// Ordered class labels; position is `class_id`.
    pub class_labels: Vec<String>,
    /// Split role assigned to every emitted sample.
    pub split: Split,
    /// Split id assigned to every emitted sample.
    pub split_id: SplitId,
    /// Analog scaling applied before encoding.
    pub normalize: TimeSeriesNormalize,
    /// Policy for events too close to an episode edge.
    pub incomplete_window_policy: IncompleteWindowPolicy,
    /// Policy for annotation symbols not listed in `class_labels`.
    pub unknown_label_policy: UnknownLabelPolicy,
    /// WFDB annotation suffix (e.g. `atr`). Required when `source_kind = wfdb`.
    #[serde(default)]
    pub annotation_suffix: Option<String>,
    /// Exhaustive map of every WFDB header signal. Required when `source_kind = wfdb`.
    #[serde(default)]
    pub channel_map: Option<Vec<WfdbChannelMap>>,
    /// Snapshot vs sample-by-sample stream. Default is snapshot (existing ECG path).
    #[serde(default)]
    pub presentation: TimeSeriesPresentation,
    /// Per-class keep percent (0..=100). Empty keeps every eligible window.
    #[serde(default)]
    pub class_keep_percents: BTreeMap<String, u32>,
}

impl TimeSeriesPackageConfig {
    /// Returns blocking configuration errors. Never repairs values.
    pub fn validate(&self) -> Result<(), String> {
        if self.dataset_name.trim().is_empty() {
            return Err("dataset_name must be non-empty".to_string());
        }
        if self.class_labels.is_empty() {
            return Err("class_labels must list at least one label".to_string());
        }
        if self.window.stream_ids.is_empty() {
            return Err("window.stream_ids must list at least one stream".to_string());
        }
        if self.window.feature_count == 0 {
            return Err("window.feature_count must be greater than zero".to_string());
        }
        let raw_window = self
            .window
            .pre_samples
            .checked_add(1)
            .and_then(|n| n.checked_add(self.window.post_samples));
        match self.presentation {
            TimeSeriesPresentation::Snapshot => {}
            TimeSeriesPresentation::StreamTrain | TimeSeriesPresentation::StreamInfer => {
                if self.window.stream_ids.len() != 1 {
                    return Err(
                        "stream presentation requires exactly one window stream_id".to_string()
                    );
                }
            }
        }
        if self.presentation == TimeSeriesPresentation::StreamTrain {
            let raw_window = raw_window
                .ok_or_else(|| "stream_train window length overflowed u64".to_string())?;
            if u64::from(self.window.feature_count) != raw_window {
                return Err(format!(
                    "stream_train feature_count must equal the raw window length {raw_window}"
                ));
            }
        }
        if self.presentation == TimeSeriesPresentation::StreamInfer
            && !self.class_keep_percents.is_empty()
        {
            return Err(
                "class_keep_percents apply to event-window training, not stream infer".to_string(),
            );
        }
        validate_class_keep_percents(&self.class_keep_percents, &self.class_labels)?;
        if self.presentation == TimeSeriesPresentation::StreamInfer
            && self.normalize == TimeSeriesNormalize::MinMaxPerWindow
        {
            return Err(
                "stream_infer cannot use min_max_per_window (no per-event window); use none or min_max_per_episode"
                    .to_string(),
            );
        }
        match self.source_kind {
            TimeSeriesSourceKind::Wfdb => {
                let suffix = self.annotation_suffix.as_deref().unwrap_or("").trim();
                if suffix.is_empty() {
                    return Err(
                        "annotation_suffix is required when source_kind is wfdb".to_string()
                    );
                }
                let map = self.channel_map.as_ref().ok_or_else(|| {
                    "channel_map is required when source_kind is wfdb".to_string()
                })?;
                if map.is_empty() {
                    return Err("channel_map must list every WFDB signal".to_string());
                }
            }
            TimeSeriesSourceKind::Package => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_infer() -> TimeSeriesPackageConfig {
        TimeSeriesPackageConfig {
            dataset_name: "ecg".to_string(),
            source_kind: TimeSeriesSourceKind::Package,
            window: TimeSeriesWindowConfig {
                pre_samples: 1,
                post_samples: 1,
                stream_ids: vec!["lead_0".to_string()],
                feature_count: 3,
            },
            class_labels: vec!["N".to_string()],
            split: Split::Train,
            split_id: SplitId("train".to_string()),
            normalize: TimeSeriesNormalize::None,
            incomplete_window_policy: IncompleteWindowPolicy::Exclude,
            unknown_label_policy: UnknownLabelPolicy::Exclude,
            annotation_suffix: None,
            channel_map: None,
            presentation: TimeSeriesPresentation::StreamInfer,
            class_keep_percents: BTreeMap::new(),
        }
    }

    #[test]
    fn stream_infer_accepts_none() {
        assert!(base_infer().validate().is_ok());
    }

    #[test]
    fn stream_infer_rejects_per_window_minmax() {
        let mut cfg = base_infer();
        cfg.normalize = TimeSeriesNormalize::MinMaxPerWindow;
        let err = cfg.validate().expect_err("window minmax");
        assert!(err.contains("min_max_per_window"));
    }

    #[test]
    fn stream_infer_rejects_class_keep_percents() {
        let mut cfg = base_infer();
        cfg.class_keep_percents.insert("N".to_string(), 50);
        let err = cfg.validate().expect_err("keep on infer");
        assert!(err.contains("class_keep_percents"));
    }
}
