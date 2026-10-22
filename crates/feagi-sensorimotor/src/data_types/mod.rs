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

mod audio_spectrum;
pub mod descriptors;
mod gaze_properties;
mod image_filtering_settings;
mod image_frame;
mod misc_data;
mod percentages;
mod pose_estimation;
pub mod processing;
mod raw_imu;
mod segmented_image_frame;
pub mod text_token;

pub use audio_spectrum::{
    AudioFrequencySpacing, AudioSpectrumFrame, AudioSpectrumProperties,
    AUDIO_SPECTRUM_MAX_BIN_COUNT, AUDIO_SPECTRUM_MAX_PHASE_STEPS, AUDIO_SPECTRUM_MAX_WINDOW_SIZE,
    AUDIO_SPECTRUM_MIN_WINDOW_SIZE,
};
pub use gaze_properties::GazeProperties;
pub use image_filtering_settings::ImageFilteringSettings;
pub use image_frame::ImageFrame;
pub use misc_data::MiscData;
pub use percentages::*;
pub use pose_estimation::{JointPosition, PoseEstimationData};
pub(crate) use processing::*;
pub use raw_imu::{
    RawIMU, RAW_IMU_INDEX_ACCELEROMETER, RAW_IMU_INDEX_GYROSCOPE, RAW_IMU_INDEX_MAGNETOMETER,
    RAW_IMU_SUBUNIT_COUNT,
};
pub use segmented_image_frame::SegmentedImageFrame;
pub use text_token::{
    decode_token_id_from_misc_data, decode_token_id_from_xyzp_bitplanes,
    encode_token_id_to_misc_data, encode_token_id_to_xyzp_bitplanes, TextToken,
};
