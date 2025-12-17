use std::ops::RangeInclusive;
use crate::define_stage_properties;
use crate::data_pipeline::stages::ImagePixelValueCountThresholdStage;
use crate::data_types::descriptors::ImageFrameProperties;
use crate::data_types::Percentage;
use crate::wrapped_io_data::WrappedIOType;

define_stage_properties! {
    /// Properties for ImagePixelValueCountThresholdStage checks for an image global pixel threshold
    name: ImagePixelValueCountThresholdStageProperties,

    fields: {
        input_definition: ImageFrameProperties,
        inclusive_pixel_range: RangeInclusive<u8>,
        acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>,
    },

    input_type: |s| WrappedIOType::ImageFrame(Some(s.input_definition)),
    output_type: |s| WrappedIOType::ImageFrame(Some(s.input_definition)),

    create_stage: |s| {
        ImagePixelValueCountThresholdStage::new_box(
            s.input_definition,
            s.inclusive_pixel_range.clone(),
            s.acceptable_amount_of_activity_in_image.clone(),
        ).unwrap()
    },

    display: (
        "ImageQuickDiffStageProperties(Input image definition: {:?}, pixels range: {:?}, Acceptable Activity Range: {:?})",
        input_definition,
        inclusive_pixel_range,
        acceptable_amount_of_activity_in_image,
    ),
}