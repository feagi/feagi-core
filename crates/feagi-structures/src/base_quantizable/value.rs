use half::f16;


/// Defines a transparent value wrapper type and all `QuantizableValue` / operator / conversion impls.
#[macro_export]
macro_rules! define_quantizable_value_type_family {
    ($base_name:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        #[cfg_attr(feature = "alloc", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "alloc", serde(transparent))]
        pub struct $base_name<T: $crate::base_quantizable::QuantizableValueType>(pub T);

        impl<T: $crate::base_quantizable::QuantizableValueType> From<T> for $base_name<T> {
            #[inline(always)]
            fn from(value: T) -> Self {
                Self(value)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::QuantizableValueType> Default for $base_name<T> {
            #[inline(always)]
            fn default() -> Self {
                Self(T::default())
            }
        }

        impl<T: $crate::base_quantizable::QuantizableValueType> From<$base_name<T>> for f32 {
            #[inline(always)]
            fn from(value: $base_name<T>) -> Self {
                value.0.to_f32()
            }
        }

        impl<T: $crate::base_quantizable::QuantizableValueType> core::ops::Add for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableValueType> core::ops::Sub for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableValueType> core::ops::Mul for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableValueType> core::ops::Div for $base_name<T> {
            type Output = Self;
            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableValueType> core::ops::AddAssign for $base_name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::QuantizableValueType> core::ops::SubAssign for $base_name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::QuantizableValueType> core::ops::MulAssign for $base_name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<T: $crate::base_quantizable::QuantizableValueType> core::ops::DivAssign for $base_name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::QuantizableValueType + core::fmt::Display> core::fmt::Display
            for $base_name<T>
        {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableValueType> $crate::base_quantizable::QuantizableValueType for $base_name<T> {
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
            fn to_f32(self) -> f32 {
                self.0.to_f32()
            }

            #[inline(always)]
            fn from_f32(value: f32) -> Self {
                Self(T::from_f32(value))
            }

            #[inline(always)]
            fn write_le_bytes(self, dst: &mut [u8]) {
                self.0.write_le_bytes(dst);
            }

            #[inline(always)]
            fn read_le_bytes(src: &[u8]) -> Self {
                Self(T::read_le_bytes(src))
            }
        }
    };
}


#[cfg(not(feature = "alloc"))]
pub trait QuantizableValueType:
    Copy
    + core::clone::Clone
    + Send
    + Sync
    + core::convert::Into<f32>
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
    fn to_f32(self) -> f32;
    fn from_f32(value: f32) -> Self;

    /// Writes this value as little-endian bytes into `dst`. `dst` must have length
    /// at least `Self::NUMBER_OF_BYTES`; excess bytes are untouched.
    fn write_le_bytes(self, dst: &mut [u8]);

    /// Reads a value from `src` interpreted as little-endian bytes. `src` must
    /// have length at least `Self::NUMBER_OF_BYTES`; excess bytes are ignored.
    fn read_le_bytes(src: &[u8]) -> Self;
}

#[cfg(feature = "alloc")]
pub trait QuantizableValueType:
    Copy
    + core::clone::Clone
    + Send
    + Sync
    + core::fmt::Debug
    + core::fmt::Display
    + core::default::Default
    + core::convert::Into<f32>
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
    fn to_f32(self) -> f32;
    fn from_f32(value: f32) -> Self;

    /// Writes this value as little-endian bytes into `dst`. `dst` must have length
    /// at least `Self::NUMBER_OF_BYTES`; excess bytes are untouched.
    fn write_le_bytes(self, dst: &mut [u8]);

    /// Reads a value from `src` interpreted as little-endian bytes. `src` must
    /// have length at least `Self::NUMBER_OF_BYTES`; excess bytes are ignored.
    fn read_le_bytes(src: &[u8]) -> Self;
}

impl QuantizableValueType for f32 {
    const NUMBER_OF_BYTES: usize = size_of::<f32>();
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const MAX_VALUE: Self = f32::MAX;
    const MIN_VALUE: Self = f32::MIN;

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        let value = self + other;
        if value.is_infinite() {
            if value.is_sign_negative() { f32::MIN } else { f32::MAX }
        } else if value.is_nan() {
            0.0
        } else {
            value
        }
    }

    #[inline(always)]
    fn checked_add(self, other: Self) -> Option<Self> {
        let value = self + other;
        if value.is_finite() { Some(value) } else { None }
    }

    #[inline(always)]
    fn saturating_sub(self, other: Self) -> Self {
        let value = self - other;
        if value.is_infinite() {
            if value.is_sign_negative() { f32::MIN } else { f32::MAX }
        } else if value.is_nan() {
            0.0
        } else {
            value
        }
    }

    #[inline(always)]
    fn checked_sub(self, other: Self) -> Option<Self> {
        let value = self - other;
        if value.is_finite() { Some(value) } else { None }
    }

    #[inline(always)]
    fn saturating_mul(self, other: Self) -> Self {
        let value = self * other;
        if value.is_infinite() {
            if value.is_sign_negative() { f32::MIN } else { f32::MAX }
        } else if value.is_nan() {
            0.0
        } else {
            value
        }
    }

    #[inline(always)]
    fn checked_mul(self, other: Self) -> Option<Self> {
        let value = self * other;
        if value.is_finite() { Some(value) } else { None }
    }

    #[inline(always)]
    fn checked_div(self, other: Self) -> Option<Self> {
        let value = self / other;
        if value.is_finite() { Some(value) } else { None }
    }

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self
    }

    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        if value.is_infinite() {
            if value.is_sign_negative() { f32::MIN } else { f32::MAX }
        } else if value.is_nan() {
            0.0
        } else {
            value
        }
    }

    #[inline(always)]
    fn write_le_bytes(self, dst: &mut [u8]) {
        dst[..4].copy_from_slice(&self.to_le_bytes());
    }

    #[inline(always)]
    fn read_le_bytes(src: &[u8]) -> Self {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&src[..4]);
        f32::from_le_bytes(buf)
    }
}

impl QuantizableValueType for f16 {
    const NUMBER_OF_BYTES: usize = size_of::<f16>();
    const ZERO: Self = f16::ZERO;
    const ONE: Self = f16::ONE;
    const MAX_VALUE: Self = f16::MAX;
    const MIN_VALUE: Self = f16::MIN;

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        Self::from_f32(self.to_f32() + other.to_f32())
    }

    #[inline(always)]
    fn checked_add(self, other: Self) -> Option<Self> {
        let value = self.to_f32() + other.to_f32();
        if value.is_finite() && value <= f16::MAX.to_f32() && value >= f16::MIN.to_f32() {
            Some(f16::from_f32(value))
        } else {
            None
        }
    }

    #[inline(always)]
    fn saturating_sub(self, other: Self) -> Self {
        Self::from_f32(self.to_f32() - other.to_f32())
    }

    #[inline(always)]
    fn checked_sub(self, other: Self) -> Option<Self> {
        let value = self.to_f32() - other.to_f32();
        if value.is_finite() && value <= f16::MAX.to_f32() && value >= f16::MIN.to_f32() {
            Some(f16::from_f32(value))
        } else {
            None
        }
    }

    #[inline(always)]
    fn saturating_mul(self, other: Self) -> Self {
        Self::from_f32(self.to_f32() * other.to_f32())
    }

    #[inline(always)]
    fn checked_mul(self, other: Self) -> Option<Self> {
        let value = self.to_f32() * other.to_f32();
        if value.is_finite() && value <= f16::MAX.to_f32() && value >= f16::MIN.to_f32() {
            Some(f16::from_f32(value))
        } else {
            None
        }
    }

    #[inline(always)]
    fn checked_div(self, other: Self) -> Option<Self> {
        let value = self.to_f32() / other.to_f32();
        if value.is_finite() && value <= f16::MAX.to_f32() && value >= f16::MIN.to_f32() {
            Some(f16::from_f32(value))
        } else {
            None
        }
    }

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self.to_f32()
    }

    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        if value.is_nan() {
            f16::from_f32(0.0)
        } else if value > f16::MAX.to_f32() {
            f16::MAX
        } else if value < f16::MIN.to_f32() {
            f16::MIN
        } else {
            f16::from_f32(value)
        }
    }

    #[inline(always)]
    fn write_le_bytes(self, dst: &mut [u8]) {
        dst[..2].copy_from_slice(&self.to_le_bytes());
    }

    #[inline(always)]
    fn read_le_bytes(src: &[u8]) -> Self {
        let mut buf = [0u8; 2];
        buf.copy_from_slice(&src[..2]);
        f16::from_le_bytes(buf)
    }
}

impl QuantizableValueType for u8 {
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
    fn to_f32(self) -> f32 {
        self as f32
    }

    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        if value.is_nan() {
            0
        } else if value.is_sign_negative() {
            0
        } else if value > u8::MAX as f32 {
            u8::MAX
        } else {
            value as u8
        }
    }

    #[inline(always)]
    fn write_le_bytes(self, dst: &mut [u8]) {
        dst[0] = self;
    }

    #[inline(always)]
    fn read_le_bytes(src: &[u8]) -> Self {
        src[0]
    }
}
