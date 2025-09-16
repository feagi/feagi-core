use crate::FeagiDataError;
use crate::data::{ImageFrame, MiscData, SegmentedImageFrame};

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
    F32Normalized0To1(f32),
    F32NormalizedM1To1(f32),
    ImageFrame(ImageFrame),
    SegmentedImageFrame(SegmentedImageFrame),
    MiscData(MiscData)
}


impl WrappedIOData {
    pub fn new_f32(value: f32) -> Result<Self, FeagiDataError> {
        if value.is_nan() || value.is_infinite() {
            return Err(FeagiDataError::BadParameters("Input value cannot be NaN or Infinite!".into()).into());
        }
        Ok(Self::F32(value))
    }
    pub fn new_0_1_f32(value: f32) -> Result<Self, FeagiDataError> {
        if value.is_nan() || value.is_infinite() {
            return Err(FeagiDataError::BadParameters("Input value cannot be NaN or Infinite!".into()).into());
        }
        if value < 0.0 || value > 1.0 {
            return Err(FeagiDataError::BadParameters("Input value must be between 0 and 1!".into()).into());
        }
        Ok(Self::F32Normalized0To1(value))
    }


    pub fn new_m1_1_f32(value: f32) -> Result<Self, FeagiDataError> {
        if value.is_nan() || value.is_infinite() {
            return Err(FeagiDataError::BadParameters("Input value cannot be NaN or Infinite!".into()).into());
        }
        if value < -1.0 || value > 1.0 {
            return Err(FeagiDataError::BadParameters("Input value must be between -1 and 1!".into()).into());
        }
        Ok(Self::F32NormalizedM1To1(value))
    }
}

impl std::fmt::Display for WrappedIOData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WrappedIOData::F32(float) => write!(f, "IOTypeData(f32({}))", float),
            WrappedIOData::F32Normalized0To1(float) => write!(f, "IOTypeData(f32[Normalized 0<->1]({}))", float),
            WrappedIOData::F32NormalizedM1To1(float) => write!(f, "IOTypeData(f32[Normalized -1<->1]({}))", float),
            WrappedIOData::ImageFrame(frame) => write!(f, "IOTypeData({})", frame),
            WrappedIOData::SegmentedImageFrame(frame) => write!(f, "IOTypeData({})", frame),
            WrappedIOData::MiscData(misc_data) => write!(f, "IOTypeData({})", misc_data),
        }
    }
}

//region Try Conversions

implement_data_conversions!(ImageFrame, ImageFrame, "image_frame");
implement_data_conversions!(SegmentedImageFrame, SegmentedImageFrame, "segmented_image_frame");
implement_data_conversions!(MiscData, MiscData, "misc_data");



impl TryFrom<WrappedIOData> for f32 {
    type Error = FeagiDataError;
    fn try_from(value: WrappedIOData) -> Result<Self, Self::Error> {
        match value {
            WrappedIOData::F32(float) => Ok(float),
            WrappedIOData::F32Normalized0To1(float) => Ok(float),
            WrappedIOData::F32NormalizedM1To1(float) => Ok(float),
            _ => Err(FeagiDataError::BadParameters("This variable is not a f32 type value!".into()).into()),
        }
    }
}

impl TryFrom<&WrappedIOData> for f32 {
    type Error = FeagiDataError;
    fn try_from(value: &WrappedIOData) -> Result<Self, Self::Error> {
        match value {
            WrappedIOData::F32(float) => Ok(*float),
            WrappedIOData::F32Normalized0To1(float) => Ok(*float),
            WrappedIOData::F32NormalizedM1To1(float) => Ok(*float),
            _ => Err(FeagiDataError::BadParameters("This variable is not a f32 type value!".into()).into()),
        }
    }
}


//endregion