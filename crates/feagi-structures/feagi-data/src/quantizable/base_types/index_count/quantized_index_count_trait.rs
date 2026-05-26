use crate::core_numerical_types::{SupportsUintOps};
use crate::quantizable::base_types::quantized_base_traits::QuantizedElementBase;

/// Trait designed to hold index and count values in a quantized form
pub trait QuantizedIndexCountTrait: QuantizedElementBase
+ SupportsUintOps
{
    /// Converts to u32
    fn to_u32(self) -> u32;

    /// Tries to convert from u32, does NOT check bounds!
    fn from_u32(value: u32) -> Self;

    /// Tries to convert from u32, clamping if it goes out of range!
    fn from_u32_clamped(value: u32) -> Self;

    // TODO should be in wrapper
    // TODO add increment with rollover
    /*
    /// Calculates what the minimum quantization level is needed to hold the current value
    fn minimum_required_quantization_level(self) -> QuantizationLevel {
        QuantizationLevel::minimum_quantization_needed_for_usize(self.to_usize())
    }

    /// Returns true if the given usize needs to be clamped to fit in the current quantization
    fn should_clamp(self, value: usize) -> bool {
        value > Self::QUANT_MAX_AS_USIZE
    }
    
    fn is_zero(self) -> bool {
        self == Self::QUANT_ZERO
    }

     */
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

/// Something all wrappers share, for easy data access
pub trait QuantizedIndexCountWrapperTrait<QuantIndex: QuantizedIndexCountTrait>:
Copy
+ Clone
+ core::ops::Add<Output = Self>
+ core::ops::Sub<Output = Self>
+ core::ops::Mul<Output = Self>
+ core::ops::Div<Output = Self>
+ core::ops::AddAssign
+ core::ops::SubAssign
+ core::ops::MulAssign
+ core::ops::DivAssign
+ core::cmp::PartialOrd
+ Send
+ Sync
+ 'static
{
    fn wrap_quant(quant: QuantIndex) -> Self;
    fn quant(self) -> QuantIndex;
    fn quant_ref(&self) -> &QuantIndex;
    fn quant_mut(&mut self) -> &mut QuantIndex;
}