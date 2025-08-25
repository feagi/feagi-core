use crate::FeagiDataError;
use crate::data::{ImageFrame, SegmentedImageFrame};

#[derive(Debug, Clone)]
pub enum WrappedIOData
{
    F32(f32),
    F32Normalized0To1(f32),
    F32NormalizedM1To1(f32),
    ImageFrame(ImageFrame),
    SegmentedImageFrame(SegmentedImageFrame),
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
        }
    }
}

//region Try Conversions

impl From<ImageFrame> for WrappedIOData {
    fn from(value: ImageFrame) -> Self {
        WrappedIOData::ImageFrame(value)
    }
}

impl From<SegmentedImageFrame> for WrappedIOData {
    fn from(value: SegmentedImageFrame) -> Self {
        WrappedIOData::SegmentedImageFrame(value)
    }
}

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


impl TryFrom<WrappedIOData> for ImageFrame {
    type Error = FeagiDataError;

    fn try_from(value: WrappedIOData) -> Result<Self, Self::Error> {
        match value {
            WrappedIOData::ImageFrame(image) => Ok(image),
            _ => Err(FeagiDataError::BadParameters("This variable is not a Image Frame!".into()).into()),
        }
    }
}

impl<'a> TryFrom<&'a WrappedIOData> for &'a ImageFrame {
    type Error = FeagiDataError;

    fn try_from(value: &'a WrappedIOData) -> Result<Self, Self::Error> {
        match value {
            WrappedIOData::ImageFrame(image_ref) => Ok(image_ref),
            _ => Err(FeagiDataError::BadParameters("This variable is not a Image Frame!".into()).into()),
        }
    }
}

impl<'a> TryFrom<&'a mut WrappedIOData> for &'a mut ImageFrame {
    type Error = FeagiDataError;

    fn try_from(value: &'a mut WrappedIOData) -> Result<Self, Self::Error> {
        match value {
            WrappedIOData::ImageFrame(image_ref) => Ok(image_ref),
            _ => Err(FeagiDataError::BadParameters("This variable is not a Image Frame!".into()).into()),
        }
    }
}

impl TryFrom<WrappedIOData> for SegmentedImageFrame {
    type Error = FeagiDataError;

    fn try_from(value: WrappedIOData) -> Result<Self, Self::Error> {
        match value {
            WrappedIOData::SegmentedImageFrame(image_ref) => Ok(image_ref),
            _ => Err(FeagiDataError::BadParameters("This variable is not a SegmentedImageFrame!".into()).into()),
        }
    }
}

impl<'a> TryFrom<&'a WrappedIOData> for &'a SegmentedImageFrame {
    type Error = FeagiDataError;

    fn try_from(value: &'a WrappedIOData) -> Result<Self, Self::Error> {
        match value {
            WrappedIOData::SegmentedImageFrame(image_ref) => Ok(image_ref),
            _ => Err(FeagiDataError::BadParameters("This variable is not a Segmented Image Frame!".into()).into()),
        }
    }
}

impl<'a> TryFrom<&'a mut WrappedIOData> for &'a mut SegmentedImageFrame {
    type Error = FeagiDataError;

    fn try_from(value: &'a mut WrappedIOData) -> Result<Self, Self::Error> {
        match value {
            WrappedIOData::SegmentedImageFrame(image_ref) => Ok(image_ref),
            _ => Err(FeagiDataError::BadParameters("This variable is not a Segmented Image Frame!".into()).into()),
        }
    }
}

//endregion