// Note this right now is very similar to IndexCount, but this will differ with time

use crate::values::quantizable::{QuantizationLevelPacking, QuantizedElementBase};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum UnsignedIntegerQuantizationLevel {
    U8 = 0,
    U16 = 1,
    U32 = 2,
    U64 = 3,
    U128 = 4,
    Usize = 5,
    // We can support a max of 8
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
            4 => Ok(UnsignedIntegerQuantizationLevel::U128),
            5 => Ok(UnsignedIntegerQuantizationLevel::Usize),
            _ => Err(()),
        }
    }
}

impl QuantizationLevelPacking for UnsignedIntegerQuantizationLevel {
    const NUMBER_BITS: usize = 3;

    unsafe fn from_packed_byte(byte: u8) -> Self {
        core::mem::transmute(byte)
    }
}

/// Trait designed to hold uint data values in a quantized form
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
    + core::cmp::PartialOrd
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
}

impl QuantizedUnsignedIntegerTrait for usize {}

impl QuantizedUnsignedIntegerTrait for u8 {}

impl QuantizedUnsignedIntegerTrait for u16 {}

// lol, lmao even
impl QuantizedUnsignedIntegerTrait for u32 {}

impl QuantizedUnsignedIntegerTrait for u64 {}

// TODO uint wrapper!
