//! Continuous-record IR emission for stream-infer presentation.

use crate::adapters::time_series::config::{
    IncompleteWindowPolicy, TimeSeriesNormalize, TimeSeriesPackageConfig, UnknownLabelPolicy,
};
use crate::adapters::time_series::corpus::AnalogEpisode;
use crate::adapters::time_series::window::class_id_for_label;
use crate::contracts::common::{MetadataValue, Modality, OutputType, SampleId};
use crate::contracts::ir_sample::{HoldEnd, IRSample, Payload, SCHEMA_VERSION};
use crate::contracts::DatasetVersionId;
use crate::error::TrainerError;
use std::collections::BTreeMap;

/// Emits one IRSample covering an entire episode, with hold ends at annotation + post.
pub fn episode_stream_sample(
    episode: &AnalogEpisode,
    config: &TimeSeriesPackageConfig,
    dataset_version_id: &DatasetVersionId,
    source_uri: &str,
) -> Result<Option<IRSample>, TrainerError> {
    if config.window.stream_ids.len() != 1 {
        return Err(TrainerError::Config(
            "stream infer requires exactly one window stream_id".to_string(),
        ));
    }
    let stream_id = &config.window.stream_ids[0];
    let series = episode.streams.get(stream_id).ok_or_else(|| {
        TrainerError::Parse(format!(
            "episode '{}': window stream '{}' is not in the corpus",
            episode.episode_id, stream_id
        ))
    })?;
    if series.is_empty() {
        return Err(TrainerError::Parse(format!(
            "episode '{}': stream '{}' is empty",
            episode.episode_id, stream_id
        )));
    }

    let as_f64: Vec<f64> = series.iter().map(|v| f64::from(*v)).collect();
    let samples = prepare_episode_samples(&as_f64, episode, config.normalize)?;
    let mut hold_ends = Vec::new();
    let mut seen_ends = BTreeMap::new();

    for event in &episode.events {
        let class_id = match class_id_for_label(&event.label, &config.class_labels) {
            Some(id) => id,
            None => match config.unknown_label_policy {
                UnknownLabelPolicy::Error => {
                    return Err(TrainerError::Parse(format!(
                        "episode '{}': unknown annotation '{}' (not in class_labels)",
                        episode.episode_id, event.label
                    )));
                }
                UnknownLabelPolicy::Exclude => continue,
            },
        };
        if event.sample_index < config.window.pre_samples {
            match config.incomplete_window_policy {
                IncompleteWindowPolicy::Error => {
                    return Err(TrainerError::Parse(format!(
                        "episode '{}': event at sample {} does not fit the configured window",
                        episode.episode_id, event.sample_index
                    )));
                }
                IncompleteWindowPolicy::Exclude => continue,
            }
        }
        let end = match event.sample_index.checked_add(config.window.post_samples) {
            Some(end) => end,
            None => match config.incomplete_window_policy {
                IncompleteWindowPolicy::Error => {
                    return Err(TrainerError::Parse(format!(
                        "episode '{}': event at sample {} does not fit the configured window",
                        episode.episode_id, event.sample_index
                    )));
                }
                IncompleteWindowPolicy::Exclude => continue,
            },
        };
        if end >= series.len() as u64 {
            match config.incomplete_window_policy {
                IncompleteWindowPolicy::Error => {
                    return Err(TrainerError::Parse(format!(
                        "episode '{}': event at sample {} does not fit the configured window",
                        episode.episode_id, event.sample_index
                    )));
                }
                IncompleteWindowPolicy::Exclude => continue,
            }
        }
        if let Some(prior) = seen_ends.insert(end, class_id) {
            if prior != class_id {
                return Err(TrainerError::Parse(format!(
                    "episode '{}': two events share hold end {end} with different classes",
                    episode.episode_id
                )));
            }
        }
        hold_ends.push(HoldEnd {
            sample_index: end,
            class_id,
        });
    }

    if hold_ends.is_empty() {
        return Ok(None);
    }

    let mut metadata = BTreeMap::new();
    metadata.insert(
        "episode_id".to_string(),
        MetadataValue::Text(episode.episode_id.clone()),
    );

    Ok(Some(IRSample {
        schema_version: SCHEMA_VERSION,
        sample_id: SampleId(format!("{source_uri}#{}", episode.episode_id)),
        dataset_version_id: dataset_version_id.clone(),
        split: config.split.clone(),
        modality: Modality::TimeSeries,
        payload: Payload::TimeSeries {
            sample_rate_hz: episode.sample_rate_hz,
            channel_ids: config.window.stream_ids.clone(),
            samples,
            hold_ends: Some(hold_ends),
        },
        target: None,
        output_type: OutputType::Class,
        coordinate_frame: None,
        timestamp: None,
        metadata,
    }))
}

fn prepare_episode_samples(
    values: &[f64],
    episode: &AnalogEpisode,
    mode: TimeSeriesNormalize,
) -> Result<Vec<f64>, TrainerError> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(TrainerError::Parse(format!(
            "episode '{}': non-finite analog value",
            episode.episode_id
        )));
    }
    match mode {
        TimeSeriesNormalize::None => Ok(values.to_vec()),
        TimeSeriesNormalize::MinMaxPerWindow => Err(TrainerError::Config(
            "stream infer cannot use min_max_per_window".to_string(),
        )),
        TimeSeriesNormalize::MinMaxPerEpisode => {
            let min = values.iter().copied().fold(f64::INFINITY, f64::min);
            let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let span = max - min;
            if span == 0.0 {
                return Err(TrainerError::Parse(format!(
                    "episode '{}': constant episode cannot be min-max normalized",
                    episode.episode_id
                )));
            }
            Ok(values.iter().map(|v| (v - min) / span).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::time_series::config::{
        TimeSeriesNormalize, TimeSeriesPresentation, TimeSeriesSourceKind, TimeSeriesWindowConfig,
    };
    use crate::adapters::time_series::corpus::AnalogEvent;
    use crate::contracts::{Split, SplitId};
    use std::collections::BTreeMap;

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
            split: Split::Train,
            split_id: SplitId("train".to_string()),
            normalize: TimeSeriesNormalize::MinMaxPerEpisode,
            incomplete_window_policy: IncompleteWindowPolicy::Exclude,
            unknown_label_policy: UnknownLabelPolicy::Exclude,
            annotation_suffix: None,
            channel_map: None,
            presentation: TimeSeriesPresentation::StreamInfer,
            class_keep_percents: BTreeMap::new(),
        }
    }

    #[test]
    fn hold_end_is_annotation_plus_post() {
        let mut streams = BTreeMap::new();
        streams.insert("lead_0".to_string(), vec![0.0, 1.0, 2.0, 3.0, 4.0]);
        let episode = AnalogEpisode {
            episode_id: "r0".to_string(),
            sample_rate_hz: 360.0,
            streams,
            events: vec![AnalogEvent {
                sample_index: 2,
                label: "N".to_string(),
            }],
        };
        let sample = episode_stream_sample(
            &episode,
            &config(),
            &DatasetVersionId("v".to_string()),
            "/tmp/x",
        )
        .expect("emit")
        .expect("present");
        match sample.payload {
            Payload::TimeSeries {
                samples, hold_ends, ..
            } => {
                assert_eq!(samples.len(), 5);
                assert_eq!(
                    hold_ends.expect("holds")[0],
                    HoldEnd {
                        sample_index: 3,
                        class_id: 0
                    }
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn none_keeps_episode_physical_units() {
        let mut cfg = config();
        cfg.normalize = TimeSeriesNormalize::None;
        let mut streams = BTreeMap::new();
        streams.insert("lead_0".to_string(), vec![-1.5, 0.0, 2.25, 3.0, 4.0]);
        let episode = AnalogEpisode {
            episode_id: "r0".to_string(),
            sample_rate_hz: 360.0,
            streams,
            events: vec![AnalogEvent {
                sample_index: 2,
                label: "N".to_string(),
            }],
        };
        let sample =
            episode_stream_sample(&episode, &cfg, &DatasetVersionId("v".to_string()), "/tmp/x")
                .expect("emit")
                .expect("present");
        match sample.payload {
            Payload::TimeSeries { samples, .. } => {
                assert_eq!(samples, vec![-1.5, 0.0, 2.25, 3.0, 4.0]);
            }
            other => panic!("{other:?}"),
        }
    }
}
