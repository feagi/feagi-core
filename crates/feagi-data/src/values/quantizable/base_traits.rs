use crate::values::quantizable::custom_data_types::StorageF8;
use half::{bf16, f16};

/// Common base for all quantizable types
#[doc(hidden)]
pub trait QuantizedElementBase {
    const QUANT_ZERO: Self;
}

// We need to support this for the indexer
impl QuantizedElementBase for usize {
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for u8 {
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for u16 {
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for u32 {
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for u64 {
    const QUANT_ZERO: Self = 0;
}

// Lol no we are not doing u128 or i128

impl QuantizedElementBase for isize {
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for i8 {
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for i16 {
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for i32 {
    const QUANT_ZERO: Self = 0;
}

impl QuantizedElementBase for i64 {
    const QUANT_ZERO: Self = 0;
}

// A bad choice for computation
impl QuantizedElementBase for StorageF8 {
    const QUANT_ZERO: Self = StorageF8::ZERO;
}

impl QuantizedElementBase for f16 {
    const QUANT_ZERO: Self = f16::ZERO;
}

impl QuantizedElementBase for bf16 {
    const QUANT_ZERO: Self = bf16::ZERO;
}

impl QuantizedElementBase for f32 {
    const QUANT_ZERO: Self = 0.0;
}

impl QuantizedElementBase for f64 {
    const QUANT_ZERO: Self = 0.0;
}
