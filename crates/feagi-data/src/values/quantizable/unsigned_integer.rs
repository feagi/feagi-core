use crate::values::quantizable::feagi_data_value_quantization_error::FeagiFailQuantizationOutOfRange;
use crate::values::quantizable::{FeagiDataValueQuantizationError, QuantizationLevelPacking, QuantizedElementBase};
use serde::{Deserialize, Serialize};

//region UInt value
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum UnsignedIntegerQuantizationLevel {
    U8 = 0,
    U16 = 1,
    U32 = 2,
    U64 = 3,
    // we are NOT doing u128 lol
    // Not doing usize since it differs per device
}

impl Into<u8> for UnsignedIntegerQuantizationLevel {
    fn into(self) -> u8 {
        self as u8
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
            _ => Err(()),
        }
    }
}

impl QuantizationLevelPacking for UnsignedIntegerQuantizationLevel {
    const NUMBER_BITS: usize = 2;

    unsafe fn from_unpacked_byte(byte: u8) -> Self {
        core::mem::transmute(byte)
    }
}

/// Trait designed to hold index and/or count values in a quantized form
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

    /// Mask for the bits
    const QUANT_BYTE_BIT_MASK: Self;

    /// Tries to convert from usize, does NOT check bounds!
    fn quant_from_usize_unchecked(value: usize) -> Self;

    /// Will wrap whatever quant this is to an `UnsignedIntegerEnum`
    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum;

    /// Tries converting from usize, returns an error if out of bounds
    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError>;

    /// Converts to usize. No need to check as we have no indexes that will exceed a usize on a
    /// system // TODO THIS IS NOT TRUE: a u64 on a 32 bit system cast to a usize truncates the bits! We should think about how to address this!
    fn quant_to_usize(self) -> usize;

    /// Tries to convert from u32, does NOT check bounds!
    fn quant_to_u8_unchecked(self) -> u8;

    /// Tries to convert to u16, does NOT check bounds!
    fn quant_to_u16_unchecked(self) -> u16;

    /// Tries to convert to u32, does NOT check bounds!
    fn quant_to_u32_unchecked(self) -> u32;

    /// Tries to convert to u64, bound checking shouldnt matter since this is the biggest type (not doing u128 lol)
    fn quant_to_u64_unchecked(self) -> u64;

    /// Creates from an index of another quantization. Does not check for validity of ranges!
    fn from_quantization_unchecked<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self;

    /// Creates from an index of another quantization, clamping its values to ensure it fits
    fn from_quantization_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self;

    /// Tries to create an index of another quantization, returns an error if it would break the bounds
    fn try_from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError>;

    /// Converts to an index of another quantization. Does not check for validity of ranges!
    fn to_quantization_unchecked<ToQuant: QuantizedUnsignedIntegerTrait>(self) -> ToQuant {
        ToQuant::from_quantization_unchecked::<Self>(self)
    }

    /// Converts to an index of another quantization, clamping its values to ensure it fits
    fn to_quantization_clamped<ToQuant: QuantizedUnsignedIntegerTrait>(self) -> ToQuant {
        ToQuant::from_quantization_clamped(self)
    }

    /// Tries to convert to an index of another quantization, returns an error if it would break the bounds
    fn try_to_quantization<ToQuant: QuantizedUnsignedIntegerTrait>(self) -> Result<ToQuant, FeagiDataValueQuantizationError> {
        ToQuant::try_from_quantization(self)
    }

    /// Clamps the value of this index for another quantization, but does not actually change the
    /// quantization itself
    fn clamp_for_quantization<ClampFor: QuantizedUnsignedIntegerTrait>(self) -> Self;

    fn clamp_for_quantization_level_runtime(self, level: UnsignedIntegerQuantizationLevel) -> Self;

    /// Returns true if the value is zero
    fn is_zero(self) -> bool {
        self == Self::QUANT_ZERO
    }
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

    fn quant_from_usize_unchecked(value: usize) -> Self {
        value as u8
    }

    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U8(value)
    }

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u8::MAX as usize {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u8!", value).into());
        }
        Ok(value as u8)
    }

    fn quant_to_usize(self) -> usize {
        self as usize
    }

    fn quant_to_u8_unchecked(self) -> u8 {
        self
    }

    fn quant_to_u16_unchecked(self) -> u16 {
        self as u16
    }

    fn quant_to_u32_unchecked(self) -> u32 {
        self as u32
    }

    fn quant_to_u64_unchecked(self) -> u64 {
        self as u64
    }

    fn from_quantization_unchecked<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_u8_unchecked()
    }

    fn from_quantization_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        if value > FromQuant::QUANT_MAX_U8 {
            return u8::MAX;
        }
        value.quant_to_u8_unchecked()
    }

    fn try_from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > FromQuant::QUANT_MAX_U8 {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized index exceeds u8 quantization!", value.quant_to_usize()).into());
        }
        Ok(value.quant_to_u8_unchecked())
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

    fn quant_from_usize_unchecked(value: usize) -> Self {
        value as u16
    }

    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U16(value)
    }

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u16::MAX as usize {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u16!", value).into());
        }
        Ok(value as u16)
    }

    fn quant_to_usize(self) -> usize {
        self as usize
    }

    fn quant_to_u8_unchecked(self) -> u8 {
        self as u8
    }

    fn quant_to_u16_unchecked(self) -> u16 {
        self
    }

    fn quant_to_u32_unchecked(self) -> u32 {
        self as u32
    }

    fn quant_to_u64_unchecked(self) -> u64 {
        self as u64
    }

    fn from_quantization_unchecked<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_u16_unchecked()
    }

    fn from_quantization_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        if value > FromQuant::QUANT_MAX_U16 {
            return u16::MAX;
        }
        value.quant_to_u16_unchecked()
    }

    fn try_from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > FromQuant::QUANT_MAX_U16 {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized index exceeds u16 quantization!", value.quant_to_usize()).into());
        }
        Ok(value.quant_to_u16_unchecked())
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

    fn quant_from_usize_unchecked(value: usize) -> Self {
        value as u32
    }

    fn quant_to_enum(value: Self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U32(value)
    }

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u32::MAX as usize {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u32!", value).into());
        }
        Ok(value as u32)
    }

    fn quant_to_usize(self) -> usize {
        self as usize
    }

    fn quant_to_u8_unchecked(self) -> u8 {
        self as u8
    }

    fn quant_to_u16_unchecked(self) -> u16 {
        self as u16
    }

    fn quant_to_u32_unchecked(self) -> u32 {
        self
    }

    fn quant_to_u64_unchecked(self) -> u64 {
        self as u64
    }

    fn from_quantization_unchecked<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_u32_unchecked()
    }

    fn from_quantization_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        if value > FromQuant::QUANT_MAX_U32 {
            return u32::MAX;
        }
        value.quant_to_u32_unchecked()
    }

    fn try_from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > FromQuant::QUANT_MAX_U32 {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized index exceeds u32 quantization!", value.quant_to_usize()).into());
        }
        Ok(value.quant_to_u32_unchecked())
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

    fn quant_from_usize_unchecked(value: usize) -> Self {
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

    fn quant_to_u8_unchecked(self) -> u8 {
        self as u8
    }

    fn quant_to_u16_unchecked(self) -> u16 {
        self as u16
    }

    fn quant_to_u32_unchecked(self) -> u32 {
        self as u32
    }

    fn quant_to_u64_unchecked(self) -> u64 {
        self
    }

    fn from_quantization_unchecked<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_u64_unchecked()
    }

    fn from_quantization_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        if value > FromQuant::QUANT_MAX_U64 {
            return u64::MAX;
        }
        value.quant_to_u64_unchecked()
    }

    fn try_from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > FromQuant::QUANT_MAX_U64 {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized index exceeds u64 quantization!", value.quant_to_usize()).into());
        }
        Ok(value.quant_to_u64_unchecked())
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
        }
    }
}

// Note: Specifically we will not support usize directly since it can vary in size depending on
// backend, which could cause some issues with device interoperability

/// Allows storing all quantized index types under a single enum
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
            UnsignedIntegerEnum::U8(value) => value.to_quantization_unchecked(),
            UnsignedIntegerEnum::U16(value) => value.to_quantization_unchecked(),
            UnsignedIntegerEnum::U32(value) => value.to_quantization_unchecked(),
            UnsignedIntegerEnum::U64(value) => value.to_quantization_unchecked(),
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

//endregion

//region Wrappers

/// A way to define a quantized uint but as a specific one to have compile time checks to avoid
/// mixups. Base trait that the others inherit
pub trait WrappedQuantizedUnsignedInteger:
    Copy
    + Clone
    + Send
    + Sync
    + Default
    + core::fmt::Debug
    + core::cmp::PartialEq
    + core::cmp::Eq
    + core::cmp::PartialOrd
    + core::cmp::Ord
    + core::hash::Hash
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::Rem<Output = Self>
    + core::ops::AddAssign
    + core::ops::SubAssign
    + core::ops::MulAssign
    + core::ops::DivAssign
    + core::ops::RemAssign
    + From<Self::Quant>
    + AsRef<Self::Quant>
    + AsMut<Self::Quant>
    + Sized
    + 'static
{
    /// The underlying quantized unsigned integer value this wrapper stores.
    type Quant: QuantizedUnsignedIntegerTrait;

    /// The quantization level of the underlying value.
    const LEVEL: UnsignedIntegerQuantizationLevel = <Self::Quant as QuantizedUnsignedIntegerTrait>::LEVEL;

    /// Zero, expressed in the wrapper's own type.
    const QUANT_ZERO: Self;
    /// One, expressed in the wrapper's own type.
    const QUANT_ONE: Self;
    /// The maximum representable value, expressed in the wrapper's own type.
    const QUANT_MAX: Self;

    const QUANT_MAX_U8: Self;
    const QUANT_MAX_U16: Self;
    const QUANT_MAX_U32: Self;
    const QUANT_MAX_U64: Self;
    const QUANT_MAX_USIZE: usize = <Self::Quant as QuantizedUnsignedIntegerTrait>::QUANT_MAX_USIZE;

    const QUANT_CLAMPED_U8: u8 = <Self::Quant as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_U8;
    const QUANT_CLAMPED_U16: u16 = <Self::Quant as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_U16;
    const QUANT_CLAMPED_U32: u32 = <Self::Quant as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_U32;
    const QUANT_CLAMPED_U64: u64 = <Self::Quant as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_U64;
    const QUANT_CLAMPED_USIZE: usize = <Self::Quant as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_USIZE;

    /// Wraps a raw quantized value into this wrapper type.
    fn new(value: Self::Quant) -> Self;

    /// Extracts the inner quantized index / count.
    fn deref(self) -> Self::Quant;

    /// Tries to convert from usize, does NOT check bounds!
    fn quant_from_usize_unchecked(value: usize) -> Self {
        Self::new(Self::Quant::quant_from_usize_unchecked(value))
    }

    /// Tries converting from usize, returns an error if out of bounds
    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        Ok(Self::new(Self::Quant::quant_try_from_usize(value)?))
    }

    /// Converts to usize. No need to check as we have no indexes that will exceed a usize on a
    /// system  // TODO THIS IS NOT TRUE: a u64 on a 32 bit system cast to a usize truncates the bits! We should think about how to address this!
    fn quant_to_usize(self) -> usize {
        self.deref().quant_to_usize()
    }

    /// Tries to convert to u8, does NOT check bounds!
    fn quant_to_u8_unchecked(self) -> u8 {
        self.deref().quant_to_u8_unchecked()
    }

    /// Tries to convert to u16, does NOT check bounds!
    fn quant_to_u16_unchecked(self) -> u16 {
        self.deref().quant_to_u16_unchecked()
    }

    /// Tries to convert to u32, does NOT check bounds!
    fn quant_to_u32_unchecked(self) -> u32 {
        self.deref().quant_to_u32_unchecked()
    }

    /// Tries to convert to u64, bound checking shouldnt matter since this is the biggest type (not doing u128 lol)
    fn quant_to_u64_unchecked(self) -> u64 {
        self.deref().quant_to_u64_unchecked()
    }

    /// Creates from an index of another quantization. Does not check for validity of ranges!
    fn from_quantization_unchecked<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        Self::new(Self::Quant::from_quantization_unchecked(value))
    }

    /// Creates from an index of another quantization, clamping its values to ensure it fits
    fn from_quantization_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Self {
        Self::new(Self::Quant::from_quantization_clamped(value))
    }

    /// Tries to create an index of another quantization, returns an error if it would break the bounds
    fn try_from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        Ok(Self::new(Self::Quant::try_from_quantization(value)?))
    }

    /// Converts to an index of another quantization. Does not check for validity of ranges!
    fn to_quantization_unchecked<ToQuant: QuantizedUnsignedIntegerTrait>(self) -> ToQuant {
        self.deref().to_quantization_unchecked()
    }

    /// Converts to an index of another quantization, clamping its values to ensure it fits
    fn to_quantization_clamped<ToQuant: QuantizedUnsignedIntegerTrait>(self) -> ToQuant {
        self.deref().to_quantization_clamped()
    }

    /// Tries to convert to an index of another quantization, returns an error if it would break the bounds
    fn try_to_quantization<ToQuant: QuantizedUnsignedIntegerTrait>(self) -> Result<ToQuant, FeagiDataValueQuantizationError> {
        self.deref().try_to_quantization()
    }

    /// Clamps the value of this index for another quantization, but does not actually change the
    /// quantization itself
    fn clamp_for_quantization<ClampFor: QuantizedUnsignedIntegerTrait>(self) -> Self {
        Self::new(self.deref().clamp_for_quantization::<ClampFor>())
    }

    /// Clamps the value of this index for a runtime-provided quantization level, but does not
    /// actually change the quantization itself
    fn clamp_for_quantization_level_runtime(self, level: UnsignedIntegerQuantizationLevel) -> Self {
        Self::new(self.deref().clamp_for_quantization_level_runtime(level))
    }

    /// Returns true if the value is zero
    fn is_zero(self) -> bool {
        self == Self::QUANT_ZERO
    }
}


/// Denotes the wrapped uint as data (IE not for indexing)
pub trait WrappedQuantizedUnsignedIntegerData: WrappedQuantizedUnsignedInteger {}

/// Denotes the wrapped uint as an index
pub trait WrappedQuantizedUnsignedIntegerIndex: WrappedQuantizedUnsignedInteger {
    // NOTE: To avoid circular dependency coupling, this should have nothing dependent on count
}

/// Denotes the wrapped uint as a count / size of something. Also needs a definition of an index.
pub trait WrappedQuantizedUnsignedIntegerCount: WrappedQuantizedUnsignedInteger {
    /// The index type paired with this count wrapper.
    type Index: WrappedQuantizedUnsignedIntegerIndex;

    /// Returns true if the index can fit in this count (less than)
    fn can_contain_index(&self, index: &Self::Index) -> bool {
        index.deref() < self.deref()
    }

    /// Returns the value of the maximum possible valid index. Returns None if the size is zero
    /// (no index possible).
    fn maximum_possible_index(&self) -> Option<Self::Index> {
        if self.deref().is_zero() {
            return None;
        }
        Some(Self::Index::new(self.deref() - Self::Quant::QUANT_ONE))
    }

    // TODO Iterators? From 0 - size, par iterator?

}


/// Shared behaviour implemented by every wrapped enum generated by the unsigned-integer wrapper
/// macros.
///
/// These enums hide the generic quantized wrapper type behind concrete variants
/// (`U8`, `U16`, `U32`, `U64`) while preserving the wrapper family semantics.
pub trait WrappedQuantizedUnsignedIntegerEnum:
Copy + Clone + Send + Sync + core::fmt::Debug + core::cmp::PartialEq + core::cmp::Eq + core::hash::Hash + Sized + 'static
{
    fn get_level(&self) -> UnsignedIntegerQuantizationLevel;

    fn try_into_quant<Quant: QuantizedUnsignedIntegerTrait>(self) -> Result<Quant, FeagiDataValueQuantizationError>;

    fn into_quant<Quant: QuantizedUnsignedIntegerTrait>(self) -> Quant;

    fn to_usize(self) -> usize;
}



//region Macros (for unsigned ints as data, and as indexes / counts)

/// Base for the other wrappers, as they just add tagging
#[doc(hidden)]
#[macro_export]
macro_rules! _create_wrapped_quantized_unsigned_integer {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(Q);

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> $struct_name<Q> {
            pub const LEVEL: $crate::values::quantizable::UnsignedIntegerQuantizationLevel = Q::LEVEL;
            pub const QUANT_ZERO: Self = Self::const_new(Q::QUANT_ZERO);
            pub const QUANT_ONE: Self = Self::const_new(Q::QUANT_ONE);
            pub const QUANT_MAX: Self = Self::const_new(Q::QUANT_MAX);

            pub const QUANT_MAX_U8: Self = Self::const_new(Q::QUANT_MAX_U8);
            pub const QUANT_MAX_U16: Self = Self::const_new(Q::QUANT_MAX_U16);
            pub const QUANT_MAX_U32: Self = Self::const_new(Q::QUANT_MAX_U32);
            pub const QUANT_MAX_U64: Self = Self::const_new(Q::QUANT_MAX_U64);
            pub const QUANT_MAX_USIZE: usize = Q::QUANT_MAX_USIZE;

            pub const QUANT_CLAMPED_U8: u8 = Q::QUANT_CLAMPED_U8;
            pub const QUANT_CLAMPED_U16: u16 = Q::QUANT_CLAMPED_U16;
            pub const QUANT_CLAMPED_U32: u32 = Q::QUANT_CLAMPED_U32;
            pub const QUANT_CLAMPED_U64: u64 = Q::QUANT_CLAMPED_U64;
            pub const QUANT_CLAMPED_USIZE: usize = Q::QUANT_CLAMPED_USIZE;

            pub const fn const_new(value: Q) -> Self
            {
                Self(value)
            }

            pub const fn const_deref(self) -> Q
            {
                self.0
            }

            pub fn new(v: Q) -> Self {
                Self(v)
            }

            /// Extracts the inner quantized index / count
            pub fn deref(self) -> Q {
                self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
            $crate::values::quantizable::WrappedQuantizedUnsignedInteger for $struct_name<Q>
        {
            type Quant = Q;

            const QUANT_ZERO: Self = Self::const_new(Q::QUANT_ZERO);
            const QUANT_ONE: Self = Self::const_new(Q::QUANT_ONE);
            const QUANT_MAX: Self = Self::const_new(Q::QUANT_MAX);
            const QUANT_MAX_U8: Self = Self::const_new(Q::QUANT_MAX_U8);
            const QUANT_MAX_U16: Self = Self::const_new(Q::QUANT_MAX_U16);
            const QUANT_MAX_U32: Self = Self::const_new(Q::QUANT_MAX_U32);
            const QUANT_MAX_U64: Self = Self::const_new(Q::QUANT_MAX_U64);

            fn new(value: Q) -> Self {
                Self(value)
            }

            fn deref(self) -> Q {
                self.0
            }
        }

        // NOTE: Into<Q> for $struct_name<Q> is not needed!

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> From<Q> for $struct_name<Q> {
            fn from(value: Q) -> Self {
                Self(value)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> From<&Q> for &$struct_name<Q> {
            fn from(value: &Q) -> Self {
                // tRust me bro
                unsafe { &*(value as *const Q as *const $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> From<&mut Q> for &mut $struct_name<Q> {
            fn from(value: &mut Q) -> Self {
                // tRust me bro
                unsafe { &mut *(value as *mut Q as *mut $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> AsRef<Q> for $struct_name<Q> {
            fn as_ref(&self) -> &Q {
                &self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> AsMut<Q> for $struct_name<Q> {
            fn as_mut(&mut self) -> &mut Q {
                &mut self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::Add for $struct_name<Q> {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::Sub for $struct_name<Q> {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::Mul for $struct_name<Q> {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::Div for $struct_name<Q> {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::Rem for $struct_name<Q> {
            type Output = Self;
            fn rem(self, rhs: Self) -> Self::Output {
                Self(self.0 % rhs.0)
            }
        }



        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::AddAssign for $struct_name<Q> {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::SubAssign for $struct_name<Q> {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::MulAssign for $struct_name<Q> {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::DivAssign for $struct_name<Q> {
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> core::ops::RemAssign for $struct_name<Q> {
            fn rem_assign(&mut self, rhs: Self) {
                self.0 %= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> Default for $struct_name<Q> {
            fn default() -> Self {
                Self(Q::default())
            }
        }

        ::paste::paste! {
            #[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
            $vis enum [<$struct_name Enum>] {
                U8($struct_name<u8>),
                U16($struct_name<u16>),
                U32($struct_name<u32>),
                U64($struct_name<u64>),
            }

            impl [<$struct_name Enum>] {
                pub fn new_from_quantized<FromQuant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    value: $struct_name<FromQuant>
                ) -> Self {
                    match FromQuant::LEVEL {
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8 => {
                            Self::U8($struct_name::<u8>::new(<u8 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::from_quantization_unchecked(value.deref())))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16 => {
                            Self::U16($struct_name::<u16>::new(<u16 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::from_quantization_unchecked(value.deref())))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32 => {
                            Self::U32($struct_name::<u32>::new(<u32 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::from_quantization_unchecked(value.deref())))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64 => {
                            Self::U64($struct_name::<u64>::new(<u64 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::from_quantization_unchecked(value.deref())))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::Usize => {
                            Self::U64($struct_name::<u64>::new(<u64 as $crate::values::quantizable::QuantizedUnsignedIntegerTrait>::from_quantization_unchecked(value.deref())))
                        }
                    }
                }

                pub fn from_unsigned_integer_enum(value: $crate::values::quantizable::UnsignedIntegerEnum) -> Self {
                    match value {
                        $crate::values::quantizable::UnsignedIntegerEnum::U8(v) => {
                            Self::U8($struct_name::<u8>::new(v))
                        }
                        $crate::values::quantizable::UnsignedIntegerEnum::U16(v) => {
                            Self::U16($struct_name::<u16>::new(v))
                        }
                        $crate::values::quantizable::UnsignedIntegerEnum::U32(v) => {
                            Self::U32($struct_name::<u32>::new(v))
                        }
                        $crate::values::quantizable::UnsignedIntegerEnum::U64(v) => {
                            Self::U64($struct_name::<u64>::new(v))
                        }
                    }
                }

                pub fn into_unsigned_integer_enum(self) -> $crate::values::quantizable::UnsignedIntegerEnum {
                    match self {
                        Self::U8(v) => $crate::values::quantizable::UnsignedIntegerEnum::U8(v.deref()),
                        Self::U16(v) => $crate::values::quantizable::UnsignedIntegerEnum::U16(v.deref()),
                        Self::U32(v) => $crate::values::quantizable::UnsignedIntegerEnum::U32(v.deref()),
                        Self::U64(v) => $crate::values::quantizable::UnsignedIntegerEnum::U64(v.deref()),
                    }
                }
                
                pub fn get_level(&self) -> $crate::values::quantizable::UnsignedIntegerQuantizationLevel {
                    match self {
                        Self::U8(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8,
                        Self::U16(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16,
                        Self::U32(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32,
                        Self::U64(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64,
                    }
                }

                pub fn try_into_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<Quant, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    match self {
                        Self::U8(value) => Quant::try_from_quantization(value.deref()),
                        Self::U16(value) => Quant::try_from_quantization(value.deref()),
                        Self::U32(value) => Quant::try_from_quantization(value.deref()),
                        Self::U64(value) => Quant::try_from_quantization(value.deref()),
                    }
                }

                pub fn into_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> Quant {
                    // TODO assert Debug Check!
                    match self {
                        Self::U8(value) => Quant::from_quantization_unchecked(value.deref()),
                        Self::U16(value) => Quant::from_quantization_unchecked(value.deref()),
                        Self::U32(value) => Quant::from_quantization_unchecked(value.deref()),
                        Self::U64(value) => Quant::from_quantization_unchecked(value.deref()),
                    }
                }

                pub fn to_usize(self) -> usize {
                    match self {
                        Self::U8(value) => value.deref() as usize,
                        Self::U16(value) => value.deref() as usize,
                        Self::U32(value) => value.deref() as usize,
                        Self::U64(value) => value.deref() as usize,
                    }
                }

                pub fn try_into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$struct_name<Quant>, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    Ok($struct_name::<Quant>::new(self.try_into_quant::<Quant>()?))
                }

                pub fn into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<Quant> {
                    $struct_name::<Quant>::new(self.into_quant::<Quant>())
                }
            }
        }

    };
}


/// Creates a wrapper for quantized unsigned integers as data (no indexing / count)
#[macro_export]
macro_rules! create_wrapped_quantized_unsigned_integer_data {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident
    ) => {

        $crate::_create_wrapped_quantized_unsigned_integer!(
            $(#[$meta])*
            $vis $struct_name
        );

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
        $crate::values::quantizable::WrappedQuantizedUnsignedIntegerData
        for  $struct_name<Q> {}
    };
}

/// Creates wrappers for an index
#[macro_export]
macro_rules! create_wrapped_quantized_unsigned_integer_index {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_index_name:ident
    ) => {
        $crate::_create_wrapped_quantized_unsigned_integer!(
            $(#[$meta])*
            $vis $struct_index_name
        );

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
        $crate::values::quantizable::WrappedQuantizedUnsignedIntegerIndex
        for  $struct_index_name<Q> {}
    };
}

/// Creates wrappers for a count, which also needs to take in what index it uses
#[macro_export]
macro_rules! create_wrapped_quantized_unsigned_integer_count {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_count_name:ident $struct_index_name:ident
    ) => {

        $crate::_create_wrapped_quantized_unsigned_integer!(
            $(#[$meta])*
            $vis $struct_count_name
        );
                
        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
        $crate::values::quantizable::WrappedQuantizedUnsignedIntegerCount
        for  $struct_count_name<Q> {
            type Index = $struct_index_name<Q>;
        }
    };
}


//endregion

//endregion