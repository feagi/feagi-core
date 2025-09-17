//! Spatial data structures for FEAGI brain coordinates and dimensions.
//!
//! This module provides fundamental spatial primitives including 3D coordinates
//! and dimensional bounds checking for neural space representation.

use std::ops::Range;
use crate::FeagiDataError;

#[macro_export]
macro_rules! define_xyz_coordinates {
    ($name:ident, $var_type:ty, $friendly_name:expr, $doc_string:expr) => {

        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
        pub struct $name {
            pub x: $var_type,
            pub y: $var_type,
            pub z: $var_type,
        }

        impl $name {
            pub fn new(x: $var_type, y: $var_type, z: $var_type) -> Self {
                Self { x, y, z }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}({}, {}, {})", $friendly_name, self.x, self.y, self.z)
            }
        }

    };
}

#[macro_export]
macro_rules! define_xyz_dimensions {
    ($name:ident, $var_type:ty, $friendly_name:expr, $invalid_zero_value:expr, $doc_string:expr) => {

        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
        pub struct $name {
            pub width: $var_type,
            pub height: $var_type,
            pub depth: $var_type,
        }

        impl $name {
            pub fn new(x: $var_type, y: $var_type, z: $var_type) -> Result<Self, FeagiDataError> {
                if x == $invalid_zero_value || y == $invalid_zero_value || z == $invalid_zero_value {
                    return Err(FeagiDataError::BadParameters(format!("Value cannot be {:?} in a {:?}!", $invalid_zero_value, $friendly_name)));
                }
                Ok(Self { width: x, height: y, depth: z })
            }

            pub fn number_elements(&self) -> $var_type {
                self.width * self.height * self.depth
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}<{}, {}, {}>", $friendly_name, self.width, self.height, self.depth)
            }
        }
    }
}

#[macro_export]
macro_rules! define_xyz_mapping{
    ($XYZ_a:ident, $XYZ_b:ident) => {
        impl From<$XYZ_a> for $XYZ_b {
            fn from(a: $XYZ_a) -> Self {
                $XYZ_b::new(a.width, a.height, a.depth).unwrap()
            }
        }
        impl From<$XYZ_b> for $XYZ_a {
            fn from(b: $XYZ_b) -> Self {
                $XYZ_a::new(b.width, b.height, b.depth).unwrap()
            }
        }
    }
}

/// Define a dimension range wrapper type with specific semantic meaning
#[macro_export]
macro_rules! define_xyz_dimension_range {
    ($name:ident, $var_type:ty, $coordinate_type:ty, $friendly_name:expr, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            pub width: std::ops::Range<$var_type>,
            pub height: std::ops::Range<$var_type>,
            pub depth: std::ops::Range<$var_type>
        }

        impl $name {
            /// Creates a new dimension range, ensuring no ranges are empty.
            pub fn new(x: std::ops::Range<$var_type>, y: std::ops::Range<$var_type>, z: std::ops::Range<$var_type>) -> Result<Self, FeagiDataError> {
                Ok($name {width: x, height: y, depth: z})
            }

            /// Verifies that a coordinate falls within all axis ranges.
            pub fn verify_coordinate_within_range(&self, coordinate: &$coordinate_type) -> Result<(), FeagiDataError> {
                if self.width.contains(&coordinate.width) && self.height.contains(&coordinate.height) && self.depth.contains(&coordinate.depth) {
                    return Ok(());
                }
                Err(FeagiDataError::BadParameters(format!("Coordinate {:?} is not contained by this given range of {:?}!", coordinate, self)))

            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}<{:?}, {:?}, {:?}>", $friendly_name, self.width, self.height, self.depth)
            }
        }
    };
}



