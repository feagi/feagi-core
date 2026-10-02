//! Image classification adapter.
//!
//! One dataset root holds `train`, `val`, and `test`, or a packed format in the root:
//! MNIST IDX, CIFAR binary batches, SVHN MAT v5, or ILSVRC validation files.
//! Folder schemas are class subfolders or filenames ending in `_x_y_z`.
//! The operator class map is shared. A label that is not on that map is an error.
//! Class voxels are `(x, 0, 0)`.

mod cifar;
mod ilsvrc2012;
mod imagenet;
mod svhn_mat;

use std::collections::{BTreeMap, BTreeSet};
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

const SPLIT_DIR_NAMES: [&str; 3] = ["train", "val", "test"];
const IMAGE_EXTENSIONS: [&str; 3] = ["jpg", "jpeg", "png"];

/// How class identity is stored on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageClassificationSchema {
    /// `split/<label>/*.{jpg,jpeg,png}`.
    ClassFolders,
    /// `split/<label>_<x>_<y>_<z>.{jpg,jpeg,png}`.
    FilenameXyz,
    /// MNIST-style IDX image and label files in the dataset root.
    IdxImages,
    /// CIFAR-10 binary batches (`data_batch_*.bin`, `test_batch.bin`).
    Cifar10Binary,
    /// CIFAR-100 binary batches (`train.bin`, `test.bin`), fine labels.
    Cifar100Binary,
    /// ILSVRC train class folders plus a flat validation directory and ground truth.
    ImageNet,
    /// SVHN cropped digits (`train_32x32.mat`, `test_32x32.mat`).
    SvhnMat,
}

/// Where one sample's pixels live.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackedRecord {
    /// A standalone image file at [`ClassifiedImage::path`].
    File,
    /// One grayscale frame inside an IDX image file.
    Idx(IdxRecord),
    /// One CIFAR-10 record inside a binary batch.
    Cifar10 { record: u32 },
    /// One CIFAR-100 record inside a binary batch. The fine label is used.
    Cifar100 { record: u32 },
    /// One SVHN frame inside an uncompressed MAT v5 file.
    /// `data_offset` is the first byte of X's raw numeric payload.
    Svhn { record: u32, data_offset: u64 },
}

/// One class label and its teacher voxel. `x` and `y` are 0. Class is depth `z`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassVoxel {
    pub label: String,
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

/// Explicit configuration for one split of an image-classification root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageFolderClassificationConfig {
    pub dataset_name: String,
    pub image_schema: ImageClassificationSchema,
    pub split: Split,
    pub split_id: SplitId,
    pub feed_width: u32,
    pub feed_height: u32,
    pub class_map: Vec<ClassVoxel>,
    /// Keep the first `floor(count * percent / 100)` samples per class. Empty keeps all.
    #[serde(default)]
    pub class_keep_percents: BTreeMap<String, u32>,
    /// Cap after class keep. `None` uses every remaining sample in this split.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_samples: Option<u64>,
    /// With `max_samples`, keep a seeded random subset instead of the first samples.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_draw_seed: Option<u64>,
}

/// One indexed image. Pixels are loaded on visit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedImage {
    pub path: PathBuf,
    pub label: String,
    pub class_id: u32,
    /// Packed container record. [`PackedRecord::File`] is a standalone image file.
    pub packed: PackedRecord,
}

/// One grayscale frame inside an IDX image file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdxRecord {
    pub record: u32,
    pub rows: u32,
    pub cols: u32,
}

/// One class label and how many samples carry it inside one split.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageClassCount {
    pub label: String,
    pub count: u64,
}

/// Measured sample totals for one split discovered during scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitSampleStats {
    pub split: String,
    pub sample_count: u64,
    pub class_counts: Vec<ImageClassCount>,
}

/// Scan of a dataset root before the operator confirms the class map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageClassificationScan {
    pub schema: Option<ImageClassificationSchema>,
    pub classes: Vec<ClassVoxel>,
    /// Split folders that exist: `train`, `val`, and/or `test`.
    pub splits: Vec<String>,
    /// Per-split totals and class histograms when the scanner could measure them.
    #[serde(default)]
    pub split_stats: Vec<SplitSampleStats>,
    /// Pixel width shared by the images. Empty when the scan found no single size.
    #[serde(rename = "imageWidth")]
    pub image_width: Option<u32>,
    /// Pixel height shared by the images. Empty when the scan found no single size.
    #[serde(rename = "imageHeight")]
    pub image_height: Option<u32>,
    pub issues: Vec<String>,
}

/// Adapter that maps an image-classification root into class-labeled `IRSample`s.
#[derive(Debug, Clone)]
pub struct ImageFolderClassificationAdapter {
    config: ImageFolderClassificationConfig,
}

impl ImageFolderClassificationAdapter {
    pub const PLUGIN_ID: &'static str = "image_folder_classification";

    pub fn new(mut config: ImageFolderClassificationConfig) -> Self {
        for row in &mut config.class_map {
            if row.y == 0 && row.z == 0 && row.x > 0 {
                row.z = row.x;
                row.x = 0;
            }
        }
        Self { config }
    }

    pub fn config(&self) -> &ImageFolderClassificationConfig {
        &self.config
    }

    /// Teacher depth. Every class `z` is in `0..class_count`.
    pub fn class_count(class_map: &[ClassVoxel]) -> Result<u32, TrainerError> {
        validate_class_map(class_map)?;
        let max_z = class_map.iter().map(|row| row.z).max().ok_or_else(|| {
            TrainerError::Config("image classification class_map is empty".to_string())
        })?;
        max_z.checked_add(1).ok_or_else(|| {
            TrainerError::Config("image classification class index overflows u32".to_string())
        })
    }

    pub fn index(
        &self,
        source: &DatasetSource,
    ) -> Result<(DatasetManifest, Vec<ClassifiedImage>), TrainerError> {
        let root = dataset_root(source)?;
        let images = self.discover_images(root)?;
        // Dataset identity is the split on disk; class keep and the cap are run settings.
        let fingerprint = fingerprint_images(&images);
        let labels: Vec<String> = self
            .config
            .class_map
            .iter()
            .map(|row| row.label.clone())
            .collect();
        let images = apply_image_class_keep(images, &self.config.class_keep_percents, &labels)?;
        let images = crate::adapters::class_keep::apply_max_samples(
            images,
            self.config.max_samples,
            self.config.sample_draw_seed,
        )?;
        Ok((self.manifest(source, &images, fingerprint), images))
    }

    pub fn load_indexed_sample(
        &self,
        image: &ClassifiedImage,
        index: usize,
        dataset_version_id: &DatasetVersionId,
    ) -> Result<IRSample, TrainerError> {
        let png = match &image.packed {
            PackedRecord::File => {
                load_resized_png(&image.path, self.config.feed_width, self.config.feed_height)?
            }
            PackedRecord::Idx(record) => load_idx_png(
                &image.path,
                record,
                self.config.feed_width,
                self.config.feed_height,
            )?,
            PackedRecord::Cifar10 { record } => cifar::load_cifar10_png(
                &image.path,
                *record,
                self.config.feed_width,
                self.config.feed_height,
            )?,
            PackedRecord::Cifar100 { record } => cifar::load_cifar100_png(
                &image.path,
                *record,
                self.config.feed_width,
                self.config.feed_height,
            )?,
            PackedRecord::Svhn {
                record,
                data_offset,
            } => svhn_mat::load_svhn_png(
                &image.path,
                *record,
                *data_offset,
                self.config.feed_width,
                self.config.feed_height,
            )?,
        };
        Ok(IRSample {
            schema_version: IR_SCHEMA_VERSION,
            sample_id: SampleId(format!("{}#{index}", image.path.to_string_lossy())),
            dataset_version_id: dataset_version_id.clone(),
            split: self.config.split.clone(),
            modality: Modality::Image,
            payload: Payload::Bytes(png),
            target: Some(TypedTarget::Class {
                class_id: image.class_id,
                label: Some(image.label.clone()),
            }),
            output_type: OutputType::Class,
            coordinate_frame: None,
            timestamp: None,
            metadata: BTreeMap::from([(
                "image_path".to_string(),
                MetadataValue::Text(image.path.to_string_lossy().into_owned()),
            )]),
        })
    }

    fn discover_images(&self, root: &Path) -> Result<Vec<ClassifiedImage>, TrainerError> {
        let class_count = Self::class_count(&self.config.class_map)?;
        match self.config.image_schema {
            ImageClassificationSchema::IdxImages => {
                let idx_dir = resolve_idx_directory(root)?.ok_or_else(|| {
                    TrainerError::Parse(format!(
                        "MNIST IDX files were not found under '{}'",
                        root.display()
                    ))
                })?;
                return discover_idx_split(
                    &idx_dir,
                    &self.config.split,
                    &self.config.class_map,
                    class_count,
                );
            }
            ImageClassificationSchema::Cifar10Binary
            | ImageClassificationSchema::Cifar100Binary => {
                return cifar::discover_split(
                    root,
                    &self.config.image_schema,
                    &self.config.split,
                    &self.config.class_map,
                    class_count,
                );
            }
            ImageClassificationSchema::SvhnMat => {
                return svhn_mat::discover_split(
                    root,
                    &self.config.split,
                    &self.config.class_map,
                    class_count,
                );
            }
            ImageClassificationSchema::ImageNet => {
                return imagenet::discover_split(
                    root,
                    &self.config.split,
                    &self.config.class_map,
                    class_count,
                );
            }
            ImageClassificationSchema::ClassFolders | ImageClassificationSchema::FilenameXyz => {}
        }
        let split_name = split_dir_name(&self.config.split)?;
        let split_dir = root.join(split_name);
        if !split_dir.is_dir() {
            return Err(TrainerError::Parse(format!(
                "image classification split folder '{split_name}' is missing under '{}'",
                root.display()
            )));
        }
        let entries = list_dir(&split_dir)?;
        let images = match self.config.image_schema {
            ImageClassificationSchema::ClassFolders => {
                class_folder_images(&split_dir, &entries, &self.config.class_map)?
            }
            ImageClassificationSchema::FilenameXyz => {
                filename_images(&split_dir, &entries, &self.config.class_map)?
            }
            ImageClassificationSchema::IdxImages
            | ImageClassificationSchema::Cifar10Binary
            | ImageClassificationSchema::Cifar100Binary
            | ImageClassificationSchema::ImageNet
            | ImageClassificationSchema::SvhnMat => {
                return Err(TrainerError::Config(
                    "packed image formats are resolved before split folders".to_string(),
                ));
            }
        };
        for image in &images {
            if image.class_id >= class_count {
                return Err(TrainerError::Parse(format!(
                    "class id {} for '{}' is outside class_count {class_count}",
                    image.class_id, image.label
                )));
            }
        }
        if images.is_empty() {
            return Err(TrainerError::Parse(format!(
                "image classification split '{split_name}' has no images"
            )));
        }
        Ok(images)
    }

    fn manifest(
        &self,
        source: &DatasetSource,
        images: &[ClassifiedImage],
        fingerprint: String,
    ) -> DatasetManifest {
        let class_count = Self::class_count(&self.config.class_map).unwrap_or(0);
        DatasetManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            dataset_version_id: DatasetVersionId(format!(
                "{}@{fingerprint}",
                self.config.dataset_name
            )),
            dataset_asset_id: DatasetAssetId(format!(
                "image-classification:{}",
                self.config.dataset_name
            )),
            dataset_version: "1.0.0".to_string(),
            source_uri: source.uri.clone(),
            content_hash: ContentHash(fingerprint.clone()),
            schema_fingerprint: ContentHash(format!(
                "image-classification:{:?}:{}x{}:{class_count}",
                self.config.image_schema, self.config.feed_width, self.config.feed_height
            )),
            modality: Modality::Image,
            output_type: OutputType::Class,
            splits: vec![SplitDescriptor {
                id: self.config.split_id.clone(),
                split: self.config.split.clone(),
                sample_count: images.len() as u64,
            }],
            metadata: BTreeMap::new(),
        }
    }
}

impl AdapterPlugin for ImageFolderClassificationAdapter {
    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId(Self::PLUGIN_ID.to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn discover(&self, source: &DatasetSource) -> Result<DatasetManifest, TrainerError> {
        Ok(self.index(source)?.0)
    }

    fn validate(&self, manifest: &DatasetManifest) -> Result<ValidationReport, TrainerError> {
        let mut issues = Vec::new();
        if manifest.modality != Modality::Image {
            issues.push(format!("modality {:?} is not Image", manifest.modality));
        }
        if manifest.output_type != OutputType::Class {
            issues.push(format!(
                "output_type {:?} is not Class",
                manifest.output_type
            ));
        }
        if self.config.feed_width == 0 || self.config.feed_height == 0 {
            issues.push("feed_width and feed_height must be non-zero".to_string());
        }
        if let Err(error) = validate_class_map(&self.config.class_map) {
            issues.push(error.to_string());
        }
        if manifest.splits.iter().any(|split| split.sample_count == 0) {
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
        let (manifest, images) = self.index(source)?;
        let mut samples = Vec::with_capacity(images.len());
        for (index, image) in images.iter().enumerate() {
            samples.push(self.load_indexed_sample(image, index, &manifest.dataset_version_id)?);
        }
        Ok(samples)
    }
}

/// Reads `train` / `val` / `test` and suggests one schema plus a class row per label.
pub fn scan_image_classification_root(
    root: &Path,
) -> Result<ImageClassificationScan, TrainerError> {
    if !root.is_dir() {
        return Err(TrainerError::Parse(format!(
            "image classification root '{}' is not a directory",
            root.display()
        )));
    }
    let mut issues = Vec::new();
    if let Some(idx_dir) = resolve_idx_directory(root)? {
        let Some(idx) = scan_idx_root(&idx_dir)? else {
            return Err(TrainerError::Parse(format!(
                "IDX names were found in '{}' but no image/label pair could be read",
                idx_dir.display()
            )));
        };
        return Ok(idx);
    }
    if let Some(cifar) = cifar::scan_if_present(root)? {
        return Ok(cifar);
    }
    if let Some(svhn) = svhn_mat::scan_if_present(root)? {
        return Ok(svhn);
    }
    if let Some(imagenet_scan) = imagenet::scan_if_flat_val(root)? {
        return Ok(imagenet_scan);
    }
    if root.join("validation").is_dir() {
        issues.push("folder 'validation' is not a split. Use 'val'.".to_string());
    }
    let mut present: Vec<&str> = Vec::new();
    for name in SPLIT_DIR_NAMES {
        if root.join(name).is_dir() {
            present.push(name);
        }
    }
    if present.is_empty() {
        issues.push(format!(
            "no train, val, or test folder, and no packed dataset files. Expected MNIST IDX, CIFAR batches, SVHN MAT files, or class folders. Found: {}",
            directory_file_names(root)?
        ));
        return Ok(ImageClassificationScan {
            schema: None,
            classes: Vec::new(),
            splits: Vec::new(),
            split_stats: Vec::new(),
            image_width: None,
            image_height: None,
            issues,
        });
    }

    let mut detected: Option<ImageClassificationSchema> = None;
    let mut per_split: BTreeMap<&str, Vec<ClassVoxel>> = BTreeMap::new();
    for name in &present {
        let split_dir = root.join(name);
        match detect_split(&split_dir) {
            Ok((schema, classes)) => {
                if let Some(prior) = detected {
                    if prior != schema {
                        issues.push(format!(
                            "split '{name}' schema does not match the other splits"
                        ));
                    }
                } else {
                    detected = Some(schema);
                }
                per_split.insert(name, classes);
            }
            Err(message) => issues.push(format!("split '{name}': {message}")),
        }
    }

    let classes = suggest_classes(&per_split, &mut issues);
    if classes.is_empty() && issues.is_empty() {
        issues.push("image classification scan found no classes".to_string());
    }
    let mut image_size = ImageSizeSet::default();
    for name in per_split.keys() {
        if let Err(message) =
            observe_split_image_sizes(&root.join(name), &mut image_size, &mut issues)
        {
            issues.push(format!("split '{name}': {message}"));
        }
    }
    let (image_width, image_height) = image_size.pair();
    let mut split_stats = Vec::new();
    for name in &present {
        match class_folder_split_stats(&root.join(name)) {
            Ok(stats) => split_stats.push(stats),
            Err(message) => issues.push(format!("split '{name}': {message}")),
        }
    }
    Ok(ImageClassificationScan {
        schema: detected,
        classes,
        splits: present.iter().map(|name| (*name).to_string()).collect(),
        split_stats,
        image_width,
        image_height,
        issues,
    })
}

/// Counts class-folder images under one split directory.
fn class_folder_split_stats(split_dir: &Path) -> Result<SplitSampleStats, String> {
    let split = split_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("split")
        .to_string();
    let entries = list_dir(split_dir).map_err(|error| error.to_string())?;
    let mut class_counts = Vec::new();
    let mut sample_count = 0_u64;
    for class_dir in entries {
        if !class_dir.is_dir() {
            continue;
        }
        let label = file_name_utf8(&class_dir)?;
        let files = list_dir(&class_dir).map_err(|error| error.to_string())?;
        let count = files
            .iter()
            .filter(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| {
                        let lower = ext.to_ascii_lowercase();
                        IMAGE_EXTENSIONS.iter().any(|allowed| *allowed == lower)
                    })
                    .unwrap_or(false)
            })
            .count() as u64;
        if count == 0 {
            continue;
        }
        sample_count += count;
        class_counts.push(ImageClassCount { label, count });
    }
    class_counts.sort_by(|left, right| left.label.cmp(&right.label));
    Ok(SplitSampleStats {
        split,
        sample_count,
        class_counts,
    })
}

fn suggest_classes(
    per_split: &BTreeMap<&str, Vec<ClassVoxel>>,
    issues: &mut Vec<String>,
) -> Vec<ClassVoxel> {
    let source = if per_split.contains_key("train") {
        per_split.get("train").cloned().unwrap_or_default()
    } else {
        let mut union = Vec::new();
        for classes in per_split.values() {
            union.extend(classes.clone());
        }
        union
    };
    if per_split.contains_key("train") {
        let train_labels: BTreeSet<String> = source.iter().map(|row| row.label.clone()).collect();
        for (name, classes) in per_split {
            if *name == "train" {
                continue;
            }
            for row in classes {
                if !train_labels.contains(&row.label) {
                    issues.push(format!(
                        "label '{}' appears in '{name}' and not in train",
                        row.label
                    ));
                }
            }
        }
    }
    let mut by_label: BTreeMap<String, ClassVoxel> = BTreeMap::new();
    for row in source {
        if row.x != 0 || row.y != 0 {
            issues.push(format!(
                "label '{}' voxel ({}, {}, {}) leaves the 1x1xC class standard",
                row.label, row.x, row.y, row.z
            ));
        }
        if let Some(prior) = by_label.get(&row.label) {
            if prior.x != row.x || prior.y != row.y || prior.z != row.z {
                issues.push(format!("label '{}' maps to more than one voxel", row.label));
            }
        } else {
            by_label.insert(row.label.clone(), row);
        }
    }
    let mut classes: Vec<ClassVoxel> = by_label.into_values().collect();
    classes.sort_by(|left, right| left.z.cmp(&right.z).then(left.label.cmp(&right.label)));
    let mut seen_z = BTreeSet::new();
    for row in &classes {
        if !seen_z.insert(row.z) {
            issues.push(format!(
                "class voxel z {} is used by more than one label",
                row.z
            ));
        }
    }
    classes
}

#[derive(Default)]
struct ImageSizeSet {
    size: Option<(u32, u32)>,
    conflict: bool,
}

impl ImageSizeSet {
    fn observe(&mut self, width: u32, height: u32) {
        if self.conflict {
            return;
        }
        match self.size {
            None => self.size = Some((width, height)),
            Some((prior_width, prior_height)) if prior_width != width || prior_height != height => {
                self.conflict = true;
                self.size = None;
            }
            Some(_) => {}
        }
    }

    fn pair(self) -> (Option<u32>, Option<u32>) {
        match self.size {
            Some((width, height)) => (Some(width), Some(height)),
            None => (None, None),
        }
    }
}

fn observe_split_image_sizes(
    split_dir: &Path,
    sizes: &mut ImageSizeSet,
    _issues: &mut Vec<String>,
) -> Result<(), String> {
    let Some(path) = first_image(split_dir)? else {
        return Ok(());
    };
    observe_file_size(&path, sizes)
}

/// One image is enough to suggest a feed size. Variable-size sets such as ImageNet
/// leave the suggestion empty when two splits disagree, without reading every file.
fn first_image(split_dir: &Path) -> Result<Option<PathBuf>, String> {
    let entries = list_dir(split_dir).map_err(|error| error.to_string())?;
    for path in entries {
        if path.is_dir() {
            let children = list_dir(&path).map_err(|error| error.to_string())?;
            for child in children {
                if is_image_file(&child) {
                    return Ok(Some(child));
                }
            }
        } else if is_image_file(&path) {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn observe_file_size(path: &Path, sizes: &mut ImageSizeSet) -> Result<(), String> {
    let (width, height) = image::image_dimensions(path)
        .map_err(|error| format!("could not read the size of '{}': {error}", path.display()))?;
    sizes.observe(width, height);
    Ok(())
}

fn detect_split(split_dir: &Path) -> Result<(ImageClassificationSchema, Vec<ClassVoxel>), String> {
    let entries = list_dir(split_dir).map_err(|error| error.to_string())?;
    if entries.is_empty() {
        return Err("split folder is empty".to_string());
    }
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for path in entries {
        if path.is_dir() {
            dirs.push(path);
        } else if path.is_file() {
            files.push(path);
        } else {
            return Err(format!("'{}' is not a file or directory", path.display()));
        }
    }
    if !dirs.is_empty() && !files.is_empty() {
        return Err("class folders and loose files are both present".to_string());
    }
    if !dirs.is_empty() {
        return class_folder_suggestion(&dirs);
    }
    filename_suggestion(&files)
}

fn class_folder_suggestion(
    dirs: &[PathBuf],
) -> Result<(ImageClassificationSchema, Vec<ClassVoxel>), String> {
    let mut labels = Vec::new();
    for dir in dirs {
        let label = file_name_utf8(dir)?;
        let children = list_dir(dir).map_err(|error| error.to_string())?;
        if children.is_empty() {
            return Err(format!("class folder '{label}' has no images"));
        }
        for child in children {
            if child.is_dir() {
                return Err(format!("class folder '{label}' contains a subdirectory"));
            }
            if !is_image_file(&child) {
                return Err(format!(
                    "class folder '{label}' contains non-image '{}'",
                    child.display()
                ));
            }
        }
        labels.push(label);
    }
    labels.sort();
    let classes = labels
        .into_iter()
        .enumerate()
        .map(|(index, label)| ClassVoxel {
            label,
            x: 0,
            y: 0,
            z: index as u32,
        })
        .collect();
    Ok((ImageClassificationSchema::ClassFolders, classes))
}

fn filename_suggestion(
    files: &[PathBuf],
) -> Result<(ImageClassificationSchema, Vec<ClassVoxel>), String> {
    let mut classes = Vec::new();
    for file in files {
        if !is_image_file(file) {
            return Err(format!("'{}' is not a jpg or png image", file.display()));
        }
        let (label, x, y, z) = parse_filename_voxel(file)?;
        classes.push(ClassVoxel { label, x, y, z });
    }
    Ok((ImageClassificationSchema::FilenameXyz, classes))
}

fn class_folder_images(
    split_dir: &Path,
    entries: &[PathBuf],
    class_map: &[ClassVoxel],
) -> Result<Vec<ClassifiedImage>, TrainerError> {
    let mut images = Vec::new();
    for path in entries {
        if path.is_file() {
            return Err(TrainerError::Parse(format!(
                "class-folder split '{}' contains loose file '{}'",
                split_dir.display(),
                path.display()
            )));
        }
        if !path.is_dir() {
            return Err(TrainerError::Parse(format!(
                "'{}' is not a class folder",
                path.display()
            )));
        }
        let label = file_name_utf8(path).map_err(TrainerError::Parse)?;
        let class_id = class_id_for_label(&label, class_map)?;
        let children = list_dir(path)?;
        if children.is_empty() {
            return Err(TrainerError::Parse(format!(
                "class folder '{label}' has no images"
            )));
        }
        for child in children {
            if !child.is_file() || !is_image_file(&child) {
                return Err(TrainerError::Parse(format!(
                    "class folder '{label}' contains '{}', which is not a jpg or png image",
                    child.display()
                )));
            }
            images.push(ClassifiedImage {
                path: child,
                label: label.clone(),
                class_id,
                packed: PackedRecord::File,
            });
        }
    }
    images.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(images)
}

fn filename_images(
    split_dir: &Path,
    entries: &[PathBuf],
    class_map: &[ClassVoxel],
) -> Result<Vec<ClassifiedImage>, TrainerError> {
    let mut images = Vec::new();
    for path in entries {
        if path.is_dir() {
            return Err(TrainerError::Parse(format!(
                "filename split '{}' contains directory '{}'",
                split_dir.display(),
                path.display()
            )));
        }
        if !is_image_file(path) {
            return Err(TrainerError::Parse(format!(
                "'{}' is not a jpg or png image",
                path.display()
            )));
        }
        let (label, x, y, z) = parse_filename_voxel(path).map_err(TrainerError::Parse)?;
        let mapped = class_map
            .iter()
            .find(|row| row.label == label)
            .ok_or_else(|| {
                TrainerError::Parse(format!("label '{label}' is not in the class map"))
            })?;
        if mapped.x != x || mapped.y != y || mapped.z != z {
            return Err(TrainerError::Parse(format!(
                "'{}' voxel ({x}, {y}, {z}) does not match class map ({}, {}, {})",
                path.display(),
                mapped.x,
                mapped.y,
                mapped.z
            )));
        }
        images.push(ClassifiedImage {
            path: path.clone(),
            label,
            class_id: z,
            packed: PackedRecord::File,
        });
    }
    images.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(images)
}

fn class_id_for_label(label: &str, class_map: &[ClassVoxel]) -> Result<u32, TrainerError> {
    class_map
        .iter()
        .find(|row| row.label == label)
        .map(|row| row.z)
        .ok_or_else(|| TrainerError::Parse(format!("label '{label}' is not in the class map")))
}

fn validate_class_map(class_map: &[ClassVoxel]) -> Result<(), TrainerError> {
    if class_map.is_empty() {
        return Err(TrainerError::Config(
            "image classification class_map is empty".to_string(),
        ));
    }
    let mut labels = BTreeSet::new();
    let mut zs = BTreeSet::new();
    for row in class_map {
        if row.label.trim().is_empty() {
            return Err(TrainerError::Config(
                "image classification class label is empty".to_string(),
            ));
        }
        if row.x != 0 || row.y != 0 {
            return Err(TrainerError::Config(format!(
                "label '{}' must use voxel (0, 0, {})",
                row.label, row.z
            )));
        }
        if !labels.insert(row.label.clone()) {
            return Err(TrainerError::Config(format!(
                "duplicate class label '{}'",
                row.label
            )));
        }
        if !zs.insert(row.z) {
            return Err(TrainerError::Config(format!(
                "duplicate class voxel z {}",
                row.z
            )));
        }
    }
    Ok(())
}

fn split_dir_name(split: &Split) -> Result<&'static str, TrainerError> {
    match split {
        Split::Train => Ok("train"),
        Split::Val => Ok("val"),
        Split::Test => Ok("test"),
        Split::Custom(name) => Err(TrainerError::Config(format!(
            "image classification does not use custom split '{name}'"
        ))),
    }
}

fn dataset_root(source: &DatasetSource) -> Result<&Path, TrainerError> {
    let path = Path::new(&source.uri);
    if !path.is_dir() {
        return Err(TrainerError::Parse(format!(
            "image classification root '{}' is not a directory",
            path.display()
        )));
    }
    Ok(path)
}

fn list_dir(dir: &Path) -> Result<Vec<PathBuf>, TrainerError> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(dir)
        .map_err(|error| TrainerError::Parse(format!("cannot read '{}': {error}", dir.display())))?
    {
        let entry = entry.map_err(|error| TrainerError::Parse(error.to_string()))?;
        paths.push(entry.path());
    }
    paths.sort();
    Ok(paths)
}

fn file_name_utf8(path: &Path) -> Result<String, String> {
    let name = path
        .file_name()
        .ok_or_else(|| format!("'{}' has no file name", path.display()))?;
    name.to_str()
        .map(|value| value.to_string())
        .ok_or_else(|| format!("'{}' is not UTF-8", path.display()))
}

fn is_image_file(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };
    IMAGE_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
}

fn parse_filename_voxel(path: &Path) -> Result<(String, u32, u32, u32), String> {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("'{}' has no UTF-8 file stem", path.display()))?;
    let mut parts = stem.rsplitn(4, '_');
    let z_text = parts
        .next()
        .ok_or_else(|| format!("'{}' does not end in _x_y_z", path.display()))?;
    let y_text = parts
        .next()
        .ok_or_else(|| format!("'{}' does not end in _x_y_z", path.display()))?;
    let x_text = parts
        .next()
        .ok_or_else(|| format!("'{}' does not end in _x_y_z", path.display()))?;
    let label = parts
        .next()
        .ok_or_else(|| format!("'{}' does not end in _x_y_z", path.display()))?;
    if label.is_empty() {
        return Err(format!(
            "'{}' has an empty label before _x_y_z",
            path.display()
        ));
    }
    let x = x_text
        .parse::<u32>()
        .map_err(|_| format!("'{}' class x is not an integer", path.display()))?;
    let y = y_text
        .parse::<u32>()
        .map_err(|_| format!("'{}' class y is not an integer", path.display()))?;
    let z = z_text
        .parse::<u32>()
        .map_err(|_| format!("'{}' class z is not an integer", path.display()))?;
    Ok((label.to_string(), x, y, z))
}

const IDX_IMAGE_MAGIC: u32 = 2051;
const IDX_LABEL_MAGIC: u32 = 2049;

struct IdxRole {
    split: &'static str,
    image_names: &'static [&'static str],
    label_names: &'static [&'static str],
}

const IDX_ROLES: &[IdxRole] = &[
    IdxRole {
        split: "train",
        image_names: &["train-images-idx3-ubyte", "train-images-idx3-ubyte.gz"],
        label_names: &["train-labels-idx1-ubyte", "train-labels-idx1-ubyte.gz"],
    },
    IdxRole {
        split: "test",
        image_names: &["t10k-images-idx3-ubyte", "t10k-images-idx3-ubyte.gz"],
        label_names: &["t10k-labels-idx1-ubyte", "t10k-labels-idx1-ubyte.gz"],
    },
];

fn scan_idx_root(root: &Path) -> Result<Option<ImageClassificationScan>, TrainerError> {
    let mut any = false;
    let mut issues = Vec::new();
    let mut splits = Vec::new();
    let mut split_stats = Vec::new();
    let mut image_size = ImageSizeSet::default();
    let mut per_split: BTreeMap<&str, Vec<ClassVoxel>> = BTreeMap::new();
    for role in IDX_ROLES {
        let images = locate_idx_file(root, role.image_names)?;
        let labels = locate_idx_file(root, role.label_names)?;
        match (images, labels) {
            (None, None) => {}
            (Some(images_path), Some(labels_path)) => {
                any = true;
                splits.push(role.split.to_string());
                let label_bytes = read_idx_labels(&labels_path)?;
                let header = read_idx_image_header(&images_path)?;
                image_size.observe(header.cols, header.rows);
                if header.count != label_bytes.len() as u32 {
                    issues.push(format!(
                        "IDX split '{}' image count {} does not match label count {}",
                        role.split,
                        header.count,
                        label_bytes.len()
                    ));
                }
                per_split.insert(role.split, classes_from_idx_labels(&label_bytes));
                split_stats.push(split_stats_from_idx_labels(role.split, &label_bytes));
            }
            (Some(_), None) => {
                any = true;
                issues.push(format!(
                    "IDX split '{}' has images and no label file",
                    role.split
                ));
            }
            (None, Some(_)) => {
                any = true;
                issues.push(format!(
                    "IDX split '{}' has labels and no image file",
                    role.split
                ));
            }
        }
    }
    if !any {
        return Ok(None);
    }
    for name in SPLIT_DIR_NAMES {
        let dir = root.join(name);
        if dir.is_dir() && !list_dir(&dir)?.is_empty() {
            issues.push(format!("IDX root also contains split folder '{name}'"));
        }
    }
    let classes = suggest_classes(&per_split, &mut issues);
    let (image_width, image_height) = image_size.pair();
    Ok(Some(ImageClassificationScan {
        schema: Some(ImageClassificationSchema::IdxImages),
        classes,
        splits,
        split_stats,
        image_width,
        image_height,
        issues,
    }))
}

fn discover_idx_split(
    root: &Path,
    split: &Split,
    class_map: &[ClassVoxel],
    class_count: u32,
) -> Result<Vec<ClassifiedImage>, TrainerError> {
    let split_name = split_dir_name(split)?;
    let Some(role) = IDX_ROLES.iter().find(|role| role.split == split_name) else {
        return Err(TrainerError::Parse(format!(
            "IDX classification has no {split_name} split. MNIST provides train and t10k (test)."
        )));
    };
    let images_path = locate_idx_file(root, role.image_names)?.ok_or_else(|| {
        TrainerError::Parse(format!(
            "IDX {} image file is missing under '{}'",
            role.split,
            root.display()
        ))
    })?;
    let labels_path = locate_idx_file(root, role.label_names)?.ok_or_else(|| {
        TrainerError::Parse(format!(
            "IDX {} label file is missing under '{}'",
            role.split,
            root.display()
        ))
    })?;
    let label_bytes = read_idx_labels(&labels_path)?;
    let header = read_idx_image_header(&images_path)?;
    if header.count != label_bytes.len() as u32 {
        return Err(TrainerError::Parse(format!(
            "IDX {} image count {} does not match label count {}",
            role.split,
            header.count,
            label_bytes.len()
        )));
    }
    let mut images = Vec::with_capacity(label_bytes.len());
    for (record, byte) in label_bytes.iter().copied().enumerate() {
        let label = byte.to_string();
        let class_id = class_id_for_label(&label, class_map)?;
        if class_id != u32::from(byte) {
            return Err(TrainerError::Parse(format!(
                "IDX label {byte} is mapped to x {class_id}. The class index is the label byte."
            )));
        }
        if class_id >= class_count {
            return Err(TrainerError::Parse(format!(
                "class id {class_id} for '{label}' is outside class_count {class_count}"
            )));
        }
        images.push(ClassifiedImage {
            path: images_path.clone(),
            label,
            class_id,
            packed: PackedRecord::Idx(IdxRecord {
                record: record as u32,
                rows: header.rows,
                cols: header.cols,
            }),
        });
    }
    if images.is_empty() {
        return Err(TrainerError::Parse(format!(
            "IDX split '{}' has no images",
            role.split
        )));
    }
    Ok(images)
}

fn classes_from_idx_labels(labels: &[u8]) -> Vec<ClassVoxel> {
    class_counts_from_idx_labels(labels)
        .into_iter()
        .map(|entry| {
            let digit: u32 = entry.label.parse().unwrap_or(0);
            ClassVoxel {
                label: entry.label,
                x: 0,
                y: 0,
                z: digit,
            }
        })
        .collect()
}

fn class_counts_from_idx_labels(labels: &[u8]) -> Vec<ImageClassCount> {
    let mut totals: BTreeMap<u8, u64> = BTreeMap::new();
    for byte in labels {
        *totals.entry(*byte).or_insert(0) += 1;
    }
    totals
        .into_iter()
        .map(|(byte, count)| ImageClassCount {
            label: byte.to_string(),
            count,
        })
        .collect()
}

fn split_stats_from_idx_labels(split: &str, labels: &[u8]) -> SplitSampleStats {
    let class_counts = class_counts_from_idx_labels(labels);
    let sample_count = class_counts.iter().map(|entry| entry.count).sum();
    SplitSampleStats {
        split: split.to_string(),
        sample_count,
        class_counts,
    }
}

/// Keeps the first `floor(count * percent / 100)` images per class label.
fn apply_image_class_keep(
    images: Vec<ClassifiedImage>,
    percents: &BTreeMap<String, u32>,
    class_labels: &[String],
) -> Result<Vec<ClassifiedImage>, TrainerError> {
    if percents.is_empty() {
        return Ok(images);
    }
    crate::adapters::class_keep::validate_class_keep_percents(percents, class_labels)
        .map_err(TrainerError::Config)?;
    let mut totals: BTreeMap<String, u64> = BTreeMap::new();
    for image in &images {
        *totals.entry(image.label.clone()).or_insert(0) += 1;
    }
    let mut remaining: BTreeMap<String, u64> = BTreeMap::new();
    for label in class_labels {
        let total = totals.get(label).copied().unwrap_or(0);
        let percent = percents.get(label).copied().unwrap_or(100);
        remaining.insert(
            label.clone(),
            crate::adapters::class_keep::class_keep_count(total, percent)
                .map_err(TrainerError::Config)?,
        );
    }
    let mut kept = Vec::new();
    for image in images {
        let left = remaining.get_mut(&image.label).ok_or_else(|| {
            TrainerError::Config(format!(
                "class keep saw unlabeled class '{}' not in class_map",
                image.label
            ))
        })?;
        if *left == 0 {
            continue;
        }
        *left -= 1;
        kept.push(image);
    }
    if kept.is_empty() {
        return Err(TrainerError::Parse(
            "class keep percents removed every sample".to_string(),
        ));
    }
    Ok(kept)
}

fn canonical_idx_name(name: &str) -> String {
    name.to_ascii_lowercase().replace('.', "-")
}

fn directory_has_idx(dir: &Path) -> Result<bool, TrainerError> {
    for role in IDX_ROLES {
        if locate_idx_file(dir, role.image_names)?.is_some()
            || locate_idx_file(dir, role.label_names)?.is_some()
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// IDX files live in the selected folder, or in torchvision's `raw` child, or in one child folder.
pub(super) fn locate_single_child(
    root: &Path,
    present: impl Fn(&Path) -> Result<bool, TrainerError>,
    what: &str,
) -> Result<Option<PathBuf>, TrainerError> {
    if present(root)? {
        return Ok(Some(root.to_path_buf()));
    }
    let mut hits = Vec::new();
    for child in list_dir(root)? {
        if child.is_dir() && present(&child)? {
            hits.push(child);
        }
    }
    match hits.len() {
        0 => Ok(None),
        1 => Ok(hits.pop()),
        _ => Err(TrainerError::Parse(format!(
            "{what} files are in more than one folder under '{}'",
            root.display()
        ))),
    }
}

fn resolve_idx_directory(root: &Path) -> Result<Option<PathBuf>, TrainerError> {
    if directory_has_idx(root)? {
        return Ok(Some(root.to_path_buf()));
    }
    let mut hits = Vec::new();
    for child in list_dir(root)? {
        if child.is_dir() && directory_has_idx(&child)? {
            hits.push(child);
        }
    }
    match hits.len() {
        0 => Ok(None),
        1 => Ok(hits.pop()),
        _ => Err(TrainerError::Parse(format!(
            "MNIST IDX files are in more than one folder under '{}'",
            root.display()
        ))),
    }
}

fn directory_file_names(root: &Path) -> Result<String, TrainerError> {
    let mut names = Vec::new();
    for path in list_dir(root)? {
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        names.push(name.to_string());
        if names.len() == 8 {
            break;
        }
    }
    if names.is_empty() {
        Ok("(empty)".to_string())
    } else {
        Ok(names.join(", "))
    }
}

fn locate_idx_file(root: &Path, names: &[&str]) -> Result<Option<PathBuf>, TrainerError> {
    let wanted: BTreeSet<String> = names.iter().copied().map(canonical_idx_name).collect();
    let mut found = Vec::new();
    for path in list_dir(root)? {
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        if wanted.contains(&canonical_idx_name(name)) {
            found.push(path);
        }
    }
    match found.len() {
        0 => Ok(None),
        1 => Ok(found.pop()),
        _ => Err(TrainerError::Parse(format!(
            "both plain and gzip IDX files are present under '{}'",
            root.display()
        ))),
    }
}

struct IdxImageHeader {
    count: u32,
    rows: u32,
    cols: u32,
}

fn read_idx_labels(path: &Path) -> Result<Vec<u8>, TrainerError> {
    let bytes = read_idx_bytes(path)?;
    if bytes.len() < 8 {
        return Err(TrainerError::Parse(format!(
            "IDX label file '{}' is shorter than its header",
            path.display()
        )));
    }
    let magic = read_u32(&bytes, 0);
    if magic != IDX_LABEL_MAGIC {
        return Err(TrainerError::Parse(format!(
            "IDX label file '{}' has magic {magic}, expected {IDX_LABEL_MAGIC}",
            path.display()
        )));
    }
    let count = read_u32(&bytes, 4) as usize;
    let labels = bytes.get(8..8 + count).ok_or_else(|| {
        TrainerError::Parse(format!(
            "IDX label file '{}' declares {count} labels and is too short",
            path.display()
        ))
    })?;
    Ok(labels.to_vec())
}

fn read_idx_image_header(path: &Path) -> Result<IdxImageHeader, TrainerError> {
    let bytes = read_idx_bytes(path)?;
    if bytes.len() < 16 {
        return Err(TrainerError::Parse(format!(
            "IDX image file '{}' is shorter than its header",
            path.display()
        )));
    }
    let magic = read_u32(&bytes, 0);
    if magic != IDX_IMAGE_MAGIC {
        return Err(TrainerError::Parse(format!(
            "IDX image file '{}' has magic {magic}, expected {IDX_IMAGE_MAGIC}",
            path.display()
        )));
    }
    let count = read_u32(&bytes, 4);
    let rows = read_u32(&bytes, 8);
    let cols = read_u32(&bytes, 12);
    if rows == 0 || cols == 0 {
        return Err(TrainerError::Parse(format!(
            "IDX image file '{}' has empty dimensions {rows}x{cols}",
            path.display()
        )));
    }
    let needed = 16 + (count as usize) * (rows as usize) * (cols as usize);
    if bytes.len() < needed {
        return Err(TrainerError::Parse(format!(
            "IDX image file '{}' declares {count} frames of {rows}x{cols} and is too short",
            path.display()
        )));
    }
    Ok(IdxImageHeader { count, rows, cols })
}

fn load_idx_png(
    path: &Path,
    record: &IdxRecord,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, TrainerError> {
    let bytes = read_idx_bytes(path)?;
    let header = read_idx_image_header(path)?;
    if header.rows != record.rows || header.cols != record.cols {
        return Err(TrainerError::Parse(format!(
            "IDX image '{}' dimensions changed",
            path.display()
        )));
    }
    let pixels = (record.rows as usize)
        .checked_mul(record.cols as usize)
        .ok_or_else(|| TrainerError::Parse("IDX frame size overflows".to_string()))?;
    let start = 16 + (record.record as usize) * pixels;
    let frame = bytes.get(start..start + pixels).ok_or_else(|| {
        TrainerError::Parse(format!(
            "IDX record {} is outside '{}'",
            record.record,
            path.display()
        ))
    })?;
    let gray =
        image::GrayImage::from_raw(record.cols, record.rows, frame.to_vec()).ok_or_else(|| {
            TrainerError::Parse(format!(
                "cannot build a grayscale frame from '{}'",
                path.display()
            ))
        })?;
    let rgb = image::DynamicImage::ImageLuma8(gray).to_rgb8();
    let resized = image::imageops::resize(&rgb, width, height, FilterType::Nearest);
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(resized)
        .write_to(&mut std::io::Cursor::new(&mut out), ImageFormat::Png)
        .map_err(|error| TrainerError::Parse(format!("cannot encode resized png: {error}")))?;
    Ok(out)
}

fn read_idx_bytes(path: &Path) -> Result<Vec<u8>, TrainerError> {
    let raw = std::fs::read(path).map_err(|error| {
        TrainerError::Parse(format!("cannot read '{}': {error}", path.display()))
    })?;
    if path.extension().and_then(|ext| ext.to_str()) == Some("gz") {
        let mut decoder = flate2::read::GzDecoder::new(raw.as_slice());
        let mut inflated = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut inflated).map_err(|error| {
            TrainerError::Parse(format!("cannot gunzip '{}': {error}", path.display()))
        })?;
        Ok(inflated)
    } else {
        Ok(raw)
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

pub(super) fn resized_rgb_png(
    rgb: image::RgbImage,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, TrainerError> {
    let resized = image::imageops::resize(&rgb, width, height, FilterType::Nearest);
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(resized)
        .write_to(&mut std::io::Cursor::new(&mut out), ImageFormat::Png)
        .map_err(|error| TrainerError::Parse(format!("cannot encode resized png: {error}")))?;
    Ok(out)
}

fn load_resized_png(path: &Path, width: u32, height: u32) -> Result<Vec<u8>, TrainerError> {
    let bytes = std::fs::read(path).map_err(|error| {
        TrainerError::Parse(format!("cannot read '{}': {error}", path.display()))
    })?;
    let image = image::load_from_memory(&bytes).map_err(|error| {
        TrainerError::Parse(format!("cannot decode '{}': {error}", path.display()))
    })?;
    let resized = image.resize_exact(width, height, FilterType::Nearest);
    let mut out = Vec::new();
    resized
        .write_to(&mut std::io::Cursor::new(&mut out), ImageFormat::Png)
        .map_err(|error| TrainerError::Parse(format!("cannot encode resized png: {error}")))?;
    Ok(out)
}

fn fingerprint_images(images: &[ClassifiedImage]) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for image in images {
        image.path.to_string_lossy().hash(&mut hasher);
        image.label.hash(&mut hasher);
        image.class_id.hash(&mut hasher);
        image.packed.hash(&mut hasher);
    }
    format!("siphash64:{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_jpeg(path: &Path) {
        let image = image::RgbImage::from_pixel(4, 4, image::Rgb([10, 20, 30]));
        image.save(path).expect("jpeg");
    }

    #[test]
    fn filename_schema_rejects_a_label_missing_from_train() {
        let root = tempfile::tempdir().expect("temp");
        fs::create_dir(root.path().join("train")).expect("train");
        fs::create_dir(root.path().join("val")).expect("val");
        write_jpeg(&root.path().join("train").join("red_1_0_0.jpg"));
        write_jpeg(&root.path().join("val").join("blue_3_0_0.jpg"));
        let scan = scan_image_classification_root(root.path()).expect("scan");
        assert_eq!(scan.schema, Some(ImageClassificationSchema::FilenameXyz));
        assert_eq!(scan.splits, vec!["train".to_string(), "val".to_string()]);
        assert!(scan
            .issues
            .iter()
            .any(|issue| issue.contains("blue") && issue.contains("not in train")));
    }

    #[test]
    fn class_index_stored_on_x_is_placed_on_z() {
        let adapter = ImageFolderClassificationAdapter::new(ImageFolderClassificationConfig {
            dataset_name: "digits".to_string(),
            image_schema: ImageClassificationSchema::IdxImages,
            split: Split::Train,
            split_id: SplitId("train".to_string()),
            feed_width: 28,
            feed_height: 28,
            class_map: vec![
                ClassVoxel {
                    label: "0".to_string(),
                    x: 0,
                    y: 0,
                    z: 0,
                },
                ClassVoxel {
                    label: "1".to_string(),
                    x: 1,
                    y: 0,
                    z: 0,
                },
            ],
            class_keep_percents: std::collections::BTreeMap::new(),
            max_samples: None,
            sample_draw_seed: None,
        });
        let map = &adapter.config().class_map;
        assert_eq!(map[1].x, 0);
        assert_eq!(map[1].z, 1);
        assert_eq!(
            ImageFolderClassificationAdapter::class_count(map).expect("count"),
            2
        );
    }

    #[test]
    fn class_folders_stream_one_class_per_image() {
        let root = tempfile::tempdir().expect("temp");
        let red = root.path().join("train").join("red");
        fs::create_dir_all(&red).expect("red");
        write_jpeg(&red.join("a.jpg"));
        let class_map = vec![ClassVoxel {
            label: "red".to_string(),
            x: 0,
            y: 0,
            z: 1,
        }];
        let adapter = ImageFolderClassificationAdapter::new(ImageFolderClassificationConfig {
            dataset_name: "colors".to_string(),
            image_schema: ImageClassificationSchema::ClassFolders,
            split: Split::Train,
            split_id: SplitId("train".to_string()),
            feed_width: 2,
            feed_height: 2,
            class_map,
            class_keep_percents: BTreeMap::new(),
            max_samples: None,
            sample_draw_seed: None,
        });
        let samples = adapter
            .stream(
                &DatasetSource {
                    uri: root.path().to_string_lossy().into_owned(),
                    bytes: Vec::new(),
                },
                &SplitId("train".to_string()),
            )
            .expect("stream");
        assert_eq!(samples.len(), 1);
        match &samples[0].target {
            Some(TypedTarget::Class { class_id, label }) => {
                assert_eq!(*class_id, 1);
                assert_eq!(label.as_deref(), Some("red"));
            }
            other => panic!("unexpected target {other:?}"),
        }
    }

    fn write_idx_pair(dir: &Path, image_name: &str, label_name: &str, frames: &[(u8, [u8; 4])]) {
        let mut images = Vec::new();
        images.extend_from_slice(&2051u32.to_be_bytes());
        images.extend_from_slice(&(frames.len() as u32).to_be_bytes());
        images.extend_from_slice(&2u32.to_be_bytes());
        images.extend_from_slice(&2u32.to_be_bytes());
        let mut labels = Vec::new();
        labels.extend_from_slice(&2049u32.to_be_bytes());
        labels.extend_from_slice(&(frames.len() as u32).to_be_bytes());
        for (label, pixels) in frames {
            images.extend_from_slice(pixels);
            labels.push(*label);
        }
        fs::write(dir.join(image_name), images).expect("images");
        fs::write(dir.join(label_name), labels).expect("labels");
    }

    #[test]
    fn idx_train_and_t10k_map_digits_onto_x() {
        let root = tempfile::tempdir().expect("temp");
        write_idx_pair(
            root.path(),
            "train-images-idx3-ubyte",
            "train-labels-idx1-ubyte",
            &[(1, [9, 9, 9, 9]), (0, [1, 1, 1, 1])],
        );
        write_idx_pair(
            root.path(),
            "t10k-images-idx3-ubyte",
            "t10k-labels-idx1-ubyte",
            &[(1, [8, 8, 8, 8])],
        );
        let scan = scan_image_classification_root(root.path()).expect("scan");
        assert_eq!(scan.schema, Some(ImageClassificationSchema::IdxImages));
        assert_eq!(scan.splits, vec!["train".to_string(), "test".to_string()]);
        assert_eq!(scan.image_width, Some(2));
        assert_eq!(scan.image_height, Some(2));
        assert!(scan.issues.is_empty());
        assert_eq!(scan.split_stats.len(), 2);
        assert_eq!(scan.split_stats[0].split, "train");
        assert_eq!(scan.split_stats[0].sample_count, 2);
        assert_eq!(scan.split_stats[1].split, "test");
        assert_eq!(scan.split_stats[1].sample_count, 1);
        let source = DatasetSource {
            uri: root.path().to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let adapter = ImageFolderClassificationAdapter::new(ImageFolderClassificationConfig {
            dataset_name: "mnist".to_string(),
            image_schema: ImageClassificationSchema::IdxImages,
            split: Split::Train,
            split_id: SplitId("train".to_string()),
            feed_width: 2,
            feed_height: 2,
            class_map: scan.classes.clone(),
            class_keep_percents: BTreeMap::new(),
            max_samples: None,
            sample_draw_seed: None,
        });
        let samples = adapter
            .stream(&source, &SplitId("train".to_string()))
            .expect("stream");
        let ids: Vec<u32> = samples
            .iter()
            .map(|sample| match &sample.target {
                Some(TypedTarget::Class { class_id, .. }) => *class_id,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(ids, vec![1, 0]);
        let capped = ImageFolderClassificationAdapter::new(ImageFolderClassificationConfig {
            dataset_name: "mnist".to_string(),
            image_schema: ImageClassificationSchema::IdxImages,
            split: Split::Train,
            split_id: SplitId("train".to_string()),
            feed_width: 2,
            feed_height: 2,
            class_map: scan.classes.clone(),
            class_keep_percents: BTreeMap::new(),
            max_samples: Some(1),
            sample_draw_seed: None,
        })
        .index(&source)
        .expect("capped index")
        .1;
        assert_eq!(capped.len(), 1);
        assert_eq!(capped[0].label, "1");

        // A seeded cap draws different images per seed; the dataset hash stays the split's.
        let full_hash = adapter.index(&source).expect("full index").0.content_hash;
        let mut drawn_labels = std::collections::BTreeSet::new();
        for seed in 0..32 {
            let (manifest, images) =
                ImageFolderClassificationAdapter::new(ImageFolderClassificationConfig {
                    dataset_name: "mnist".to_string(),
                    image_schema: ImageClassificationSchema::IdxImages,
                    split: Split::Train,
                    split_id: SplitId("train".to_string()),
                    feed_width: 2,
                    feed_height: 2,
                    class_map: scan.classes.clone(),
                    class_keep_percents: BTreeMap::new(),
                    max_samples: Some(1),
                    sample_draw_seed: Some(seed),
                })
                .index(&source)
                .expect("drawn index");
            assert_eq!(images.len(), 1);
            assert_eq!(manifest.content_hash, full_hash);
            drawn_labels.insert(images[0].label.clone());
        }
        assert_eq!(drawn_labels.len(), 2, "both images are drawn across seeds");
    }

    #[test]
    fn gzip_idx_is_the_same_train_split() {
        let root = tempfile::tempdir().expect("temp");
        let plain = tempfile::tempdir().expect("plain");
        write_idx_pair(
            plain.path(),
            "train-images-idx3-ubyte",
            "train-labels-idx1-ubyte",
            &[(7, [3, 3, 3, 3])],
        );
        for name in ["train-images-idx3-ubyte", "train-labels-idx1-ubyte"] {
            let raw = fs::read(plain.path().join(name)).expect("raw");
            let file = fs::File::create(root.path().join(format!("{name}.gz"))).expect("gz");
            let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
            std::io::Write::write_all(&mut encoder, &raw).expect("write");
            encoder.finish().expect("finish");
        }
        let scan = scan_image_classification_root(root.path()).expect("scan");
        assert_eq!(scan.splits, vec!["train".to_string()]);
        assert_eq!(scan.classes[0].label, "7");
        assert_eq!(scan.classes[0].z, 7);
    }

    #[test]
    fn idx_under_raw_or_with_dotted_names_is_mnist() {
        let root = tempfile::tempdir().expect("temp");
        let raw = root.path().join("raw");
        fs::create_dir(&raw).expect("raw");
        write_idx_pair(
            &raw,
            "train-images.idx3-ubyte",
            "train-labels.idx1-ubyte",
            &[(4, [2, 2, 2, 2])],
        );
        let scan = scan_image_classification_root(root.path()).expect("scan");
        assert_eq!(scan.schema, Some(ImageClassificationSchema::IdxImages));
        assert_eq!(scan.classes[0].z, 4);
        assert!(scan.issues.is_empty());
    }
}
