use core::fmt::{Debug, Display};

/// Defines all implementations of QuantizableUInt and its dependent traits.
#[macro_export]
macro_rules! impl_quantizable_uint_wrapper {
    ($wrapper:ident) => {
        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> From<T> for $wrapper<T> {
            #[inline(always)]
            fn from(value: T) -> Self {
                Self(value)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> Default for $wrapper<T> {
            #[inline(always)]
            fn default() -> Self {
                Self(T::default())
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> From<$wrapper<T>> for usize {
            #[inline(always)]
            fn from(value: $wrapper<T>) -> Self {
                value.0.to_usize()
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> core::ops::Add for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> core::ops::Sub for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> core::ops::Mul for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> core::ops::Div for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> core::ops::AddAssign for $wrapper<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> core::ops::SubAssign for $wrapper<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> core::ops::MulAssign for $wrapper<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> core::ops::DivAssign for $wrapper<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt + core::fmt::Display> core::fmt::Display for $wrapper<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> $crate::base_quantizable::unsigned_integer::QuantizableUInt for $wrapper<T> {
            const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES;
            const ZERO: Self = Self(T::ZERO);
            const ONE: Self = Self(T::ONE);
            const MAX_VALUE: Self = Self(T::MAX_VALUE);
            const MIN_VALUE: Self = Self(T::MIN_VALUE);

            #[inline(always)]
            fn saturating_add(self, other: Self) -> Self {
                Self(self.0.saturating_add(other.0))
            }

            #[inline(always)]
            fn checked_add(self, other: Self) -> Option<Self> {
                self.0.checked_add(other.0).map(Self)
            }

            #[inline(always)]
            fn saturating_sub(self, other: Self) -> Self {
                Self(self.0.saturating_sub(other.0))
            }

            #[inline(always)]
            fn checked_sub(self, other: Self) -> Option<Self> {
                self.0.checked_sub(other.0).map(Self)
            }

            #[inline(always)]
            fn saturating_mul(self, other: Self) -> Self {
                Self(self.0.saturating_mul(other.0))
            }

            #[inline(always)]
            fn checked_mul(self, other: Self) -> Option<Self> {
                self.0.checked_mul(other.0).map(Self)
            }

            #[inline(always)]
            fn checked_div(self, other: Self) -> Option<Self> {
                self.0.checked_div(other.0).map(Self)
            }

            #[inline(always)]
            fn to_usize(self) -> usize {
                self.0.to_usize()
            }

            #[inline(always)]
            fn from_usize(value: usize) -> Self {
                Self(T::from_usize(value))
            }
        }
    };
}

/// Defines a transparent wrapper type and all quantized-width aliases.
#[macro_export]
macro_rules! define_quantizable_uint_type_family {
    ($base_name:ident) => {
        $crate::base_quantizable::descriptor_macros::paste! {
            #[repr(transparent)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
            pub struct [<$base_name Type>]<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt>(pub T);

            $crate::impl_quantizable_uint_wrapper!([<$base_name Type>]);

            #[cfg(feature = "support_64bit_indexing_quantization")]
            pub type [<$base_name U64>] = [<$base_name Type>]<u64>;
            pub type [<$base_name U32>] = [<$base_name Type>]<u32>;
            pub type [<$base_name U16>] = [<$base_name Type>]<u16>;
            pub type [<$base_name U8>] = [<$base_name Type>]<u8>;
        }
    };
}

// TODO implement display on alloc builds

#[cfg(feature = "support_64bit_indexing_quantization")]
pub type QuantizableUIntU64 = u64;
pub type QuantizableUIntU32 = u32;
pub type QuantizableUIntU16 = u16;
pub type QuantizableUIntU8 = u8;


#[cfg(not(feature = "alloc"))]
pub trait QuantizableUInt:
    Copy
    + Clone
    + Send
    + Sync
    + core::cmp::PartialOrd
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::AddAssign
    + core::ops::SubAssign
    + core::ops::MulAssign
    + core::ops::DivAssign
    + 'static
{
    const NUMBER_OF_BYTES: usize;
    const ZERO: Self;
    const ONE: Self;
    const MAX_VALUE: Self;
    const MIN_VALUE: Self;
    fn saturating_add(self, other: Self) -> Self;
    fn checked_add(self, other: Self) -> Option<Self>;
    fn saturating_sub(self, other: Self) -> Self;
    fn checked_sub(self, other: Self) -> Option<Self>;
    fn saturating_mul(self, other: Self) -> Self;
    fn checked_mul(self, other: Self) -> Option<Self>;
    fn checked_div(self, other: Self) -> Option<Self>;
    fn to_usize(self) -> usize;
    fn from_usize(value: usize) -> Self;
}

#[cfg(feature = "alloc")]
pub trait QuantizableUInt:
    Copy
    + Clone
    + Send
    + Sync
    + Debug
    + Display
    + Default
    + core::cmp::PartialOrd
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::AddAssign
    + core::ops::SubAssign
    + core::ops::MulAssign
    + core::ops::DivAssign
    + 'static
{
    const NUMBER_OF_BYTES: usize;
    const ZERO: Self;
    const ONE: Self;
    const MAX_VALUE: Self;
    const MIN_VALUE: Self;
    fn saturating_add(self, other: Self) -> Self;
    fn checked_add(self, other: Self) -> Option<Self>;
    fn saturating_sub(self, other: Self) -> Self;
    fn checked_sub(self, other: Self) -> Option<Self>;
    fn saturating_mul(self, other: Self) -> Self;
    fn checked_mul(self, other: Self) -> Option<Self>;
    fn checked_div(self, other: Self) -> Option<Self>;
    fn to_usize(self) -> usize;
    fn from_usize(value: usize) -> Self;
}

impl QuantizableUInt for usize {
    const NUMBER_OF_BYTES: usize = size_of::<usize>();
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = usize::MAX;
    const MIN_VALUE: Self = usize::MIN;

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        self.saturating_add(other)
    }

    #[inline(always)]
    fn checked_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }

    #[inline(always)]
    fn saturating_sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }

    #[inline(always)]
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }

    #[inline(always)]
    fn saturating_mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }

    #[inline(always)]
    fn checked_mul(self, other: Self) -> Option<Self> {
        self.checked_mul(other)
    }

    #[inline(always)]
    fn checked_div(self, other: Self) -> Option<Self> {
        self.checked_div(other)
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value
    }
}

impl QuantizableUInt for u8 {
    const NUMBER_OF_BYTES: usize = size_of::<u8>();
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = u8::MAX;
    const MIN_VALUE: Self = u8::MIN;

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        self.saturating_add(other)
    }

    #[inline(always)]
    fn checked_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }

    #[inline(always)]
    fn saturating_sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }

    #[inline(always)]
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }

    #[inline(always)]
    fn saturating_mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }

    #[inline(always)]
    fn checked_mul(self, other: Self) -> Option<Self> {
        self.checked_mul(other)
    }

    #[inline(always)]
    fn checked_div(self, other: Self) -> Option<Self> {
        self.checked_div(other)
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        match u8::try_from(value) {
            Ok(v) => v,
            Err(_) => u8::MAX,
        }
    }
}

impl QuantizableUInt for u16 {
    const NUMBER_OF_BYTES: usize = size_of::<u16>();
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = u16::MAX;
    const MIN_VALUE: Self = u16::MIN;

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        self.saturating_add(other)
    }

    #[inline(always)]
    fn checked_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }

    #[inline(always)]
    fn saturating_sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }

    #[inline(always)]
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }

    #[inline(always)]
    fn saturating_mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }

    #[inline(always)]
    fn checked_mul(self, other: Self) -> Option<Self> {
        self.checked_mul(other)
    }

    #[inline(always)]
    fn checked_div(self, other: Self) -> Option<Self> {
        self.checked_div(other)
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        match u16::try_from(value) {
            Ok(v) => v,
            Err(_) => u16::MAX,
        }
    }
}

impl QuantizableUInt for u32 {
    const NUMBER_OF_BYTES: usize = size_of::<u32>();
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = u32::MAX;
    const MIN_VALUE: Self = u32::MIN;

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        self.saturating_add(other)
    }

    #[inline(always)]
    fn checked_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }

    #[inline(always)]
    fn saturating_sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }

    #[inline(always)]
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }

    #[inline(always)]
    fn saturating_mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }

    #[inline(always)]
    fn checked_mul(self, other: Self) -> Option<Self> {
        self.checked_mul(other)
    }

    #[inline(always)]
    fn checked_div(self, other: Self) -> Option<Self> {
        self.checked_div(other)
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        match u32::try_from(value) {
            Ok(v) => v,
            Err(_) => u32::MAX,
        }
    }
}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl QuantizableUInt for u64 {
    const NUMBER_OF_BYTES: usize = size_of::<u64>();
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = u64::MAX;
    const MIN_VALUE: Self = u64::MIN;

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        self.saturating_add(other)
    }

    #[inline(always)]
    fn checked_add(self, other: Self) -> Option<Self> {
        self.checked_add(other)
    }

    #[inline(always)]
    fn saturating_sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }

    #[inline(always)]
    fn checked_sub(self, other: Self) -> Option<Self> {
        self.checked_sub(other)
    }

    #[inline(always)]
    fn saturating_mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }

    #[inline(always)]
    fn checked_mul(self, other: Self) -> Option<Self> {
        self.checked_mul(other)
    }

    #[inline(always)]
    fn checked_div(self, other: Self) -> Option<Self> {
        self.checked_div(other)
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        match usize::try_from(self) {
            Ok(v) => v,
            Err(_) => usize::MAX,
        }
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as u64
    }
}

