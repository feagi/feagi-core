//! Segmented vision frame processing for FEAGI peripheral vision simulation.
//!
//! This module provides the `SegmentedVisionFrame` struct which divides an input image
//! into nine segments with different resolutions to simulate peripheral vision. The center
//! segment typically has higher resolution while peripheral segments have lower resolution,
//! mimicking how human vision works with high acuity in the center and lower acuity in
//! the periphery.

use ndarray::Array3;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::{CorticalID, CorticalType, SensorCorticalType};
use feagi_data_structures::genomic::descriptors::{CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::neurons::xyzp::CorticalMappedXYZPNeuronData;
use super::ImageFrame;
use super::descriptors::{ColorChannelLayout, ColorSpace, SegmentedImageFrameProperties, SegmentedXYImageResolutions};



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

    /// Creates a SegmentedImageFrame from SegmentedImageFrameProperties.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::{SegmentedImageFrame, descriptors::*};
    ///
    /// let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
    ///     ImageXYResolution::new(64, 64).unwrap(), // center
    ///     ImageXYResolution::new(32, 32).unwrap()  // peripherals
    /// );
    /// let props = SegmentedImageFrameProperties::new(
    ///     &resolutions, &ColorChannelLayout::RGB,
    ///     &ColorChannelLayout::RGB, &ColorSpace::Linear
    /// );
    /// let frame = SegmentedImageFrame::from_segmented_image_frame_properties(&props).unwrap();
    /// ```
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
    /// * `is_incremental` - If the encoder / decoder is incremental instead of instant change
    ///
    /// # Returns
    /// Array of 9 CorticalID values arranged as:
    /// ```text
    /// [6] Top-Left     [7] Top-Middle     [8] Top-Right
    /// [3] Middle-Left  [4] Center         [5] Middle-Right
    /// [0] Bottom-Left  [1] Bottom-Middle  [2] Bottom-Right
    /// ```
    ///
    /// # ImageCamera Segmentation
    /// - **Center**: Primary focus area for detailed processing
    /// - **Surrounding segments**: Peripheral vision areas for context and motion detection
    pub fn create_ordered_cortical_ids_for_segmented_vision(camera_index: CorticalGroupIndex, is_incremental: bool) -> [CorticalID; 9] {
        if is_incremental {
            return         [
                SensorCorticalType::ImageCameraBottomLeftIncremental.to_cortical_id(camera_index),
                SensorCorticalType::ImageCameraBottomMiddleIncremental.to_cortical_id(camera_index),
                SensorCorticalType::ImageCameraBottomRightIncremental.to_cortical_id(camera_index),
                SensorCorticalType::ImageCameraMiddleLeftIncremental.to_cortical_id(camera_index),
                SensorCorticalType::ImageCameraCenterIncremental.to_cortical_id(camera_index),
                SensorCorticalType::ImageCameraMiddleRightIncremental.to_cortical_id(camera_index),
                SensorCorticalType::ImageCameraTopLeftIncremental.to_cortical_id(camera_index),
                SensorCorticalType::ImageCameraTopMiddleIncremental.to_cortical_id(camera_index),
                SensorCorticalType::ImageCameraTopRightIncremental.to_cortical_id(camera_index),
            ]
        }

        [
            SensorCorticalType::ImageCameraBottomLeftInstant.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraBottomMiddleInstant.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraBottomRightInstant.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraMiddleLeftInstant.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraCenterInstant.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraMiddleRightInstant.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraTopLeftInstant.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraTopMiddleInstant.to_cortical_id(camera_index),
            SensorCorticalType::ImageCameraTopRightInstant.to_cortical_id(camera_index),
        ]
    }

    /// Returns cortical types for each segment in order.
    ///
    /// # Arguments
    /// * `is_incremental` - If the encoder / decoder is incremental instead of instant change
    ///
    /// # Example
    /// ```
    /// use feagi_data_structures::data::SegmentedImageFrame;
    /// use feagi_data_structures::genomic::{CorticalType, SensorCorticalType};
    /// 
    /// let types = SegmentedImageFrame::create_ordered_cortical_types_for_segmented_vision(false);
    /// assert_eq!(types[4], CorticalType::Sensory(SensorCorticalType::ImageCameraCenterInstant));
    /// ```
    pub fn create_ordered_cortical_types_for_segmented_vision(is_incremental: bool) -> [CorticalType; 9] {

        if is_incremental {
            return         [
                SensorCorticalType::ImageCameraBottomLeftIncremental.into(),
                SensorCorticalType::ImageCameraBottomMiddleIncremental.into(),
                SensorCorticalType::ImageCameraBottomRightIncremental.into(),
                SensorCorticalType::ImageCameraMiddleLeftIncremental.into(),
                SensorCorticalType::ImageCameraCenterIncremental.into(),
                SensorCorticalType::ImageCameraMiddleRightIncremental.into(),
                SensorCorticalType::ImageCameraTopLeftIncremental.into(),
                SensorCorticalType::ImageCameraTopMiddleIncremental.into(),
                SensorCorticalType::ImageCameraTopRightIncremental.into(),
            ]
        }

        [
            SensorCorticalType::ImageCameraBottomLeftInstant.into(),
            SensorCorticalType::ImageCameraBottomMiddleInstant.into(),
            SensorCorticalType::ImageCameraBottomRightInstant.into(),
            SensorCorticalType::ImageCameraMiddleLeftInstant.into(),
            SensorCorticalType::ImageCameraCenterInstant.into(),
            SensorCorticalType::ImageCameraMiddleRightInstant.into(),
            SensorCorticalType::ImageCameraTopLeftInstant.into(),
            SensorCorticalType::ImageCameraTopMiddleInstant.into(),
            SensorCorticalType::ImageCameraTopRightInstant.into(),
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

    /// Returns the resolution configuration for all segments.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::{SegmentedImageFrame, descriptors::*};
    /// 
    /// let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
    ///     ImageXYResolution::new(64, 64).unwrap(),  // center
    ///     ImageXYResolution::new(32, 32).unwrap(), // peripherals
    /// );
    /// let props = SegmentedImageFrameProperties::new(
    ///     &resolutions, &ColorChannelLayout::RGB,
    ///     &ColorChannelLayout::RGB, &ColorSpace::Linear
    /// );
    /// let frame = SegmentedImageFrame::from_segmented_image_frame_properties(&props).unwrap();
    /// let res = frame.get_segmented_frame_target_resolutions();
    /// assert_eq!(res.center.width, 64);
    /// ```
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
    pub fn get_image_internal_data(&self) -> [&Array3<u8>; 9] {
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

    /// Returns references to all image frames in cortical order.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::{SegmentedImageFrame, descriptors::*};
    ///
    /// let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
    ///     ImageXYResolution::new(32, 32).unwrap(), // peripherals
    ///     ImageXYResolution::new(64, 64).unwrap()  // center
    /// );
    /// let props = SegmentedImageFrameProperties::new(
    ///     &resolutions, &ColorChannelLayout::RGB,
    ///     &ColorChannelLayout::RGB, &ColorSpace::Linear
    /// );
    /// let frame = SegmentedImageFrame::from_segmented_image_frame_properties(&props).unwrap();
    /// let refs = frame.get_ordered_image_frame_references();
    /// assert_eq!(refs.len(), 9);
    /// assert_eq!(refs[4].get_xy_resolution().width, 32); // center segment
    /// ```
    pub fn get_ordered_image_frame_references(&self) -> [&ImageFrame; 9] {
        [&self.lower_left, &self.lower_middle, &self.lower_right, &self.middle_left,
            &self.center, &self.middle_right, &self.upper_left,
            &self.upper_middle, &self.upper_right]
    }

    /// Returns mutable references to all image frames in cortical order.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_structures::data::{SegmentedImageFrame, descriptors::*};
    ///
    /// let resolutions = SegmentedXYImageResolutions::create_with_same_sized_peripheral(
    ///     ImageXYResolution::new(32, 32).unwrap(), // peripherals
    ///     ImageXYResolution::new(64, 64).unwrap()  // center
    /// );
    /// let props = SegmentedImageFrameProperties::new(
    ///     &resolutions, &ColorChannelLayout::RGB,
    ///     &ColorChannelLayout::RGB, &ColorSpace::Linear
    /// );
    /// let mut frame = SegmentedImageFrame::from_segmented_image_frame_properties(&props).unwrap();
    /// let mut_refs = frame.get_mut_ordered_image_frame_references();
    /// mut_refs[0].skip_encoding = true; // Modify first segment
    /// ```
    pub fn get_mut_ordered_image_frame_references(&mut self) -> [&mut ImageFrame; 9] {
        [&mut self.lower_left, &mut self.lower_middle, &mut self.lower_right, &mut self.middle_left, &mut self.center,
            &mut self.middle_right, &mut self.upper_left, &mut self.upper_middle,
            &mut self.upper_right]
    }

    pub(crate) fn get_image_internal_data_mut(&mut self) -> [&mut Array3<u8>; 9] {
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
            if ordered_refs[index].skip_encoding{
                _ = write_target.ensure_clear_and_borrow_mut(&ordered_cortical_ids[index], ordered_refs[index].get_number_elements());
            }
            else {
                ordered_refs[index].overwrite_neuron_data(write_target, ordered_cortical_ids[index], channel_index)?;
            }

        }
        Ok(())
    }

    //endregion


    pub fn blink_segments(&mut self)  {
        self.lower_left.blink_image();
        self.lower_middle.blink_image();
        self.lower_right.blink_image();
        self.middle_left.blink_image();
        self.center.blink_image();
        self.middle_right.blink_image();
        self.upper_left.blink_image();
        self.upper_middle.blink_image();
        self.upper_right.blink_image();
    }
}

impl std::fmt::Display for SegmentedImageFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SegmentedImageFrame()")
    }
}