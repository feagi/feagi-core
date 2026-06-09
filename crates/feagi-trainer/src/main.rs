//! Thin `feagi-trainer` binary.
//!
//! At this stage the engine is not yet wired; the binary only reports the crate identity
//! and the current contract `schema_version`s so the build target exists and is exercised.
//! Command-line orchestration is added with the engine.

use feagi_trainer::contracts::{
    dataset_manifest, evaluation_spec, ir_sample, prediction_record, run_spec, run_summary,
    scorecard,
};

fn main() {
    println!("feagi-trainer {}", env!("CARGO_PKG_VERSION"));
    println!("contract schema versions:");
    println!("  DatasetManifest  v{}", dataset_manifest::SCHEMA_VERSION);
    println!("  IRSample         v{}", ir_sample::SCHEMA_VERSION);
    println!("  RunSpec          v{}", run_spec::SCHEMA_VERSION);
    println!("  EvaluationSpec   v{}", evaluation_spec::SCHEMA_VERSION);
    println!("  PredictionRecord v{}", prediction_record::SCHEMA_VERSION);
    println!("  RunSummary       v{}", run_summary::SCHEMA_VERSION);
    println!("  Scorecard        v{}", scorecard::SCHEMA_VERSION);
}
