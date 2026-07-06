use crate::values::quantizable::QuantizedElementBase;


// TODO serde serialization / deserialization? is it a good idea?

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

    const QUANT_ONE: Self;
    
    // TODO to other quantizations

    fn to_usize(self) -> usize;

    /// Converts to u32
    fn to_u32(self) -> u32;

    /// Tries to convert from u32, does NOT check bounds!
    fn from_u32(value: u32) -> Self;

    /// Tries to convert from u32, clamping if it goes out of range!
    fn from_u32_clamped(value: u32) -> Self;

    /// Minimum number of bytes needed to hold this number of bits // TODO why is this here?
    fn number_bits_to_number_bytes(number_bits: Self) -> Self;
}

impl QuantizedIndexCountTrait for u8 {
    const QUANT_MAX: Self = u8::MAX;
    const QUANT_MAX_AS_USIZE: usize = u8::MAX as usize;
    const QUANT_ONE: Self = 1;

    fn to_usize(self) -> usize {
        self as usize
    }
    
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
    const QUANT_MAX: Self = u16::MAX;
    const QUANT_MAX_AS_USIZE: usize = u16::MAX as usize;
    const QUANT_ONE: Self = 1;

    fn to_usize(self) -> usize {
        self as usize
    }
    
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
    const QUANT_MAX: Self = u32::MAX;
    const QUANT_MAX_AS_USIZE: usize = u32::MAX as usize;
    const QUANT_ONE: Self = 1;

    fn to_usize(self) -> usize {
        self as usize
    }
    
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
    const QUANT_MAX: Self = u64::MAX;
    const QUANT_MAX_AS_USIZE: usize = u64::MAX as usize;
    const QUANT_ONE: Self = 1;

    fn to_usize(self) -> usize {
        self as usize
    }
    
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
            pub const fn const_new(value: Q) -> Self
            {
                Self(value)
            }

            pub const fn const_deref(self) -> Q
            {
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