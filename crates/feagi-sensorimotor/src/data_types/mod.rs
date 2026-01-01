//! Data types for sensor inputs and motor outputs.
//!
//! Provides specialized data structures for various types of sensory and motor data:
//!
//! - **[`ImageFrame`]** - Raw image data with color space support
//! - **[`SegmentedImageFrame`]** - Images with segmentation labels
//! - **[`MiscData`]** - Generic multi-dimensional data arrays
//! - **[`Percentage`]** and variants - Normalized values in various dimensionalities
//! - **[`SignedPercentage`]** and variants - Signed normalized values (-1 to 1)
//!
//! These types handle memory layout, color space conversions, and provide
//! efficient interfaces for common sensor/actuator data formats.

pub mod descriptors;
mod gaze_properties;
mod image_frame;
mod misc_data;
mod percentages;
pub mod text_token;
pub mod processing;
mod segmented_image_frame;

pub use gaze_properties::GazeProperties;
pub use image_frame::ImageFrame;
pub use misc_data::MiscData;
pub use percentages::*;
pub use text_token::{
    decode_token_id_from_misc_data,
    decode_token_id_from_xyzp_bitplanes,
    encode_token_id_to_misc_data,
    encode_token_id_to_xyzp_bitplanes,
    TextToken,
};
pub(crate) use processing::*;
pub use segmented_image_frame::SegmentedImageFrame;
