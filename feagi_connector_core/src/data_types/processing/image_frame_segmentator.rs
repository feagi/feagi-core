use feagi_data_structures::FeagiDataError;
use crate::data_types::descriptors::{ColorChannelLayout, GazeProperties, ImageFrameProperties, SegmentedImageFrameProperties};
use crate::data_types::{ImageFrame, ImageFrameProcessor, SegmentedImageFrame};

#[derive(Debug, Clone)]
pub struct ImageFrameSegmentator {
    input_properties: ImageFrameProperties,
    output_properties: SegmentedImageFrameProperties,
    ordered_transformers: [ImageFrameProcessor; 9],
    gaze_being_used: GazeProperties
}

impl ImageFrameSegmentator {
    pub fn new(input_properties: ImageFrameProperties, output_properties: SegmentedImageFrameProperties, initial_gaze: GazeProperties) -> Result<ImageFrameSegmentator, FeagiDataError> {
        Ok(
            ImageFrameSegmentator{
                input_properties: input_properties.clone(),
                output_properties: output_properties.clone(),
                ordered_transformers: Self::get_new_ordered_transformers(
                    &input_properties,
                    &output_properties,
                    &initial_gaze,
                )?,
                gaze_being_used: initial_gaze
            }
        )
    }
    
    pub fn update_gaze(&mut self, gaze: &GazeProperties) -> Result<(), FeagiDataError> {
        self.ordered_transformers = Self::get_new_ordered_transformers(&self.input_properties, &self.output_properties, gaze)?;
        self.gaze_being_used = gaze.clone();
        Ok(())
    }

    pub fn get_used_gaze(&self) -> GazeProperties {
        self.gaze_being_used
    }
    
    pub fn verify_input_image(&self, input: &ImageFrame) -> Result<(), FeagiDataError> {
        self.input_properties.verify_image_frame_matches_properties(input)
    }
    
    pub fn verify_output_image(&self, output: &SegmentedImageFrame) -> Result<(), FeagiDataError> {
        self.output_properties.verify_segmented_image_frame_matches_properties(output)
    }
    
    pub fn segment_image(&mut self, input: &ImageFrame, target: &mut SegmentedImageFrame) -> Result<(), FeagiDataError> {
        if input.get_xy_resolution() != self.input_properties.get_image_resolution() {
            return Err(FeagiDataError::BadParameters(format!("Expected Image Resolution of {}, but got {}!", self.input_properties.get_image_resolution(), input.get_xy_resolution())));
        }
        if *input.get_channel_layout() != self.input_properties.get_color_channel_layout() {
            return Err(FeagiDataError::BadParameters(format!("Expected Image Color Channels of {} but got {}!", self.input_properties.get_color_channel_layout(), input.get_channel_layout())));
        }
        if target.get_segmented_image_frame_properties() != self.output_properties {
            return Err(FeagiDataError::BadParameters("Write Target SegmentedImageFrame does not have expected properties!".into()));
        }

        let output_image_frames = target.get_mut_ordered_image_frame_references();
        
        self.ordered_transformers[0].process_image(input, output_image_frames[0])?;
        self.ordered_transformers[1].process_image(input, output_image_frames[1])?;
        self.ordered_transformers[2].process_image(input, output_image_frames[2])?;
        self.ordered_transformers[3].process_image(input, output_image_frames[3])?;
        self.ordered_transformers[4].process_image(input, output_image_frames[4])?;
        self.ordered_transformers[5].process_image(input, output_image_frames[5])?;
        self.ordered_transformers[6].process_image(input, output_image_frames[6])?;
        self.ordered_transformers[7].process_image(input, output_image_frames[7])?;
        self.ordered_transformers[8].process_image(input, output_image_frames[8])?;
        
        Ok(())
    }
    
    
    
    
    fn get_new_ordered_transformers(input_properties: &ImageFrameProperties, output_properties: &SegmentedImageFrameProperties, gaze: &GazeProperties) 
        -> Result<[ImageFrameProcessor; 9], FeagiDataError> {
        
        let cropping_points = gaze.calculate_source_corner_points_for_segmented_video_frame(input_properties.get_image_resolution())?;
        let center_color_channels = output_properties.get_center_color_channel();
        let peripheral_color_channels = output_properties.get_peripheral_color_channels();
        let color_space = output_properties.get_color_space();
        let output_resolutions = output_properties.get_resolutions().as_ordered_array();
        
        let center_to_grayscale: bool = center_color_channels == &ColorChannelLayout::GrayScale;
        let peripheral_to_grayscale: bool = peripheral_color_channels == &ColorChannelLayout::GrayScale;

        Ok([
            ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[0])?
                .set_resizing_to(*output_resolutions[0])?
                .set_color_space_to(color_space)?
                .set_conversion_to_grayscale(peripheral_to_grayscale)?.to_owned(),
            ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[1])?
                .set_resizing_to(*output_resolutions[1])?
                .set_color_space_to(color_space)?
                .set_conversion_to_grayscale(peripheral_to_grayscale)?.to_owned(),
            ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[2])?
                .set_resizing_to(*output_resolutions[2])?
                .set_color_space_to(color_space)?
                .set_conversion_to_grayscale(peripheral_to_grayscale)?.to_owned(),
            ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[3])?
                .set_resizing_to(*output_resolutions[3])?
                .set_color_space_to(color_space)?
                .set_conversion_to_grayscale(peripheral_to_grayscale)?.to_owned(),
            ImageFrameProcessor::new(*input_properties) // center
                .set_cropping_from(cropping_points[4])?
                .set_resizing_to(*output_resolutions[4])?
                .set_color_space_to(color_space)?
                .set_conversion_to_grayscale(center_to_grayscale)?.to_owned(),
            ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[5])?
                .set_resizing_to(*output_resolutions[5])?
                .set_color_space_to(color_space)?
                .set_conversion_to_grayscale(peripheral_to_grayscale)?.to_owned(),
            ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[6])?
                .set_resizing_to(*output_resolutions[6])?
                .set_color_space_to(color_space)?
                .set_conversion_to_grayscale(peripheral_to_grayscale)?.to_owned(),
            ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[7])?
                .set_resizing_to(*output_resolutions[7])?
                .set_color_space_to(color_space)?
                .set_conversion_to_grayscale(peripheral_to_grayscale)?.to_owned(),
            ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[8])?
                .set_resizing_to(*output_resolutions[8])?
                .set_color_space_to(color_space)?
                .set_conversion_to_grayscale(peripheral_to_grayscale)?.to_owned(),
        ])


        
        
    }
    
}



