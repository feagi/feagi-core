
use std::fmt::Display;
use std::time::Instant;
use ndarray::{Array3, Zip};
use feagi_data_structures::data::image_descriptors::ImageFrameProperties;
use feagi_data_structures::data::ImageFrame;
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
    threshold: u8,
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
        quick_diff(value, &self.previous_frame_cache, &mut self.diff_cache, self.threshold)?;
        self.previous_frame_cache = value.clone();
        Ok(&self.diff_cache)
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }
}

impl ImageFrameQuickDiffStage {
    /// Creates a new ImageFrameQuickDiffProcessor with specified properties and threshold.
    ///
    /// Initializes the processor with three internal image buffers (two for alternating storage
    /// and one for the output difference) all matching the specified properties. The threshold
    /// determines the minimum pixel difference required for changes to be considered significant.
    ///
    /// # Arguments
    ///
    /// * `image_properties` - Properties defining the input image format (resolution, color space, channels)
    /// * `threshold` - Minimum difference threshold for pixel changes
    ///
    /// # Returns
    ///
    /// * `Ok(ImageFrameQuickDiffProcessor)` - Successfully created processor
    /// * `Err(FeagiDataError)` - If threshold is negative or image creation fails
    pub fn new(image_properties: ImageFrameProperties, threshold: u8) -> Result<Self, FeagiDataError> {
        
        let cache_image = ImageFrame::new_from_image_frame_properties(&image_properties)?;
        Ok(ImageFrameQuickDiffStage {
            diff_cache: WrappedIOData::ImageFrame(cache_image.clone()),
            previous_frame_cache: WrappedIOData::ImageFrame(cache_image.clone()), // Image Frame
            input_definition: image_properties,
            threshold,
        })
    }
}

fn quick_diff(minuend: &WrappedIOData, subtrahend: &WrappedIOData, diff_result: &mut WrappedIOData, threshold: u8) -> Result<(), FeagiDataError> {
    let minuend: &ImageFrame = minuend.try_into()?;
    let subtrahend: &ImageFrame = subtrahend.try_into()?;
    let diff_result: &mut ImageFrame = diff_result.try_into()?;

    let minuend_arr: &Array3<u8> = minuend.get_internal_data();
    let subtrahend_arr: &Array3<u8> = subtrahend.get_internal_data();
    let diff_arr: &mut Array3<u8> = diff_result.get_internal_data_mut();

    Zip::from(minuend_arr)
        .and(subtrahend_arr)
        .and(diff_arr)
        .par_for_each(|&minuend, &subtrahend, diff| {
            let absolute_diff = if minuend >= subtrahend {
                minuend - subtrahend
            } else {
                subtrahend - minuend
            };
            *diff = if absolute_diff >= threshold { subtrahend } else { 0 };
        });

    Ok(())
}


