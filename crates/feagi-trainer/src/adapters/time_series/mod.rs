//! Annotated time-series adapter — WFDB or a canonical analog package to class-labeled windows.
//!
//! Cityscapes is a folder-layout codec for image+mask. This adapter is the same idea for
//! analog+events: the dataset brand (MIT-BIH, PTB-XL, a local package) is configuration,
//! not a separate engine path.

mod config;
mod corpus;
mod encoded_preview;
mod package;
mod preview;
mod stream_episode;
mod wfdb;
mod window;

pub use config::{
    IncompleteWindowPolicy, TimeSeriesNormalize, TimeSeriesPackageConfig, TimeSeriesPresentation,
    TimeSeriesSourceKind, TimeSeriesWindowConfig, UnknownLabelPolicy, WfdbChannelMap,
};
pub use encoded_preview::EncodedWindowPreview;
pub use preview::{PreviewClassCount, TimeSeriesPreview, TimeSeriesPreviewFrame};
pub use wfdb::write_test_record as write_wfdb_record;

use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::adapters::time_series::corpus::AnalogEpisode;
use crate::adapters::time_series::stream_episode::episode_stream_sample;
use crate::adapters::time_series::window::{window_event, EpisodeScale};
use crate::contracts::common::{
    ContentHash, DatasetAssetId, DatasetVersionId, MetadataValue, Modality, OutputType, PluginId,
    PluginRef,
};
use crate::contracts::dataset_manifest::SCHEMA_VERSION as MANIFEST_SCHEMA_VERSION;
use crate::contracts::{DatasetManifest, IRSample, SplitDescriptor};
use crate::error::TrainerError;
use crate::plugins::{AdapterPlugin, DatasetSource, ValidationReport};

/// Manifest metadata key: how many samples were left out because the range used to scale them
/// (window, episode or dataset, per the normalize mode) was flat. Stored as `MetadataValue::Int`.
pub const SKIPPED_FLAT_WINDOWS_KEY: &str = "skipped_flat_windows";

/// Per-episode range for `MinMaxPerEpisode`, `None` for other modes. Shared by ingest and the
/// encoded preview.
fn episode_range_for(
    config: &TimeSeriesPackageConfig,
    episode: &AnalogEpisode,
) -> Result<Option<(f64, f64)>, TrainerError> {
    if config.normalize != TimeSeriesNormalize::MinMaxPerEpisode {
        return Ok(None);
    }
    preview::streamed_value_range(std::slice::from_ref(episode), &config.window.stream_ids)
        .map(Some)
}

/// What a load depends on. Window, normalize and feature settings are applied at preview time and
/// are not part of it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct LoadKey {
    source_kind: TimeSeriesSourceKind,
    uri: String,
    annotation_suffix: Option<String>,
    channel_map: Option<Vec<WfdbChannelMap>>,
}

/// A corpus loaded once, reusable by every preview of an adapter whose config loads the same
/// source (same source kind, uri, annotation suffix and channel map). Opaque by design.
#[derive(Debug)]
pub struct LoadedTimeSeries {
    key: LoadKey,
    episodes: Vec<AnalogEpisode>,
}

/// Adapter that windows annotated analog streams into classification `IRSample`s.
#[derive(Debug, Clone)]
pub struct TimeSeriesPackageAdapter {
    config: TimeSeriesPackageConfig,
}

impl TimeSeriesPackageAdapter {
    /// Stable plugin id.
    pub const PLUGIN_ID: &'static str = "time_series_package";

    /// Creates an adapter from explicit configuration.
    pub fn new(config: TimeSeriesPackageConfig) -> Self {
        Self { config }
    }

    fn root<'a>(&self, source: &'a DatasetSource) -> Result<&'a Path, TrainerError> {
        if source.uri.is_empty() {
            return Err(TrainerError::Config(
                "time-series source uri must be a filesystem directory".to_string(),
            ));
        }
        Ok(Path::new(&source.uri))
    }

    /// Loads the corpus once so several previews can share it.
    pub fn load_series(&self, source: &DatasetSource) -> Result<LoadedTimeSeries, TrainerError> {
        Ok(LoadedTimeSeries {
            key: self.load_key(&source.uri),
            episodes: self.load_episodes(source)?,
        })
    }

    /// Loads the corpus and returns one page of raw event windows.
    pub fn preview(
        &self,
        source: &DatasetSource,
        frame_offset: usize,
        frame_count: usize,
    ) -> Result<TimeSeriesPreview, TrainerError> {
        let loaded = self.load_series(source)?;
        self.preview_from(&loaded, frame_offset, frame_count)
    }

    /// One page of raw event windows from an already loaded corpus.
    pub fn preview_from(
        &self,
        loaded: &LoadedTimeSeries,
        frame_offset: usize,
        frame_count: usize,
    ) -> Result<TimeSeriesPreview, TrainerError> {
        self.check_loaded(loaded)?;
        preview::preview_episodes(&loaded.episodes, &self.config, frame_offset, frame_count)
    }

    fn load_key(&self, uri: &str) -> LoadKey {
        LoadKey {
            source_kind: self.config.source_kind,
            uri: uri.to_string(),
            annotation_suffix: self.config.annotation_suffix.clone(),
            channel_map: self.config.channel_map.clone(),
        }
    }

    /// Refuses a corpus that this adapter's config would not have loaded itself.
    fn check_loaded(&self, loaded: &LoadedTimeSeries) -> Result<(), TrainerError> {
        if self.load_key(&loaded.key.uri) != loaded.key {
            return Err(TrainerError::Config(
                "the loaded time series came from a different source kind, channel map or \
                 annotation suffix than this adapter's config needs; load it again"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn load_episodes(&self, source: &DatasetSource) -> Result<Vec<AnalogEpisode>, TrainerError> {
        self.config.validate().map_err(TrainerError::Config)?;
        let root = self.root(source)?;
        match self.config.source_kind {
            TimeSeriesSourceKind::Package => package::load_package(root),
            TimeSeriesSourceKind::Wfdb => wfdb::load_wfdb_directory(root, &self.config),
        }
    }

    fn fingerprint(episodes: &[AnalogEpisode]) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for episode in episodes {
            episode.episode_id.hash(&mut hasher);
            episode.sample_rate_hz.to_bits().hash(&mut hasher);
            for (stream_id, values) in &episode.streams {
                stream_id.hash(&mut hasher);
                values.len().hash(&mut hasher);
                for value in values {
                    value.to_bits().hash(&mut hasher);
                }
            }
            for event in &episode.events {
                event.sample_index.hash(&mut hasher);
                event.label.hash(&mut hasher);
            }
        }
        format!("siphash64:{:016x}", hasher.finish())
    }

    /// Config with the dataset-wide range resolved when the normalize mode needs it.
    /// The one place the dataset range is set up, shared by ingest and the encoded preview.
    fn config_with_dataset_range(
        &self,
        episodes: &[AnalogEpisode],
    ) -> Result<TimeSeriesPackageConfig, TrainerError> {
        let mut config = self.config.clone();
        if config.normalize == TimeSeriesNormalize::MinMaxDataset {
            config.dataset_unit_range = Some(preview::streamed_value_range(
                episodes,
                &config.window.stream_ids,
            )?);
        }
        Ok(config)
    }

    fn emit_samples(
        &self,
        source: &DatasetSource,
        episodes: &[AnalogEpisode],
        dataset_version_id: &DatasetVersionId,
    ) -> Result<(Vec<IRSample>, u64), TrainerError> {
        let config = self.config_with_dataset_range(episodes)?;
        let mut scale = EpisodeScale::default();
        let mut flat_skips = 0_u64;
        let mut samples = Vec::new();
        match config.presentation {
            TimeSeriesPresentation::Snapshot | TimeSeriesPresentation::StreamTrain => {
                for episode in episodes {
                    scale.episode_range = episode_range_for(&config, episode)?;
                    for (ordinal, event) in episode.events.iter().enumerate() {
                        if let Some(sample) = window_event(
                            episode,
                            event,
                            ordinal,
                            &config,
                            dataset_version_id,
                            &source.uri,
                            &mut scale,
                        )? {
                            samples.push(sample);
                        }
                    }
                }
            }
            TimeSeriesPresentation::StreamInfer => {
                for episode in episodes {
                    if let Some(sample) = episode_stream_sample(
                        episode,
                        &config,
                        dataset_version_id,
                        &source.uri,
                        &mut flat_skips,
                    )? {
                        samples.push(sample);
                    }
                }
            }
        }
        if samples.is_empty() {
            return Err(TrainerError::Parse(format!(
                "time-series ingest produced no samples (check class_labels and window policies; \
                 {} skipped for a flat normalization range)",
                flat_skips + scale.flat_skips
            )));
        }
        let samples = crate::adapters::class_keep::apply_class_keep_percents(
            samples,
            &self.config.class_keep_percents,
            &self.config.class_labels,
        )?;
        let samples = crate::adapters::class_keep::apply_balanced_max_samples(
            samples,
            self.config.max_samples,
            self.config.sample_draw_seed,
            crate::adapters::class_keep::ir_sample_class_id,
        )?;
        Ok((samples, flat_skips + scale.flat_skips))
    }
}

impl AdapterPlugin for TimeSeriesPackageAdapter {
    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId(Self::PLUGIN_ID.to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn discover(&self, source: &DatasetSource) -> Result<DatasetManifest, TrainerError> {
        let episodes = self.load_episodes(source)?;
        let fingerprint = Self::fingerprint(&episodes);
        let version_id = DatasetVersionId(format!("{}@{fingerprint}", self.config.dataset_name));
        let (samples, flat_skips) = self.emit_samples(source, &episodes, &version_id)?;
        let mut metadata = std::collections::BTreeMap::new();
        metadata.insert(
            SKIPPED_FLAT_WINDOWS_KEY.to_string(),
            MetadataValue::Int(flat_skips as i64),
        );
        Ok(DatasetManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            dataset_version_id: version_id,
            dataset_asset_id: DatasetAssetId(format!("local:{}", self.config.dataset_name)),
            dataset_version: "1.0.0".to_string(),
            source_uri: source.uri.clone(),
            content_hash: ContentHash(fingerprint),
            schema_fingerprint: ContentHash("time_series_event_window_v1".to_string()),
            modality: Modality::TimeSeries,
            output_type: OutputType::Class,
            splits: vec![SplitDescriptor {
                id: self.config.split_id.clone(),
                split: self.config.split.clone(),
                sample_count: samples.len() as u64,
            }],
            metadata,
        })
    }

    fn validate(&self, manifest: &DatasetManifest) -> Result<ValidationReport, TrainerError> {
        let mut issues = Vec::new();
        if manifest.modality != Modality::TimeSeries {
            issues.push(format!(
                "modality {:?} is not TimeSeries",
                manifest.modality
            ));
        }
        if manifest.output_type != OutputType::Class {
            issues.push(format!(
                "output_type {:?} is not Class",
                manifest.output_type
            ));
        }
        if manifest.splits.iter().all(|split| split.sample_count == 0) {
            issues.push("all splits have zero samples".to_string());
        }
        Ok(ValidationReport {
            passed: issues.is_empty(),
            issues,
        })
    }

    fn stream(
        &self,
        source: &DatasetSource,
        split: &crate::contracts::SplitId,
    ) -> Result<Vec<IRSample>, TrainerError> {
        if split != &self.config.split_id {
            return Err(TrainerError::Parse(format!("unknown split '{}'", split.0)));
        }
        let episodes = self.load_episodes(source)?;
        let fingerprint = Self::fingerprint(&episodes);
        let version_id = DatasetVersionId(format!("{}@{fingerprint}", self.config.dataset_name));
        self.emit_samples(source, &episodes, &version_id)
            .map(|(samples, _)| samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::time_series::config::{
        IncompleteWindowPolicy, TimeSeriesNormalize, TimeSeriesWindowConfig, UnknownLabelPolicy,
    };
    use crate::contracts::ir_sample::Payload;
    use crate::contracts::{Split, SplitId};
    use std::collections::BTreeMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("feagi-ts-{tag}-{nanos}"));
        std::fs::create_dir_all(&dir).expect("temp");
        dir
    }

    fn package_config() -> TimeSeriesPackageConfig {
        TimeSeriesPackageConfig {
            dataset_name: "ecg".to_string(),
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
            normalize: TimeSeriesNormalize::MinMaxPerWindow,
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

    #[test]
    fn package_discover_and_stream() {
        let dir = temp_dir("pkg");
        let samples: Vec<f32> = (0..20).map(|v| v as f32).collect();
        package::write_test_package(
            &dir,
            360.0,
            "rec0",
            "lead_0",
            &samples,
            &[(8, "N"), (12, "V")],
        )
        .expect("write package");
        let adapter = TimeSeriesPackageAdapter::new(package_config());
        let source = DatasetSource {
            uri: dir.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let manifest = adapter.discover(&source).expect("discover");
        assert_eq!(manifest.modality, Modality::TimeSeries);
        assert_eq!(manifest.splits[0].sample_count, 2);
        let report = adapter.validate(&manifest).expect("validate");
        assert!(report.passed);
        let ir = adapter
            .stream(&source, &SplitId("train".to_string()))
            .expect("stream");
        assert_eq!(ir.len(), 2);
        match &ir[0].payload {
            Payload::TimeSeries { samples, .. } => assert_eq!(samples.len(), 5),
            other => panic!("unexpected {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn class_keep_percents_keep_first_windows_per_class() {
        let dir = temp_dir("keep");
        let samples: Vec<f32> = (0..24).map(|v| v as f32).collect();
        package::write_test_package(
            &dir,
            360.0,
            "rec0",
            "lead_0",
            &samples,
            &[(4, "N"), (8, "N"), (12, "V"), (16, "N")],
        )
        .expect("write package");
        let mut config = package_config();
        config.class_keep_percents.insert("N".to_string(), 50);
        config.class_keep_percents.insert("V".to_string(), 100);
        let adapter = TimeSeriesPackageAdapter::new(config);
        let source = DatasetSource {
            uri: dir.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let ir = adapter
            .stream(&source, &SplitId("train".to_string()))
            .expect("stream");
        let labels: Vec<_> = ir
            .iter()
            .map(|sample| match &sample.target {
                Some(crate::contracts::TypedTarget::Class {
                    label: Some(label), ..
                }) => label.clone(),
                other => panic!("unexpected {other:?}"),
            })
            .collect();
        assert_eq!(labels, vec!["N", "V"]);
        let preview = adapter.preview(&source, 0, 8).expect("preview");
        assert_eq!(preview.total_frames, 4);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wfdb_directory_streams_beats() {
        let dir = temp_dir("wfdb");
        let mut samples = Vec::new();
        for t in 0..16 {
            samples.push(t as i16);
            samples.push((20 - t) as i16);
        }
        wfdb::write_test_record(&dir, "100", &samples, 2, &[(6, 1), (10, 5)]).expect("write wfdb");
        let config = TimeSeriesPackageConfig {
            dataset_name: "wfdb".to_string(),
            source_kind: TimeSeriesSourceKind::Wfdb,
            window: TimeSeriesWindowConfig {
                pre_samples: 2,
                post_samples: 2,
                stream_ids: vec!["lead_0".to_string()],
                feature_count: 5,
            },
            class_labels: vec!["N".to_string(), "V".to_string()],
            split: Split::Train,
            split_id: SplitId("train".to_string()),
            normalize: TimeSeriesNormalize::MinMaxPerWindow,
            incomplete_window_policy: IncompleteWindowPolicy::Exclude,
            unknown_label_policy: UnknownLabelPolicy::Exclude,
            annotation_suffix: Some("atr".to_string()),
            channel_map: Some(vec![
                WfdbChannelMap {
                    source_index: 0,
                    stream_id: "lead_0".to_string(),
                },
                WfdbChannelMap {
                    source_index: 1,
                    stream_id: "lead_1".to_string(),
                },
            ]),
            presentation: TimeSeriesPresentation::Snapshot,
            amplitude_offset: 0.0,
            class_keep_percents: BTreeMap::new(),
            max_samples: None,
            sample_draw_seed: None,
            dataset_unit_range: None,
        };
        let adapter = TimeSeriesPackageAdapter::new(config);
        let source = DatasetSource {
            uri: dir.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let ir = adapter
            .stream(&source, &SplitId("train".to_string()))
            .expect("stream");
        assert_eq!(ir.len(), 2);
        let preview = adapter.preview(&source, 0, 2).expect("preview");
        assert_eq!(preview.record_id, "100");
        assert_eq!(preview.frames.len(), 2);
        assert_eq!(preview.frames[0].label, "N");
        assert_eq!(preview.frames[0].values.len(), 5);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn flat_windows_are_skipped_and_counted_in_manifest_metadata() {
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
        let adapter = TimeSeriesPackageAdapter::new(package_config());
        let source = DatasetSource {
            uri: dir.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let manifest = adapter.discover(&source).expect("discover");
        assert_eq!(manifest.splits[0].sample_count, 1);
        assert_eq!(
            manifest.metadata.get(SKIPPED_FLAT_WINDOWS_KEY),
            Some(&MetadataValue::Int(1))
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
