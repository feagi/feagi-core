//! This module defines various quantizable structs with shared traits.
//! Quantization is the act of representing the same information with varying amounts of bits,
//! with there being a tradeoff between memory usage and precision/representable range.

mod base_traits;
mod decimal;
mod index_count;
mod quantization_levels;
mod signed_integer;
mod unsigned_integer;

pub mod custom_data_types;

pub use base_traits::QuantizedElementBase;
pub use decimal::QuantizedDecimalTrait;
pub use index_count::QuantizedIndexCountTrait;
pub use quantization_levels::QuantizationLevel;
pub use signed_integer::QuantizedSignedIntegerTrait;
pub use unsigned_integer::QuantizedUnsignedIntegerTrait;
