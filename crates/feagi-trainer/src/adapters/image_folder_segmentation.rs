//! Image-folder semantic segmentation adapter — maps paired image/label files into `IRSample`s.
//!
//! Dataset-specific layout is selected by explicit configuration (`cityscapes_fine` or
//! `paired_folders`). All filesystem access uses `source.uri` as the dataset root; `source.bytes`
//! is unused.

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::ImageFormat;
use serde::{Deserialize, Serialize};

use crate::contracts::common::{
    ContentHash, DatasetAssetId, DatasetVersionId, MetadataValue, Modality, OutputType, PluginId,
    PluginRef, SampleId, Split, SplitId,
};
use crate::contracts::dataset_manifest::SCHEMA_VERSION as MANIFEST_SCHEMA_VERSION;
use crate::contracts::ir_sample::SCHEMA_VERSION as IR_SCHEMA_VERSION;
use crate::contracts::{DatasetManifest, IRSample, Payload, SplitDescriptor, TypedTarget};
use crate::error::TrainerError;
use crate::plugins::{AdapterPlugin, DatasetSource, ValidationReport};

/// Supported on-disk folder layouts for image + per-pixel label pairs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentationDatasetLayout {
    /// Cityscapes fine annotations (`leftImg8bit` + `gtFine` `*_labelTrainIds.png`).
    CityscapesFine,
    /// Generic paired folders under the dataset root.
    PairedFolders,
}

/// Explicit configuration for the image-folder segmentation adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageFolderSegmentationConfig {
    /// Logical dataset name, used to derive the dataset version id and asset id.
    pub dataset_name: String,
    /// On-disk layout selector.
    pub layout: SegmentationDatasetLayout,
    /// Split role all discovered pairs belong to.
    pub split: Split,
    /// Split id all discovered pairs are assigned to.
    pub split_id: SplitId,
    /// Number of semantic classes (excluding the ignore label).
    pub class_count: u32,
    /// Per-pixel label value excluded from metrics and reward.
    pub ignore_label: u8,
    /// Width fed to the vision encoder after resize.
    pub feed_width: u32,
    /// Height fed to the vision encoder after resize.
    pub feed_height: u32,
    /// Cityscapes split folder name (`train` or `val`) when `layout = cityscapes_fine`.
    #[serde(default)]
    pub cityscapes_split: Option<String>,
    /// Relative images directory when `layout = paired_folders`.
    #[serde(default)]
    pub images_subdir: Option<String>,
    /// Relative labels directory when `layout = paired_folders`.
    #[serde(default)]
    pub labels_subdir: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImageLabelPair {
    image_path: PathBuf,
    label_path: PathBuf,
}

/// Adapter that converts paired image/label folders into segmentation `IRSample`s.
#[derive(Debug, Clone)]
pub struct ImageFolderSegmentationAdapter {
    config: ImageFolderSegmentationConfig,
}

impl ImageFolderSegmentationAdapter {
    /// Stable plugin id for this adapter.
    pub const PLUGIN_ID: &'static str = "image_folder_segmentation";

    /// Creates a new adapter from explicit configuration.
    pub fn new(config: ImageFolderSegmentationConfig) -> Self {
        Self { config }
    }

    fn dataset_root(source: &DatasetSource) -> Result<&Path, TrainerError> {
        let root = Path::new(&source.uri);
        if !root.is_dir() {
            return Err(TrainerError::Parse(format!(
                "image-folder segmentation requires a directory dataset root, got '{}'",
                source.uri
            )));
        }
        Ok(root)
    }

    fn dataset_asset_id(&self) -> DatasetAssetId {
        DatasetAssetId(format!("local:{}", self.config.dataset_name))
    }

    fn dataset_version_id(&self, content_fingerprint: &str) -> DatasetVersionId {
        let short = content_fingerprint
            .rsplit(':')
            .next()
            .unwrap_or(content_fingerprint);
        DatasetVersionId(format!("{}@{}", self.config.dataset_name, short))
    }

    fn fingerprint_pairs(pairs: &[ImageLabelPair]) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for pair in pairs {
            pair.image_path.to_string_lossy().hash(&mut hasher);
            pair.label_path.to_string_lossy().hash(&mut hasher);
        }
        format!("siphash64:{:016x}", hasher.finish())
    }

    fn discover_pairs(&self, root: &Path) -> Result<Vec<ImageLabelPair>, TrainerError> {
        match self.config.layout {
            SegmentationDatasetLayout::CityscapesFine => self.discover_cityscapes_pairs(root),
            SegmentationDatasetLayout::PairedFolders => self.discover_paired_folders(root),
        }
    }

    fn discover_cityscapes_pairs(&self, root: &Path) -> Result<Vec<ImageLabelPair>, TrainerError> {
        let split = self.config.cityscapes_split.as_deref().ok_or_else(|| {
            TrainerError::Config(
                "cityscapes_fine layout requires cityscapes_split (train or val)".to_string(),
            )
        })?;
        let img_root = root.join("leftImg8bit").join(split);
        let lbl_root = root.join("gtFine").join(split);
        if !img_root.is_dir() {
            return Err(TrainerError::Parse(format!(
                "missing Cityscapes image directory '{}'",
                img_root.display()
            )));
        }
        if !lbl_root.is_dir() {
            return Err(TrainerError::Parse(format!(
                "missing Cityscapes label directory '{}'",
                lbl_root.display()
            )));
        }

        let mut pairs = Vec::new();
        for city_entry in std::fs::read_dir(&img_root).map_err(|e| {
            TrainerError::Parse(format!("cannot read '{}': {e}", img_root.display()))
        })? {
            let city_entry = city_entry.map_err(|e| TrainerError::Parse(e.to_string()))?;
            if !city_entry
                .file_type()
                .map_err(|e| TrainerError::Parse(e.to_string()))?
                .is_dir()
            {
                continue;
            }
            let city_name = city_entry.file_name();
            let city_img_dir = city_entry.path();
            let city_lbl_dir = lbl_root.join(&city_name);
            for img_entry in std::fs::read_dir(&city_img_dir).map_err(|e| {
                TrainerError::Parse(format!("cannot read '{}': {e}", city_img_dir.display()))
            })? {
                let img_entry = img_entry.map_err(|e| TrainerError::Parse(e.to_string()))?;
                let img_path = img_entry.path();
                if img_path.extension().and_then(|s| s.to_str()) != Some("png") {
                    continue;
                }
                let file_name = img_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| TrainerError::Parse("invalid image file name".to_string()))?;
                if !file_name.ends_with("_leftImg8bit.png") {
                    continue;
                }
                let stem = file_name.trim_end_matches("_leftImg8bit.png");
                let label_name = format!("{stem}_gtFine_labelTrainIds.png");
                let label_path = city_lbl_dir.join(label_name);
                if !label_path.is_file() {
                    return Err(TrainerError::Parse(format!(
                        "missing Cityscapes label for '{}'",
                        img_path.display()
                    )));
                }
                pairs.push(ImageLabelPair {
                    image_path: img_path,
                    label_path,
                });
            }
        }
        pairs.sort_by(|a, b| a.image_path.cmp(&b.image_path));
        Ok(pairs)
    }

    fn discover_paired_folders(&self, root: &Path) -> Result<Vec<ImageLabelPair>, TrainerError> {
        let images_subdir = self.config.images_subdir.as_deref().ok_or_else(|| {
            TrainerError::Config("paired_folders layout requires images_subdir".to_string())
        })?;
        let labels_subdir = self.config.labels_subdir.as_deref().ok_or_else(|| {
            TrainerError::Config("paired_folders layout requires labels_subdir".to_string())
        })?;
        let img_root = root.join(images_subdir);
        let lbl_root = root.join(labels_subdir);
        if !img_root.is_dir() || !lbl_root.is_dir() {
            return Err(TrainerError::Parse(format!(
                "paired_folders requires directories '{}' and '{}'",
                img_root.display(),
                lbl_root.display()
            )));
        }

        let mut pairs = Vec::new();
        for img_entry in walk_png_files(&img_root)? {
            let rel = img_entry
                .strip_prefix(&img_root)
                .map_err(|e| TrainerError::Parse(e.to_string()))?;
            let label_path = lbl_root.join(rel);
            if !label_path.is_file() {
                return Err(TrainerError::Parse(format!(
                    "missing label file for '{}'",
                    img_entry.display()
                )));
            }
            pairs.push(ImageLabelPair {
                image_path: img_entry,
                label_path,
            });
        }
        pairs.sort_by(|a, b| a.image_path.cmp(&b.image_path));
        Ok(pairs)
    }

    fn load_resized_png_bytes(
        path: &Path,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, TrainerError> {
        let bytes = std::fs::read(path)
            .map_err(|e| TrainerError::Parse(format!("cannot read '{}': {e}", path.display())))?;
        let img = image::load_from_memory_with_format(&bytes, ImageFormat::Png).map_err(|e| {
            TrainerError::Parse(format!("cannot decode png '{}': {e}", path.display()))
        })?;
        let resized = img.resize_exact(width, height, FilterType::Nearest);
        let mut out = Vec::new();
        resized
            .write_to(&mut std::io::Cursor::new(&mut out), ImageFormat::Png)
            .map_err(|e| TrainerError::Parse(format!("cannot encode resized png: {e}")))?;
        Ok(out)
    }

    fn load_resized_label_mask(
        path: &Path,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, TrainerError> {
        let bytes = std::fs::read(path)
            .map_err(|e| TrainerError::Parse(format!("cannot read '{}': {e}", path.display())))?;
        let img = image::load_from_memory_with_format(&bytes, ImageFormat::Png).map_err(|e| {
            TrainerError::Parse(format!("cannot decode label png '{}': {e}", path.display()))
        })?;
        let gray = img.to_luma8();
        let resized = image::imageops::resize(&gray, width, height, FilterType::Nearest);
        Ok(resized.into_raw())
    }
}

fn walk_png_files(root: &Path) -> Result<Vec<PathBuf>, TrainerError> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)
            .map_err(|e| TrainerError::Parse(format!("cannot read '{}': {e}", dir.display())))?
        {
            let entry = entry.map_err(|e| TrainerError::Parse(e.to_string()))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("png") {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

impl AdapterPlugin for ImageFolderSegmentationAdapter {
    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId(Self::PLUGIN_ID.to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn discover(&self, source: &DatasetSource) -> Result<DatasetManifest, TrainerError> {
        let root = Self::dataset_root(source)?;
        let pairs = self.discover_pairs(root)?;
        let content_fingerprint = Self::fingerprint_pairs(&pairs);
        let dataset_version_id = self.dataset_version_id(&content_fingerprint);

        Ok(DatasetManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            dataset_version_id,
            dataset_asset_id: self.dataset_asset_id(),
            dataset_version: "1.0.0".to_string(),
            source_uri: source.uri.clone(),
            content_hash: ContentHash(content_fingerprint),
            schema_fingerprint: ContentHash(format!(
                "segmentation:{}x{}:{}",
                self.config.feed_width, self.config.feed_height, self.config.class_count
            )),
            modality: Modality::Image,
            output_type: OutputType::SegmentationMask,
            splits: vec![SplitDescriptor {
                id: self.config.split_id.clone(),
                split: self.config.split.clone(),
                sample_count: pairs.len() as u64,
            }],
            metadata: BTreeMap::new(),
        })
    }

    fn validate(&self, manifest: &DatasetManifest) -> Result<ValidationReport, TrainerError> {
        let mut issues = Vec::new();
        if manifest.modality != Modality::Image {
            issues.push(format!("modality {:?} is not Image", manifest.modality));
        }
        if manifest.output_type != OutputType::SegmentationMask {
            issues.push(format!(
                "output_type {:?} is not SegmentationMask",
                manifest.output_type
            ));
        }
        if self.config.feed_width == 0 || self.config.feed_height == 0 {
            issues.push("feed_width and feed_height must be non-zero".to_string());
        }
        if self.config.class_count == 0 {
            issues.push("class_count must be non-zero".to_string());
        }
        if manifest.splits.is_empty() {
            issues.push("manifest declares no splits".to_string());
        }
        if manifest.splits.iter().any(|s| s.sample_count == 0) {
            issues.push("manifest declares an empty split".to_string());
        }
        Ok(ValidationReport {
            passed: issues.is_empty(),
            issues,
        })
    }

    fn stream(
        &self,
        source: &DatasetSource,
        split: &SplitId,
    ) -> Result<Vec<IRSample>, TrainerError> {
        if split.0 != self.config.split_id.0 {
            return Err(TrainerError::Parse(format!(
                "unknown split '{}' (adapter configured for '{}')",
                split.0, self.config.split_id.0
            )));
        }
        let root = Self::dataset_root(source)?;
        let pairs = self.discover_pairs(root)?;
        let dataset_version_id = self.dataset_version_id(&Self::fingerprint_pairs(&pairs));
        let width = self.config.feed_width;
        let height = self.config.feed_height;

        let mut samples = Vec::with_capacity(pairs.len());
        for (index, pair) in pairs.iter().enumerate() {
            let image_bytes = Self::load_resized_png_bytes(&pair.image_path, width, height)?;
            let labels = Self::load_resized_label_mask(&pair.label_path, width, height)?;
            if labels.len() != (width as usize) * (height as usize) {
                return Err(TrainerError::Parse(format!(
                    "label mask size mismatch for '{}'",
                    pair.label_path.display()
                )));
            }
            samples.push(IRSample {
                schema_version: IR_SCHEMA_VERSION,
                sample_id: SampleId(format!("{}#{}", pair.image_path.to_string_lossy(), index)),
                dataset_version_id: dataset_version_id.clone(),
                split: self.config.split.clone(),
                modality: Modality::Image,
                payload: Payload::Bytes(image_bytes),
                target: Some(TypedTarget::SegmentationMask {
                    width,
                    height,
                    labels,
                    ignore_label: Some(self.config.ignore_label),
                }),
                output_type: OutputType::SegmentationMask,
                coordinate_frame: None,
                timestamp: None,
                metadata: BTreeMap::from([
                    (
                        "image_path".to_string(),
                        MetadataValue::Text(pair.image_path.to_string_lossy().into_owned()),
                    ),
                    (
                        "label_path".to_string(),
                        MetadataValue::Text(pair.label_path.to_string_lossy().into_owned()),
                    ),
                ]),
            });
        }
        Ok(samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::common::SplitId;
    use std::fs;
    use tempfile::tempdir;

    fn write_png(path: &Path, width: u32, height: u32, gray: u8) {
        let img = image::GrayImage::from_pixel(width, height, image::Luma([gray]));
        img.save(path).expect("write png");
    }

    fn write_rgb_png(path: &Path, width: u32, height: u32) {
        let img = image::RgbImage::from_pixel(width, height, image::Rgb([10, 20, 30]));
        img.save(path).expect("write png");
    }

    #[test]
    fn paired_folders_adapter_streams_resized_masks() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let images = root.join("images");
        let labels = root.join("labels");
        fs::create_dir_all(&images).expect("mkdir");
        fs::create_dir_all(&labels).expect("mkdir");
        write_rgb_png(&images.join("a.png"), 4, 2);
        write_png(&labels.join("a.png"), 4, 2, 3);

        let config = ImageFolderSegmentationConfig {
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
        };
        let adapter = ImageFolderSegmentationAdapter::new(config);
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let (manifest, samples) = {
            let manifest = adapter.discover(&source).expect("discover");
            let report = adapter.validate(&manifest).expect("validate");
            assert!(report.passed, "{:?}", report.issues);
            let samples = adapter
                .stream(&source, &SplitId("train".to_string()))
                .expect("stream");
            (manifest, samples)
        };
        assert_eq!(manifest.output_type, OutputType::SegmentationMask);
        assert_eq!(samples.len(), 1);
        match &samples[0].target {
            Some(TypedTarget::SegmentationMask {
                width,
                height,
                labels,
                ..
            }) => {
                assert_eq!(*width, 2);
                assert_eq!(*height, 2);
                assert_eq!(labels.len(), 4);
                assert!(labels.iter().all(|&v| v == 3));
            }
            other => panic!("expected segmentation target, got {other:?}"),
        }
    }
}
