//! This module defines various quantizable structs with shared traits.
//! Quantization is the act of representing the same information with varying amounts of bits,
//! with there being a tradeoff between memory usage and precision/representable range.

mod quantization_levels;
mod feagi_data_quantized_error;
pub mod base_types;
pub mod spatial;
pub mod collections;
mod shared_traits;

pub use feagi_data_quantized_error::FeagiDataQuantizedError;
pub use quantization_levels::*;
pub use shared_traits::*;
