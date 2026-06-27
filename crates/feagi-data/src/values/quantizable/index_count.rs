use std::ops::Index;
use crate::values::quantizable::shared_traits::{QuantizedElementBase, SupportsUintOps};

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

    /// Minimum number of bytes needed to hold this number of bits
    fn number_bits_to_number_bytes(number_bits: Self) -> Self;

    fn start_length_to_usize_range(start: Self, length: Self) -> core::ops::Range<usize> { // TODO delete me!
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

    fn number_bits_to_number_bytes(number_bits: Self) -> Self {
        if number_bits % 8 != 0 {
            1 + number_bits / 8
        } else { 
            number_bits / 8 
        }
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

    fn number_bits_to_number_bytes(number_bits: Self) -> Self {
        if number_bits % 8 != 0 {
            1 + number_bits / 8
        } else {
            number_bits / 8
        }
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

    fn number_bits_to_number_bytes(number_bits: Self) -> Self {
        if number_bits % 8 != 0 {
            1 + number_bits / 8
        } else {
            number_bits / 8
        }
    }
}

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

    fn number_bits_to_number_bytes(number_bits: Self) -> Self {
        if number_bits % 8 != 0 {
            1 + number_bits / 8
        } else {
            number_bits / 8
        }
    }
}



// TODO serde serialization / deserialization? is it a good idea?


#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WrappedIndexCountExample<Q: QuantizedIndexCountTrait>(Q);

impl<Q: QuantizedIndexCountTrait> WrappedIndexCountExample<Q> {
}

// NOTE: Into<Q> for WrappedIndexCountExample<Q> is not needed!

impl<Q: QuantizedIndexCountTrait> From<Q> for WrappedIndexCountExample<Q> {
    fn from(value: Q) -> Self {
        Self(value)
    }
}

impl<Q: QuantizedIndexCountTrait> From<&Q> for &WrappedIndexCountExample<Q> {
    fn from(value: &Q) -> Self {
        // tRust me bro
        unsafe { &*(value as *const Q as *const Self) }
    }
}

impl<Q: QuantizedIndexCountTrait> AsRef<Q> for WrappedIndexCountExample<Q> {
    fn as_ref(&self) -> &Q {
        &self.0
    }
}

impl<Q: QuantizedIndexCountTrait> AsMut<Q> for WrappedIndexCountExample<Q> {
    fn as_mut(&mut self) -> &mut Q {
        &mut self.0
    }
}

impl<Q: QuantizedIndexCountTrait> core::ops::Add for WrappedIndexCountExample<Q> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<Q: QuantizedIndexCountTrait> core::ops::Sub for WrappedIndexCountExample<Q> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<Q: QuantizedIndexCountTrait> core::ops::Mul for WrappedIndexCountExample<Q> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl<Q: QuantizedIndexCountTrait> core::ops::Div for WrappedIndexCountExample<Q> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl<Q: QuantizedIndexCountTrait> core::ops::Rem for WrappedIndexCountExample<Q> {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl<Q: QuantizedIndexCountTrait> core::ops::AddAssign for WrappedIndexCountExample<Q> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<Q: QuantizedIndexCountTrait> core::ops::SubAssign for WrappedIndexCountExample<Q> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl<Q: QuantizedIndexCountTrait> core::ops::MulAssign for WrappedIndexCountExample<Q> {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl<Q: QuantizedIndexCountTrait> core::ops::DivAssign for WrappedIndexCountExample<Q> {
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}

impl<Q: QuantizedIndexCountTrait> core::ops::RemAssign for WrappedIndexCountExample<Q> {
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0;
    }
}

impl<Q: QuantizedIndexCountTrait, A> Index<WrappedIndexCountExample<Q>> for Vec<A> {
    type Output = A;

    fn index(&self, index: WrappedIndexCountExample<Q>) -> &Self::Output {
        let u = index.0.to_usize();
        &self[u]
    }
}


