//! Raw event-window preview for an ingested analog corpus.
//!
//! Returns physical-unit samples around the first events. This is a view of the
//! source, not the normalized encoder features.

use serde::{Deserialize, Serialize};

use crate::adapters::time_series::config::{
    IncompleteWindowPolicy, TimeSeriesPackageConfig, UnknownLabelPolicy,
};
use crate::adapters::time_series::corpus::AnalogEpisode;
use crate::error::TrainerError;

/// How many eligible windows carry a given class label.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewClassCount {
    pub label: String,
    pub count: u64,
}

/// One beat/event window of raw analog samples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeSeriesPreviewFrame {
    /// WFDB record name or package episode id for this window.
    pub record_id: String,
    /// Annotation symbol at the event sample.
    pub label: String,
    /// Sample index of the annotated event.
    pub event_sample_index: u64,
    /// Stream the `values` were taken from.
    pub stream_id: String,
    /// Physical-unit samples from `event - pre` through `event + post`.
    pub values: Vec<f64>,
}

/// A page of raw event windows plus the corpus size for paging.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeSeriesPreview {
    /// WFDB record name or package episode id of the first frame on this page.
    pub record_id: String,
    /// Shared sampling rate of the first frame's record.
    pub sample_rate_hz: f64,
    /// Number of samples in the first frame's record.
    pub sample_count: u64,
    /// Portable stream ids present on the first frame's record.
    pub stream_ids: Vec<String>,
    /// Zero-based index of the first returned eligible window.
    pub frame_offset: u64,
    /// Number of eligible windows in the corpus (all records).
    pub total_frames: u64,
    /// Number of records/episodes loaded from the folder.
    pub record_count: u64,
    /// Eligible-window counts per configured class, then any extra labels.
    pub class_counts: Vec<PreviewClassCount>,
    /// Minimum physical sample on the configured streams, across every loaded record.
    pub value_min: f64,
    /// Maximum physical sample on the configured streams, across every loaded record.
    pub value_max: f64,
    /// Event windows for this page, in annotation order.
    pub frames: Vec<TimeSeriesPreviewFrame>,
}

/// Builds a raw preview from already-loaded episodes.
pub fn preview_episodes(
    episodes: &[AnalogEpisode],
    config: &TimeSeriesPackageConfig,
    frame_offset: usize,
    frame_count: usize,
) -> Result<TimeSeriesPreview, TrainerError> {
    if frame_count == 0 {
        return Err(TrainerError::Config(
            "preview frame_count must be greater than zero".to_string(),
        ));
    }
    if episodes.is_empty() {
        return Err(TrainerError::Parse(
            "time-series preview found no records in the dataset".to_string(),
        ));
    }
    let stream_id = config.window.stream_ids.first().ok_or_else(|| {
        TrainerError::Config("window.stream_ids must list at least one stream".to_string())
    })?;

    let mut eligible = Vec::new();
    for episode in episodes {
        collect_eligible_frames(episode, config, stream_id, &mut eligible)?;
    }
    let total_frames = eligible.len();
    if total_frames == 0 {
        return Err(TrainerError::Parse(
            "time-series preview produced no frames (check class_labels and window policies)"
                .to_string(),
        ));
    }
    if frame_offset >= total_frames {
        return Err(TrainerError::Config(format!(
            "preview frame_offset {frame_offset} is past the {total_frames} eligible frames"
        )));
    }
    let (value_min, value_max) = streamed_value_range(episodes, &config.window.stream_ids)?;
    let class_counts = class_counts_from_frames(&eligible, &config.class_labels);
    let frames: Vec<TimeSeriesPreviewFrame> = eligible
        .into_iter()
        .skip(frame_offset)
        .take(frame_count)
        .collect();
    let first = frames.first().ok_or_else(|| {
        TrainerError::Parse("preview page is empty after applying frame_offset".to_string())
    })?;
    let page_episode = episodes
        .iter()
        .find(|episode| episode.episode_id == first.record_id)
        .ok_or_else(|| {
            TrainerError::Parse(format!(
                "preview page record '{}' is missing from the corpus",
                first.record_id
            ))
        })?;
    Ok(TimeSeriesPreview {
        record_id: first.record_id.clone(),
        sample_rate_hz: page_episode.sample_rate_hz,
        sample_count: page_episode.sample_count().map_err(TrainerError::Parse)?,
        stream_ids: page_episode.streams.keys().cloned().collect(),
        frame_offset: frame_offset as u64,
        total_frames: total_frames as u64,
        record_count: episodes.len() as u64,
        class_counts,
        value_min,
        value_max,
        frames,
    })
}

/// Min and max of every physical sample on `stream_ids`. This is the recording range, not a window.
pub(crate) fn streamed_value_range(
    episodes: &[AnalogEpisode],
    stream_ids: &[String],
) -> Result<(f64, f64), TrainerError> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut saw = false;
    for episode in episodes {
        for stream_id in stream_ids {
            let series = episode.streams.get(stream_id).ok_or_else(|| {
                TrainerError::Parse(format!(
                    "episode '{}': stream '{}' is not in the corpus",
                    episode.episode_id, stream_id
                ))
            })?;
            for value in series {
                let sample = f64::from(*value);
                if !sample.is_finite() {
                    return Err(TrainerError::Parse(format!(
                        "episode '{}': non-finite sample on stream '{}'",
                        episode.episode_id, stream_id
                    )));
                }
                saw = true;
                min = min.min(sample);
                max = max.max(sample);
            }
        }
    }
    if !saw {
        return Err(TrainerError::Parse(
            "time-series preview found no samples on the configured streams".to_string(),
        ));
    }
    Ok((min, max))
}

fn class_counts_from_frames(
    frames: &[TimeSeriesPreviewFrame],
    class_labels: &[String],
) -> Vec<PreviewClassCount> {
    let mut tallies = std::collections::BTreeMap::<String, u64>::new();
    for frame in frames {
        *tallies.entry(frame.label.clone()).or_insert(0) += 1;
    }
    let mut counts = Vec::new();
    for label in class_labels {
        if let Some(count) = tallies.remove(label) {
            counts.push(PreviewClassCount {
                label: label.clone(),
                count,
            });
        }
    }
    for (label, count) in tallies {
        counts.push(PreviewClassCount { label, count });
    }
    counts
}

fn collect_eligible_frames(
    episode: &AnalogEpisode,
    config: &TimeSeriesPackageConfig,
    stream_id: &str,
    frames: &mut Vec<TimeSeriesPreviewFrame>,
) -> Result<(), TrainerError> {
    let series = episode.streams.get(stream_id).ok_or_else(|| {
        TrainerError::Parse(format!(
            "episode '{}': preview stream '{}' is not in the corpus",
            episode.episode_id, stream_id
        ))
    })?;
    let sample_count = episode.sample_count().map_err(TrainerError::Parse)?;
    for event in &episode.events {
        if !config
            .class_labels
            .iter()
            .any(|label| label == &event.label)
        {
            match config.unknown_label_policy {
                UnknownLabelPolicy::Error => {
                    return Err(TrainerError::Parse(format!(
                        "episode '{}': unknown annotation '{}' (not in class_labels)",
                        episode.episode_id, event.label
                    )));
                }
                UnknownLabelPolicy::Exclude => continue,
            }
        }
        let start = match event.sample_index.checked_sub(config.window.pre_samples) {
            Some(start) => start,
            None => match config.incomplete_window_policy {
                IncompleteWindowPolicy::Error => {
                    return Err(edge_error(episode, event.sample_index));
                }
                IncompleteWindowPolicy::Exclude => continue,
            },
        };
        let end_inclusive = match event.sample_index.checked_add(config.window.post_samples) {
            Some(end) => end,
            None => match config.incomplete_window_policy {
                IncompleteWindowPolicy::Error => {
                    return Err(edge_error(episode, event.sample_index));
                }
                IncompleteWindowPolicy::Exclude => continue,
            },
        };
        if end_inclusive >= sample_count {
            match config.incomplete_window_policy {
                IncompleteWindowPolicy::Error => {
                    return Err(edge_error(episode, event.sample_index));
                }
                IncompleteWindowPolicy::Exclude => continue,
            }
        }
        let values = series[start as usize..=end_inclusive as usize]
            .iter()
            .map(|value| f64::from(*value))
            .collect();
        frames.push(TimeSeriesPreviewFrame {
            record_id: episode.episode_id.clone(),
            label: event.label.clone(),
            event_sample_index: event.sample_index,
            stream_id: stream_id.to_string(),
            values,
        });
    }
    Ok(())
}

fn edge_error(episode: &AnalogEpisode, sample_index: u64) -> TrainerError {
    TrainerError::Parse(format!(
        "episode '{}': event at sample {sample_index} does not fit the configured window",
        episode.episode_id
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::time_series::config::{
        TimeSeriesNormalize, TimeSeriesSourceKind, TimeSeriesWindowConfig,
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
            normalize: TimeSeriesNormalize::MinMaxPerWindow,
            incomplete_window_policy: IncompleteWindowPolicy::Exclude,
            unknown_label_policy: UnknownLabelPolicy::Exclude,
            annotation_suffix: None,
            channel_map: None,
            presentation: crate::adapters::time_series::config::TimeSeriesPresentation::Snapshot,
            amplitude_offset: 0.0,
            class_keep_percents: BTreeMap::new(),
            dataset_unit_range: None,
        }
    }

    fn episode() -> AnalogEpisode {
        let mut streams = BTreeMap::new();
        streams.insert(
            "lead_0".to_string(),
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
        );
        AnalogEpisode {
            episode_id: "100".to_string(),
            sample_rate_hz: 360.0,
            streams,
            events: vec![
                AnalogEvent {
                    sample_index: 2,
                    label: "N".to_string(),
                },
                AnalogEvent {
                    sample_index: 5,
                    label: "V".to_string(),
                },
            ],
        }
    }

    #[test]
    fn preview_returns_raw_windows_in_event_order() {
        let preview = preview_episodes(&[episode()], &config(), 0, 2).expect("preview");
        assert_eq!(preview.record_id, "100");
        assert_eq!(preview.sample_rate_hz, 360.0);
        assert_eq!(preview.total_frames, 2);
        assert_eq!(preview.frame_offset, 0);
        assert_eq!(preview.record_count, 1);
        assert_eq!(
            preview.class_counts,
            vec![
                PreviewClassCount {
                    label: "N".to_string(),
                    count: 1,
                },
                PreviewClassCount {
                    label: "V".to_string(),
                    count: 1,
                },
            ]
        );
        assert_eq!(preview.frames.len(), 2);
        assert_eq!(preview.frames[0].label, "N");
        assert_eq!(preview.frames[0].record_id, "100");
        assert_eq!(preview.value_min, 1.0);
        assert_eq!(preview.value_max, 7.0);
        assert_eq!(preview.frames[0].values, vec![2.0, 3.0, 4.0]);
        assert_eq!(preview.frames[1].label, "V");
        assert_eq!(preview.frames[1].values, vec![5.0, 6.0, 7.0]);
    }

    #[test]
    fn preview_pages_from_frame_offset() {
        let preview = preview_episodes(&[episode()], &config(), 1, 1).expect("page");
        assert_eq!(preview.frame_offset, 1);
        assert_eq!(preview.total_frames, 2);
        assert_eq!(preview.frames.len(), 1);
        assert_eq!(preview.frames[0].label, "V");
    }

    #[test]
    fn preview_rejects_zero_frame_count() {
        let error = preview_episodes(&[episode()], &config(), 0, 0).expect_err("zero");
        assert!(error.to_string().contains("frame_count"));
    }
}
