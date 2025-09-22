
use std::fmt::Display;
use std::ops::RangeInclusive;
use std::time::Instant;
use ndarray::{Array3, Zip};
use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;
use feagi_data_structures::data::descriptors::{ImageFrameProperties};
use feagi_data_structures::data::{ImageFrame, Percentage};
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::wrapped_io_data::{WrappedIOData, WrappedIOType};
use crate::data_pipeline::pipeline_stage::PipelineStage;

#[derive(Debug, Clone)]
pub struct ImageFrameQuickDiffStage {
    /// The output buffer containing the computed difference image
    diff_cache: WrappedIOData, // Image Frame
    previous_frame_cache: WrappedIOData, // Image Frame
    /// Properties that input images must match (resolution, color space, channels)
    input_definition: ImageFrameProperties,
    /// Minimum difference threshold for pixel changes to be considered significant
    inclusive_pixel_range: RangeInclusive<u8>,
    samples_count_lower_bound: usize,
    samples_count_upper_bound: usize,
}

impl Display for ImageFrameQuickDiffStage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ImageFrameQuickDiffProcessor()")
    }
}

impl PipelineStage for ImageFrameQuickDiffStage {
    fn get_input_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.input_definition))
    }

    fn get_output_data_type(&self) -> WrappedIOType {
        WrappedIOType::ImageFrame(Some(self.input_definition))
    }

    fn get_most_recent_output(&self) -> &WrappedIOData {
        &self.diff_cache
    }

    fn process_new_input(&mut self, value: &WrappedIOData, _time_of_input: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        quick_diff_and_check_if_pass(value, &self.previous_frame_cache, &mut self.diff_cache, &self.inclusive_pixel_range, self.samples_count_lower_bound, self.samples_count_upper_bound)?;
        self.previous_frame_cache = value.clone();
        Ok(&self.diff_cache)
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }
}

impl ImageFrameQuickDiffStage {

    pub fn new(image_properties: ImageFrameProperties, per_pixel_allowed_range: RangeInclusive<u8>, acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>) -> Result<Self, FeagiDataError> {
        
        let cache_image = ImageFrame::new_from_image_frame_properties(&image_properties)?;
        let number_of_samples = image_properties.get_number_of_channels();
        Ok(ImageFrameQuickDiffStage {
            diff_cache: WrappedIOData::ImageFrame(cache_image.clone()),
            previous_frame_cache: WrappedIOData::ImageFrame(cache_image.clone()), // Image Frame
            input_definition: image_properties,
            inclusive_pixel_range: per_pixel_allowed_range,
            samples_count_lower_bound: (acceptable_amount_of_activity_in_image.start().get_as_0_1() * number_of_samples as f32) as usize,
            samples_count_upper_bound: (acceptable_amount_of_activity_in_image.end().get_as_0_1() * number_of_samples as f32) as usize
        })
    }
}

fn quick_diff_and_check_if_pass(minuend: &WrappedIOData, subtrahend: &WrappedIOData, diff_result: &mut WrappedIOData, pixel_bounds: &RangeInclusive<u8>, samples_count_lower_bound: usize, samples_count_upper_bound: usize) -> Result<(), FeagiDataError> {
    let minuend: &ImageFrame = minuend.try_into()?;
    let subtrahend: &ImageFrame = subtrahend.try_into()?;
    let diff_result: &mut ImageFrame = diff_result.try_into()?;

    let pixel_val_lower_bound = *pixel_bounds.start();
    let pixel_val_upper_bound = *pixel_bounds.end();

    let minuend_arr: &Array3<u8> = minuend.get_internal_data();
    let subtrahend_arr: &Array3<u8> = subtrahend.get_internal_data();
    let diff_arr: &mut Array3<u8> = diff_result.get_internal_data_mut();

    let total_pass_count: usize = Zip::from(minuend_arr)
        .and(subtrahend_arr)
        .and(diff_arr)
        .par_map_collect(|&minuend, &subtrahend, diff| {
            let absolute_diff = if minuend >= subtrahend {
                minuend - subtrahend
            } else {
                subtrahend - minuend
            };
            let passed = absolute_diff >= pixel_val_lower_bound && absolute_diff <= pixel_val_upper_bound;
            *diff = if passed { subtrahend } else { 0 };
            passed as usize
        })
        .into_par_iter()
        .sum();

    let should_pass = total_pass_count >= samples_count_lower_bound && total_pass_count<= samples_count_upper_bound;
    diff_result.skip_encoding = !should_pass ||  diff_result.skip_encoding;
    Ok(())
}


