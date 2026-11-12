//! Sample thumbnails for the Trainer's dataset step.
//!
//! One public function, [`preview_image_classification`], pages through the labelled images of a
//! split, optionally for one class, in a seeded shuffle. It never applies the class keep or the
//! sample cap of a run: it shows what is on disk.
//!
//! The discovered index of one `(root, split)` is cached so paging does not rescan a large
//! dataset. A rescan ([`super::scan_image_classification_root`]) drops the cache.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use image::imageops::FilterType;
use image::ImageFormat;

use super::{
    cifar, read_idx_bytes, read_idx_image_header, scan_image_classification_root, svhn_mat,
    ClassifiedImage, ImageClassCount, ImageClassificationSchema, ImageFolderClassificationAdapter,
    ImageFolderClassificationConfig, PackedRecord,
};
use crate::adapters::sample_draw::shuffled_order;
use crate::contracts::{Split, SplitId};
use crate::error::TrainerError;

/// What to show: one page of a split, filtered by class, in a seeded order.
#[derive(Debug, Clone)]
pub struct ImageClassificationPreviewRequest<'a> {
    pub root: &'a Path,
    /// `train`, `val` or `test`.
    pub split: &'a str,
    /// One class label, or `None` for every class.
    pub class_label: Option<&'a str>,
    /// Fixes the shuffled order. The same seed pages through the same order.
    pub seed: u64,
    pub offset: usize,
    pub count: usize,
    /// Longest thumbnail side in pixels. Smaller images keep their size.
    pub max_side: u32,
}

/// One thumbnail with its class and its position in the split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageClassificationPreviewFrame {
    pub png: Vec<u8>,
    pub label: String,
    /// Position of the image in the split's on-disk order.
    pub index: u64,
}

/// One page of thumbnails plus the totals the page was cut from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageClassificationPreviewPage {
    pub frames: Vec<ImageClassificationPreviewFrame>,
    /// Images that match the class filter in this split.
    pub total_for_filter: u64,
    /// Images per class in this split, in class-map order.
    pub class_counts: Vec<ImageClassCount>,
}

struct CachedIndex {
    root: PathBuf,
    split: String,
    images: Arc<Vec<ClassifiedImage>>,
    class_counts: Vec<ImageClassCount>,
}

static CACHE: Mutex<Option<CachedIndex>> = Mutex::new(None);

/// Forgets the cached index. Called by every rescan so a changed dataset is read again.
pub(super) fn drop_cache() {
    if let Ok(mut guard) = CACHE.lock() {
        *guard = None;
    }
}

/// Pages through the labelled images of one split.
///
/// # Errors
/// An unknown class label, an unavailable or unlabelled split, an offset past the end of the
/// filtered list, or an image that cannot be read.
pub fn preview_image_classification(
    request: &ImageClassificationPreviewRequest<'_>,
) -> Result<ImageClassificationPreviewPage, TrainerError> {
    if request.count == 0 {
        return Err(TrainerError::Config(
            "image preview count must be greater than zero".to_string(),
        ));
    }
    let (images, class_counts) = cached_index(request.root, request.split)?;
    if let Some(label) = request.class_label {
        if !class_counts.iter().any(|entry| entry.label == label) {
            return Err(TrainerError::Parse(format!(
                "class '{label}' is not in this dataset. Known classes: {}",
                class_counts
                    .iter()
                    .map(|entry| entry.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
    }
    let matching: Vec<usize> = images
        .iter()
        .enumerate()
        .filter(|(_, image)| {
            request.class_label.is_none() || request.class_label == Some(image.label.as_str())
        })
        .map(|(index, _)| index)
        .collect();
    let total = matching.len();
    if request.offset > 0 && request.offset >= total {
        return Err(TrainerError::Parse(format!(
            "preview offset {} is past the end: {total} images match in split '{}'",
            request.offset, request.split
        )));
    }
    let order = shuffled_order(total, request.seed);
    let end = request.offset.saturating_add(request.count).min(total);
    let picked: Vec<usize> = order
        .get(request.offset.min(total)..end)
        .unwrap_or(&[])
        .iter()
        .map(|&slot| matching[slot])
        .collect();
    let pngs = thumbnails(&images, &picked, request.max_side)?;
    let frames = picked
        .iter()
        .zip(pngs)
        .map(|(&index, png)| ImageClassificationPreviewFrame {
            png,
            label: images[index].label.clone(),
            index: index as u64,
        })
        .collect();
    Ok(ImageClassificationPreviewPage {
        frames,
        total_for_filter: total as u64,
        class_counts,
    })
}

fn cached_index(
    root: &Path,
    split: &str,
) -> Result<(Arc<Vec<ClassifiedImage>>, Vec<ImageClassCount>), TrainerError> {
    if let Ok(guard) = CACHE.lock() {
        if let Some(entry) = guard.as_ref() {
            if entry.root == root && entry.split == split {
                return Ok((entry.images.clone(), entry.class_counts.clone()));
            }
        }
    }
    let scan = scan_image_classification_root(root)?;
    let schema = scan.schema.ok_or_else(|| {
        TrainerError::Parse(format!(
            "no image classification dataset was found under '{}'",
            root.display()
        ))
    })?;
    if !scan.splits.iter().any(|name| name == split) {
        let unlabelled = if schema == ImageClassificationSchema::ImageNet && split == "test" {
            " This split is unlabelled, so its images have no class to show."
        } else {
            ""
        };
        return Err(TrainerError::Parse(format!(
            "split '{split}' is not available for this dataset (found: {}).{unlabelled}",
            scan.splits.join(", ")
        )));
    }
    let split_value = match split {
        "train" => Split::Train,
        "val" => Split::Val,
        "test" => Split::Test,
        other => {
            return Err(TrainerError::Config(format!(
                "image classification does not use split '{other}'"
            )))
        }
    };
    let adapter = ImageFolderClassificationAdapter::new(ImageFolderClassificationConfig {
        dataset_name: "preview".to_string(),
        image_schema: schema,
        split: split_value,
        split_id: SplitId(split.to_string()),
        feed_width: 1,
        feed_height: 1,
        class_map: scan.classes.clone(),
        class_keep_percents: BTreeMap::new(),
        max_samples: None,
        sample_draw_seed: None,
    });
    let images = adapter.discover_images(root)?;
    let mut totals: BTreeMap<&str, u64> = BTreeMap::new();
    for image in &images {
        *totals.entry(image.label.as_str()).or_insert(0) += 1;
    }
    let class_counts: Vec<ImageClassCount> = scan
        .classes
        .iter()
        .map(|class| ImageClassCount {
            label: class.label.clone(),
            count: totals.get(class.label.as_str()).copied().unwrap_or(0),
        })
        .collect();
    let images = Arc::new(images);
    if let Ok(mut guard) = CACHE.lock() {
        *guard = Some(CachedIndex {
            root: root.to_path_buf(),
            split: split.to_string(),
            images: images.clone(),
            class_counts: class_counts.clone(),
        });
    }
    Ok((images, class_counts))
}

/// Encodes the picked images, in the order given.
fn thumbnails(
    images: &[ClassifiedImage],
    picked: &[usize],
    max_side: u32,
) -> Result<Vec<Vec<u8>>, TrainerError> {
    // IDX: read and gunzip the image file once for the whole page.
    let mut idx_bytes: Option<(PathBuf, Vec<u8>)> = None;
    let mut out = Vec::with_capacity(picked.len());
    for &index in picked {
        let image = &images[index];
        let png = match &image.packed {
            PackedRecord::File => file_thumbnail(&image.path, max_side)?,
            PackedRecord::Idx(record) => {
                if idx_bytes.as_ref().map(|(path, _)| path) != Some(&image.path) {
                    read_idx_image_header(&image.path)?;
                    idx_bytes = Some((image.path.clone(), read_idx_bytes(&image.path)?));
                }
                let bytes = &idx_bytes.as_ref().expect("idx bytes just loaded").1;
                idx_frame_png(bytes, record.record, record.rows, record.cols, &image.path)?
            }
            PackedRecord::Cifar10 { record } => {
                cifar::load_cifar10_png(&image.path, *record, 32, 32)?
            }
            PackedRecord::Cifar100 { record } => {
                cifar::load_cifar100_png(&image.path, *record, 32, 32)?
            }
            PackedRecord::Svhn {
                record,
                data_offset,
            } => svhn_mat::load_svhn_png(&image.path, *record, *data_offset, 32, 32)?,
        };
        out.push(png);
    }
    Ok(out)
}

/// Decodes one file and fits it inside `max_side` with its aspect kept. Small images stay as is.
fn file_thumbnail(path: &Path, max_side: u32) -> Result<Vec<u8>, TrainerError> {
    let bytes = std::fs::read(path).map_err(|error| {
        TrainerError::Parse(format!("cannot read '{}': {error}", path.display()))
    })?;
    let decoded = image::load_from_memory(&bytes).map_err(|error| {
        TrainerError::Parse(format!("cannot decode '{}': {error}", path.display()))
    })?;
    let fitted = if decoded.width() > max_side || decoded.height() > max_side {
        decoded.resize(max_side, max_side, FilterType::Triangle)
    } else {
        decoded
    };
    encode_png(&image::DynamicImage::ImageRgb8(fitted.to_rgb8()))
}

fn idx_frame_png(
    bytes: &[u8],
    record: u32,
    rows: u32,
    cols: u32,
    path: &Path,
) -> Result<Vec<u8>, TrainerError> {
    let pixels = (rows as usize)
        .checked_mul(cols as usize)
        .ok_or_else(|| TrainerError::Parse("IDX frame size overflows".to_string()))?;
    let start = 16 + (record as usize) * pixels;
    let frame = bytes.get(start..start + pixels).ok_or_else(|| {
        TrainerError::Parse(format!(
            "IDX record {record} is outside '{}'",
            path.display()
        ))
    })?;
    let gray = image::GrayImage::from_raw(cols, rows, frame.to_vec()).ok_or_else(|| {
        TrainerError::Parse(format!(
            "cannot build a grayscale frame from '{}'",
            path.display()
        ))
    })?;
    encode_png(&image::DynamicImage::ImageLuma8(gray))
}

fn encode_png(image: &image::DynamicImage) -> Result<Vec<u8>, TrainerError> {
    let mut out = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut out), ImageFormat::Png)
        .map_err(|error| TrainerError::Parse(format!("cannot encode thumbnail png: {error}")))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn page(
        root: &Path,
        split: &str,
        class: Option<&str>,
        offset: usize,
        count: usize,
    ) -> Result<ImageClassificationPreviewPage, TrainerError> {
        preview_image_classification(&ImageClassificationPreviewRequest {
            root,
            split,
            class_label: class,
            seed: 7,
            offset,
            count,
            max_side: 16,
        })
    }

    fn assert_png(bytes: &[u8]) {
        assert!(bytes.starts_with(&[137, 80, 78, 71]));
    }

    fn write_image(path: &Path, width: u32, height: u32) {
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        image::RgbImage::from_pixel(width, height, image::Rgb([9, 8, 7]))
            .save(path)
            .expect("image");
    }

    fn idx_fixture(root: &Path) {
        let mut images = Vec::new();
        for value in [2051_u32, 3, 2, 2] {
            images.extend_from_slice(&value.to_be_bytes());
        }
        images.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        fs::write(root.join("train-images-idx3-ubyte"), images).expect("images");
        let mut labels = Vec::new();
        for value in [2049_u32, 3] {
            labels.extend_from_slice(&value.to_be_bytes());
        }
        labels.extend_from_slice(&[0, 1, 1]);
        fs::write(root.join("train-labels-idx1-ubyte"), labels).expect("labels");
    }

    #[test]
    fn class_folders_filter_by_class_and_keep_aspect() {
        let root = tempfile::tempdir().expect("temp");
        write_image(&root.path().join("train/red/a.png"), 40, 20);
        write_image(&root.path().join("train/red/b.png"), 4, 4);
        write_image(&root.path().join("train/blue/c.png"), 4, 4);
        let all = page(root.path(), "train", None, 0, 24).expect("all");
        assert_eq!(all.total_for_filter, 3);
        assert_eq!(all.frames.len(), 3);
        let red = page(root.path(), "train", Some("red"), 0, 24).expect("red");
        assert_eq!(red.total_for_filter, 2);
        assert!(red.frames.iter().all(|frame| frame.label == "red"));
        let counts: Vec<(&str, u64)> = red
            .class_counts
            .iter()
            .map(|entry| (entry.label.as_str(), entry.count))
            .collect();
        assert_eq!(counts, vec![("blue", 1), ("red", 2)]);
        let wide = red
            .frames
            .iter()
            .map(|frame| image::load_from_memory(&frame.png).expect("png"))
            .find(|decoded| decoded.width() > 4)
            .expect("wide frame");
        assert_eq!((wide.width(), wide.height()), (16, 8));
        let again = page(root.path(), "train", Some("red"), 0, 24).expect("again");
        assert_eq!(red, again);
    }

    #[test]
    fn filename_xyz_schema_previews() {
        let root = tempfile::tempdir().expect("temp");
        write_image(&root.path().join("train/red_0_0_0.jpg"), 4, 4);
        write_image(&root.path().join("train/blue_0_0_1.jpg"), 4, 4);
        let blue = page(root.path(), "train", Some("blue"), 0, 24).expect("blue");
        assert_eq!(blue.total_for_filter, 1);
        assert_eq!(blue.frames[0].label, "blue");
        assert_png(&blue.frames[0].png);
    }

    #[test]
    fn idx_frames_decode_from_one_read() {
        let root = tempfile::tempdir().expect("temp");
        idx_fixture(root.path());
        let ones = page(root.path(), "train", Some("1"), 0, 24).expect("ones");
        assert_eq!(ones.total_for_filter, 2);
        let mut indexes: Vec<u64> = ones.frames.iter().map(|frame| frame.index).collect();
        indexes.sort_unstable();
        assert_eq!(indexes, vec![1, 2]);
        let decoded = image::load_from_memory(&ones.frames[0].png).expect("png");
        assert_eq!((decoded.width(), decoded.height()), (2, 2));
    }

    #[test]
    fn cifar_binary_previews() {
        let root = tempfile::tempdir().expect("temp");
        let mut record = vec![3_u8];
        record.extend(vec![9_u8; 3072]);
        fs::write(root.path().join("data_batch_1.bin"), record).expect("batch");
        let cats = page(root.path(), "train", Some("cat"), 0, 24).expect("cats");
        assert_eq!(cats.total_for_filter, 1);
        assert_png(&cats.frames[0].png);
    }

    #[test]
    fn svhn_mat_previews() {
        let root = tempfile::tempdir().expect("temp");
        super::super::svhn_mat::tests::write_one_frame(&root.path().join("train_32x32.mat"), 10.0);
        let zeros = page(root.path(), "train", Some("0"), 0, 24).expect("zeros");
        assert_eq!(zeros.total_for_filter, 1);
        assert_png(&zeros.frames[0].png);
    }

    #[test]
    fn imagenet_val_previews_and_test_is_unlabelled() {
        let root = tempfile::tempdir().expect("temp");
        write_image(&root.path().join("train/n02119789/a.jpg"), 4, 4);
        write_image(&root.path().join("val/ILSVRC2012_val_00000001.jpg"), 4, 4);
        fs::write(
            root.path().join("ILSVRC2012_validation_ground_truth.txt"),
            "1\n",
        )
        .expect("gt");
        let val = page(root.path(), "val", Some("n02119789"), 0, 24).expect("val");
        assert_eq!(val.total_for_filter, 1);
        assert_eq!(val.frames[0].label, "n02119789");
        let error = page(root.path(), "test", None, 0, 24).expect_err("unlabelled");
        assert!(error.to_string().contains("unlabelled"), "{error}");
    }

    #[test]
    fn unknown_label_and_offset_past_end_give_clear_errors() {
        let root = tempfile::tempdir().expect("temp");
        write_image(&root.path().join("train/red/a.png"), 4, 4);
        let unknown = page(root.path(), "train", Some("green"), 0, 24).expect_err("label");
        assert!(unknown.to_string().contains("class 'green'"), "{unknown}");
        let past = page(root.path(), "train", None, 5, 24).expect_err("offset");
        assert!(past.to_string().contains("past the end"), "{past}");
        let missing = page(root.path(), "val", None, 0, 24).expect_err("split");
        assert!(missing.to_string().contains("not available"), "{missing}");
    }
}
