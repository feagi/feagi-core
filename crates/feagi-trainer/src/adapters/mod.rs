//! Built-in adapter implementations.

pub mod class_keep;
pub mod image_folder_segmentation;
pub mod tabular_csv;
pub mod time_series;

pub use image_folder_segmentation::{
    ImageFolderSegmentationAdapter, ImageFolderSegmentationConfig, ImageLabelPair,
    SegmentationDatasetLayout, SegmentationPreview, SegmentationPreviewFrame,
};
pub use tabular_csv::{TabularCsvAdapter, TabularCsvConfig};
pub use time_series::{
    IncompleteWindowPolicy, PreviewClassCount, TimeSeriesNormalize, TimeSeriesPackageAdapter,
    TimeSeriesPackageConfig, TimeSeriesPresentation, TimeSeriesPreview, TimeSeriesPreviewFrame,
    TimeSeriesSourceKind, TimeSeriesWindowConfig, UnknownLabelPolicy, WfdbChannelMap,
};

/// Fixture writers used by crate and integration tests.
pub mod time_series_test {
    pub use super::time_series::write_wfdb_record;
}
