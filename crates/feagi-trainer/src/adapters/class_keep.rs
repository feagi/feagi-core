//! Deterministic per-class keep quotas for imbalanced training sets.

use std::collections::BTreeMap;

use crate::contracts::{IRSample, TypedTarget};
use crate::error::TrainerError;

/// Rejects unknown labels and percents outside 0..=100.
///
/// A missing label keeps 100% of that class. An empty map keeps every sample.
pub fn validate_class_keep_percents(
    percents: &BTreeMap<String, u32>,
    class_labels: &[String],
) -> Result<(), String> {
    if percents.is_empty() {
        return Ok(());
    }
    for (key, percent) in percents {
        if !class_labels.iter().any(|label| label == key) {
            return Err(format!("class_keep_percents has unknown label '{key}'"));
        }
        if *percent > 100 {
            return Err(format!(
                "class_keep_percents '{key}' is {percent}; must be 0..=100"
            ));
        }
    }
    Ok(())
}

/// `floor(total * percent / 100)`. Errors when a positive percent would keep zero.
pub fn class_keep_count(total: u64, percent: u32) -> Result<u64, String> {
    if percent > 100 {
        return Err(format!("class keep percent {percent} is outside 0..=100"));
    }
    let keep = total.saturating_mul(u64::from(percent)) / 100;
    if percent > 0 && total > 0 && keep == 0 {
        return Err(format!(
            "class keep {percent}% of {total} samples is 0; raise the percent"
        ));
    }
    Ok(keep)
}

/// Keeps the first `keep_count` samples of each class in encounter order.
pub fn apply_class_keep_percents(
    samples: Vec<IRSample>,
    percents: &BTreeMap<String, u32>,
    class_labels: &[String],
) -> Result<Vec<IRSample>, TrainerError> {
    if percents.is_empty() {
        return Ok(samples);
    }
    validate_class_keep_percents(percents, class_labels).map_err(TrainerError::Config)?;
    let mut totals: BTreeMap<String, u64> = BTreeMap::new();
    for sample in &samples {
        let label = class_label(sample)?;
        *totals.entry(label).or_insert(0) += 1;
    }
    let mut remaining: BTreeMap<String, u64> = BTreeMap::new();
    for label in class_labels {
        let total = totals.get(label).copied().unwrap_or(0);
        let percent = percents.get(label).copied().unwrap_or(100);
        remaining.insert(
            label.clone(),
            class_keep_count(total, percent).map_err(TrainerError::Config)?,
        );
    }
    let mut kept = Vec::new();
    for sample in samples {
        let label = class_label(&sample)?;
        let left = remaining.get_mut(&label).ok_or_else(|| {
            TrainerError::Config(format!(
                "class keep saw unlabeled class '{label}' not in class_labels"
            ))
        })?;
        if *left == 0 {
            continue;
        }
        *left -= 1;
        kept.push(sample);
    }
    if kept.is_empty() {
        return Err(TrainerError::Parse(
            "class keep percents removed every sample".to_string(),
        ));
    }
    Ok(kept)
}

fn class_label(sample: &IRSample) -> Result<String, TrainerError> {
    match &sample.target {
        Some(TypedTarget::Class {
            label: Some(label), ..
        }) => Ok(label.clone()),
        Some(TypedTarget::Class {
            class_id,
            label: None,
        }) => Err(TrainerError::Parse(format!(
            "class keep requires a class label on sample (class_id {class_id})"
        ))),
        other => Err(TrainerError::Parse(format!(
            "class keep requires a class target, got {other:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::common::{DatasetVersionId, Modality, OutputType, SampleId, Split};
    use crate::contracts::ir_sample::{Payload, SCHEMA_VERSION};

    fn sample(label: &str, class_id: u32) -> IRSample {
        IRSample {
            schema_version: SCHEMA_VERSION,
            sample_id: SampleId(label.to_string()),
            dataset_version_id: DatasetVersionId("t@1".to_string()),
            split: Split::Train,
            modality: Modality::Tabular,
            payload: Payload::Tabular(vec![0.0]),
            target: Some(TypedTarget::Class {
                class_id,
                label: Some(label.to_string()),
            }),
            output_type: OutputType::Class,
            coordinate_frame: None,
            timestamp: None,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn keeps_first_windows_in_order() {
        let samples = vec![
            sample("N", 0),
            sample("N", 0),
            sample("V", 1),
            sample("N", 0),
            sample("V", 1),
        ];
        let mut percents = BTreeMap::new();
        percents.insert("N".to_string(), 50);
        percents.insert("V".to_string(), 100);
        let kept =
            apply_class_keep_percents(samples, &percents, &["N".to_string(), "V".to_string()])
                .expect("keep");
        let labels: Vec<_> = kept
            .iter()
            .map(|sample| class_label(sample).expect("label"))
            .collect();
        assert_eq!(labels, vec!["N", "V", "V"]);
    }

    #[test]
    fn empty_percents_keep_all() {
        let samples = vec![sample("N", 0), sample("V", 1)];
        let kept = apply_class_keep_percents(
            samples.clone(),
            &BTreeMap::new(),
            &["N".to_string(), "V".to_string()],
        )
        .expect("keep");
        assert_eq!(kept.len(), 2);
    }

    #[test]
    fn positive_percent_of_small_class_is_error() {
        let err = class_keep_count(3, 10).expect_err("zero keep");
        assert!(err.contains("is 0"));
    }

    #[test]
    fn omitted_label_keeps_all_of_that_class() {
        let samples = vec![sample("N", 0), sample("N", 0), sample("V", 1)];
        let mut percents = BTreeMap::new();
        percents.insert("N".to_string(), 50);
        let kept =
            apply_class_keep_percents(samples, &percents, &["N".to_string(), "V".to_string()])
                .expect("keep");
        let labels: Vec<_> = kept
            .iter()
            .map(|sample| class_label(sample).expect("label"))
            .collect();
        assert_eq!(labels, vec!["N", "V"]);
    }
}
