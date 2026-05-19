use crate::base_feagi_types::quantizable_types::shared::{FeagiBaseQuantizationType, FeagiBaseSingleElementQuantizationType};

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
            const QUANTIZATION_LEVEL: crate::quantization_level::QuantizationLevel = T::QUANTIZATION_LEVEL;

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

    // isize range? I dont think we need this

    fn through_isize_to_quant<Q: QuantizableIntType>(self) -> Q {
        Q::from_isize(self.to_isize())
    }
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

