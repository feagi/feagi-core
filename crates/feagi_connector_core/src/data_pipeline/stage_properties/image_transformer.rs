use crate::data_pipeline::stages::ImageFrameProcessorStage;
use crate::define_stage_properties;
use crate::data_types::{ImageFrameProcessor};
use crate::wrapped_io_data::WrappedIOType;

define_stage_properties! {
    /// Properties for ImageFrameProcessorStage that configures various image modification transformations
    name: ImageFrameProcessorStageProperties,

    fields: {
        transformer_definition: ImageFrameProcessor
    },

    input_type: |s| WrappedIOType::ImageFrame(Some(*s.transformer_definition.get_input_image_properties())),
    output_type: |s| WrappedIOType::ImageFrame(Some(s.transformer_definition.get_output_image_properties())),

    create_stage: |s| {
        ImageFrameProcessorStage::new_box(
            s.transformer_definition.clone()
        ).unwrap()
    },

    display: (
        "ImageFrameProcessorStageProperties(Transformer: {:?}",
        transformer_definition
    ),
}