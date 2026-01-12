use crate::data_types::{Percentage, Percentage2D};
use std::fmt::Display;

#[derive(PartialEq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct ImageFilteringSettings {
    pub(crate) brightness: Percentage,
    pub(crate) contrast: Percentage,
    pub(crate) per_pixel_diff_threshold: Percentage2D, // Lower, Upper
    pub(crate) image_diff_threshold: Percentage2D,
}

impl ImageFilteringSettings {
    pub fn new(
        brightness: Percentage,
        contrast: Percentage,
        per_pixel_diff_threshold: Percentage2D,
        image_diff_threshold: Percentage2D,
    ) -> Self {
        ImageFilteringSettings {
            brightness,
            contrast,
            per_pixel_diff_threshold,
            image_diff_threshold,
        }
    }

    /// Get the brightness value
    pub fn brightness(&self) -> &Percentage {
        &self.brightness
    }

    /// Get a mutable reference to the brightness value
    pub fn brightness_mut(&mut self) -> &mut Percentage {
        &mut self.brightness
    }

    /// Get the contrast value
    pub fn contrast(&self) -> &Percentage {
        &self.contrast
    }

    /// Get a mutable reference to the contrast value
    pub fn contrast_mut(&mut self) -> &mut Percentage {
        &mut self.contrast
    }

    /// Get the per-pixel diff threshold (lower, upper)
    pub fn per_pixel_diff_threshold(&self) -> &Percentage2D {
        &self.per_pixel_diff_threshold
    }

    /// Get a mutable reference to the per-pixel diff threshold
    pub fn per_pixel_diff_threshold_mut(&mut self) -> &mut Percentage2D {
        &mut self.per_pixel_diff_threshold
    }

    pub fn image_diff_threshold(&self) -> &Percentage2D {
        &self.image_diff_threshold
    }

    pub fn image_diff_threshold_mut(&mut self) -> &mut Percentage2D {
        &mut self.image_diff_threshold
    }
}

impl Display for ImageFilteringSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ImageFilteringSettings: Brightness {}, Contrast {}, Per Pixel Diff Threshold ({}, {})",
            self.brightness,
            self.contrast,
            self.per_pixel_diff_threshold.a,
            self.per_pixel_diff_threshold.b
        )
    }
}

impl Default for ImageFilteringSettings {
    fn default() -> Self {
        ImageFilteringSettings::new(
            Percentage::new_from_0_1(0.5).unwrap(),
            Percentage::new_from_0_1(0.5).unwrap(),
            Percentage2D::new_identical_percentages(Percentage::new_from_0_1(0.5).unwrap()),
            Percentage2D::new_identical_percentages(Percentage::new_from_0_1(0.5).unwrap()),
        )
    }
}
