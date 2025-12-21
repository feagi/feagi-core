use crate::data_pipeline::pipeline_stage::PipelineStage;
use crate::data_pipeline::PipelineStageProperties;
use crate::data_types::descriptors::ImageFrameProperties;
use crate::data_types::{ImageFrame, Percentage};
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use feagi_data_structures::FeagiDataError;
use ndarray::{Array3, Zip};
use rayon::prelude::*;
use std::any::Any;
use std::fmt::Display;
use std::ops::RangeInclusive;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ImagePixelValueCountThresholdStage {
    cache: WrappedIOData, // Image Frame
    input_definition: ImageFrameProperties,
    /// Minimum difference threshold for pixel changes to be considered significant
    inclusive_pixel_range: RangeInclusive<u8>,
    acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>,
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

    fn process_new_input(
        &mut self,
        value: &WrappedIOData,
        _time_of_input: Instant,
    ) -> Result<&WrappedIOData, FeagiDataError> {
        filter_and_set_if_pass(
            value,
            &mut self.cache,
            &self.inclusive_pixel_range,
            self.samples_count_lower_bound,
            self.samples_count_upper_bound,
        )?;
        Ok(&self.cache)
    }

    fn clone_box(&self) -> Box<dyn PipelineStage> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn create_properties(&self) -> PipelineStageProperties {
        PipelineStageProperties::ImagePixelValueCountThreshold {
            input_definition: self.input_definition,
            inclusive_pixel_range: self.inclusive_pixel_range.clone(),
            acceptable_amount_of_activity_in_image: self
                .acceptable_amount_of_activity_in_image
                .clone(),
        }
    }

    fn load_properties(
        &mut self,
        properties: PipelineStageProperties,
    ) -> Result<(), FeagiDataError> {
        match properties {
            PipelineStageProperties::ImagePixelValueCountThreshold {
                inclusive_pixel_range,
                acceptable_amount_of_activity_in_image,
                ..
            } => {
                if inclusive_pixel_range.is_empty() {
                    return Err(FeagiDataError::BadParameters("per_pixel_allowed_range appears to be empty! Are your bounds correct?".into()));
                }

                if acceptable_amount_of_activity_in_image.is_empty() {
                    return Err(FeagiDataError::BadParameters("acceptable_amount_of_activity_in_image appears to be empty! Are your bounds correct?".into()));
                }

                self.inclusive_pixel_range = inclusive_pixel_range;
                self.acceptable_amount_of_activity_in_image = acceptable_amount_of_activity_in_image;

                let sample_count_bounds = get_sample_count_lower_upper_bounds(&self.acceptable_amount_of_activity_in_image, self.input_definition.get_number_of_channels());
                self.samples_count_lower_bound = sample_count_bounds.0;
                self.samples_count_upper_bound = sample_count_bounds.1;
                Ok(())
            }
            _ => Err(FeagiDataError::BadParameters(
                "load_properties called with incompatible properties type for ImagePixelValueCountThresholdStage".into()
            ))
        }
    }
}

impl ImagePixelValueCountThresholdStage {
    pub fn new(
        image_properties: ImageFrameProperties,
        per_pixel_allowed_range: RangeInclusive<u8>,
        acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>,
    ) -> Result<Self, FeagiDataError> {
        if per_pixel_allowed_range.is_empty() {
            return Err(FeagiDataError::BadParameters(
                "per_pixel_allowed_range appears to be empty! Are your bounds correct?".into(),
            ));
        }

        if acceptable_amount_of_activity_in_image.is_empty() {
            return Err(FeagiDataError::BadParameters("acceptable_amount_of_activity_in_image appears to be empty! Are your bounds correct?".into()));
        }

        let sample_count_bounds = get_sample_count_lower_upper_bounds(
            &acceptable_amount_of_activity_in_image,
            image_properties.get_number_of_channels(),
        );

        Ok(ImagePixelValueCountThresholdStage {
            cache: WrappedIOData::ImageFrame(ImageFrame::new_from_image_frame_properties(
                &image_properties,
            )?),
            input_definition: image_properties,
            inclusive_pixel_range: per_pixel_allowed_range,
            acceptable_amount_of_activity_in_image: acceptable_amount_of_activity_in_image.clone(),
            samples_count_lower_bound: sample_count_bounds.0,
            samples_count_upper_bound: sample_count_bounds.1,
        })
    }

    pub fn new_box(
        image_properties: ImageFrameProperties,
        per_pixel_allowed_range: RangeInclusive<u8>,
        acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>,
    ) -> Result<Box<dyn PipelineStage + Send + Sync + 'static>, FeagiDataError> {
        Ok(Box::new(ImagePixelValueCountThresholdStage::new(
            image_properties,
            per_pixel_allowed_range,
            acceptable_amount_of_activity_in_image,
        )?))
    }
}

fn filter_and_set_if_pass(
    source: &WrappedIOData,
    filter_result: &mut WrappedIOData,
    pixel_bounds: &RangeInclusive<u8>,
    samples_count_lower_bound: usize,
    samples_count_upper_bound: usize,
) -> Result<(), FeagiDataError> {
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
            *filter = if source >= pixel_val_lower_bound && source <= pixel_val_upper_bound {
                source
            } else {
                0
            };
            passed as usize
        })
        .into_par_iter()
        .sum();
    let should_pass = total_pass_count >= samples_count_lower_bound
        && total_pass_count <= samples_count_upper_bound;
    filter_result.skip_encoding = !should_pass || filter_result.skip_encoding;
    Ok(())
}

fn get_sample_count_lower_upper_bounds(
    acceptable_amount_of_activity_in_image: &RangeInclusive<Percentage>,
    number_channels: usize,
) -> (usize, usize) {
    (
        (acceptable_amount_of_activity_in_image.start().get_as_0_1() * number_channels as f32)
            as usize,
        (acceptable_amount_of_activity_in_image.end().get_as_0_1() * number_channels as f32)
            as usize,
    )
}
