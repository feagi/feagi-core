use crate::values::quantizable::{QuantizationLevelPacking, QuantizedElementBase};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum SignedIntegerQuantizationLevel {
    I8 = 0,
    I16 = 1,
    I32 = 2,
    I64 = 3,
    I128 = 4,
    Isize = 5,
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
            4 => Ok(SignedIntegerQuantizationLevel::I128),
            5 => Ok(SignedIntegerQuantizationLevel::Isize),
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
pub trait QuantizedSignedIntegerTrait: QuantizedElementBase {
    fn is_negative(&self) -> bool;
    fn is_zero_or_negative(&self) -> bool;
}

impl QuantizedSignedIntegerTrait for isize {
    #[inline(always)]
    fn is_negative(&self) -> bool {
        *self < 0
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self <= 0
    }
}

impl QuantizedSignedIntegerTrait for i8 {
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
    #[inline(always)]
    fn is_negative(&self) -> bool {
        *self < 0
    }

    #[inline(always)]
    fn is_zero_or_negative(&self) -> bool {
        *self <= 0
    }
}
