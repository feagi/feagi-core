use std::any::Any;
use std::fmt::Display;
use std::ops::RangeInclusive;
use std::time::Instant;
use ndarray::{Array3, Zip};
use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;
use feagi_data_structures::FeagiDataError;
use crate::data_pipeline::pipeline_stage::PipelineStage;
use crate::data_pipeline::pipeline_stage_properties::PipelineStageProperties;
use crate::data_types::descriptors::ImageFrameProperties;
use crate::data_types::{ImageFrame, Percentage};
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

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
    acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn create_properties(&self) -> PipelineStageProperties {
        PipelineStageProperties::ImageQuickDiff {
            per_pixel_allowed_range: self.inclusive_pixel_range.clone(),
            acceptable_amount_of_activity_in_image: self.acceptable_amount_of_activity_in_image.clone(),
            image_properties: self.input_definition,
        }
    }

    fn load_properties(&mut self, properties: PipelineStageProperties) -> Result<(), FeagiDataError> {
        match properties {
            PipelineStageProperties::ImageQuickDiff { 
                per_pixel_allowed_range,
                acceptable_amount_of_activity_in_image,
                ..
            } => {
                if per_pixel_allowed_range.is_empty() {
                    return Err(FeagiDataError::BadParameters("per_pixel_allowed_range appears to be empty! Are your bounds correct?".into()));
                }

                if acceptable_amount_of_activity_in_image.is_empty() {
                    return Err(FeagiDataError::BadParameters("acceptable_amount_of_activity_in_image appears to be empty! Are your bounds correct?".into()));
                }

                // Update the threshold and activity range
                self.inclusive_pixel_range = per_pixel_allowed_range;
                self.acceptable_amount_of_activity_in_image = acceptable_amount_of_activity_in_image;
                
                // Recalculate sample count bounds based on new activity range
                let total_pixels = self.input_definition.get_number_of_samples();
                let sample_count_bounds = get_sample_count_lower_upper_bounds(&self.acceptable_amount_of_activity_in_image, total_pixels);
                self.samples_count_lower_bound = sample_count_bounds.0;
                self.samples_count_upper_bound = sample_count_bounds.1;
                
                Ok(())
            }
            _ => Err(FeagiDataError::BadParameters(
                "load_properties called with incompatible properties type for ImageFrameQuickDiffStage".into()
            ))
        }
    }
}

impl ImageFrameQuickDiffStage {

    pub fn new(image_properties: ImageFrameProperties, per_pixel_allowed_range: RangeInclusive<u8>, acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>) -> Result<Self, FeagiDataError> {

        if per_pixel_allowed_range.is_empty() {
            return Err(FeagiDataError::BadParameters("per_pixel_allowed_range appears to be empty! Are your bounds correct?".into()));
        }

        if acceptable_amount_of_activity_in_image.is_empty() {
            return Err(FeagiDataError::BadParameters("acceptable_amount_of_activity_in_image appears to be empty! Are your bounds correct?".into()));
        }

        let cache_image = ImageFrame::new_from_image_frame_properties(&image_properties)?;
        // Calculate total number of pixels (width * height * channels) for activity percentage calculation
        let total_pixels = image_properties.get_number_of_samples();
        let sample_count_bounds = get_sample_count_lower_upper_bounds(&acceptable_amount_of_activity_in_image, total_pixels);
        Ok(ImageFrameQuickDiffStage {
            diff_cache: WrappedIOData::ImageFrame(cache_image.clone()),
            previous_frame_cache: WrappedIOData::ImageFrame(cache_image.clone()), // Image Frame
            input_definition: image_properties,
            inclusive_pixel_range: per_pixel_allowed_range,
            samples_count_lower_bound: sample_count_bounds.0,
            samples_count_upper_bound: sample_count_bounds.1,
            acceptable_amount_of_activity_in_image,
        })
    }

    pub fn new_box(image_properties: ImageFrameProperties, per_pixel_allowed_range: RangeInclusive<u8>, acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>) -> Result<Box<dyn PipelineStage + Send + Sync + 'static>, FeagiDataError> {
        Ok(Box::new(ImageFrameQuickDiffStage::new(image_properties, per_pixel_allowed_range, acceptable_amount_of_activity_in_image)?))
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
            // If pixel changed by acceptable amount, output NEW value (minuend), otherwise filter out (0)
            *diff = if passed { minuend } else { 0 };
            passed as usize
        })
        .into_par_iter()
        .sum();

    let should_pass = total_pass_count >= samples_count_lower_bound && total_pass_count<= samples_count_upper_bound;
    // Set skip_encoding based on should_pass: if should_pass is false, skip encoding; otherwise allow encoding
    // Note: We don't preserve previous skip_encoding value - the diff stage controls this flag
    diff_result.skip_encoding = !should_pass;
    Ok(())
}

fn get_sample_count_lower_upper_bounds(acceptable_amount_of_activity_in_image: &RangeInclusive<Percentage>, total_pixels: usize) -> (usize, usize) {
    ((acceptable_amount_of_activity_in_image.start().get_as_0_1() * total_pixels as f32) as usize,
     (acceptable_amount_of_activity_in_image.end().get_as_0_1() * total_pixels as f32) as usize)
}
