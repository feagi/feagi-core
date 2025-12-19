use crate::data_types::{
    GazeProperties, ImageFrame, MiscData, Percentage, Percentage2D, Percentage3D, Percentage4D,
    SegmentedImageFrame, SignedPercentage, SignedPercentage2D, SignedPercentage3D,
    SignedPercentage4D,
};
use feagi_data_structures::FeagiDataError;

/// Internal macro to define the WrappedIOData enum with automatic trait implementations.
///
/// Generates the enum and From/TryFrom conversions for all variants.
macro_rules! define_wrapped_io_data_enum {
    ( $( $enum_type:ident : $data_type:ty => $friendly_name:expr),+ $(,)? ) => {
        /// Type-safe wrapper for heterogeneous I/O data.
        ///
        /// Enables storing different types of sensor/motor data in a single enum,
        /// supporting dynamic dispatch while maintaining type safety through pattern matching.
        ///
        /// Provides conversions from/to concrete types via From/TryFrom traits.
        ///
        /// # Examples
        /// ```
        /// use feagi_sensorimotor::wrapped_io_data::WrappedIOData;
        /// use feagi_sensorimotor::data_types::Percentage;
        ///
        /// let percentage = Percentage::new_from_0_1(0.75).unwrap();
        /// let wrapped: WrappedIOData = percentage.into();
        ///
        /// // Extract back to concrete type
        /// let extracted: Percentage = wrapped.try_into().unwrap();
        /// ```
        #[derive(Debug, Clone)]
        #[allow(non_camel_case_types)]
        /// Due to Rust's memory management, WrappedIOData is used to pass around various data structures around.
        pub enum WrappedIOData
        {
            $( $enum_type($data_type), )*
        }

        impl std::fmt::Display for WrappedIOData {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $( WrappedIOData::$enum_type(data) => write!(f, concat!("IOTypeData(", $friendly_name, ")"), data), )*
                }
            }
        }

        $(
        impl From<$data_type> for WrappedIOData {
            fn from(value: $data_type) -> Self {WrappedIOData::$enum_type(value)}
        }

        impl TryFrom<WrappedIOData> for $data_type {
            type Error = FeagiDataError;

            fn try_from(value: WrappedIOData) -> Result<Self, Self::Error> {
                match value {
                    WrappedIOData::$enum_type(data) => Ok(data),
                    _ => Err(FeagiDataError::BadParameters(format!("This variable is not a {}!", $friendly_name)).into()),
                }
            }
        }

        impl TryFrom<&WrappedIOData> for $data_type {
            type Error = FeagiDataError;

            fn try_from(value: &WrappedIOData) -> Result<Self, Self::Error> {
                match value {
                    WrappedIOData::$enum_type(data) => Ok(data.clone()),
                    _ => Err(FeagiDataError::BadParameters(format!("This variable is not a {}!", $friendly_name)).into()),
                }
            }
        }

        impl<'a> TryFrom<&'a WrappedIOData> for &'a $data_type {
            type Error = FeagiDataError;

            fn try_from(value: &'a WrappedIOData) -> Result<Self, Self::Error> {
                match value {
                    WrappedIOData::$enum_type(data) => Ok(data),
                    _ => Err(FeagiDataError::BadParameters(format!("This variable is not a {}!", $friendly_name)).into()),
                }
            }
        }

        impl<'a> TryFrom<&'a mut WrappedIOData> for &'a mut $data_type {
            type Error = FeagiDataError;

            fn try_from(value: &'a mut WrappedIOData) -> Result<Self, Self::Error> {
                match value {
                    WrappedIOData::$enum_type(data) => Ok(data),
                    _ => Err(FeagiDataError::BadParameters(format!("This variable is not a {}!", $friendly_name)).into()),
                }
            }
        }
        )*


    }
}

define_wrapped_io_data_enum!(
    Boolean: bool => "{}",
    Percentage: Percentage => "{}",
    Percentage_2D: Percentage2D => "{}",
    Percentage_3D: Percentage3D => "{}",
    Percentage_4D: Percentage4D => "{}",
    SignedPercentage: SignedPercentage => "{}",
    SignedPercentage_2D: SignedPercentage2D => "{}",
    SignedPercentage_3D: SignedPercentage3D => "{}",
    SignedPercentage_4D: SignedPercentage4D => "{}",
    ImageFrame: ImageFrame => "{}",
    SegmentedImageFrame: SegmentedImageFrame => "{}",
    MiscData: MiscData => "{}",
    GazeProperties: GazeProperties => "{}",
);
