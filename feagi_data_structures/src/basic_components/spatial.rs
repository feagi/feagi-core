//! Spatial data structures for FEAGI brain coordinates and dimensions.
//!
//! This module provides fundamental spatial primitives including 3D coordinates
//! and dimensional bounds checking for neural space representation.

use std::ops::Range;
use crate::FeagiDataError;

macro_rules! define_xyz_coordinates {
    ($name:ident, $var_type:ty, $friendly_name:expr, $doc_string:expr) => {

        /// $doc_string
        #[derive(Clone, Debug, PartialEq)]
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

macro_rules! define_xyz_dimensions {
    ($name:ident, $var_type:ty, $friendly_name:expr, $invalid_zero_value:expr, $doc_string:expr) => {

        /// $doc_string
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            pub x: $var_type,
            pub y: $var_type,
            pub z: $var_type,
        }

        impl $name {
            pub fn new(x: $var_type, y: $var_type, z: $var_type) -> Result<Self, FeagiDataError> {
                if x == $invalid_zero_value || y == $invalid_zero_value || z == $invalid_zero_value {
                    return Err(FeagiDataError::BadParameters(format!("Value cannot be {:?} in a {:?}!", $invalid_zero_value, $friendly_name)));
                }
                Ok(Self { x, y, z })
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}<{}, {}, {}>", $friendly_name, self.x, self.y, self.z)
            }
        }

    }
}

define_xyz_coordinates!(U32XYZCoordinate, u32, "U32XYZCoordinate", "3D u32 coordinate");
define_xyz_coordinates!(I32XYZCoordinate, i32, "I32XYZCoordinate", "3D i32 coordinate");

define_xyz_dimensions!(U32XYZDimensions, u32, "U32XYZDimensions", 0, "3D u32 dimensions");



/// A 3D range defining acceptable coordinate bounds.
/// Each axis has its own range for flexible boundary definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DimensionRange{
    pub x: Range<u32>,
    pub y: Range<u32>,
    pub z: Range<u32>,
}

impl DimensionRange {
    /// Creates a new dimension range, ensuring no ranges are empty.
    pub fn new(x: Range<u32>, y: Range<u32>, z: Range<u32>) -> Result<DimensionRange, FeagiDataError> {
        if x.is_empty() || y.is_empty() || z.is_empty() {
            return Err(FeagiDataError::BadParameters("A given range has some empty or invalid ranges!".into()))
        }
        Ok(DimensionRange { x, y, z })
    }
    
    /// Returns true if any axis spans more than one value (i.e., not a single point).
    pub fn is_ambiguous(&self) -> bool {
        self.x.len() != 1 || self.y.len() != 1 || self.z.len() != 1
    }
    
    /// Verifies that a coordinate falls within all axis ranges.
    pub fn verify_coordinate_u32_within_range(&self, checking: &U32XYZCoordinate) -> Result<(), FeagiDataError> {
        if !self.x.contains(&checking.x) || !self.y.contains(&checking.y) || !self.z.contains(&checking.z){
            return Err(FeagiDataError::BadParameters(format!("Point {} is not within the acceptable range of {}!", checking, self)));
        }
        Ok(())

    }
}

impl std::fmt::Display for DimensionRange{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<{} - {}, {} - {}, {} - {}>", self.x.start, self.x.end, self.y.start, self.y.end, self.z.start, self.z.end)
    }
}



