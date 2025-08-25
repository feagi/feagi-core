use crate::data::image_descriptors::{ImageFrameProperties, SegmentedImageFrameProperties};
use crate::wrapped_io_data::WrappedIOData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WrappedIOType {
    F32,
    F32Normalized0To1,
    F32NormalizedM1To1,
    ImageFrame(Option<ImageFrameProperties>),
    SegmentedImageFrame(Option<SegmentedImageFrameProperties>),
}

impl WrappedIOType {
    pub fn is_of(&self, io_type: &WrappedIOData) -> bool {
        WrappedIOType::from(io_type) == *self
    }
}

impl std::fmt::Display for WrappedIOType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WrappedIOType::F32 => write!(f, "IOTypeVariant(F32)"),
            WrappedIOType::F32Normalized0To1 => write!(f, "IOTypeVariant(F32 [Normalized 0<->1])"),
            WrappedIOType::F32NormalizedM1To1 => write!(f, "IOTypeVariant(F32 [Normalized -1<->1])"),
            WrappedIOType::ImageFrame(image_properties) => {
                let s: String = match image_properties {
                    Some(properties) => format!("ImageFrame({})", properties.to_string()),
                    None => "ImageFrame(No Requirements)".to_string(),
                };
                write!(f, "{}", s)
            }
            WrappedIOType::SegmentedImageFrame(segment_properties) => {
                let s: String = match segment_properties {
                    None => "No Requirements".to_string(),
                    Some(properties) => {
                        format!("SegmentedImageFrame({})", properties.to_string())
                    }
                };
                write!(f, "SegmentedImageFrame({})", s)
            }
        }
    }
}

impl From<WrappedIOData> for WrappedIOType {
    fn from(io_type: WrappedIOData) -> Self {
        match io_type {
            WrappedIOData::F32(_) => WrappedIOType::F32,
            WrappedIOData::F32Normalized0To1(_) => WrappedIOType::F32Normalized0To1,
            WrappedIOData::F32NormalizedM1To1(_) => WrappedIOType::F32NormalizedM1To1,
            WrappedIOData::ImageFrame(image) => WrappedIOType::ImageFrame(Some(image.get_image_frame_properties())),
            WrappedIOData::SegmentedImageFrame(segments) => WrappedIOType::SegmentedImageFrame(Some(segments.get_segmented_image_frame_properties())),
        }
    }
}

impl From<&WrappedIOData> for WrappedIOType {
    fn from(io_type: &WrappedIOData) -> Self {
        match io_type {
            WrappedIOData::F32(_) => WrappedIOType::F32,
            WrappedIOData::F32Normalized0To1(_) => WrappedIOType::F32Normalized0To1,
            WrappedIOData::F32NormalizedM1To1(_) => WrappedIOType::F32NormalizedM1To1,
            WrappedIOData::ImageFrame(image) => WrappedIOType::ImageFrame(Some(image.get_image_frame_properties())),
            WrappedIOData::SegmentedImageFrame(segments) => WrappedIOType::SegmentedImageFrame(Some(segments.get_segmented_image_frame_properties())),
        }
    }
}