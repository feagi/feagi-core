use crate::values::quantizable::feagi_data_value_quantization_error::FeagiFailQuantizationOutOfRange;
use crate::values::quantizable::{FeagiDataValueQuantizationError, QuantizationLevelPacking, QuantizedElementBase};

//region Value

#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum SignedIntegerQuantizationLevel {
    I8 = 0,
    I16 = 1,
    I32 = 2,
    I64 = 3,
    Isize = 4,
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
            4 => Ok(SignedIntegerQuantizationLevel::Isize),
            _ => Err(()),
        }
    }
}

impl QuantizationLevelPacking for SignedIntegerQuantizationLevel {
    const NUMBER_BITS: usize = 3;

    unsafe fn from_unpacked_byte(byte: u8) -> Self {
        core::mem::transmute(byte)
    }
}

/// Trait designed to hold Sint data values in a quantized form
pub trait QuantizedSignedIntegerTrait:
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
{
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
    fn quant_from_isize(value: isize) -> Self;

    /// Will wrap whatever quant this is to an `SignedIntegerEnum`
    fn quant_to_enum(value: Self) -> SignedIntegerEnum;

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
    fn try_from_quantization<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError>;

    /// Converts to another quantization. Does not check for validity of ranges!
    fn to_quantization_unchecked<ToQuant: QuantizedSignedIntegerTrait>(self) -> ToQuant {
        ToQuant::from_quantization_unchecked::<Self>(self)
    }

    /// Converts to another quantization, clamping its values to ensure it fits
    fn to_quantization_clamped<ToQuant: QuantizedSignedIntegerTrait>(self) -> ToQuant {
        ToQuant::from_quantization_clamped(self)
    }

    /// Tries to convert to another quantization, returns an error if it would break the bounds
    fn try_to_quantization<ToQuant: QuantizedSignedIntegerTrait>(self) -> Result<ToQuant, FeagiDataValueQuantizationError> {
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

impl QuantizedSignedIntegerTrait for i8 {
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

    fn quant_from_isize(value: isize) -> Self {
        value as i8
    }

    fn quant_to_enum(value: Self) -> SignedIntegerEnum {
        SignedIntegerEnum::I8(value)
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
        Self::quant_from_isize(clamp_isize_for_signed_quant::<Self>(value.quant_to_isize()))
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
        Self::quant_from_isize(clamp_isize_for_signed_quant_level(self.quant_to_isize(), level))
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

impl QuantizedSignedIntegerTrait for i16 {
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

    fn quant_from_isize(value: isize) -> Self {
        value as i16
    }

    fn quant_to_enum(value: Self) -> SignedIntegerEnum {
        SignedIntegerEnum::I16(value)
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
        Self::quant_from_isize(clamp_isize_for_signed_quant::<Self>(value.quant_to_isize()))
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
        Self::quant_from_isize(clamp_isize_for_signed_quant_level(self.quant_to_isize(), level))
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

// lol, lmao even
impl QuantizedSignedIntegerTrait for i32 {
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

    fn quant_from_isize(value: isize) -> Self {
        value as i32
    }

    fn quant_to_enum(value: Self) -> SignedIntegerEnum {
        SignedIntegerEnum::I32(value)
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
        Self::quant_from_isize(clamp_isize_for_signed_quant::<Self>(value.quant_to_isize()))
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
        Self::quant_from_isize(clamp_isize_for_signed_quant_level(self.quant_to_isize(), level))
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

impl QuantizedSignedIntegerTrait for i64 {
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

    fn quant_from_isize(value: isize) -> Self {
        value as i64
    }

    fn quant_to_enum(value: Self) -> SignedIntegerEnum {
        SignedIntegerEnum::I64(value)
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
        Self::quant_from_isize(clamp_isize_for_signed_quant::<Self>(value.quant_to_isize()))
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
        Self::quant_from_isize(clamp_isize_for_signed_quant_level(self.quant_to_isize(), level))
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

// Note: Specifically we will not support isize directly since it can vary in size depending on
// backend, which could cause some issues with device interoperability
impl QuantizedSignedIntegerTrait for isize {
    const LEVEL: SignedIntegerQuantizationLevel = SignedIntegerQuantizationLevel::Isize;
    const QUANT_MAX: Self = isize::MAX;
    const QUANT_MAX_I8: Self = i8::MAX as isize;
    const QUANT_MAX_I16: Self = i16::MAX as isize;
    const QUANT_MAX_I32: Self = i32::MAX as isize;
    const QUANT_MAX_I64: Self = isize::MAX;
    const QUANT_MAX_ISIZE: usize = isize::MAX as usize;

    const QUANT_CLAMPED_I8: i8 = i8::MAX;
    const QUANT_CLAMPED_I16: i16 = i16::MAX;
    const QUANT_CLAMPED_I32: i32 = i32::MAX;
    const QUANT_CLAMPED_I64: i64 = i64::MAX;
    const QUANT_CLAMPED_ISIZE: isize = isize::MAX;

    fn quant_from_isize(value: isize) -> Self {
        value
    }

    fn quant_to_enum(value: Self) -> SignedIntegerEnum {
        SignedIntegerEnum::I64(value as i64)
    }

    fn quant_try_from_isize(value: isize) -> Result<Self, FeagiDataValueQuantizationError> {
        Ok(value)
    }

    fn quant_to_isize(self) -> isize {
        self
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
        self as i64
    }

    fn from_quantization_unchecked<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self {
        value.quant_to_isize()
    }

    fn from_quantization_clamped<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self {
        clamp_isize_for_signed_quant::<Self>(value.quant_to_isize())
    }

    fn try_from_quantization<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        Ok(value.quant_to_isize())
    }

    fn clamp_for_quantization<ClampFor: QuantizedSignedIntegerTrait>(self) -> Self {
        self.clamp_for_quantization_level_runtime(ClampFor::LEVEL)
    }

    fn clamp_for_quantization_level_runtime(self, level: SignedIntegerQuantizationLevel) -> Self {
        clamp_isize_for_signed_quant_level(self, level)
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
        FromQuant::quant_to_enum(value)
    }

    pub fn get_level(&self) -> SignedIntegerQuantizationLevel {
        match self {
            SignedIntegerEnum::I8(_) => SignedIntegerQuantizationLevel::I8,
            SignedIntegerEnum::I16(_) => SignedIntegerQuantizationLevel::I16,
            SignedIntegerEnum::I32(_) => SignedIntegerQuantizationLevel::I32,
            SignedIntegerEnum::I64(_) => SignedIntegerQuantizationLevel::I64,
        }
    }

    pub fn try_into_quant<Quant: QuantizedSignedIntegerTrait>(self) -> Result<Quant, FeagiDataValueQuantizationError> {
        let value = self.to_isize();
        if !signed_value_fits_quant::<Quant>(value) {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized signed integer exceeds target quantization!", value as usize).into());
        }
        Ok(Quant::quant_from_isize(value))
    }

    pub fn into_quant<Quant: QuantizedSignedIntegerTrait>(self) -> Quant {
        Quant::quant_from_isize(self.to_isize())
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
        SignedIntegerQuantizationLevel::Isize => value,
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
        SignedIntegerQuantizationLevel::Isize => true,
    }
}

//endregion

//region Wrapper

/// Shared behaviour implemented by every strongly-typed wrapper generated by
/// [`create_wrapped_quantized_signed_integer`].
///
/// Each wrapper produced by the macro is a distinct `#[repr(transparent)]` newtype, so that
/// logically different signed integer values cannot be accidentally mixed at compile time. This
/// trait exposes the common surface those wrappers share, allowing functions to generically accept
/// "some wrapped signed integer" (e.g. `fn foo<I: WrappedQuantizedSignedInteger>(value: I)`) while
/// still preserving the compile-time distinctness of the concrete wrapper types.
///
/// The bulk of the behaviour is provided here as default methods that delegate to the underlying
/// [`QuantizedSignedIntegerTrait`] value; the macro only needs to supply [`Self::new`],
/// [`Self::dewrap`] and the `Self`-typed constants.
pub trait WrappedQuantizedSignedInteger:
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
    /// The underlying quantized signed integer value this wrapper stores.
    type Quant: QuantizedSignedIntegerTrait;

    /// The quantization level of the underlying value.
    const LEVEL: SignedIntegerQuantizationLevel = <Self::Quant as QuantizedSignedIntegerTrait>::LEVEL;

    /// Zero, expressed in the wrapper's own type.
    const QUANT_ZERO: Self;
    /// One, expressed in the wrapper's own type.
    const QUANT_ONE: Self;
    /// The maximum representable value, expressed in the wrapper's own type.
    const QUANT_MAX: Self;

    const QUANT_MAX_I8: Self;
    const QUANT_MAX_I16: Self;
    const QUANT_MAX_I32: Self;
    const QUANT_MAX_I64: Self;
    const QUANT_MAX_ISIZE: usize = <Self::Quant as QuantizedSignedIntegerTrait>::QUANT_MAX_ISIZE;

    const QUANT_CLAMPED_I8: i8 = <Self::Quant as QuantizedSignedIntegerTrait>::QUANT_CLAMPED_I8;
    const QUANT_CLAMPED_I16: i16 = <Self::Quant as QuantizedSignedIntegerTrait>::QUANT_CLAMPED_I16;
    const QUANT_CLAMPED_I32: i32 = <Self::Quant as QuantizedSignedIntegerTrait>::QUANT_CLAMPED_I32;
    const QUANT_CLAMPED_I64: i64 = <Self::Quant as QuantizedSignedIntegerTrait>::QUANT_CLAMPED_I64;
    const QUANT_CLAMPED_ISIZE: isize = <Self::Quant as QuantizedSignedIntegerTrait>::QUANT_CLAMPED_ISIZE;

    /// Wraps a raw quantized value into this wrapper type.
    fn new(value: Self::Quant) -> Self;

    /// Extracts the inner quantized signed integer.
    fn dewrap(self) -> Self::Quant;

    /// Tries to convert from isize, does NOT check bounds!
    fn quant_from_isize(value: isize) -> Self {
        Self::new(Self::Quant::quant_from_isize(value))
    }

    /// Tries converting from isize, returns an error if out of bounds
    fn quant_try_from_isize(value: isize) -> Result<Self, FeagiDataValueQuantizationError> {
        Ok(Self::new(Self::Quant::quant_try_from_isize(value)?))
    }

    /// Converts to isize.
    fn quant_to_isize(self) -> isize {
        self.dewrap().quant_to_isize()
    }

    /// Tries to convert to i8, does NOT check bounds!
    fn quant_to_i8_unchecked(self) -> i8 {
        self.dewrap().quant_to_i8_unchecked()
    }

    /// Tries to convert to i16, does NOT check bounds!
    fn quant_to_i16_unchecked(self) -> i16 {
        self.dewrap().quant_to_i16_unchecked()
    }

    /// Tries to convert to i32, does NOT check bounds!
    fn quant_to_i32_unchecked(self) -> i32 {
        self.dewrap().quant_to_i32_unchecked()
    }

    /// Tries to convert to i64, does NOT check bounds!
    fn quant_to_i64_unchecked(self) -> i64 {
        self.dewrap().quant_to_i64_unchecked()
    }

    /// Will wrap whatever quant this is to a [`SignedIntegerEnum`]
    fn quant_to_enum(self) -> SignedIntegerEnum {
        Self::Quant::quant_to_enum(self.dewrap())
    }

    /// Creates from a value of another quantization. Does not check for validity of ranges!
    fn from_quantization_unchecked<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self {
        Self::new(Self::Quant::from_quantization_unchecked(value))
    }

    /// Creates from a value of another quantization, clamping its values to ensure it fits
    fn from_quantization_clamped<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Self {
        Self::new(Self::Quant::from_quantization_clamped(value))
    }

    /// Tries to create a value of another quantization, returns an error if it would break the bounds
    fn try_from_quantization<FromQuant: QuantizedSignedIntegerTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        Ok(Self::new(Self::Quant::try_from_quantization(value)?))
    }

    /// Converts to another quantization. Does not check for validity of ranges!
    fn to_quantization_unchecked<ToQuant: QuantizedSignedIntegerTrait>(self) -> ToQuant {
        self.dewrap().to_quantization_unchecked()
    }

    /// Converts to another quantization, clamping its values to ensure it fits
    fn to_quantization_clamped<ToQuant: QuantizedSignedIntegerTrait>(self) -> ToQuant {
        self.dewrap().to_quantization_clamped()
    }

    /// Tries to convert to another quantization, returns an error if it would break the bounds
    fn try_to_quantization<ToQuant: QuantizedSignedIntegerTrait>(self) -> Result<ToQuant, FeagiDataValueQuantizationError> {
        self.dewrap().try_to_quantization()
    }

    /// Clamps the value for another quantization, but does not actually change the quantization itself
    fn clamp_for_quantization<ClampFor: QuantizedSignedIntegerTrait>(self) -> Self {
        Self::new(self.dewrap().clamp_for_quantization::<ClampFor>())
    }

    /// Clamps the value for a runtime-provided quantization level, but does not actually change the quantization itself
    fn clamp_for_quantization_level_runtime(self, level: SignedIntegerQuantizationLevel) -> Self {
        Self::new(self.dewrap().clamp_for_quantization_level_runtime(level))
    }

    /// Returns true if the value is zero
    fn is_zero(self) -> bool {
        self == Self::QUANT_ZERO
    }

    fn is_negative(&self) -> bool {
        self.dewrap().is_negative()
    }

    fn is_zero_or_negative(&self) -> bool {
        self.dewrap().is_zero_or_negative()
    }
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

    fn try_into_quant<Quant: QuantizedSignedIntegerTrait>(self) -> Result<Quant, FeagiDataValueQuantizationError>;

    fn into_quant<Quant: QuantizedSignedIntegerTrait>(self) -> Quant;

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
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait>(Q);

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> $struct_name<Q> {
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

            /// Extracts the inner quantized signed integer
            pub fn deref(self) -> Q {
                self.0
            }

            pub fn is_negative(&self) -> bool {
                self.0.is_negative()
            }

            pub fn is_zero_or_negative(&self) -> bool {
                self.0.is_zero_or_negative()
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait>
            $crate::values::quantizable::WrappedQuantizedSignedInteger for $struct_name<Q>
        {
            type Quant = Q;

            const QUANT_ZERO: Self = Self::const_new(Q::QUANT_ZERO);
            const QUANT_ONE: Self = Self::const_new(Q::QUANT_ONE);
            const QUANT_MAX: Self = Self::const_new(Q::QUANT_MAX);
            const QUANT_MAX_I8: Self = Self::const_new(Q::QUANT_MAX_I8);
            const QUANT_MAX_I16: Self = Self::const_new(Q::QUANT_MAX_I16);
            const QUANT_MAX_I32: Self = Self::const_new(Q::QUANT_MAX_I32);
            const QUANT_MAX_I64: Self = Self::const_new(Q::QUANT_MAX_I64);

            fn new(value: Q) -> Self {
                Self(value)
            }

            fn deref(self) -> Q {
                self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> From<Q> for $struct_name<Q> {
            fn from(value: Q) -> Self {
                Self(value)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> From<&Q> for &$struct_name<Q> {
            fn from(value: &Q) -> Self {
                unsafe { &*(value as *const Q as *const $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> From<&mut Q> for &mut $struct_name<Q> {
            fn from(value: &mut Q) -> Self {
                unsafe { &mut *(value as *mut Q as *mut $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> AsRef<Q> for $struct_name<Q> {
            fn as_ref(&self) -> &Q {
                &self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> AsMut<Q> for $struct_name<Q> {
            fn as_mut(&mut self) -> &mut Q {
                &mut self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> core::ops::Add for $struct_name<Q> {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> core::ops::Sub for $struct_name<Q> {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> core::ops::Mul for $struct_name<Q> {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> core::ops::Div for $struct_name<Q> {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> core::ops::Rem for $struct_name<Q> {
            type Output = Self;
            fn rem(self, rhs: Self) -> Self::Output {
                Self(self.0 % rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> core::ops::AddAssign for $struct_name<Q> {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> core::ops::SubAssign for $struct_name<Q> {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> core::ops::MulAssign for $struct_name<Q> {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> core::ops::DivAssign for $struct_name<Q> {
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> core::ops::RemAssign for $struct_name<Q> {
            fn rem_assign(&mut self, rhs: Self) {
                self.0 %= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedSignedIntegerTrait> Default for $struct_name<Q> {
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
                pub fn new_from_quantized<FromQuant: $crate::values::quantizable::QuantizedSignedIntegerTrait>(
                    value: $struct_name<FromQuant>
                ) -> Self {
                    let as_isize = $crate::values::quantizable::SignedIntegerEnum::new_from_quantized(value.deref()).to_isize();
                    match FromQuant::LEVEL {
                        $crate::values::quantizable::SignedIntegerQuantizationLevel::I8 => {
                            Self::I8($struct_name::<i8>::new(<i8 as $crate::values::quantizable::QuantizedSignedIntegerTrait>::quant_from_isize(as_isize)))
                        }
                        $crate::values::quantizable::SignedIntegerQuantizationLevel::I16 => {
                            Self::I16($struct_name::<i16>::new(<i16 as $crate::values::quantizable::QuantizedSignedIntegerTrait>::quant_from_isize(as_isize)))
                        }
                        $crate::values::quantizable::SignedIntegerQuantizationLevel::I32 => {
                            Self::I32($struct_name::<i32>::new(<i32 as $crate::values::quantizable::QuantizedSignedIntegerTrait>::quant_from_isize(as_isize)))
                        }
                        $crate::values::quantizable::SignedIntegerQuantizationLevel::I64 => {
                            Self::I64($struct_name::<i64>::new(<i64 as $crate::values::quantizable::QuantizedSignedIntegerTrait>::quant_from_isize(as_isize)))
                        }
                        $crate::values::quantizable::SignedIntegerQuantizationLevel::Isize => {
                            Self::I64($struct_name::<i64>::new(<i64 as $crate::values::quantizable::QuantizedSignedIntegerTrait>::quant_from_isize(as_isize)))
                        }
                    }
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

                pub fn try_into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedSignedIntegerTrait>(
                    self
                ) -> Result<$struct_name<Quant>, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    Ok($struct_name::<Quant>::new(
                        <Self as $crate::values::quantizable::WrappedQuantizedSignedIntegerEnum>::try_into_quant::<Quant>(self)?
                    ))
                }

                pub fn into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedSignedIntegerTrait>(
                    self
                ) -> $struct_name<Quant> {
                    $struct_name::<Quant>::new(
                        <Self as $crate::values::quantizable::WrappedQuantizedSignedIntegerEnum>::into_quant::<Quant>(self)
                    )
                }
            }

            impl $crate::values::quantizable::WrappedQuantizedSignedIntegerEnum for [<$struct_name Enum>] {
                fn get_level(&self) -> $crate::values::quantizable::SignedIntegerQuantizationLevel {
                    match self {
                        Self::I8(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I8,
                        Self::I16(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I16,
                        Self::I32(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I32,
                        Self::I64(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I64,
                    }
                }

                fn try_into_quant<Quant: $crate::values::quantizable::QuantizedSignedIntegerTrait>(
                    self
                ) -> Result<Quant, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    match self {
                        Self::I8(value) => $crate::values::quantizable::SignedIntegerEnum::I8(value.deref()).try_into_quant(),
                        Self::I16(value) => $crate::values::quantizable::SignedIntegerEnum::I16(value.deref()).try_into_quant(),
                        Self::I32(value) => $crate::values::quantizable::SignedIntegerEnum::I32(value.deref()).try_into_quant(),
                        Self::I64(value) => $crate::values::quantizable::SignedIntegerEnum::I64(value.deref()).try_into_quant(),
                    }
                }

                fn into_quant<Quant: $crate::values::quantizable::QuantizedSignedIntegerTrait>(self) -> Quant {
                    match self {
                        Self::I8(value) => Quant::quant_from_isize(value.deref() as isize),
                        Self::I16(value) => Quant::quant_from_isize(value.deref() as isize),
                        Self::I32(value) => Quant::quant_from_isize(value.deref() as isize),
                        Self::I64(value) => Quant::quant_from_isize(value.deref() as isize),
                    }
                }

                fn to_isize(self) -> isize {
                    match self {
                        Self::I8(value) => value.deref() as isize,
                        Self::I16(value) => value.deref() as isize,
                        Self::I32(value) => value.deref() as isize,
                        Self::I64(value) => value.deref() as isize,
                    }
                }
            }
        }
    };
}

//endregion