use crate::data_types::descriptors::{CornerPoints, ImageXYPoint, ImageXYResolution};
use crate::data_types::{Percentage, Percentage2D, Percentage3D};
use feagi_data_structures::FeagiDataError;
use std::cmp;
use std::fmt::Display;

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
        GazeProperties::new(
            Percentage2D::new_identical_percentages(Percentage::new_from_0_1_unchecked(0.5)),
            Percentage::new_from_0_1_unchecked(0.5),
        )
    }

    pub fn calculate_source_corner_points_for_segmented_video_frame(
        &self,
        source_frame_resolution: ImageXYResolution,
    ) -> Result<[CornerPoints; 9], FeagiDataError> {
        if source_frame_resolution.width < 3 || source_frame_resolution.height < 3 {
            return Err(FeagiDataError::BadParameters(
                "Source frame width and height must be at least 3!".into(),
            )
            .into());
        }

        let center_corner_points =
            self.calculate_pixel_coordinates_of_center_corners(source_frame_resolution)?;
        Ok([
            CornerPoints::new(
                ImageXYPoint::new(0, center_corner_points.lower_right.y),
                ImageXYPoint::new(
                    center_corner_points.upper_left.x,
                    source_frame_resolution.height,
                ),
            )?,
            CornerPoints::new(
                center_corner_points.get_lower_left(),
                ImageXYPoint::new(
                    center_corner_points.lower_right.x,
                    source_frame_resolution.height,
                ),
            )?,
            CornerPoints::new(
                center_corner_points.lower_right,
                ImageXYPoint::new(
                    source_frame_resolution.width,
                    source_frame_resolution.height,
                ),
            )?,
            CornerPoints::new(
                ImageXYPoint::new(0, center_corner_points.upper_left.y),
                center_corner_points.get_lower_left(),
            )?,
            center_corner_points,
            CornerPoints::new(
                center_corner_points.get_upper_right(),
                ImageXYPoint::new(
                    source_frame_resolution.width,
                    center_corner_points.lower_right.y,
                ),
            )?,
            CornerPoints::new(ImageXYPoint::new(0, 0), center_corner_points.upper_left)?,
            CornerPoints::new(
                ImageXYPoint::new(center_corner_points.upper_left.x, 0),
                center_corner_points.get_upper_right(),
            )?,
            CornerPoints::new(
                ImageXYPoint::new(center_corner_points.lower_right.x, 0),
                ImageXYPoint::new(
                    source_frame_resolution.width,
                    center_corner_points.upper_left.y,
                ),
            )?,
        ])
    }

    fn calculate_pixel_coordinates_of_center_corners(
        &self,
        source_frame_resolution: ImageXYResolution,
    ) -> Result<CornerPoints, FeagiDataError> {
        let source_frame_width_height_f: (f32, f32) = (
            source_frame_resolution.width as f32,
            source_frame_resolution.height as f32,
        );
        let center_size_normalized_half_xy: (f32, f32) = (
            self.modulation_size.get_as_0_1() / 2.0,
            self.modulation_size.get_as_0_1() / 2.0,
        );

        // We use max / min to ensure that there is always a 1 pixel buffer along all edges for use in peripheral vision (since we cannot use a resolution of 0)
        let bottom_pixel: usize = cmp::min(
            source_frame_resolution.height as usize - 1,
            ((self.eccentricity_location_xy.b.get_as_0_1() + center_size_normalized_half_xy.1)
                * source_frame_width_height_f.1)
                .floor() as usize,
        );
        let top_pixel: usize = cmp::max(
            1,
            ((self.eccentricity_location_xy.b.get_as_0_1() - center_size_normalized_half_xy.1)
                * source_frame_width_height_f.1)
                .floor() as usize,
        );
        let left_pixel: usize = cmp::max(
            1,
            ((self.eccentricity_location_xy.a.get_as_0_1() - center_size_normalized_half_xy.0)
                * source_frame_width_height_f.0)
                .floor() as usize,
        );
        let right_pixel: usize = cmp::min(
            source_frame_resolution.width as usize - 1,
            ((self.eccentricity_location_xy.a.get_as_0_1() + center_size_normalized_half_xy.0)
                * source_frame_width_height_f.0)
                .floor() as usize,
        );

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
