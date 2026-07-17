use serde_json::Value;
use crate::values::quantizable::feagi_data_value_quantization_error::FeagiFailQuantizationOutOfRange;
use crate::values::quantizable::{FeagiDataValueQuantizationError, QuantizationLevelPacking, QuantizedElementBase};

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

    unsafe fn from_packed_byte(byte: u8) -> Self {
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

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u8::MAX as usize {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u8!", value as usize).into());
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

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u16::MAX as usize {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u16!", value as usize).into());
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

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u32::MAX as usize {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u32!", value as usize).into());
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

            /// Tries to convert from usize, does NOT check bounds!
            pub fn quant_from_usize(value: usize) -> Self {
                Self(Q::quant_from_usize(value))
            }

            /// Tries converting from usize, returns an error if out of bounds
            pub fn quant_try_from_usize(value: usize) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                Ok(Self(Q::quant_try_from_usize(value)?))
            }

            /// Converts to usize. No need to check as we have no indexes that will exceed a usize on a
            /// system
            pub fn quant_to_usize(self) -> usize {
                self.0.quant_to_usize()
            }

            /// Tries to convert from u32, does NOT check bounds!
            pub fn quant_to_u8(self) -> u8 {
                self.0.quant_to_u8()
            }

            /// Tries to convert to u16, does NOT check bounds!
            pub fn quant_to_u16(self) -> u16 {
                self.0.quant_to_u16()
            }

            /// Tries to convert to u32, does NOT check bounds!
            pub fn quant_to_u32(self) -> u32 {
                self.0.quant_to_u32()
            }

            /// Tries to convert to u64, bound checking shouldnt matter since this is the biggest type (not doing u128 lol)
            pub fn quant_to_u64(self) -> u64 {
                self.0.quant_to_u64()
            }

            /// Creates from an index of another quantization. Does not check for validity of ranges!
            pub fn from_quantization<FromQuant: $crate::values::quantizable::QuantizedIndexCountTrait>(value: FromQuant) -> Self {
                Self(Q::from_quantization(value))
            }

            /// Creates from an index of another quantization, clamping its values to ensure it fits
            pub fn from_quantization_clamped<FromQuant: $crate::values::quantizable::QuantizedIndexCountTrait>(value: FromQuant) -> Self {
                Self(Q::from_quantization_clamped(value))
            }

            /// Tries to create an index of another quantization, returns an error if it would break the bounds
            pub fn try_from_quantization<FromQuant: $crate::values::quantizable::QuantizedIndexCountTrait>(value: FromQuant) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                Ok(Self(Q::try_from_quantization(value)?))
            }

            /// Converts to an index of another quantization. Does not check for validity of ranges!
            pub fn to_quantization<ToQuant: $crate::values::quantizable::QuantizedIndexCountTrait>(self) -> ToQuant {
                self.0.to_quantization()
            }

            /// Converts to an index of another quantization, clamping its values to ensure it fits
            pub fn to_quantization_clamped<ToQuant: $crate::values::quantizable::QuantizedIndexCountTrait>(self) -> ToQuant {
                self.0.to_quantization_clamped()
            }

            /// Tries to convert to an index of another quantization, returns an error if it would break the bounds
            pub fn try_to_quantization<ToQuant: $crate::values::quantizable::QuantizedIndexCountTrait>(self) -> Result<ToQuant, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                self.0.try_to_quantization()
            }

            /// Clamps the value of this index for another quantization, but does not actually change the
            /// quantization itself
            pub fn clamp_for_quantization<ClampFor: $crate::values::quantizable::QuantizedIndexCountTrait>(self) -> Self {
                Self(self.0.clamp_for_quantization::<ClampFor>())
            }

            pub fn clamp_for_quantization_level_runtime(self, level: $crate::values::quantizable::IndexCountQuantizationLevel) -> Self {
                Self(self.0.clamp_for_quantization_level_runtime(level))
            }

            /// Extracts the inner quantized index / count
            pub fn deref(self) -> Q {
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
                unsafe { &*(value as *const Q as *const Self) }
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

    };
}

