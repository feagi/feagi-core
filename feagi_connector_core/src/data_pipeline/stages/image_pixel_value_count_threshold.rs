use std::fmt::Display;
use std::ops::RangeInclusive;
use std::time::Instant;
use ndarray::{Array3, Zip};
use rayon::prelude::*;
use feagi_data_structures::data::descriptors::{ImageFrameProperties};
use feagi_data_structures::data::{ImageFrame, Percentage};
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::data_pipeline::pipeline_stage::PipelineStage;

#[derive(Debug, Clone)]
pub struct ImagePixelValueCountThresholdStage {
    cache: WrappedIOData, // Image Frame
    input_definition: ImageFrameProperties,
    /// Minimum difference threshold for pixel changes to be considered significant
    inclusive_pixel_range: RangeInclusive<u8>,
    samples_count_lower_bound: usize,
    samples_count_upper_bound: usize,
}

impl Display for ImagePixelValueCountThresholdStage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ImagePixelValueCountThresholdStage()")
    }
}

impl PipelineStage for ImagePixelValueCountThresholdStage {
    fn get_input_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.input_definition))
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.input_definition))
    }

    fn get_most_recent_output(&self) -> &WrappedIOData {
        &self.cache
    }

    fn process_new_input(&mut self, value: &WrappedIOData, _time_of_input: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        filter_and_set_if_pass(value, &mut self.cache, &self.inclusive_pixel_range, self.samples_count_lower_bound, self.samples_count_upper_bound)?;
        Ok(&self.cache)
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }
}

impl ImagePixelValueCountThresholdStage {

    pub fn new(image_properties: ImageFrameProperties, per_pixel_allowed_range: RangeInclusive<u8>, acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>) -> Result<Self, FeagiDataError> {

        let number_of_samples = image_properties.get_number_of_channels();
        Ok(ImagePixelValueCountThresholdStage {
            cache: WrappedIOData::ImageFrame(ImageFrame::new_from_image_frame_properties(&image_properties)?),
            input_definition: image_properties,
            inclusive_pixel_range: per_pixel_allowed_range,
            samples_count_lower_bound: (acceptable_amount_of_activity_in_image.start().get_as_0_1() * number_of_samples as f32) as usize,
            samples_count_upper_bound: (acceptable_amount_of_activity_in_image.end().get_as_0_1() * number_of_samples as f32) as usize
        })
    }
}

fn filter_and_set_if_pass(source: &WrappedIOData, filter_result: &mut WrappedIOData, pixel_bounds: &RangeInclusive<u8>, samples_count_lower_bound: usize, samples_count_upper_bound: usize) -> Result<(), FeagiDataError> {
    let source: &ImageFrame = source.try_into()?;
    let filter_result: &mut ImageFrame = filter_result.try_into()?;
    let pixel_val_lower_bound = *pixel_bounds.start();
    let pixel_val_upper_bound = *pixel_bounds.end();


    let source_arr: &Array3<u8> = source.get_internal_data();
    let filter_arr: &mut Array3<u8> = filter_result.get_internal_data_mut();

    let total_pass_count: usize = Zip::from(source_arr)
        .and(filter_arr)
        .par_map_collect(|&source, filter| {
            let passed = source >= pixel_val_lower_bound && source <= pixel_val_upper_bound;
            *filter = if (source >= pixel_val_lower_bound && source <= pixel_val_upper_bound) {source} else {0};
            passed as usize
        })
        .into_par_iter()
        .sum();
    let should_pass = total_pass_count >= samples_count_lower_bound && total_pass_count<= samples_count_upper_bound;
    filter_result.skip_encoding = !should_pass || filter_result.skip_encoding;
    Ok(())
}


