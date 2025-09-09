use std::fmt::Display;
use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::data::image_descriptors::{ImageFrameProperties, SegmentedImageFrameProperties};
use feagi_data_structures::data::{ImageFrame, SegmentedImageFrame};
use feagi_data_structures::processing::ImageFrameSegmentator;
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::data_pipeline::pipeline_stage::PipelineStage;

#[derive(Debug, Clone)]
pub struct ImageFrameSegmentatorStage {
    input_image_properties: ImageFrameProperties,
    output_image_properties: SegmentedImageFrameProperties,
    image_segmentator: ImageFrameSegmentator,
    cached: WrappedIOData
}

impl Display for ImageFrameSegmentatorStage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ImageFrameSegmentatorProcessor()") // TODO fill out
    }
}

impl PipelineStage for ImageFrameSegmentatorStage {
    fn get_input_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.input_image_properties))
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        WrappedIOType::SegmentedImageFrame(Some(self.output_image_properties))
    }

    fn get_most_recent_output(&self) -> &WrappedIOData { &self.cached }

    fn process_new_input(&mut self, value: &WrappedIOData, time_of_input: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        // TODO use time
        let read_from: &ImageFrame = value.try_into()?;
        let write_to: &mut SegmentedImageFrame = (&mut self.cached).try_into()?;
        
        self.image_segmentator.segment_image(read_from, write_to)?;
        Ok(self.get_most_recent_output())
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }
}

impl ImageFrameSegmentatorStage {
    pub fn new(input_image_properties: ImageFrameProperties, output_image_properties: SegmentedImageFrameProperties, image_segmentator: ImageFrameSegmentator) -> Self {
        let cached: SegmentedImageFrame = SegmentedImageFrame::from_segmented_image_frame_properties(&output_image_properties).unwrap();

        ImageFrameSegmentatorStage {
            input_image_properties,
            output_image_properties,
            image_segmentator,
            cached: cached.into()
        }

    }
}




