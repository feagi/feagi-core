use crate::quantization::{FeagiBaseQuantizationType, QuantizableUIntType};

/// Defines a transparent non-zero count wrapper type and forwarding impls.
#[macro_export]
macro_rules! define_nonzero_count_family {
    ($base_name:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "alloc", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "alloc", serde(transparent))]
        pub struct $base_name<T: $crate::quantization::QuantizableNonzeroUIntType>(pub T);

        impl<T: $crate::quantization::QuantizableNonzeroUIntType> $base_name<T> {
            pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES;
            pub const ONE: Self = Self(T::ONE);
            pub const MAX_VALUE: Self = Self(T::MAX_VALUE);
            pub const MIN_VALUE: Self = Self(T::MIN_VALUE);

            #[inline(always)]
            pub(crate) const fn from_const(value: T) -> Self {
                Self(value)
            }

            #[inline(always)]
            pub const fn new_unchecked(value: T) -> Self {
                Self(value)
            }

            #[inline(always)]
            pub fn new(value: T) -> Result<Self, $crate::FeagiStructuresError> {
                if value < T::ONE {
                    return Err($crate::FeagiStructuresError::InvalidValue {
                        context: concat!(stringify!($base_name), " cannot be zero"),
                    });
                }
                Ok(Self(value))
            }

            #[inline(always)]
            pub const fn get(self) -> T {
                self.0
            }

            #[inline(always)]
            pub fn to_usize(self) -> usize {
                self.0.to_usize()
            }

            #[inline(always)]
            pub fn from_usize(value: usize) -> Option<Self> {
                T::from_usize(value).map(Self)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::quantization::QuantizableNonzeroUIntType> Default for $base_name<T> {
            #[inline(always)]
            fn default() -> Self {
                Self(T::ONE)
            }
        }

        impl<T: $crate::quantization::QuantizableNonzeroUIntType> From<$base_name<T>> for usize {
            #[inline(always)]
            fn from(value: $base_name<T>) -> Self {
                value.0.to_usize()
            }
        }

        impl<T: $crate::quantization::QuantizableNonzeroUIntType> core::ops::Add for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T: $crate::quantization::QuantizableNonzeroUIntType> core::ops::Sub for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0.floor_sub(rhs.0))
            }
        }

        impl<T: $crate::quantization::QuantizableNonzeroUIntType> core::ops::Mul for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<T: $crate::quantization::QuantizableNonzeroUIntType> core::ops::Div for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0.checked_div(rhs.0).unwrap_or(T::ONE))
            }
        }

        impl<T: $crate::quantization::QuantizableNonzeroUIntType> core::ops::AddAssign for $base_name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T: $crate::quantization::QuantizableNonzeroUIntType> core::ops::SubAssign for $base_name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 = self.0.floor_sub(rhs.0);
            }
        }

        impl<T: $crate::quantization::QuantizableNonzeroUIntType> core::ops::MulAssign for $base_name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<T: $crate::quantization::QuantizableNonzeroUIntType> core::ops::DivAssign for $base_name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.0 = self.0.checked_div(rhs.0).unwrap_or(T::ONE);
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::quantization::QuantizableNonzeroUIntType + core::fmt::Display> core::fmt::Display
            for $base_name<T>
        {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T: $crate::quantization::QuantizableNonzeroUIntType> $crate::quantization::FeagiBaseQuantizationType for $base_name<T> {
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
                Self(self.0.floor_sub(other.0))
            }

            #[inline(always)]
            fn checked_sub(self, other: Self) -> Option<Self> {
                self.0.checked_sub(other.0).filter(|value| *value >= T::ONE).map(Self)
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
                self.0.checked_div(other.0).filter(|value| *value >= T::ONE).map(Self)
            }
        }

        impl<T: $crate::quantization::QuantizableNonzeroUIntType> $crate::quantization::QuantizableNonzeroUIntType
            for $base_name<T>
        {
            const ONE: Self = Self(T::ONE);
            const MAX_VALUE: Self = Self(T::MAX_VALUE);
            const MIN_VALUE: Self = Self(T::MIN_VALUE);

            #[inline(always)]
            fn floor_sub(self, other: Self) -> Self {
                Self(self.0.floor_sub(other.0))
            }

            #[inline(always)]
            fn to_usize(self) -> usize {
                self.0.to_usize()
            }

            #[inline(always)]
            fn from_usize(value: usize) -> Option<Self> {
                T::from_usize(value).map(Self)
            }
        }
    };
}

pub trait QuantizableNonzeroUIntType: FeagiBaseQuantizationType
+ core::cmp::PartialOrd {

    // Skipping FeagiBaseSingleElementQuantizationType since we cannot define 0!
    const ONE: Self;
    const MAX_VALUE: Self;
    const MIN_VALUE: Self;

    /// Subtraction with a minimum value of 1.
    fn floor_sub(self, other: Self) -> Self;
    fn to_usize(self) -> usize;
    fn from_usize(value: usize) -> Option<Self>;
}

impl<T: QuantizableUIntType> QuantizableNonzeroUIntType for T {
    const ONE: Self = T::ONE;
    const MAX_VALUE: Self = T::MAX_VALUE;
    const MIN_VALUE: Self = T::ONE;

    #[inline(always)]
    fn floor_sub(self, other: Self) -> Self {
        self.checked_sub(other).filter(|value| *value >= T::ONE).unwrap_or(T::ONE)
    }

    #[inline(always)]
    fn to_usize(self) -> usize {
        QuantizableUIntType::to_usize(self)
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Option<Self> {
        let value = T::from_usize(value);
        if value < T::ONE {
            None
        } else {
            Some(value)
        }
    }
}
