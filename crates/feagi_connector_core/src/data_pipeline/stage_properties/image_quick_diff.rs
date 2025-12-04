use std::any::Any;
use std::fmt::{Display, Formatter};
use std::ops::RangeInclusive;
use feagi_data_structures::FeagiDataError;
use crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties;
use crate::data_pipeline::PipelineStage;
use crate::data_pipeline::stages::ImageFrameQuickDiffStage;
use crate::data_types::descriptors::ImageFrameProperties;
use crate::data_types::Percentage;
use crate::pipeline_stage_property_implementations;
use crate::wrapped_io_data::WrappedIOType;

#[derive(Debug, Clone)]
pub struct ImageQuickDiffStageProperties {
    pub per_pixel_allowed_range: RangeInclusive<u8>,
    pub acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>,
    pub image_properties: ImageFrameProperties
}


impl PipelineStageProperties for ImageQuickDiffStageProperties {
    fn get_input_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.image_properties))
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.image_properties))
    }

    fn clone_box(&self) -> Box<dyn PipelineStageProperties + Sync + Send> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn create_stage(&self) -> Box<dyn PipelineStage> {
        ImageFrameQuickDiffStage::new_box(self.image_properties, self.per_pixel_allowed_range.clone(), self.acceptable_amount_of_activity_in_image.clone()).unwrap()
    }
}

impl ImageQuickDiffStageProperties {
    pub fn new(per_pixel_allowed_range: RangeInclusive<u8>, acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>, image_properties: ImageFrameProperties) -> Self {
        ImageQuickDiffStageProperties {
            per_pixel_allowed_range,
            acceptable_amount_of_activity_in_image,
            image_properties
        }
    }

    pub fn new_box(per_pixel_allowed_range: RangeInclusive<u8>, acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>, image_properties: ImageFrameProperties) -> Box<dyn PipelineStageProperties + Send + Sync> {
        Box::new(ImageQuickDiffStageProperties::new(per_pixel_allowed_range, acceptable_amount_of_activity_in_image, image_properties))
    }
}

pipeline_stage_property_implementations!(
    ImageQuickDiffStageProperties,
    "ImageQuickDiffStageProperties(per pixel allow range: {:?}, acceptable_amount_of_activity_in_image: {:?}, image_properties: {:?})",
    per_pixel_allowed_range, acceptable_amount_of_activity_in_image, image_properties,
);