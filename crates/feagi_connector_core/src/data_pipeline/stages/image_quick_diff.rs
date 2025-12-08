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
use crate::data_pipeline::stage_properties::ImageQuickDiffStageProperties;
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

    fn process_new_input(&mut self, value: &WrappedIOData, time_of_input: Instant) -> Result<&WrappedIOData, FeagiDataError> {
        use tracing::info;
        let t_total_start = std::time::Instant::now();
        
        // Check if dimensions match before diffing (prevents panic on first frame or resolution changes)
        let t_convert_start = std::time::Instant::now();
        let current_frame: &ImageFrame = value.try_into()?;
        let previous_frame: &ImageFrame = (&self.previous_frame_cache).try_into()?;
        let t_convert = t_convert_start.elapsed();
        
        // If dimensions don't match, reset cache and skip diff (first frame or resolution change)
        if current_frame.get_xy_resolution() != previous_frame.get_xy_resolution() 
            || current_frame.get_channel_layout() != previous_frame.get_channel_layout() {
            tracing::debug!(
                "🦀 [IMAGE-QUICK-DIFF] Dimension/resolution change detected, resetting cache. Current: {:?}, Previous: {:?}",
                current_frame.get_xy_resolution(), previous_frame.get_xy_resolution()
            );
            let t_clone_start = std::time::Instant::now();
            // Reset diff_cache to match current frame
            self.diff_cache = value.clone();
            // Reset previous_frame_cache for next iteration
            self.previous_frame_cache = value.clone();
            let t_clone = t_clone_start.elapsed();
            // Allow encoding on first frame or after resolution change
            // Set skip_encoding to false by converting and setting directly
            let diff_frame: &mut ImageFrame = (&mut self.diff_cache).try_into()?;
            diff_frame.skip_encoding = false;
            let t_total = t_total_start.elapsed();
            info!(
                "⏱️ [PERF-DIFF] process_new_input (reset): total={:.2}ms | convert={:.2}ms | clone={:.2}ms",
                t_total.as_secs_f64() * 1000.0,
                t_convert.as_secs_f64() * 1000.0,
                t_clone.as_secs_f64() * 1000.0
            );
            return Ok(&self.diff_cache);
        }
        
        // Dimensions match - proceed with diff
        let t_diff_start = std::time::Instant::now();
        quick_diff_and_check_if_pass(value, &self.previous_frame_cache, &mut self.diff_cache, &self.inclusive_pixel_range, self.samples_count_lower_bound, self.samples_count_upper_bound)?;
        let t_diff = t_diff_start.elapsed();
        
        let t_clone_start = std::time::Instant::now();
        self.previous_frame_cache = value.clone();
        let t_clone = t_clone_start.elapsed();
        
        let t_total = t_total_start.elapsed();
        info!(
            "⏱️ [PERF-DIFF] process_new_input: total={:.2}ms | convert={:.2}ms | diff={:.2}ms | clone={:.2}ms",
            t_total.as_secs_f64() * 1000.0,
            t_convert.as_secs_f64() * 1000.0,
            t_diff.as_secs_f64() * 1000.0,
            t_clone.as_secs_f64() * 1000.0
        );
        
        Ok(&self.diff_cache)
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn create_properties(&self) -> Box<dyn PipelineStageProperties + Sync + Send> {
        Box::new(ImageQuickDiffStageProperties {
            image_properties: self.input_definition.clone(),
            per_pixel_allowed_range: self.inclusive_pixel_range.clone(),
            acceptable_amount_of_activity_in_image: self.acceptable_amount_of_activity_in_image.clone(),
        })
    }

    fn load_properties(&mut self, properties: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        todo!()
    }
}

impl ImageFrameQuickDiffStage {

    pub fn new(image_properties: ImageFrameProperties, per_pixel_allowed_range: RangeInclusive<u8>, acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>) -> Result<Self, FeagiDataError> {

        let cache_image = ImageFrame::new_from_image_frame_properties(&image_properties)?;
        // Calculate total number of pixels (width * height * channels) for activity percentage calculation
        let total_pixels = image_properties.get_number_of_samples();
        Ok(ImageFrameQuickDiffStage {
            diff_cache: WrappedIOData::ImageFrame(cache_image.clone()),
            previous_frame_cache: WrappedIOData::ImageFrame(cache_image.clone()), // Image Frame
            input_definition: image_properties,
            inclusive_pixel_range: per_pixel_allowed_range,
            samples_count_lower_bound: (acceptable_amount_of_activity_in_image.start().get_as_0_1() * total_pixels as f32) as usize,
            samples_count_upper_bound: (acceptable_amount_of_activity_in_image.end().get_as_0_1() * total_pixels as f32) as usize,
            acceptable_amount_of_activity_in_image,
        })
    }

    pub fn new_box(image_properties: ImageFrameProperties, per_pixel_allowed_range: RangeInclusive<u8>, acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>) -> Result<Box<dyn PipelineStage + Send + Sync + 'static>, FeagiDataError> {
        Ok(Box::new(ImageFrameQuickDiffStage::new(image_properties, per_pixel_allowed_range, acceptable_amount_of_activity_in_image)?))
    }
}

fn quick_diff_and_check_if_pass(minuend: &WrappedIOData, subtrahend: &WrappedIOData, diff_result: &mut WrappedIOData, pixel_bounds: &RangeInclusive<u8>, samples_count_lower_bound: usize, samples_count_upper_bound: usize) -> Result<(), FeagiDataError> {
    use tracing::info;
    let t_total_start = std::time::Instant::now();
    
    let t_convert_start = std::time::Instant::now();
    let minuend: &ImageFrame = minuend.try_into()?;
    let subtrahend: &ImageFrame = subtrahend.try_into()?;
    let diff_result: &mut ImageFrame = diff_result.try_into()?;
    let t_convert = t_convert_start.elapsed();

    let pixel_val_lower_bound = *pixel_bounds.start();
    let pixel_val_upper_bound = *pixel_bounds.end();

    let t_get_data_start = std::time::Instant::now();
    let minuend_arr: &Array3<u8> = minuend.get_internal_data();
    let subtrahend_arr: &Array3<u8> = subtrahend.get_internal_data();
    let diff_arr: &mut Array3<u8> = diff_result.get_internal_data_mut();
    let t_get_data = t_get_data_start.elapsed();

    // Validate dimensions match before Zip operation (prevents panic)
    if minuend_arr.shape() != subtrahend_arr.shape() || minuend_arr.shape() != diff_arr.shape() {
        let minuend_shape = minuend_arr.shape();
        let subtrahend_shape = subtrahend_arr.shape();
        let diff_shape = diff_arr.shape();
        tracing::warn!(
            "🦀 [IMAGE-QUICK-DIFF] ⚠️ Dimension mismatch: minuend={:?}, subtrahend={:?}, diff={:?}. Resetting previous frame cache.",
            minuend_shape, subtrahend_shape, diff_shape
        );
        // Reset diff_result to match minuend (current frame) - this handles first frame or resolution changes
        *diff_result = minuend.clone();
        diff_result.skip_encoding = false; // Allow encoding on first frame or after resolution change
        return Ok(());
    }

    let t_zip_start = std::time::Instant::now();
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
    let t_zip = t_zip_start.elapsed();

    let t_check_start = std::time::Instant::now();
    let should_pass = total_pass_count >= samples_count_lower_bound && total_pass_count<= samples_count_upper_bound;
    // Set skip_encoding based on should_pass: if should_pass is false, skip encoding; otherwise allow encoding
    // Note: We don't preserve previous skip_encoding value - the diff stage controls this flag
    diff_result.skip_encoding = !should_pass;
    let t_check = t_check_start.elapsed();
    
    let t_total = t_total_start.elapsed();
    let total_pixels = minuend_arr.len();
    info!(
        "⏱️ [PERF-DIFF] quick_diff_and_check_if_pass: total={:.2}ms | convert={:.2}ms | get_data={:.2}ms | zip_parallel={:.2}ms | check={:.2}ms | pixels={} | passed={}",
        t_total.as_secs_f64() * 1000.0,
        t_convert.as_secs_f64() * 1000.0,
        t_get_data.as_secs_f64() * 1000.0,
        t_zip.as_secs_f64() * 1000.0,
        t_check.as_secs_f64() * 1000.0,
        total_pixels,
        total_pass_count
    );
    
    // Debug logging to understand why skip_encoding is being set
    if diff_result.skip_encoding {
        tracing::warn!(
            "🦀 [IMAGE-QUICK-DIFF] ⚠️ skip_encoding=true: total_pass_count={}, lower_bound={}, upper_bound={}, should_pass={}",
            total_pass_count, samples_count_lower_bound, samples_count_upper_bound, should_pass
        );
    } else {
        tracing::debug!(
            "🦀 [IMAGE-QUICK-DIFF] ✅ skip_encoding=false: total_pass_count={}, lower_bound={}, upper_bound={}, should_pass={}",
            total_pass_count, samples_count_lower_bound, samples_count_upper_bound, should_pass
        );
    }
    
    Ok(())
}
