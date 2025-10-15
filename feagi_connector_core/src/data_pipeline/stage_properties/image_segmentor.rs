use std::any::Any;
use feagi_data_structures::FeagiDataError;
use crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties;
use crate::data_pipeline::PipelineStage;
use crate::data_pipeline::stages::ImageFrameSegmentatorStage;
use crate::data_types::descriptors::{GazeProperties, ImageFrameProperties, SegmentedImageFrameProperties};
use crate::data_types::ImageFrameSegmentator;
use crate::wrapped_io_data::WrappedIOType;

/// Properties for ImageFrameSegmentatorStage that store configuration for image segmentation
#[derive(Debug, Clone)]
pub struct ImageSegmentorStageProperties {
    input_image_properties: ImageFrameProperties,
    output_image_properties: SegmentedImageFrameProperties,
    image_segmentator: ImageFrameSegmentator,
}

impl PipelineStageProperties for ImageSegmentorStageProperties {
    fn get_input_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.input_image_properties))
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        WrappedIOType::SegmentedImageFrame(Some(self.output_image_properties))
    }

    fn clone_box(&self) -> Box<dyn PipelineStageProperties + Sync + Send> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn create_stage(&self) -> Box<dyn PipelineStage> {
        ImageFrameSegmentatorStage::new_box(
            self.input_image_properties,
            self.output_image_properties,
            self.image_segmentator.clone()
        ).unwrap()
    }
}

impl ImageSegmentorStageProperties {
    pub fn new(
        input_image_properties: ImageFrameProperties,
        output_image_properties: SegmentedImageFrameProperties,
        image_segmentator: ImageFrameSegmentator
    ) -> Self {
        ImageSegmentorStageProperties {
            input_image_properties,
            output_image_properties,
            image_segmentator,
        }
    }
    
    pub fn new_box(
        input_image_properties: ImageFrameProperties,
        output_image_properties: SegmentedImageFrameProperties,
        initial_gaze: GazeProperties

    ) -> Result<Box<dyn PipelineStageProperties + Send + Sync + 'static>, FeagiDataError> {

        let image_segmentator = ImageFrameSegmentator::new(
            input_image_properties,
            output_image_properties,
            initial_gaze
        )?;

        Ok(Box::new(ImageSegmentorStageProperties::new(
            input_image_properties,
            output_image_properties,
            image_segmentator
        )))
    }

    pub fn update_from_gaze(&mut self, new_gaze: GazeProperties) -> Result<(), FeagiDataError> {
        self.image_segmentator.update_gaze(&new_gaze)?;
        Ok(())
    }
}

impl std::fmt::Display for ImageSegmentorStageProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ImageSegmentorStage(input: {:?}, output: {:?})", 
               self.input_image_properties, self.output_image_properties)
    }
}

impl<'a> TryFrom<&'a Box::<dyn PipelineStageProperties>> for &'a ImageSegmentorStageProperties {
    type Error = FeagiDataError;
    fn try_from(value: &Box::<dyn PipelineStageProperties>) -> Result<Self, Self::Error> {
        match value.as_any().downcast_ref::<ImageSegmentorStageProperties>() {
            Some(p) => Ok(p),
            None => Err(FeagiDataError::InternalError("Given stage attempted to be cast as '&ImageSegmentorStageProperties' when it isn't!".into()))
        }
    }
}

impl TryFrom<Box::<dyn PipelineStageProperties>> for ImageSegmentorStageProperties {
    type Error = FeagiDataError;
    fn try_from(value: Box::<dyn PipelineStageProperties>) -> Result<Self, Self::Error> {
        match value.as_any().downcast_ref::<ImageSegmentorStageProperties>() {
            Some(p) => Ok(p.clone()),
            None => Err(FeagiDataError::InternalError("Given stage attempted to be cast as 'ImageSegmentorStageProperties' when it isn't!".into()))
        }
    }
}

