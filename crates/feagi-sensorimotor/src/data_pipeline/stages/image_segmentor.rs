use crate::data_pipeline::pipeline_stage::PipelineStage;
use crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties;
use crate::data_types::descriptors::{ImageFrameProperties, SegmentedImageFrameProperties};
use crate::data_types::ImageFrameSegmentator;
use crate::data_types::{ImageFrame, SegmentedImageFrame};
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_data_structures::FeagiDataError;
use std::any::Any;
use std::fmt::Display;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ImageFrameSegmentatorStage {
    input_image_properties: ImageFrameProperties,
    output_image_properties: SegmentedImageFrameProperties,
    image_segmentator: ImageFrameSegmentator,
    cached: WrappedIOData,
}

impl Display for ImageFrameSegmentatorStage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "ImageFrameSegmentatorStage(input: {:?}, output: {:?})",
            self.input_image_properties, self.output_image_properties
        )
    }
}

impl PipelineStage for ImageFrameSegmentatorStage {
    fn get_input_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.input_image_properties))
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        WrappedIOType::SegmentedImageFrame(Some(self.output_image_properties))
    }

    fn get_most_recent_output(&self) -> &WrappedIOData {
        &self.cached
    }

    fn process_new_input(
        &mut self,
        value: &WrappedIOData,
        _time_of_input: Instant,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        let read_from: &ImageFrame = value.try_into()?;
        let write_to: &mut SegmentedImageFrame = (&mut self.cached).try_into()?;

        self.image_segmentator.segment_image(read_from, write_to)?;
        Ok(self.get_most_recent_output())
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn create_properties(&self) -> PipelineStageProperties {
        PipelineStageProperties::ImageFrameSegmentator {
            input_image_properties: self.input_image_properties,
            output_image_properties: self.output_image_properties,
            segmentation_gaze: self.image_segmentator.get_used_gaze(),
        }
    }

    fn load_properties(
        &mut self,
        properties: PipelineStageProperties,
    ) -> Result<(), FeagiDataError> {
        match properties {
            PipelineStageProperties::ImageFrameSegmentator { segmentation_gaze, .. } => { // Other properties not allowed to be changed
                self.image_segmentator.update_gaze(&segmentation_gaze)?; // TODO for only updating gaze, we dont need to change this much!
                Ok(())
            }
            _ => Err(FeagiDataError::BadParameters(
                "load_properties called with incompatible properties type for ImageFrameSegmentatorStage".into()
            ))
        }
    }
}

impl ImageFrameSegmentatorStage {
    pub fn new(
        input_image_properties: ImageFrameProperties,
        output_image_properties: SegmentedImageFrameProperties,
        image_segmentator: ImageFrameSegmentator,
    ) -> Result<Self, FeagiDataError> {
        let cached: SegmentedImageFrame =
            SegmentedImageFrame::from_segmented_image_frame_properties(&output_image_properties)?;

        Ok(ImageFrameSegmentatorStage {
            input_image_properties,
            output_image_properties,
            image_segmentator,
            cached: cached.into(),
        })
    }

    pub(crate) fn new_box(
        input_image_properties: ImageFrameProperties,
        output_image_properties: SegmentedImageFrameProperties,
        image_segmentator: ImageFrameSegmentator,
    ) -> Result<Box<dyn PipelineStage + 'static>, FeagiDataError> {
        Ok(Box::new(ImageFrameSegmentatorStage::new(
            input_image_properties,
            output_image_properties,
            image_segmentator,
        )?))
    }
}
