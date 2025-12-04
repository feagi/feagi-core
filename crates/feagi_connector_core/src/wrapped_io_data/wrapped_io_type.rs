use std::fmt::write;
use std::mem::discriminant;
use feagi_data_structures::FeagiDataError;
use crate::data_types::descriptors::{ImageFrameProperties, MiscDataDimensions, SegmentedImageFrameProperties};
use crate::data_types::{GazeProperties, ImageFrame, MiscData, Percentage, Percentage2D, Percentage3D, Percentage4D, SegmentedImageFrame, SignedPercentage, SignedPercentage2D, SignedPercentage3D, SignedPercentage4D};
use crate::wrapped_io_data::WrappedIOData;


/// Type descriptor for wrapped I/O data.
///
/// Describes the variant of data stored in a [`WrappedIOData`] enum without holding
/// the actual data. Used for type checking, validation, and creating appropriately-typed
/// blank data instances.
///
/// Some variants (images, misc data) can optionally include dimensional properties
/// to enable efficient memory pre-allocation.
///
/// # Examples
/// ```
/// use feagi_connector_core::wrapped_io_data::WrappedIOType;
///
/// let io_type = WrappedIOType::Percentage;
/// let blank_data = io_type.create_blank_data_of_type().unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum WrappedIOType {
    Boolean,
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
    MiscData(Option<MiscDataDimensions>),
    GazeProperties
}

// NOTE: Due to some variations in some of the types, this isn't practical to turn into a macro.
// This isn't too bad though.

impl WrappedIOType {

    /// Checks if two types are the same variant, ignoring associated data.
    ///
    /// # Example
    /// ```
    /// use feagi_connector_core::wrapped_io_data::WrappedIOType;
    ///
    /// let a = WrappedIOType::Percentage;
    /// let b = WrappedIOType::Percentage;
    /// assert!(WrappedIOType::is_same_variant(&a, &b));
    /// ```
    pub fn is_same_variant(a: &WrappedIOType, b: &WrappedIOType) -> bool {
        discriminant(a) == discriminant(b)
    }

    /// Checks if actual I/O data matches this type descriptor.
    pub fn is_of(&self, io_type: &WrappedIOData) -> bool {
        WrappedIOType::from(io_type) == *self
    }
    
    /// Creates a zero-initialized instance of wrapped data for this type.
    ///
    /// For types with associated properties (images, misc data), those properties
    /// must be provided or this will return an error.
    pub fn create_blank_data_of_type(&self) -> Result<WrappedIOData, FeagiDataError> {
        match self {
            WrappedIOType::Boolean => Ok(WrappedIOData::Boolean(false)),
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
            WrappedIOType::GazeProperties => {
                Ok(WrappedIOData::GazeProperties(GazeProperties::create_default_centered()))
            }
        }
    }
}

impl std::fmt::Display for WrappedIOType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WrappedIOType::Boolean => write!(f, "IOTypeVariant(Boolean)"),
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
            WrappedIOType::GazeProperties => write!(f, "IOTypeVariant(GazeProperties)"),
        }
    }
}

impl From<WrappedIOData> for WrappedIOType {
    fn from(io_type: WrappedIOData) -> Self {
        match io_type {
            WrappedIOData::Boolean(_) => WrappedIOType::Boolean,
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
            WrappedIOData::GazeProperties(_) => WrappedIOType::GazeProperties,
        }
    }
}

impl From<&WrappedIOData> for WrappedIOType {
    fn from(io_type: &WrappedIOData) -> Self {
        match io_type {
            WrappedIOData::Boolean(_) => WrappedIOType::Boolean,
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
            WrappedIOData::GazeProperties(_) => WrappedIOType::GazeProperties,
        }
    }
}
