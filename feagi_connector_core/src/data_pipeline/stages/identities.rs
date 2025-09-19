//! Identity processing that pass data through unchanged.
//!
//! This module provides "pass-through" processing that implement the StreamCacheProcessor
//! interface but don't modify the data in any way. As at least 1 processor is required when
//! adding channels, these are useful if the user does not wish to transform the data

use std::fmt::{Display, Formatter};
use std::time::Instant;
use feagi_data_structures::data::{ImageFrame, Percentage, Percentage4D, SegmentedImageFrame, SignedPercentage};
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::data_pipeline::pipeline_stage::PipelineStage;

//region Identity Float
/// A stream processor that passes float values through unchanged.
#[derive(Debug, Clone)]
pub struct IdentityFloatStage {
    previous_value: WrappedIOData,
}

impl Display for IdentityFloatStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdentityFloatProcessor({:?})", self.previous_value)
    }
}

impl PipelineStage for IdentityFloatStage {
    fn get_input_data_type(&self) -> WrappedIOType {
        WrappedIOType::F32
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        WrappedIOType::F32
    }

    fn get_most_recent_output(&self) -> &WrappedIOData {
        &self.previous_value
    }
    
    /// Process new input and store it unchanged.
    fn process_new_input(&mut self, value: &WrappedIOData, _: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        self.previous_value = value.clone();
        Ok(&self.previous_value)
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }
}

impl IdentityFloatStage {
    /// Creates a new IdentityFloatProcessor.
    ///
    /// # Arguments
    /// * `initial_value` - The initial float value to store (must be finite)
    ///
    /// # Returns
    /// * `Ok(IdentityFloatProcessor)` - A new processor instance
    /// * `Err(FeagiDataError)` - If initial_value is invalid (NaN/infinite)
    pub fn new(initial_value: f32) -> Result<Self, FeagiDataError> {
        if initial_value.is_nan() || initial_value.is_infinite() {
            return Err(FeagiDataError::BadParameters(format!("Given float {} is not valid!", initial_value)));
        }
        Ok(IdentityFloatStage {
            previous_value: WrappedIOData::F32(initial_value),
        })
    }
}
//endregion

//region Identity Percentage

#[derive(Debug, Clone)]
pub struct IdentityPercentageStage {
    previous_value: WrappedIOData,
}

impl Display for IdentityPercentageStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdentityPercentageStage({:?})", self.previous_value)
    }
}

impl PipelineStage for IdentityPercentageStage {
    fn get_input_data_type(&self) -> WrappedIOType {
        WrappedIOType::Percentage
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        WrappedIOType::Percentage
    }

    fn get_most_recent_output(&self) -> &WrappedIOData {
        &self.previous_value
    }

    /// Process new input and store it unchanged.
    fn process_new_input(&mut self, value: &WrappedIOData, _: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        self.previous_value = value.clone();
        Ok(&self.previous_value)
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }
}

impl IdentityPercentageStage {
    /// Creates a new IdentityFloatProcessor.
    ///
    /// # Arguments
    /// * `initial_value` - The initial float value to store (must be finite)
    ///
    /// # Returns
    /// * `Ok(IdentityFloatProcessor)` - A new processor instance
    /// * `Err(FeagiDataError)` - If initial_value is invalid (NaN/infinite)
    pub fn new(initial_value: Percentage) -> Result<Self, FeagiDataError> {
        Ok(IdentityPercentageStage {
            previous_value: WrappedIOData::Percentage(initial_value),
        })
    }
}

//endregion

//region Identity Signed Percentage

#[derive(Debug, Clone)]
pub struct IdentitySignedPercentageStage {
    previous_value: WrappedIOData,
}

impl Display for IdentitySignedPercentageStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdentitySignedPercentageStage({:?})", self.previous_value)
    }
}

impl PipelineStage for IdentitySignedPercentageStage {
    fn get_input_data_type(&self) -> WrappedIOType {
        WrappedIOType::SignedPercentage
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        WrappedIOType::SignedPercentage
    }

    fn get_most_recent_output(&self) -> &WrappedIOData {
        &self.previous_value
    }

    /// Process new input and store it unchanged.
    fn process_new_input(&mut self, value: &WrappedIOData, _: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        self.previous_value = value.clone();
        Ok(&self.previous_value)
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }
}

impl IdentitySignedPercentageStage {
    /// Creates a new IdentityFloatProcessor.
    ///
    /// # Arguments
    /// * `initial_value` - The initial float value to store (must be finite)
    ///
    /// # Returns
    /// * `Ok(IdentityFloatProcessor)` - A new processor instance
    /// * `Err(FeagiDataError)` - If initial_value is invalid (NaN/infinite)
    pub fn new(initial_value: SignedPercentage) -> Result<Self, FeagiDataError> {
        Ok(IdentitySignedPercentageStage {
            previous_value: WrappedIOData::SignedPercentage(initial_value),
        })
    }
}

//endregion

//region Identity Percentage4D

#[derive(Debug, Clone)]
pub struct IdentityPercentage4DStage {
    previous_value: WrappedIOData,
}

impl Display for IdentityPercentage4DStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdentityPercentage4DStage({:?})", self.previous_value)
    }
}

impl PipelineStage for IdentityPercentage4DStage {
    fn get_input_data_type(&self) -> WrappedIOType {
        WrappedIOType::Percentage4D
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        WrappedIOType::Percentage4D
    }

    fn get_most_recent_output(&self) -> &WrappedIOData {
        &self.previous_value
    }

    /// Process new input and store it unchanged.
    fn process_new_input(&mut self, value: &WrappedIOData, _: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        self.previous_value = value.clone();
        Ok(&self.previous_value)
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }
}

impl IdentityPercentage4DStage {
    /// Creates a new IdentityFloatProcessor.
    ///
    /// # Arguments
    /// * `initial_value` - The initial float value to store (must be finite)
    ///
    /// # Returns
    /// * `Ok(IdentityFloatProcessor)` - A new processor instance
    /// * `Err(FeagiDataError)` - If initial_value is invalid (NaN/infinite)
    pub fn new(initial_value: Percentage4D) -> Result<Self, FeagiDataError> {
        Ok(IdentityPercentage4DStage {
            previous_value: WrappedIOData::Percentage4D(initial_value),
        })
    }
}

//endregion

//region Identity Image Frame
/// A stream processor that passes image frames through unchanged.
#[derive(Debug, Clone)]
pub struct IdentityImageFrameStage {
    previous_value: WrappedIOData,
    expected_image_variant: WrappedIOType,  // WrappedIOType::ImageFrame(ImageFrameProperties)
}

impl Display for IdentityImageFrameStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdentityImageFrameProcessor({:?})", self.previous_value)
    }
}

impl PipelineStage for IdentityImageFrameStage {
    fn get_input_data_type(&self) -> WrappedIOType {
        self.expected_image_variant
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        self.expected_image_variant
    }

    fn get_most_recent_output(&self) -> &WrappedIOData {
        &self.previous_value
    }

    fn process_new_input(&mut self, value: &WrappedIOData, _: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        self.previous_value = value.clone();
        Ok(&self.previous_value)
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }
}

impl IdentityImageFrameStage {
    /// Creates a new IdentityImageFrameProcessor.
    ///
    /// # Arguments
    /// * `initial_image` - The initial ImageFrame to store
    ///
    /// # Returns
    /// * `Ok(IdentityImageFrameProcessor)` - A new processor instance
    pub fn new(initial_image: ImageFrame) -> Result<Self, FeagiDataError> {
        Ok(IdentityImageFrameStage {
            expected_image_variant: WrappedIOType::ImageFrame(Some(initial_image.get_image_frame_properties())),
            previous_value: WrappedIOData::ImageFrame(initial_image),
        })
    }
}
//endregion

//region Identity Segmented Image Frame
/// A stream processor that passes segmented image frames through unchanged.
#[derive(Debug, Clone)]
pub struct IdentitySegmentedImageFrameStage {
    previous_value: WrappedIOData,
    expected_segmented_image_variant: WrappedIOType,  // WrappedIOType::SegmentedImageFrame(Some([ImageFrameProperties ;9]))
}

impl Display for IdentitySegmentedImageFrameStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdentitySegmentedImageFrameProcessor({:?})", self.previous_value)
    }
}

impl PipelineStage for IdentitySegmentedImageFrameStage {
    fn get_input_data_type(&self) -> WrappedIOType {
        self.expected_segmented_image_variant
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        self.expected_segmented_image_variant
    }

    fn get_most_recent_output(&self) -> &WrappedIOData {
        &self.previous_value
    }

    fn process_new_input(&mut self, value: &WrappedIOData, _: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        self.previous_value = value.clone();
        Ok(&self.previous_value)
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }
}

impl IdentitySegmentedImageFrameStage {
    /// Creates a new IdentitySegmentedImageFrameProcessor.
    ///
    /// # Arguments
    /// * `initial_segmented_image` - The initial SegmentedImageFrame to store
    ///
    /// # Returns
    /// * `Ok(IdentitySegmentedImageFrameProcessor)` - A new processor instance
    pub fn new(initial_segmented_image: SegmentedImageFrame) -> Result<Self, FeagiDataError> {
        Ok(IdentitySegmentedImageFrameStage {
            expected_segmented_image_variant: WrappedIOType::SegmentedImageFrame(Some(initial_segmented_image.get_segmented_image_frame_properties())),
            previous_value: WrappedIOData::SegmentedImageFrame(initial_segmented_image),
        })
    }
}
//endregion

