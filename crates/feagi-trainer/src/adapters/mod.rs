//! Built-in adapter implementations.

pub mod image_folder_segmentation;
pub mod tabular_csv;

pub use image_folder_segmentation::{
    ImageFolderSegmentationAdapter, ImageFolderSegmentationConfig, SegmentationDatasetLayout,
};
pub use tabular_csv::{TabularCsvAdapter, TabularCsvConfig};
