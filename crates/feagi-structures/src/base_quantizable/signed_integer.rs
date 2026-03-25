use core::fmt::{Debug, Display};

/// Defines all implementations of QuantizableInt and its dependent traits.
#[macro_export]
macro_rules! impl_quantizable_int_wrapper {
    ($wrapper:ident) => {
        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> From<T> for $wrapper<T> {
            #[inline(always)]
            fn from(value: T) -> Self {
                Self(value)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> Default for $wrapper<T> {
            #[inline(always)]
            fn default() -> Self {
                Self(T::default())
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> From<$wrapper<T>> for isize {
            #[inline(always)]
            fn from(value: $wrapper<T>) -> Self {
                value.0.to_isize()
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> core::ops::Add for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> core::ops::Sub for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> core::ops::Mul for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> core::ops::Div for $wrapper<T> {
            type Output = Self;
            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> core::ops::AddAssign for $wrapper<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> core::ops::SubAssign for $wrapper<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> core::ops::MulAssign for $wrapper<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> core::ops::DivAssign for $wrapper<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt + core::fmt::Display> core::fmt::Display for $wrapper<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> $crate::base_quantizable::signed_integer::QuantizableInt for $wrapper<T> {
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
            fn to_isize(self) -> isize {
                self.0.to_isize()
            }

            #[inline(always)]
            fn from_isize(value: isize) -> Self {
                Self(T::from_isize(value))
            }
        }
    };
}

/// Defines a transparent wrapper type and all quantized-width aliases.
#[macro_export]
macro_rules! define_quantizable_int_type_family {
    ($base_name:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $base_name<T: $crate::base_quantizable::signed_integer::QuantizableInt>(pub T);

        $crate::impl_quantizable_int_wrapper!($base_name);
    };
}

#[cfg(feature = "support_64bit_indexing_quantization")]
pub type QuantizableSIntI64 = i64;
pub type QuantizableSIntI32 = i32;
pub type QuantizableSIntI16 = i16;
pub type QuantizableSIntI8 = i8;

#[cfg(not(feature = "alloc"))]
pub trait QuantizableInt:
    Copy
    + Clone
    + Send
    + Sync
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::AddAssign
    + core::ops::SubAssign
    + core::ops::MulAssign
    + core::ops::DivAssign
    + core::cmp::PartialOrd
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
    fn to_isize(self) -> isize;
    fn from_isize(value: isize) -> Self;
}

#[cfg(feature = "alloc")]
pub trait QuantizableInt:
    Copy
    + Clone
    + Send
    + Sync
    + Debug
    + Display
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
    fn to_isize(self) -> isize;
    fn from_isize(value: isize) -> Self;
}

impl QuantizableInt for isize {
    const NUMBER_OF_BYTES: usize = size_of::<isize>();
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = isize::MAX;
    const MIN_VALUE: Self = isize::MIN;

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
    fn to_isize(self) -> isize {
        self
    }

    #[inline(always)]
    fn from_isize(value: isize) -> Self {
        value
    }
}

impl QuantizableInt for i8 {
    const NUMBER_OF_BYTES: usize = size_of::<i8>();
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = i8::MAX;
    const MIN_VALUE: Self = i8::MIN;

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
    fn to_isize(self) -> isize {
        self as isize
    }

    #[inline(always)]
    fn from_isize(value: isize) -> Self {
        match i8::try_from(value) {
            Ok(v) => v,
            Err(_) => {
                if value.is_negative() {
                    i8::MIN
                } else {
                    i8::MAX
                }
            }
        }
    }
}

impl QuantizableInt for i16 {
    const NUMBER_OF_BYTES: usize = size_of::<i16>();
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = i16::MAX;
    const MIN_VALUE: Self = i16::MIN;

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
    fn to_isize(self) -> isize {
        self as isize
    }

    #[inline(always)]
    fn from_isize(value: isize) -> Self {
        match i16::try_from(value) {
            Ok(v) => v,
            Err(_) => {
                if value.is_negative() {
                    i16::MIN
                } else {
                    i16::MAX
                }
            }
        }
    }
}

impl QuantizableInt for i32 {
    const NUMBER_OF_BYTES: usize = size_of::<i32>();
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = i32::MAX;
    const MIN_VALUE: Self = i32::MIN;

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
    fn to_isize(self) -> isize {
        self as isize
    }

    #[inline(always)]
    fn from_isize(value: isize) -> Self {
        match i32::try_from(value) {
            Ok(v) => v,
            Err(_) => {
                if value.is_negative() {
                    i32::MIN
                } else {
                    i32::MAX
                }
            }
        }
    }
}

#[cfg(feature = "support_64bit_indexing_quantization")]
impl QuantizableInt for i64 {
    const NUMBER_OF_BYTES: usize = size_of::<i64>();
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = i64::MAX;
    const MIN_VALUE: Self = i64::MIN;

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
    fn to_isize(self) -> isize {
        self as isize
    }

    #[inline(always)]
    fn from_isize(value: isize) -> Self {
        value as i64
    }
}

