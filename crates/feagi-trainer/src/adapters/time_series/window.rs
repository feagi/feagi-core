//! Event-centered windowing and explicit analog normalization.

use crate::adapters::time_series::config::{
    IncompleteWindowPolicy, TimeSeriesNormalize, TimeSeriesPackageConfig, UnknownLabelPolicy,
};
use crate::adapters::time_series::corpus::{AnalogEpisode, AnalogEvent};
use crate::contracts::common::{MetadataValue, Modality, OutputType, SampleId};
use crate::contracts::ir_sample::{IRSample, Payload, TypedTarget, SCHEMA_VERSION};
use crate::contracts::DatasetVersionId;
use crate::error::TrainerError;
use std::collections::BTreeMap;

/// Cuts one event-centered sample from an episode.
pub fn window_event(
    episode: &AnalogEpisode,
    event: &AnalogEvent,
    event_ordinal: usize,
    config: &TimeSeriesPackageConfig,
    dataset_version_id: &DatasetVersionId,
    source_uri: &str,
) -> Result<Option<IRSample>, TrainerError> {
    let class_id = match class_id_for_label(&event.label, &config.class_labels) {
        Some(id) => id,
        None => {
            return match config.unknown_label_policy {
                UnknownLabelPolicy::Error => Err(TrainerError::Parse(format!(
                    "episode '{}': unknown annotation '{}' (not in class_labels)",
                    episode.episode_id, event.label
                ))),
                UnknownLabelPolicy::Exclude => Ok(None),
            };
        }
    };

    let sample_count = episode.sample_count().map_err(|e| TrainerError::Parse(e))?;
    let start = match event.sample_index.checked_sub(config.window.pre_samples) {
        Some(start) => start,
        None => {
            return match config.incomplete_window_policy {
                IncompleteWindowPolicy::Error => Err(edge_error(episode, event)),
                IncompleteWindowPolicy::Exclude => Ok(None),
            };
        }
    };
    let end_inclusive = match event.sample_index.checked_add(config.window.post_samples) {
        Some(end) => end,
        None => {
            return match config.incomplete_window_policy {
                IncompleteWindowPolicy::Error => Err(edge_error(episode, event)),
                IncompleteWindowPolicy::Exclude => Ok(None),
            };
        }
    };
    if end_inclusive >= sample_count {
        return match config.incomplete_window_policy {
            IncompleteWindowPolicy::Error => Err(edge_error(episode, event)),
            IncompleteWindowPolicy::Exclude => Ok(None),
        };
    }
    if start > event.sample_index {
        return match config.incomplete_window_policy {
            IncompleteWindowPolicy::Error => Err(edge_error(episode, event)),
            IncompleteWindowPolicy::Exclude => Ok(None),
        };
    }

    let mut channel_series: Vec<Vec<f64>> = Vec::with_capacity(config.window.stream_ids.len());
    for stream_id in &config.window.stream_ids {
        let series = episode.streams.get(stream_id).ok_or_else(|| {
            TrainerError::Parse(format!(
                "episode '{}': window stream '{}' is not in the corpus",
                episode.episode_id, stream_id
            ))
        })?;
        let slice = &series[start as usize..=end_inclusive as usize];
        let resampled = resample_linear(slice, config.window.feature_count as usize)?;
        let normalized = normalize_window(&resampled, config.normalize, episode, event)?;
        channel_series.push(normalized);
    }

    let time_points = config.window.feature_count as usize;
    let mut samples = Vec::with_capacity(time_points * channel_series.len());
    for t in 0..time_points {
        for channel in &channel_series {
            samples.push(channel[t]);
        }
    }

    let mut metadata = BTreeMap::new();
    metadata.insert(
        "episode_id".to_string(),
        MetadataValue::Text(episode.episode_id.clone()),
    );
    metadata.insert(
        "event_sample_index".to_string(),
        MetadataValue::Int(event.sample_index as i64),
    );

    Ok(Some(IRSample {
        schema_version: SCHEMA_VERSION,
        sample_id: SampleId(format!(
            "{source_uri}#{}#{event_ordinal}",
            episode.episode_id
        )),
        dataset_version_id: dataset_version_id.clone(),
        split: config.split.clone(),
        modality: Modality::TimeSeries,
        payload: Payload::TimeSeries {
            sample_rate_hz: episode.sample_rate_hz,
            channel_ids: config.window.stream_ids.clone(),
            samples,
            hold_ends: None,
        },
        target: Some(TypedTarget::Class {
            class_id,
            label: Some(event.label.clone()),
        }),
        output_type: OutputType::Class,
        coordinate_frame: None,
        timestamp: None,
        metadata,
    }))
}

pub(crate) fn class_id_for_label(label: &str, class_labels: &[String]) -> Option<u32> {
    class_labels
        .iter()
        .position(|item| item == label)
        .map(|idx| idx as u32)
}

fn edge_error(episode: &AnalogEpisode, event: &AnalogEvent) -> TrainerError {
    TrainerError::Parse(format!(
        "episode '{}': event at sample {} does not fit the configured window",
        episode.episode_id, event.sample_index
    ))
}

fn resample_linear(values: &[f32], feature_count: usize) -> Result<Vec<f64>, TrainerError> {
    if values.is_empty() {
        return Err(TrainerError::Parse(
            "cannot resample an empty analog window".to_string(),
        ));
    }
    if feature_count == values.len() {
        return Ok(values.iter().map(|v| f64::from(*v)).collect());
    }
    if feature_count == 1 {
        return Ok(vec![f64::from(values[0])]);
    }
    let last = values.len() - 1;
    let mut out = Vec::with_capacity(feature_count);
    for i in 0..feature_count {
        let pos = (i as f64) * (last as f64) / ((feature_count - 1) as f64);
        let left = pos.floor() as usize;
        let right = pos.ceil() as usize;
        let frac = pos - left as f64;
        let interpolated = f64::from(values[left]) * (1.0 - frac) + f64::from(values[right]) * frac;
        out.push(interpolated);
    }
    Ok(out)
}

fn require_finite(
    values: &[f64],
    episode: &AnalogEpisode,
    event: &AnalogEvent,
) -> Result<(), TrainerError> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(TrainerError::Parse(format!(
            "episode '{}': non-finite analog value at sample {}",
            episode.episode_id, event.sample_index
        )));
    }
    Ok(())
}

fn normalize_window(
    values: &[f64],
    mode: TimeSeriesNormalize,
    episode: &AnalogEpisode,
    event: &AnalogEvent,
) -> Result<Vec<f64>, TrainerError> {
    require_finite(values, episode, event)?;
    match mode {
        TimeSeriesNormalize::None => Ok(values.to_vec()),
        TimeSeriesNormalize::MinMaxPerWindow | TimeSeriesNormalize::MinMaxPerEpisode => {
            let min = values.iter().copied().fold(f64::INFINITY, f64::min);
            let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let span = max - min;
            if span == 0.0 {
                return Err(TrainerError::Parse(format!(
                    "episode '{}': constant window at sample {} cannot be min-max normalized",
                    episode.episode_id, event.sample_index
                )));
            }
            Ok(values.iter().map(|v| (v - min) / span).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::time_series::config::{TimeSeriesSourceKind, TimeSeriesWindowConfig};
    use crate::contracts::{DatasetVersionId, SplitId};

    fn config() -> TimeSeriesPackageConfig {
        TimeSeriesPackageConfig {
            dataset_name: "ecg".to_string(),
            source_kind: TimeSeriesSourceKind::Package,
            window: TimeSeriesWindowConfig {
                pre_samples: 1,
                post_samples: 1,
                stream_ids: vec!["lead_0".to_string()],
                feature_count: 3,
            },
            class_labels: vec!["N".to_string(), "V".to_string()],
            split: crate::contracts::Split::Train,
            split_id: SplitId("train".to_string()),
            normalize: TimeSeriesNormalize::MinMaxPerWindow,
            incomplete_window_policy: IncompleteWindowPolicy::Error,
            unknown_label_policy: UnknownLabelPolicy::Error,
            annotation_suffix: None,
            channel_map: None,
            presentation: crate::adapters::time_series::config::TimeSeriesPresentation::Snapshot,
        }
    }

    fn episode() -> AnalogEpisode {
        let mut streams = BTreeMap::new();
        streams.insert("lead_0".to_string(), vec![0.0, 1.0, 2.0, 3.0, 4.0]);
        AnalogEpisode {
            episode_id: "rec".to_string(),
            sample_rate_hz: 360.0,
            streams,
            events: vec![AnalogEvent {
                sample_index: 2,
                label: "N".to_string(),
            }],
        }
    }

    #[test]
    fn window_emits_normalized_class_sample() {
        let cfg = config();
        let ep = episode();
        let sample = window_event(
            &ep,
            &ep.events[0],
            0,
            &cfg,
            &DatasetVersionId("ecg@1".to_string()),
            "file://pkg",
        )
        .expect("window")
        .expect("kept");
        assert_eq!(sample.modality, Modality::TimeSeries);
        match sample.payload {
            Payload::TimeSeries { samples, .. } => {
                assert_eq!(samples, vec![0.0, 0.5, 1.0]);
            }
            other => panic!("unexpected payload {other:?}"),
        }
        assert_eq!(
            sample.target,
            Some(TypedTarget::Class {
                class_id: 0,
                label: Some("N".to_string()),
            })
        );
    }

    #[test]
    fn early_event_exclude_drops_window() {
        let mut cfg = config();
        cfg.window.pre_samples = 90;
        cfg.window.post_samples = 180;
        cfg.incomplete_window_policy = IncompleteWindowPolicy::Exclude;
        let mut ep = episode();
        ep.events[0].sample_index = 77;
        let sample = window_event(
            &ep,
            &ep.events[0],
            0,
            &cfg,
            &DatasetVersionId("ecg@1".to_string()),
            "file://pkg",
        )
        .expect("window");
        assert!(sample.is_none());
    }

    #[test]
    fn early_event_error_policy_is_parse_error() {
        let mut cfg = config();
        cfg.window.pre_samples = 90;
        cfg.incomplete_window_policy = IncompleteWindowPolicy::Error;
        let mut ep = episode();
        ep.events[0].sample_index = 77;
        let err = window_event(
            &ep,
            &ep.events[0],
            0,
            &cfg,
            &DatasetVersionId("ecg@1".to_string()),
            "file://pkg",
        )
        .unwrap_err();
        assert!(err.to_string().contains("sample 77"));
    }

    #[test]
    fn unknown_label_exclude_drops_event() {
        let mut cfg = config();
        cfg.unknown_label_policy = UnknownLabelPolicy::Exclude;
        let mut ep = episode();
        ep.events[0].label = "Q".to_string();
        let sample = window_event(
            &ep,
            &ep.events[0],
            0,
            &cfg,
            &DatasetVersionId("ecg@1".to_string()),
            "file://pkg",
        )
        .expect("window");
        assert!(sample.is_none());
    }

    #[test]
    fn constant_window_is_an_error() {
        let cfg = config();
        let mut ep = episode();
        ep.streams
            .insert("lead_0".to_string(), vec![1.0, 1.0, 1.0, 1.0, 1.0]);
        let err = window_event(
            &ep,
            &ep.events[0],
            0,
            &cfg,
            &DatasetVersionId("ecg@1".to_string()),
            "file://pkg",
        )
        .unwrap_err();
        assert!(matches!(err, TrainerError::Parse(_)));
    }

    #[test]
    fn none_preserves_physical_units() {
        let mut cfg = config();
        cfg.normalize = TimeSeriesNormalize::None;
        let ep = episode();
        let sample = window_event(
            &ep,
            &ep.events[0],
            0,
            &cfg,
            &DatasetVersionId("ecg@1".to_string()),
            "file://pkg",
        )
        .expect("window")
        .expect("kept");
        match sample.payload {
            Payload::TimeSeries { samples, .. } => {
                assert_eq!(samples, vec![1.0, 2.0, 3.0]);
            }
            other => panic!("unexpected payload {other:?}"),
        }
    }
}
