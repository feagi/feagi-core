//! Neuron voxel encoding and decoding systems.
//!
//! Provides encoders and decoders for converting between application data types
//! (percentages, images, etc.) and FEAGI's neuron voxel representations (XYZP format).
//! Supports linear and exponential encoding strategies for various dimensionalities.

mod coder_types;
pub(crate) mod xyzp;
