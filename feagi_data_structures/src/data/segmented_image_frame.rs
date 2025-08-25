//! Segmented vision frame processing for FEAGI peripheral vision simulation.
//!
//! This module provides the `SegmentedVisionFrame` struct which divides an input image
//! into nine segments with different resolutions to simulate peripheral vision. The center
//! segment typically has higher resolution while peripheral segments have lower resolution,
//! mimicking how human vision works with high acuity in the center and lower acuity in
//! the periphery.

use ndarray::Array3;
use crate::data::image_descriptors::{ColorChannelLayout, ColorSpace, SegmentedImageFrameProperties, SegmentedXYImageResolutions};
use crate::FeagiDataError;
use crate::data::ImageFrame;
use crate::genomic::{CorticalID, CorticalType, SensorCorticalType};
use crate::genomic::descriptors::{CorticalChannelIndex, CorticalGroupIndex};
use crate::neurons::xyzp::CorticalMappedXYZPNeuronData;

/// A frame divided into nine segments with different resolutions for peripheral vision simulation.
///
/// This structure represents a segmented view of a source frame, dividing it into nine regions:
/// - **Center**: High-resolution central region (foveal vision)
/// - **Eight peripheral segments**: Lower-resolution surrounding regions (peripheral vision)
///
/// The segmentation pattern follows this layout:
/// ```text
/// ┌─────────┬─────────┬─────────┐
/// │ upper_  │ upper_  │ upper_  │
/// │ left    │ middle  │ right   │
/// ├─────────┼─────────┼─────────┤
/// │ middle_ │ center  │ middle_ │
/// │ left    │         │ right   │
/// ├─────────┼─────────┼─────────┤
/// │ lower_  │ lower_  │ lower_  │
/// │ left    │ middle  │ right   │
/// └─────────┴─────────┴─────────┘
/// ```
///
/// This design allows FEAGI to process visual information with varying levels of detail,
/// concentrating computational resources in the center of attention while maintaining
/// awareness of the broader visual field.
#[derive(Clone, Debug)]
pub struct SegmentedImageFrame {
    /// Lower-left segment of the vision frame
    lower_left: ImageFrame,
    /// Middle-left segment of the vision frame
    middle_left: ImageFrame,
    /// Upper-left segment of the vision frame
    upper_left: ImageFrame,
    /// Upper-middle segment of the vision frame
    upper_middle: ImageFrame,
    /// Upper-right segment of the vision frame
    upper_right: ImageFrame,
    /// Middle-right segment of the vision frame
    middle_right: ImageFrame,
    /// Lower-right segment of the vision frame
    lower_right: ImageFrame,
    /// Lower-middle segment of the vision frame
    lower_middle: ImageFrame,
    /// Center segment of the vision frame (typically higher resolution)
    center: ImageFrame,

}

impl SegmentedImageFrame {
    
    //region Constructors

    /// Creates a new SegmentedVisionFrame with specified resolutions and color properties.
    ///
    /// This constructor initializes all nine segments with their respective resolutions
    /// and the same color format and color space. Each segment is created as an empty
    /// ImageFrame ready to receive cropped and resized data from source images.
    ///
    /// # Arguments
    ///
    /// * `segment_resolutions` - The target resolutions for each of the nine segments
    /// * `segment_color_channels` - The color channel format (GrayScale, RG, RGB, or RGBA)
    /// * `segment_color_space` - The color space (Linear or Gamma)
    /// * `input_frames_source_width_height` - The expected resolution of source frames (width, height)
    ///
    /// # Returns
    ///
    /// A Result containing either:
    /// - Ok(SegmentedVisionFrame) if all segments were created successfully
    /// - Err(DataProcessingError) if any segment creation fails
    pub fn new(segment_resolutions: &SegmentedXYImageResolutions, color_space: &ColorSpace,
               center_color_channels: &ColorChannelLayout, peripheral_color_channels: &ColorChannelLayout) -> Result<SegmentedImageFrame, FeagiDataError> {
        Ok(SegmentedImageFrame {
            lower_left: ImageFrame::new(peripheral_color_channels, &color_space, &segment_resolutions.lower_left)?,
            middle_left: ImageFrame::new(peripheral_color_channels, &color_space, &segment_resolutions.middle_left)?,
            upper_left: ImageFrame::new(peripheral_color_channels, &color_space, &segment_resolutions.upper_left)?,
            upper_middle: ImageFrame::new(peripheral_color_channels, &color_space, &segment_resolutions.upper_middle)?,
            upper_right: ImageFrame::new(peripheral_color_channels, &color_space, &segment_resolutions.upper_right)?,
            middle_right: ImageFrame::new(peripheral_color_channels, &color_space, &segment_resolutions.middle_right)?,
            lower_right: ImageFrame::new(peripheral_color_channels, &color_space, &segment_resolutions.lower_right)?,
            lower_middle: ImageFrame::new(peripheral_color_channels, &color_space, &segment_resolutions.lower_middle)?,
            center: ImageFrame::new(center_color_channels, &color_space, &segment_resolutions.center)?,
        })
    }

    pub fn from_segmented_image_frame_properties(properties: &SegmentedImageFrameProperties) -> Result<SegmentedImageFrame, FeagiDataError> {
        Self::new(
            properties.get_resolutions(),
            properties.get_color_space(),
            properties.get_center_color_channel(),
            properties.get_peripheral_color_channels()
        )
    }
    
    //region
    
    
    
    //region Static Methods

    /// Creates a predefined set of cortical areas for segmented vision processing.
    ///
    /// This utility method generates 9 cortical areas arranged in a 3x3 grid pattern
    /// for processing segmented vision data. Each segment processes a different region
    /// of the visual field, allowing for spatial attention and region-specific processing.
    ///
    /// # Arguments
    /// * `camera_index` - The grouping index for this camera system (0-255)
    ///
    /// # Returns
    /// Array of 9 CorticalID values arranged as:
    /// ```text
    /// [6] Top-Left     [7] Top-Middle     [8] Top-Togjt
    /// [3] Middle-Left  [4] Center         [5] Middle-Right
    /// [0] Bottom-Left  [1] Bottom-Middle  [2] Bottom-Right
    /// ```
    ///
    /// # ImageCamera Segmentation
    /// - **Center**: Primary focus area for detailed processing
    /// - **Surrounding segments**: Peripheral vision areas for context and motion detection
    pub fn create_ordered_cortical_ids_for_segmented_vision(camera_index: CorticalGroupIndex) -> [CorticalID; 9] {
        [
            SensorCorticalType::ImageCameraBottomLeft.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraBottomMiddle.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraBottomRight.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraMiddleLeft.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraCenter.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraMiddleRight.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraTopLeft.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraTopMiddle.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraTopRight.to_cortical_id(camera_index),
        ]
    }

    pub fn create_ordered_cortical_types_for_segmented_vision() -> [CorticalType; 9] {
        [
            SensorCorticalType::ImageCameraBottomLeft.into(),
            SensorCorticalType::ImageCameraBottomMiddle.into(),
            SensorCorticalType::ImageCameraBottomRight.into(),
            SensorCorticalType::ImageCameraMiddleLeft.into(),
            SensorCorticalType::ImageCameraCenter.into(),
            SensorCorticalType::ImageCameraMiddleRight.into(),
            SensorCorticalType::ImageCameraTopLeft.into(),
            SensorCorticalType::ImageCameraTopMiddle.into(),
            SensorCorticalType::ImageCameraTopRight.into(),
        ]
    }
    
    //endregion



    //region get properties

    pub fn get_segmented_image_frame_properties(&self) -> SegmentedImageFrameProperties {
        SegmentedImageFrameProperties::new(
            &self.get_segmented_frame_target_resolutions(),
            self.center.get_channel_layout(),
            self.lower_right.get_channel_layout(), // all peripherals should be the same
            self.get_color_space()
        )
    }

    /// Returns the color space used by all segments in this frame.
    ///
    /// Since all segments share the same color space, this method returns
    /// a reference to the color space from any segment (using upper_left as representative).
    ///
    /// # Returns
    ///
    /// A reference to the ColorSpace enum value.
    pub fn get_color_space(&self) -> &ColorSpace {
        self.upper_left.get_color_space()
    }

    /// Returns the channel layout of the center segment.
    ///
    /// # Returns
    ///
    /// A reference to the ChannelLayout enum value for the center segment.
    pub fn get_center_channel_layout(&self) -> &ColorChannelLayout {
        self.center.get_channel_layout()
    }

    /// Returns the channel layout of the peripheral segments.
    ///
    /// All peripheral segments (non-center) are expected to have the same channel layout.
    /// This method returns the layout from the lower_left segment as representative.
    ///
    /// # Returns
    ///
    /// A reference to the ChannelLayout enum value for the peripheral segments.
    pub fn get_peripheral_channel_layout(&self) -> &ColorChannelLayout {
        self.lower_left.get_channel_layout() // All peripherals should be the same
    }

    pub fn get_segmented_frame_target_resolutions(&self) -> SegmentedXYImageResolutions {
        SegmentedXYImageResolutions::new(
            self.lower_left.get_xy_resolution(),
            self.lower_middle.get_xy_resolution(),
            self.lower_right.get_xy_resolution(),
            self.middle_left.get_xy_resolution(),
            self.center.get_xy_resolution(),
            self.middle_right.get_xy_resolution(),
            self.upper_left.get_xy_resolution(),
            self.upper_middle.get_xy_resolution(),
            self.upper_right.get_xy_resolution()
        )
    }

    /// Returns references to the internal pixel data arrays for all nine segments.
    ///
    /// Provides direct access to the underlying 3D arrays containing pixel data
    /// for each segment. The arrays are returned in the standard cortical ordering.
    ///
    /// # Returns
    ///
    /// An array of 9 references to Array3<f32>, one for each segment in cortical order.
    pub fn get_image_internal_data(&self) -> [&Array3<f32>; 9] {
        // return in same order as cortical IDs
        [
            self.lower_left.get_internal_data(),
            self.lower_middle.get_internal_data(),
            self.lower_right.get_internal_data(),
            self.middle_left.get_internal_data(),
            self.center.get_internal_data(),
            self.middle_right.get_internal_data(),
            self.upper_left.get_internal_data(),
            self.upper_middle.get_internal_data(),
            self.upper_right.get_internal_data(),
        ]
    }

    pub fn get_ordered_image_frame_references(&self) -> [&ImageFrame; 9] {
        [&self.center, &self.lower_left, &self.middle_left, &self.upper_left, &self.upper_middle,
            &self.upper_right, &self.middle_right, &self.lower_right,
            &self.lower_middle]
    }

    pub fn get_mut_ordered_image_frame_references(&mut self) -> [&mut ImageFrame; 9] {
        [&mut self.center, &mut self.lower_left, &mut self.middle_left, &mut self.upper_left, &mut self.upper_middle,
            &mut self.upper_right, &mut self.middle_right, &mut self.lower_right,
            &mut self.lower_middle]
    }

    pub(crate) fn get_image_internal_data_mut(&mut self) -> [&mut Array3<f32>; 9] {
        // return in same order as cortical IDs
        [
            self.lower_left.get_internal_data_mut(),
            self.lower_middle.get_internal_data_mut(),
            self.lower_right.get_internal_data_mut(),
            self.middle_left.get_internal_data_mut(),
            self.center.get_internal_data_mut(),
            self.middle_right.get_internal_data_mut(),
            self.upper_left.get_internal_data_mut(),
            self.upper_middle.get_internal_data_mut(),
            self.upper_right.get_internal_data_mut(),
        ]
    }

    //endregion

    //region neuron export
    pub fn write_as_neuron_xyzp_data(&self, write_target: &mut CorticalMappedXYZPNeuronData, channel_index: CorticalChannelIndex, ordered_cortical_ids: &[CorticalID; 9]) -> Result<(), FeagiDataError> {
        let ordered_refs: [&ImageFrame; 9] = self.get_ordered_image_frame_references();
        for index in 0..9 {
            ordered_refs[index].write_as_neuron_xyzp_data(write_target,ordered_cortical_ids[index], channel_index)?;
        }
        Ok(())
    }

    //endregion
    
}

impl std::fmt::Display for SegmentedImageFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SegmentedImageFrame()")
    }
}