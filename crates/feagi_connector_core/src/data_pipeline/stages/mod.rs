//! Concrete pipeline stage implementations.
//!
//! Provides actual processing stages that transform data flowing through
//! the FEAGI connector pipeline. Each stage has corresponding properties
//! defined in [`crate::data_pipeline::stage_properties`].
mod image_segmentor;
mod image_quick_diff;
mod image_transformer;
mod image_pixel_value_count_threshold;

pub use image_segmentor::ImageFrameSegmentatorStage;
pub use image_quick_diff::ImageFrameQuickDiffStage;
pub use image_transformer::ImageFrameProcessorStage;
pub use image_pixel_value_count_threshold::ImagePixelValueCountThresholdStage;