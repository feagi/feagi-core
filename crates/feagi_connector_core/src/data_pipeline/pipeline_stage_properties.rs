//! Configuration properties for pipeline stages.
//!
//! This enum defines all possible pipeline stage configurations.
//! Properties are serializable and can be dynamically updated at runtime.

use std::fmt;
use std::ops::RangeInclusive;
use crate::data_pipeline::pipeline_stage::PipelineStage;
use crate::data_pipeline::stages::{
    ImageFrameProcessorStage,
    ImageFrameSegmentatorStage,
    ImageFrameQuickDiffStage,
    ImagePixelValueCountThresholdStage,
};
use crate::data_types::{ImageFrameProcessor, GazeProperties, ImageFrameSegmentator, Percentage};
use crate::data_types::descriptors::{ImageFrameProperties, SegmentedImageFrameProperties};
use crate::wrapped_io_data::WrappedIOType;

/// Enum representing all possible pipeline stage configurations.
///
/// Each variant contains the specific configuration data needed to create
/// and configure its corresponding pipeline stage.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PipelineStageProperties {
    /// Properties for ImageFrameProcessorStage that configures various image modification transformations
    ImageFrameProcessor {
        transformer_definition: ImageFrameProcessor,
    },

    /// Properties for ImageFrameSegmentatorStage that store configuration for image segmentation
    ImageFrameSegmentator {
        input_image_properties: ImageFrameProperties,
        output_image_properties: SegmentedImageFrameProperties,
        segmentation_gaze: GazeProperties,
    },

    /// Properties for ImageFrameQuickDiffStage that configures quick difference detection
    /// between consecutive image frames.
    ImageQuickDiff {
        per_pixel_allowed_range: RangeInclusive<u8>,
        acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>,
        image_properties: ImageFrameProperties,
    },

    /// Properties for ImagePixelValueCountThresholdStage checks for an image global pixel threshold
    ImagePixelValueCountThreshold {
        input_definition: ImageFrameProperties,
        inclusive_pixel_range: RangeInclusive<u8>,
        acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>,
    },
}

impl PipelineStageProperties {
    /// Returns the data type this stage expects as input.
    pub fn get_input_data_type(&self) -> WrappedIOType {
        match self {
            Self::ImageFrameProcessor { transformer_definition } => {
                WrappedIOType::ImageFrame(Some(*transformer_definition.get_input_image_properties()))
            }
            Self::ImageFrameSegmentator { input_image_properties, .. } => {
                WrappedIOType::ImageFrame(Some(*input_image_properties))
            }
            Self::ImageQuickDiff { image_properties, .. } => {
                WrappedIOType::ImageFrame(Some(*image_properties))
            }
            Self::ImagePixelValueCountThreshold { input_definition, .. } => {
                WrappedIOType::ImageFrame(Some(*input_definition))
            }
        }
    }

    /// Returns the data type this stage produces as output.
    pub fn get_output_data_type(&self) -> WrappedIOType {
        match self {
            Self::ImageFrameProcessor { transformer_definition } => {
                WrappedIOType::ImageFrame(Some(transformer_definition.get_output_image_properties()))
            }
            Self::ImageFrameSegmentator { output_image_properties, .. } => {
                WrappedIOType::SegmentedImageFrame(Some(*output_image_properties))
            }
            Self::ImageQuickDiff { image_properties, .. } => {
                WrappedIOType::ImageFrame(Some(*image_properties))
            }
            Self::ImagePixelValueCountThreshold { input_definition, .. } => {
                WrappedIOType::ImageFrame(Some(*input_definition))
            }
        }
    }

    /// Creates the corresponding pipeline stage from these properties.
    pub fn create_stage(&self) -> Box<dyn PipelineStage> {
        match self {
            Self::ImageFrameProcessor { transformer_definition } => {
                ImageFrameProcessorStage::new_box(transformer_definition.clone()).unwrap()
            }
            Self::ImageFrameSegmentator { input_image_properties, output_image_properties, segmentation_gaze } => {
                ImageFrameSegmentatorStage::new_box(
                    *input_image_properties,
                    *output_image_properties,
                    ImageFrameSegmentator::new(*input_image_properties, *output_image_properties, *segmentation_gaze).unwrap()
                ).unwrap()
            }
            Self::ImageQuickDiff { per_pixel_allowed_range, acceptable_amount_of_activity_in_image, image_properties } => {
                ImageFrameQuickDiffStage::new_box(
                    *image_properties,
                    per_pixel_allowed_range.clone(),
                    acceptable_amount_of_activity_in_image.clone()
                ).unwrap()
            }
            Self::ImagePixelValueCountThreshold { input_definition, inclusive_pixel_range, acceptable_amount_of_activity_in_image } => {
                ImagePixelValueCountThresholdStage::new_box(
                    *input_definition,
                    inclusive_pixel_range.clone(),
                    acceptable_amount_of_activity_in_image.clone(),
                ).unwrap()
            }
        }
    }

    /// Returns the variant name as a string for display purposes
    pub fn variant_name(&self) -> &'static str {
        match self {
            Self::ImageFrameProcessor { .. } => "ImageFrameProcessor",
            Self::ImageFrameSegmentator { .. } => "ImageFrameSegmentator",
            Self::ImageQuickDiff { .. } => "ImageQuickDiff",
            Self::ImagePixelValueCountThreshold { .. } => "ImagePixelValueCountThreshold",
        }
    }
}

impl fmt::Display for PipelineStageProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ImageFrameProcessor { transformer_definition } => {
                write!(f, "ImageFrameProcessor(transformer: {:?})", transformer_definition)
            }
            Self::ImageFrameSegmentator { input_image_properties, output_image_properties, segmentation_gaze } => {
                write!(f, "ImageFrameSegmentator(input: {:?}, output: {:?}, gaze: {:?})", 
                    input_image_properties, output_image_properties, segmentation_gaze)
            }
            Self::ImageQuickDiff { per_pixel_allowed_range, acceptable_amount_of_activity_in_image, image_properties } => {
                write!(f, "ImageQuickDiff(pixel_range: {:?}, activity: {:?}, image: {:?})",
                    per_pixel_allowed_range, acceptable_amount_of_activity_in_image, image_properties)
            }
            Self::ImagePixelValueCountThreshold { input_definition, inclusive_pixel_range, acceptable_amount_of_activity_in_image } => {
                write!(f, "ImagePixelValueCountThreshold(input: {:?}, pixel_range: {:?}, activity: {:?})",
                    input_definition, inclusive_pixel_range, acceptable_amount_of_activity_in_image)
            }
        }
    }
}

// Convenience constructors for each variant
impl PipelineStageProperties {
    /// Creates ImageFrameProcessor properties
    pub fn new_image_frame_processor(transformer_definition: ImageFrameProcessor) -> Self {
        Self::ImageFrameProcessor { transformer_definition }
    }

    /// Creates ImageFrameSegmentator properties
    pub fn new_image_frame_segmentator(
        input_image_properties: ImageFrameProperties,
        output_image_properties: SegmentedImageFrameProperties,
        segmentation_gaze: GazeProperties,
    ) -> Self {
        Self::ImageFrameSegmentator {
            input_image_properties,
            output_image_properties,
            segmentation_gaze,
        }
    }

    /// Creates ImageQuickDiff properties
    pub fn new_image_quick_diff(
        per_pixel_allowed_range: RangeInclusive<u8>,
        acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>,
        image_properties: ImageFrameProperties,
    ) -> Self {
        Self::ImageQuickDiff {
            per_pixel_allowed_range,
            acceptable_amount_of_activity_in_image,
            image_properties,
        }
    }

    /// Creates ImagePixelValueCountThreshold properties
    pub fn new_image_pixel_value_count_threshold(
        input_definition: ImageFrameProperties,
        inclusive_pixel_range: RangeInclusive<u8>,
        acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>,
    ) -> Self {
        Self::ImagePixelValueCountThreshold {
            input_definition,
            inclusive_pixel_range,
            acceptable_amount_of_activity_in_image,
        }
    }
}
