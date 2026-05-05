use crate::base_feagi_types::percentage::shared::FeagiBasePercentageType;

#[macro_export]
macro_rules! define_unsigned_percentage_type_family {
    ($base_name:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "alloc", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "alloc", serde(transparent))]
        pub struct $base_name(u8);

        impl $base_name {
            pub const RAW_ZERO_PERCENT: u8 = u8::MIN;
            pub const RAW_HUNDRED_PERCENT: u8 = u8::MAX;
            pub const MAX_AS_F32: f32 = u8::MAX as f32;

            #[inline(always)]
            pub const fn from_raw(raw: u8) -> Self {
                Self(raw)
            }

            #[inline(always)]
            pub const fn to_raw(self) -> u8 {
                self.0
            }

            #[inline(always)]
            pub fn from_f32_saturating(float: f32) -> Self {
                if float.is_nan() {
                    Self(Self::RAW_ZERO_PERCENT)
                } else if float.is_sign_negative() {
                    Self(Self::RAW_ZERO_PERCENT)
                } else if float >= 1.0 {
                    Self(Self::RAW_HUNDRED_PERCENT)
                } else {
                    Self((float * Self::MAX_AS_F32).round() as u8)
                }
            }

            #[inline(always)]
            pub fn from_f32(float: f32) -> Option<Self> {
                if float.is_finite() && (0.0..=1.0).contains(&float) {
                    Some(Self::from_f32_saturating(float))
                } else {
                    None
                }
            }

            #[inline(always)]
            pub fn to_f32(self) -> f32 {
                self.0 as f32 / Self::MAX_AS_F32
            }
        }

        impl core::ops::Add for $base_name {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                <Self as $crate::base_feagi_types::percentage::shared::FeagiBasePercentageType>::saturating_add(self, rhs)
            }
        }

        impl core::ops::Sub for $base_name {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                <Self as $crate::base_feagi_types::percentage::shared::FeagiBasePercentageType>::saturating_sub(self, rhs)
            }
        }

        impl core::ops::Mul for $base_name {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                <Self as $crate::base_feagi_types::percentage::shared::FeagiBasePercentageType>::saturating_mul(self, rhs)
            }
        }

        impl core::ops::Div for $base_name {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self::from_f32_saturating(self.to_f32() / rhs.to_f32())
            }
        }

        impl core::ops::AddAssign for $base_name {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl core::ops::SubAssign for $base_name {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        impl core::ops::MulAssign for $base_name {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                *self = *self * rhs;
            }
        }

        impl core::ops::DivAssign for $base_name {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                *self = *self / rhs;
            }
        }

        #[cfg(feature = "alloc")]
        impl Default for $base_name {
            #[inline(always)]
            fn default() -> Self {
                Self(Self::RAW_ZERO_PERCENT)
            }
        }

        #[cfg(feature = "alloc")]
        impl core::fmt::Display for $base_name {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}%", self.to_f32() * 100.0)
            }
        }

        impl $crate::base_feagi_types::percentage::shared::FeagiBasePercentageType for $base_name {
            const ZERO_PERCENT: Self = Self(Self::RAW_ZERO_PERCENT);
            const HUNDRED_PERCENT: Self = Self(Self::RAW_HUNDRED_PERCENT);
            const MAX_AS_F32: f32 = Self::MAX_AS_F32;

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
                Self::from_f32_saturating(self.to_f32() * other.to_f32())
            }

            #[inline(always)]
            fn checked_mul(self, other: Self) -> Option<Self> {
                Self::from_f32(self.to_f32() * other.to_f32())
            }

            #[inline(always)]
            fn checked_div(self, other: Self) -> Option<Self> {
                Self::from_f32(self.to_f32() / other.to_f32())
            }

            #[inline(always)]
            fn from_f32(float: f32) -> Option<Self> {
                Self::from_f32(float)
            }

            #[inline(always)]
            fn to_f32(self) -> f32 {
                self.to_f32()
            }
        }

        impl $crate::base_feagi_types::percentage::UnsignedPercentageType for $base_name {}
    };
}

pub trait UnsignedPercentageType: FeagiBasePercentageType {
    
}
