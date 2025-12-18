use std::cmp;
use std::fmt::Display;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::cortical_area::CorticalAreaDimensions;
use crate::data_types::{Percentage, Percentage2D, Percentage3D};
use crate::data_types::descriptors::{CornerPoints, ImageXYPoint, ImageXYResolution};

/// Properties defining the center region of a segmented vision frame
///
/// This structure defines the coordinates and size of the central region
/// in a normalized coordinate space (0.0 to 1.0).
#[derive(PartialEq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct GazeProperties {
    pub(crate) eccentricity_location_xy: Percentage2D,
    pub(crate) modulation_size: Percentage,
}

impl GazeProperties {

    pub fn new(eccentricity_center_xy: Percentage2D, modularity_size: Percentage) -> Self {
        GazeProperties {
            eccentricity_location_xy: eccentricity_center_xy,
            modulation_size: modularity_size,
        }
    }

    pub fn new_from_3d(percentage3d: Percentage3D) -> Self {
        let eccentricity_location_xy = Percentage2D::new(percentage3d.a, percentage3d.b);
        GazeProperties::new(eccentricity_location_xy, percentage3d.c)
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
        GazeProperties::new(Percentage2D::new_identical_percentages(Percentage::new_from_0_1_unchecked(0.5)),
                            Percentage::new_from_0_1_unchecked(0.5))
    }

    pub fn calculate_source_corner_points_for_segmented_video_frame(&self, source_frame_resolution: ImageXYResolution, destination_segmented_center_cortical_dimensions: CorticalAreaDimensions) -> Result<[CornerPoints; 9], FeagiDataError> {
        if source_frame_resolution.width < 3 || source_frame_resolution.height < 3 {
            return Err(FeagiDataError::BadParameters("Source frame width and height must be at least 3!".into()).into())
        }


        let center_corner_points = self.calculate_pixel_coordinates_of_center_corners(source_frame_resolution, destination_segmented_center_cortical_dimensions)?;
        Ok([
            CornerPoints::new(ImageXYPoint::new(0, center_corner_points.lower_right.y), ImageXYPoint::new(center_corner_points.upper_left.x, source_frame_resolution.height))?,
            CornerPoints::new(center_corner_points.get_lower_left(), ImageXYPoint::new(center_corner_points.lower_right.x, source_frame_resolution.height))?,
            CornerPoints::new(center_corner_points.lower_right, ImageXYPoint::new(source_frame_resolution.width, source_frame_resolution.height))?,
            CornerPoints::new(ImageXYPoint::new(0, center_corner_points.upper_left.y), center_corner_points.get_lower_left())?,
            center_corner_points,
            CornerPoints::new(center_corner_points.get_upper_right(), ImageXYPoint::new(source_frame_resolution.width, center_corner_points.lower_right.y))?,
            CornerPoints::new(ImageXYPoint::new(0,0), center_corner_points.upper_left)?,
            CornerPoints::new(ImageXYPoint::new(center_corner_points.upper_left.x, 0), center_corner_points.get_upper_right())?,
            CornerPoints::new(ImageXYPoint::new(center_corner_points.lower_right.x, 0), ImageXYPoint::new(source_frame_resolution.width, center_corner_points.upper_left.y))?,
        ])
    }

    fn calculate_pixel_coordinates_of_center_corners(&self, source_frame_resolution: ImageXYResolution, destination_segmented_center_cortical_dimensions: CorticalAreaDimensions) -> Result<CornerPoints, FeagiDataError> {

        let source_frame_center_normal: (f32, f32) = (
            0.5 + self.eccentricity_location_xy.a.get_as_0_1(),
            0.5 - self.eccentricity_location_xy.b.get_as_0_1(), // Flip y eccentricity direction
        );

        let source_frame_max_off_center_normal: (f32, f32) = {

            // To keep aspect ratio to the center cortical area XY, but also not allow the size to grow past it
            let source_frame_max_offset_normal: (f32, f32) = {
                if destination_segmented_center_cortical_dimensions.width > destination_segmented_center_cortical_dimensions.height {
                    // widescreen
                    let max_cortical_length = destination_segmented_center_cortical_dimensions.width;
                    let min_cortical_length = destination_segmented_center_cortical_dimensions.height;
                    let max_offset = ((min_cortical_length as f32) / (max_cortical_length as f32)) * 0.5;
                    (
                        max_offset,
                        0.5
                    )
                } else {
                    // portrait / square
                    let max_cortical_length = destination_segmented_center_cortical_dimensions.height;
                    let min_cortical_length = destination_segmented_center_cortical_dimensions.width;
                    let max_offset = ((min_cortical_length as f32) / (max_cortical_length as f32)) * 0.5;
                    (
                        0.5,
                        max_offset
                    )
                }
            };

            (source_frame_max_offset_normal.0 * self.modulation_size.get_as_0_1(), source_frame_max_offset_normal.1 * self.modulation_size.get_as_0_1())
        };

        // Remember that in an image, Y increases downward
        let left_position_normal: f32 = source_frame_center_normal.0 - source_frame_max_off_center_normal.0;
        let top_position_normal: f32 = source_frame_center_normal.1 - source_frame_max_off_center_normal.1;
        let right_position_normal: f32 = source_frame_center_normal.0 + source_frame_max_off_center_normal.0;
        let bottom_position_normal: f32 = source_frame_center_normal.1 + source_frame_max_off_center_normal.1;

        let source_frame_width_height_pixel: (f32, f32) = (source_frame_resolution.width as f32, source_frame_resolution.height as f32);

        let left_position_pixel: f32 = left_position_normal * source_frame_width_height_pixel.0;
        let top_position_pixel: f32 = top_position_normal * source_frame_width_height_pixel.1;
        let right_position_pixel: f32 = right_position_normal + source_frame_width_height_pixel.0;
        let bottom_position_pixel: f32 = bottom_position_normal + source_frame_width_height_pixel.1;

        let left_pixel = cmp::max(1, left_position_pixel.floor() as i32);
        let top_pixel = cmp::max(1, top_position_pixel.floor() as i32);
        let right_pixel = cmp::min(source_frame_resolution.width as i32 - 1, right_position_pixel.floor() as i32);
        let bottom_pixel = cmp::min(source_frame_resolution.height as i32 - 1, bottom_position_pixel.floor() as i32);

        let top_left = ImageXYPoint::new(left_pixel as u32, top_pixel as u32);
        let bottom_right = ImageXYPoint::new(right_pixel as u32, bottom_pixel as u32);

        let corner_points: CornerPoints = CornerPoints::new(top_left, bottom_right)?;
        Ok(corner_points)
    }
}

impl Display for GazeProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "GazeProperties(TODO)") // TODO
    }
}