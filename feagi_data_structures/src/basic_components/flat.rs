// 2D data

#[macro_export]
macro_rules! define_xy_coordinates {
    ($name:ident, $var_type:ty, $friendly_name:expr, $doc_string:expr) => {

        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
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

#[macro_export]
macro_rules! define_xy_dimensions {
    ($name:ident, $var_type:ty, $friendly_name:expr, $invalid_zero_value:expr, $doc_string:expr) => {

        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Copy, Hash, Eq)]
        pub struct $name {
            pub width: $var_type,
            pub height: $var_type,
        }

        impl $name {
            pub fn new(x: $var_type, y: $var_type) -> Result<Self, crate::FeagiDataError> {
                if x == $invalid_zero_value || y == $invalid_zero_value {
                    return Err(crate::FeagiDataError::BadParameters(format!("Value cannot be {:?} in a {:?}!", $invalid_zero_value, $friendly_name)));
                }
                Ok(Self { width: x, height: y })
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}<{}, {}>", $friendly_name, self.width, self.height)
            }
        }

    }
}

#[macro_export]
macro_rules! define_xy_percentage_dimensions {
    ($name:ident, $percentage_type:ty, $friendly_name:expr, $doc_string:expr) => {

        #[doc = $doc_string]
        #[derive(Clone, Debug, PartialEq, Copy)]
        pub struct $name {
            pub width: $percentage_type,
            pub height: $percentage_type,
        }

        impl $name {
            pub fn new(x: $percentage_type, y: $percentage_type) -> Result<Self, crate::FeagiDataError> {
                Ok(Self { width: x, height: y })
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}<{}, {}>", $friendly_name, self.width, self.height)
            }
        }
    }
}