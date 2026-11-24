//! Encoded preview: one eligible window as the population encoder will see it.

use serde::{Deserialize, Serialize};

use crate::adapters::time_series::config::{TimeSeriesNormalize, TimeSeriesPackageConfig};
use crate::adapters::time_series::corpus::AnalogEpisode;
use crate::adapters::time_series::window::{raw_window_range, window_event, EpisodeScale};
use crate::adapters::time_series::{episode_range_for, LoadedTimeSeries, TimeSeriesPackageAdapter};
use crate::binding::population_encoder::PopulationEncoder;
use crate::contracts::ir_sample::Payload;
use crate::contracts::DatasetVersionId;
use crate::error::TrainerError;
use crate::plugins::DatasetSource;

/// One window after normalization and population-code binning.
///
/// When `skipped_flat` is true the window's scaling range had zero span, so no values exist:
/// `values01` and `z_bins` are empty, and `range_used` is the flat range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncodedWindowPreview {
    /// Configured streams, in window order.
    pub stream_ids: Vec<String>,
    /// Normalized values in `[0, 1]`, one series per stream (de-interleaved), time order.
    pub values01: Vec<Vec<f64>>,
    /// Population-code Z bin per value, same shape as `values01`.
    pub z_bins: Vec<Vec<u32>>,
    /// `(min, max)` the window was scaled by. For per-window scaling over several streams this
    /// is the union of the streams' raw window ranges.
    pub range_used: (f64, f64),
    /// Which normalization produced `range_used`.
    pub range_kind: TimeSeriesNormalize,
    /// Class label of the event the window is centered on.
    pub class_label: String,
    /// Episode (record) the window comes from.
    pub episode_id: String,
    /// Sample index of the event inside the episode.
    pub sample_index: u64,
    /// True when the first eligible window was left out for a flat scaling range.
    pub skipped_flat: bool,
    /// Bin count used for `z_bins`.
    pub encoder_bins: u32,
}

impl TimeSeriesPackageAdapter {
    /// Previews the eligible window at `frame_index` of the source, encoded with the population
    /// code. The index counts the same windows, in the same order, as the raw
    /// [`Self::preview_from`] `frame_offset`, so frame `i` raw and frame `i` encoded are one window.
    ///
    /// "Eligible" means the window survives label and edge policies; it is taken before the
    /// class-keep percentages and the balanced sample draw, so it is not necessarily the first
    /// sample a run presents. Event windows are used whatever the presentation mode. The range
    /// setup is shared with ingest. The population code ignores the amplitude offset, so it is
    /// not applied here. Requires a `[0, 1]` normalization (not `None`).
    pub fn preview_encoded(
        &self,
        source: &DatasetSource,
        encoder_bins: u32,
        frame_index: usize,
    ) -> Result<EncodedWindowPreview, TrainerError> {
        if self.config.normalize == TimeSeriesNormalize::None {
            return Err(none_error());
        }
        let loaded = self.load_series(source)?;
        self.preview_encoded_from(&loaded, encoder_bins, frame_index)
    }

    /// Same as [`Self::preview_encoded`] on a corpus loaded earlier with
    /// [`Self::load_series`]. Window, normalize and feature settings are read from this adapter's
    /// config at call time, so one loaded corpus serves different settings.
    pub fn preview_encoded_from(
        &self,
        loaded: &LoadedTimeSeries,
        encoder_bins: u32,
        frame_index: usize,
    ) -> Result<EncodedWindowPreview, TrainerError> {
        if self.config.normalize == TimeSeriesNormalize::None {
            return Err(none_error());
        }
        self.check_loaded(loaded)?;
        let episodes = &loaded.episodes;
        let source_uri = loaded.key.uri.as_str();
        let mut config = self.config_with_dataset_range(episodes)?;
        config.amplitude_offset = 0.0;
        let version = DatasetVersionId("preview".to_string());
        // Windows seen so far, in raw order: every window that passes the label and edge
        // policies counts, including ones the encoder leaves out for a flat range.
        let mut seen = 0_usize;
        for episode in episodes {
            let mut scale = EpisodeScale {
                episode_range: episode_range_for(&config, episode)?,
                flat_skips: 0,
            };
            for (ordinal, event) in episode.events.iter().enumerate() {
                let flat_before = scale.flat_skips;
                let sample = window_event(
                    episode, event, ordinal, &config, &version, source_uri, &mut scale,
                )?;
                let identity = |skipped_flat: bool, range: (f64, f64)| EncodedWindowPreview {
                    stream_ids: config.window.stream_ids.clone(),
                    values01: Vec::new(),
                    z_bins: Vec::new(),
                    range_used: range,
                    range_kind: config.normalize,
                    class_label: event.label.clone(),
                    episode_id: episode.episode_id.clone(),
                    sample_index: event.sample_index,
                    skipped_flat,
                    encoder_bins,
                };
                let is_flat = sample.is_none() && scale.flat_skips > flat_before;
                if sample.is_none() && !is_flat {
                    continue;
                }
                if seen != frame_index {
                    seen += 1;
                    continue;
                }
                match sample {
                    Some(sample) => {
                        let Payload::TimeSeries { samples, .. } = sample.payload else {
                            return Err(TrainerError::Parse(
                                "time-series window produced a non-time-series payload".to_string(),
                            ));
                        };
                        let range = range_used(&config, &scale, episode, event.sample_index)?;
                        let mut out = identity(false, range);
                        fill_values(&mut out, &samples)?;
                        return Ok(out);
                    }
                    None => {
                        let range = range_used(&config, &scale, episode, event.sample_index)?;
                        return Ok(identity(true, range));
                    }
                }
            }
        }
        Err(TrainerError::Config(format!(
            "preview frame_index {frame_index} is past the {seen} eligible frames"
        )))
    }
}

fn none_error() -> TrainerError {
    TrainerError::Config("an encoded preview requires a [0,1] normalization (not none)".to_string())
}

fn range_used(
    config: &TimeSeriesPackageConfig,
    scale: &EpisodeScale,
    episode: &AnalogEpisode,
    event_sample_index: u64,
) -> Result<(f64, f64), TrainerError> {
    let missing = || TrainerError::Config("the normalization range was not resolved".to_string());
    match config.normalize {
        TimeSeriesNormalize::MinMaxDataset => config.dataset_unit_range.ok_or_else(missing),
        TimeSeriesNormalize::MinMaxPerEpisode => scale.episode_range.ok_or_else(missing),
        _ => raw_window_range(episode, event_sample_index, config),
    }
}

/// De-interleaves time-then-channel samples and bins each value.
fn fill_values(out: &mut EncodedWindowPreview, samples: &[f64]) -> Result<(), TrainerError> {
    let streams = out.stream_ids.len();
    out.values01 = vec![Vec::new(); streams];
    out.z_bins = vec![Vec::new(); streams];
    for (index, value) in samples.iter().enumerate() {
        let stream = index % streams;
        out.z_bins[stream].push(PopulationEncoder::linear_z_index(*value, out.encoder_bins)?);
        out.values01[stream].push(*value);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::time_series::config::{
        IncompleteWindowPolicy, TimeSeriesPresentation, TimeSeriesSourceKind,
        TimeSeriesWindowConfig, UnknownLabelPolicy, WfdbChannelMap,
    };
    use crate::adapters::time_series::{package, wfdb};
    use crate::contracts::{Split, SplitId};
    use crate::plugins::AdapterPlugin;
    use std::collections::BTreeMap;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("feagi-ts-enc-{tag}-{nanos}"));
        std::fs::create_dir_all(&dir).expect("temp");
        dir
    }

    fn config(normalize: TimeSeriesNormalize) -> TimeSeriesPackageConfig {
        TimeSeriesPackageConfig {
            dataset_name: "set".to_string(),
            source_kind: TimeSeriesSourceKind::Package,
            window: TimeSeriesWindowConfig {
                pre_samples: 2,
                post_samples: 2,
                stream_ids: vec!["lead_0".to_string()],
                feature_count: 5,
            },
            class_labels: vec!["N".to_string(), "V".to_string()],
            split: Split::Train,
            split_id: SplitId("train".to_string()),
            normalize,
            incomplete_window_policy: IncompleteWindowPolicy::Exclude,
            unknown_label_policy: UnknownLabelPolicy::Exclude,
            annotation_suffix: None,
            channel_map: None,
            presentation: TimeSeriesPresentation::Snapshot,
            amplitude_offset: 0.0,
            class_keep_percents: BTreeMap::new(),
            max_samples: None,
            sample_draw_seed: None,
            dataset_unit_range: None,
        }
    }

    fn source(dir: &std::path::Path) -> DatasetSource {
        DatasetSource {
            uri: dir.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        }
    }

    fn write_ramp(dir: &std::path::Path, samples: &[f32]) {
        package::write_test_package(
            dir,
            360.0,
            "rec0",
            "lead_0",
            samples,
            &[(8, "N"), (12, "V")],
        )
        .expect("write package");
    }

    #[test]
    fn matches_first_emitted_sample_for_every_range_mode() {
        let dir = temp_dir("modes");
        let samples: Vec<f32> = (0..20).map(|v| (v * v) as f32).collect();
        write_ramp(&dir, &samples);
        for mode in [
            TimeSeriesNormalize::MinMaxPerWindow,
            TimeSeriesNormalize::MinMaxPerEpisode,
            TimeSeriesNormalize::MinMaxDataset,
        ] {
            let adapter = TimeSeriesPackageAdapter::new(config(mode));
            let src = source(&dir);
            let preview = adapter.preview_encoded(&src, 16, 0).expect("preview");
            let first = adapter
                .stream(&src, &SplitId("train".to_string()))
                .expect("stream")
                .remove(0);
            let Payload::TimeSeries { samples, .. } = first.payload else {
                panic!("payload");
            };
            assert_eq!(preview.values01, vec![samples], "{mode:?}");
            assert_eq!(preview.range_kind, mode);
            assert_eq!(preview.class_label, "N");
            assert_eq!(preview.sample_index, 8);
            assert_eq!(preview.episode_id, "rec0");
            assert!(!preview.skipped_flat);
            let expected_range = match mode {
                TimeSeriesNormalize::MinMaxPerWindow => (36.0, 100.0),
                _ => (0.0, 361.0),
            };
            assert_eq!(preview.range_used, expected_range, "{mode:?}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn z_bins_match_the_population_encoder() {
        let dir = temp_dir("bins");
        let samples: Vec<f32> = (0..20).map(|v| v as f32).collect();
        write_ramp(&dir, &samples);
        let adapter = TimeSeriesPackageAdapter::new(config(TimeSeriesNormalize::MinMaxDataset));
        let preview = adapter
            .preview_encoded(&source(&dir), 10, 0)
            .expect("preview");
        assert_eq!(preview.encoder_bins, 10);
        for (values, bins) in preview.values01.iter().zip(&preview.z_bins) {
            assert_eq!(values.len(), bins.len());
            for (value, bin) in values.iter().zip(bins) {
                assert_eq!(
                    *bin,
                    PopulationEncoder::linear_z_index(*value, 10).expect("bin")
                );
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn flat_first_window_is_marked_skipped() {
        let dir = temp_dir("flat");
        let mut samples = vec![1.0_f32; 10];
        samples.extend((0..10).map(|v| v as f32));
        package::write_test_package(
            &dir,
            360.0,
            "rec0",
            "lead_0",
            &samples,
            &[(5, "N"), (15, "V")],
        )
        .expect("write package");
        let adapter = TimeSeriesPackageAdapter::new(config(TimeSeriesNormalize::MinMaxPerWindow));
        let preview = adapter
            .preview_encoded(&source(&dir), 8, 0)
            .expect("preview");
        assert!(preview.skipped_flat);
        assert!(preview.values01.is_empty() && preview.z_bins.is_empty());
        assert_eq!(preview.range_used, (1.0, 1.0));
        assert_eq!(preview.class_label, "N");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn none_normalization_is_a_config_error() {
        let dir = temp_dir("none");
        write_ramp(&dir, &(0..20).map(|v| v as f32).collect::<Vec<_>>());
        let adapter = TimeSeriesPackageAdapter::new(config(TimeSeriesNormalize::None));
        let err = adapter.preview_encoded(&source(&dir), 8, 0).unwrap_err();
        assert!(matches!(err, TrainerError::Config(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn two_stream_window_is_de_interleaved() {
        let dir = temp_dir("two");
        let mut samples = Vec::new();
        for t in 0..16 {
            samples.push(t as i16);
            samples.push((20 - t) as i16);
        }
        wfdb::write_test_record(&dir, "100", &samples, 2, &[(6, 1)]).expect("write wfdb");
        let mut cfg = config(TimeSeriesNormalize::MinMaxPerWindow);
        cfg.source_kind = TimeSeriesSourceKind::Wfdb;
        cfg.window.stream_ids = vec!["lead_0".to_string(), "lead_1".to_string()];
        cfg.annotation_suffix = Some("atr".to_string());
        cfg.channel_map = Some(vec![
            WfdbChannelMap {
                source_index: 0,
                stream_id: "lead_0".to_string(),
            },
            WfdbChannelMap {
                source_index: 1,
                stream_id: "lead_1".to_string(),
            },
        ]);
        let adapter = TimeSeriesPackageAdapter::new(cfg);
        let preview = adapter
            .preview_encoded(&source(&dir), 4, 0)
            .expect("preview");
        assert_eq!(preview.stream_ids, vec!["lead_0", "lead_1"]);
        let close = |got: &[f64], want: [f64; 5]| {
            assert!(
                got.iter().zip(want).all(|(g, w)| (g - w).abs() < 1e-5),
                "{got:?}"
            );
        };
        close(&preview.values01[0], [0.0, 0.25, 0.5, 0.75, 1.0]);
        close(&preview.values01[1], [1.0, 0.75, 0.5, 0.25, 0.0]);
        for stream in 0..2 {
            let expected: Vec<u32> = preview.values01[stream]
                .iter()
                .map(|v| PopulationEncoder::linear_z_index(*v, 4).expect("bin"))
                .collect();
            assert_eq!(preview.z_bins[stream], expected);
        }
        assert!(preview.z_bins[0][0] > preview.z_bins[0][4]);
        assert!(preview.z_bins[1][0] < preview.z_bins[1][4]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn encoded_from_loaded_equals_encoded_for_every_mode() {
        let dir = temp_dir("from");
        write_ramp(&dir, &(0..20).map(|v| (v * v) as f32).collect::<Vec<_>>());
        let src = source(&dir);
        for mode in [
            TimeSeriesNormalize::MinMaxPerWindow,
            TimeSeriesNormalize::MinMaxPerEpisode,
            TimeSeriesNormalize::MinMaxDataset,
        ] {
            let adapter = TimeSeriesPackageAdapter::new(config(mode));
            let loaded = adapter.load_series(&src).expect("load");
            assert_eq!(
                adapter.preview_encoded_from(&loaded, 16, 0).expect("from"),
                adapter.preview_encoded(&src, 16, 0).expect("direct"),
                "{mode:?}"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn one_loaded_series_serves_different_window_and_normalize_settings() {
        let dir = temp_dir("reuse");
        write_ramp(&dir, &(0..20).map(|v| (v * v) as f32).collect::<Vec<_>>());
        let src = source(&dir);
        let first = TimeSeriesPackageAdapter::new(config(TimeSeriesNormalize::MinMaxPerWindow));
        let mut wide = config(TimeSeriesNormalize::MinMaxDataset);
        wide.window.pre_samples = 3;
        wide.window.post_samples = 3;
        wide.window.feature_count = 7;
        let second = TimeSeriesPackageAdapter::new(wide);
        let loaded = first.load_series(&src).expect("load");
        let a = first.preview_encoded_from(&loaded, 16, 0).expect("a");
        let b = second.preview_encoded_from(&loaded, 16, 0).expect("b");
        assert_eq!(a, first.preview_encoded(&src, 16, 0).expect("a direct"));
        assert_eq!(b, second.preview_encoded(&src, 16, 0).expect("b direct"));
        assert_ne!(a.range_used, b.range_used);
        assert_ne!(a.values01[0].len(), b.values01[0].len());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn loaded_series_from_another_source_kind_is_refused() {
        let dir = temp_dir("kind");
        write_ramp(&dir, &(0..20).map(|v| v as f32).collect::<Vec<_>>());
        let src = source(&dir);
        let loaded = TimeSeriesPackageAdapter::new(config(TimeSeriesNormalize::MinMaxDataset))
            .load_series(&src)
            .expect("load");
        let mut other = config(TimeSeriesNormalize::MinMaxDataset);
        other.source_kind = TimeSeriesSourceKind::Wfdb;
        let adapter = TimeSeriesPackageAdapter::new(other);
        assert!(matches!(
            adapter.preview_encoded_from(&loaded, 8, 0),
            Err(TrainerError::Config(_))
        ));
        assert!(matches!(
            adapter.preview_from(&loaded, 0, 1),
            Err(TrainerError::Config(_))
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn raw_preview_from_loaded_equals_raw_preview() {
        let dir = temp_dir("raw");
        write_ramp(&dir, &(0..20).map(|v| v as f32).collect::<Vec<_>>());
        let src = source(&dir);
        let adapter = TimeSeriesPackageAdapter::new(config(TimeSeriesNormalize::MinMaxDataset));
        let loaded = adapter.load_series(&src).expect("load");
        assert_eq!(
            adapter.preview_from(&loaded, 0, 2).expect("from"),
            adapter.preview(&src, 0, 2).expect("direct")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn encoded_frame_index_matches_raw_frame_in_every_mode() {
        let dir = temp_dir("index");
        let samples: Vec<f32> = (0..30).map(|v| (v * v) as f32).collect();
        package::write_test_package(
            &dir,
            360.0,
            "rec0",
            "lead_0",
            &samples,
            &[(4, "N"), (10, "V"), (16, "N"), (22, "V")],
        )
        .expect("write package");
        let src = source(&dir);
        for mode in [
            TimeSeriesNormalize::MinMaxPerWindow,
            TimeSeriesNormalize::MinMaxPerEpisode,
            TimeSeriesNormalize::MinMaxDataset,
        ] {
            let adapter = TimeSeriesPackageAdapter::new(config(mode));
            let loaded = adapter.load_series(&src).expect("load");
            for index in [0_usize, 1, 3] {
                let raw = adapter.preview_from(&loaded, index, 1).expect("raw");
                let frame = &raw.frames[0];
                let enc = adapter
                    .preview_encoded_from(&loaded, 16, index)
                    .expect("encoded");
                assert_eq!(enc.episode_id, frame.record_id, "{mode:?} {index}");
                assert_eq!(
                    enc.sample_index, frame.event_sample_index,
                    "{mode:?} {index}"
                );
                assert_eq!(enc.class_label, frame.label, "{mode:?} {index}");
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn encoded_frame_index_past_the_end_names_the_total() {
        let dir = temp_dir("range");
        write_ramp(&dir, &(0..20).map(|v| v as f32).collect::<Vec<_>>());
        let adapter = TimeSeriesPackageAdapter::new(config(TimeSeriesNormalize::MinMaxDataset));
        let err = adapter.preview_encoded(&source(&dir), 8, 2).unwrap_err();
        match err {
            TrainerError::Config(msg) => assert!(msg.contains("2 eligible"), "{msg}"),
            other => panic!("{other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn flat_window_keeps_its_raw_index() {
        let dir = temp_dir("flatindex");
        let mut samples = vec![1.0_f32; 10];
        samples.extend((0..10).map(|v| v as f32));
        package::write_test_package(
            &dir,
            360.0,
            "rec0",
            "lead_0",
            &samples,
            &[(5, "N"), (15, "V")],
        )
        .expect("write package");
        let adapter = TimeSeriesPackageAdapter::new(config(TimeSeriesNormalize::MinMaxPerWindow));
        let second = adapter
            .preview_encoded(&source(&dir), 8, 1)
            .expect("second");
        assert!(!second.skipped_flat);
        assert_eq!(
            (second.sample_index, second.class_label.as_str()),
            (15, "V")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
