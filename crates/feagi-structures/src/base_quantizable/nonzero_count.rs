use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::FeagiStructuresError;

/// Defines forwarding implementations for wrappers over `NonzeroCountType<T>`.
#[macro_export]
macro_rules! impl_nonzero_count_wrapper {
    ($wrapper:ident) => {
        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> $wrapper<T> {
            #[inline(always)]
            pub fn new(value: T) -> Result<Self, $crate::FeagiStructuresError> {
                $crate::base_quantizable::nonzero_count::NonzeroCountType::new(value).map(Self)
            }

            #[inline(always)]
            pub(crate) fn new_unchecked(value: T) -> Self {
                Self($crate::base_quantizable::nonzero_count::NonzeroCountType::new_unchecked(value))
            }

            #[inline(always)]
            pub fn get(self) -> T {
                self.0.get()
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> core::ops::Deref for $wrapper<T> {
            type Target = $crate::base_quantizable::nonzero_count::NonzeroCountType<T>;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> core::fmt::Display for $wrapper<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> TryFrom<T> for $wrapper<T> {
            type Error = $crate::FeagiStructuresError;
            #[inline(always)]
            fn try_from(value: T) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> From<$crate::base_quantizable::nonzero_count::NonzeroCountType<T>> for $wrapper<T> {
            #[inline(always)]
            fn from(value: $crate::base_quantizable::nonzero_count::NonzeroCountType<T>) -> Self {
                Self(value)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> From<$wrapper<T>> for $crate::base_quantizable::nonzero_count::NonzeroCountType<T> {
            #[inline(always)]
            fn from(value: $wrapper<T>) -> Self {
                value.0
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
    };
}

/// Defines a transparent wrapper over `NonzeroCountType<T>` with quantized-width aliases.
#[macro_export]
macro_rules! define_nonzero_count_type_family {
    ($base_name:ident) => {
            #[repr(transparent)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
            pub struct $base_name<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt>(
                pub $crate::base_quantizable::nonzero_count::NonzeroCountType<T>,
            );

            $crate::impl_nonzero_count_wrapper!($base_name);
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
pub struct NonzeroCountType<T: QuantizableUInt>(T);

impl<T: QuantizableUInt> NonzeroCountType<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES;

    pub(crate) fn new_unchecked(n: T) -> Self {
        Self(n)
    }

    pub fn new(n: T) -> Result<Self, FeagiStructuresError> {
        if n < T::ONE {
            return Err(FeagiStructuresError::ValueCannotBeZero {
                context: "nonzero count must be >= 1",
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

impl<T: QuantizableUInt> core::ops::Deref for NonzeroCountType<T> {
    type Target = T;


    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: QuantizableUInt> core::fmt::Display for NonzeroCountType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: QuantizableUInt> core::ops::Add for NonzeroCountType<T> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        self.saturating_add(rhs)
    }
}

impl<T: QuantizableUInt> core::ops::Sub for NonzeroCountType<T> {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        self.saturating_sub(rhs)
    }
}

impl<T: QuantizableUInt> core::ops::Mul for NonzeroCountType<T> {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        self.saturating_mul(rhs)
    }
}

impl<T: QuantizableUInt> core::ops::Div for NonzeroCountType<T> {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        // Divisor is guaranteed nonzero by type invariant.
        self.checked_div(rhs).unwrap_or_else(|| Self::new_unchecked(T::ONE))
    }
}

impl<T: QuantizableUInt> core::ops::AddAssign for NonzeroCountType<T> {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = self.saturating_add(rhs);
    }
}

impl<T: QuantizableUInt> core::ops::SubAssign for NonzeroCountType<T> {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.saturating_sub(rhs);
    }
}

impl<T: QuantizableUInt> core::ops::MulAssign for NonzeroCountType<T> {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.saturating_mul(rhs);
    }
}

impl<T: QuantizableUInt> core::ops::DivAssign for NonzeroCountType<T> {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        *self = self.checked_div(rhs).unwrap_or_else(|| Self::new_unchecked(T::ONE));
    }
}
