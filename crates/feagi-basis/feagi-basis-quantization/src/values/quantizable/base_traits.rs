//! Base traits deployed by all Quantized types

// TODO Add FeagiDisplay, FeagiDebug traits, and force them here

use half::{bf16, f16};
use serde::{Serialize};
use serde::de::DeserializeOwned;

/// Common base for all quantizable types
#[doc(hidden)]
pub trait QuantizedElementBase:
    Copy + Clone + Send + Sync + Default + core::fmt::Debug + core::fmt::Display
    + core::cmp::PartialEq + core::cmp::PartialOrd + Sized + Serialize + DeserializeOwned + 'static
{
    const QUANT_ZERO: Self;
    const QUANT_ONE: Self;
}

// We need to support this for the indexer
impl QuantizedElementBase for usize {
    const QUANT_ZERO: Self = 0;
    const QUANT_ONE: Self = 1;
}

impl QuantizedElementBase for u8 {
    const QUANT_ZERO: Self = 0;
    const QUANT_ONE: Self = 1;
}

impl QuantizedElementBase for u16 {
    const QUANT_ZERO: Self = 0;
    const QUANT_ONE: Self = 1;
}

impl QuantizedElementBase for u32 {
    const QUANT_ZERO: Self = 0;
    const QUANT_ONE: Self = 1;
}

impl QuantizedElementBase for u64 {
    const QUANT_ZERO: Self = 0;
    const QUANT_ONE: Self = 1;
}

// Lol no we are not doing u128 or i128

impl QuantizedElementBase for isize {
    const QUANT_ZERO: Self = 0;
    const QUANT_ONE: Self = 1;
}

impl QuantizedElementBase for i8 {
    const QUANT_ZERO: Self = 0;
    const QUANT_ONE: Self = 1;
}

impl QuantizedElementBase for i16 {
    const QUANT_ZERO: Self = 0;
    const QUANT_ONE: Self = 1;
}

impl QuantizedElementBase for i32 {
    const QUANT_ZERO: Self = 0;
    const QUANT_ONE: Self = 1;
}

impl QuantizedElementBase for i64 {
    const QUANT_ZERO: Self = 0;
    const QUANT_ONE: Self = 1;
}

impl QuantizedElementBase for f16 {
    const QUANT_ZERO: Self = f16::ZERO;
    const QUANT_ONE: Self = f16::ONE;
}

impl QuantizedElementBase for bf16 {
    const QUANT_ZERO: Self = bf16::ZERO;
    const QUANT_ONE: Self = bf16::ONE;
}

impl QuantizedElementBase for f32 {
    const QUANT_ZERO: Self = 0.0;
    const QUANT_ONE: Self = 1.0;
}

impl QuantizedElementBase for f64 {
    const QUANT_ZERO: Self = 0.0;
    const QUANT_ONE: Self = 1.0;
}

pub(crate) mod sealed {
    /// Prevent the creation of more unwrapped types
    pub trait QuantizedUnwrappedSeal {}
}