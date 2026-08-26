use crate::values::quantizable::feagi_data_value_quantization_error::FeagiFailQuantizationOutOfRange;
use crate::values::quantizable::{FeagiDataValueQuantizationError, QuantizationLevelPacking, QuantizedElementBase};
use serde::Serialize;


#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum SignedIntegerQuantizationLevel {
    I8 = 0,
    I16 = 1,
    I32 = 2,
    I64 = 3,
    // We can support a max of 8
}

impl Into<u8> for SignedIntegerQuantizationLevel {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for SignedIntegerQuantizationLevel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SignedIntegerQuantizationLevel::I8),
            1 => Ok(SignedIntegerQuantizationLevel::I16),
            2 => Ok(SignedIntegerQuantizationLevel::I32),
            3 => Ok(SignedIntegerQuantizationLevel::I64),
            _ => Err(()),
        }
    }
}

impl QuantizationLevelPacking for SignedIntegerQuantizationLevel {
    const NUMBER_BITS: usize = 2;

    unsafe fn from_unpacked_byte(byte: u8) -> Self {
        core::mem::transmute(byte)
    }
}

/// Shared signed-integer quantization semantics for both raw and wrapped values.
///
/// Use this as a generic bound when a function should accept either an unwrapped primitive
/// (`i8`, `i16`, …) or a wrapped newtype implementing
/// [`QuantizedSignedIntegerWrappedTrait`].
pub trait QuantizedSignedIntegerTrait:
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
    + core::iter::Sum
    + core::iter::Product
{
    /// Defines the quantization type. Is simply Self for unwrapped, but for wrapped is the quant type.
    /// This allows universal compatibility checking between wrapped and unwrapped that the quantization level
    /// is the same
    type QuantType: QuantizedSignedIntegerTrait;
    const LEVEL: SignedIntegerQuantizationLevel;

    const QUANT_MAX: Self;

    const QUANT_MAX_I8: Self;
    const QUANT_MAX_I16: Self;
    const QUANT_MAX_I32: Self;
    const QUANT_MAX_I64: Self;
    const QUANT_MAX_ISIZE: usize;

    const QUANT_CLAMPED_I8: i8;
    const QUANT_CLAMPED_I16: i16;
    const QUANT_CLAMPED_I32: i32;
    const QUANT_CLAMPED_I64: i64;
    const QUANT_CLAMPED_ISIZE: isize;

    /// Tries to convert from isize, does NOT check bounds!
    fn quant_from_isize_unchecked(value: isize) -> Self;

    /// Converts this value to a [`SignedIntegerEnum`].
    fn quant_to_enum(self) -> SignedIntegerEnum;

    /// Tries converting from isize, returns an error if out of bounds
    fn quant_try_from_isize(value: isize) -> Result<Self, FeagiDataValueQuantizationError>;

    /// Converts to isize.
    fn quant_to_isize(self) -> isize;

    /// Tries to convert to i8, does NOT check bounds!
    fn quant_to_i8_unchecked(self) -> i8;

    /// Tries to convert to i16, does NOT check bounds!
    fn quant_to_i16_unchecked(self) -> i16;

    /// Tries to convert to i32, does NOT check bounds!
    fn quant_to_i32_unchecked(self) -> i32;

    /// Tries to convert to i64, does NOT check bounds!
    fn quant_to_i64_unchecked(self) -> i64;

    /// Creates from a value of another quantization. Does not check for validity of ranges!
    fn from_quantization_unchecked<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self;

    /// Creates from a value of another quantization, clamping its values to ensure it fits
    fn from_quantization_clamped<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self;

    /// Tries to create a value of another quantization, returns an error if it would break the bounds
    fn try_from_quantization<FromQuant: QuantizedSignedIntegerTrait>(
        value: FromQuant,
    ) -> Result<Self, FeagiDataValueQuantizationError>;

    /// Converts to another quantization. Does not check for validity of ranges!
    fn to_quantization_unchecked<ToQuant: QuantizedSignedIntegerTrait>(self) -> ToQuant {
        ToQuant::from_quantization_unchecked(self)
    }

    /// Converts to another quantization, clamping its values to ensure it fits
    fn to_quantization_clamped<ToQuant: QuantizedSignedIntegerTrait>(self) -> ToQuant {
        ToQuant::from_quantization_clamped(self)
    }

    /// Tries to convert to another quantization, returns an error if it would break the bounds
    fn try_to_quantization<ToQuant: QuantizedSignedIntegerTrait>(
        self,
    ) -> Result<ToQuant, FeagiDataValueQuantizationError> {
        ToQuant::try_from_quantization(self)
    }

    /// Clamps the value for another quantization, but does not actually change the quantization itself
    fn clamp_for_quantization<ClampFor: QuantizedSignedIntegerTrait>(self) -> Self;

    fn clamp_for_quantization_level_runtime(self, level: SignedIntegerQuantizationLevel) -> Self;

    /// Returns true if the value is zero
    fn is_zero(self) -> bool {
        self == Self::QUANT_ZERO
    }

    fn is_negative(&self) -> bool;
    fn is_zero_or_negative(&self) -> bool;
}

/// Marker trait for raw signed integer quantization types (`i8`, `i16`, `i32`, `i64`).
pub trait QuantizedSignedIntegerUnwrappedTrait: QuantizedSignedIntegerTrait {}

impl QuantizedSignedIntegerTrait for i8 {
    type QuantType = Self;
    const LEVEL: SignedIntegerQuantizationLevel = SignedIntegerQuantizationLevel::I8;
    const QUANT_MAX: Self = i8::MAX;
    const QUANT_MAX_I8: Self = i8::MAX;
    const QUANT_MAX_I16: Self = i8::MAX;
    const QUANT_MAX_I32: Self = i8::MAX;
    const QUANT_MAX_I64: Self = i8::MAX;
    const QUANT_MAX_ISIZE: usize = i8::MAX as usize;

    const QUANT_CLAMPED_I8: i8 = i8::MAX;
    const QUANT_CLAMPED_I16: i16 = i8::MAX as i16;
    const QUANT_CLAMPED_I32: i32 = i8::MAX as i32;
    const QUANT_CLAMPED_I64: i64 = i8::MAX as i64;
    const QUANT_CLAMPED_ISIZE: isize = i8::MAX as isize;

    fn quant_from_isize_unchecked(value: isize) -> Self {
        value as i8
    }

    fn quant_to_enum(self) -> SignedIntegerEnum {
        SignedIntegerEnum::I8(self)
    }

    fn quant_try_from_isize(value: isize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value < i8::MIN as isize || value > i8::MAX as isize {
            return Err(
                FeagiFailQuantizationOutOfRange::new("Given signed integer value cannot fit in a quantized i8!", value as usize).into(),
            );
        }
        Ok(value as i8)
    }

    fn quant_to_isize(self) -> isize {
        self as isize
    }

    fn quant_to_i8_unchecked(self) -> i8 {
        self
    }

    fn quant_to_i16_unchecked(self) -> i16 {
        self as i16
    }

    fn quant_to_i32_unchecked(self) -> i32 {
        self as i32
    }

    fn quant_to_i64_unchecked(self) -> i64 {
        self as i64
    }

    fn from_quantization_unchecked<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_i8_unchecked()
    }

    fn from_quantization_clamped<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self {
        Self::quant_from_isize_unchecked(clamp_isize_for_signed_quant::<Self>(value.quant_to_isize()))
    }

    fn try_from_quantization<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        let as_isize = value.quant_to_isize();
        if !signed_value_fits_quant::<Self>(as_isize) {
            return Err(
                FeagiFailQuantizationOutOfRange::new("Quantized signed integer exceeds i8 quantization!", as_isize as usize).into(),
            );
        }
        Ok(value.quant_to_i8_unchecked())
    }

    fn clamp_for_quantization<ClampFor: QuantizedSignedIntegerTrait>(self) -> Self {
        self.clamp_for_quantization_level_runtime(ClampFor::LEVEL)
    }

    fn clamp_for_quantization_level_runtime(self, level: SignedIntegerQuantizationLevel) -> Self {
        Self::quant_from_isize_unchecked(clamp_isize_for_signed_quant_level(self.quant_to_isize(), level))
    }

    #[inline(always)]
    fn is_negative(&self) -> bool {
        *self < 0
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self <= 0
    }
}

impl QuantizedSignedIntegerUnwrappedTrait for i8 {}

impl QuantizedSignedIntegerTrait for i16 {
    type QuantType = Self;
    const LEVEL: SignedIntegerQuantizationLevel = SignedIntegerQuantizationLevel::I16;
    const QUANT_MAX: Self = i16::MAX;
    const QUANT_MAX_I8: Self = i8::MAX as i16;
    const QUANT_MAX_I16: Self = i16::MAX;
    const QUANT_MAX_I32: Self = i16::MAX;
    const QUANT_MAX_I64: Self = i16::MAX;
    const QUANT_MAX_ISIZE: usize = i16::MAX as usize;

    const QUANT_CLAMPED_I8: i8 = i8::MAX;
    const QUANT_CLAMPED_I16: i16 = i16::MAX;
    const QUANT_CLAMPED_I32: i32 = i16::MAX as i32;
    const QUANT_CLAMPED_I64: i64 = i16::MAX as i64;
    const QUANT_CLAMPED_ISIZE: isize = i16::MAX as isize;

    fn quant_from_isize_unchecked(value: isize) -> Self {
        value as i16
    }

    fn quant_to_enum(self) -> SignedIntegerEnum {
        SignedIntegerEnum::I16(self)
    }

    fn quant_try_from_isize(value: isize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value < i16::MIN as isize || value > i16::MAX as isize {
            return Err(
                FeagiFailQuantizationOutOfRange::new("Given signed integer value cannot fit in a quantized i16!", value as usize).into(),
            );
        }
        Ok(value as i16)
    }

    fn quant_to_isize(self) -> isize {
        self as isize
    }

    fn quant_to_i8_unchecked(self) -> i8 {
        self as i8
    }

    fn quant_to_i16_unchecked(self) -> i16 {
        self
    }

    fn quant_to_i32_unchecked(self) -> i32 {
        self as i32
    }

    fn quant_to_i64_unchecked(self) -> i64 {
        self as i64
    }

    fn from_quantization_unchecked<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_i16_unchecked()
    }

    fn from_quantization_clamped<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self {
        Self::quant_from_isize_unchecked(clamp_isize_for_signed_quant::<Self>(value.quant_to_isize()))
    }

    fn try_from_quantization<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        let as_isize = value.quant_to_isize();
        if !signed_value_fits_quant::<Self>(as_isize) {
            return Err(
                FeagiFailQuantizationOutOfRange::new("Quantized signed integer exceeds i16 quantization!", as_isize as usize).into(),
            );
        }
        Ok(value.quant_to_i16_unchecked())
    }

    fn clamp_for_quantization<ClampFor: QuantizedSignedIntegerTrait>(self) -> Self {
        self.clamp_for_quantization_level_runtime(ClampFor::LEVEL)
    }

    fn clamp_for_quantization_level_runtime(self, level: SignedIntegerQuantizationLevel) -> Self {
        Self::quant_from_isize_unchecked(clamp_isize_for_signed_quant_level(self.quant_to_isize(), level))
    }

    #[inline(always)]
    fn is_negative(&self) -> bool {
        *self < 0
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self <= 0
    }
}

impl QuantizedSignedIntegerUnwrappedTrait for i16 {}

// lol, lmao even
impl QuantizedSignedIntegerTrait for i32 {
    type QuantType = Self;
    const LEVEL: SignedIntegerQuantizationLevel = SignedIntegerQuantizationLevel::I32;
    const QUANT_MAX: Self = i32::MAX;
    const QUANT_MAX_I8: Self = i8::MAX as i32;
    const QUANT_MAX_I16: Self = i16::MAX as i32;
    const QUANT_MAX_I32: Self = i32::MAX;
    const QUANT_MAX_I64: Self = i32::MAX;
    const QUANT_MAX_ISIZE: usize = i32::MAX as usize;

    const QUANT_CLAMPED_I8: i8 = i8::MAX;
    const QUANT_CLAMPED_I16: i16 = i16::MAX;
    const QUANT_CLAMPED_I32: i32 = i32::MAX;
    const QUANT_CLAMPED_I64: i64 = i32::MAX as i64;
    const QUANT_CLAMPED_ISIZE: isize = i32::MAX as isize;

    fn quant_from_isize_unchecked(value: isize) -> Self {
        value as i32
    }

    fn quant_to_enum(self) -> SignedIntegerEnum {
        SignedIntegerEnum::I32(self)
    }

    fn quant_try_from_isize(value: isize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value < i32::MIN as isize || value > i32::MAX as isize {
            return Err(
                FeagiFailQuantizationOutOfRange::new("Given signed integer value cannot fit in a quantized i32!", value as usize).into(),
            );
        }
        Ok(value as i32)
    }

    fn quant_to_isize(self) -> isize {
        self as isize
    }

    fn quant_to_i8_unchecked(self) -> i8 {
        self as i8
    }

    fn quant_to_i16_unchecked(self) -> i16 {
        self as i16
    }

    fn quant_to_i32_unchecked(self) -> i32 {
        self
    }

    fn quant_to_i64_unchecked(self) -> i64 {
        self as i64
    }

    fn from_quantization_unchecked<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_i32_unchecked()
    }

    fn from_quantization_clamped<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self {
        Self::quant_from_isize_unchecked(clamp_isize_for_signed_quant::<Self>(value.quant_to_isize()))
    }

    fn try_from_quantization<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        let as_isize = value.quant_to_isize();
        if !signed_value_fits_quant::<Self>(as_isize) {
            return Err(
                FeagiFailQuantizationOutOfRange::new("Quantized signed integer exceeds i32 quantization!", as_isize as usize).into(),
            );
        }
        Ok(value.quant_to_i32_unchecked())
    }

    fn clamp_for_quantization<ClampFor: QuantizedSignedIntegerTrait>(self) -> Self {
        self.clamp_for_quantization_level_runtime(ClampFor::LEVEL)
    }

    fn clamp_for_quantization_level_runtime(self, level: SignedIntegerQuantizationLevel) -> Self {
        Self::quant_from_isize_unchecked(clamp_isize_for_signed_quant_level(self.quant_to_isize(), level))
    }

    #[inline(always)]
    fn is_negative(&self) -> bool {
        *self < 0
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self <= 0
    }
}

impl QuantizedSignedIntegerUnwrappedTrait for i32 {}

impl QuantizedSignedIntegerTrait for i64 {
    type QuantType = Self;
    const LEVEL: SignedIntegerQuantizationLevel = SignedIntegerQuantizationLevel::I64;
    const QUANT_MAX: Self = i64::MAX;
    const QUANT_MAX_I8: Self = i8::MAX as i64;
    const QUANT_MAX_I16: Self = i16::MAX as i64;
    const QUANT_MAX_I32: Self = i32::MAX as i64;
    const QUANT_MAX_I64: Self = i64::MAX;
    const QUANT_MAX_ISIZE: usize = isize::MAX as usize;

    const QUANT_CLAMPED_I8: i8 = i8::MAX;
    const QUANT_CLAMPED_I16: i16 = i16::MAX;
    const QUANT_CLAMPED_I32: i32 = i32::MAX;
    const QUANT_CLAMPED_I64: i64 = i64::MAX;
    const QUANT_CLAMPED_ISIZE: isize = isize::MAX;

    fn quant_from_isize_unchecked(value: isize) -> Self {
        value as i64
    }

    fn quant_to_enum(self) -> SignedIntegerEnum {
        SignedIntegerEnum::I64(self)
    }

    fn quant_try_from_isize(value: isize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value < i64::MIN as isize || value > i64::MAX as isize {
            return Err(
                FeagiFailQuantizationOutOfRange::new("Given signed integer value cannot fit in a quantized i64!", value as usize).into(),
            );
        }
        Ok(value as i64)
    }

    fn quant_to_isize(self) -> isize {
        self as isize
    }

    fn quant_to_i8_unchecked(self) -> i8 {
        self as i8
    }

    fn quant_to_i16_unchecked(self) -> i16 {
        self as i16
    }

    fn quant_to_i32_unchecked(self) -> i32 {
        self as i32
    }

    fn quant_to_i64_unchecked(self) -> i64 {
        self
    }

    fn from_quantization_unchecked<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_i64_unchecked()
    }

    fn from_quantization_clamped<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self {
        Self::quant_from_isize_unchecked(clamp_isize_for_signed_quant::<Self>(value.quant_to_isize()))
    }

    fn try_from_quantization<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        let as_isize = value.quant_to_isize();
        if !signed_value_fits_quant::<Self>(as_isize) {
            return Err(
                FeagiFailQuantizationOutOfRange::new("Quantized signed integer exceeds i64 quantization!", as_isize as usize).into(),
            );
        }
        Ok(value.quant_to_i64_unchecked())
    }

    fn clamp_for_quantization<ClampFor: QuantizedSignedIntegerTrait>(self) -> Self {
        self.clamp_for_quantization_level_runtime(ClampFor::LEVEL)
    }

    fn clamp_for_quantization_level_runtime(self, level: SignedIntegerQuantizationLevel) -> Self {
        Self::quant_from_isize_unchecked(clamp_isize_for_signed_quant_level(self.quant_to_isize(), level))
    }

    #[inline(always)]
    fn is_negative(&self) -> bool {
        *self < 0
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self <= 0
    }
}

impl QuantizedSignedIntegerUnwrappedTrait for i64 {}

/// Allows storing all quantized signed integer types under a single enum
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum SignedIntegerEnum {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
}

impl SignedIntegerEnum {
    pub fn new_from_quantized<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_enum()
    }

    pub fn get_level(&self) -> SignedIntegerQuantizationLevel {
        match self {
            SignedIntegerEnum::I8(_) => SignedIntegerQuantizationLevel::I8,
            SignedIntegerEnum::I16(_) => SignedIntegerQuantizationLevel::I16,
            SignedIntegerEnum::I32(_) => SignedIntegerQuantizationLevel::I32,
            SignedIntegerEnum::I64(_) => SignedIntegerQuantizationLevel::I64,
        }
    }

    pub fn try_into_quant<Quant: QuantizedSignedIntegerUnwrappedTrait>(self) -> Result<Quant, FeagiDataValueQuantizationError> {
        match self {
            SignedIntegerEnum::I8(value) => value.try_to_quantization(),
            SignedIntegerEnum::I16(value) => value.try_to_quantization(),
            SignedIntegerEnum::I32(value) => value.try_to_quantization(),
            SignedIntegerEnum::I64(value) => value.try_to_quantization(),
        }
    }

    pub fn into_quant<Quant: QuantizedSignedIntegerUnwrappedTrait>(self) -> Quant {
        match self {
            SignedIntegerEnum::I8(value) => value.to_quantization_unchecked(),
            SignedIntegerEnum::I16(value) => value.to_quantization_unchecked(),
            SignedIntegerEnum::I32(value) => value.to_quantization_unchecked(),
            SignedIntegerEnum::I64(value) => value.to_quantization_unchecked(),
        }
    }

    pub fn to_isize(self) -> isize {
        match self {
            SignedIntegerEnum::I8(value) => value as isize,
            SignedIntegerEnum::I16(value) => value as isize,
            SignedIntegerEnum::I32(value) => value as isize,
            SignedIntegerEnum::I64(value) => value as isize,
        }
    }

    // TODO from isize that is CPU dependent to be either 32 bit or 64 bit
}

fn clamp_isize_for_signed_quant_level(value: isize, level: SignedIntegerQuantizationLevel) -> isize {
    match level {
        SignedIntegerQuantizationLevel::I8 => value.clamp(i8::MIN as isize, i8::MAX as isize),
        SignedIntegerQuantizationLevel::I16 => value.clamp(i16::MIN as isize, i16::MAX as isize),
        SignedIntegerQuantizationLevel::I32 => value.clamp(i32::MIN as isize, i32::MAX as isize),
        SignedIntegerQuantizationLevel::I64 => value.clamp(i64::MIN as isize, i64::MAX as isize),
    }
}

fn clamp_isize_for_signed_quant<Quant: QuantizedSignedIntegerTrait>(value: isize) -> isize {
    clamp_isize_for_signed_quant_level(value, Quant::LEVEL)
}

fn signed_value_fits_quant<Quant: QuantizedSignedIntegerTrait>(value: isize) -> bool {
    match Quant::LEVEL {
        SignedIntegerQuantizationLevel::I8 => (i8::MIN as isize) <= value && value <= (i8::MAX as isize),
        SignedIntegerQuantizationLevel::I16 => (i16::MIN as isize) <= value && value <= (i16::MAX as isize),
        SignedIntegerQuantizationLevel::I32 => (i32::MIN as isize) <= value && value <= (i32::MAX as isize),
        SignedIntegerQuantizationLevel::I64 => (i64::MIN as isize) <= value && value <= (i64::MAX as isize),
    }
}


/// Shared behaviour implemented by every strongly-typed signed wrapper generated by
/// [`create_wrapped_quantized_signed_integer`].
///
/// Wrapper-specific behaviour is limited to [`Self::wrap`] and [`Self::deref`]; the macro must
/// also supply the `Self`-typed constants. Arithmetic and iterator semantics come from
/// [`QuantizedSignedIntegerTrait`].
pub trait QuantizedSignedIntegerWrappedTrait:
    QuantizedSignedIntegerTrait + From<Self::QuantType> + AsRef<Self::QuantType> + AsMut<Self::QuantType>
{
    /// The quantization level of the underlying value.
    const LEVEL: SignedIntegerQuantizationLevel = <Self::QuantType as QuantizedSignedIntegerTrait>::LEVEL;

    const QUANT_MAX: Self;
    const QUANT_MAX_I8: Self;
    const QUANT_MAX_I16: Self;
    const QUANT_MAX_I32: Self;
    const QUANT_MAX_I64: Self;
    const QUANT_MAX_ISIZE: usize = <Self::QuantType as QuantizedSignedIntegerTrait>::QUANT_MAX_ISIZE;

    const QUANT_CLAMPED_I8: i8 = <Self::QuantType as QuantizedSignedIntegerTrait>::QUANT_CLAMPED_I8;
    const QUANT_CLAMPED_I16: i16 = <Self::QuantType as QuantizedSignedIntegerTrait>::QUANT_CLAMPED_I16;
    const QUANT_CLAMPED_I32: i32 = <Self::QuantType as QuantizedSignedIntegerTrait>::QUANT_CLAMPED_I32;
    const QUANT_CLAMPED_I64: i64 = <Self::QuantType as QuantizedSignedIntegerTrait>::QUANT_CLAMPED_I64;
    const QUANT_CLAMPED_ISIZE: isize = <Self::QuantType as QuantizedSignedIntegerTrait>::QUANT_CLAMPED_ISIZE;

    /// Wraps a raw quantized value into this wrapper type.
    fn wrap(value: Self::QuantType) -> Self;

    /// Extracts the inner quantized signed integer.
    fn deref(self) -> Self::QuantType;
}

/// Shared behaviour implemented by every wrapped enum generated by
/// [`create_wrapped_quantized_signed_integer`].
///
/// These enums hide the generic quantized wrapper type behind concrete variants
/// (`I8`, `I16`, `I32`, `I64`) while preserving the wrapper family semantics.
pub trait WrappedQuantizedSignedIntegerEnum:
    Copy + Clone + Send + Sync + core::fmt::Debug + core::cmp::PartialEq + core::cmp::Eq + core::hash::Hash + Sized + 'static
{
    fn get_level(&self) -> SignedIntegerQuantizationLevel;

    fn try_into_quant<Quant: QuantizedSignedIntegerUnwrappedTrait>(self) -> Result<Quant, FeagiDataValueQuantizationError>;

    fn into_quant<Quant: QuantizedSignedIntegerUnwrappedTrait>(self) -> Quant;

    fn to_isize(self) -> isize;
}

/// Creates a wrapper for quantized signed integers
#[macro_export]
macro_rules! create_wrapped_quantized_signed_integer {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(Q);

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> $struct_name<Q> {
            pub const LEVEL: $crate::values::quantizable::SignedIntegerQuantizationLevel = Q::LEVEL;
            pub const QUANT_ZERO: Self = Self::const_new(Q::QUANT_ZERO);
            pub const QUANT_ONE: Self = Self::const_new(Q::QUANT_ONE);
            pub const QUANT_MAX: Self = Self::const_new(Q::QUANT_MAX);

            pub const QUANT_MAX_I8: Self = Self::const_new(Q::QUANT_MAX_I8);
            pub const QUANT_MAX_I16: Self = Self::const_new(Q::QUANT_MAX_I16);
            pub const QUANT_MAX_I32: Self = Self::const_new(Q::QUANT_MAX_I32);
            pub const QUANT_MAX_I64: Self = Self::const_new(Q::QUANT_MAX_I64);
            pub const QUANT_MAX_ISIZE: usize = Q::QUANT_MAX_ISIZE;

            pub const QUANT_CLAMPED_I8: i8 = Q::QUANT_CLAMPED_I8;
            pub const QUANT_CLAMPED_I16: i16 = Q::QUANT_CLAMPED_I16;
            pub const QUANT_CLAMPED_I32: i32 = Q::QUANT_CLAMPED_I32;
            pub const QUANT_CLAMPED_I64: i64 = Q::QUANT_CLAMPED_I64;
            pub const QUANT_CLAMPED_ISIZE: isize = Q::QUANT_CLAMPED_ISIZE;

            pub const fn const_new(value: Q) -> Self {
                Self(value)
            }

            pub const fn const_deref(self) -> Q {
                self.0
            }

            pub fn new(v: Q) -> Self {
                Self(v)
            }

            /// Extracts the inner quantized signed integer.
            pub fn deref(self) -> Q {
                self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>
            $crate::values::quantizable::QuantizedElementBase for $struct_name<Q>
        {
            const QUANT_ZERO: Self = Self::const_new(Q::QUANT_ZERO);
            const QUANT_ONE: Self = Self::const_new(Q::QUANT_ONE);
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>
            $crate::values::quantizable::QuantizedSignedIntegerTrait for $struct_name<Q>
        {
            type QuantType = Q;
            const LEVEL: $crate::values::quantizable::SignedIntegerQuantizationLevel = Q::LEVEL;
            const QUANT_MAX: Self = Self::const_new(Q::QUANT_MAX);
            const QUANT_MAX_I8: Self = Self::const_new(Q::QUANT_MAX_I8);
            const QUANT_MAX_I16: Self = Self::const_new(Q::QUANT_MAX_I16);
            const QUANT_MAX_I32: Self = Self::const_new(Q::QUANT_MAX_I32);
            const QUANT_MAX_I64: Self = Self::const_new(Q::QUANT_MAX_I64);
            const QUANT_MAX_ISIZE: usize = Q::QUANT_MAX_ISIZE;

            const QUANT_CLAMPED_I8: i8 = Q::QUANT_CLAMPED_I8;
            const QUANT_CLAMPED_I16: i16 = Q::QUANT_CLAMPED_I16;
            const QUANT_CLAMPED_I32: i32 = Q::QUANT_CLAMPED_I32;
            const QUANT_CLAMPED_I64: i64 = Q::QUANT_CLAMPED_I64;
            const QUANT_CLAMPED_ISIZE: isize = Q::QUANT_CLAMPED_ISIZE;

            fn quant_from_isize_unchecked(value: isize) -> Self {
                Self::const_new(Q::quant_from_isize_unchecked(value))
            }

            fn quant_to_enum(self) -> $crate::values::quantizable::SignedIntegerEnum {
                self.0.quant_to_enum()
            }

            fn quant_try_from_isize(value: isize) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                Ok(Self::const_new(Q::quant_try_from_isize(value)?))
            }

            fn quant_to_isize(self) -> isize {
                self.0.quant_to_isize()
            }

            fn quant_to_i8_unchecked(self) -> i8 {
                self.0.quant_to_i8_unchecked()
            }

            fn quant_to_i16_unchecked(self) -> i16 {
                self.0.quant_to_i16_unchecked()
            }

            fn quant_to_i32_unchecked(self) -> i32 {
                self.0.quant_to_i32_unchecked()
            }

            fn quant_to_i64_unchecked(self) -> i64 {
                self.0.quant_to_i64_unchecked()
            }

            fn from_quantization_unchecked<FromQuant: $crate::values::quantizable::QuantizedSignedIntegerTrait>(
                value: FromQuant,
            ) -> Self {
                Self::const_new(Q::from_quantization_unchecked(value))
            }

            fn from_quantization_clamped<FromQuant: $crate::values::quantizable::QuantizedSignedIntegerTrait>(
                value: FromQuant,
            ) -> Self {
                Self::const_new(Q::from_quantization_clamped(value))
            }

            fn try_from_quantization<FromQuant: $crate::values::quantizable::QuantizedSignedIntegerTrait>(
                value: FromQuant,
            ) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                Ok(Self::const_new(Q::try_from_quantization(value)?))
            }

            fn clamp_for_quantization<ClampFor: $crate::values::quantizable::QuantizedSignedIntegerTrait>(
                self,
            ) -> Self {
                Self::const_new(self.0.clamp_for_quantization::<ClampFor>())
            }

            fn clamp_for_quantization_level_runtime(
                self,
                level: $crate::values::quantizable::SignedIntegerQuantizationLevel,
            ) -> Self {
                Self::const_new(self.0.clamp_for_quantization_level_runtime(level))
            }

            fn is_negative(&self) -> bool {
                self.0.is_negative()
            }

            fn is_zero_or_negative(&self) -> bool {
                self.0.is_zero_or_negative()
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>
            $crate::values::quantizable::QuantizedSignedIntegerWrappedTrait for $struct_name<Q>
        {
            const QUANT_MAX: Self = Self::const_new(Q::QUANT_MAX);
            const QUANT_MAX_I8: Self = Self::const_new(Q::QUANT_MAX_I8);
            const QUANT_MAX_I16: Self = Self::const_new(Q::QUANT_MAX_I16);
            const QUANT_MAX_I32: Self = Self::const_new(Q::QUANT_MAX_I32);
            const QUANT_MAX_I64: Self = Self::const_new(Q::QUANT_MAX_I64);

            fn wrap(value: Q) -> Self {
                Self(value)
            }

            fn deref(self) -> Q {
                self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> From<Q> for $struct_name<Q> {
            fn from(value: Q) -> Self {
                Self(value)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> From<&Q> for &$struct_name<Q> {
            fn from(value: &Q) -> Self {
                unsafe { &*(value as *const Q as *const $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> From<&mut Q> for &mut $struct_name<Q> {
            fn from(value: &mut Q) -> Self {
                unsafe { &mut *(value as *mut Q as *mut $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> AsRef<Q> for $struct_name<Q> {
            fn as_ref(&self) -> &Q {
                &self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> AsMut<Q> for $struct_name<Q> {
            fn as_mut(&mut self) -> &mut Q {
                &mut self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> core::fmt::Display for $struct_name<Q> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Display::fmt(&self.0, f)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> ::serde::Serialize for $struct_name<Q> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                ::serde::Serialize::serialize(&self.0, serializer)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> core::iter::Sum for $struct_name<Q> {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                Self(
                    iter.map($crate::values::quantizable::QuantizedSignedIntegerWrappedTrait::deref)
                        .sum(),
                )
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> core::iter::Product for $struct_name<Q> {
            fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
                Self(
                    iter.map($crate::values::quantizable::QuantizedSignedIntegerWrappedTrait::deref)
                        .product(),
                )
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> core::ops::Add for $struct_name<Q> {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> core::ops::Sub for $struct_name<Q> {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> core::ops::Mul for $struct_name<Q> {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> core::ops::Div for $struct_name<Q> {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> core::ops::Rem for $struct_name<Q> {
            type Output = Self;
            fn rem(self, rhs: Self) -> Self::Output {
                Self(self.0 % rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> core::ops::AddAssign for $struct_name<Q> {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> core::ops::SubAssign for $struct_name<Q> {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> core::ops::MulAssign for $struct_name<Q> {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> core::ops::DivAssign for $struct_name<Q> {
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> core::ops::RemAssign for $struct_name<Q> {
            fn rem_assign(&mut self, rhs: Self) {
                self.0 %= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> Default for $struct_name<Q> {
            fn default() -> Self {
                Self(Q::default())
            }
        }

        ::paste::paste! {
            #[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
            $vis enum [<$struct_name Enum>] {
                I8($struct_name<i8>),
                I16($struct_name<i16>),
                I32($struct_name<i32>),
                I64($struct_name<i64>),
            }

            impl [<$struct_name Enum>] {
                pub fn new_from_quantized<FromQuant: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    value: $struct_name<FromQuant>
                ) -> Self {
                    Self::from_signed_integer_enum(
                        $crate::values::quantizable::SignedIntegerEnum::new_from_quantized(value.deref())
                    )
                }

                pub fn from_signed_integer_enum(value: $crate::values::quantizable::SignedIntegerEnum) -> Self {
                    match value {
                        $crate::values::quantizable::SignedIntegerEnum::I8(v) => {
                            Self::I8($struct_name::<i8>::new(v))
                        }
                        $crate::values::quantizable::SignedIntegerEnum::I16(v) => {
                            Self::I16($struct_name::<i16>::new(v))
                        }
                        $crate::values::quantizable::SignedIntegerEnum::I32(v) => {
                            Self::I32($struct_name::<i32>::new(v))
                        }
                        $crate::values::quantizable::SignedIntegerEnum::I64(v) => {
                            Self::I64($struct_name::<i64>::new(v))
                        }
                    }
                }

                pub fn into_signed_integer_enum(self) -> $crate::values::quantizable::SignedIntegerEnum {
                    match self {
                        Self::I8(v) => $crate::values::quantizable::SignedIntegerEnum::I8(v.deref()),
                        Self::I16(v) => $crate::values::quantizable::SignedIntegerEnum::I16(v.deref()),
                        Self::I32(v) => $crate::values::quantizable::SignedIntegerEnum::I32(v.deref()),
                        Self::I64(v) => $crate::values::quantizable::SignedIntegerEnum::I64(v.deref()),
                    }
                }

                pub fn get_level(&self) -> $crate::values::quantizable::SignedIntegerQuantizationLevel {
                    match self {
                        Self::I8(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I8,
                        Self::I16(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I16,
                        Self::I32(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I32,
                        Self::I64(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I64,
                    }
                }

                pub fn try_into_quant<Quant: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> Result<Quant, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    match self {
                        Self::I8(value) => Quant::try_from_quantization(value.deref()),
                        Self::I16(value) => Quant::try_from_quantization(value.deref()),
                        Self::I32(value) => Quant::try_from_quantization(value.deref()),
                        Self::I64(value) => Quant::try_from_quantization(value.deref()),
                    }
                }

                pub fn into_quant<Quant: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(self) -> Quant {
                    match self {
                        Self::I8(value) => Quant::from_quantization_unchecked(value.deref()),
                        Self::I16(value) => Quant::from_quantization_unchecked(value.deref()),
                        Self::I32(value) => Quant::from_quantization_unchecked(value.deref()),
                        Self::I64(value) => Quant::from_quantization_unchecked(value.deref()),
                    }
                }

                pub fn to_isize(self) -> isize {
                    match self {
                        Self::I8(value) => value.deref() as isize,
                        Self::I16(value) => value.deref() as isize,
                        Self::I32(value) => value.deref() as isize,
                        Self::I64(value) => value.deref() as isize,
                    }
                }

                pub fn try_into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> Result<$struct_name<Quant>, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    Ok($struct_name::<Quant>::new(self.try_into_quant::<Quant>()?))
                }

                pub fn into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> $struct_name<Quant> {
                    $struct_name::<Quant>::new(self.into_quant::<Quant>())
                }
            }

            impl $crate::values::quantizable::WrappedQuantizedSignedIntegerEnum for [<$struct_name Enum>] {
                fn get_level(&self) -> $crate::values::quantizable::SignedIntegerQuantizationLevel {
                    [<$struct_name Enum>]::get_level(self)
                }

                fn try_into_quant<Quant: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> Result<Quant, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    [<$struct_name Enum>]::try_into_quant(self)
                }

                fn into_quant<Quant: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(self) -> Quant {
                    [<$struct_name Enum>]::into_quant(self)
                }

                fn to_isize(self) -> isize {
                    [<$struct_name Enum>]::to_isize(self)
                }
            }
        }
    };
}
