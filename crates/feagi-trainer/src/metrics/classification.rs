//! Classification metric pack — accuracy, macro precision/recall/F1, and confusion matrix.
//!
//! Operates on single-label class predictions and targets. Aggregates are computed over the
//! union of observed class ids; macro averages weight every class equally.

use std::collections::BTreeMap;

use crate::contracts::common::{PluginId, PluginRef};
use crate::contracts::{TypedPrediction, TypedTarget};
use crate::error::TrainerError;
use crate::plugins::{ClassMetrics, ConfusionMatrix, MetricPackPlugin, MetricResult};

/// Computes classification metrics from class predictions/targets.
#[derive(Debug, Clone, Default)]
pub struct ClassificationMetricPack;

impl ClassificationMetricPack {
    /// Stable plugin id for this metric pack.
    pub const PLUGIN_ID: &'static str = "classification";

    /// Creates a new classification metric pack.
    pub fn new() -> Self {
        Self
    }

    /// Extracts the predicted class id, erroring on a non-class prediction variant.
    fn predicted_class(prediction: &TypedPrediction) -> Result<u32, TrainerError> {
        match prediction {
            TypedPrediction::Class { class_id, .. } => Ok(*class_id),
            other => Err(TrainerError::Evaluation(format!(
                "classification pack requires Class predictions, got {other:?}"
            ))),
        }
    }

    /// Extracts the target class id, erroring on a non-class target variant.
    fn target_class(target: &TypedTarget) -> Result<u32, TrainerError> {
        match target {
            TypedTarget::Class { class_id, .. } => Ok(*class_id),
            other => Err(TrainerError::Evaluation(format!(
                "classification pack requires Class targets, got {other:?}"
            ))),
        }
    }
}

impl MetricPackPlugin for ClassificationMetricPack {
    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId(Self::PLUGIN_ID.to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn evaluate(
        &self,
        predictions: &[TypedPrediction],
        targets: &[TypedTarget],
    ) -> Result<MetricResult, TrainerError> {
        if predictions.len() != targets.len() {
            return Err(TrainerError::Evaluation(format!(
                "prediction/target length mismatch: {} vs {}",
                predictions.len(),
                targets.len()
            )));
        }
        if predictions.is_empty() {
            return Err(TrainerError::Evaluation(
                "cannot evaluate an empty prediction set".to_string(),
            ));
        }

        let predicted: Vec<u32> = predictions
            .iter()
            .map(Self::predicted_class)
            .collect::<Result<_, _>>()?;
        let actual: Vec<u32> = targets
            .iter()
            .map(Self::target_class)
            .collect::<Result<_, _>>()?;

        // Sorted union of all observed class ids defines the matrix axes.
        let mut class_ids: Vec<u32> = predicted
            .iter()
            .chain(actual.iter())
            .copied()
            .collect::<std::collections::BTreeSet<u32>>()
            .into_iter()
            .collect();
        class_ids.sort_unstable();

        let index_of: BTreeMap<u32, usize> = class_ids
            .iter()
            .enumerate()
            .map(|(idx, id)| (*id, idx))
            .collect();
        let n = class_ids.len();

        let mut counts = vec![vec![0u64; n]; n];
        let mut correct = 0u64;
        for (p, a) in predicted.iter().zip(actual.iter()) {
            let pi = index_of[p];
            let ai = index_of[a];
            counts[ai][pi] += 1;
            if p == a {
                correct += 1;
            }
        }

        let total = predicted.len() as f64;
        let accuracy = correct as f64 / total;

        // Per-class precision/recall/F1 from the confusion matrix.
        let mut per_class: Vec<ClassMetrics> = Vec::with_capacity(n);
        for (i, row) in counts.iter().enumerate() {
            let tp = row[i] as f64;
            let predicted_positives: u64 = counts.iter().map(|r| r[i]).sum();
            let actual_positives: u64 = row.iter().sum();

            let precision = if predicted_positives == 0 {
                0.0
            } else {
                tp / predicted_positives as f64
            };
            let recall = if actual_positives == 0 {
                0.0
            } else {
                tp / actual_positives as f64
            };
            let f1 = if precision + recall == 0.0 {
                0.0
            } else {
                2.0 * precision * recall / (precision + recall)
            };
            per_class.push(ClassMetrics {
                precision,
                recall,
                f1,
            });
        }

        let class_count = n as f64;
        let macro_precision = per_class.iter().map(|c| c.precision).sum::<f64>() / class_count;
        let macro_recall = per_class.iter().map(|c| c.recall).sum::<f64>() / class_count;
        let macro_f1 = per_class.iter().map(|c| c.f1).sum::<f64>() / class_count;

        let mut metrics = BTreeMap::new();
        metrics.insert("accuracy".to_string(), accuracy);
        metrics.insert("macro_precision".to_string(), macro_precision);
        metrics.insert("macro_recall".to_string(), macro_recall);
        metrics.insert("macro_f1".to_string(), macro_f1);

        Ok(MetricResult {
            metrics,
            confusion: Some(ConfusionMatrix { class_ids, counts }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class_pred(id: u32) -> TypedPrediction {
        TypedPrediction::Class {
            class_id: id,
            scores: vec![],
        }
    }

    fn class_target(id: u32) -> TypedTarget {
        TypedTarget::Class {
            class_id: id,
            label: None,
        }
    }

    #[test]
    fn perfect_prediction_scores_one() {
        let pack = ClassificationMetricPack::new();
        let preds = vec![class_pred(0), class_pred(1), class_pred(2)];
        let targets = vec![class_target(0), class_target(1), class_target(2)];
        let result = pack.evaluate(&preds, &targets).expect("evaluate");
        assert_eq!(result.metrics["accuracy"], 1.0);
        assert_eq!(result.metrics["macro_f1"], 1.0);
    }

    #[test]
    fn confusion_matrix_counts_are_correct() {
        let pack = ClassificationMetricPack::new();
        // true: 0,0,1 ; pred: 0,1,1  -> accuracy 2/3
        let preds = vec![class_pred(0), class_pred(1), class_pred(1)];
        let targets = vec![class_target(0), class_target(0), class_target(1)];
        let result = pack.evaluate(&preds, &targets).expect("evaluate");
        let cm = result.confusion.expect("confusion");
        assert_eq!(cm.class_ids, vec![0, 1]);
        // row 0 (true=0): pred 0 -> 1, pred 1 -> 1
        assert_eq!(cm.counts[0], vec![1, 1]);
        // row 1 (true=1): pred 0 -> 0, pred 1 -> 1
        assert_eq!(cm.counts[1], vec![0, 1]);
        assert!((result.metrics["accuracy"] - 2.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn length_mismatch_is_error() {
        let pack = ClassificationMetricPack::new();
        let err = pack
            .evaluate(&[class_pred(0)], &[class_target(0), class_target(1)])
            .unwrap_err();
        assert!(matches!(err, TrainerError::Evaluation(_)));
    }

    #[test]
    fn empty_input_is_error() {
        let pack = ClassificationMetricPack::new();
        assert!(pack.evaluate(&[], &[]).is_err());
    }

    #[test]
    fn non_class_prediction_is_error() {
        let pack = ClassificationMetricPack::new();
        let err = pack
            .evaluate(&[TypedPrediction::Scalar(1.0)], &[class_target(0)])
            .unwrap_err();
        assert!(matches!(err, TrainerError::Evaluation(_)));
    }
}
