#![allow(unused_imports)] // These are used, but by macros that some IDEs cannot see

mod gaze_properties_exponential;
mod gaze_properties_linear;
mod misc_data;
mod percentage_decoder;

pub(crate) use gaze_properties_exponential::GazePropertiesExponentialNeuronVoxelXYZPDecoder;
pub(crate) use gaze_properties_linear::GazePropertiesLinearNeuronVoxelXYZPDecoder;
pub(crate) use misc_data::MiscDataNeuronVoxelXYZPDecoder;

// Percentage decoder (uses PercentageNeuronPositioning and PercentageChannelDimensionality from other modules)
pub(crate) use percentage_decoder::PercentageNeuronVoxelXYZPDecoder;
