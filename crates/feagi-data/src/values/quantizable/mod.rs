//! This module defines various quantizable structs with shared traits.
//! Quantization is the act of representing the same information with varying amounts of bits,
//! with there being a tradeoff between memory usage and precision/representable range.

mod base_traits;
mod decimal;
mod index_count;
mod percentage_unsigned;
mod quantization_level_packing;
mod signed_integer;
mod unsigned_integer;

pub mod custom_data_types;
pub mod feagi_data_value_quantization_error;

pub use base_traits::QuantizedElementBase;
pub use decimal::{DecimalEnum, DecimalQuantizationLevel, QuantizedDecimalTrait, WrappedQuantizedDecimal, WrappedQuantizedDecimalEnum};
pub use feagi_data_value_quantization_error::FeagiDataValueQuantizationError;
pub use index_count::{IndexCountEnum, IndexCountQuantizationLevel, QuantizedIndexCountTrait, WrappedQuantizedIndexCount, WrappedQuantizedIndexCountEnum};
pub use percentage_unsigned::{PercentageUnsigned, WrappedPercentageUnsigned};
pub use quantization_level_packing::QuantizationLevelPacking;
pub use signed_integer::{QuantizedSignedIntegerTrait, SignedIntegerQuantizationLevel};
pub use unsigned_integer::{QuantizedUnsignedIntegerTrait, UnsignedIntegerQuantizationLevel};
