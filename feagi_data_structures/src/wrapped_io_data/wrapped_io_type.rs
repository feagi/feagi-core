use std::mem::discriminant;
use crate::data::descriptors::{ImageFrameProperties, MiscDataDimensions, SegmentedImageFrameProperties};
use crate::data::{ImageFrame, MiscData, Percentage, Percentage4D, SegmentedImageFrame, SignedPercentage};
use crate::FeagiDataError;
use crate::wrapped_io_data::WrappedIOData;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WrappedIOType {
    F32, // Used for neither sensors or motors
    Percentage, // used for both
    SignedPercentage,
    Percentage4D,
    ImageFrame(Option<ImageFrameProperties>),
    SegmentedImageFrame(Option<SegmentedImageFrameProperties>),
    MiscData(Option<MiscDataDimensions>)
}



impl WrappedIOType {

    pub fn is_same_variant(a: &WrappedIOType, b: &WrappedIOType) -> bool {
        discriminant(a) == discriminant(b)
    }

    pub fn is_of(&self, io_type: &WrappedIOData) -> bool {
        WrappedIOType::from(io_type) == *self
    }
    
    pub fn create_blank_data_of_type(&self) -> Result<WrappedIOData, FeagiDataError> {
        match self {
            WrappedIOType::F32 => Ok(WrappedIOData::F32(0.0)),
            WrappedIOType::Percentage => Ok(WrappedIOData::Percentage(Percentage::new_from_0_1_unchecked(0.0))),
            WrappedIOType::SignedPercentage => Ok(WrappedIOData::SignedPercentage(SignedPercentage::new_from_m1_1_unchecked(0.0))),
            WrappedIOType::Percentage4D => Ok(WrappedIOData::Percentage4D(Percentage4D::new(Percentage::new_from_0_1_unchecked(0.0), Percentage::new_from_0_1_unchecked(0.0), Percentage::new_from_0_1_unchecked(0.0), Percentage::new_from_0_1_unchecked(0.0)))),
            WrappedIOType::ImageFrame(image_properties) => {
                if image_properties.is_none() {
                    return Err(FeagiDataError::BadParameters(format!("Image frame properties is None! Cannot Created Default Wrapped Data!")));
                }
                Ok(WrappedIOData::ImageFrame(ImageFrame::new_from_image_frame_properties(&image_properties.unwrap())?))
            }
            WrappedIOType::SegmentedImageFrame(segmented_image_properties) => {
                if segmented_image_properties.is_none() {
                    return Err(FeagiDataError::BadParameters(format!("Segmented Image frame properties is None! Cannot Created Default Wrapped Data!")));
                }
                Ok(WrappedIOData::SegmentedImageFrame(SegmentedImageFrame::from_segmented_image_frame_properties(&segmented_image_properties.unwrap())?))
            }
            WrappedIOType::MiscData(misc_dimensions) => {
                if misc_dimensions.is_none() {
                    return Err(FeagiDataError::BadParameters(format!("M<isc Dimensions is None! Cannot Created Default Wrapped Data!")));
                }
                Ok(WrappedIOData::MiscData(MiscData::new(&misc_dimensions.unwrap())?))
            }
        }
    }
}

impl std::fmt::Display for WrappedIOType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WrappedIOType::F32 => write!(f, "IOTypeVariant(F32)"),
            WrappedIOType::Percentage => write!(f, "IOTypeVariant(Percentage)"),
            WrappedIOType::SignedPercentage => write!(f, "IOTypeVariant(SignedPercentage)"),
            WrappedIOType::Percentage4D => write!(f, "IOTypeVariant(Percentage4D)"),
            WrappedIOType::ImageFrame(image_properties) => {
                let s: String = match image_properties {
                    Some(properties) => properties.to_string(),
                    None => "ImageFrame(No Requirements)".to_string(),
                };
                write!(f, "{}", s)
            }
            WrappedIOType::SegmentedImageFrame(segment_properties) => {
                let s: String = match segment_properties {
                    None => "No Requirements".to_string(),
                    Some(properties) => {
                        properties.to_string()
                    }
                };
                write!(f, "SegmentedImageFrame({})", s)
            }
            WrappedIOType::MiscData(dimensions) => {
                let s: String = match dimensions {
                    Some(dimensions) => dimensions.to_string(),
                    None => "No Requirements".to_string(),
                };
                write!(f, "Misc({})", s)
            }
        }
    }
}

impl From<WrappedIOData> for WrappedIOType {
    fn from(io_type: WrappedIOData) -> Self {
        match io_type {
            WrappedIOData::F32(_) => WrappedIOType::F32,
            WrappedIOData::Percentage(_) => WrappedIOType::Percentage,
            WrappedIOData::SignedPercentage(_) => WrappedIOType::SignedPercentage,
            WrappedIOData::Percentage4D(_) => WrappedIOType::Percentage4D,
            WrappedIOData::ImageFrame(image) => WrappedIOType::ImageFrame(Some(image.get_image_frame_properties())),
            WrappedIOData::SegmentedImageFrame(segments) => WrappedIOType::SegmentedImageFrame(Some(segments.get_segmented_image_frame_properties())),
            WrappedIOData::MiscData(dimensions) => {WrappedIOType::MiscData(Some(dimensions.get_dimensions()))}
        }
    }
}

impl From<&WrappedIOData> for WrappedIOType {
    fn from(io_type: &WrappedIOData) -> Self {
        match io_type {
            WrappedIOData::F32(_) => WrappedIOType::F32,
            WrappedIOData::Percentage(_) => WrappedIOType::Percentage,
            WrappedIOData::SignedPercentage(_) => WrappedIOType::SignedPercentage,
            WrappedIOData::Percentage4D(_) => WrappedIOType::Percentage4D,
            WrappedIOData::ImageFrame(image) => WrappedIOType::ImageFrame(Some(image.get_image_frame_properties())),
            WrappedIOData::SegmentedImageFrame(segments) => WrappedIOType::SegmentedImageFrame(Some(segments.get_segmented_image_frame_properties())),
            WrappedIOData::MiscData(dimensions) => {WrappedIOType::MiscData(Some(dimensions.get_dimensions()))}
        }
    }
}
