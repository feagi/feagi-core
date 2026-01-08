use crate::data_types::{Percentage, Percentage2D};

#[derive(PartialEq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct ImageFilteringSettings {
    brightness: Percentage,
    contrast: Percentage,
    per_pixel_diff_threshold: Percentage2D // Lower, Upper
}

impl ImageFilteringSettings {

    pub fn new(brightness: Percentage, contrast: Percentage,
    per_pixel_diff_threshold: Percentage2D) -> Self {

        ImageFilteringSettings {
            brightness,
            contrast,
            per_pixel_diff_threshold,
        }
    }
}