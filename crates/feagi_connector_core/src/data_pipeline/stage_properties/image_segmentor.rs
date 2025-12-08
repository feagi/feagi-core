
use crate::{define_stage_properties};
use crate::data_pipeline::stages::ImageFrameSegmentatorStage;
use crate::data_types::{ImageFrameSegmentator};
use crate::data_types::descriptors::{ImageFrameProperties, SegmentedImageFrameProperties};
use crate::wrapped_io_data::WrappedIOType;

define_stage_properties! {
    /// Properties for ImageFrameSegmentatorStage that store configuration for image segmentation
    name: ImageFrameSegmentatorStageProperties,

    fields: {
        input_image_properties: ImageFrameProperties,
        output_image_properties: SegmentedImageFrameProperties,
        image_segmentator: ImageFrameSegmentator,
    },

    input_type: |s| WrappedIOType::ImageFrame(Some(s.input_image_properties)),
    output_type: |s| WrappedIOType::SegmentedImageFrame(Some(s.output_image_properties)),

    create_stage: |s| {
        ImageFrameSegmentatorStage::new_box(
            s.input_image_properties,
            s.output_image_properties
            s.image_segmentator.clone()
        ).unwrap()
    },

    display: (
        "ImageFrameSegmentatorStageProperties(input: {:?}, output: {:?})",
        input_image_properties,
        output_image_properties,
    ),
}
