//! This module defines various quantizable structs with shared traits.
//! Quantization is the act of representing the same information with varying amounts of bits,
//! with there being a tradeoff between memory usage and precision/representable range.

mod base_traits;
mod decimal;
mod quantization_level_packing;
mod signed_integer;
mod unsigned_integer;
mod unsigned_percentage;

pub mod custom_data_types;
pub mod feagi_data_value_quantization_error;

pub use base_traits::QuantizedElementBase;
pub use decimal::{
    DecimalEnum, DecimalQuantizationLevel, QuantizedDecimalTrait, QuantizedDecimalUnwrappedTrait, QuantizedDecimalWrappedTrait,
    WrappedQuantizedDecimalEnum,
};
pub use feagi_data_value_quantization_error::FeagiDataValueQuantizationError;
pub use quantization_level_packing::QuantizationLevelPacking;
pub use signed_integer::{
    QuantizedSignedIntegerTrait, QuantizedSignedIntegerUnwrappedTrait, QuantizedSignedIntegerWrappedTrait, SignedIntegerEnum,
    SignedIntegerQuantizationLevel, WrappedQuantizedSignedIntegerEnum,
};
pub use unsigned_integer::{
    QuantizedUnsignedIntegerTrait, QuantizedUnsignedIntegerUnwrappedTrait, QuantizedUnsignedIntegerWrappedTrait, UnsignedIntegerEnum,
    UnsignedIntegerQuantizationLevel, WrappedQuantizedUnsignedIntegerEnum,
};
pub use unsigned_percentage::{
    PercentageUnsigned, QuantizedUnsignedPercentageTrait, QuantizedUnsignedPercentageUnwrappedTrait, QuantizedUnsignedPercentageWrappedTrait,
};
