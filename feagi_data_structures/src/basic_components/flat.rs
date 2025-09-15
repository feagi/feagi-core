// 2D data

use crate::FeagiDataError;

macro_rules! define_xy_coordinates {
    ($name:ident, $var_type:ty, $friendly_name:expr, $doc_string:expr) => {

        /// $doc_string
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            pub x: $var_type,
            pub y: $var_type,
        }

        impl $name {
            pub fn new(x: $var_type, y: $var_type) -> Self {
                Self { x, y }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}({}, {})", $friendly_name, self.x, self.y)
            }
        }

    };
}



macro_rules! define_xy_dimensions {
    ($name:ident, $var_type:ty, $friendly_name:expr, $invalid_zero_value:expr, $doc_string:expr) => {

        /// $doc_string
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            pub x: $var_type,
            pub y: $var_type,
        }

        impl $name {
            pub fn new(x: $var_type, y: $var_type) -> Result<Self, FeagiDataError> {
                if x == $invalid_zero_value || y == $invalid_zero_value {
                    return Err(FeagiDataError::BadParameters(format!("Value cannot be {:?} in a {:?}!", $invalid_zero_value, $friendly_name)));
                }
                Ok(Self { x, y })
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}<{}, {}>", $friendly_name, self.x, self.y)
            }
        }

    }
}


define_xy_coordinates!(U32XY, u32, "U32XY", "2D u32 coordinate");

define_xy_dimensions!(U32XYDimensions, u32, "U32XYDimensions", 0, "2D Dimensions");