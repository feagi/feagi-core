//! Paired image + colorized-mask preview for an ingested segmentation corpus.
//!
//! Returns resized RGB and label frames for the Trainer source step. This is a
//! view of the source pairs, not a second ingest path.

use image::{ImageFormat, Rgb, RgbImage};
use serde::{Deserialize, Serialize};

use crate::adapters::time_series::PreviewClassCount;
use crate::error::TrainerError;
use crate::plugins::DatasetSource;

use super::{ImageFolderSegmentationAdapter, ImageLabelPair};

/// Official Cityscapes `trainId` colors (Cordts et al.), index = class id.
const CITYSCAPES_TRAIN_ID_COLORS: [[u8; 3]; 19] = [
    [128, 64, 128],
    [244, 35, 232],
    [70, 70, 70],
    [102, 102, 156],
    [190, 153, 153],
    [153, 153, 153],
    [250, 170, 30],
    [220, 220, 0],
    [107, 142, 35],
    [152, 251, 152],
    [70, 130, 180],
    [220, 20, 60],
    [255, 0, 0],
    [0, 0, 142],
    [0, 0, 70],
    [0, 60, 100],
    [0, 80, 100],
    [0, 0, 230],
    [119, 11, 32],
];

/// One image + colorized mask pair on a preview page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentationPreviewFrame {
    /// File name (city/file for Cityscapes) of the RGB image.
    pub name: String,
    /// Resized RGB PNG bytes.
    pub image_png: Vec<u8>,
    /// Colorized label PNG bytes at the same preview size.
    pub mask_png: Vec<u8>,
    /// Pixel counts per class id present on this mask (ignore label omitted).
    pub classes_present: Vec<PreviewClassCount>,
}

/// A page of paired frames plus the pair count for paging.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentationPreview {
    /// Zero-based index of the first returned pair.
    pub frame_offset: u64,
    /// Number of discovered image/mask pairs.
    pub total_frames: u64,
    /// Preview resize width used for both image and mask.
    pub preview_width: u32,
    /// Preview resize height used for both image and mask.
    pub preview_height: u32,
    /// Configured semantic class count (excluding ignore).
    pub class_count: u32,
    /// Configured ignore label (omitted from class chips).
    pub ignore_label: u8,
    /// Image + colorized-mask pairs for this page.
    pub frames: Vec<SegmentationPreviewFrame>,
}

/// Builds one page of image + colorized-mask pairs from the adapter layout.
pub fn preview_pairs(
    adapter: &ImageFolderSegmentationAdapter,
    source: &DatasetSource,
    frame_offset: usize,
    frame_count: usize,
) -> Result<SegmentationPreview, TrainerError> {
    if frame_count == 0 {
        return Err(TrainerError::Config(
            "preview frame_count must be greater than zero".to_string(),
        ));
    }
    if adapter.config.feed_width == 0 || adapter.config.feed_height == 0 {
        return Err(TrainerError::Config(
            "preview requires non-zero feed_width and feed_height".to_string(),
        ));
    }
    if adapter.config.class_count == 0 {
        return Err(TrainerError::Config(
            "preview requires a non-zero class_count".to_string(),
        ));
    }
    let root = ImageFolderSegmentationAdapter::dataset_root(source)?;
    let pairs = adapter.discover_pairs(root)?;
    if pairs.is_empty() {
        return Err(TrainerError::Parse(
            "segmentation preview found no image/mask pairs".to_string(),
        ));
    }
    let total_frames = pairs.len();
    if frame_offset >= total_frames {
        return Err(TrainerError::Config(format!(
            "preview frame_offset {frame_offset} is past the {total_frames} image/mask pairs"
        )));
    }
    let width = adapter.config.feed_width;
    let height = adapter.config.feed_height;
    let mut frames = Vec::new();
    for pair in pairs.iter().skip(frame_offset).take(frame_count) {
        frames.push(load_preview_frame(
            adapter,
            pair,
            width,
            height,
            adapter.config.class_count,
            adapter.config.ignore_label,
        )?);
    }
    if frames.is_empty() {
        return Err(TrainerError::Parse(
            "preview page is empty after applying frame_offset".to_string(),
        ));
    }
    Ok(SegmentationPreview {
        frame_offset: frame_offset as u64,
        total_frames: total_frames as u64,
        preview_width: width,
        preview_height: height,
        class_count: adapter.config.class_count,
        ignore_label: adapter.config.ignore_label,
        frames,
    })
}

fn load_preview_frame(
    adapter: &ImageFolderSegmentationAdapter,
    pair: &ImageLabelPair,
    width: u32,
    height: u32,
    class_count: u32,
    ignore_label: u8,
) -> Result<SegmentationPreviewFrame, TrainerError> {
    let image_png =
        ImageFolderSegmentationAdapter::load_resized_png_bytes(&pair.image_path, width, height)?;
    let labels = adapter.load_resized_label_mask(pair, width, height)?;
    if labels.len() != (width as usize) * (height as usize) {
        return Err(TrainerError::Parse(format!(
            "label mask size mismatch for '{}'",
            pair.label_path.display()
        )));
    }
    let mask_png = colorize_mask_png(&labels, width, height, class_count, Some(ignore_label))?;
    Ok(SegmentationPreviewFrame {
        name: pair_display_name(pair),
        image_png,
        mask_png,
        classes_present: class_pixel_counts(&labels, ignore_label),
    })
}

fn pair_display_name(pair: &ImageLabelPair) -> String {
    let file = pair
        .image_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("image");
    match pair
        .image_path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
    {
        Some(city) if !city.is_empty() => format!("{city}/{file}"),
        _ => file.to_string(),
    }
}

fn class_pixel_counts(labels: &[u8], ignore_label: u8) -> Vec<PreviewClassCount> {
    let mut tallies = std::collections::BTreeMap::<u8, u64>::new();
    for &label in labels {
        if label == ignore_label {
            continue;
        }
        *tallies.entry(label).or_insert(0) += 1;
    }
    tallies
        .into_iter()
        .map(|(label, count)| PreviewClassCount {
            label: label.to_string(),
            count,
        })
        .collect()
}

/// Paints trainId labels with the Cityscapes palette. Unknown ids fail.
pub fn colorize_mask_png(
    labels: &[u8],
    width: u32,
    height: u32,
    class_count: u32,
    ignore_label: Option<u8>,
) -> Result<Vec<u8>, TrainerError> {
    if class_count as usize > CITYSCAPES_TRAIN_ID_COLORS.len() {
        return Err(TrainerError::Config(format!(
            "preview class_count {class_count} exceeds the {}-class Cityscapes palette",
            CITYSCAPES_TRAIN_ID_COLORS.len()
        )));
    }
    let mut rgb = RgbImage::new(width, height);
    for (index, &label) in labels.iter().enumerate() {
        let x = (index as u32) % width;
        let y = (index as u32) / width;
        let color = if ignore_label == Some(label) {
            [0, 0, 0]
        } else if (label as u32) < class_count {
            CITYSCAPES_TRAIN_ID_COLORS[label as usize]
        } else {
            return Err(TrainerError::Parse(format!(
                "mask label {label} is outside class_count {class_count}"
            )));
        };
        rgb.put_pixel(x, y, Rgb(color));
    }
    let mut out = Vec::new();
    rgb.write_to(&mut std::io::Cursor::new(&mut out), ImageFormat::Png)
        .map_err(|e| TrainerError::Parse(format!("cannot encode colorized mask png: {e}")))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::image_folder_segmentation::{
        ImageFolderSegmentationConfig, SegmentationDatasetLayout,
    };
    use crate::contracts::common::{Split, SplitId};
    use crate::plugins::DatasetSource;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn write_rgb_png(path: &Path, width: u32, height: u32, color: [u8; 3]) {
        let img = image::RgbImage::from_pixel(width, height, image::Rgb(color));
        img.save(path).expect("write rgb");
    }

    fn write_label_png(path: &Path, width: u32, height: u32, labels: &[u8]) {
        assert_eq!(labels.len(), (width as usize) * (height as usize));
        let img = image::GrayImage::from_raw(width, height, labels.to_vec()).expect("label");
        img.save(path).expect("write label");
    }

    fn paired_config() -> ImageFolderSegmentationConfig {
        ImageFolderSegmentationConfig {
            dataset_name: "mini_seg".to_string(),
            layout: SegmentationDatasetLayout::PairedFolders,
            split: Split::Train,
            split_id: SplitId("train".to_string()),
            class_count: 4,
            ignore_label: 255,
            feed_width: 2,
            feed_height: 2,
            cityscapes_split: None,
            images_subdir: Some("images".to_string()),
            labels_subdir: Some("labels".to_string()),
            train_id_remap: None,
        }
    }

    #[test]
    fn preview_returns_image_and_colorized_mask_pairs() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let images = root.join("images");
        let labels = root.join("labels");
        fs::create_dir_all(&images).expect("mkdir");
        fs::create_dir_all(&labels).expect("mkdir");
        write_rgb_png(&images.join("a.png"), 2, 2, [10, 20, 30]);
        write_label_png(&labels.join("a.png"), 2, 2, &[0, 1, 2, 255]);

        let adapter = ImageFolderSegmentationAdapter::new(paired_config());
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let preview = adapter.preview(&source, 0, 1).expect("preview");
        assert_eq!(preview.total_frames, 1);
        assert_eq!(preview.frame_offset, 0);
        assert_eq!(preview.preview_width, 2);
        assert_eq!(preview.preview_height, 2);
        assert_eq!(preview.frames.len(), 1);
        assert_eq!(preview.frames[0].name, "images/a.png");
        assert!(preview.frames[0].image_png.starts_with(&[137, 80, 78, 71]));
        assert!(preview.frames[0].mask_png.starts_with(&[137, 80, 78, 71]));
        assert_eq!(
            preview.frames[0]
                .classes_present
                .iter()
                .map(|entry| (entry.label.as_str(), entry.count))
                .collect::<Vec<_>>(),
            vec![("0", 1), ("1", 1), ("2", 1)]
        );
    }

    #[test]
    fn preview_pages_from_frame_offset() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let images = root.join("images");
        let labels = root.join("labels");
        fs::create_dir_all(&images).expect("mkdir");
        fs::create_dir_all(&labels).expect("mkdir");
        write_rgb_png(&images.join("a.png"), 2, 2, [1, 2, 3]);
        write_label_png(&labels.join("a.png"), 2, 2, &[0, 0, 0, 0]);
        write_rgb_png(&images.join("b.png"), 2, 2, [4, 5, 6]);
        write_label_png(&labels.join("b.png"), 2, 2, &[1, 1, 1, 1]);
        write_rgb_png(&images.join("c.png"), 2, 2, [7, 8, 9]);
        write_label_png(&labels.join("c.png"), 2, 2, &[2, 2, 2, 2]);

        let adapter = ImageFolderSegmentationAdapter::new(paired_config());
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let preview = adapter.preview(&source, 1, 2).expect("page");
        assert_eq!(preview.frame_offset, 1);
        assert_eq!(preview.total_frames, 3);
        assert_eq!(preview.frames.len(), 2);
        assert_eq!(preview.frames[0].name, "images/b.png");
        assert_eq!(preview.frames[1].name, "images/c.png");
        assert_eq!(preview.frames[0].classes_present[0].label, "1");
    }

    #[test]
    fn preview_rejects_unknown_class_id() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let images = root.join("images");
        let labels = root.join("labels");
        fs::create_dir_all(&images).expect("mkdir");
        fs::create_dir_all(&labels).expect("mkdir");
        write_rgb_png(&images.join("a.png"), 2, 2, [1, 2, 3]);
        write_label_png(&labels.join("a.png"), 2, 2, &[0, 9, 0, 0]);

        let adapter = ImageFolderSegmentationAdapter::new(paired_config());
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let error = adapter.preview(&source, 0, 1).expect_err("unknown id");
        assert!(error.to_string().contains("outside class_count"));
    }

    #[test]
    fn preview_rejects_zero_frame_count() {
        let adapter = ImageFolderSegmentationAdapter::new(paired_config());
        let source = DatasetSource {
            uri: "/tmp".to_string(),
            bytes: Vec::new(),
        };
        let error = adapter.preview(&source, 0, 0).expect_err("zero");
        assert!(error.to_string().contains("frame_count"));
    }

    #[test]
    fn cityscapes_preview_returns_named_train_id_pair() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let img_dir = root.join("leftImg8bit").join("train").join("aachen");
        let lbl_dir = root.join("gtFine").join("train").join("aachen");
        fs::create_dir_all(&img_dir).expect("mkdir");
        fs::create_dir_all(&lbl_dir).expect("mkdir");
        write_rgb_png(
            &img_dir.join("aachen_000000_000019_leftImg8bit.png"),
            2,
            2,
            [10, 20, 30],
        );
        write_label_png(
            &lbl_dir.join("aachen_000000_000019_gtFine_labelTrainIds.png"),
            2,
            2,
            &[0, 13, 255, 8],
        );

        let config = ImageFolderSegmentationConfig {
            dataset_name: "cityscapes".to_string(),
            layout: SegmentationDatasetLayout::CityscapesFine,
            split: Split::Train,
            split_id: SplitId("train".to_string()),
            class_count: 19,
            ignore_label: 255,
            feed_width: 2,
            feed_height: 2,
            cityscapes_split: Some("train".to_string()),
            images_subdir: None,
            labels_subdir: None,
            train_id_remap: None,
        };
        let adapter = ImageFolderSegmentationAdapter::new(config);
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let preview = adapter.preview(&source, 0, 1).expect("preview");
        assert_eq!(preview.total_frames, 1);
        assert_eq!(
            preview.frames[0].name,
            "aachen/aachen_000000_000019_leftImg8bit.png"
        );
        assert_eq!(
            preview.frames[0]
                .classes_present
                .iter()
                .map(|entry| entry.label.as_str())
                .collect::<Vec<_>>(),
            vec!["0", "8", "13"]
        );
    }

    #[test]
    fn cityscapes_preview_requires_matching_label_train_ids() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let img_dir = root.join("leftImg8bit").join("train").join("aachen");
        fs::create_dir_all(&img_dir).expect("mkdir");
        fs::create_dir_all(root.join("gtFine").join("train").join("aachen")).expect("mkdir");
        write_rgb_png(
            &img_dir.join("aachen_000000_000019_leftImg8bit.png"),
            4,
            2,
            [10, 20, 30],
        );

        let config = ImageFolderSegmentationConfig {
            dataset_name: "cityscapes".to_string(),
            layout: SegmentationDatasetLayout::CityscapesFine,
            split: Split::Train,
            split_id: SplitId("train".to_string()),
            class_count: 19,
            ignore_label: 255,
            feed_width: 2,
            feed_height: 2,
            cityscapes_split: Some("train".to_string()),
            images_subdir: None,
            labels_subdir: None,
            train_id_remap: None,
        };
        let adapter = ImageFolderSegmentationAdapter::new(config);
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let error = adapter.preview(&source, 0, 1).expect_err("missing label");
        assert!(error.to_string().contains("no Cityscapes image/mask pairs"));
    }
}
