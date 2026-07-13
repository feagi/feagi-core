use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// A Crappy f8 implementation written by some LLM. Uses 1 sign bit, 4 exponent bits, and 3
/// mantissa bits, meaning it has an effective range of -7 to 8, with the smallest positive number
/// supported being ~0.001953. The main intent of this struct is storage, as it will
/// convert to a f32 and back for any computations, meaning it is not performant in essentially
/// any situation in contrast to using f32 directly
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StorageF8(u8);

impl StorageF8 {
    const SIGN_MASK: u8 = 0b1000_0000;
    const EXPONENT_MASK: u8 = 0b0111_1000;
    const MANTISSA_MASK: u8 = 0b0000_0111;
    const EXPONENT_SHIFT: u8 = 3;
    const EXPONENT_BIAS: i32 = 7;
    const MANTISSA_SCALE: f32 = 8.0;
    const MAX_FINITE_BITS: u8 = 0b0111_1111;

    pub const ZERO: Self = Self(0);
    pub const NEGATIVE_ZERO: Self = Self(Self::SIGN_MASK);
    pub const MAX: Self = Self(Self::MAX_FINITE_BITS);
    pub const MIN: Self = Self(Self::SIGN_MASK | Self::MAX_FINITE_BITS);

    #[inline(always)]
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    #[inline(always)]
    pub const fn to_bits(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub fn from_f32(value: f32) -> Self {
        if value == 0.0 || value.is_nan() {
            return Self::ZERO;
        }

        let sign = if value.is_sign_negative() { Self::SIGN_MASK } else { 0 };
        let abs_value = value.abs();

        if abs_value.is_infinite() {
            return Self(sign | Self::MAX_FINITE_BITS);
        }

        let mut best_bits = 0;
        let mut best_delta = f32::INFINITY;
        let mut bits = 0;

        loop {
            let candidate = Self::positive_bits_to_f32(bits);
            let delta = (candidate - abs_value).abs();

            if delta < best_delta {
                best_bits = bits;
                best_delta = delta;
            }

            if bits == Self::MAX_FINITE_BITS {
                break;
            }

            bits += 1;
        }

        Self(sign | best_bits)
    }

    #[inline(always)]
    pub fn to_f32(self) -> f32 {
        let value = Self::positive_bits_to_f32(self.0 & !Self::SIGN_MASK);

        if self.is_negative() {
            -value
        } else {
            value
        }
    }

    #[inline(always)]
    pub const fn is_negative(self) -> bool {
        self.0 & Self::SIGN_MASK != 0
    }

    #[inline(always)]
    const fn exponent(self) -> u8 {
        (self.0 & Self::EXPONENT_MASK) >> Self::EXPONENT_SHIFT
    }

    #[inline(always)]
    const fn mantissa(self) -> u8 {
        self.0 & Self::MANTISSA_MASK
    }

    #[inline(always)]
    fn positive_bits_to_f32(bits: u8) -> f32 {
        let value = Self(bits);
        let exponent = value.exponent();
        let mantissa = value.mantissa() as f32 / Self::MANTISSA_SCALE;

        if exponent == 0 {
            mantissa * Self::power_of_two(1 - Self::EXPONENT_BIAS)
        } else {
            (1.0 + mantissa) * Self::power_of_two(exponent as i32 - Self::EXPONENT_BIAS)
        }
    }

    #[inline(always)]
    fn power_of_two(exponent: i32) -> f32 {
        match exponent {
            -6 => 0.015625,
            -5 => 0.03125,
            -4 => 0.0625,
            -3 => 0.125,
            -2 => 0.25,
            -1 => 0.5,
            0 => 1.0,
            1 => 2.0,
            2 => 4.0,
            3 => 8.0,
            4 => 16.0,
            5 => 32.0,
            6 => 64.0,
            7 => 128.0,
            8 => 256.0,
            _ => unreachable!("StorageF8 exponent is outside the E4M3 range"),
        }
    }
}

impl Default for StorageF8 {
    #[inline(always)]
    fn default() -> Self {
        Self::ZERO
    }
}

unsafe impl Send for StorageF8 {}
unsafe impl Sync for StorageF8 {}

impl From<f32> for StorageF8 {
    #[inline(always)]
    fn from(value: f32) -> Self {
        Self::from_f32(value)
    }
}

impl From<StorageF8> for f32 {
    #[inline(always)]
    fn from(value: StorageF8) -> Self {
        value.to_f32()
    }
}

impl Add for StorageF8 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self::from_f32(self.to_f32() + rhs.to_f32())
    }
}

impl Sub for StorageF8 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_f32(self.to_f32() - rhs.to_f32())
    }
}

impl Mul for StorageF8 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_f32(self.to_f32() * rhs.to_f32())
    }
}

impl Div for StorageF8 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        Self::from_f32(self.to_f32() / rhs.to_f32())
    }
}

impl AddAssign for StorageF8 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for StorageF8 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign for StorageF8 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl DivAssign for StorageF8 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl PartialEq for StorageF8 {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.to_f32() == other.to_f32()
    }
}

impl PartialOrd for StorageF8 {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.to_f32().partial_cmp(&other.to_f32())
    }
}

#[cfg(feature = "alloc")]
use core::fmt;

#[cfg(feature = "alloc")]
impl fmt::Debug for StorageF8 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("f8")
            .field("bits", &self.0)
            .field("value", &self.to_f32())
            .finish()
    }
}

#[cfg(feature = "alloc")]
impl fmt::Display for StorageF8 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.to_f32(), formatter)
    }
}
