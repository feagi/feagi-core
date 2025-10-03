use std::mem::discriminant;
use feagi_data_structures::FeagiDataError;
use crate::data_types::descriptors::{ImageFrameProperties, MiscDataDimensions, SegmentedImageFrameProperties};
use crate::data_types::{ImageFrame, MiscData, Percentage, Percentage2D, Percentage3D, Percentage4D, SegmentedImageFrame, SignedPercentage, SignedPercentage2D, SignedPercentage3D, SignedPercentage4D};
use crate::wrapped_io_data::WrappedIOData;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
/// Generally used to describe a type of IO data, such as that found in [WrappedIOData].
/// Some variants which describe various sized data structures, such as images, Optionally can
/// include properties which aid in avoiding memory reallocation.
pub enum WrappedIOType {
    F32, // NOTE: No Feagi Neurons encode floats directly!
    F32_2D,
    F32_3D,
    F32_4D,
    Percentage,
    Percentage_2D,
    Percentage_3D,
    Percentage_4D,
    SignedPercentage,
    SignedPercentage_2D,
    SignedPercentage_3D,
    SignedPercentage_4D,
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
            WrappedIOType::F32_2D => Ok(WrappedIOData::F32_2D((0.0, 0.0))),
            WrappedIOType::F32_3D => Ok(WrappedIOData::F32_3D((0.0, 0.0, 0.0))),
            WrappedIOType::F32_4D => Ok(WrappedIOData::F32_4D((0.0, 0.0, 0.0, 0.0))),

            WrappedIOType::Percentage => Ok(WrappedIOData::Percentage(Percentage::new_from_0_1_unchecked(0.0))),
            WrappedIOType::Percentage_2D => Ok(WrappedIOData::Percentage_2D(Percentage2D::new_identical_percentages(Percentage::new_from_0_1_unchecked(0.0)))),
            WrappedIOType::Percentage_3D => Ok(WrappedIOData::Percentage_3D(Percentage3D::new_identical_percentages(Percentage::new_from_0_1_unchecked(0.0)))),
            WrappedIOType::Percentage_4D => Ok(WrappedIOData::Percentage_4D(Percentage4D::new_identical_percentages(Percentage::new_from_0_1_unchecked(0.0)))),

            WrappedIOType::SignedPercentage => Ok(WrappedIOData::SignedPercentage(SignedPercentage::new_from_m1_1_unchecked(0.0))),
            WrappedIOType::SignedPercentage_2D => Ok(WrappedIOData::SignedPercentage_2D(SignedPercentage2D::new_identical_percentages(SignedPercentage::new_from_m1_1_unchecked(0.0)))),
            WrappedIOType::SignedPercentage_3D => Ok(WrappedIOData::SignedPercentage_3D(SignedPercentage3D::new_identical_percentages(SignedPercentage::new_from_m1_1_unchecked(0.0)))),
            WrappedIOType::SignedPercentage_4D => Ok(WrappedIOData::SignedPercentage_4D(SignedPercentage4D::new_identical_percentages(SignedPercentage::new_from_m1_1_unchecked(0.0)))),

            WrappedIOType::ImageFrame(image_properties) => {
                if image_properties.is_none() {
                    return Err(FeagiDataError::BadParameters("Image frame properties is None! Cannot Created Default Wrapped Data!".into()));
                }
                Ok(WrappedIOData::ImageFrame(ImageFrame::new_from_image_frame_properties(&image_properties.unwrap())?))
            }
            WrappedIOType::SegmentedImageFrame(segmented_image_properties) => {
                if segmented_image_properties.is_none() {
                    return Err(FeagiDataError::BadParameters("Segmented Image frame properties is None! Cannot Created Default Wrapped Data!".into()));
                }
                Ok(WrappedIOData::SegmentedImageFrame(SegmentedImageFrame::from_segmented_image_frame_properties(&segmented_image_properties.unwrap())?))
            }
            WrappedIOType::MiscData(misc_dimensions) => {
                if misc_dimensions.is_none() {
                    return Err(FeagiDataError::BadParameters("Misc Dimensions is None! Cannot Created Default Wrapped Data!".into()));
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
            WrappedIOType::F32_2D => write!(f, "IOTypeVariant(F32_2D)"),
            WrappedIOType::F32_3D => write!(f, "IOTypeVariant(F32_3D)"),
            WrappedIOType::F32_4D => write!(f, "IOTypeVariant(F32_4D)"),
            WrappedIOType::Percentage => write!(f, "IOTypeVariant(Percentage)"),
            WrappedIOType::Percentage_2D => write!(f, "IOTypeVariant(Percentage_2D)"),
            WrappedIOType::Percentage_3D => write!(f, "IOTypeVariant(Percentage_3D)"),
            WrappedIOType::Percentage_4D => write!(f, "IOTypeVariant(Percentage_4D)"),
            WrappedIOType::SignedPercentage => write!(f, "IOTypeVariant(SignedPercentage)"),
            WrappedIOType::SignedPercentage_2D => write!(f, "IOTypeVariant(SignedPercentage_2D)"),
            WrappedIOType::SignedPercentage_3D => write!(f, "IOTypeVariant(SignedPercentage_3D)"),
            WrappedIOType::SignedPercentage_4D => write!(f, "IOTypeVariant(SignedPercentage_4D)"),
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
            WrappedIOData::F32_2D(_) => WrappedIOType::F32_2D,
            WrappedIOData::F32_3D(_) => WrappedIOType::F32_3D,
            WrappedIOData::F32_4D(_) => WrappedIOType::F32_4D,
            WrappedIOData::Percentage(_) => WrappedIOType::Percentage,
            WrappedIOData::Percentage_2D(_) => WrappedIOType::Percentage_2D,
            WrappedIOData::Percentage_3D(_) => WrappedIOType::Percentage_3D,
            WrappedIOData::Percentage_4D(_) => WrappedIOType::Percentage_4D,
            WrappedIOData::SignedPercentage(_) => WrappedIOType::SignedPercentage,
            WrappedIOData::SignedPercentage_2D(_) => WrappedIOType::SignedPercentage_2D,
            WrappedIOData::SignedPercentage_3D(_) => WrappedIOType::SignedPercentage_3D,
            WrappedIOData::SignedPercentage_4D(_) => WrappedIOType::SignedPercentage_4D,
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
            WrappedIOData::F32_2D(_) => WrappedIOType::F32_2D,
            WrappedIOData::F32_3D(_) => WrappedIOType::F32_3D,
            WrappedIOData::F32_4D(_) => WrappedIOType::F32_4D,
            WrappedIOData::Percentage(_) => WrappedIOType::Percentage,
            WrappedIOData::Percentage_2D(_) => WrappedIOType::Percentage_2D,
            WrappedIOData::Percentage_3D(_) => WrappedIOType::Percentage_3D,
            WrappedIOData::Percentage_4D(_) => WrappedIOType::Percentage_4D,
            WrappedIOData::SignedPercentage(_) => WrappedIOType::SignedPercentage,
            WrappedIOData::SignedPercentage_2D(_) => WrappedIOType::SignedPercentage_2D,
            WrappedIOData::SignedPercentage_3D(_) => WrappedIOType::SignedPercentage_3D,
            WrappedIOData::SignedPercentage_4D(_) => WrappedIOType::SignedPercentage_4D,
            WrappedIOData::ImageFrame(image) => WrappedIOType::ImageFrame(Some(image.get_image_frame_properties())),
            WrappedIOData::SegmentedImageFrame(segments) => WrappedIOType::SegmentedImageFrame(Some(segments.get_segmented_image_frame_properties())),
            WrappedIOData::MiscData(dimensions) => {WrappedIOType::MiscData(Some(dimensions.get_dimensions()))}
        }
    }
}
