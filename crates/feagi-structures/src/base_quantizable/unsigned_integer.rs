use crate::base_quantizable::{FeagiBaseQuantizationType, FeagiBaseSingleElementQuantizationType};

/// Defines a transparent wrapper type and all `QuantizableUInt` / operator / conversion impls.
#[macro_export]
macro_rules! define_quantizable_uint_type_family {
    ($base_name:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "alloc", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "alloc", serde(transparent))]
        pub struct $base_name<T: $crate::base_quantizable::QuantizableUIntType>(pub T);

        impl<T: $crate::base_quantizable::QuantizableUIntType> $base_name<T> {
            pub const fn from_const(value: T) -> Self {
                Self(value)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> From<T> for $base_name<T> {
            #[inline(always)]
            fn from(value: T) -> Self {
                Self(value)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::QuantizableUIntType> Default for $base_name<T> {
            #[inline(always)]
            fn default() -> Self {
                Self(T::default())
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> From<$base_name<T>> for usize {
            #[inline(always)]
            fn from(value: $base_name<T>) -> Self {
                value.0.to_usize()
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

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::QuantizableUIntType + core::fmt::Display> core::fmt::Display
            for $base_name<T>
        {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> $crate::base_quantizable::FeagiBaseQuantizationType for $base_name<T> {
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

        impl<T: $crate::base_quantizable::QuantizableUIntType> $crate::base_quantizable::FeagiBaseSingleElementQuantizationType
            for $base_name<T>
        {
            const ZERO: Self = Self(T::ZERO);
            const ONE: Self = Self(T::ONE);
            const MAX_VALUE: Self = Self(T::MAX_VALUE);
            const MIN_VALUE: Self = Self(T::MIN_VALUE);
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> $crate::base_quantizable::QuantizableUIntType for $base_name<T> {
            #[inline(always)]
            fn to_usize(self) -> usize {
                self.0.to_usize()
            }

            #[inline(always)]
            fn from_usize(value: usize) -> Self {
                Self(T::from_usize(value))
            }

            #[inline(always)]
            fn to_usize_range(range: core::ops::Range<Self>) -> core::ops::Range<usize> {
                range.start.0.to_usize()..range.end.0.to_usize()
            }
        }
    };
}


pub trait QuantizableUIntType: FeagiBaseSingleElementQuantizationType {
    fn to_usize(self) -> usize;
    fn from_usize(value: usize) -> Self;

    fn to_usize_range(range: core::ops::Range<Self>) -> core::ops::Range<usize>;
}

impl FeagiBaseQuantizationType for usize {
    const NUMBER_OF_BYTES: usize = size_of::<usize>();

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

impl FeagiBaseSingleElementQuantizationType for usize {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = usize::MAX;
    const MIN_VALUE: Self = usize::MIN;
}

impl QuantizableUIntType for usize {
    #[inline(always)]
    fn to_usize(self) -> usize {
        self
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value
    }

    #[inline(always)]
    fn to_usize_range(range: core::ops::Range<Self>) -> core::ops::Range<usize> {
        range.start.to_usize()..range.end.to_usize()
    }
}



impl FeagiBaseQuantizationType for u8 {
    const NUMBER_OF_BYTES: usize = size_of::<u8>();

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

impl FeagiBaseSingleElementQuantizationType for u8 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = u8::MAX;
    const MIN_VALUE: Self = u8::MIN;
}

impl QuantizableUIntType for u8 {
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

    #[inline(always)]
    fn to_usize_range(range: core::ops::Range<Self>) -> core::ops::Range<usize> {
        range.start.to_usize()..range.end.to_usize()
    }
}



impl FeagiBaseQuantizationType for u16 {
    const NUMBER_OF_BYTES: usize = size_of::<u16>();

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

impl FeagiBaseSingleElementQuantizationType for u16 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = u16::MAX;
    const MIN_VALUE: Self = u16::MIN;
}

impl QuantizableUIntType for u16 {
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

    #[inline(always)]
    fn to_usize_range(range: core::ops::Range<Self>) -> core::ops::Range<usize> {
        range.start.to_usize()..range.end.to_usize()
    }
}



impl FeagiBaseQuantizationType for u32 {
    const NUMBER_OF_BYTES: usize = size_of::<u32>();

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

impl FeagiBaseSingleElementQuantizationType for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = u32::MAX;
    const MIN_VALUE: Self = u32::MIN;
}

impl QuantizableUIntType for u32 {
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

    #[inline(always)]
    fn to_usize_range(range: core::ops::Range<Self>) -> core::ops::Range<usize> {
        range.start.to_usize()..range.end.to_usize()
    }
}



#[cfg(feature = "support_64bit_indexing")]
impl FeagiBaseQuantizationType for u64 {
    const NUMBER_OF_BYTES: usize = size_of::<u64>();

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
impl FeagiBaseSingleElementQuantizationType for u64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX_VALUE: Self = u64::MAX;
    const MIN_VALUE: Self = u64::MIN;
}

#[cfg(feature = "support_64bit_indexing")]
impl QuantizableUIntType for u64 {
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

    #[inline(always)]
    fn to_usize_range(range: core::ops::Range<Self>) -> core::ops::Range<usize> {
        range.start.to_usize()..range.end.to_usize()
    }
}

