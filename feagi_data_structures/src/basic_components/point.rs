// Single points  of data

use std::ops::Range;
use crate::FeagiDataError;

/// Defines the index of something as an integer of a certain type
#[macro_export]
macro_rules! define_index {
    ($name:ident, $inner:ty, $doc:expr) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord
        )]
        pub struct $name($inner);

        impl std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                $name(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

/// Define the count of something that cannot be 0
#[macro_export]
macro_rules! define_nonzero_count {
    ($name:ident, $base:ty, $doc:expr) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name($base);

        impl $name {
            /// Creates a new instance, returns Err if validation fails
            pub fn new(value: $base) -> Result<Self, FeagiDataError> {
                if value == 0 {
                    return Err(FeagiDataError::BadParameters("Count cannot be zero!".into()));
                }
                Ok($name(value))
            }
        }
        impl From<$base> for $name {
            fn from(value: $base) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $base {
            fn from(value: $name) -> $base {
                value.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = $base;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

    }
}

#[macro_export]
macro_rules! define_unsigned_percentage {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct $name {
            value: f32,
        }

        impl $name
        {

            //region Constructors

            pub(crate) fn new_from_0_1_unchecked(value: f32) -> Self {
                $name { value }
            }

            pub fn new_from_0_1(value: f32) -> Result<$name, crate::FeagiDataError> {
                if value > 1.0 || value < 0.0 {
                    return Err(crate::FeagiDataError::BadParameters("Percentage Value must be between 0 and 1!".into()));
                }
                Ok($name { value })
            }

            pub fn new_from_u8_0_255(value: u8) -> Result<$name, crate::FeagiDataError> {
                $name::new_from_0_1(value as f32 / u8::MAX as f32)
            }

            pub fn new_from_0_100(value: f32) -> Result<$name, crate::FeagiDataError> {
                if value > 100.0 || value < 0.0 {
                    return Err(crate::FeagiDataError::BadParameters("Percentage Value must be between 0 and 100!".into()));
                }
                Ok($name { value: value / 100.0 })
            }

            pub fn new_from_linear_interp(value: f32, range: &std::ops::Range<f32>) -> Result<$name, crate::FeagiDataError> {
                if value < range.start || value > range.end {
                    return Err(crate::FeagiDataError::BadParameters(format!("Given value {} is out of range {:?}!", value, range)));
                }
                Ok($name { value: Self::linear_interp(value, range) })

            }

            //endregion

            //region Update

            pub(crate) fn inplace_update(&mut self, value: f32)  {
                self.value = value;
            }

            pub fn inplace_update_from_0_1(&mut self, value: f32) -> Result<(), crate::FeagiDataError> {
                if value > 1.0 || value < 0.0 {
                    return Err(crate::FeagiDataError::BadParameters("Percentage Value must be between 0 and 1!".into()));
                }
                self.value = value;
                Ok(())
            }

            pub fn inplace_update_u8_0_255(&mut self, value: u8) -> Result<(), crate::FeagiDataError> {
                self.inplace_update_from_0_1(value as f32 / u8::MAX as f32)
            }

            pub fn inplace_update_0_100(&mut self, value: f32) -> Result<(), crate::FeagiDataError> {
                if value > 100.0 || value < 0.0 {
                    return Err(crate::FeagiDataError::BadParameters("Percentage Value must be between 0 and 100!".into()));
                }
                self.value = value / 100.0;
                Ok(())
            }

            pub fn inplace_update_linear_interp(&mut self, value: f32, range: &std::ops::Range<f32>) -> Result<(), crate::FeagiDataError> {
                if value < range.start || value > range.end {
                    return Err(crate::FeagiDataError::BadParameters(format!("Given value {} is out of range {:?}!", value, range)));
                }
                self.value = Self::linear_interp(value, range);
                Ok(())
            }

            //endregion

            //region Properties

            pub fn get_as_0_1(&self) -> f32 {
                self.value
            }

            pub fn get_as_u8(&self) -> u8 {
                (self.value * u8::MAX as f32) as u8
            }

            pub fn get_as_0_100(&self) -> f32 {
                self.value * 100.0
            }

            //endregion

            //region Internal

            #[inline]
            fn linear_interp(input: f32, range: &std::ops::Range<f32>) -> f32 {
               (input - range.start) / (range.end - range.start)
            }

            //endregion

            }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "Percent({} %)", self.get_as_0_100())
            }
        }

        impl TryFrom<f32> for $name {
            type Error = crate::FeagiDataError;
            fn try_from(value: f32) -> Result<Self, Self::Error> {
                $name::new_from_0_1(value)
            }
        }

        impl TryFrom<&f32> for $name {
            type Error = crate::FeagiDataError;
            fn try_from(value: &f32) -> Result<Self, Self::Error> {
                $name::new_from_0_1(*value)
            }
        }

        impl From<$name> for f32 {
            fn from(value: $name) -> Self {
                value.value
            }
        }

        impl From<&$name> for f32 {
            fn from(value: &$name) -> Self {
                value.value
            }
        }

    }
}
#[macro_export]
macro_rules! map_unsigned_percentages {
    ($percentage_a:ident, $percentage_b:ident) => {

        impl From<$percentage_a> for $percentage_b {
            fn from(value: $percentage_a) -> Self {
                $percentage_b::new_from_0_1_unchecked(value.get_as_0_1())
            }
        }

        impl From<$percentage_b> for $percentage_a {
            fn from(value: $percentage_b) -> Self {
                $percentage_a::new_from_0_1_unchecked(value.get_as_0_1())
            }
        }
    };
}

#[macro_export]
macro_rules! define_signed_percentage {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct $name {
            value: f32,
        }

        impl $name
        {

            //region Constructors

            pub(crate) fn new_from_m1_1_unchecked(value: f32) -> Self {
                $name { value }
            }

            pub fn new_from_m1_1(value: f32) -> Result<$name, crate::FeagiDataError> {
                if value > 1.0 || value < -1.0 {
                    return Err(crate::FeagiDataError::BadParameters("Signed Percentage Value must be between -1 and 1!".into()));
                }
                Ok($name { value })
            }


            pub fn new_from_m100_100(value: f32) -> Result<$name, crate::FeagiDataError> {
                if value > 100.0 || value < -100.0 {
                    return Err(crate::FeagiDataError::BadParameters("Signed Percentage Value must be between -100 and 100!".into()));
                }
                Ok($name { value: value / 100.0 })
            }

            pub fn new_from_linear_interp(value: f32, range: &std::ops::Range<f32>) -> Result<$name, crate::FeagiDataError> {
                if value < range.start || value > range.end {
                    return Err(crate::FeagiDataError::BadParameters(format!("Given value {} is out of range {:?}!", value, range)));
                }
                Ok($name { value: Self::linear_interp(value, range) })

            }

            //endregion

            //region Update

            pub(crate) fn inplace_update_unchecked(&mut self, value: f32)  {
                self.value = value;
            }

            pub fn inplace_update_from_m1_1(&mut self, value: f32) -> Result<(), crate::FeagiDataError> {
                if value > 1.0 || value < -1.0 {
                    return Err(crate::FeagiDataError::BadParameters("Percentage Value must be between -1 and 1!".into()));
                }
                self.value = value;
                Ok(())
            }

            pub fn inplace_update_m100_100(&mut self, value: f32) -> Result<(), crate::FeagiDataError> {
                if value > 100.0 || value < -100.0 {
                    return Err(crate::FeagiDataError::BadParameters("Percentage Value must be between -1 and 1!".into()));
                }
                self.value = value / 100.0;
                Ok(())
            }

            pub fn inplace_update_linear_interp(&mut self, value: f32, range: &std::ops::Range<f32>) -> Result<(), crate::FeagiDataError> {
                if value < range.start || value > range.end {
                    return Err(crate::FeagiDataError::BadParameters(format!("Given value {} is out of range {:?}!", value, range)));
                }
                self.value = Self::linear_interp(value, range);
                Ok(())
            }

            //endregion

            //region Properties

            pub fn get_as_m1_1(&self) -> f32 {
                self.value
            }

            pub fn get_as_m100_100(&self) -> f32 {
                self.value * 100.0
            }

            //endregion

            //region Internal

            #[inline]
            fn linear_interp(input: f32, range: &std::ops::Range<f32>) -> f32 {
                (range.start + range.end - (2.0 * input)) / (range.start - range.end)
            }

            //endregion

            }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "SignedPercent({} %)", self.get_as_m100_100())
            }
        }

        impl TryFrom<f32> for $name {
            type Error = crate::FeagiDataError;
            fn try_from(value: f32) -> Result<Self, Self::Error> {
                $name::new_from_m1_1(value)
            }
        }

        impl TryFrom<&f32> for $name {
            type Error = crate::FeagiDataError;
            fn try_from(value: &f32) -> Result<Self, Self::Error> {
                $name::new_from_m1_1(*value)
            }
        }

        impl From<$name> for f32 {
            fn from(value: $name) -> Self {
                value.value
            }
        }

        impl From<&$name> for f32 {
            fn from(value: &$name) -> Self {
                value.value
            }
        }

    }
}

#[macro_export]
macro_rules! map_signed_percentages {
    ($percentage_a:ident, $percentage_b:ident) => {

        impl From<$percentage_a> for $percentage_b {
            fn from(value: $percentage_a) -> Self {
                $percentage_b::new_from_m1_1_unchecked(value.get_as_m1_1())
            }
        }

        impl From<$percentage_b> for $percentage_a {
            fn from(value: $percentage_b) -> Self {
                $percentage_a::new_from_m1_1_unchecked(value.get_as_m1_1())
            }
        }
    };
}