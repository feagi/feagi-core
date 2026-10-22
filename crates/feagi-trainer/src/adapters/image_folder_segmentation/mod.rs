//! Image-folder semantic segmentation adapter — maps paired image/label files into `IRSample`s.
//!
//! Dataset-specific layout is selected by explicit configuration (`cityscapes_fine` or
//! `paired_folders`). All filesystem access uses `source.uri` as the dataset root; `source.bytes`
//! is unused.

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

mod preview;

pub use preview::{colorize_mask_png, SegmentationPreview, SegmentationPreviewFrame};

/// Supported on-disk folder layouts for image + per-pixel label pairs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentationDatasetLayout {
    /// Cityscapes fine annotations (`leftImg8bit` + `gtFine` `labelIds` or `labelTrainIds`).
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
    /// Official Cityscapes trainId → output Z or `ignore_label`. Length 19 when set.
    #[serde(default)]
    pub train_id_remap: Option<Vec<u8>>,
}

/// One indexed image/mask pair. Pixels are loaded on visit, not during discover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageLabelPair {
    pub(crate) image_path: PathBuf,
    pub(crate) label_path: PathBuf,
    /// Official `*_gtFine_labelIds.png` pixels; map to evaluation trainIds on load.
    pub(crate) map_label_ids_to_train_ids: bool,
}

/// Official Cityscapes `id` → evaluation `trainId` (Cordts et al., cityscapesscripts `labels.py`).
/// Index is the class `id` stored in the official `gtFine` zip (`*_gtFine_labelIds.png`).
const CITYSCAPES_ID_TO_TRAIN_ID: [u8; 34] = [
    255, 255, 255, 255, 255, 255, 255, 0, 1, 255, 255, 2, 3, 4, 255, 255, 255, 5, 255, 6, 7, 8, 9,
    10, 11, 12, 13, 14, 15, 255, 255, 16, 17, 18,
];

fn cityscapes_label_id_to_train_id(label_id: u8) -> Result<u8, TrainerError> {
    if label_id == 255 {
        return Ok(255);
    }
    CITYSCAPES_ID_TO_TRAIN_ID
        .get(usize::from(label_id))
        .copied()
        .ok_or_else(|| TrainerError::Parse(format!("unsupported Cityscapes label id {label_id}")))
}

fn cityscapes_label_for_stem(city_lbl_dir: &Path, stem: &str) -> Option<(PathBuf, bool)> {
    let train_ids = city_lbl_dir.join(format!("{stem}_gtFine_labelTrainIds.png"));
    if train_ids.is_file() {
        return Some((train_ids, false));
    }
    let label_ids = city_lbl_dir.join(format!("{stem}_gtFine_labelIds.png"));
    if label_ids.is_file() {
        return Some((label_ids, true));
    }
    None
}

/// Adapter that converts paired image/label folders into segmentation `IRSample`s.
#[derive(Clone)]
pub struct ImageFolderSegmentationAdapter {
    config: ImageFolderSegmentationConfig,
    ingest_progress: Option<Arc<dyn Fn(u64, u64) + Send + Sync>>,
}

impl std::fmt::Debug for ImageFolderSegmentationAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageFolderSegmentationAdapter")
            .field("config", &self.config)
            .finish()
    }
}

impl ImageFolderSegmentationAdapter {
    /// Stable plugin id for this adapter.
    pub const PLUGIN_ID: &'static str = "image_folder_segmentation";

    /// Creates a new adapter from explicit configuration.
    pub fn new(config: ImageFolderSegmentationConfig) -> Self {
        Self {
            config,
            ingest_progress: None,
        }
    }

    /// Reports loaded-pair counts while `stream` resizes each image/mask.
    pub fn with_ingest_progress(mut self, progress: Arc<dyn Fn(u64, u64) + Send + Sync>) -> Self {
        self.ingest_progress = Some(progress);
        self
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

    /// One page of image + colorized-mask pairs for the Trainer source step.
    pub fn preview(
        &self,
        source: &DatasetSource,
        frame_offset: usize,
        frame_count: usize,
    ) -> Result<SegmentationPreview, TrainerError> {
        preview::preview_pairs(self, source, frame_offset, frame_count)
    }

    /// Indexes image/mask paths and builds the manifest without decoding pixels.
    pub fn index(
        &self,
        source: &DatasetSource,
    ) -> Result<(DatasetManifest, Vec<ImageLabelPair>), TrainerError> {
        let root = Self::dataset_root(source)?;
        let pairs = self.discover_pairs(root)?;
        Ok((self.manifest_from_pairs(source, &pairs), pairs))
    }

    /// Decodes one indexed pair into an [`IRSample`].
    pub fn load_indexed_sample(
        &self,
        pair: &ImageLabelPair,
        index: usize,
        dataset_version_id: &DatasetVersionId,
    ) -> Result<IRSample, TrainerError> {
        let width = self.config.feed_width;
        let height = self.config.feed_height;
        let image_bytes = Self::load_resized_png_bytes(&pair.image_path, width, height)?;
        let labels = self.load_resized_label_mask(pair, width, height)?;
        if labels.len() != (width as usize) * (height as usize) {
            return Err(TrainerError::Parse(format!(
                "label mask size mismatch for '{}'",
                pair.label_path.display()
            )));
        }
        Ok(IRSample {
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
        })
    }

    fn manifest_from_pairs(
        &self,
        source: &DatasetSource,
        pairs: &[ImageLabelPair],
    ) -> DatasetManifest {
        let content_fingerprint = Self::fingerprint_pairs(pairs);
        let dataset_version_id = self.dataset_version_id(&content_fingerprint);
        DatasetManifest {
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
        }
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
        let (img_root, lbl_root) = cityscapes_split_dirs(root, split)?;

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
                let Some((label_path, map_label_ids_to_train_ids)) =
                    cityscapes_label_for_stem(&city_lbl_dir, stem)
                else {
                    continue;
                };
                pairs.push(ImageLabelPair {
                    image_path: img_path,
                    label_path,
                    map_label_ids_to_train_ids,
                });
            }
        }
        if pairs.is_empty() {
            return Err(TrainerError::Parse(format!(
                "no Cityscapes image/mask pairs under images '{}' and labels '{}'. Official gtFine zip ships *_gtFine_labelIds.png next to each labeled frame (plus color, instance, polygons). Only those frames are used; unlabeled video frames are skipped. You do not generate labelTrainIds",
                img_root.display(),
                lbl_root.display()
            )));
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
                map_label_ids_to_train_ids: false,
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

    fn apply_train_id_remap(&self, labels: &mut [u8]) -> Result<(), TrainerError> {
        let Some(remap) = self.config.train_id_remap.as_ref() else {
            return Ok(());
        };
        if remap.len() != 19 {
            return Err(TrainerError::Config(
                "train_id_remap must have 19 entries, one per Cityscapes trainId".to_string(),
            ));
        }
        for label in labels.iter_mut() {
            if *label == self.config.ignore_label {
                continue;
            }
            let source = usize::from(*label);
            let dest = remap.get(source).copied().ok_or_else(|| {
                TrainerError::Parse(format!("mask trainId {label} is outside train_id_remap"))
            })?;
            if dest != self.config.ignore_label && u32::from(dest) >= self.config.class_count {
                return Err(TrainerError::Parse(format!(
                    "train_id_remap destination {dest} is outside class_count {}",
                    self.config.class_count
                )));
            }
            *label = dest;
        }
        Ok(())
    }

    fn load_resized_label_mask(
        &self,
        pair: &ImageLabelPair,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, TrainerError> {
        let path = &pair.label_path;
        let bytes = std::fs::read(path)
            .map_err(|e| TrainerError::Parse(format!("cannot read '{}': {e}", path.display())))?;
        let img = image::load_from_memory_with_format(&bytes, ImageFormat::Png).map_err(|e| {
            TrainerError::Parse(format!("cannot decode label png '{}': {e}", path.display()))
        })?;
        let gray = img.to_luma8();
        let resized = image::imageops::resize(&gray, width, height, FilterType::Nearest);
        let mut labels = resized.into_raw();
        if pair.map_label_ids_to_train_ids {
            for label in &mut labels {
                *label = cityscapes_label_id_to_train_id(*label)?;
            }
        }
        self.apply_train_id_remap(&mut labels)?;
        Ok(labels)
    }
}

/// Image prefix and matching label prefix. Zip extracts are tried before a combined tree
/// so a leftover `gtFine/` cannot steal pairs from `gtFine_trainvaltest/`.
const CITYSCAPES_LAYOUTS: &[(&[&str], &[&str])] = &[
    (
        &["leftImg8bit_trainvaltest", "leftImg8bit"],
        &["gtFine_trainvaltest", "gtFine"],
    ),
    (&["leftImg8bit"], &["gtFine"]),
];

fn join_prefixed(root: &Path, prefix: &[&str], split: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    for part in prefix {
        path.push(part);
    }
    path.push(split);
    path
}

fn first_matching_cityscapes_dirs(root: &Path, split: &str) -> Option<(PathBuf, PathBuf)> {
    CITYSCAPES_LAYOUTS
        .iter()
        .find_map(|(image_prefix, label_prefix)| {
            let img_root = join_prefixed(root, image_prefix, split);
            let lbl_root = join_prefixed(root, label_prefix, split);
            if img_root.is_dir() && lbl_root.is_dir() {
                Some((img_root, lbl_root))
            } else {
                None
            }
        })
}

/// Official zip-extract folder names. A click inside one of these resolves to its parent.
const CITYSCAPES_ZIP_EXTRACT_NAMES: &[&str] = &["gtFine_trainvaltest", "leftImg8bit_trainvaltest"];

/// Combined-layout folder names. A click inside one of these resolves to its parent.
const CITYSCAPES_COMBINED_FOLDER_NAMES: &[&str] = &["gtFine", "leftImg8bit"];

fn ancestor_named<'a>(path: &'a Path, name: &str) -> Option<&'a Path> {
    path.ancestors()
        .find(|ancestor| ancestor.file_name().and_then(|n| n.to_str()) == Some(name))
}

fn push_unique_root(roots: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !roots.iter().any(|existing| existing == &candidate) {
        roots.push(candidate);
    }
}

/// Selected folder, then the parent of an official zip extract or combined `gtFine`/`leftImg8bit`.
fn cityscapes_anchor_roots(selected: &Path) -> Vec<PathBuf> {
    let mut roots = vec![selected.to_path_buf()];
    for zip_name in CITYSCAPES_ZIP_EXTRACT_NAMES {
        if let Some(extract) = ancestor_named(selected, zip_name) {
            if let Some(parent) = extract.parent() {
                push_unique_root(&mut roots, parent.to_path_buf());
            }
        }
    }
    for folder in CITYSCAPES_COMBINED_FOLDER_NAMES {
        if let Some(combined) = ancestor_named(selected, folder) {
            let parent = match combined.parent() {
                Some(parent) => parent,
                None => continue,
            };
            let parent_name = parent.file_name().and_then(|n| n.to_str());
            if parent_name.is_some_and(|name| CITYSCAPES_ZIP_EXTRACT_NAMES.contains(&name)) {
                continue;
            }
            push_unique_root(&mut roots, parent.to_path_buf());
        }
    }
    roots
}

fn cityscapes_city_selection_note(selected: &Path) -> Option<String> {
    let parts: Vec<&str> = selected.iter().filter_map(|part| part.to_str()).collect();
    let gt_index = parts.iter().position(|part| *part == "gtFine")?;
    let after = &parts[gt_index + 1..];
    match after {
        [split, city, ..] if *split == "train" || *split == "val" || *split == "test" => Some(
            format!(
                "Selected folder is Cityscapes label city '{city}' inside gtFine/{split}. That city folder is not the dataset root. "
            ),
        ),
        [split] if *split == "train" || *split == "val" || *split == "test" => Some(format!(
            "Selected folder is Cityscapes label split gtFine/{split}. That split folder is not the dataset root. "
        )),
        [] => Some(
            "Selected folder is the Cityscapes gtFine label tree. That tree has no RGB frames. "
                .to_string(),
        ),
        _ => None,
    }
}

fn diagnose_cityscapes_dirs(
    selected: &Path,
    candidate: &Path,
    split: &str,
) -> Result<(PathBuf, PathBuf), TrainerError> {
    let matched = first_matching_cityscapes_dirs(candidate, split);
    let img_root = matched.as_ref().map(|(img, _)| img.clone()).or_else(|| {
        CITYSCAPES_LAYOUTS
            .iter()
            .map(|(image_prefix, _)| join_prefixed(candidate, image_prefix, split))
            .find(|path| path.is_dir())
    });
    let lbl_root = matched.as_ref().map(|(_, lbl)| lbl.clone()).or_else(|| {
        CITYSCAPES_LAYOUTS
            .iter()
            .map(|(_, label_prefix)| join_prefixed(candidate, label_prefix, split))
            .find(|path| path.is_dir())
    });
    let note = cityscapes_city_selection_note(selected).unwrap_or_default();
    match (img_root, lbl_root) {
        (Some(img_root), Some(lbl_root)) => Ok((img_root, lbl_root)),
        (None, Some(lbl_root)) => Err(TrainerError::Parse(format!(
            "{note}found Cityscapes labels at '{}' but no images. Cityscapes ships RGB frames as a separate zip (leftImg8bit_trainvaltest). Select the parent folder that contains both leftImg8bit (or leftImg8bit_trainvaltest) and gtFine (or gtFine_trainvaltest)",
            lbl_root.display()
        ))),
        (Some(img_root), None) => Err(TrainerError::Parse(format!(
            "{note}found Cityscapes images at '{}' but no labels. Cityscapes ships fine masks as a separate zip (gtFine_trainvaltest). Select the parent folder that contains both leftImg8bit (or leftImg8bit_trainvaltest) and gtFine (or gtFine_trainvaltest)",
            img_root.display()
        ))),
        (None, None) => Err(TrainerError::Parse(format!(
            "{note}missing Cityscapes image and label directories under '{}'. Expected leftImg8bit/{split} and gtFine/{split}, or the official zip extracts leftImg8bit_trainvaltest/leftImg8bit/{split} and gtFine_trainvaltest/gtFine/{split}",
            candidate.display()
        ))),
    }
}

/// Resolves the image and label split directories for a Cityscapes fine selection.
fn cityscapes_split_dirs(root: &Path, split: &str) -> Result<(PathBuf, PathBuf), TrainerError> {
    let candidates = cityscapes_anchor_roots(root);
    for candidate in &candidates {
        if let Some(dirs) = first_matching_cityscapes_dirs(candidate, split) {
            return Ok(dirs);
        }
    }
    let widest = candidates.last().map_or(root, PathBuf::as_path);
    diagnose_cityscapes_dirs(root, widest, split)
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
        let (manifest, _) = self.index(source)?;
        Ok(manifest)
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
        if let Some(remap) = &self.config.train_id_remap {
            if remap.len() != 19 {
                issues.push("train_id_remap must have 19 entries".to_string());
            }
            for dest in remap {
                if *dest != self.config.ignore_label && u32::from(*dest) >= self.config.class_count
                {
                    issues.push(format!(
                        "train_id_remap destination {dest} is outside class_count {}",
                        self.config.class_count
                    ));
                }
            }
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
        let (manifest, pairs) = self.index(source)?;
        let mut samples = Vec::with_capacity(pairs.len());
        let total = pairs.len() as u64;
        if total > 0 {
            if let Some(progress) = &self.ingest_progress {
                progress(0, total);
            }
        }
        for (index, pair) in pairs.iter().enumerate() {
            samples.push(self.load_indexed_sample(pair, index, &manifest.dataset_version_id)?);
            if let Some(progress) = &self.ingest_progress {
                progress((index + 1) as u64, total);
            }
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
            train_id_remap: None,
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
        let (indexed_manifest, pairs) = adapter.index(&source).expect("index");
        assert_eq!(indexed_manifest.splits[0].sample_count, 1);
        assert_eq!(pairs.len(), 1);
        let loaded = adapter
            .load_indexed_sample(&pairs[0], 0, &indexed_manifest.dataset_version_id)
            .expect("load one");
        assert_eq!(loaded.sample_id, samples[0].sample_id);
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

    #[test]
    fn stream_reports_ingest_progress() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let images = root.join("images");
        let labels = root.join("labels");
        fs::create_dir_all(&images).expect("mkdir");
        fs::create_dir_all(&labels).expect("mkdir");
        write_rgb_png(&images.join("a.png"), 2, 2);
        write_png(&labels.join("a.png"), 2, 2, 1);
        write_rgb_png(&images.join("b.png"), 2, 2);
        write_png(&labels.join("b.png"), 2, 2, 2);

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
            train_id_remap: None,
        };
        let reports = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let adapter = ImageFolderSegmentationAdapter::new(config.clone()).with_ingest_progress({
            let reports = std::sync::Arc::clone(&reports);
            std::sync::Arc::new(move |done, total| {
                reports.lock().expect("progress lock").push((done, total));
            })
        });
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let samples = adapter
            .stream(&source, &SplitId("train".to_string()))
            .expect("stream");
        assert_eq!(samples.len(), 2);
        assert_eq!(
            reports.lock().expect("progress lock").as_slice(),
            &[(0, 2), (1, 2), (2, 2)]
        );
        let reports = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let index_adapter = ImageFolderSegmentationAdapter::new(config).with_ingest_progress({
            let reports = std::sync::Arc::clone(&reports);
            std::sync::Arc::new(move |done, total| {
                reports.lock().expect("progress lock").push((done, total));
            })
        });
        let (_manifest, pairs) = index_adapter.index(&source).expect("index");
        assert_eq!(pairs.len(), 2);
        assert!(reports.lock().expect("progress lock").is_empty());
    }

    fn cityscapes_config() -> ImageFolderSegmentationConfig {
        ImageFolderSegmentationConfig {
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
        }
    }

    #[test]
    fn cityscapes_accepts_official_zip_extract_folder_names() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let img_dir = root
            .join("leftImg8bit_trainvaltest")
            .join("leftImg8bit")
            .join("train")
            .join("aachen");
        let lbl_dir = root
            .join("gtFine_trainvaltest")
            .join("gtFine")
            .join("train")
            .join("aachen");
        fs::create_dir_all(&img_dir).expect("mkdir");
        fs::create_dir_all(&lbl_dir).expect("mkdir");
        write_rgb_png(&img_dir.join("aachen_000000_000019_leftImg8bit.png"), 2, 2);
        write_png(
            &lbl_dir.join("aachen_000000_000019_gtFine_labelTrainIds.png"),
            2,
            2,
            3,
        );

        let adapter = ImageFolderSegmentationAdapter::new(cityscapes_config());
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let samples = adapter
            .stream(&source, &SplitId("train".to_string()))
            .expect("stream");
        assert_eq!(samples.len(), 1);
    }

    #[test]
    fn cityscapes_city_folder_resolves_to_parent_of_both_zip_extracts() {
        let dir = tempdir().expect("tempdir");
        let parent = dir.path();
        let img_dir = parent
            .join("leftImg8bit_trainvaltest")
            .join("leftImg8bit")
            .join("train")
            .join("hamburg");
        let lbl_city = parent
            .join("gtFine_trainvaltest")
            .join("gtFine")
            .join("train")
            .join("hamburg");
        fs::create_dir_all(&img_dir).expect("mkdir");
        fs::create_dir_all(&lbl_city).expect("mkdir");
        write_rgb_png(&img_dir.join("hamburg_000000_000019_leftImg8bit.png"), 2, 2);
        write_png(
            &lbl_city.join("hamburg_000000_000019_gtFine_labelTrainIds.png"),
            2,
            2,
            8,
        );

        let adapter = ImageFolderSegmentationAdapter::new(cityscapes_config());
        let source = DatasetSource {
            uri: lbl_city.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let samples = adapter
            .stream(&source, &SplitId("train".to_string()))
            .expect("stream from city folder");
        assert_eq!(samples.len(), 1);
    }

    #[test]
    fn cityscapes_city_folder_without_images_names_hamburg_and_the_image_zip() {
        let dir = tempdir().expect("tempdir");
        let city = dir
            .path()
            .join("gtFine_trainvaltest")
            .join("gtFine")
            .join("train")
            .join("hamburg");
        fs::create_dir_all(&city).expect("mkdir");
        let adapter = ImageFolderSegmentationAdapter::new(cityscapes_config());
        let source = DatasetSource {
            uri: city.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let error = adapter.discover(&source).expect_err("labels city only");
        let message = error.to_string();
        assert!(message.contains("label city 'hamburg'"), "{message}");
        assert!(message.contains("leftImg8bit_trainvaltest"), "{message}");
    }

    #[test]
    fn cityscapes_labels_only_extract_names_the_image_zip() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path().join("gtFine_trainvaltest");
        fs::create_dir_all(root.join("gtFine").join("train").join("aachen")).expect("mkdir");
        let adapter = ImageFolderSegmentationAdapter::new(cityscapes_config());
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let error = adapter.discover(&source).expect_err("labels only");
        assert!(error.to_string().contains("leftImg8bit_trainvaltest"));
        assert!(error.to_string().contains("parent folder"));
    }

    #[test]
    fn official_label_id_26_car_maps_to_train_id_13() {
        assert_eq!(cityscapes_label_id_to_train_id(7).expect("road"), 0);
        assert_eq!(cityscapes_label_id_to_train_id(26).expect("car"), 13);
        assert_eq!(cityscapes_label_id_to_train_id(0).expect("unlabeled"), 255);
        assert_eq!(cityscapes_label_id_to_train_id(255).expect("ignore"), 255);
        assert!(cityscapes_label_id_to_train_id(200).is_err());
    }

    #[test]
    fn cityscapes_reads_official_gtfine_label_ids() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let img_dir = root
            .join("leftImg8bit_trainvaltest")
            .join("leftImg8bit")
            .join("train")
            .join("zurich");
        let lbl_dir = root
            .join("gtFine_trainvaltest")
            .join("gtFine")
            .join("train")
            .join("zurich");
        fs::create_dir_all(&img_dir).expect("mkdir");
        fs::create_dir_all(&lbl_dir).expect("mkdir");
        write_rgb_png(&img_dir.join("zurich_000069_000019_leftImg8bit.png"), 2, 2);
        write_png(
            &lbl_dir.join("zurich_000069_000019_gtFine_labelIds.png"),
            2,
            2,
            26,
        );

        let adapter = ImageFolderSegmentationAdapter::new(cityscapes_config());
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let samples = adapter
            .stream(&source, &SplitId("train".to_string()))
            .expect("official labelIds");
        assert_eq!(samples.len(), 1);
        match &samples[0].target {
            Some(TypedTarget::SegmentationMask { labels, .. }) => {
                assert!(labels.iter().all(|&v| v == 13), "{labels:?}");
            }
            other => panic!("expected mask, got {other:?}"),
        }
    }

    #[test]
    fn cityscapes_missing_label_ids_names_official_zip_files() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let img_dir = root.join("leftImg8bit").join("train").join("zurich");
        let lbl_dir = root.join("gtFine").join("train").join("zurich");
        fs::create_dir_all(&img_dir).expect("mkdir");
        fs::create_dir_all(&lbl_dir).expect("mkdir");
        write_rgb_png(&img_dir.join("zurich_000069_000019_leftImg8bit.png"), 2, 2);
        write_png(
            &lbl_dir.join("zurich_000069_000019_gtFine_color.png"),
            2,
            2,
            1,
        );

        let adapter = ImageFolderSegmentationAdapter::new(cityscapes_config());
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let error = adapter.discover(&source).expect_err("color only");
        let message = error.to_string();
        assert!(message.contains("labelIds.png"), "{message}");
        assert!(
            message.contains("no Cityscapes image/mask pairs"),
            "{message}"
        );
    }

    #[test]
    fn cityscapes_skips_unlabeled_video_frames() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let img_dir = root.join("leftImg8bit").join("train").join("hamburg");
        let lbl_dir = root.join("gtFine").join("train").join("hamburg");
        fs::create_dir_all(&img_dir).expect("mkdir");
        fs::create_dir_all(&lbl_dir).expect("mkdir");
        write_rgb_png(&img_dir.join("hamburg_000000_000019_leftImg8bit.png"), 2, 2);
        write_rgb_png(&img_dir.join("hamburg_000000_061790_leftImg8bit.png"), 2, 2);
        write_png(
            &lbl_dir.join("hamburg_000000_000019_gtFine_labelIds.png"),
            2,
            2,
            26,
        );

        let adapter = ImageFolderSegmentationAdapter::new(cityscapes_config());
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let samples = adapter
            .stream(&source, &SplitId("train".to_string()))
            .expect("skip unlabeled");
        assert_eq!(samples.len(), 1);
        assert!(samples[0]
            .sample_id
            .0
            .contains("hamburg_000000_000019_leftImg8bit.png"));
    }

    #[test]
    fn cityscapes_applies_operator_train_id_remap() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let img_dir = root.join("leftImg8bit").join("train").join("aachen");
        let lbl_dir = root.join("gtFine").join("train").join("aachen");
        fs::create_dir_all(&img_dir).expect("mkdir");
        fs::create_dir_all(&lbl_dir).expect("mkdir");
        write_rgb_png(&img_dir.join("aachen_000000_000019_leftImg8bit.png"), 2, 2);
        write_png(
            &lbl_dir.join("aachen_000000_000019_gtFine_labelTrainIds.png"),
            2,
            2,
            14,
        );
        let mut config = cityscapes_config();
        config.class_count = 1;
        let mut remap = vec![255_u8; 19];
        remap[13] = 0;
        remap[14] = 0;
        config.train_id_remap = Some(remap);

        let adapter = ImageFolderSegmentationAdapter::new(config);
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let samples = adapter
            .stream(&source, &SplitId("train".to_string()))
            .expect("remap");
        match &samples[0].target {
            Some(TypedTarget::SegmentationMask { labels, .. }) => {
                assert!(labels.iter().all(|&v| v == 0), "{labels:?}");
            }
            other => panic!("expected mask, got {other:?}"),
        }
    }

    #[test]
    fn cityscapes_prefers_zip_extract_labels_over_stub_gtfine() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let img_dir = root
            .join("leftImg8bit_trainvaltest")
            .join("leftImg8bit")
            .join("train")
            .join("hamburg");
        let stub = root.join("gtFine").join("train").join("hamburg");
        let lbl_dir = root
            .join("gtFine_trainvaltest")
            .join("gtFine")
            .join("train")
            .join("hamburg");
        fs::create_dir_all(&img_dir).expect("mkdir");
        fs::create_dir_all(&stub).expect("mkdir");
        fs::create_dir_all(&lbl_dir).expect("mkdir");
        write_rgb_png(&img_dir.join("hamburg_000000_061790_leftImg8bit.png"), 2, 2);
        write_png(
            &lbl_dir.join("hamburg_000000_061790_gtFine_labelIds.png"),
            2,
            2,
            26,
        );

        let adapter = ImageFolderSegmentationAdapter::new(cityscapes_config());
        let source = DatasetSource {
            uri: root.to_string_lossy().into_owned(),
            bytes: Vec::new(),
        };
        let samples = adapter
            .stream(&source, &SplitId("train".to_string()))
            .expect("zip extract labels");
        assert_eq!(samples.len(), 1);
        match &samples[0].target {
            Some(TypedTarget::SegmentationMask { labels, .. }) => {
                assert!(labels.iter().all(|&v| v == 13), "{labels:?}");
            }
            other => panic!("expected mask, got {other:?}"),
        }
    }
}
