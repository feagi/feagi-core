//! Configuration properties for pipeline stages.
//!
//! Defines property structures that configure how pipeline stages behave.
//! Properties are serializable and can be dynamically updated at runtime.
//!
//! Each property type corresponds to a stage in [`crate::data_pipeline::stages`].

mod identities;
mod image_segmentor;
//mod image_quick_diff;

pub use identities::IdentityStageProperties;
pub use image_segmentor::ImageSegmentorStageProperties;
//pub use image_quick_diff::ImageQuickDiffStageProperties;