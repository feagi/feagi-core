//! Classification metric pack — accuracy, macro precision/recall/F1, and confusion matrix.
//!
//! Operates on single-label class predictions and targets. Aggregates are computed over the
//! union of target class ids and answered prediction class ids; macro averages weight every
//! class equally. A readout with no class (silent) or a tie for the top score is wrong for its
//! true class, counted in that class's total, and never a false positive for any class.

use std::collections::BTreeMap;

use crate::contracts::common::{PluginId, PluginRef};
use crate::contracts::{
    ClassBreakdown, ClassReadout, ConfusionSummary, TypedPrediction, TypedTarget,
};
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

    /// Extracts the readout, erroring on a non-class variant.
    fn predicted_readout(prediction: &TypedPrediction) -> Result<ClassReadout, TrainerError> {
        match prediction {
            TypedPrediction::Class { readout, .. } => Ok(*readout),
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

/// First recorded target label per class id.
fn target_labels(targets: &[TypedTarget]) -> BTreeMap<u32, String> {
    let mut labels: BTreeMap<u32, String> = BTreeMap::new();
    for target in targets {
        if let TypedTarget::Class {
            class_id,
            label: Some(label),
        } = target
        {
            if label.is_empty() {
                continue;
            }
            labels.entry(*class_id).or_insert_with(|| label.clone());
        }
    }
    labels
}

/// Sparse, publishable form of a confusion matrix. Labels are the first target label of each
/// class, or the class id text.
pub fn confusion_summary(matrix: &ConfusionMatrix, targets: &[TypedTarget]) -> ConfusionSummary {
    let labels = target_labels(targets);
    let mut cells = Vec::new();
    for (true_index, row) in matrix.counts.iter().enumerate() {
        for (answered_index, &count) in row.iter().enumerate() {
            if count > 0 {
                cells.push([true_index as u64, answered_index as u64, count]);
            }
        }
    }
    ConfusionSummary {
        class_ids: matrix.class_ids.clone(),
        labels: matrix
            .class_ids
            .iter()
            .map(|id| labels.get(id).cloned().unwrap_or_else(|| id.to_string()))
            .collect(),
        cells,
        no_answer: matrix.no_answer.clone(),
        tied: matrix.tied.clone(),
    }
}

/// Per-class correct counts for the results view.
///
/// A class is included only when at least one scored target used it. The label is the first
/// target label recorded for that class id. When a target has no label, the class id text is used.
pub fn class_breakdown_rows(
    matrix: &ConfusionMatrix,
    targets: &[TypedTarget],
) -> Vec<ClassBreakdown> {
    let labels = target_labels(targets);
    matrix
        .class_ids
        .iter()
        .enumerate()
        .filter_map(|(index, class_id)| {
            let no_answer = matrix.no_answer.get(index).copied().unwrap_or(0);
            let tied = matrix.tied.get(index).copied().unwrap_or(0);
            let total: u64 = matrix.counts[index].iter().sum::<u64>() + no_answer + tied;
            if total == 0 {
                return None;
            }
            let correct = matrix.counts[index][index];
            Some(ClassBreakdown {
                class_id: *class_id,
                label: labels
                    .get(class_id)
                    .cloned()
                    .unwrap_or_else(|| class_id.to_string()),
                correct,
                total,
                no_answer,
                tied,
            })
        })
        .collect()
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

        let predicted: Vec<ClassReadout> = predictions
            .iter()
            .map(Self::predicted_readout)
            .collect::<Result<_, _>>()?;
        let actual: Vec<u32> = targets
            .iter()
            .map(Self::target_class)
            .collect::<Result<_, _>>()?;

        // Sorted union of target classes and answered prediction classes defines the axes.
        let mut class_ids: Vec<u32> = predicted
            .iter()
            .filter_map(ClassReadout::class_id)
            .chain(actual.iter().copied())
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
        let mut no_answer = vec![0u64; n];
        let mut tied = vec![0u64; n];
        let mut correct = 0u64;
        for (p, a) in predicted.iter().zip(actual.iter()) {
            let ai = index_of[a];
            match p {
                ClassReadout::Class(p) => {
                    counts[ai][index_of[p]] += 1;
                    if p == a {
                        correct += 1;
                    }
                }
                ClassReadout::NoAnswer => no_answer[ai] += 1,
                ClassReadout::Tied => tied[ai] += 1,
            }
        }

        let total = predicted.len() as f64;
        let accuracy = correct as f64 / total;

        // Per-class precision/recall/F1 from the confusion matrix.
        let mut per_class: Vec<ClassMetrics> = Vec::with_capacity(n);
        for (i, row) in counts.iter().enumerate() {
            let tp = row[i] as f64;
            let predicted_positives: u64 = counts.iter().map(|r| r[i]).sum();
            let actual_positives: u64 = row.iter().sum::<u64>() + no_answer[i] + tied[i];

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
            confusion: Some(ConfusionMatrix {
                class_ids,
                counts,
                no_answer,
                tied,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class_pred(id: u32) -> TypedPrediction {
        TypedPrediction::Class {
            readout: ClassReadout::Class(id),
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
        let rows = class_breakdown_rows(&cm, &targets);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].label, "0");
        assert_eq!(rows[0].correct, 1);
        assert_eq!(rows[0].total, 2);
        assert_eq!(rows[1].label, "1");
        assert_eq!(rows[1].correct, 1);
        assert_eq!(rows[1].total, 1);
    }

    #[test]
    fn class_breakdown_uses_target_labels() {
        let pack = ClassificationMetricPack::new();
        let preds = vec![class_pred(0), class_pred(3)];
        let targets = vec![
            TypedTarget::Class {
                class_id: 0,
                label: Some("cat".to_string()),
            },
            TypedTarget::Class {
                class_id: 3,
                label: Some("dog".to_string()),
            },
        ];
        let result = pack.evaluate(&preds, &targets).expect("evaluate");
        let rows = class_breakdown_rows(result.confusion.as_ref().expect("confusion"), &targets);
        assert_eq!(rows[0].label, "cat");
        assert_eq!(rows[0].correct, 1);
        assert_eq!(rows[0].total, 1);
        assert_eq!(rows[1].class_id, 3);
        assert_eq!(rows[1].label, "dog");
    }

    fn silent_pred() -> TypedPrediction {
        TypedPrediction::Class {
            readout: ClassReadout::NoAnswer,
            scores: vec![0.0; 10],
        }
    }

    #[test]
    fn all_silent_readout_scores_zero_with_no_answer_rows() {
        let pack = ClassificationMetricPack::new();
        let mut preds = Vec::new();
        let mut targets = Vec::new();
        for class in 0..10u32 {
            for _ in 0..10 {
                preds.push(silent_pred());
                targets.push(class_target(class));
            }
        }
        let result = pack.evaluate(&preds, &targets).expect("evaluate");
        for key in ["accuracy", "macro_precision", "macro_recall", "macro_f1"] {
            assert_eq!(result.metrics[key], 0.0, "{key}");
        }
        let rows = class_breakdown_rows(result.confusion.as_ref().expect("confusion"), &targets);
        assert_eq!(rows.len(), 10);
        for row in &rows {
            assert_eq!((row.correct, row.total, row.no_answer), (0, 10, 10));
        }
        assert_eq!(rows.iter().map(|r| r.total).sum::<u64>(), 100);
    }

    #[test]
    fn no_answer_counts_in_recall_but_not_precision() {
        let pack = ClassificationMetricPack::new();
        // class 0: one right, one silent; class 1: one right.
        let preds = vec![class_pred(0), silent_pred(), class_pred(1)];
        let targets = vec![class_target(0), class_target(0), class_target(1)];
        let result = pack.evaluate(&preds, &targets).expect("evaluate");
        assert!((result.metrics["accuracy"] - 2.0 / 3.0).abs() < 1e-12);
        // recall: class 0 = 1/2, class 1 = 1; precision unchanged: both 1.
        assert!((result.metrics["macro_recall"] - 0.75).abs() < 1e-12);
        assert_eq!(result.metrics["macro_precision"], 1.0);
        let cm = result.confusion.expect("confusion");
        assert_eq!(cm.no_answer, vec![1, 0]);
    }

    #[test]
    fn tied_counts_as_wrong_in_accuracy_and_recall_but_not_precision() {
        let pack = ClassificationMetricPack::new();
        let tied = || TypedPrediction::Class {
            readout: ClassReadout::Tied,
            scores: vec![0.5, 0.5],
        };
        let mut preds = Vec::new();
        let mut targets = Vec::new();
        // Class 0: 6 right, 1 answered 1, 2 tied, 1 silent.
        for _ in 0..6 {
            preds.push(class_pred(0));
            targets.push(class_target(0));
        }
        preds.push(class_pred(1));
        targets.push(class_target(0));
        for _ in 0..2 {
            preds.push(tied());
            targets.push(class_target(0));
        }
        preds.push(silent_pred());
        targets.push(class_target(0));
        // Class 1: 10 right.
        for _ in 0..10 {
            preds.push(class_pred(1));
            targets.push(class_target(1));
        }
        let result = pack.evaluate(&preds, &targets).expect("evaluate");
        assert!((result.metrics["accuracy"] - 0.80).abs() < 1e-12);
        let cm = result.confusion.expect("confusion");
        assert_eq!(cm.tied, vec![2, 0]);
        assert_eq!(cm.no_answer, vec![1, 0]);
        let row_sum = |i: usize| cm.counts[i].iter().sum::<u64>() + cm.no_answer[i] + cm.tied[i];
        assert_eq!((row_sum(0), row_sum(1)), (10, 10));
        // recall(0) = 6/10, precision(0) = 6/6, precision(1) = 10/11.
        let macro_recall = (0.6 + 1.0) / 2.0;
        assert!((result.metrics["macro_recall"] - macro_recall).abs() < 1e-12);
        let macro_precision = (1.0 + 10.0 / 11.0) / 2.0;
        assert!((result.metrics["macro_precision"] - macro_precision).abs() < 1e-12);
        let rows = class_breakdown_rows(&cm, &targets);
        assert_eq!((rows[0].total, rows[0].tied, rows[0].no_answer), (10, 2, 1));
        let summary = confusion_summary(&cm, &targets);
        assert_eq!(summary.cells, vec![[0, 0, 6], [0, 1, 1], [1, 1, 10]]);
        assert_eq!(summary.tied, vec![2, 0]);
        assert_eq!(summary.labels, vec!["0".to_string(), "1".to_string()]);
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
