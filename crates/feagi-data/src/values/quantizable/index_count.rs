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
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::AddAssign
    + core::ops::SubAssign
    + core::ops::MulAssign
    + core::ops::DivAssign
    + core::cmp::PartialOrd
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
    const QUANT_MAX: Self;
    const QUANT_MAX_AS_USIZE: usize;
    const QUANT_MAX_U8: Self;
    const QUANT_MAX_U16: Self;
    const QUANT_MAX_U32: Self;
    const QUANT_MAX_U64: Self;

    // TODO to other quantizations

    /// Tries to convert from usize, does NOT check bounds!
    fn quant_from_usize(value: usize) -> Self;

    /// Converts to usize
    fn quant_to_usize(self) -> usize;

    /// Tries to convert from u8 (this can never fail, we have nothing smaller than u8)
    fn quant_from_u8(value: u8) -> Self;
    /// Tries to convert from u32, does NOT check bounds!
    fn quant_to_u8(self) -> u8;
    /// Tries to convert from u16, does NOT check bounds!
    fn quant_from_u16(value: u16) -> Self;
    /// Tries to convert to u16, does NOT check bounds!
    fn quant_to_u16(self) -> u16;
    /// Tries to convert from u32, does NOT check bounds!
    fn quant_from_u32(value: u32) -> Self;
    /// Tries to convert to u32, does NOT check bounds!
    fn quant_to_u32(self) -> u32;
    /// Tries to convert from u64, does NOT check bounds!
    fn quant_from_u64(value: u64) -> Self;
    /// Tries to convert to u64, bound checking shouldnt matter since this is the biggest type (not doing u128 lol)
    fn quant_to_u64(self) -> u64;

    /// Tries to convert from u8, clamping if it goes out of range!
    fn quant_from_u8_clamped(value: u8) -> Self;
    /// Tries to convert from u16, clamping if it goes out of range!
    fn quant_from_u16_clamped(value: u16) -> Self;
    /// Tries to convert from u32, clamping if it goes out of range!
    fn quant_from_u32_clamped(value: u32) -> Self;

    /// Just converts to u64, added for completeness but there is no situation where we need to
    /// clamp an u64 value
    fn quant_from_u64_clamped(value: u64) -> Self {
        Self::quant_from_u64(value)
    }
    
    /// Tries converting from usize, returns an error if out of bounds
    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError>;
    
    /// Tries converting from u8, returns an error if out of bounds
    fn quant_try_from_u8(value: u8) -> Result<Self, FeagiDataValueQuantizationError>;
    /// Tries converting to u8, returns an error if out of bounds
    fn quant_try_to_u8(self) -> Result<u8, FeagiDataValueQuantizationError>;
    /// Tries converting from u16, returns an error if out of bounds
    fn quant_try_from_u16(value: u16) -> Result<Self, FeagiDataValueQuantizationError>;
    /// Tries converting to u16, returns an error if out of bounds
    fn quant_try_to_u16(self) -> Result<u16, FeagiDataValueQuantizationError>;
    /// Tries converting from u32, returns an error if out of bounds
    fn quant_try_from_u32(value: u32) -> Result<Self, FeagiDataValueQuantizationError>;
    /// Tries converting to u32, returns an error if out of bounds
    fn quant_try_to_u32(self) -> Result<u32, FeagiDataValueQuantizationError>;
    /// Tries converting from u64, returns an error if out of bounds
    fn quant_try_from_u64(value: u64) -> Result<Self, FeagiDataValueQuantizationError>;
    /// Tries converting to u64, returns an error if out of bounds
    fn quant_try_to_u64(self) -> Result<u64, FeagiDataValueQuantizationError>;
}

impl QuantizedIndexCountTrait for u8 {
    const QUANT_MAX: Self = u8::MAX;
    const QUANT_MAX_AS_USIZE: usize = u8::MAX as usize;
    const QUANT_MAX_U8: Self = u8::MAX;
    const QUANT_MAX_U16: Self = u8::MAX;
    const QUANT_MAX_U32: Self = u8::MAX;
    const QUANT_MAX_U64: Self = u8::MAX;

    fn quant_to_usize(self) -> usize {
        self as usize
    }

    fn quant_from_usize(value: usize) -> Self {
        value as u8
    }

    fn quant_from_u8(value: u8) -> Self {
        value
    }

    fn quant_to_u8(self) -> u8 {
        self
    }

    fn quant_from_u16(value: u16) -> Self {
        value as u8
    }

    fn quant_to_u16(self) -> u16 {
        self as u16
    }

    fn quant_from_u32(value: u32) -> Self {
        value as u8
    }

    fn quant_to_u32(self) -> u32 {
        self as u32
    }

    fn quant_from_u64(value: u64) -> Self {
        value as u8
    }

    fn quant_to_u64(self) -> u64 {
        self as u64
    }

    fn quant_from_u8_clamped(value: u8) -> Self {
        value
    }

    fn quant_from_u16_clamped(value: u16) -> Self {
        value.min(u8::MAX as u16) as u8
    }

    fn quant_from_u32_clamped(value: u32) -> Self {
        value.min(u8::MAX as u32) as u8
    }

    fn quant_try_from_u8(value: u8) -> Result<Self, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(value)
    }

    fn quant_try_to_u8(self) -> Result<u8, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(self)
    }

    fn quant_try_from_u16(value: u16) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u8::MAX as u16 {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u8!", value as usize).into());
        }
        Ok(value as u8)
    }

    fn quant_try_to_u16(self) -> Result<u16, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(self as u16)
    }

    fn quant_try_from_u32(value: u32) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u8::MAX as u32 {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u8!", value as usize).into());
        }
        Ok(value as u8)
    }

    fn quant_try_to_u32(self) -> Result<u32, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(self as u32)
    }

    fn quant_try_from_u64(value: u64) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u8::MAX as u64 {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u8!", value as usize).into());
        }
        Ok(value as u8)
    }

    fn quant_try_to_u64(self) -> Result<u64, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(self as u64)
    }

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u8::MAX as usize {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u8!", value as usize).into());
        }
        Ok(value as u8)
    }
}

impl QuantizedIndexCountTrait for u16 {
    const QUANT_MAX: Self = u16::MAX;
    const QUANT_MAX_AS_USIZE: usize = u16::MAX as usize;
    const QUANT_MAX_U8: Self = u8::MAX as u16;
    const QUANT_MAX_U16: Self = u16::MAX;
    const QUANT_MAX_U32: Self = u16::MAX;
    const QUANT_MAX_U64: Self = u16::MAX;

    fn quant_to_usize(self) -> usize {
        self as usize
    }

    fn quant_from_usize(value: usize) -> Self {
        value as u16
    }

    fn quant_from_u8(value: u8) -> Self {
        value as u16
    }

    fn quant_to_u8(self) -> u8 {
        self as u8
    }

    fn quant_from_u16(value: u16) -> Self {
        value
    }

    fn quant_to_u16(self) -> u16 {
        self
    }

    fn quant_from_u32(value: u32) -> Self {
        value as u16
    }

    fn quant_to_u32(self) -> u32 {
        self as u32
    }

    fn quant_from_u64(value: u64) -> Self {
        value as u16
    }

    fn quant_to_u64(self) -> u64 {
        self as u64
    }

    fn quant_from_u8_clamped(value: u8) -> Self {
        value as u16
    }

    fn quant_from_u16_clamped(value: u16) -> Self {
        value
    }

    fn quant_from_u32_clamped(value: u32) -> Self {
        value.min(u16::MAX as u32) as u16
    }

    fn quant_try_from_u8(value: u8) -> Result<Self, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(value as u16)
    }

    fn quant_try_to_u8(self) -> Result<u8, FeagiDataValueQuantizationError> {
        if self > u8::MAX as u16 {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u8!", self as usize).into());
        }
        Ok(self as u8)
    }

    fn quant_try_from_u16(value: u16) -> Result<Self, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(value)
    }

    fn quant_try_to_u16(self) -> Result<u16, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(self)
    }

    fn quant_try_from_u32(value: u32) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u16::MAX as u32 {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u16!", value as usize).into());
        }
        Ok(value as u16)
    }

    fn quant_try_to_u32(self) -> Result<u32, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(self as u32)
    }

    fn quant_try_from_u64(value: u64) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u16::MAX as u64 {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u16!", value as usize).into());
        }
        Ok(value as u16)
    }

    fn quant_try_to_u64(self) -> Result<u64, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(self as u64)
    }

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u16::MAX as usize {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u16!", value as usize).into());
        }
        Ok(value as u16)
    }
}

// lol, lmao even
impl QuantizedIndexCountTrait for u32 {
    const QUANT_MAX: Self = u32::MAX;
    const QUANT_MAX_AS_USIZE: usize = u32::MAX as usize;
    const QUANT_MAX_U8: Self = u8::MAX as u32;
    const QUANT_MAX_U16: Self = u16::MAX as u32;
    const QUANT_MAX_U32: Self = u32::MAX;
    const QUANT_MAX_U64: Self = u32::MAX;

    fn quant_to_usize(self) -> usize {
        self as usize
    }

    fn quant_from_usize(value: usize) -> Self {
        value as u32
    }

    fn quant_from_u8(value: u8) -> Self {
        value as u32
    }

    fn quant_to_u8(self) -> u8 {
        self as u8
    }

    fn quant_from_u16(value: u16) -> Self {
        value as u32
    }

    fn quant_to_u16(self) -> u16 {
        self as u16
    }

    fn quant_from_u32(value: u32) -> Self {
        value
    }

    fn quant_to_u32(self) -> u32 {
        self
    }

    fn quant_from_u64(value: u64) -> Self {
        value as u32
    }

    fn quant_to_u64(self) -> u64 {
        self as u64
    }

    fn quant_from_u8_clamped(value: u8) -> Self {
        value as u32
    }

    fn quant_from_u16_clamped(value: u16) -> Self {
        value as u32
    }

    fn quant_from_u32_clamped(value: u32) -> Self {
        value
    }

    fn quant_try_from_u8(value: u8) -> Result<Self, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(value as u32)
    }

    fn quant_try_to_u8(self) -> Result<u8, FeagiDataValueQuantizationError> {
        if self > u8::MAX as u32 {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u8!", self as usize).into());
        }
        Ok(self as u8)
    }

    fn quant_try_from_u16(value: u16) -> Result<Self, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(value as u32)
    }

    fn quant_try_to_u16(self) -> Result<u16, FeagiDataValueQuantizationError> {
        if self > u16::MAX as u32 {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u16!", self as usize).into());
        }
        Ok(self as u16)
    }

    fn quant_try_from_u32(value: u32) -> Result<Self, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(value)
    }

    fn quant_try_to_u32(self) -> Result<u32, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(self)
    }

    fn quant_try_from_u64(value: u64) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u32::MAX as u64 {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u32!", value as usize).into());
        }
        Ok(value as u32)
    }

    fn quant_try_to_u64(self) -> Result<u64, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(self as u64)
    }

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        if value > u32::MAX as usize {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u32!", value as usize).into());
        }
        Ok(value as u32)
    }
}

impl QuantizedIndexCountTrait for u64 {
    const QUANT_MAX: Self = u64::MAX;
    const QUANT_MAX_AS_USIZE: usize = u64::MAX as usize;
    const QUANT_MAX_U8: Self = u8::MAX as u64;
    const QUANT_MAX_U16: Self = u16::MAX as u64;
    const QUANT_MAX_U32: Self = u32::MAX as u64;
    const QUANT_MAX_U64: Self = u64::MAX;

    fn quant_to_usize(self) -> usize {
        self as usize
    }

    fn quant_from_usize(value: usize) -> Self {
        value as u64
    }

    fn quant_from_u8(value: u8) -> Self {
        value as u64
    }

    fn quant_to_u8(self) -> u8 {
        self as u8
    }

    fn quant_from_u16(value: u16) -> Self {
        value as u64
    }

    fn quant_to_u16(self) -> u16 {
        self as u16
    }

    fn quant_from_u32(value: u32) -> Self {
        value as u64
    }

    fn quant_to_u32(self) -> u32 {
        self as u32
    }

    fn quant_from_u64(value: u64) -> Self {
        value
    }

    fn quant_to_u64(self) -> u64 {
        self
    }

    fn quant_from_u8_clamped(value: u8) -> Self {
        value as u64
    }

    fn quant_from_u16_clamped(value: u16) -> Self {
        value as u64
    }

    fn quant_from_u32_clamped(value: u32) -> Self {
        value as u64
    }

    fn quant_try_from_u8(value: u8) -> Result<Self, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(value as u64)
    }

    fn quant_try_to_u8(self) -> Result<u8, FeagiDataValueQuantizationError> {
        if self > u8::MAX as u64 {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u8!", self as usize).into());
        }
        Ok(self as u8)
    }

    fn quant_try_from_u16(value: u16) -> Result<Self, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(value as u64)
    }

    fn quant_try_to_u16(self) -> Result<u16, FeagiDataValueQuantizationError> {
        if self > u16::MAX as u64 {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u16!", self as usize).into());
        }
        Ok(self as u16)
    }

    fn quant_try_from_u32(value: u32) -> Result<Self, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(value as u64)
    }

    fn quant_try_to_u32(self) -> Result<u32, FeagiDataValueQuantizationError> {
        if self > u32::MAX as u64 {
            return Err(FeagiFailQuantizationOutOfRange::new("Given index value cannot fit in a quantized u32!", self as usize).into());
        }
        Ok(self as u32)
    }

    fn quant_try_from_u64(value: u64) -> Result<Self, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(value)
    }

    fn quant_try_to_u64(self) -> Result<u64, FeagiDataValueQuantizationError> {
        // This will never fail
        Ok(self)
    }

    fn quant_try_from_usize(value: usize) -> Result<Self, FeagiDataValueQuantizationError> {
        // never fails
        Ok(value as u64)
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
            pub const QUANT_ZERO: Self = Self::const_new(Q::QUANT_ZERO);
            pub const QUANT_ONE: Self = Self::const_new(Q::QUANT_ONE);

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

            pub fn from_usize(value: usize) -> Self {
                Self(Q::quant_from_usize(value))
            }

            pub fn to_usize(self) -> usize {
                self.0.quant_to_usize()
            }

            pub fn from_u8(value: u8) -> Self {
                Self(Q::quant_from_u8(value))
            }

            pub fn to_u8(self) -> u8 {
                self.0.quant_to_u8()
            }

            pub fn from_u16(value: u16) -> Self {
                Self(Q::quant_from_u16(value))
            }

            pub fn to_u16(self) -> u16 {
                self.0.quant_to_u16()
            }

            pub fn from_u32(value: u32) -> Self {
                Self(Q::quant_from_u32(value))
            }

            pub fn to_u32(self) -> u32 {
                self.0.quant_to_u32()
            }

            pub fn from_u64(value: u64) -> Self {
                Self(Q::quant_from_u64(value))
            }

            pub fn to_u64(self) -> u64 {
                self.0.quant_to_u64()
            }

            pub fn from_u8_clamped(value: u8) -> Self {
                Self(Q::quant_from_u8_clamped(value))
            }

            pub fn from_u16_clamped(value: u16) -> Self {
                Self(Q::quant_from_u16_clamped(value))
            }

            pub fn from_u32_clamped(value: u32) -> Self {
                Self(Q::quant_from_u32_clamped(value))
            }

            pub fn from_u64_clamped(value: u64) -> Self {
                Self(Q::quant_from_u64_clamped(value))
            }

            pub fn try_from_u8(value: u8) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                Ok(Self(Q::quant_try_from_u8(value)?))
            }

            pub fn try_to_u8(self) -> Result<u8, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                self.0.quant_try_to_u8()
            }

            pub fn try_from_u16(value: u16) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                Ok(Self(Q::quant_try_from_u16(value)?))
            }

            pub fn try_to_u16(self) -> Result<u16, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                self.0.quant_try_to_u16()
            }

            pub fn try_from_u32(value: u32) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                Ok(Self(Q::quant_try_from_u32(value)?))
            }

            pub fn try_to_u32(self) -> Result<u32, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                self.0.quant_try_to_u32()
            }

            pub fn try_from_u64(value: u64) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                Ok(Self(Q::quant_try_from_u64(value)?))
            }

            pub fn try_to_u64(self) -> Result<u64, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                self.0.quant_try_to_u64()
            }
            
            pub fn try_from_usize(value: usize) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                 Ok(Self(Q::quant_try_from_usize(value)?))
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
