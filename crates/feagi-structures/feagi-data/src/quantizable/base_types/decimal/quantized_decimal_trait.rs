use half::f16;
use crate::quantizable::base_types::decimal::custom_data_types::StorageF8;
use crate::quantizable::base_types::quantized_base_traits::QuantizedElementBase;

pub trait QuantizedDecimalTrait: QuantizedElementBase {
    fn to_f32(self) -> f32;
    fn from_f32(value: f32) -> Self;

    fn load_f32_inplace(&mut self, value: f32);
    
    /// Creates a new value from an average of multiple given
    fn from_average_of_slice(slice: &[Self]) -> Self {
        let sum = slice.iter()
            .fold(
                Self::QUANT_ZERO,
                |v, &n|
                    v.add(n)
            );
        sum / Self::from_f32(slice.len() as f32)
    }
    
    /// For rapidly averaging and loading into place. Must pass in slice length as a decimal
    fn load_average_of_slice_inplace_fast(&mut self, slice: &[Self], len_as_dec: &Self) {
        *self = slice.iter()
            .fold(
                Self::QUANT_ZERO,
                |v, &n|
                    v.add(n)
            );
        *self = *self / *len_as_dec;
    }
    
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


/// Something all wrappers share, for easy data access
pub trait QuantizedDecimalWrapperTrait<QuantDeci: QuantizedDecimalTrait>:
Copy
+ Clone
+ core::ops::Add<Output = Self>
+ core::ops::Sub<Output = Self>
+ core::ops::Mul<Output = Self>
+ core::ops::Div<Output = Self>
+ core::ops::AddAssign
+ core::ops::SubAssign
+ core::ops::MulAssign
+ core::ops::DivAssign
+ core::cmp::PartialOrd
{
    fn wrap_quant(quant: QuantDeci) -> Self;
    fn quant(self) -> QuantDeci;
    fn quant_ref(&self) -> &QuantDeci;
    fn quant_mut(&mut self) -> &mut QuantDeci;
}