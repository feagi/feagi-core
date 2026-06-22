//! Values that hold some sort of decimal (float) value

use half::f16;
use crate::custom_data_types::StorageF8;
use crate::shared_traits::QuantizedElementBase;

/// Quantizable data for some decimal value (float)
pub trait QuantizedDecimalTrait: QuantizedElementBase {
    fn to_f32(self) -> f32;
    fn from_f32(value: f32) -> Self;

    fn load_f32_inplace(&mut self, value: f32);
}

impl QuantizedDecimalTrait for StorageF8 {
    fn to_f32(self) -> f32 {
        StorageF8::to_f32(self)
    }

    fn from_f32(value: f32) -> Self {
        StorageF8::from_f32(value)
    }

    fn load_f32_inplace(&mut self, value: f32) {
        *self = StorageF8::from_f32(value)
    }
}

impl QuantizedDecimalTrait for f16 {
    fn to_f32(self) -> f32 {
        f16::to_f32(self)
    }

    fn from_f32(value: f32) -> Self {
        f16::from_f32(value)
    }

    fn load_f32_inplace(&mut self, value: f32) {
        *self = f16::from_f32(value)
    }
}

// lol
impl QuantizedDecimalTrait for f32 {
    fn to_f32(self) -> f32 {
        self
    }

    fn from_f32(value: f32) -> Self {
        value
    }

    fn load_f32_inplace(&mut self, value: f32) {
        *self = value;
    }
}

#[cfg(feature = "support_64bit_values")]
impl QuantizedDecimalTrait for f64 {
    fn to_f32(self) -> f32 {
        self as f32
    }

    fn from_f32(value: f32) -> Self {
        value as f64
    }

    fn load_f32_inplace(&mut self, value: f32) {
        *self = value as f64
    }
}
