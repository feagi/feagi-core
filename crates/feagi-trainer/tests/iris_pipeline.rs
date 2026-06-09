//! Integration test: the pure data-pipeline axes compose end-to-end on an IRIS-shaped CSV.
//!
//! Exercises Adapter (CSV -> IRSample) -> Sampler (deterministic order) -> MetricPack
//! (classification) without any FEAGI runtime. Predictions are synthesized deterministically
//! here (one deliberate error) purely to validate that the axes plumb together and the
//! metrics are computed correctly; real predictions come from the FEAGI binding layer later.

use feagi_trainer::adapters::{TabularCsvAdapter, TabularCsvConfig};
use feagi_trainer::contracts::common::Split;
use feagi_trainer::contracts::{SplitId, TypedPrediction, TypedTarget};
use feagi_trainer::metrics::ClassificationMetricPack;
use feagi_trainer::plugins::{AdapterPlugin, DatasetSource, MetricPackPlugin, SamplerPlugin};
use feagi_trainer::samplers::SequentialSampler;

const IRIS_CSV: &str = "sepal_length,sepal_width,petal_length,petal_width,species\n\
5.1,3.5,1.4,0.2,setosa\n\
7.0,3.2,4.7,1.4,versicolor\n\
6.3,3.3,6.0,2.5,virginica\n\
4.9,3.0,1.4,0.2,setosa\n";

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

#[test]
fn iris_pure_pipeline_composes() {
    let adapter = TabularCsvAdapter::new(config());
    let source = DatasetSource {
        uri: "mem://iris.csv".to_string(),
        bytes: IRIS_CSV.as_bytes().to_vec(),
    };

    // Ingest + validate.
    let manifest = adapter.discover(&source).expect("discover");
    assert!(adapter.validate(&manifest).expect("validate").passed);

    // Stream samples and plan a deterministic order over them.
    let samples = adapter
        .stream(&source, &SplitId("train".to_string()))
        .expect("stream");
    assert_eq!(samples.len(), 4);
    let order = SequentialSampler::new().plan(samples.len(), 42);
    assert_eq!(order, vec![0, 1, 2, 3]);

    // Collect targets in planned order; synthesize predictions with one deliberate error.
    let mut targets = Vec::new();
    let mut predictions = Vec::new();
    for (visit, &idx) in order.iter().enumerate() {
        let target = samples[idx].target.clone().expect("labeled sample");
        let TypedTarget::Class { class_id, .. } = target else {
            panic!("expected class target");
        };
        // Misclassify exactly the last visited sample.
        let predicted = if visit + 1 == order.len() {
            (class_id + 1) % 3
        } else {
            class_id
        };
        targets.push(TypedTarget::Class {
            class_id,
            label: None,
        });
        predictions.push(TypedPrediction::Class {
            class_id: predicted,
            scores: vec![],
        });
    }

    // Evaluate: 3 of 4 correct.
    let result = ClassificationMetricPack::new()
        .evaluate(&predictions, &targets)
        .expect("evaluate");
    assert!((result.metrics["accuracy"] - 0.75).abs() < 1e-12);
    assert!(result.confusion.is_some());
}
