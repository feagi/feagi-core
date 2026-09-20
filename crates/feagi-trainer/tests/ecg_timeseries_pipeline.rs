//! Integration test: WFDB analog ingest windows into classification IR and metrics.
//!
//! Uses a synthetic two-channel format-212 record (not a downloaded PhysioNet archive).

use feagi_trainer::adapters::{
    IncompleteWindowPolicy, TimeSeriesNormalize, TimeSeriesPackageAdapter, TimeSeriesPackageConfig,
    TimeSeriesPresentation, TimeSeriesSourceKind, TimeSeriesWindowConfig, UnknownLabelPolicy,
    WfdbChannelMap,
};
use feagi_trainer::contracts::common::Split;
use feagi_trainer::contracts::ir_sample::Payload;
use feagi_trainer::contracts::{SplitId, TypedPrediction, TypedTarget};
use feagi_trainer::metrics::ClassificationMetricPack;
use feagi_trainer::plugins::{AdapterPlugin, DatasetSource, MetricPackPlugin, SamplerPlugin};
use feagi_trainer::samplers::SequentialSampler;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn wfdb_event_windows_compose_with_classification_metrics() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("feagi-ecg-pipeline-{nanos}"));
    std::fs::create_dir_all(&dir).expect("temp");

    let mut samples = Vec::new();
    for t in 0..32 {
        samples.push(t as i16);
        samples.push((40 - t) as i16);
    }
    feagi_trainer::adapters::time_series_test::write_wfdb_record(
        &dir,
        "100",
        &samples,
        2,
        &[(8, 1), (16, 5), (24, 1)],
    )
    .expect("write wfdb");

    let adapter = TimeSeriesPackageAdapter::new(TimeSeriesPackageConfig {
        dataset_name: "synthetic_ecg".to_string(),
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
    });

    let source = DatasetSource {
        uri: dir.to_string_lossy().into_owned(),
        bytes: Vec::new(),
    };
    let manifest = adapter.discover(&source).expect("discover");
    assert_eq!(
        manifest.modality,
        feagi_trainer::contracts::Modality::TimeSeries
    );
    assert!(adapter.validate(&manifest).expect("validate").passed);

    let ir = adapter
        .stream(&source, &SplitId("train".to_string()))
        .expect("stream");
    assert_eq!(ir.len(), 3);
    match &ir[0].payload {
        Payload::TimeSeries {
            samples,
            channel_ids,
            ..
        } => {
            assert_eq!(channel_ids, &["lead_0".to_string()]);
            assert_eq!(samples.len(), 5);
        }
        other => panic!("expected time-series payload, got {other:?}"),
    }

    let order = SequentialSampler::new().plan(ir.len(), 7);
    let mut targets = Vec::new();
    let mut predictions = Vec::new();
    for &idx in &order {
        let TypedTarget::Class { class_id, .. } = ir[idx].target.clone().expect("labeled") else {
            panic!("expected class target");
        };
        targets.push(TypedTarget::Class {
            class_id,
            label: None,
        });
        predictions.push(TypedPrediction::Class {
            class_id,
            scores: vec![],
        });
    }
    let result = ClassificationMetricPack::new()
        .evaluate(&predictions, &targets)
        .expect("evaluate");
    assert!((result.metrics["accuracy"] - 1.0).abs() < 1e-12);

    let _ = std::fs::remove_dir_all(&dir);
}
