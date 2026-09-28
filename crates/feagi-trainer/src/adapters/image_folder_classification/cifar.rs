//! CIFAR-10 and CIFAR-100 binary batches.
//!
//! CIFAR-10 records are 3,073 bytes: one label, then 1,024 red, green, and blue bytes.
//! CIFAR-100 records are 3,074 bytes: coarse label, fine label, then the same pixels.
//! This reader uses the fine label. Pixels are row-major inside each channel.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::contracts::Split;
use crate::error::TrainerError;

use super::{
    class_id_for_label, locate_single_child, resized_rgb_png, split_dir_name, ClassVoxel,
    ClassifiedImage, ImageClassificationScan, ImageClassificationSchema, PackedRecord,
};

const WIDTH: u32 = 32;
const HEIGHT: u32 = 32;
const CHANNEL_BYTES: usize = 1024;
const CIFAR10_STRIDE: usize = 1 + CHANNEL_BYTES * 3;
const CIFAR100_STRIDE: usize = 2 + CHANNEL_BYTES * 3;

const CIFAR10_TRAIN: [&str; 5] = [
    "data_batch_1.bin",
    "data_batch_2.bin",
    "data_batch_3.bin",
    "data_batch_4.bin",
    "data_batch_5.bin",
];
const CIFAR10_TEST: &str = "test_batch.bin";
const CIFAR100_TRAIN: &str = "train.bin";
const CIFAR100_TEST: &str = "test.bin";

const CIFAR10_LABELS: [&str; 10] = [
    "airplane",
    "automobile",
    "bird",
    "cat",
    "deer",
    "dog",
    "frog",
    "horse",
    "ship",
    "truck",
];

/// Fine-label order published with the CIFAR-100 binary batches.
const CIFAR100_FINE_LABELS: [&str; 100] = [
    "apple",
    "aquarium_fish",
    "baby",
    "bear",
    "beaver",
    "bed",
    "bee",
    "beetle",
    "bicycle",
    "bottle",
    "bowl",
    "boy",
    "bridge",
    "bus",
    "butterfly",
    "camel",
    "can",
    "castle",
    "caterpillar",
    "cattle",
    "chair",
    "chimpanzee",
    "clock",
    "cloud",
    "cockroach",
    "couch",
    "crab",
    "crocodile",
    "cup",
    "dinosaur",
    "dolphin",
    "elephant",
    "flatfish",
    "forest",
    "fox",
    "girl",
    "hamster",
    "house",
    "kangaroo",
    "keyboard",
    "lamp",
    "lawn_mower",
    "leopard",
    "lion",
    "lizard",
    "lobster",
    "man",
    "maple_tree",
    "motorcycle",
    "mountain",
    "mouse",
    "mushroom",
    "oak_tree",
    "orange",
    "orchid",
    "otter",
    "palm_tree",
    "pear",
    "pickup_truck",
    "pine_tree",
    "plain",
    "plate",
    "poppy",
    "porcupine",
    "possum",
    "rabbit",
    "raccoon",
    "ray",
    "road",
    "rocket",
    "rose",
    "sea",
    "seal",
    "shark",
    "shrew",
    "skunk",
    "skyscraper",
    "snail",
    "snake",
    "spider",
    "squirrel",
    "streetcar",
    "sunflower",
    "sweet_pepper",
    "table",
    "tank",
    "telephone",
    "television",
    "tiger",
    "tractor",
    "train",
    "trout",
    "tulip",
    "turtle",
    "wardrobe",
    "whale",
    "willow_tree",
    "wolf",
    "woman",
    "worm",
];

pub(super) fn scan_if_present(
    root: &Path,
) -> Result<Option<ImageClassificationScan>, TrainerError> {
    let Some(dir) = locate_single_child(root, directory_has_cifar, "CIFAR")? else {
        return Ok(None);
    };
    if directory_has_cifar10(&dir)? {
        return Ok(Some(scan_cifar10(&dir)?));
    }
    Ok(Some(scan_cifar100(&dir)?))
}

pub(super) fn discover_split(
    root: &Path,
    schema: &ImageClassificationSchema,
    split: &Split,
    class_map: &[ClassVoxel],
    class_count: u32,
) -> Result<Vec<ClassifiedImage>, TrainerError> {
    let dir = locate_single_child(root, directory_has_cifar, "CIFAR")?.ok_or_else(|| {
        TrainerError::Parse(format!(
            "CIFAR batches were not found under '{}'",
            root.display()
        ))
    })?;
    let split_name = split_dir_name(split)?;
    let files = match (schema, split_name) {
        (ImageClassificationSchema::Cifar10Binary, "train") => CIFAR10_TRAIN
            .iter()
            .map(|name| dir.join(name))
            .filter(|path| path.is_file())
            .collect::<Vec<_>>(),
        (ImageClassificationSchema::Cifar10Binary, "test") => {
            let path = dir.join(CIFAR10_TEST);
            if path.is_file() {
                vec![path]
            } else {
                Vec::new()
            }
        }
        (ImageClassificationSchema::Cifar100Binary, "train") => existing_file(&dir, CIFAR100_TRAIN),
        (ImageClassificationSchema::Cifar100Binary, "test") => existing_file(&dir, CIFAR100_TEST),
        (_, "val") => {
            return Err(TrainerError::Parse(
                "CIFAR has no validation split".to_string(),
            ));
        }
        _ => {
            return Err(TrainerError::Config(
                "CIFAR discovery was called for a different schema".to_string(),
            ));
        }
    };
    if files.is_empty() {
        return Err(TrainerError::Parse(format!(
            "CIFAR split '{split_name}' is missing under '{}'",
            dir.display()
        )));
    }
    let stride = match schema {
        ImageClassificationSchema::Cifar10Binary => CIFAR10_STRIDE,
        ImageClassificationSchema::Cifar100Binary => CIFAR100_STRIDE,
        _ => {
            return Err(TrainerError::Config(
                "CIFAR discovery was called for a different schema".to_string(),
            ));
        }
    };
    let packed = |record| match schema {
        ImageClassificationSchema::Cifar10Binary => PackedRecord::Cifar10 { record },
        _ => PackedRecord::Cifar100 { record },
    };
    let mut images = Vec::new();
    for path in files {
        let count = record_count(&path, stride)?;
        for record in 0..count {
            let label_index = read_label_index(&path, record, stride, schema)?;
            let label = label_name(schema, label_index)?;
            let class_id = class_id_for_label(&label, class_map)?;
            if class_id >= class_count {
                return Err(TrainerError::Parse(format!(
                    "class id {class_id} for '{label}' is outside class_count {class_count}"
                )));
            }
            images.push(ClassifiedImage {
                path: path.clone(),
                label,
                class_id,
                packed: packed(record),
            });
        }
    }
    Ok(images)
}

pub(super) fn load_cifar10_png(
    path: &Path,
    record: u32,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, TrainerError> {
    load_png(path, record, CIFAR10_STRIDE, 1, width, height)
}

pub(super) fn load_cifar100_png(
    path: &Path,
    record: u32,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, TrainerError> {
    load_png(path, record, CIFAR100_STRIDE, 2, width, height)
}

fn load_png(
    path: &Path,
    record: u32,
    stride: usize,
    pixel_offset: usize,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, TrainerError> {
    let record_bytes = read_record(path, record, stride)?;
    let pixels = record_bytes
        .get(pixel_offset..pixel_offset + CHANNEL_BYTES * 3)
        .ok_or_else(|| {
            TrainerError::Parse(format!(
                "CIFAR record {record} in '{}' is shorter than one image",
                path.display()
            ))
        })?;
    let mut rgb = Vec::with_capacity(CHANNEL_BYTES * 3);
    for index in 0..CHANNEL_BYTES {
        rgb.push(pixels[index]);
        rgb.push(pixels[CHANNEL_BYTES + index]);
        rgb.push(pixels[CHANNEL_BYTES * 2 + index]);
    }
    let image = image::RgbImage::from_raw(WIDTH, HEIGHT, rgb).ok_or_else(|| {
        TrainerError::Parse(format!(
            "cannot build a CIFAR frame from '{}'",
            path.display()
        ))
    })?;
    resized_rgb_png(image, width, height)
}

fn scan_cifar10(dir: &Path) -> Result<ImageClassificationScan, TrainerError> {
    let mut splits = Vec::new();
    let mut issues = Vec::new();
    if CIFAR10_TRAIN.iter().any(|name| dir.join(name).is_file()) {
        for name in CIFAR10_TRAIN {
            let path = dir.join(name);
            if path.is_file() {
                check_stride(&path, CIFAR10_STRIDE, &mut issues)?;
            }
        }
        splits.push("train".to_string());
    }
    let test = dir.join(CIFAR10_TEST);
    if test.is_file() {
        check_stride(&test, CIFAR10_STRIDE, &mut issues)?;
        splits.push("test".to_string());
    }
    Ok(scan_result(
        ImageClassificationSchema::Cifar10Binary,
        &CIFAR10_LABELS,
        splits,
        issues,
    ))
}

fn scan_cifar100(dir: &Path) -> Result<ImageClassificationScan, TrainerError> {
    let mut splits = Vec::new();
    let mut issues = Vec::new();
    for (name, split) in [(CIFAR100_TRAIN, "train"), (CIFAR100_TEST, "test")] {
        let path = dir.join(name);
        if path.is_file() {
            check_stride(&path, CIFAR100_STRIDE, &mut issues)?;
            splits.push(split.to_string());
        }
    }
    Ok(scan_result(
        ImageClassificationSchema::Cifar100Binary,
        &CIFAR100_FINE_LABELS,
        splits,
        issues,
    ))
}

fn scan_result(
    schema: ImageClassificationSchema,
    labels: &[&str],
    splits: Vec<String>,
    issues: Vec<String>,
) -> ImageClassificationScan {
    ImageClassificationScan {
        schema: Some(schema),
        classes: labels
            .iter()
            .enumerate()
            .map(|(index, label)| ClassVoxel {
                label: (*label).to_string(),
                x: index as u32,
                y: 0,
                z: 0,
            })
            .collect(),
        splits,
        split_stats: Vec::new(),
        image_width: Some(WIDTH),
        image_height: Some(HEIGHT),
        issues,
    }
}

fn directory_has_cifar(dir: &Path) -> Result<bool, TrainerError> {
    Ok(directory_has_cifar10(dir)? || directory_has_cifar100(dir)?)
}

fn directory_has_cifar10(dir: &Path) -> Result<bool, TrainerError> {
    Ok(CIFAR10_TRAIN.iter().any(|name| dir.join(name).is_file())
        || dir.join(CIFAR10_TEST).is_file())
}

fn directory_has_cifar100(dir: &Path) -> Result<bool, TrainerError> {
    let train = dir.join(CIFAR100_TRAIN);
    let test = dir.join(CIFAR100_TEST);
    if !train.is_file() && !test.is_file() {
        return Ok(false);
    }
    // A CIFAR-10 folder does not use these names. A stray train.bin must be the 3074-byte stride.
    Ok(file_matches_stride(&train, CIFAR100_STRIDE)?
        || file_matches_stride(&test, CIFAR100_STRIDE)?)
}

fn file_matches_stride(path: &Path, stride: usize) -> Result<bool, TrainerError> {
    if !path.is_file() {
        return Ok(false);
    }
    let length = path
        .metadata()
        .map_err(|error| TrainerError::Parse(format!("cannot stat '{}': {error}", path.display())))?
        .len() as usize;
    // `is_multiple_of` needs Rust 1.87. Workspace MSRV is 1.75.
    #[allow(clippy::manual_is_multiple_of)]
    Ok(length > 0 && length % stride == 0)
}

fn existing_file(dir: &Path, name: &str) -> Vec<PathBuf> {
    let path = dir.join(name);
    if path.is_file() {
        vec![path]
    } else {
        Vec::new()
    }
}

fn check_stride(path: &Path, stride: usize, issues: &mut Vec<String>) -> Result<(), TrainerError> {
    if !file_matches_stride(path, stride)? {
        issues.push(format!(
            "'{}' length is not a multiple of {stride} bytes",
            path.display()
        ));
    }
    Ok(())
}

fn record_count(path: &Path, stride: usize) -> Result<u32, TrainerError> {
    let length = path
        .metadata()
        .map_err(|error| TrainerError::Parse(format!("cannot stat '{}': {error}", path.display())))?
        .len();
    if length == 0 || length % stride as u64 != 0 {
        return Err(TrainerError::Parse(format!(
            "'{}' length is not a multiple of {stride} bytes",
            path.display()
        )));
    }
    u32::try_from(length / stride as u64).map_err(|_| {
        TrainerError::Parse(format!("'{}' has too many CIFAR records", path.display()))
    })
}

fn read_label_index(
    path: &Path,
    record: u32,
    stride: usize,
    schema: &ImageClassificationSchema,
) -> Result<u8, TrainerError> {
    let bytes = read_record(path, record, stride)?;
    let index = match schema {
        ImageClassificationSchema::Cifar100Binary => 1,
        _ => 0,
    };
    bytes.get(index).copied().ok_or_else(|| {
        TrainerError::Parse(format!(
            "CIFAR record {record} in '{}' has no label",
            path.display()
        ))
    })
}

fn label_name(schema: &ImageClassificationSchema, index: u8) -> Result<String, TrainerError> {
    let labels: &[&str] = match schema {
        ImageClassificationSchema::Cifar10Binary => &CIFAR10_LABELS,
        ImageClassificationSchema::Cifar100Binary => &CIFAR100_FINE_LABELS,
        _ => {
            return Err(TrainerError::Config(
                "CIFAR label lookup was called for a different schema".to_string(),
            ));
        }
    };
    labels
        .get(index as usize)
        .map(|label| (*label).to_string())
        .ok_or_else(|| {
            TrainerError::Parse(format!(
                "CIFAR label byte {index} is outside the class list"
            ))
        })
}

fn read_record(path: &Path, record: u32, stride: usize) -> Result<Vec<u8>, TrainerError> {
    let mut file = File::open(path).map_err(|error| {
        TrainerError::Parse(format!("cannot read '{}': {error}", path.display()))
    })?;
    let offset = u64::from(record) * stride as u64;
    file.seek(SeekFrom::Start(offset)).map_err(|error| {
        TrainerError::Parse(format!(
            "cannot seek to CIFAR record {record} in '{}': {error}",
            path.display()
        ))
    })?;
    let mut bytes = vec![0_u8; stride];
    file.read_exact(&mut bytes).map_err(|error| {
        TrainerError::Parse(format!(
            "cannot read CIFAR record {record} from '{}': {error}",
            path.display()
        ))
    })?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cifar100_fine_list_has_one_hundred_names() {
        assert_eq!(CIFAR100_FINE_LABELS.len(), 100);
        assert_eq!(CIFAR10_LABELS.len(), 10);
    }

    #[test]
    fn cifar10_batch_round_trip() {
        let root = tempfile::tempdir().expect("temp");
        let path = root.path().join("data_batch_1.bin");
        let mut record = vec![3_u8];
        // `repeat_n` needs Rust 1.82. Workspace MSRV is 1.75.
        #[allow(clippy::manual_repeat_n)]
        {
            record.extend(std::iter::repeat(9_u8).take(1024));
            record.extend(std::iter::repeat(8_u8).take(1024));
            record.extend(std::iter::repeat(7_u8).take(1024));
        }
        std::fs::write(&path, &record).expect("write");
        let scan = scan_if_present(root.path()).expect("scan").expect("cifar");
        assert_eq!(scan.schema, Some(ImageClassificationSchema::Cifar10Binary));
        assert_eq!(scan.splits, vec!["train".to_string()]);
        assert_eq!(scan.classes[3].label, "cat");
        assert_eq!(scan.image_width, Some(32));
        let class_map = scan.classes.clone();
        let images = discover_split(
            root.path(),
            &ImageClassificationSchema::Cifar10Binary,
            &Split::Train,
            &class_map,
            10,
        )
        .expect("discover");
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].label, "cat");
        assert_eq!(images[0].class_id, 3);
        let png = load_cifar10_png(&path, 0, 32, 32).expect("png");
        assert!(png.starts_with(&[137, 80, 78, 71]));
    }
}
