//! Segmentation metric pack — mean IoU and pixel accuracy for dense masks.

use crate::contracts::common::{PluginId, PluginRef};
use crate::contracts::{TypedPrediction, TypedTarget};
use crate::error::TrainerError;
use crate::plugins::{MetricPackPlugin, MetricResult};

/// Dense mask dimensions and label slice extracted from a prediction.
type SegmentationMaskParts<'a> = (u32, u32, &'a [u8]);

/// Dense mask dimensions, label slice, and optional ignore label from a target.
type SegmentationTargetParts<'a> = (u32, u32, &'a [u8], Option<u8>);

/// Computes segmentation metrics from dense mask predictions and targets.
#[derive(Debug, Clone, Default)]
pub struct SegmentationMetricPack;

impl SegmentationMetricPack {
    /// Stable plugin id for this metric pack.
    pub const PLUGIN_ID: &'static str = "segmentation";

    /// Creates a new segmentation metric pack.
    pub fn new() -> Self {
        Self
    }

    fn extract_mask(value: &TypedPrediction) -> Result<SegmentationMaskParts<'_>, TrainerError> {
        match value {
            TypedPrediction::SegmentationMask {
                width,
                height,
                labels,
            } => Ok((*width, *height, labels.as_slice())),
            other => Err(TrainerError::Evaluation(format!(
                "segmentation pack requires SegmentationMask predictions, got {other:?}"
            ))),
        }
    }

    fn extract_target(value: &TypedTarget) -> Result<SegmentationTargetParts<'_>, TrainerError> {
        match value {
            TypedTarget::SegmentationMask {
                width,
                height,
                labels,
                ignore_label,
            } => Ok((*width, *height, labels.as_slice(), *ignore_label)),
            other => Err(TrainerError::Evaluation(format!(
                "segmentation pack requires SegmentationMask targets, got {other:?}"
            ))),
        }
    }
}

impl MetricPackPlugin for SegmentationMetricPack {
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

        let mut correct = 0_u64;
        let mut total = 0_u64;
        let mut true_positive = vec![0_u64; 256];
        let mut false_positive = vec![0_u64; 256];
        let mut false_negative = vec![0_u64; 256];

        for (prediction, target) in predictions.iter().zip(targets.iter()) {
            let (pw, ph, pred) = Self::extract_mask(prediction)?;
            let (tw, th, actual, ignore_label) = Self::extract_target(target)?;
            if pw != tw || ph != th || pred.len() != actual.len() {
                return Err(TrainerError::Evaluation(format!(
                    "mask shape mismatch: prediction {pw}x{ph} vs target {tw}x{th}"
                )));
            }
            for (p, t) in pred.iter().zip(actual.iter()) {
                if ignore_label == Some(*t) {
                    continue;
                }
                total += 1;
                if p == t {
                    correct += 1;
                    true_positive[*p as usize] += 1;
                } else {
                    false_positive[*p as usize] += 1;
                    false_negative[*t as usize] += 1;
                }
            }
        }

        if total == 0 {
            return Err(TrainerError::Evaluation(
                "no labeled pixels after applying ignore_label".to_string(),
            ));
        }

        let pixel_accuracy = correct as f64 / total as f64;
        let mut iou_sum = 0.0_f64;
        let mut iou_count = 0_u64;
        for class_id in 0..256 {
            let tp = true_positive[class_id];
            let fp = false_positive[class_id];
            let fn_count = false_negative[class_id];
            let denom = tp + fp + fn_count;
            if denom == 0 {
                continue;
            }
            iou_sum += tp as f64 / denom as f64;
            iou_count += 1;
        }
        let mean_iou = if iou_count > 0 {
            iou_sum / iou_count as f64
        } else {
            0.0
        };

        Ok(MetricResult {
            metrics: [
                ("pixel_accuracy".to_string(), pixel_accuracy),
                ("mean_iou".to_string(), mean_iou),
            ]
            .into_iter()
            .collect(),
            confusion: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mask(labels: &[u8]) -> TypedPrediction {
        TypedPrediction::SegmentationMask {
            width: 2,
            height: 2,
            labels: labels.to_vec(),
        }
    }

    fn target(labels: &[u8]) -> TypedTarget {
        TypedTarget::SegmentationMask {
            width: 2,
            height: 2,
            labels: labels.to_vec(),
            ignore_label: None,
        }
    }

    #[test]
    fn perfect_mask_scores_one() {
        let pack = SegmentationMetricPack::new();
        let labels = vec![0, 1, 2, 3];
        let result = pack
            .evaluate(&[mask(&labels)], &[target(&labels)])
            .expect("evaluate");
        assert_eq!(result.metrics["pixel_accuracy"], 1.0);
        assert_eq!(result.metrics["mean_iou"], 1.0);
    }

    #[test]
    fn half_wrong_pixel_accuracy() {
        let pack = SegmentationMetricPack::new();
        let result = pack
            .evaluate(&[mask(&[0, 0, 0, 0])], &[target(&[0, 1, 0, 0])])
            .expect("evaluate");
        assert_eq!(result.metrics["pixel_accuracy"], 0.75);
    }
}
