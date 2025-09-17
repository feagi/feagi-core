use crate::FeagiDataError;
use crate::data::{ImageFrame, MiscData, Percentage, Percentage4D, SegmentedImageFrame, SignedPercentage};

macro_rules! implement_data_conversions {
    ($data_type:ident, $enum_type:ident, $friendly_name:expr) => {

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

    };
}

#[derive(Debug, Clone)]
pub enum WrappedIOData
{
    F32(f32),
    Percentage(Percentage),
    SignedPercentage(SignedPercentage),
    Percentage4D(Percentage4D),
    ImageFrame(ImageFrame),
    SegmentedImageFrame(SegmentedImageFrame),
    MiscData(MiscData)
}


impl std::fmt::Display for WrappedIOData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WrappedIOData::F32(float) => write!(f, "IOTypeData(f32({}))", float),
            WrappedIOData::Percentage(percentage) => write!(f, "IOTypeData({})", percentage),
            WrappedIOData::SignedPercentage(signed_percentage) => write!(f, "IOTypeData({})", signed_percentage),
            WrappedIOData::Percentage4D(percentage4_d) => write!(f, "IOTypeData({})", percentage4_d),
            WrappedIOData::ImageFrame(frame) => write!(f, "IOTypeData({})", frame),
            WrappedIOData::SegmentedImageFrame(frame) => write!(f, "IOTypeData({})", frame),
            WrappedIOData::MiscData(misc_data) => write!(f, "IOTypeData({})", misc_data),
        }
    }
}

//region Try Conversions

implement_data_conversions!(Percentage, Percentage, "percentage");
implement_data_conversions!(SignedPercentage, SignedPercentage, "signed_percentage");
implement_data_conversions!(Percentage4D, Percentage4D, "percentage_4d");
implement_data_conversions!(ImageFrame, ImageFrame, "image_frame");
implement_data_conversions!(SegmentedImageFrame, SegmentedImageFrame, "segmented_image_frame");
implement_data_conversions!(MiscData, MiscData, "misc_data");


impl TryFrom<WrappedIOData> for f32 {
    type Error = FeagiDataError;
    fn try_from(value: WrappedIOData) -> Result<Self, Self::Error> {
        match value {
            WrappedIOData::F32(float) => Ok(float),
            WrappedIOData::Percentage(float) => Ok(float.into()),
            WrappedIOData::SignedPercentage(float) => Ok(float.into()),
            _ => Err(FeagiDataError::BadParameters("This variable is not a f32 type value!".into()).into()),
        }
    }
}

impl TryFrom<&WrappedIOData> for f32 {
    type Error = FeagiDataError;
    fn try_from(value: &WrappedIOData) -> Result<Self, Self::Error> {
        match value {
            WrappedIOData::F32(float) => Ok(*float),
            WrappedIOData::Percentage(float) => Ok(float.into()),
            WrappedIOData::SignedPercentage(float) => Ok(float.into()),
            _ => Err(FeagiDataError::BadParameters("This variable is not a f32 type value!".into()).into()),
        }
    }
}


//endregion