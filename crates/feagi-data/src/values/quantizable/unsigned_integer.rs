use crate::values::quantizable::feagi_data_value_quantization_error::FeagiFailQuantizationOutOfRange;
use crate::values::quantizable::{FeagiDataValueQuantizationError, QuantizationLevelPacking, QuantizedElementBase};
use serde::{Deserialize, Serialize};


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

/// Shared unsigned-integer quantization semantics for both raw and wrapped values.
///
/// Use this as a generic bound when a function should accept either an unwrapped primitive
/// (`u8`, `u16`, …) or a wrapped newtype implementing
/// [`QuantizedUnsignedIntegerWrappedTrait`].
pub trait QuantizedUnsignedIntegerTrait:
    QuantizedElementBase
    + core::cmp::Ord
    + core::cmp::Eq
    + core::hash::Hash
    + Serialize
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
    + core::ops::BitAnd<Output = Self>
    + core::ops::BitOr<Output = Self>
    + core::ops::BitXor<Output = Self>
    + core::ops::Shl<Output = Self>
    + core::ops::Shr<Output = Self>
    + core::ops::BitAndAssign
    + core::ops::BitOrAssign
    + core::ops::BitXorAssign
    + core::iter::Sum
    + core::iter::Product
{

    /// Defines the quantization type. Is simply Self for unwrapped, but for wrapped is the quant type.
    /// This allows universal compatibility checking between wrapped and unwrapped that the quantization level
    /// is the same
    type QuantType: QuantizedUnsignedIntegerTrait;
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

    /// Converts this value to an [`UnsignedIntegerEnum`].
    fn quant_to_enum(self) -> UnsignedIntegerEnum;

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
    fn try_from_quantization<FromQuant: QuantizedUnsignedIntegerTrait>(
        value: FromQuant,
    ) -> Result<Self, FeagiDataValueQuantizationError>;

    /// Converts to an index of another quantization. Does not check for validity of ranges!
    fn to_quantization_unchecked<ToQuant: QuantizedUnsignedIntegerTrait>(self) -> ToQuant {
        ToQuant::from_quantization_unchecked(self)
    }

    /// Converts to an index of another quantization, clamping its values to ensure it fits
    fn to_quantization_clamped<ToQuant: QuantizedUnsignedIntegerTrait>(self) -> ToQuant {
        ToQuant::from_quantization_clamped(self)
    }

    /// Tries to convert to an index of another quantization, returns an error if it would break the bounds
    fn try_to_quantization<ToQuant: QuantizedUnsignedIntegerTrait>(
        self,
    ) -> Result<ToQuant, FeagiDataValueQuantizationError> {
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

/// Marker trait for raw unsigned integer quantization types (`u8`, `u16`, `u32`, `u64`).
pub trait QuantizedUnsignedIntegerUnwrappedTrait: QuantizedUnsignedIntegerTrait {}

impl QuantizedUnsignedIntegerTrait for u8 {
    type QuantType = Self;
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

    fn quant_to_enum(self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U8(self)
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

impl QuantizedUnsignedIntegerUnwrappedTrait for u8 {}

impl QuantizedUnsignedIntegerTrait for u16 {
    type QuantType = Self;
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

    fn quant_to_enum(self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U16(self)
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

impl QuantizedUnsignedIntegerUnwrappedTrait for u16 {}

// lol, lmao even
impl QuantizedUnsignedIntegerTrait for u32 {
    type QuantType = Self;
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

    fn quant_to_enum(self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U32(self)
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

impl QuantizedUnsignedIntegerUnwrappedTrait for u32 {}

impl QuantizedUnsignedIntegerTrait for u64 {
    type QuantType = Self;
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

    fn quant_to_enum(self) -> UnsignedIntegerEnum {
        UnsignedIntegerEnum::U64(self)
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

impl QuantizedUnsignedIntegerUnwrappedTrait for u64 {}

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
        value.quant_to_enum()
    }

    pub fn get_level(&self) -> UnsignedIntegerQuantizationLevel {
        match self {
            UnsignedIntegerEnum::U8(_) => UnsignedIntegerQuantizationLevel::U8,
            UnsignedIntegerEnum::U16(_) => UnsignedIntegerQuantizationLevel::U16,
            UnsignedIntegerEnum::U32(_) => UnsignedIntegerQuantizationLevel::U32,
            UnsignedIntegerEnum::U64(_) => UnsignedIntegerQuantizationLevel::U64,
        }
    }

    pub fn try_into_quant<Quant: QuantizedUnsignedIntegerUnwrappedTrait>(self) -> Result<Quant, FeagiDataValueQuantizationError> {
        // TODO assert Debug Check!
        match self {
            UnsignedIntegerEnum::U8(value) => value.try_to_quantization(),
            UnsignedIntegerEnum::U16(value) => value.try_to_quantization(),
            UnsignedIntegerEnum::U32(value) => value.try_to_quantization(),
            UnsignedIntegerEnum::U64(value) => value.try_to_quantization(),
        }
    }

    pub fn into_quant<Quant: QuantizedUnsignedIntegerUnwrappedTrait>(self) -> Quant {
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


/// Shared behaviour implemented by every strongly-typed unsigned wrapper generated by
/// [`create_wrapped_quantized_unsigned_integer`].
///
/// Wrapper-specific behaviour is limited to [`Self::wrap`] and [`Self::deref`]; the macro must
/// also supply the `Self`-typed constants. Arithmetic, bit, and iterator semantics come from
/// [`QuantizedUnsignedIntegerTrait`].
pub trait QuantizedUnsignedIntegerWrappedTrait:
    QuantizedUnsignedIntegerTrait + From<Self::QuantType> + AsRef<Self::QuantType> + AsMut<Self::QuantType>
{

    /// The quantization level of the underlying value.
    const LEVEL: UnsignedIntegerQuantizationLevel = <Self::QuantType as QuantizedUnsignedIntegerTrait>::LEVEL;

    const QUANT_MAX: Self;
    const QUANT_MAX_U8: Self;
    const QUANT_MAX_U16: Self;
    const QUANT_MAX_U32: Self;
    const QUANT_MAX_U64: Self;
    const QUANT_MAX_USIZE: usize = <Self::QuantType as QuantizedUnsignedIntegerTrait>::QUANT_MAX_USIZE;

    const QUANT_CLAMPED_U8: u8 = <Self::QuantType as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_U8;
    const QUANT_CLAMPED_U16: u16 = <Self::QuantType as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_U16;
    const QUANT_CLAMPED_U32: u32 = <Self::QuantType as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_U32;
    const QUANT_CLAMPED_U64: u64 = <Self::QuantType as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_U64;
    const QUANT_CLAMPED_USIZE: usize = <Self::QuantType as QuantizedUnsignedIntegerTrait>::QUANT_CLAMPED_USIZE;

    const QUANT_BYTE_BIT_MASK: Self;

    /// Wraps a raw quantized value into this wrapper type.
    fn wrap(value: Self::QuantType) -> Self;

    /// Extracts the inner quantized index / count.
    fn deref(self) -> Self::QuantType;
}


/// Shared behaviour implemented by every wrapped enum generated by
/// [`create_wrapped_quantized_unsigned_integer`].
///
/// These enums hide the generic quantized wrapper type behind concrete variants
/// (`U8`, `U16`, `U32`, `U64`) while preserving the wrapper family semantics.
pub trait WrappedQuantizedUnsignedIntegerEnum:
Copy + Clone + Send + Sync + core::fmt::Debug + core::cmp::PartialEq + core::cmp::Eq + core::hash::Hash + Sized + 'static
{
    fn get_level(&self) -> UnsignedIntegerQuantizationLevel;

    fn try_into_quant<Quant: QuantizedUnsignedIntegerUnwrappedTrait>(self) -> Result<Quant, FeagiDataValueQuantizationError>;

    fn into_quant<Quant: QuantizedUnsignedIntegerUnwrappedTrait>(self) -> Quant;

    fn to_usize(self) -> usize;
}


/// Creates a wrapper for quantized unsigned integers
#[macro_export]
macro_rules! create_wrapped_quantized_unsigned_integer {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait>(Q);

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> $struct_name<Q> {
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
            pub const QUANT_BYTE_BIT_MASK: Self = Self::const_new(Q::QUANT_BYTE_BIT_MASK);

            pub const fn const_new(value: Q) -> Self {
                Self(value)
            }

            pub const fn const_deref(self) -> Q {
                self.0
            }

            pub fn new(v: Q) -> Self {
                Self(v)
            }

            /// Extracts the inner quantized index / count.
            pub fn deref(self) -> Q {
                self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait>
            $crate::values::quantizable::QuantizedElementBase for $struct_name<Q>
        {

            const QUANT_ZERO: Self = Self::const_new(Q::QUANT_ZERO);
            const QUANT_ONE: Self = Self::const_new(Q::QUANT_ONE);
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait>
            $crate::values::quantizable::QuantizedUnsignedIntegerTrait for $struct_name<Q>
        {
            type QuantType = Q;
            const LEVEL: $crate::values::quantizable::UnsignedIntegerQuantizationLevel = Q::LEVEL;
            const QUANT_MAX: Self = Self::const_new(Q::QUANT_MAX);
            const QUANT_MAX_U8: Self = Self::const_new(Q::QUANT_MAX_U8);
            const QUANT_MAX_U16: Self = Self::const_new(Q::QUANT_MAX_U16);
            const QUANT_MAX_U32: Self = Self::const_new(Q::QUANT_MAX_U32);
            const QUANT_MAX_U64: Self = Self::const_new(Q::QUANT_MAX_U64);
            const QUANT_MAX_USIZE: usize = Q::QUANT_MAX_USIZE;

            const QUANT_CLAMPED_U8: u8 = Q::QUANT_CLAMPED_U8;
            const QUANT_CLAMPED_U16: u16 = Q::QUANT_CLAMPED_U16;
            const QUANT_CLAMPED_U32: u32 = Q::QUANT_CLAMPED_U32;
            const QUANT_CLAMPED_U64: u64 = Q::QUANT_CLAMPED_U64;
            const QUANT_CLAMPED_USIZE: usize = Q::QUANT_CLAMPED_USIZE;

            const QUANT_BYTE_BIT_MASK: Self = Self::const_new(Q::QUANT_BYTE_BIT_MASK);

            fn quant_from_usize_unchecked(value: usize) -> Self {
                Self::const_new(Q::quant_from_usize_unchecked(value))
            }

            fn quant_to_enum(self) -> $crate::values::quantizable::UnsignedIntegerEnum {
                self.0.quant_to_enum()
            }

            fn quant_try_from_usize(value: usize) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                Ok(Self::const_new(Q::quant_try_from_usize(value)?))
            }

            fn quant_to_usize(self) -> usize {
                self.0.quant_to_usize()
            }

            fn quant_to_u8_unchecked(self) -> u8 {
                self.0.quant_to_u8_unchecked()
            }

            fn quant_to_u16_unchecked(self) -> u16 {
                self.0.quant_to_u16_unchecked()
            }

            fn quant_to_u32_unchecked(self) -> u32 {
                self.0.quant_to_u32_unchecked()
            }

            fn quant_to_u64_unchecked(self) -> u64 {
                self.0.quant_to_u64_unchecked()
            }

            fn from_quantization_unchecked<FromQuant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                value: FromQuant,
            ) -> Self {
                Self::const_new(Q::from_quantization_unchecked(value))
            }

            fn from_quantization_clamped<FromQuant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                value: FromQuant,
            ) -> Self {
                Self::const_new(Q::from_quantization_clamped(value))
            }

            fn try_from_quantization<FromQuant: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                value: FromQuant,
            ) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                Ok(Self::const_new(Q::try_from_quantization(value)?))
            }

            fn clamp_for_quantization<ClampFor: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                self,
            ) -> Self {
                Self::const_new(self.0.clamp_for_quantization::<ClampFor>())
            }

            fn clamp_for_quantization_level_runtime(
                self,
                level: $crate::values::quantizable::UnsignedIntegerQuantizationLevel,
            ) -> Self {
                Self::const_new(self.0.clamp_for_quantization_level_runtime(level))
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait>
            $crate::values::quantizable::QuantizedUnsignedIntegerWrappedTrait for $struct_name<Q>
        {
            const QUANT_MAX: Self = Self::const_new(Q::QUANT_MAX);
            const QUANT_MAX_U8: Self = Self::const_new(Q::QUANT_MAX_U8);
            const QUANT_MAX_U16: Self = Self::const_new(Q::QUANT_MAX_U16);
            const QUANT_MAX_U32: Self = Self::const_new(Q::QUANT_MAX_U32);
            const QUANT_MAX_U64: Self = Self::const_new(Q::QUANT_MAX_U64);
            const QUANT_BYTE_BIT_MASK: Self = Self::const_new(Q::QUANT_BYTE_BIT_MASK);

            fn wrap(value: Q) -> Self {
                Self(value)
            }

            fn deref(self) -> Q {
                self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> From<Q> for $struct_name<Q> {
            fn from(value: Q) -> Self {
                Self(value)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> From<&Q> for &$struct_name<Q> {
            fn from(value: &Q) -> Self {
                unsafe { &*(value as *const Q as *const $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> From<&mut Q> for &mut $struct_name<Q> {
            fn from(value: &mut Q) -> Self {
                unsafe { &mut *(value as *mut Q as *mut $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> AsRef<Q> for $struct_name<Q> {
            fn as_ref(&self) -> &Q {
                &self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> AsMut<Q> for $struct_name<Q> {
            fn as_mut(&mut self) -> &mut Q {
                &mut self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::fmt::Display for $struct_name<Q> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Display::fmt(&self.0, f)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> ::serde::Serialize for $struct_name<Q> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                ::serde::Serialize::serialize(&self.0, serializer)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::iter::Sum for $struct_name<Q> {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                Self(
                    iter.map($crate::values::quantizable::QuantizedUnsignedIntegerWrappedTrait::deref)
                        .sum(),
                )
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::iter::Product for $struct_name<Q> {
            fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
                Self(
                    iter.map($crate::values::quantizable::QuantizedUnsignedIntegerWrappedTrait::deref)
                        .product(),
                )
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::Add for $struct_name<Q> {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::Sub for $struct_name<Q> {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::Mul for $struct_name<Q> {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::Div for $struct_name<Q> {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::Rem for $struct_name<Q> {
            type Output = Self;
            fn rem(self, rhs: Self) -> Self::Output {
                Self(self.0 % rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::AddAssign for $struct_name<Q> {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::SubAssign for $struct_name<Q> {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::MulAssign for $struct_name<Q> {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::DivAssign for $struct_name<Q> {
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::RemAssign for $struct_name<Q> {
            fn rem_assign(&mut self, rhs: Self) {
                self.0 %= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::BitAnd for $struct_name<Q> {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::BitOr for $struct_name<Q> {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::BitXor for $struct_name<Q> {
            type Output = Self;
            fn bitxor(self, rhs: Self) -> Self::Output {
                Self(self.0 ^ rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::Shl for $struct_name<Q> {
            type Output = Self;
            fn shl(self, rhs: Self) -> Self::Output {
                Self(self.0 << rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::Shr for $struct_name<Q> {
            type Output = Self;
            fn shr(self, rhs: Self) -> Self::Output {
                Self(self.0 >> rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::BitAndAssign for $struct_name<Q> {
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::BitOrAssign for $struct_name<Q> {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> core::ops::BitXorAssign for $struct_name<Q> {
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait> Default for $struct_name<Q> {
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
                pub fn new_from_quantized<FromQuant: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait>(
                    value: $struct_name<FromQuant>
                ) -> Self {
                    Self::from_unsigned_integer_enum(
                        $crate::values::quantizable::UnsignedIntegerEnum::new_from_quantized(value.deref())
                    )
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

                pub fn try_into_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait>(
                    self
                ) -> Result<Quant, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    match self {
                        Self::U8(value) => Quant::try_from_quantization(value.deref()),
                        Self::U16(value) => Quant::try_from_quantization(value.deref()),
                        Self::U32(value) => Quant::try_from_quantization(value.deref()),
                        Self::U64(value) => Quant::try_from_quantization(value.deref()),
                    }
                }

                pub fn into_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait>(self) -> Quant {
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

                pub fn try_into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait>(
                    self
                ) -> Result<$struct_name<Quant>, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    Ok($struct_name::<Quant>::new(self.try_into_quant::<Quant>()?))
                }

                pub fn into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait>(
                    self
                ) -> $struct_name<Quant> {
                    $struct_name::<Quant>::new(self.into_quant::<Quant>())
                }
            }

            impl $crate::values::quantizable::WrappedQuantizedUnsignedIntegerEnum for [<$struct_name Enum>] {
                fn get_level(&self) -> $crate::values::quantizable::UnsignedIntegerQuantizationLevel {
                    [<$struct_name Enum>]::get_level(self)
                }

                fn try_into_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait>(
                    self
                ) -> Result<Quant, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    [<$struct_name Enum>]::try_into_quant(self)
                }

                fn into_quant<Quant: $crate::values::quantizable::QuantizedUnsignedIntegerUnwrappedTrait>(self) -> Quant {
                    [<$struct_name Enum>]::into_quant(self)
                }

                fn to_usize(self) -> usize {
                    [<$struct_name Enum>]::to_usize(self)
                }
            }
        }
    };
}

