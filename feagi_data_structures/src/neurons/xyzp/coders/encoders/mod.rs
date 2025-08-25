//! Neural encoder implementations for converting I/O data to XYZP activations.
//!
//! This module contains concrete implementations of the `NeuronXYZPEncoder` trait,
//! each specialized for different types of input data and neural encoding strategies.
//! These encoders convert external sensor data into spatial patterns of neural
//! activation within cortical areas.
//!
//! # Available Encoders
//!
//! ## Continuous Value Encoders
//! - **F32LinearNeuronXYZPEncoder**: Direct linear mapping for normalized [0,1] values
//! - **F32PSPBidirectionalNeuronXYZPEncoder**: Post-synaptic potential encoding for [-1,1] values
//! - **F32SplitSignDividedNeuronXYZPEncoder**: Separate positive/negative populations for [-1,1] values
//!
//! ## Visual Data Encoders
//! - **ImageFrameNeuronXYZPEncoder**: Spatial encoding of 2D image data
//!
//! # Encoding Strategies
//!
//! ## Linear Encoding
//! Maps input values directly to neural activation levels, preserving proportional
//! relationships. Optimal for data that naturally represents intensity or magnitude.
//!
//! ## Bidirectional Encoding
//! Handles signed values using biological principles:
//! - **PSP Bidirectional**: Uses inhibitory/excitatory neural responses
//! - **Split Sign Divided**: Uses separate neural populations for positive/negative values
//!
//! ## Spatial Encoding
//! Preserves spatial relationships in input data by mapping to corresponding
//! neural coordinates, maintaining topological structure in the neural representation.
//!
//! # Performance Characteristics
//!
//! - **Linear encoders**: Fastest, minimal computation required
//! - **Bidirectional encoders**: Moderate overhead for sign handling
//! - **Visual encoders**: Higher computational cost but maintains spatial structure
//!
//! # Thread Safety
//! All encoder implementations are designed to be `Sync + Send` for safe use
//! in multi-threaded neural simulation environments.

mod image_frame;
mod f32_split_sign_divided;
mod f32_psp_bidirectional;
mod f32_linear;
mod segmented_image_frame;

pub use image_frame::{ImageFrameNeuronXYZPEncoder};
pub use f32_split_sign_divided::{F32SplitSignDividedNeuronXYZPEncoder};
pub use f32_psp_bidirectional::{F32PSPBidirectionalNeuronXYZPEncoder};
pub use f32_linear::{F32LinearNeuronXYZPEncoder};
pub use segmented_image_frame::{SegmentedImageFrameNeuronXYZPEncoder};