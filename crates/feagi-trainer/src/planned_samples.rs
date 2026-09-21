//! Planned visits over a split: tables/ECG stay in memory; image folders keep paths until visit.

use crate::adapters::image_folder_segmentation::{ImageFolderSegmentationAdapter, ImageLabelPair};
use crate::contracts::{DatasetVersionId, IRSample};
use crate::error::TrainerError;

/// Supplies one [`IRSample`] per sampler visit.
pub trait SampleVisit {
    /// Number of visits the sampler scheduled.
    fn visit_count(&self) -> usize;

    /// Loads the sample for zero-based visit `index` in sampler order.
    fn load_visit(&self, index: usize) -> Result<IRSample, TrainerError>;
}

/// Indexed sample list produced by [`crate::run_config::RunConfig::plan`].
pub struct PlannedSamples {
    store: SampleStore,
}

enum SampleStore {
    Memory(Vec<IRSample>),
    ImageFolder {
        adapter: Box<ImageFolderSegmentationAdapter>,
        pairs: Vec<ImageLabelPair>,
        dataset_version_id: DatasetVersionId,
        order: Vec<usize>,
    },
}

impl PlannedSamples {
    /// Samples already in sampler order (tables and ECG).
    pub fn from_ordered(samples: Vec<IRSample>) -> Self {
        Self {
            store: SampleStore::Memory(samples),
        }
    }

    /// Image/mask paths plus visit order. Pixels are decoded in [`Self::load_visit`].
    pub fn from_image_folder(
        adapter: ImageFolderSegmentationAdapter,
        pairs: Vec<ImageLabelPair>,
        dataset_version_id: DatasetVersionId,
        order: Vec<usize>,
    ) -> Result<Self, TrainerError> {
        for (visit, source_index) in order.iter().enumerate() {
            if *source_index >= pairs.len() {
                return Err(TrainerError::Config(format!(
                    "sampler visit {visit} indexes pair {source_index} past {} pairs",
                    pairs.len()
                )));
            }
        }
        Ok(Self {
            store: SampleStore::ImageFolder {
                adapter: Box::new(adapter),
                pairs,
                dataset_version_id,
                order,
            },
        })
    }

    /// Visit count scheduled for this plan.
    pub fn len(&self) -> usize {
        self.visit_count()
    }

    /// True when the sampler scheduled no visits.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Materialized slice in visit order. Image-folder plans have no slice.
    pub fn as_ordered_slice(&self) -> Result<&[IRSample], TrainerError> {
        match &self.store {
            SampleStore::Memory(samples) => Ok(samples),
            SampleStore::ImageFolder { .. } => Err(TrainerError::Config(
                "image-folder plans load one sample per visit and have no materialized slice"
                    .to_string(),
            )),
        }
    }
}

impl SampleVisit for PlannedSamples {
    fn visit_count(&self) -> usize {
        match &self.store {
            SampleStore::Memory(samples) => samples.len(),
            SampleStore::ImageFolder { order, .. } => order.len(),
        }
    }

    fn load_visit(&self, index: usize) -> Result<IRSample, TrainerError> {
        match &self.store {
            SampleStore::Memory(samples) => samples.get(index).cloned().ok_or_else(|| {
                TrainerError::Config(format!(
                    "visit {index} is outside {} planned samples",
                    samples.len()
                ))
            }),
            SampleStore::ImageFolder {
                adapter,
                pairs,
                dataset_version_id,
                order,
            } => {
                let source_index = *order.get(index).ok_or_else(|| {
                    TrainerError::Config(format!(
                        "visit {index} is outside {} planned samples",
                        order.len()
                    ))
                })?;
                adapter.load_indexed_sample(&pairs[source_index], source_index, dataset_version_id)
            }
        }
    }
}

impl SampleVisit for [IRSample] {
    fn visit_count(&self) -> usize {
        self.len()
    }

    fn load_visit(&self, index: usize) -> Result<IRSample, TrainerError> {
        self.get(index).cloned().ok_or_else(|| {
            TrainerError::Config(format!(
                "visit {index} is outside {} planned samples",
                self.len()
            ))
        })
    }
}

impl SampleVisit for Vec<IRSample> {
    fn visit_count(&self) -> usize {
        self.len()
    }

    fn load_visit(&self, index: usize) -> Result<IRSample, TrainerError> {
        self.as_slice().load_visit(index)
    }
}

impl<const N: usize> SampleVisit for [IRSample; N] {
    fn visit_count(&self) -> usize {
        N
    }

    fn load_visit(&self, index: usize) -> Result<IRSample, TrainerError> {
        self.as_slice().load_visit(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::common::{DatasetVersionId, Modality, OutputType, SampleId, Split};
    use crate::contracts::ir_sample::{Payload, SCHEMA_VERSION};
    use std::collections::BTreeMap;

    fn tabular(id: &str) -> IRSample {
        IRSample {
            schema_version: SCHEMA_VERSION,
            sample_id: SampleId(id.to_string()),
            dataset_version_id: DatasetVersionId("d".to_string()),
            split: Split::Train,
            modality: Modality::Tabular,
            payload: Payload::Tabular(vec![1.0]),
            target: None,
            output_type: OutputType::Class,
            coordinate_frame: None,
            timestamp: None,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn memory_plan_loads_visit_in_order() {
        let planned = PlannedSamples::from_ordered(vec![tabular("a"), tabular("b")]);
        assert_eq!(planned.len(), 2);
        assert_eq!(planned.load_visit(0).expect("a").sample_id.0, "a");
        assert_eq!(planned.load_visit(1).expect("b").sample_id.0, "b");
        assert!(planned.as_ordered_slice().expect("slice").len() == 2);
    }

    #[test]
    fn memory_plan_rejects_out_of_range_visit() {
        let planned = PlannedSamples::from_ordered(vec![tabular("a")]);
        let err = planned.load_visit(1).expect_err("oob");
        assert!(err.to_string().contains("visit 1"));
    }
}
