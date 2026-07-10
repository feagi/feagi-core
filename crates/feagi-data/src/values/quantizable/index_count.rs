use crate::values::quantizable::QuantizedElementBase;


// TODO serde serialization / deserialization? is it a good idea?

// TODO have from/to other quantization levels besides u32
// TODO have try_from support, with error reporting the invalid number as a 3 char string (const)

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

    /// Converts to usize
    fn to_usize(self) -> usize;

    /// Converts to u32
    fn to_u32(self) -> u32;

    /// Tries to convert from usize, does NOT check bounds!
    fn from_usize(value: usize) -> Self;

    /// Tries to convert from u32, does NOT check bounds!
    fn from_u32(value: u32) -> Self;

    /// Tries to convert from u32, clamping if it goes out of range!
    fn from_u32_clamped(value: u32) -> Self;

    
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

    fn from_usize(value: usize) -> Self {value as u8}

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
    const QUANT_MAX: Self = u16::MAX;
    const QUANT_MAX_AS_USIZE: usize = u16::MAX as usize;
    const QUANT_ONE: Self = 1;

    fn to_usize(self) -> usize {
        self as usize
    }
    
    fn to_u32(self) -> u32 {
        self as u32
    }

    fn from_usize(value: usize) -> Self {value as u16}

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
    const QUANT_MAX: Self = u32::MAX;
    const QUANT_MAX_AS_USIZE: usize = u32::MAX as usize;
    const QUANT_ONE: Self = 1;

    fn to_usize(self) -> usize {
        self as usize
    }
    
    fn to_u32(self) -> u32 {
        self
    }

    fn from_usize(value: usize) -> Self {value as u32}

    fn from_u32(value: u32) -> Self {
        value
    }

    fn from_u32_clamped(value: u32) -> Self {
        value
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

    fn from_usize(value: usize) -> Self {value as u64}

    fn from_u32(value: u32) -> Self {
        value as u64
    }

    fn from_u32_clamped(value: u32) -> Self {
        value as u64 // no way it can escape the clamp
    }
}

// Note: Specifically we will not support usize directly since it can vary in size depending on
// backend, which could cause some issues

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

            pub fn from_usize_unchecked(u: usize) -> Self {
                Self(Q::from_usize(u))
            }

            /// Bounds-checked conversion from usiz
            pub fn try_from_usize(u: usize) -> Result<Self, $crate::values::feagi_data_value_error::FeagiValueError> {
                if u > Q::QUANT_MAX_AS_USIZE {
                    return Err($crate::values::feagi_data_value_error::FeagiInvalidQuantizationErrKey::new("Given usize exceeds current quantization bounds!").into())
                }
                Ok(Self(Q::from_usize(u)))
            }
            
            pub fn to_usize(&self) -> usize {
                self.0.to_usize()
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