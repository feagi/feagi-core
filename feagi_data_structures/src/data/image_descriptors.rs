//! Vision processing descriptors and parameter structures for FEAGI.
//!
//! This module provides data structures and enums for describing image properties

use std::cmp;
use std::ops::RangeInclusive;
use crate::FeagiDataError;
use crate::basic_components::{CartesianResolution, FlatCoordinateU32};
use crate::data::{ImageFrame, SegmentedImageFrame};

//region Image XY

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct ImageXYPoint(FlatCoordinateU32);

impl ImageXYPoint {
    pub fn new(x: u32, y: u32) -> Self {
        ImageXYPoint(FlatCoordinateU32::new(x, y))
    }
}

impl std::ops::Deref for ImageXYPoint {
    type Target = FlatCoordinateU32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<FlatCoordinateU32> for ImageXYPoint {
    fn from(x: FlatCoordinateU32) -> Self {
        ImageXYPoint(x)
    }
}

impl From<ImageXYPoint> for FlatCoordinateU32 {
    fn from(x: ImageXYPoint) -> Self {
        x.0
    }
}

impl std::fmt::Display for ImageXYPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.0, self.0)
    }
}



/// Describes the resolution of the image (width and height)
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct ImageXYResolution(CartesianResolution);

impl ImageXYResolution {
    pub fn new(x_width: usize, y_height: usize,) -> Result<Self,FeagiDataError> {
        Ok(ImageXYResolution(CartesianResolution::new(x_width, y_height)?))
    }
}

impl std::ops::Deref for ImageXYResolution {
    type Target = CartesianResolution;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<CartesianResolution> for ImageXYResolution {
    fn from(coord: CartesianResolution) -> Self {
        ImageXYResolution(coord)
    }
}

impl From<ImageXYResolution> for CartesianResolution {
    fn from(coord: ImageXYResolution) -> Self {
        coord.0
    }
}

impl std::fmt::Display for ImageXYResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<{}w by {}h>", self.0, self.0)
    }
}


//endregion

//region Segmented Image XY Resolutions
/// Target resolutions for each of the nine segments in a segmented vision frame
///
/// This structure stores the desired output resolution for each of the segments
/// in a grid arrangement (3x3): corners, edges, and center.
#[derive(PartialEq, Clone, Copy, Debug, Eq, Hash)]
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
    pub fn create_with_same_sized_peripheral(center_resolution: ImageXYResolution, peripheral_resolutions: ImageXYResolution) -> SegmentedXYImageResolutions {

        SegmentedXYImageResolutions::new(peripheral_resolutions, peripheral_resolutions,
                                         peripheral_resolutions, peripheral_resolutions,
                                         center_resolution, peripheral_resolutions,
                                         peripheral_resolutions, peripheral_resolutions,
                                         peripheral_resolutions)
    }

    pub fn as_ordered_array(&self) ->[&ImageXYResolution; 9] {
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

impl std::fmt::Display for SegmentedXYImageResolutions {
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
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum ColorSpace {
    Linear,
    Gamma
}

impl std::fmt::Display for ColorSpace {
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
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum ColorChannelLayout {
    GrayScale = 1, // R
    RG = 2,
    RGB = 3,
    RGBA = 4,
}

impl std::fmt::Display for ColorChannelLayout {
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
            _ => Err(FeagiDataError::BadParameters(format!("No Channel Layout has {} channels! Acceptable values are 1,2,3,4!", value)).into())
        }
    }
}

impl From<ColorChannelLayout> for usize {
    fn from(value: ColorChannelLayout) -> usize {
        value as usize
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
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MemoryOrderLayout {
    HeightsWidthsChannels, // default, also called row major
    ChannelsHeightsWidths, // common in machine learning
    WidthsHeightsChannels, // cartesian, the best one
    HeightsChannelsWidths,
    ChannelsWidthsHeights,
    WidthsChannelsHeights,
}
//endregion

//region Image Frame Properties

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn new(image_resolution: ImageXYResolution, color_space: ColorSpace, color_channel_layout: ColorChannelLayout) -> Result<Self, FeagiDataError> {
        Ok(ImageFrameProperties{
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
    pub fn verify_image_frame_matches_properties(&self, image_frame: &ImageFrame) -> Result<(), FeagiDataError> {
        if image_frame.get_xy_resolution() != self.image_resolution {
            return Err(FeagiDataError::BadParameters(format!{"Expected resolution of {} but received an image with resolution of {}!",
                                                             self.image_resolution, image_frame.get_xy_resolution()}).into())
        }
        if image_frame.get_color_space() != &self.color_space {
            return Err(FeagiDataError::BadParameters(format!("Expected color space of {}, but got image with color space of {}!", self.color_space.to_string(), self.color_space.to_string())).into())
        }
        if image_frame.get_channel_layout() != &self.color_channel_layout {
            return Err(FeagiDataError::BadParameters(format!("Expected color channel layout of {}, but got image with color channel layout of {}!", self.color_channel_layout.to_string(), self.color_channel_layout.to_string())).into())
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
}

impl std::fmt::Display for ImageFrameProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = format!("ImageFrameProperties({}, {}, {})", self.image_resolution, self.color_space.to_string(), self.color_channel_layout.to_string());
        write!(f, "{}", s)
    }
}

//endregion

//region Segmented Image Frame Properties

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SegmentedImageFrameProperties {
    segment_xy_resolutions: SegmentedXYImageResolutions,
    center_color_channel: ColorChannelLayout,
    peripheral_color_channels: ColorChannelLayout,
    color_space: ColorSpace,
}
impl SegmentedImageFrameProperties {
    pub fn new(
        segment_xy_resolutions: &SegmentedXYImageResolutions,
        center_color_channels: &ColorChannelLayout,
        peripheral_color_channels: &ColorChannelLayout,
        color_space: &ColorSpace,
    ) -> SegmentedImageFrameProperties {
        SegmentedImageFrameProperties {
            segment_xy_resolutions: segment_xy_resolutions.clone(),
            center_color_channel: center_color_channels.clone(),
            peripheral_color_channels: peripheral_color_channels.clone(),
            color_space: *color_space,
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

    pub fn verify_segmented_image_frame_matches_properties(&self, segmented_image_frame: &SegmentedImageFrame) -> Result<(), FeagiDataError> {
        if self != &segmented_image_frame.get_segmented_image_frame_properties() {
            return Err(FeagiDataError::BadParameters("Segmented image frame does not match the expected segmented frame properties!".into()).into())
        }
        Ok(())
    }


}

impl std::fmt::Display for SegmentedImageFrameProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SegmentedImageFrameProperties(TODO)") // TODO
    }
}

//endregion

//region Corner Points
/// Holds pixel coordinates for cropping in row-major order.
///
/// The coordinates are inclusive on the bottom-left and exclusive on the top-right.
/// In row-major order, (0,0) is in the top-left corner, with Y increasing downward
/// and X increasing rightward.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CornerPoints {
    /// The bottom-left corner coordinate as (y, x), where the top left is towards (0,0)
    lower_left: (usize, usize),
    /// The top-right corner coordinate as (y, x), where the top left is towards (0,0)
    upper_right: (usize, usize),
}

// TODO move to ImageXYPoint

impl CornerPoints {
    /// Creates a new CornerPoints instance from row-major coordinates.
    ///
    /// # Arguments
    ///
    /// * `lower_left_yx` - The bottom-left corner as (y, x)
    /// * `upper_right_ux` - The top-right corner as (y, x)
    ///
    /// # Returns
    ///
    /// A Result containing either:
    /// - Ok(CornerPoints) if the coordinates are valid
    /// - Err(DataProcessingError) if the coordinates are invalid
    pub fn new_from_row_major(lower_left_yx: (usize, usize), upper_right_yx: (usize, usize)) -> Result<CornerPoints,  FeagiDataError> {
        if lower_left_yx.1 >= upper_right_yx.1 {
            return Err(FeagiDataError::BadParameters(format!("The lower left point must have a smaller X ({}) index than the upper right point ({})!", lower_left_yx.1, upper_right_yx.1).into()).into());
        }

        if lower_left_yx.0 <= upper_right_yx.0 {
            return Err(FeagiDataError::BadParameters(format!("The lower left point must have a greater Y ({}) index than the upper right point ({})!", lower_left_yx.0, upper_right_yx.0).into()).into());
        };
        Ok(CornerPoints {
            lower_left: lower_left_yx,
            upper_right: upper_right_yx
        })
    }

    /// Creates a new CornerPoints instance from cartesian coordinates.
    ///
    /// # Arguments
    ///
    /// * `left_lower_xy` - The bottom-left corner in cartesian coordinates (x, y)
    /// * `right_upper_xy` - The top-right corner in cartesian coordinates (x, y)
    /// * `total_source_resolution_width_height` - The total resolution as (width, height)
    ///
    /// # Returns
    ///
    /// A Result containing either:
    /// - Ok(CornerPoints) if the coordinates are valid
    /// - Err(DataProcessingError) if the coordinates are invalid or out of bounds
    pub fn new_from_cartesian(
        left_lower_xy: (usize, usize), right_upper_xy: (usize, usize),
        total_source_resolution: ImageXYResolution)
        -> Result<CornerPoints,  FeagiDataError> {

        if left_lower_xy.0 >= right_upper_xy.0 || left_lower_xy.1 >= right_upper_xy.1 {
            return Err(FeagiDataError::BadParameters("Given corner points do not enclose a valid area!".into()).into());
        }

        if right_upper_xy.0 > total_source_resolution.width || right_upper_xy.1 > total_source_resolution.height {
            return Err(FeagiDataError::BadParameters("Corner bounds must be within the total resolution!".into()).into());
        }

        Ok(CornerPoints {
            lower_left: (total_source_resolution.height - left_lower_xy.1, left_lower_xy.0),
            upper_right: (total_source_resolution.height - right_upper_xy.1, right_upper_xy.0)
        })
    }

    /// Gets the row major coordinates of the lower-left corner (Lower Exclusive, Left Inclusive)
    ///
    /// # Returns
    ///
    /// * `(usize, usize)` - Coordinate pair (y, x) for the lower-left corner
    pub fn lower_left_row_major(&self) -> (usize, usize) {
        self.lower_left
    }

    /// Gets the row major coordinates of the upper-right corner (Upper Inclusive, Right Exclusive)
    ///
    /// # Returns
    ///
    /// * `(usize, usize)` - Coordinate pair (y, x) for the upper-right corner
    pub fn upper_right_row_major(&self) -> (usize, usize) {
        self.upper_right
    }

    /// Gets the row major coordinates of the lower-right corner (Lower Exclusive, Right Exclusive)
    ///
    /// # Returns
    ///
    /// * `(usize, usize)` - Coordinate pair (y, x) for the lower-right corner
    pub fn lower_right_row_major(&self) -> (usize, usize) {
        (self.lower_left.0, self.upper_right.1)
    }


    pub fn upper_left_row_major(&self) -> (usize, usize) {
        (self.upper_right.0, self.lower_left.1)
    }
    
    pub fn does_fit_in_frame_of_width_height(&self, resolution: ImageXYResolution) -> bool {
        self.upper_right.1 <= resolution.width && self.lower_left.1 <= resolution.height
    }
    
    pub fn enclosed_area_width_height(&self) -> ImageXYResolution {
        ImageXYResolution::new(self.upper_right.1 - self.lower_left.1, self.lower_left.0 - self.upper_right.0).unwrap()
    }
}

impl std::fmt::Display for CornerPoints {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CornerPoints(TODO)") // TODO
    }
}

//endregion

//region Gaze Properties
/// Properties defining the center region of a segmented vision frame
///
/// This structure defines the coordinates and size of the central region
/// in a normalized coordinate space (0.0 to 1.0).
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct GazeProperties {
    /// Center point coordinates in normalized space (0.0-1.0), from the top left
    pub(crate) eccentricity_normalized_yx: (f32, f32), // Scaled from 0 to 1 //
    /// Size of the center region in normalized space (0.0-1.0)
    pub(crate) modularity_normalized_yx: (f32, f32), // ditto
}

impl GazeProperties {
    /// Creates a new SegmentedVisionCenterProperties with row-major coordinates.
    ///
    /// This constructor creates center properties using normalized coordinates where
    /// the origin (0,0) is in the top-left corner of the image. This is typical of many toolboxes such as Numpy
    ///
    /// # Arguments
    ///
    /// * `center_coordinates_normalized_yx` - Center point as (y, x) in normalized space (0.0-1.0)
    /// * `center_size_normalized_yx` - Size as (height, width) in normalized space (0.0-1.0)
    ///
    /// # Returns
    ///
    /// A Result containing either:
    /// - Ok(SegmentedVisionCenterProperties) if the parameters are valid
    /// - Err(DataProcessingError) if coordinates or size are outside valid ranges
    pub(crate) fn new_row_major_where_origin_top_left(center_coordinates_normalized_yx: (f32, f32), center_size_normalized_yx: (f32, f32)) -> Result<GazeProperties, FeagiDataError> {
        let range_0_1: RangeInclusive<f32> = 0.0..=1.0;
        if !(range_0_1.contains(&center_coordinates_normalized_yx.0) && range_0_1.contains(&center_coordinates_normalized_yx.1)) {
            return Err(FeagiDataError::BadParameters("Central vision center coordinates are to be normalized and must be between 0 and 1!".into()).into())
        }
        if !(range_0_1.contains(&center_size_normalized_yx.0) && range_0_1.contains(&center_size_normalized_yx.1)) {
            return Err(FeagiDataError::BadParameters("Central vision size is to be normalized and must be between 0 and 1!".into()).into())
        }

        let range_overlap_y: RangeInclusive<f32> = (center_size_normalized_yx.0 / 2.0)..=(1.0 + (center_size_normalized_yx.0 / 2.0));
        let range_overlap_x: RangeInclusive<f32> = (center_size_normalized_yx.1 / 2.0)..=(1.0 + (center_size_normalized_yx.1 / 2.0));

        if !(range_overlap_y.contains(&center_coordinates_normalized_yx.0) && range_overlap_x.contains(&center_coordinates_normalized_yx.1)) {
            return Err(FeagiDataError::BadParameters("Resulting central vision crop includes regions outside input image!".into()).into())
        }

        Ok(GazeProperties {
            eccentricity_normalized_yx: center_coordinates_normalized_yx,
            modularity_normalized_yx: center_size_normalized_yx,
        })
    }

    /// Creates a new SegmentedVisionCenterProperties with Cartesian coordinates.
    ///
    /// This constructor creates center properties using normalized Cartesian coordinates
    /// where the origin (0,0) is in the bottom-left corner of the image. This is typical in graphics pipelines.
    ///
    /// # Arguments
    ///
    /// * `center_coordinates_normalized_cartesian_xy` - Center point as (x, y) in normalized space (0.0-1.0)
    /// * `center_size_normalized_xy` - Size as (width, height) in normalized space (0.0-1.0)
    ///
    /// # Returns
    ///
    /// A Result containing either:
    /// - Ok(SegmentedVisionCenterProperties) if the parameters are valid
    /// - Err(DataProcessingError) if coordinates or size are outside valid ranges
    pub fn cartesian_where_origin_bottom_left(center_coordinates_normalized_cartesian_xy: (f32, f32), center_size_normalized_xy: (f32, f32)) -> Result<GazeProperties, FeagiDataError> {
        GazeProperties::new_row_major_where_origin_top_left(
            (center_coordinates_normalized_cartesian_xy.1, 1.0 - center_coordinates_normalized_cartesian_xy.0),
            (center_size_normalized_xy.1, center_size_normalized_xy.0))
    }

    /// Creates a default centered SegmentedFrameCenterProperties.
    ///
    /// This convenience method creates center properties with the center region
    /// positioned at the middle of the image with a moderate size.
    ///
    /// # Returns
    ///
    /// A SegmentedFrameCenterProperties with default centered configuration.
    pub fn create_default_centered() -> GazeProperties {
        GazeProperties::new_row_major_where_origin_top_left((0.5, 0.5), (0.5, 0.5)).unwrap()
    }

    pub fn calculate_source_corner_points_for_segmented_video_frame(&self, source_frame_resolution: ImageXYResolution) -> Result<[CornerPoints; 9], FeagiDataError> {
        if source_frame_resolution.width < 3 || source_frame_resolution.height < 3 {
            return Err(FeagiDataError::BadParameters("Source frame width and height must be at least 3!".into()).into())
        }

        let center_corner_points = self.calculate_pixel_coordinates_of_center_corners(source_frame_resolution)?;
        Ok([
            CornerPoints::new_from_row_major((source_frame_resolution.height, 0), center_corner_points.lower_left_row_major())?,
            CornerPoints::new_from_row_major((source_frame_resolution.height, center_corner_points.lower_left_row_major().1), center_corner_points.lower_right_row_major())?,
            CornerPoints::new_from_row_major((source_frame_resolution.height, center_corner_points.upper_right_row_major().1), (center_corner_points.lower_left_row_major().1, source_frame_resolution.width))?,
            CornerPoints::new_from_row_major((center_corner_points.lower_left_row_major().0, 0), center_corner_points.upper_left_row_major())?,
            center_corner_points,
            CornerPoints::new_from_row_major(center_corner_points.lower_right_row_major(), (center_corner_points.upper_right_row_major().0, source_frame_resolution.width))?,
            CornerPoints::new_from_row_major((center_corner_points.upper_right_row_major().0, 0), (0, center_corner_points.lower_left_row_major().1))?,
            CornerPoints::new_from_row_major(center_corner_points.upper_left_row_major(), (0, center_corner_points.upper_right_row_major().1))?,
            CornerPoints::new_from_row_major(center_corner_points.upper_right_row_major(), (0, source_frame_resolution.width))?,
        ])
    }

    fn calculate_pixel_coordinates_of_center_corners(&self, source_frame_resolution: ImageXYResolution) -> Result<CornerPoints, FeagiDataError> {
        let source_frame_width_height_f: (f32, f32) = (source_frame_resolution.width as f32, source_frame_resolution.height as f32);
        let center_size_normalized_half_yx: (f32, f32) = (self.modularity_normalized_yx.0 / 2.0, self.modularity_normalized_yx.1 / 2.0);

        // We use max / min to ensure that there is always a 1 pixel buffer along all edges for use in peripheral vision (since we cannot use a resolution of 0)
        let bottom_pixel: usize = cmp::min(source_frame_resolution.width - 1,
                                           ((self.eccentricity_normalized_yx.0 + center_size_normalized_half_yx.0) * source_frame_width_height_f.1).ceil() as usize);
        let top_pixel: usize = cmp::max(1,
                                        (( self.eccentricity_normalized_yx.0 - center_size_normalized_half_yx.0) * source_frame_width_height_f.1).floor() as usize);
        let left_pixel: usize = cmp::max(1,
                                         ((self.eccentricity_normalized_yx.1 - center_size_normalized_half_yx.1) * source_frame_width_height_f.0).floor() as usize);
        let right_pixel: usize = cmp::min(source_frame_resolution.width - 1,
                                          (( self.eccentricity_normalized_yx.1 + center_size_normalized_half_yx.1) * source_frame_width_height_f.0).ceil() as usize);

        let corner_points: CornerPoints = CornerPoints::new_from_row_major(
            (bottom_pixel, left_pixel),
            (top_pixel, right_pixel)
        )?;
        Ok(corner_points)
    }
}

impl std::fmt::Display for GazeProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "GazeProperties(TODO)") // TODO
    }
}
//endregion


