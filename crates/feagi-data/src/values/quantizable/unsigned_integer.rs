//! The single quantized unsigned integer type family.
//!
//! Indexes, counts and plain unsigned data all quantize identically: an index into a collection
//! and the length of that same collection have to be representable at the same width, or the two
//! could not be compared. This module therefore owns one trait, one quantization level and one
//! value enum for all three roles. What a given value *means* is expressed by the wrapper types in
//! [`unsigned_wrappers`](crate::values::quantizable), not by its quantization.

use crate::values::quantizable::feagi_data_value_quantization_error::FeagiFailQuantizationOutOfRange;
use crate::values::quantizable::{FeagiDataValueQuantizationError, QuantizationLevelPacking, QuantizedElementBase};
use serde::Serialize;

/// The width an unsigned integer is quantized to.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum UnsignedIntegerQuantizationLevel {
    U8 = 0,
    U16 = 1,
    U32 = 2,
    U64 = 3,
    // we are NOT doing u128 lol
    Usize = 4,
    // We can support a max of 8
}

impl From<UnsignedIntegerQuantizationLevel> for u8 {
    fn from(value: UnsignedIntegerQuantizationLevel) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for UnsignedIntegerQuantizationLevel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(UnsignedIntegerQuantizationLevel::U8),
            1 => Ok(UnsignedIntegerQuantizationLevel::U16),
            2 => Ok(UnsignedIntegerQuantizationLevel::U32),
            3 => Ok(UnsignedIntegerQuantizationLevel::U64),
            4 => Ok(UnsignedIntegerQuantizationLevel::Usize),
            _ => Err(()),
        }
    }
}

impl QuantizationLevelPacking for UnsignedIntegerQuantizationLevel {
    const NUMBER_BITS: usize = 3;

    unsafe fn from_unpacked_byte(byte: u8) -> Self {
        core::mem::transmute(byte)
    }
}

/// Trait designed to hold unsigned integer values in a quantized form. Implemented by the
/// primitive unsigned integers only; give a value its meaning by putting it in one of the
/// wrapper families rather than by choosing a different quantization.
pub trait QuantizedUnsignedIntegerTrait:
    Copy
    + Clone
    + Send
    + Sync
    + Default
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::AddAssign
    + core::ops::SubAssign
    + core::ops::MulAssign
    + core::ops::DivAssign
    + core::ops::BitAnd<Output = Self>
    + core::ops::BitOr<Output = Self>
    + core::ops::BitXor<Output = Self>
    + core::ops::Shl<Output = Self>
    + core::ops::Shr<Output = Self>
    + core::ops::BitAndAssign
    + core::ops::BitOrAssign
    + core::ops::BitXorAssign
    + core::cmp::PartialOrd
    + core::cmp::Ord
    + core::iter::Sum
    + core::fmt::Debug
    + core::fmt::Display
    + core::ops::Rem<Output = Self>
    + core::ops::RemAssign
    + core::cmp::Eq
    + core::hash::Hash
    + Sized
    + 'static
    + QuantizedElementBase
    + Serialize // TODO should be conditional
{
    const LEVEL: UnsignedIntegerQuantizationLevel;
    const QUANT_MAX: Self;

    const QUANT_MAX_U8: Self;
    const QUANT_MAX_U16: Self;
    const QUANT_MAX_U32: Self;
    const QUANT_MAX_U64: Self;
    const QUANT_MAX_USIZE: usize;

    const QUANT_CLAMPED_U8: u8;
    const QUANT_CLAMPED_U16: u16;
    const QUANT_CLAMPED_U32: u32;
    const QUANT_CLAMPED_U64: u64;
    const QUANT_CLAMPED_USIZE: usize;

    /// Mask for the first 3 bits
    const QUANT_BYTE_BIT_MASK: Self;

    /// Tries to convert from usize, does NOT check bounds!
    fn quant_from_usize(value: usize) -> Self;

    /// Will wrap whatever quant this is to an `UnsignedIntegerEnum`
    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum;

    /// Tries converting from usize, returns an error if out of bounds
    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError>;

    /// Converts to usize. No need to check as we have no values that will exceed a usize on a
    /// system
    fn quant_to_usize(self) -> usize;

    /// Tries to convert to u8, does NOT check bounds!
    fn quant_to_u8(self) -> u8;

    /// Tries to convert to u16, does NOT check bounds!
    fn quant_to_u16(self) -> u16;

    /// Tries to convert to u32, does NOT check bounds!
    fn quant_to_u32(self) -> u32;

    /// Tries to convert to u64, bound checking shouldnt matter since this is the biggest type (not doing u128 lol)
    fn quant_to_u64(self) -> u64;

    /// Creates from a value of another quantization. Does not check for validity of ranges!
    fn from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self;

    /// Creates from a value of another quantization, clamping its value to ensure it fits
    fn from_quantization_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self;

    /// Tries to create from a value of another quantization, returns an error if it would break the bounds
    fn try_from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError>;

    /// Converts to another quantization. Does not check for validity of ranges!
    fn to_quantization<ToQuant: QuantizedUnsignedIntegerTrait>(self) -> ToQuant {
        ToQuant::from_quantization::<Self>(self)
    }

    /// Converts to another quantization, clamping its value to ensure it fits
    fn to_quantization_clamped<ToQuant: QuantizedUnsignedIntegerTrait>(self) -> ToQuant {
        ToQuant::from_quantization_clamped(self)
    }

    /// Tries to convert to another quantization, returns an error if it would break the bounds
    fn try_to_quantization<ToQuant: QuantizedUnsignedIntegerTrait>(self) -> Result<ToQuant, FeagiDataValueQuantizationError> {
        ToQuant::try_from_quantization(self)
    }

    /// Clamps this value for another quantization, but does not actually change the
    /// quantization itself
    fn clamp_for_quantization<ClampFor: QuantizedUnsignedIntegerTrait>(self) -> Self;

    fn clamp_for_quantization_level_runtime(self, level: UnsignedIntegerQuantizationLevel) -> Self;
}

impl QuantizedUnsignedIntegerTrait for u8 {
    const LEVEL: UnsignedIntegerQuantizationLevel = UnsignedIntegerQuantizationLevel::U8;
    const QUANT_MAX: Self = u8::MAX;
    const QUANT_MAX_U8: Self = u8::MAX;
    const QUANT_MAX_U16: Self = u8::MAX;
    const QUANT_MAX_U32: Self = u8::MAX;
    const QUANT_MAX_U64: Self = u8::MAX;
    const QUANT_MAX_USIZE: usize = u8::MAX as usize;

    const QUANT_CLAMPED_U8: u8 = u8::MAX;
    const QUANT_CLAMPED_U16: u16 = u8::MAX as u16;
    const QUANT_CLAMPED_U32: u32 = u8::MAX as u32;
    const QUANT_CLAMPED_U64: u64 = u8::MAX as u64;
    const QUANT_CLAMPED_USIZE: usize = u8::MAX as usize;

    const QUANT_BYTE_BIT_MASK: Self = 0b0000_0111;

    fn quant_from_usize(value: usize) -> Self {
        value as u8
    }

    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U8(value)
    }

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u8::MAX as usize {
            return Err(FeagiFailQuantizationOutOfRange::new("Given value cannot fit in a quantized u8!", value).into());
        }
        Ok(value as u8)
    }

    fn quant_to_usize(self) -> usize {
        self as usize
    }

    fn quant_to_u8(self) -> u8 {
        self
    }

    fn quant_to_u16(self) -> u16 {
        self as u16
    }

    fn quant_to_u32(self) -> u32 {
        self as u32
    }

    fn quant_to_u64(self) -> u64 {
        self as u64
    }

    fn from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_u8()
    }

    fn from_quantization_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        if value > FromQuant::QUANT_MAX_U8 {
            return u8::MAX;
        }
        value.quant_to_u8()
    }

    fn try_from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > FromQuant::QUANT_MAX_U8 {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized value exceeds u8 quantization!", value.quant_to_usize()).into());
        }
        Ok(value.quant_to_u8())
    }

    fn clamp_for_quantization<ClampFor: QuantizedUnsignedIntegerTrait>(self) -> Self {
        self.min(ClampFor::QUANT_CLAMPED_U8)
    }

    fn clamp_for_quantization_level_runtime(self, level: UnsignedIntegerQuantizationLevel) -> Self {
        match level {
            UnsignedIntegerQuantizationLevel::U8 => self,
            UnsignedIntegerQuantizationLevel::U16 => self,
            UnsignedIntegerQuantizationLevel::U32 => self,
            UnsignedIntegerQuantizationLevel::U64 => self,
            UnsignedIntegerQuantizationLevel::Usize => self,
        }
    }
}

impl QuantizedUnsignedIntegerTrait for u16 {
    const LEVEL: UnsignedIntegerQuantizationLevel = UnsignedIntegerQuantizationLevel::U16;
    const QUANT_MAX: Self = u16::MAX;
    const QUANT_MAX_U8: Self = u8::MAX as u16;
    const QUANT_MAX_U16: Self = u16::MAX;
    const QUANT_MAX_U32: Self = u16::MAX;
    const QUANT_MAX_U64: Self = u16::MAX;
    const QUANT_MAX_USIZE: usize = u16::MAX as usize;

    const QUANT_CLAMPED_U8: u8 = u8::MAX;
    const QUANT_CLAMPED_U16: u16 = u16::MAX;
    const QUANT_CLAMPED_U32: u32 = u16::MAX as u32;
    const QUANT_CLAMPED_U64: u64 = u16::MAX as u64;
    const QUANT_CLAMPED_USIZE: usize = u16::MAX as usize;

    const QUANT_BYTE_BIT_MASK: Self = 0b0000_0000_0000_0111;

    fn quant_from_usize(value: usize) -> Self {
        value as u16
    }

    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U16(value)
    }

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u16::MAX as usize {
            return Err(FeagiFailQuantizationOutOfRange::new("Given value cannot fit in a quantized u16!", value).into());
        }
        Ok(value as u16)
    }

    fn quant_to_usize(self) -> usize {
        self as usize
    }

    fn quant_to_u8(self) -> u8 {
        self as u8
    }

    fn quant_to_u16(self) -> u16 {
        self
    }

    fn quant_to_u32(self) -> u32 {
        self as u32
    }

    fn quant_to_u64(self) -> u64 {
        self as u64
    }

    fn from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_u16()
    }

    fn from_quantization_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        if value > FromQuant::QUANT_MAX_U16 {
            return u16::MAX;
        }
        value.quant_to_u16()
    }

    fn try_from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > FromQuant::QUANT_MAX_U16 {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized value exceeds u16 quantization!", value.quant_to_usize()).into());
        }
        Ok(value.quant_to_u16())
    }

    fn clamp_for_quantization<ClampFor: QuantizedUnsignedIntegerTrait>(self) -> Self {
        self.min(ClampFor::QUANT_CLAMPED_U16)
    }

    fn clamp_for_quantization_level_runtime(self, level: UnsignedIntegerQuantizationLevel) -> Self {
        match level {
            UnsignedIntegerQuantizationLevel::U8 => self.min(255),
            UnsignedIntegerQuantizationLevel::U16 => self,
            UnsignedIntegerQuantizationLevel::U32 => self,
            UnsignedIntegerQuantizationLevel::U64 => self,
            UnsignedIntegerQuantizationLevel::Usize => self,
        }
    }
}

// lol, lmao even
impl QuantizedUnsignedIntegerTrait for u32 {
    const LEVEL: UnsignedIntegerQuantizationLevel = UnsignedIntegerQuantizationLevel::U32;
    const QUANT_MAX: Self = u32::MAX;
    const QUANT_MAX_U8: Self = u8::MAX as u32;
    const QUANT_MAX_U16: Self = u16::MAX as u32;
    const QUANT_MAX_U32: Self = u32::MAX;
    const QUANT_MAX_U64: Self = u32::MAX;
    const QUANT_MAX_USIZE: usize = u32::MAX as usize;

    const QUANT_CLAMPED_U8: u8 = u8::MAX;
    const QUANT_CLAMPED_U16: u16 = u16::MAX;
    const QUANT_CLAMPED_U32: u32 = u32::MAX;
    const QUANT_CLAMPED_U64: u64 = u32::MAX as u64;
    const QUANT_CLAMPED_USIZE: usize = u32::MAX as usize;

    const QUANT_BYTE_BIT_MASK: Self = 7;

    fn quant_from_usize(value: usize) -> Self {
        value as u32
    }

    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U32(value)
    }

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u32::MAX as usize {
            return Err(FeagiFailQuantizationOutOfRange::new("Given value cannot fit in a quantized u32!", value).into());
        }
        Ok(value as u32)
    }

    fn quant_to_usize(self) -> usize {
        self as usize
    }

    fn quant_to_u8(self) -> u8 {
        self as u8
    }

    fn quant_to_u16(self) -> u16 {
        self as u16
    }

    fn quant_to_u32(self) -> u32 {
        self
    }

    fn quant_to_u64(self) -> u64 {
        self as u64
    }

    fn from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_u32()
    }

    fn from_quantization_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        if value > FromQuant::QUANT_MAX_U32 {
            return u32::MAX;
        }
        value.quant_to_u32()
    }

    fn try_from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > FromQuant::QUANT_MAX_U32 {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized value exceeds u32 quantization!", value.quant_to_usize()).into());
        }
        Ok(value.quant_to_u32())
    }

    fn clamp_for_quantization<ClampFor: QuantizedUnsignedIntegerTrait>(self) -> Self {
        self.min(ClampFor::QUANT_CLAMPED_U32)
    }

    fn clamp_for_quantization_level_runtime(self, level: UnsignedIntegerQuantizationLevel) -> Self {
        match level {
            UnsignedIntegerQuantizationLevel::U8 => self.min(u8::MAX as u32),
            UnsignedIntegerQuantizationLevel::U16 => self.min(u16::MAX as u32),
            UnsignedIntegerQuantizationLevel::U32 => self,
            UnsignedIntegerQuantizationLevel::U64 => self,
            UnsignedIntegerQuantizationLevel::Usize => self,
        }
    }
}

impl QuantizedUnsignedIntegerTrait for u64 {
    const LEVEL: UnsignedIntegerQuantizationLevel = UnsignedIntegerQuantizationLevel::U64;
    const QUANT_MAX: Self = u64::MAX;
    const QUANT_MAX_U8: Self = u8::MAX as u64;
    const QUANT_MAX_U16: Self = u16::MAX as u64;
    const QUANT_MAX_U32: Self = u32::MAX as u64;
    const QUANT_MAX_U64: Self = u64::MAX;
    const QUANT_MAX_USIZE: usize = usize::MAX;

    const QUANT_CLAMPED_U8: u8 = u8::MAX;
    const QUANT_CLAMPED_U16: u16 = u16::MAX;
    const QUANT_CLAMPED_U32: u32 = u32::MAX;
    const QUANT_CLAMPED_U64: u64 = u64::MAX;
    const QUANT_CLAMPED_USIZE: usize = usize::MAX;

    const QUANT_BYTE_BIT_MASK: Self = 7;

    fn quant_from_usize(value: usize) -> Self {
        value as u64
    }

    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U64(value)
    }

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        // never fails
        Ok(value as u64)
    }

    fn quant_to_usize(self) -> usize {
        self as usize
    }

    fn quant_to_u8(self) -> u8 {
        self as u8
    }

    fn quant_to_u16(self) -> u16 {
        self as u16
    }

    fn quant_to_u32(self) -> u32 {
        self as u32
    }

    fn quant_to_u64(self) -> u64 {
        self
    }

    fn from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_u64()
    }

    fn from_quantization_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        if value > FromQuant::QUANT_MAX_U64 {
            return u64::MAX;
        }
        value.quant_to_u64()
    }

    fn try_from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > FromQuant::QUANT_MAX_U64 {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized value exceeds u64 quantization!", value.quant_to_usize()).into());
        }
        Ok(value.quant_to_u64())
    }

    fn clamp_for_quantization<ClampFor: QuantizedUnsignedIntegerTrait>(self) -> Self {
        self.min(ClampFor::QUANT_CLAMPED_U64)
    }

    fn clamp_for_quantization_level_runtime(self, level: UnsignedIntegerQuantizationLevel) -> Self {
        match level {
            UnsignedIntegerQuantizationLevel::U8 => self.min(u8::MAX as u64),
            UnsignedIntegerQuantizationLevel::U16 => self.min(u16::MAX as u64),
            UnsignedIntegerQuantizationLevel::U32 => self.min(u32::MAX as u64),
            UnsignedIntegerQuantizationLevel::U64 => self,
            UnsignedIntegerQuantizationLevel::Usize => self.min(usize::MAX as u64),
        }
    }
}

// Note: Specifically we will not support usize directly since it can vary in size depending on
// backend, which could cause some issues with device interoperability

/// Allows storing all quantized unsigned integer types under a single enum
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum UnsignedIntegerEnum {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

impl UnsignedIntegerEnum {
    pub fn new_from_quantized<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        FromQuant::quant_to_enum(value)
    }

    pub fn get_level(&self) -> UnsignedIntegerQuantizationLevel {
        match self {
            UnsignedIntegerEnum::U8(_) => UnsignedIntegerQuantizationLevel::U8,
            UnsignedIntegerEnum::U16(_) => UnsignedIntegerQuantizationLevel::U16,
            UnsignedIntegerEnum::U32(_) => UnsignedIntegerQuantizationLevel::U32,
            UnsignedIntegerEnum::U64(_) => UnsignedIntegerQuantizationLevel::U64,
        }
    }

    pub fn try_into_quant<Quant: QuantizedUnsignedIntegerTrait>(self) -> Result<Quant, FeagiDataValueQuantizationError> {
        // TODO assert Debug Check!
        match self {
            UnsignedIntegerEnum::U8(value) => value.try_to_quantization(),
            UnsignedIntegerEnum::U16(value) => value.try_to_quantization(),
            UnsignedIntegerEnum::U32(value) => value.try_to_quantization(),
            UnsignedIntegerEnum::U64(value) => value.try_to_quantization(),
        }
    }

    pub fn into_quant<Quant: QuantizedUnsignedIntegerTrait>(self) -> Quant {
        match self {
            UnsignedIntegerEnum::U8(value) => value.to_quantization(),
            UnsignedIntegerEnum::U16(value) => value.to_quantization(),
            UnsignedIntegerEnum::U32(value) => value.to_quantization(),
            UnsignedIntegerEnum::U64(value) => value.to_quantization(),
        }
    }

    pub fn to_usize(self) -> usize {
        match self {
            UnsignedIntegerEnum::U8(value) => value as usize,
            UnsignedIntegerEnum::U16(value) => value as usize,
            UnsignedIntegerEnum::U32(value) => value as usize,
            UnsignedIntegerEnum::U64(value) => value as usize,
        }
    }

    // TODO from usize that is CPU dependent to be either 32 bit or 64 bit
}
