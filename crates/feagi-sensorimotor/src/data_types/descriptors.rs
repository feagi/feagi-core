//! Descriuptors and Parameter structures for FEAGI Data.
//!
//! This module provides data structures and enums for describing dat properties

use super::{ImageFrame, SegmentedImageFrame};
use feagi_data_structures::genomic::cortical_area::descriptors::CorticalChannelDimensions;
// NeuronDepth is used in macro expansion
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::{
    define_xy_coordinates, define_xy_dimensions, define_xyz_dimensions, define_xyz_mapping,
};
use std::fmt::Display;

//region Images

//region Image XY

define_xy_coordinates!(ImageXYPoint, u32, "ImageXYPoint", "Represents a coordinate on an image. +x goes to the right, +y goes downward. (0,0) is in the top_left");

define_xy_dimensions!(
    ImageXYResolution,
    u32,
    "ImageXYResolution",
    0,
    "Describes the resolution of the image (width and height)"
);

//endregion

//region Image XYZ

define_xyz_dimensions!(ImageXYZDimensions, u32, "ImageXYZDimensions", 0, "Describes the 3D dimensions of an image, with the 3rd dimension being the number of color channels");

//endregion

//region Segmented Image XY Resolutions
/// Target resolutions for each of the nine segments in a segmented vision frame
///
/// This structure stores the desired output resolution for each of the segments
/// in a grid arrangement (3x3): corners, edges, and center.
#[derive(PartialEq, Clone, Copy, Debug, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SegmentedXYImageResolutions {
    pub lower_left: ImageXYResolution,
    pub lower_middle: ImageXYResolution,
    pub lower_right: ImageXYResolution,
    pub middle_left: ImageXYResolution,
    pub center: ImageXYResolution,
    pub middle_right: ImageXYResolution,
    pub upper_left: ImageXYResolution,
    pub upper_middle: ImageXYResolution,
    pub upper_right: ImageXYResolution,
}

impl SegmentedXYImageResolutions {
    pub fn new(
        lower_left: ImageXYResolution,
        lower_middle: ImageXYResolution,
        lower_right: ImageXYResolution,
        middle_left: ImageXYResolution,
        center: ImageXYResolution,
        middle_right: ImageXYResolution,
        upper_left: ImageXYResolution,
        upper_middle: ImageXYResolution,
        upper_right: ImageXYResolution,
    ) -> SegmentedXYImageResolutions {
        SegmentedXYImageResolutions {
            lower_left,
            lower_middle,
            lower_right,
            middle_left,
            center,
            middle_right,
            upper_left,
            upper_middle,
            upper_right,
        }
    }

    /// Creates a SegmentedVisionTargetResolutions with uniform peripheral segment sizes.
    ///
    /// This convenience method creates a configuration where all eight peripheral segments
    /// have the same resolution, while the center segment can have a different resolution.
    ///
    /// # Arguments
    ///
    /// * `center_width_height` - Resolution for the center segment as (width, height)
    /// * `peripheral_width_height` - Resolution for all peripheral segments as (width, height)
    ///
    /// # Returns
    ///
    /// A Result containing either:
    /// - Ok(SegmentedVisionTargetResolutions) if all resolutions are valid (non-zero)
    /// - Err(DataProcessingError) if any resolution has zero width or height
    pub fn create_with_same_sized_peripheral(
        center_resolution: ImageXYResolution,
        peripheral_resolutions: ImageXYResolution,
    ) -> SegmentedXYImageResolutions {
        SegmentedXYImageResolutions::new(
            peripheral_resolutions,
            peripheral_resolutions,
            peripheral_resolutions,
            peripheral_resolutions,
            center_resolution,
            peripheral_resolutions,
            peripheral_resolutions,
            peripheral_resolutions,
            peripheral_resolutions,
        )
    }

    pub fn as_ordered_array(&self) -> [&ImageXYResolution; 9] {
        [
            &self.lower_left,
            &self.lower_middle,
            &self.lower_right,
            &self.middle_left,
            &self.center,
            &self.middle_right,
            &self.upper_left,
            &self.upper_middle,
            &self.upper_right,
        ]
    }
}

impl Display for SegmentedXYImageResolutions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "LowerLeft:{}, LowerMiddle:{}, LowerRight:{}, MiddleLeft:{}, Center:{}, MiddleRight:{}, TopLeft:{}, TopMiddle:{}, TopRight:{}",
               self.lower_left, self.lower_middle, self.lower_right, self.middle_left, self.center, self.middle_right, self.upper_left, self.upper_middle, self.upper_right)
    }
}

//endregion

//region Enums

/// Represents the color space of an image.
///
/// This enum defines the possible color spaces:
/// - Linear: Linear color space
/// - Gamma: Gamma-corrected color space
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ColorSpace {
    Linear,
    Gamma,
}

impl Display for ColorSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ColorSpace::Linear => write!(f, "Linear"),
            ColorSpace::Gamma => write!(f, "Gamma"),
        }
    }
}

/// Represents the color channel format of an image.
///
/// This enum defines the possible color channel configurations for an image:
/// - GrayScale: Single channel (grayscale, or red)
/// - RG: Two channels (red, green)
/// - RGB: Three channels (red, green, blue)
/// - RGBA: Four channels (red, green, blue, alpha)
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ColorChannelLayout {
    GrayScale = 1, // R
    RG = 2,
    RGB = 3,
    RGBA = 4,
}

impl Display for ColorChannelLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ColorChannelLayout::GrayScale => write!(f, "ChannelLayout(GrayScale)"),
            ColorChannelLayout::RG => write!(f, "ChannelLayout(RedGreen)"),
            ColorChannelLayout::RGB => write!(f, "ChannelLayout(RedGreenBlue)"),
            ColorChannelLayout::RGBA => write!(f, "ChannelLayout(RedGreenBlueAlpha)"),
        }
    }
}

impl TryFrom<usize> for ColorChannelLayout {
    type Error = FeagiDataError;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ColorChannelLayout::GrayScale),
            2 => Ok(ColorChannelLayout::RG),
            3 => Ok(ColorChannelLayout::RGB),
            4 => Ok(ColorChannelLayout::RGBA),
            _ => Err(FeagiDataError::BadParameters(format!(
                "No Channel Layout has {} channels! Acceptable values are 1,2,3,4!",
                value
            ))
            .into()),
        }
    }
}

impl TryFrom<image::ColorType> for ColorChannelLayout {
    type Error = FeagiDataError;
    fn try_from(value: image::ColorType) -> Result<Self, Self::Error> {
        match value {
            image::ColorType::L8 => Ok(ColorChannelLayout::GrayScale),
            image::ColorType::La8 => Ok(ColorChannelLayout::RG),
            image::ColorType::Rgb8 => Ok(ColorChannelLayout::RGB),
            image::ColorType::Rgba8 => Ok(ColorChannelLayout::RGBA),
            _ => Err(FeagiDataError::BadParameters(
                "Unsupported image color!".to_string(),
            )),
        }
    }
}

impl From<ColorChannelLayout> for usize {
    fn from(value: ColorChannelLayout) -> usize {
        value as usize
    }
}

impl From<ColorChannelLayout> for u32 {
    fn from(value: ColorChannelLayout) -> u32 {
        value as u32
    }
}

/// Represents the memory layout of an image array.
///
/// This enum defines the possible memory layouts for image data:
/// - HeightsWidthsChannels: Row-major format (default)
/// - ChannelsHeightsWidths: Common in machine learning
/// - WidthsHeightsChannels: Cartesian format
/// - HeightsChannelsWidths: Alternative format
/// - ChannelsWidthsHeights: Alternative format
/// - WidthsChannelsHeights: Alternative format
#[derive(Debug, PartialEq, Clone, Copy, Hash, serde::Serialize, serde::Deserialize)]
pub enum MemoryOrderLayout {
    HeightsWidthsChannels, // default, also called row major
    ChannelsHeightsWidths, // common in machine learning
    WidthsHeightsChannels, // cartesian, the best one
    HeightsChannelsWidths,
    ChannelsWidthsHeights,
    WidthsChannelsHeights,
}

impl Display for MemoryOrderLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MemoryOrderLayout::HeightsWidthsChannels => write!(f, "HeightsWidthsChannels"),
            MemoryOrderLayout::ChannelsHeightsWidths => write!(f, "ChannelsHeightsWidths"),
            MemoryOrderLayout::WidthsHeightsChannels => write!(f, "WidthsHeightsChannels"),
            MemoryOrderLayout::HeightsChannelsWidths => write!(f, "HeightsChannelsWidths"),
            MemoryOrderLayout::ChannelsWidthsHeights => write!(f, "ChannelsWidthsHeights"),
            MemoryOrderLayout::WidthsChannelsHeights => write!(f, "WidthsChannelsHeights"),
        }
    }
}
//endregion

//region Image Frame Properties

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ImageFrameProperties {
    image_resolution: ImageXYResolution,
    color_space: ColorSpace,
    color_channel_layout: ColorChannelLayout,
}

impl ImageFrameProperties {
    /// Creates a new ImageFrameProperties instance.
    ///
    /// # Arguments
    ///
    /// * `image_resolution` - The image dimensions
    /// * `color_space` - The color space (Linear or Gamma-corrected)
    /// * `color_channel_layout` - The channel configuration (Grayscale, RGB, RGBA, etc.)
    ///
    /// # Returns
    ///
    /// A new ImageFrameProperties instance with the specified configuration.
    pub fn new(
        image_resolution: ImageXYResolution,
        color_space: ColorSpace,
        color_channel_layout: ColorChannelLayout,
    ) -> Result<Self, FeagiDataError> {
        Ok(ImageFrameProperties {
            image_resolution,
            color_space,
            color_channel_layout,
        })
    }

    /// Verifies that an image frame matches these properties.
    ///
    /// Checks if the given image frame has the same resolution, color space,
    /// and channel layout as specified in these properties.
    ///
    /// # Arguments
    ///
    /// * `image` - The image frame to verify against these properties
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the image frame matches these properties
    /// * `Err(FeagiDataError)` if any property doesn't match
    ///
    /// # Errors
    ///
    /// Returns an error with a descriptive message if:
    /// - The resolution doesn't match
    /// - The color space doesn't match
    /// - The channel layout doesn't match
    pub fn verify_image_frame_matches_properties(
        &self,
        image_frame: &ImageFrame,
    ) -> Result<(), FeagiDataError> {
        if image_frame.get_xy_resolution() != self.image_resolution {
            return Err(FeagiDataError::BadParameters(
                format! {"Expected resolution of {} but received an image with resolution of {}!",
                self.image_resolution, image_frame.get_xy_resolution()},
            )
            .into());
        }
        if image_frame.get_color_space() != &self.color_space {
            return Err(FeagiDataError::BadParameters(format!(
                "Expected color space of {}, but got image with color space of {}!",
                self.color_space.to_string(),
                self.color_space.to_string()
            ))
            .into());
        }
        if image_frame.get_channel_layout() != &self.color_channel_layout {
            return Err(FeagiDataError::BadParameters(format!("Expected color channel layout of {}, but got image with color channel layout of {}!", self.color_channel_layout.to_string(), self.color_channel_layout.to_string())).into());
        }
        Ok(())
    }

    /// Returns the XY resolution.
    ///
    /// # Returns
    ///
    /// An ImageXYResolution
    pub fn get_image_resolution(&self) -> ImageXYResolution {
        self.image_resolution
    }

    /// Returns the color space.
    ///
    /// # Returns
    ///
    /// The ColorSpace enum value (Linear or Gamma).
    pub fn get_color_space(&self) -> ColorSpace {
        self.color_space
    }

    /// Returns the color channel layout.
    ///
    /// # Returns
    ///
    /// The ChannelLayout enum value (Grayscale, RGB, RGBA, etc.).
    pub fn get_color_channel_layout(&self) -> ColorChannelLayout {
        self.color_channel_layout
    }

    pub fn get_number_of_channels(&self) -> usize {
        self.color_channel_layout.into()
    }

    pub fn get_number_of_samples(&self) -> usize {
        self.image_resolution.width as usize
            * self.image_resolution.height as usize
            * self.get_number_of_channels()
    }
}

impl Display for ImageFrameProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = format!(
            "ImageFrameProperties({}, {}, {})",
            self.image_resolution,
            self.color_space.to_string(),
            self.color_channel_layout.to_string()
        );
        write!(f, "{}", s)
    }
}

//endregion

//region Segmented Image Frame Properties

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SegmentedImageFrameProperties {
    segment_xy_resolutions: SegmentedXYImageResolutions,
    center_color_channel: ColorChannelLayout,
    peripheral_color_channels: ColorChannelLayout,
    color_space: ColorSpace,
}
impl SegmentedImageFrameProperties {
    pub fn new(
        segment_xy_resolutions: SegmentedXYImageResolutions,
        center_color_channel: ColorChannelLayout,
        peripheral_color_channels: ColorChannelLayout,
        color_space: ColorSpace,
    ) -> SegmentedImageFrameProperties {
        SegmentedImageFrameProperties {
            segment_xy_resolutions,
            center_color_channel,
            peripheral_color_channels,
            color_space,
        }
    }

    pub fn get_resolutions(&self) -> &SegmentedXYImageResolutions {
        &self.segment_xy_resolutions
    }

    pub fn get_center_color_channel(&self) -> &ColorChannelLayout {
        &self.center_color_channel
    }

    pub fn get_peripheral_color_channels(&self) -> &ColorChannelLayout {
        &self.peripheral_color_channels
    }

    pub fn get_color_space(&self) -> &ColorSpace {
        &self.color_space
    }

    pub fn verify_segmented_image_frame_matches_properties(
        &self,
        segmented_image_frame: &SegmentedImageFrame,
    ) -> Result<(), FeagiDataError> {
        if self != &segmented_image_frame.get_segmented_image_frame_properties() {
            return Err(FeagiDataError::BadParameters(
                "Segmented image frame does not match the expected segmented frame properties!"
                    .into(),
            )
            .into());
        }
        Ok(())
    }
}

impl Display for SegmentedImageFrameProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SegmentedImageFrameProperties(TODO)") // TODO
    }
}

//endregion

//region Corner Points
/// Holds pixel coordinates for cropping
#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct CornerPoints {
    pub upper_left: ImageXYPoint,
    pub lower_right: ImageXYPoint,
}

impl CornerPoints {
    pub fn new(
        upper_left: ImageXYPoint,
        lower_right: ImageXYPoint,
    ) -> Result<Self, FeagiDataError> {
        if lower_right.x <= upper_left.x || lower_right.y <= upper_left.y {
            return Err(FeagiDataError::BadParameters(
                "Given Points are not forming a proper rectangle!".into(),
            )
            .into());
        }
        Ok(CornerPoints {
            upper_left,
            lower_right,
        })
    }
    pub fn get_upper_right(&self) -> ImageXYPoint {
        ImageXYPoint::new(self.lower_right.x, self.upper_left.y)
    }

    pub fn get_lower_left(&self) -> ImageXYPoint {
        ImageXYPoint::new(self.upper_left.x, self.lower_right.y)
    }

    pub fn get_width(&self) -> u32 {
        self.lower_right.x - self.upper_left.x
    }

    pub fn get_height(&self) -> u32 {
        self.lower_right.y - self.upper_left.y
    }

    pub fn enclosed_area_width_height(&self) -> ImageXYResolution {
        ImageXYResolution::new(self.get_width(), self.get_height()).unwrap()
    }

    pub fn verify_fits_in_resolution(
        &self,
        resolution: ImageXYResolution,
    ) -> Result<(), FeagiDataError> {
        if self.lower_right.x > resolution.width || self.lower_right.y > resolution.height {
            return Err(FeagiDataError::BadParameters(format!(
                "Corner Points {} do not fit in given resolution {}!",
                self, resolution
            ))
            .into());
        }
        Ok(())
    }
}

impl Display for CornerPoints {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "CornerPoints(Upper Left: {}, Lower Right: {})",
            self.upper_left.to_string(),
            self.lower_right.to_string()
        )
    }
}

//endregion

//endregion

//region Misc

define_xyz_dimensions!(MiscDataDimensions, u32, "MiscDataDimensions", 0, "The dimensions of the internal 3D array of a Misc Data Struct. Coordinates align with the position of neuron coordinates in FEAGI");
define_xyz_mapping!(MiscDataDimensions, ImageXYZDimensions);
define_xyz_mapping!(MiscDataDimensions, CorticalChannelDimensions);

//endregion
