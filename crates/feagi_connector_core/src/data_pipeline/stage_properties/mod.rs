//! Configuration properties for pipeline stages.
//!
//! Defines property structures that configure how pipeline stages behave.
//! Properties are serializable and can be dynamically updated at runtime.
//!
//! Each property type corresponds to a stage in [`crate::data_pipeline::stages`].

mod image_segmentor;
mod image_quick_diff;
mod image_pixel_value_count_threshold;
mod image_transformer;

pub use image_segmentor::ImageFrameSegmentatorStageProperties;
pub use image_quick_diff::ImageQuickDiffStageProperties;