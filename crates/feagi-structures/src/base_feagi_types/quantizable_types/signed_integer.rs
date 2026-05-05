use crate::base_feagi_types::quantizable_types::{FeagiBaseQuantizationType, FeagiBaseSingleElementQuantizationType};

/// Defines a transparent wrapper type and all `QuantizableInt` / operator / conversion impls.
#[macro_export]
macro_rules! define_quantizable_int_type_family {
    ($base_name:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "alloc", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "alloc", serde(transparent))]
        pub struct $base_name<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType>(pub T);

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> From<T> for $base_name<T> {
            #[inline(always)]
            fn from(value: T) -> Self {
                Self(value)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> Default for $base_name<T> {
            #[inline(always)]
            fn default() -> Self {
                Self(T::default())
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> From<$base_name<T>> for isize {
            #[inline(always)]
            fn from(value: $base_name<T>) -> Self {
                value.0.to_isize()
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::Add for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::Sub for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::Mul for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::Div for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::AddAssign for $base_name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::SubAssign for $base_name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::MulAssign for $base_name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::DivAssign for $base_name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType + core::fmt::Display> core::fmt::Display
            for $base_name<T>
        {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseQuantizationType for $base_name<T> {
            const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES;

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
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseSingleElementQuantizationType
            for $base_name<T>
        {
            const ZERO: Self = Self(T::ZERO);
            const ONE: Self = Self(T::ONE);
            const MAX_VALUE: Self = Self(T::MAX_VALUE);
            const MIN_VALUE: Self = Self(T::MIN_VALUE);
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> $crate::base_feagi_types::quantizable_types::QuantizableIntType for $base_name<T> {
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


pub trait QuantizableIntType: FeagiBaseSingleElementQuantizationType {
    fn to_isize(self) -> isize;
    fn from_isize(value: isize) -> Self;
}

impl FeagiBaseQuantizationType for isize {
    const NUMBER_OF_BYTES: usize = size_of::<isize>();

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
}

impl FeagiBaseSingleElementQuantizationType for isize {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = isize::MAX;
    const MIN_VALUE: Self = isize::MIN;
}

impl QuantizableIntType for isize {
    #[inline(always)]
    fn to_isize(self) -> isize {
        self
    }

    #[inline(always)]
    fn from_isize(value: isize) -> Self {
        value
    }
}



impl FeagiBaseQuantizationType for i8 {
    const NUMBER_OF_BYTES: usize = size_of::<i8>();

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
}

impl FeagiBaseSingleElementQuantizationType for i8 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = i8::MAX;
    const MIN_VALUE: Self = i8::MIN;
}

impl QuantizableIntType for i8 {
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



impl FeagiBaseQuantizationType for i16 {
    const NUMBER_OF_BYTES: usize = size_of::<i16>();

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
}

impl FeagiBaseSingleElementQuantizationType for i16 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = i16::MAX;
    const MIN_VALUE: Self = i16::MIN;
}

impl QuantizableIntType for i16 {
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



impl FeagiBaseQuantizationType for i32 {
    const NUMBER_OF_BYTES: usize = size_of::<i32>();

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
}

impl FeagiBaseSingleElementQuantizationType for i32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = i32::MAX;
    const MIN_VALUE: Self = i32::MIN;
}

impl QuantizableIntType for i32 {
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



#[cfg(feature = "support_64bit_indexing")]
impl FeagiBaseQuantizationType for i64 {
    const NUMBER_OF_BYTES: usize = size_of::<i64>();

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
}

#[cfg(feature = "support_64bit_indexing")]
impl FeagiBaseSingleElementQuantizationType for i64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = i64::MAX;
    const MIN_VALUE: Self = i64::MIN;
}

#[cfg(feature = "support_64bit_indexing")]
impl QuantizableIntType for i64 {
    #[inline(always)]
    fn to_isize(self) -> isize {
        self as isize
    }

    #[inline(always)]
    fn from_isize(value: isize) -> Self {
        value as i64
    }
}

