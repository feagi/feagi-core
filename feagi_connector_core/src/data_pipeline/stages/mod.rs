//! Concrete pipeline stage implementations.
//!
//! Provides actual processing stages that transform data flowing through
//! the FEAGI connector pipeline. Each stage has corresponding properties
//! defined in [`crate::data_pipeline::stage_properties`].
//!
//! Available stages:
//! - **[`IdentityStage`]** - Pass-through stage (no transformation)
//! - **[`ImageFrameSegmentatorStage`]** - Segments images for peripheral vision

mod identities;
mod image_segmentor;
mod image_quick_diff;
/*
mod rolling_windows;
mod ranges;
mod image_transformer;
mod image_quick_diff;
mod image_pixel_value_count_threshold;

 */

pub use identities::IdentityStage;
pub use image_segmentor::ImageFrameSegmentatorStage;
pub use image_quick_diff::ImageFrameQuickDiffStage;
/*
pub use rolling_windows::*;
pub use ranges::*;
pub use image_transformer::*;
pub use image_quick_diff::*;
pub use image_pixel_value_count_threshold::*;

 */