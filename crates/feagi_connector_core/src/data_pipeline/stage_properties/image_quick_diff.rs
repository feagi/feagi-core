use std::ops::RangeInclusive;
use crate::define_stage_properties;
use crate::data_pipeline::stages::ImageFrameQuickDiffStage;
use crate::data_types::descriptors::ImageFrameProperties;
use crate::data_types::Percentage;
use crate::wrapped_io_data::WrappedIOType;

define_stage_properties! {
    /// Properties for ImageFrameQuickDiffStage that configures quick difference detection
    /// between consecutive image frames.
    name: ImageQuickDiffStageProperties,

    fields: {
        per_pixel_allowed_range: RangeInclusive<u8>,
        acceptable_amount_of_activity_in_image: RangeInclusive<Percentage>,
        image_properties: ImageFrameProperties,
    },

    input_type: |s| WrappedIOType::ImageFrame(Some(s.image_properties)),
    output_type: |s| WrappedIOType::ImageFrame(Some(s.image_properties)),

    create_stage: |s| {
        ImageFrameQuickDiffStage::new_box(
            s.image_properties,
            s.per_pixel_allowed_range.clone(),
            s.acceptable_amount_of_activity_in_image.clone()
        ).unwrap()
    },

    display: (
        "ImageQuickDiffStageProperties(per pixel allow range: {:?}, acceptable_amount_of_activity_in_image: {:?}, image_properties: {:?})",
        per_pixel_allowed_range,
        acceptable_amount_of_activity_in_image,
        image_properties,
    ),
}