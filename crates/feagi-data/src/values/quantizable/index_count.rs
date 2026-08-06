use crate::values::quantizable::feagi_data_value_quantization_error::FeagiFailQuantizationOutOfRange;
use crate::values::quantizable::{FeagiDataValueQuantizationError, QuantizationLevelPacking, QuantizedElementBase};

///
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum IndexCountQuantizationLevel {
    U8 = 0,
    U16 = 1,
    U32 = 2,
    U64 = 3,
    // we are NOT doing u128 lol
    Usize = 4,
    // We can support a max of 8
}

impl Into<u8> for IndexCountQuantizationLevel {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for IndexCountQuantizationLevel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(IndexCountQuantizationLevel::U8),
            1 => Ok(IndexCountQuantizationLevel::U16),
            2 => Ok(IndexCountQuantizationLevel::U32),
            3 => Ok(IndexCountQuantizationLevel::U64),
            4 => Ok(IndexCountQuantizationLevel::Usize),
            _ => Err(()),
        }
    }
}

impl QuantizationLevelPacking for IndexCountQuantizationLevel {
    const NUMBER_BITS: usize = 3;

    unsafe fn from_unpacked_byte(byte: u8) -> Self {
        core::mem::transmute(byte)
    }
}

/// Trait designed to hold index and/or count values in a quantized form
pub trait QuantizedIndexCountTrait:
Copy
+ Clone
+ Send
+ Sync
+ Default
+ core::ops::Add<Output=Self>
+ core::ops::Sub<Output=Self>
+ core::ops::Mul<Output=Self>
+ core::ops::Div<Output=Self>
+ core::ops::AddAssign
+ core::ops::SubAssign
+ core::ops::MulAssign
+ core::ops::DivAssign
+ core::cmp::PartialOrd
+ core::cmp::Ord
+ core::iter::Sum
+ core::fmt::Debug
+ core::fmt::Display
+ core::ops::Rem<Output=Self>
+ core::ops::RemAssign
+ core::cmp::Eq
+ core::hash::Hash
+ Sized
+ 'static
+ QuantizedElementBase
{
    const LEVEL: IndexCountQuantizationLevel;
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

    /// Tries to convert from usize, does NOT check bounds!
    fn quant_from_usize(value: usize) -> Self;

    /// Will wrap whatever quant this is to an `IndexCountEnum`
    fn quant_to_enum(value: Self) -> IndexCountEnum;

    /// Tries converting from usize, returns an error if out of bounds
    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError>;

    /// Converts to usize. No need to check as we have no indexes that will exceed a usize on a
    /// system
    fn quant_to_usize(self) -> usize;

    /// Tries to convert from u32, does NOT check bounds!
    fn quant_to_u8(self) -> u8;
    
    /// Tries to convert to u16, does NOT check bounds!
    fn quant_to_u16(self) -> u16;

    /// Tries to convert to u32, does NOT check bounds!
    fn quant_to_u32(self) -> u32;
    
    /// Tries to convert to u64, bound checking shouldnt matter since this is the biggest type (not doing u128 lol)
    fn quant_to_u64(self) -> u64;

    /// Creates from an index of another quantization. Does not check for validity of ranges!
    fn from_quantization<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Self;

    /// Creates from an index of another quantization, clamping its values to ensure it fits
    fn from_quantization_clamped<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Self;

    /// Tries to create an index of another quantization, returns an error if it would break the bounds
    fn try_from_quantization<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError>;

    /// Converts to an index of another quantization. Does not check for validity of ranges!
    fn to_quantization<ToQuant: QuantizedIndexCountTrait>(self) -> ToQuant {
        ToQuant::from_quantization::<Self>(self)
    }

    /// Converts to an index of another quantization, clamping its values to ensure it fits
    fn to_quantization_clamped<ToQuant: QuantizedIndexCountTrait>(self) -> ToQuant {
        ToQuant::from_quantization_clamped(self)
    }

    /// Tries to convert to an index of another quantization, returns an error if it would break the bounds
    fn try_to_quantization<ToQuant: QuantizedIndexCountTrait>(self) -> Result<ToQuant, FeagiDataValueQuantizationError> {
        ToQuant::try_from_quantization(self)
    }

    /// Clamps the value of this index for another quantization, but does not actually change the
    /// quantization itself
    fn clamp_for_quantization<ClampFor: QuantizedIndexCountTrait>(self) -> Self;

    
    fn clamp_for_quantization_level_runtime(self, level: IndexCountQuantizationLevel) -> Self;
}

impl QuantizedIndexCountTrait for u8 {
    const LEVEL: IndexCountQuantizationLevel = IndexCountQuantizationLevel::U8;
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

    fn quant_from_usize(value: usize) -> Self {
        value as u8
    }

    fn quant_to_enum(value: Self) -> IndexCountEnum {
        IndexCountEnum::U8(value)
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

    fn from_quantization<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Self {
        value.quant_to_u8()
    }

    fn from_quantization_clamped<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Self {
        if value > FromQuant::QUANT_MAX_U8 {
            return u8::MAX
        }
        value.quant_to_u8()
    }

    fn try_from_quantization<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > FromQuant::QUANT_MAX_U8 {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized index exceeds u8 quantization!", value.quant_to_usize()).into());
        }
        Ok(value.quant_to_u8())
    }

    fn clamp_for_quantization<ClampFor: QuantizedIndexCountTrait>(self) -> Self {
        self.min(ClampFor::QUANT_CLAMPED_U8)
    }

    fn clamp_for_quantization_level_runtime(self, level: IndexCountQuantizationLevel) -> Self {
        match level {
            IndexCountQuantizationLevel::U8 => self,
            IndexCountQuantizationLevel::U16 => self,
            IndexCountQuantizationLevel::U32 => self,
            IndexCountQuantizationLevel::U64 => self,
            IndexCountQuantizationLevel::Usize => self,
        }
    }
}

impl QuantizedIndexCountTrait for u16 {
    const LEVEL: IndexCountQuantizationLevel = IndexCountQuantizationLevel::U16;
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

    fn quant_from_usize(value: usize) -> Self {
        value as u16
    }

    fn quant_to_enum(value: Self) -> IndexCountEnum {
        IndexCountEnum::U16(value)
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

    fn from_quantization<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Self {
        value.quant_to_u16()
    }

    fn from_quantization_clamped<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Self {
        if value > FromQuant::QUANT_MAX_U16 {
            return u16::MAX
        }
        value.quant_to_u16()
    }

    fn try_from_quantization<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > FromQuant::QUANT_MAX_U16 {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized index exceeds u16 quantization!", value.quant_to_usize()).into());
        }
        Ok(value.quant_to_u16())
    }

    fn clamp_for_quantization<ClampFor: QuantizedIndexCountTrait>(self) -> Self {
        self.min(ClampFor::QUANT_CLAMPED_U16)
    }

    fn clamp_for_quantization_level_runtime(self, level: IndexCountQuantizationLevel) -> Self {
        match level {
            IndexCountQuantizationLevel::U8 => self.min(255),
            IndexCountQuantizationLevel::U16 => self,
            IndexCountQuantizationLevel::U32 => self,
            IndexCountQuantizationLevel::U64 => self,
            IndexCountQuantizationLevel::Usize => self,
        }
    }
}


// lol, lmao even
impl QuantizedIndexCountTrait for u32 {
    const LEVEL: IndexCountQuantizationLevel = IndexCountQuantizationLevel::U32;
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

    fn quant_from_usize(value: usize) -> Self {
        value as u32
    }

    fn quant_to_enum(value: Self) -> IndexCountEnum {
        IndexCountEnum::U32(value)
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

    fn from_quantization<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Self {
        value.quant_to_u32()
    }

    fn from_quantization_clamped<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Self {
        if value > FromQuant::QUANT_MAX_U32 {
            return u32::MAX
        }
        value.quant_to_u32()
    }

    fn try_from_quantization<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > FromQuant::QUANT_MAX_U32 {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized index exceeds u32 quantization!", value.quant_to_usize()).into());
        }
        Ok(value.quant_to_u32())
    }

    fn clamp_for_quantization<ClampFor: QuantizedIndexCountTrait>(self) -> Self {
        self.min(ClampFor::QUANT_CLAMPED_U32)
    }

    fn clamp_for_quantization_level_runtime(self, level: IndexCountQuantizationLevel) -> Self {
        match level {
            IndexCountQuantizationLevel::U8 => self.min(u8::MAX as u32),
            IndexCountQuantizationLevel::U16 => self.min(u16::MAX as u32),
            IndexCountQuantizationLevel::U32 => self,
            IndexCountQuantizationLevel::U64 => self,
            IndexCountQuantizationLevel::Usize => self,
        }
    }
}

impl QuantizedIndexCountTrait for u64 {
    const LEVEL: IndexCountQuantizationLevel = IndexCountQuantizationLevel::U64;
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

    fn quant_from_usize(value: usize) -> Self {
        value as u64
    }

    fn quant_to_enum(value: Self) -> IndexCountEnum {
        IndexCountEnum::U64(value)
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

    fn from_quantization<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Self {
        value.quant_to_u64()
    }

    fn from_quantization_clamped<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Self {
        if value > FromQuant::QUANT_MAX_U64 {
            return u64::MAX
        }
        value.quant_to_u64()
    }

    fn try_from_quantization<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > FromQuant::QUANT_MAX_U64 {
            return Err(FeagiFailQuantizationOutOfRange::new("Quantized index exceeds u64 quantization!", value.quant_to_usize()).into());
        }
        Ok(value.quant_to_u64())
    }

    fn clamp_for_quantization<ClampFor: QuantizedIndexCountTrait>(self) -> Self {
        self.min(ClampFor::QUANT_CLAMPED_U64)
    }

    fn clamp_for_quantization_level_runtime(self, level: IndexCountQuantizationLevel) -> Self {
        match level {
            IndexCountQuantizationLevel::U8 => self.min(u8::MAX as u64),
            IndexCountQuantizationLevel::U16 => self.min(u16::MAX as u64),
            IndexCountQuantizationLevel::U32 => self.min(u32::MAX as u64),
            IndexCountQuantizationLevel::U64 => self,
            IndexCountQuantizationLevel::Usize => self.min(usize::MAX as u64),
        }
    }
}

// Note: Specifically we will not support usize directly since it can vary in size depending on
// backend, which could cause some issues with device interoperability

/// Allows storing all quantized index types under a single enum
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum IndexCountEnum {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

impl IndexCountEnum {
    pub fn new_from_quantized<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Self {
        FromQuant::quant_to_enum(value)
    }

    pub fn get_level(&self) -> IndexCountQuantizationLevel {
        match self {
            IndexCountEnum::U8(_) => { IndexCountQuantizationLevel::U8 }
            IndexCountEnum::U16(_) => { IndexCountQuantizationLevel::U16 }
            IndexCountEnum::U32(_) => { IndexCountQuantizationLevel::U32 }
            IndexCountEnum::U64(_) => { IndexCountQuantizationLevel::U64 }
        }
    }

    pub fn try_into_quant<Quant: QuantizedIndexCountTrait>(self) -> Result<Quant, FeagiDataValueQuantizationError> {
        // TODO assert Debug Check!
        match self {
            IndexCountEnum::U8(value) => {value.try_to_quantization()}
            IndexCountEnum::U16(value) => {value.try_to_quantization()}
            IndexCountEnum::U32(value) => {value.try_to_quantization()}
            IndexCountEnum::U64(value) => {value.try_to_quantization()}
        }
    }

    pub fn into_quant<Quant: QuantizedIndexCountTrait>(self) -> Quant {
        match self {
            IndexCountEnum::U8(value) => {value.to_quantization()}
            IndexCountEnum::U16(value) => {value.to_quantization()}
            IndexCountEnum::U32(value) => {value.to_quantization()}
            IndexCountEnum::U64(value) => {value.to_quantization()}
        }
    }

    pub fn to_usize(self) -> usize {
        match self {
            IndexCountEnum::U8(value) => value as usize,
            IndexCountEnum::U16(value) => value as usize,
            IndexCountEnum::U32(value) => value as usize,
            IndexCountEnum::U64(value) => value as usize,
        }
    }

    // TODO from usize that is CPU dependent to be either 32 bit or 64 bit
}


/// Shared behaviour implemented by every strongly-typed wrapper generated by
/// [`create_wrapped_quantized_index`].
///
/// Each wrapper produced by the macro is a distinct `#[repr(transparent)]` newtype, so that
/// logically different indices / counts cannot be accidentally mixed at compile time. This trait
/// exposes the common surface those wrappers share, allowing functions to generically accept
/// "some wrapped index / count" (e.g. `fn foo<I: WrappedQuantizedIndexCount>(index: I)`) while
/// still preserving the compile-time distinctness of the concrete wrapper types.
///
/// The bulk of the behaviour is provided here as default methods that delegate to the underlying
/// [`QuantizedIndexCountTrait`] value; the macro only needs to supply [`Self::new`],
/// [`Self::deref`] and the `Self`-typed constants.
pub trait WrappedQuantizedIndexCount:
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
+ core::ops::Add<Output=Self>
+ core::ops::Sub<Output=Self>
+ core::ops::Mul<Output=Self>
+ core::ops::Div<Output=Self>
+ core::ops::Rem<Output=Self>
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
    /// The underlying quantized index / count value this wrapper stores.
    type Quant: QuantizedIndexCountTrait;

    /// The quantization level of the underlying value.
    const LEVEL: IndexCountQuantizationLevel = <Self::Quant as QuantizedIndexCountTrait>::LEVEL;

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
    const QUANT_MAX_USIZE: usize = <Self::Quant as QuantizedIndexCountTrait>::QUANT_MAX_USIZE;

    const QUANT_CLAMPED_U8: u8 = <Self::Quant as QuantizedIndexCountTrait>::QUANT_CLAMPED_U8;
    const QUANT_CLAMPED_U16: u16 = <Self::Quant as QuantizedIndexCountTrait>::QUANT_CLAMPED_U16;
    const QUANT_CLAMPED_U32: u32 = <Self::Quant as QuantizedIndexCountTrait>::QUANT_CLAMPED_U32;
    const QUANT_CLAMPED_U64: u64 = <Self::Quant as QuantizedIndexCountTrait>::QUANT_CLAMPED_U64;
    const QUANT_CLAMPED_USIZE: usize = <Self::Quant as QuantizedIndexCountTrait>::QUANT_CLAMPED_USIZE;

    /// Wraps a raw quantized value into this wrapper type.
    fn new(value: Self::Quant) -> Self;

    /// Extracts the inner quantized index / count.
    fn deref(self) -> Self::Quant;

    /// Tries to convert from usize, does NOT check bounds!
    fn quant_from_usize(value: usize) -> Self {
        Self::new(Self::Quant::quant_from_usize(value))
    }

    /// Tries converting from usize, returns an error if out of bounds
    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        Ok(Self::new(Self::Quant::quant_try_from_usize(value)?))
    }

    /// Converts to usize. No need to check as we have no indexes that will exceed a usize on a
    /// system
    fn quant_to_usize(self) -> usize {
        self.deref().quant_to_usize()
    }

    /// Tries to convert to u8, does NOT check bounds!
    fn quant_to_u8(self) -> u8 {
        self.deref().quant_to_u8()
    }

    /// Tries to convert to u16, does NOT check bounds!
    fn quant_to_u16(self) -> u16 {
        self.deref().quant_to_u16()
    }

    /// Tries to convert to u32, does NOT check bounds!
    fn quant_to_u32(self) -> u32 {
        self.deref().quant_to_u32()
    }

    /// Tries to convert to u64, bound checking shouldnt matter since this is the biggest type (not doing u128 lol)
    fn quant_to_u64(self) -> u64 {
        self.deref().quant_to_u64()
    }

    /// Creates from an index of another quantization. Does not check for validity of ranges!
    fn from_quantization<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Self {
        Self::new(Self::Quant::from_quantization(value))
    }

    /// Creates from an index of another quantization, clamping its values to ensure it fits
    fn from_quantization_clamped<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Self {
        Self::new(Self::Quant::from_quantization_clamped(value))
    }

    /// Tries to create an index of another quantization, returns an error if it would break the bounds
    fn try_from_quantization<FromQuant: QuantizedIndexCountTrait>(value: FromQuant) -> Result<Self, FeagiDataValueQuantizationError> {
        Ok(Self::new(Self::Quant::try_from_quantization(value)?))
    }

    /// Converts to an index of another quantization. Does not check for validity of ranges!
    fn to_quantization<ToQuant: QuantizedIndexCountTrait>(self) -> ToQuant {
        self.deref().to_quantization()
    }

    /// Converts to an index of another quantization, clamping its values to ensure it fits
    fn to_quantization_clamped<ToQuant: QuantizedIndexCountTrait>(self) -> ToQuant {
        self.deref().to_quantization_clamped()
    }

    /// Tries to convert to an index of another quantization, returns an error if it would break the bounds
    fn try_to_quantization<ToQuant: QuantizedIndexCountTrait>(self) -> Result<ToQuant, FeagiDataValueQuantizationError> {
        self.deref().try_to_quantization()
    }

    /// Clamps the value of this index for another quantization, but does not actually change the
    /// quantization itself
    fn clamp_for_quantization<ClampFor: QuantizedIndexCountTrait>(self) -> Self {
        Self::new(self.deref().clamp_for_quantization::<ClampFor>())
    }

    /// Clamps the value of this index for a runtime-provided quantization level, but does not
    /// actually change the quantization itself
    fn clamp_for_quantization_level_runtime(self, level: IndexCountQuantizationLevel) -> Self {
        Self::new(self.deref().clamp_for_quantization_level_runtime(level))
    }
}

/// Shared behaviour implemented by every wrapped enum generated by
/// [`create_wrapped_quantized_index`].
///
/// These enums hide the generic quantized wrapper type behind concrete variants
/// (`U8`, `U16`, `U32`, `U64`) while preserving the wrapper family semantics.
pub trait WrappedQuantizedIndexCountEnum:
    Copy
    + Clone
    + Send
    + Sync
    + core::fmt::Debug
    + core::cmp::PartialEq
    + core::cmp::Eq
    + core::hash::Hash
    + Sized
    + 'static
{
    fn get_level(&self) -> IndexCountQuantizationLevel;

    fn try_into_quant<Quant: QuantizedIndexCountTrait>(self) -> Result<Quant, FeagiDataValueQuantizationError>;

    fn into_quant<Quant: QuantizedIndexCountTrait>(self) -> Quant;

    fn to_usize(self) -> usize;
}

/// Creates a wrapper for quantized indexes / counts
#[macro_export]
macro_rules! create_wrapped_quantized_index {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedIndexCountTrait>(Q);

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> $struct_name<Q> {
            pub const LEVEL: $crate::values::quantizable::IndexCountQuantizationLevel = Q::LEVEL;
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

        // The bulk of the wrapper's behaviour lives on the shared
        // `WrappedQuantizedIndexCount` trait so that functions can generically accept any
        // wrapped index / count. See its definition for the available methods.
        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait>
            $crate::values::quantizable::WrappedQuantizedIndexCount for $struct_name<Q>
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

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> From<Q> for $struct_name<Q> {
            fn from(value: Q) -> Self {
                Self(value)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> From<&Q> for &$struct_name<Q> {
            fn from(value: &Q) -> Self {
                // tRust me bro
                unsafe { &*(value as *const Q as *const $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> From<&mut Q> for &mut $struct_name<Q> {
            fn from(value: &mut Q) -> Self {
                // tRust me bro
                unsafe { &mut *(value as *mut Q as *mut $struct_name<Q>) }
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> AsRef<Q> for $struct_name<Q> {
            fn as_ref(&self) -> &Q {
                &self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> AsMut<Q> for $struct_name<Q> {
            fn as_mut(&mut self) -> &mut Q {
                &mut self.0
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> core::ops::Add for $struct_name<Q> {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> core::ops::Sub for $struct_name<Q> {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> core::ops::Mul for $struct_name<Q> {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> core::ops::Div for $struct_name<Q> {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> core::ops::Rem for $struct_name<Q> {
            type Output = Self;
            fn rem(self, rhs: Self) -> Self::Output {
                Self(self.0 % rhs.0)
            }
        }
        


        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> core::ops::AddAssign for $struct_name<Q> {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> core::ops::SubAssign for $struct_name<Q> {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> core::ops::MulAssign for $struct_name<Q> {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> core::ops::DivAssign for $struct_name<Q> {
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> core::ops::RemAssign for $struct_name<Q> {
            fn rem_assign(&mut self, rhs: Self) {
                self.0 %= rhs.0;
            }
        }
        
                impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> Default for $struct_name<Q> {
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
                pub fn new_from_quantized<FromQuant: $crate::values::quantizable::QuantizedIndexCountTrait>(
                    value: $struct_name<FromQuant>
                ) -> Self {
                    match FromQuant::LEVEL {
                        $crate::values::quantizable::IndexCountQuantizationLevel::U8 => {
                            Self::U8($struct_name::<u8>::new(<u8 as $crate::values::quantizable::QuantizedIndexCountTrait>::from_quantization(value.deref())))
                        }
                        $crate::values::quantizable::IndexCountQuantizationLevel::U16 => {
                            Self::U16($struct_name::<u16>::new(<u16 as $crate::values::quantizable::QuantizedIndexCountTrait>::from_quantization(value.deref())))
                        }
                        $crate::values::quantizable::IndexCountQuantizationLevel::U32 => {
                            Self::U32($struct_name::<u32>::new(<u32 as $crate::values::quantizable::QuantizedIndexCountTrait>::from_quantization(value.deref())))
                        }
                        $crate::values::quantizable::IndexCountQuantizationLevel::U64 => {
                            Self::U64($struct_name::<u64>::new(<u64 as $crate::values::quantizable::QuantizedIndexCountTrait>::from_quantization(value.deref())))
                        }
                        $crate::values::quantizable::IndexCountQuantizationLevel::Usize => {
                            Self::U64($struct_name::<u64>::new(<u64 as $crate::values::quantizable::QuantizedIndexCountTrait>::from_quantization(value.deref())))
                        }
                    }
                }

                pub fn from_index_count_enum(value: $crate::values::quantizable::IndexCountEnum) -> Self {
                    match value {
                        $crate::values::quantizable::IndexCountEnum::U8(v) => {
                            Self::U8($struct_name::<u8>::new(v))
                        }
                        $crate::values::quantizable::IndexCountEnum::U16(v) => {
                            Self::U16($struct_name::<u16>::new(v))
                        }
                        $crate::values::quantizable::IndexCountEnum::U32(v) => {
                            Self::U32($struct_name::<u32>::new(v))
                        }
                        $crate::values::quantizable::IndexCountEnum::U64(v) => {
                            Self::U64($struct_name::<u64>::new(v))
                        }
                    }
                }

                pub fn into_index_count_enum(self) -> $crate::values::quantizable::IndexCountEnum {
                    match self {
                        Self::U8(v) => $crate::values::quantizable::IndexCountEnum::U8(v.deref()),
                        Self::U16(v) => $crate::values::quantizable::IndexCountEnum::U16(v.deref()),
                        Self::U32(v) => $crate::values::quantizable::IndexCountEnum::U32(v.deref()),
                        Self::U64(v) => $crate::values::quantizable::IndexCountEnum::U64(v.deref()),
                    }
                }

                pub fn try_into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedIndexCountTrait>(
                    self
                ) -> Result<$struct_name<Quant>, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    Ok($struct_name::<Quant>::new(
                        <Self as $crate::values::quantizable::WrappedQuantizedIndexCountEnum>::try_into_quant::<Quant>(self)?
                    ))
                }

                pub fn into_wrapped_quant<Quant: $crate::values::quantizable::QuantizedIndexCountTrait>(
                    self
                ) -> $struct_name<Quant> {
                    $struct_name::<Quant>::new(
                        <Self as $crate::values::quantizable::WrappedQuantizedIndexCountEnum>::into_quant::<Quant>(self)
                    )
                }
            }

            impl $crate::values::quantizable::WrappedQuantizedIndexCountEnum for [<$struct_name Enum>] {
                fn get_level(&self) -> $crate::values::quantizable::IndexCountQuantizationLevel {
                    match self {
                        Self::U8(_) => $crate::values::quantizable::IndexCountQuantizationLevel::U8,
                        Self::U16(_) => $crate::values::quantizable::IndexCountQuantizationLevel::U16,
                        Self::U32(_) => $crate::values::quantizable::IndexCountQuantizationLevel::U32,
                        Self::U64(_) => $crate::values::quantizable::IndexCountQuantizationLevel::U64,
                    }
                }

                fn try_into_quant<Quant: $crate::values::quantizable::QuantizedIndexCountTrait>(
                    self
                ) -> Result<Quant, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    match self {
                        Self::U8(value) => Quant::try_from_quantization(value.deref()),
                        Self::U16(value) => Quant::try_from_quantization(value.deref()),
                        Self::U32(value) => Quant::try_from_quantization(value.deref()),
                        Self::U64(value) => Quant::try_from_quantization(value.deref()),
                    }
                }

                fn into_quant<Quant: $crate::values::quantizable::QuantizedIndexCountTrait>(self) -> Quant {
                    match self {
                        Self::U8(value) => Quant::from_quantization(value.deref()),
                        Self::U16(value) => Quant::from_quantization(value.deref()),
                        Self::U32(value) => Quant::from_quantization(value.deref()),
                        Self::U64(value) => Quant::from_quantization(value.deref()),
                    }
                }

                fn to_usize(self) -> usize {
                    match self {
                        Self::U8(value) => value.deref() as usize,
                        Self::U16(value) => value.deref() as usize,
                        Self::U32(value) => value.deref() as usize,
                        Self::U64(value) => value.deref() as usize,
                    }
                }
            }
        }

    };
}

