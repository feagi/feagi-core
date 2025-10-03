//! Identity processing that pass data through unchanged.
//!
//! This module provides "pass-through" processing that implement the StreamCacheProcessor
//! interface but don't modify the data in any way. As at least 1 processor is required when
//! adding channels, these are useful if the user does not wish to transform the data

use std::any::Any;
use std::fmt::{Display, Formatter};
use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use crate::data_pipeline::pipeline_stage::PipelineStage;
use crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties;
use crate::data_pipeline::stage_properties::IdentityStageProperties;
use crate::data_types::{ImageFrame, Percentage, Percentage4D, SegmentedImageFrame, SignedPercentage};
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

/// A stream processor that passes data through unchanged.
#[derive(Debug, Clone)]
pub struct IdentityStage {
    previous_value: WrappedIOData,
    expected_data_variant: WrappedIOType,
}

impl Display for IdentityStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdentityStage({:?})", self.previous_value)
    }
}

impl PipelineStage for IdentityStage {
    fn get_input_data_type(&self) -> WrappedIOType {
        self.expected_data_variant
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        self.expected_data_variant
    }

    fn get_most_recent_output(&self) -> &WrappedIOData {
        &self.previous_value
    }

    fn process_new_input(&mut self, value: &WrappedIOData, _: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        self.previous_value = value.clone(); // does nothing lol
        Ok(&self.previous_value)
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn create_properties(&self) -> Box<dyn PipelineStageProperties> {
        IdentityStageProperties::new_box(self.expected_data_variant).unwrap()
    }

    fn load_properties(&mut self, properties: Box<dyn PipelineStageProperties>) -> Result<(), FeagiDataError> {
        todo!()
    }
}

impl IdentityStage {

    pub fn new(identity_type: WrappedIOType) -> Result<Self, FeagiDataError> {

        Ok(IdentityStage{
            previous_value: identity_type.create_blank_data_of_type()?,
            expected_data_variant: identity_type,
        })
    }

    pub fn new_box(identity_type: WrappedIOType) -> Result<Box<dyn PipelineStage + 'static>, FeagiDataError> {
        Ok(
            Box::new(IdentityStage::new(identity_type)?)
        )
    }
}
