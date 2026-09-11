//! Metric-pack axis — evaluation from persisted predictions + targets (design Section 5.8).
//!
//! Metric packs are pure: they consume aligned prediction/target sequences and emit named
//! metric values plus an optional confusion matrix. They never read the runtime.

use std::collections::BTreeMap;

use crate::contracts::common::PluginRef;
use crate::contracts::{TypedPrediction, TypedTarget};
use crate::error::TrainerError;

/// Per-class precision/recall/F1, keyed by class id.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassMetrics {
    /// Precision (`tp / (tp + fp)`), or `0.0` when undefined (no predicted positives).
    pub precision: f64,
    /// Recall (`tp / (tp + fn)`), or `0.0` when undefined (no actual positives).
    pub recall: f64,
    /// F1 (harmonic mean of precision and recall), or `0.0` when both are zero.
    pub f1: f64,
}

/// A confusion matrix over the union of observed class ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfusionMatrix {
    /// Sorted class ids; row/column indices map to this ordering.
    pub class_ids: Vec<u32>,
    /// `counts[true_index][pred_index]` occurrences.
    pub counts: Vec<Vec<u64>>,
}

/// The result of evaluating a metric pack.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricResult {
    /// Named aggregate metric values (deterministically ordered).
    pub metrics: BTreeMap<String, f64>,
    /// Optional confusion matrix (classification packs).
    pub confusion: Option<ConfusionMatrix>,
}

impl MetricResult {
    /// Merges this result's metric values into a scorecard-bound metrics map.
    ///
    /// This is the wiring used to combine an orthogonal pack (e.g. the spike-cost pack, whose
    /// keys are `cost.`-namespaced) into the same `Scorecard::metrics` map as a primary pack.
    /// A key collision is an explicit error rather than a silent overwrite, so two packs can
    /// never quietly clobber each other's metrics. The confusion matrix is intentionally not
    /// merged (it has no scalar representation).
    pub fn merge_into(&self, target: &mut BTreeMap<String, f64>) -> Result<(), TrainerError> {
        for (key, value) in &self.metrics {
            if target.contains_key(key) {
                return Err(TrainerError::Evaluation(format!(
                    "metric key '{key}' already present; refusing to overwrite on merge"
                )));
            }
            target.insert(key.clone(), *value);
        }
        Ok(())
    }
}

/// Scores predictions against targets for a task family.
pub trait MetricPackPlugin {
    /// Identifies this metric pack (axis provenance).
    fn plugin_ref(&self) -> PluginRef;

    /// Computes metrics from aligned `predictions` and `targets`.
    ///
    /// Returns an explicit error on length mismatch, empty input, or a prediction/target
    /// whose variant does not match the pack's task family.
    fn evaluate(
        &self,
        predictions: &[TypedPrediction],
        targets: &[TypedTarget],
    ) -> Result<MetricResult, TrainerError>;
}
