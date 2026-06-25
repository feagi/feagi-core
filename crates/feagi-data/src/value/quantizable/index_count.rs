use crate::shared_traits::{QuantizedElementBase, SupportsUintOps};

/// Trait designed to hold index and/or count values in a quantized form
pub trait QuantizedIndexCountTrait: QuantizedElementBase
+ SupportsUintOps
{
    /// Converts to u32
    fn to_u32(self) -> u32;

    /// Tries to convert from u32, does NOT check bounds!
    fn from_u32(value: u32) -> Self;

    /// Tries to convert from u32, clamping if it goes out of range!
    fn from_u32_clamped(value: u32) -> Self;

    fn start_length_to_usize_range(start: Self, length: Self) -> core::ops::Range<usize> {
        start.to_usize().. length.to_usize()
    }
}

impl QuantizedIndexCountTrait for u8 {
    fn to_u32(self) -> u32 {
        self as u32
    }

    fn from_u32(value: u32) -> Self {
        value as u8
    }

    fn from_u32_clamped(value: u32) -> Self {
        const MAX_AS_U32: u32 = u8::MAX as u32;
        if value > MAX_AS_U32 {
            return u8::MAX;
        }
        value as u8
    }
}

impl QuantizedIndexCountTrait for u16 {
    fn to_u32(self) -> u32 {
        self as u32
    }

    fn from_u32(value: u32) -> Self {
        value as u16
    }

    fn from_u32_clamped(value: u32) -> Self {
        const MAX_AS_U32: u32 = u16::MAX as u32;
        if value > MAX_AS_U32 {
            return u16::MAX;
        }
        value as u16
    }
}

// lol, lmao even
impl QuantizedIndexCountTrait for u32 {
    fn to_u32(self) -> u32 {
        self
    }

    fn from_u32(value: u32) -> Self {
        value
    }

    fn from_u32_clamped(value: u32) -> Self {
        value
    }
}

#[cfg(feature = "support_64bit_indexing")]
impl QuantizedIndexCountTrait for u64 {
    fn to_u32(self) -> u32 {
        self as u32
    }

    fn from_u32(value: u32) -> Self {
        value as u64
    }

    fn from_u32_clamped(value: u32) -> Self {
        value as u64 // no way it can escape the clamp
    }
}