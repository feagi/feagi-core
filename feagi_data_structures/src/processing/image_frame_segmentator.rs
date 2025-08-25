use crate::FeagiDataError;
use crate::processing::image_frame_processor::ImageFrameProcessor;
use crate::data::image_descriptors::{ColorChannelLayout, GazeProperties, ImageFrameProperties, SegmentedImageFrameProperties};
use crate::data::{ImageFrame, SegmentedImageFrame};

#[derive(Debug, Clone)]
pub struct ImageFrameSegmentator {
    input_properties: ImageFrameProperties,
    output_properties: SegmentedImageFrameProperties,
    ordered_transformers: [ImageFrameProcessor; 9],
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
                )?
            }
        )
    }
    
    pub fn update_gaze(&mut self, gaze: &GazeProperties) -> Result<(), FeagiDataError> {
        self.ordered_transformers = Self::get_new_ordered_transformers(&self.input_properties, &self.output_properties, gaze)?;
        Ok(())
    }
    
    pub fn verify_input_image(&self, input: &ImageFrame) -> Result<(), FeagiDataError> {
        self.input_properties.verify_image_frame_matches_properties(input)
    }
    
    pub fn verify_output_image(&self, output: &SegmentedImageFrame) -> Result<(), FeagiDataError> {
        self.output_properties.verify_segmented_image_frame_matches_properties(output)
    }
    
    pub fn segment_image(&self, input: &ImageFrame, target: &mut SegmentedImageFrame) -> Result<(), FeagiDataError> {
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
            *ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[0].lower_left_row_major(), cropping_points[0].upper_right_row_major())?
                .set_resizing_to(*output_resolutions[0])?.set_color_space_to(color_space)?.set_conversion_to_grayscale(peripheral_to_grayscale)?,
            *ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[1].lower_left_row_major(), cropping_points[1].upper_right_row_major())?
                .set_resizing_to(*output_resolutions[1])?.set_color_space_to(color_space)?.set_conversion_to_grayscale(peripheral_to_grayscale)?,
            *ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[2].lower_left_row_major(), cropping_points[2].upper_right_row_major())?
                .set_resizing_to(*output_resolutions[2])?.set_color_space_to(color_space)?.set_conversion_to_grayscale(peripheral_to_grayscale)?,
            *ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[3].lower_left_row_major(), cropping_points[3].upper_right_row_major())?
                .set_resizing_to(*output_resolutions[3])?.set_color_space_to(color_space)?.set_conversion_to_grayscale(peripheral_to_grayscale)?,
            *ImageFrameProcessor::new(*input_properties) // center
                .set_cropping_from(cropping_points[4].lower_left_row_major(), cropping_points[4].upper_right_row_major())?
                .set_resizing_to(*output_resolutions[4])?.set_color_space_to(color_space)?.set_conversion_to_grayscale(center_to_grayscale)?,
            *ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[5].lower_left_row_major(), cropping_points[5].upper_right_row_major())?
                .set_resizing_to(*output_resolutions[5])?.set_color_space_to(color_space)?.set_conversion_to_grayscale(peripheral_to_grayscale)?,
            *ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[6].lower_left_row_major(), cropping_points[6].upper_right_row_major())?
                .set_resizing_to(*output_resolutions[6])?.set_color_space_to(color_space)?.set_conversion_to_grayscale(peripheral_to_grayscale)?,
            *ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[7].lower_left_row_major(), cropping_points[7].upper_right_row_major())?
                .set_resizing_to(*output_resolutions[7])?.set_color_space_to(color_space)?.set_conversion_to_grayscale(peripheral_to_grayscale)?,
            *ImageFrameProcessor::new(*input_properties)
                .set_cropping_from(cropping_points[8].lower_left_row_major(), cropping_points[8].upper_right_row_major())?
                .set_resizing_to(*output_resolutions[8])?.set_color_space_to(color_space)?.set_conversion_to_grayscale(peripheral_to_grayscale)?,
        ])
        
        
    }
    
}



