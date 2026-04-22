use crate::base_quantizable::QuantizableUIntType;
use crate::FeagiStructuresError;

/// Defines a transparent wrapper over `NonzeroCountType<T>` and forwarding impls.
#[macro_export]
macro_rules! define_nonzero_count_family {
    ($base_name:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        pub struct $base_name<T: $crate::base_quantizable::QuantizableUIntType>(
            pub $crate::base_quantizable::NonzeroCount<T>,
        );

        impl<T: $crate::base_quantizable::QuantizableUIntType> $base_name<T> {
            #[inline(always)]
            pub fn new(value: T) -> Result<Self, $crate::FeagiStructuresError> {
                $crate::base_quantizable::NonzeroCount::new(value).map(Self)
            }

            #[inline(always)]
            pub(crate) fn new_unchecked(value: T) -> Self {
                Self($crate::base_quantizable::NonzeroCount::new_unchecked(
                    value,
                ))
            }

            #[inline(always)]
            pub fn get(self) -> T {
                self.0.get()
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> core::ops::Deref for $base_name<T> {
            type Target = $crate::base_quantizable::NonzeroCount<T>;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::QuantizableUIntType> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType>
            From<$crate::base_quantizable::NonzeroCount<T>> for $base_name<T>
        {
            #[inline(always)]
            fn from(value: $crate::base_quantizable::NonzeroCount<T>) -> Self {
                Self(value)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType>
            From<$base_name<T>> for $crate::base_quantizable::NonzeroCount<T>
        {
            #[inline(always)]
            fn from(value: $base_name<T>) -> Self {
                value.0
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> core::ops::Add for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> core::ops::Sub for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> core::ops::Mul for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> core::ops::Div for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> core::ops::AddAssign for $base_name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> core::ops::SubAssign for $base_name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> core::ops::MulAssign for $base_name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> core::ops::DivAssign for $base_name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }
    };
}

#[repr(transparent)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct NonzeroCount<T: QuantizableUIntType>(T);

impl<T: QuantizableUIntType> NonzeroCount<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES;
    
    pub const ONE: Self = Self(T::ONE);

    pub(crate) fn new_unchecked(n: T) -> Self {
        Self(n)
    }

    pub fn new(n: T) -> Result<Self, FeagiStructuresError> {
        if n < T::ONE {
            return Err(FeagiStructuresError::InvalidValue {
                context: "nonzero count for a NonZeroType value must be >= 1",
            });
        }
        Ok(Self(n))
    }

    pub fn get(self) -> T {
        self.0
    }

    #[inline(always)]
    pub fn saturating_add(self, other: Self) -> Self {
        Self::new_unchecked(self.0.saturating_add(other.0))
    }

    #[inline(always)]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).and_then(Self::new_checked_nonzero)
    }

    #[inline(always)]
    pub fn saturating_sub(self, other: Self) -> Self {
        let value = self.0.saturating_sub(other.0);
        Self::new_checked_nonzero(value).unwrap_or_else(|| Self::new_unchecked(T::ONE))
    }

    #[inline(always)]
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).and_then(Self::new_checked_nonzero)
    }

    #[inline(always)]
    pub fn saturating_mul(self, other: Self) -> Self {
        Self::new_unchecked(self.0.saturating_mul(other.0))
    }

    #[inline(always)]
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        self.0.checked_mul(other.0).and_then(Self::new_checked_nonzero)
    }

    #[inline(always)]
    pub fn checked_div(self, other: Self) -> Option<Self> {
        self.0.checked_div(other.0).and_then(Self::new_checked_nonzero)
    }

    #[inline(always)]
    fn new_checked_nonzero(value: T) -> Option<Self> {
        if value < T::ONE {
            None
        } else {
            Some(Self::new_unchecked(value))
        }
    }
}

impl<T: QuantizableUIntType> core::ops::Deref for NonzeroCount<T> {
    type Target = T;


    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: QuantizableUIntType> core::fmt::Display for NonzeroCount<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: QuantizableUIntType> core::ops::Add for NonzeroCount<T> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        self.saturating_add(rhs)
    }
}

impl<T: QuantizableUIntType> core::ops::Sub for NonzeroCount<T> {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        self.saturating_sub(rhs)
    }
}

impl<T: QuantizableUIntType> core::ops::Mul for NonzeroCount<T> {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        self.saturating_mul(rhs)
    }
}

impl<T: QuantizableUIntType> core::ops::Div for NonzeroCount<T> {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        // Divisor is guaranteed nonzero by type invariant.
        self.checked_div(rhs).unwrap_or_else(|| Self::new_unchecked(T::ONE))
    }
}

impl<T: QuantizableUIntType> core::ops::AddAssign for NonzeroCount<T> {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = self.saturating_add(rhs);
    }
}

impl<T: QuantizableUIntType> core::ops::SubAssign for NonzeroCount<T> {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.saturating_sub(rhs);
    }
}

impl<T: QuantizableUIntType> core::ops::MulAssign for NonzeroCount<T> {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.saturating_mul(rhs);
    }
}

impl<T: QuantizableUIntType> core::ops::DivAssign for NonzeroCount<T> {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        *self = self.checked_div(rhs).unwrap_or_else(|| Self::new_unchecked(T::ONE));
    }
}
