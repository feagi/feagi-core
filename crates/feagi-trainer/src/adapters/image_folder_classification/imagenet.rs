//! ILSVRC2012 classification layout.
//!
//! Prepared copies already use class folders (`train/<wnid>`, `val/<wnid>`) and the folder reader.
//! This module is the official validation tar: flat `val/*.JPEG` plus
//! `ILSVRC2012_validation_ground_truth.txt`. Each line is an ILSVRC2012_ID, 1 through 1000,
//! in sorted filename order. The id is not the alphabetical WordNet order.

use std::path::{Path, PathBuf};

use crate::contracts::Split;
use crate::error::TrainerError;

use super::ilsvrc2012::ILSVRC2012_WNIDS;
use super::{
    class_folder_images, class_id_for_label, file_name_utf8, is_image_file, list_dir,
    split_dir_name, ClassVoxel, ClassifiedImage, ImageClassificationScan,
    ImageClassificationSchema, PackedRecord,
};

const GROUND_TRUTH_NAME: &str = "ILSVRC2012_validation_ground_truth.txt";
const DEVKIT_GROUND_TRUTH: &str =
    "ILSVRC2012_devkit_t12/data/ILSVRC2012_validation_ground_truth.txt";

pub(super) fn scan_if_flat_val(
    root: &Path,
) -> Result<Option<ImageClassificationScan>, TrainerError> {
    let train = root.join("train");
    let val = root.join("val");
    if !train.is_dir() || !val.is_dir() || !train_is_wnid_folders(&train)? || !val_is_flat(&val)? {
        return Ok(None);
    }
    let ground_truth = ground_truth_file(root)?;
    let ids = read_ground_truth(&ground_truth)?;
    let val_images = flat_images(&val)?;
    let mut issues = Vec::new();
    if ids.len() != val_images.len() {
        issues.push(format!(
            "validation ground truth has {} lines and '{}' has {} images",
            ids.len(),
            val.display(),
            val_images.len()
        ));
    }
    let classes = wnid_classes(&train)?;
    let known: std::collections::BTreeSet<&str> =
        classes.iter().map(|row| row.label.as_str()).collect();
    for id in &ids {
        let wnid = wnid_for_id(*id)?;
        if !known.contains(wnid) {
            issues.push(format!(
                "validation id {id} is WordNet {wnid}, which is not a train folder"
            ));
            break;
        }
    }
    let (image_width, image_height) = first_image_size(val_images.first());
    Ok(Some(ImageClassificationScan {
        schema: Some(ImageClassificationSchema::ImageNet),
        classes,
        splits: vec!["train".to_string(), "val".to_string()],
        split_stats: Vec::new(),
        image_width,
        image_height,
        issues,
    }))
}

pub(super) fn discover_split(
    root: &Path,
    split: &Split,
    class_map: &[ClassVoxel],
    class_count: u32,
) -> Result<Vec<ClassifiedImage>, TrainerError> {
    let split_name = split_dir_name(split)?;
    match split_name {
        "train" => {
            let split_dir = root.join("train");
            let entries = list_dir(&split_dir)?;
            let images = class_folder_images(&split_dir, &entries, class_map)?;
            for image in &images {
                if image.class_id >= class_count {
                    return Err(TrainerError::Parse(format!(
                        "class id {} for '{}' is outside class_count {class_count}",
                        image.class_id, image.label
                    )));
                }
            }
            Ok(images)
        }
        "val" => discover_flat_val(root, class_map, class_count),
        "test" => Err(TrainerError::Parse(
            "ILSVRC2012 has no labeled test split".to_string(),
        )),
        other => Err(TrainerError::Parse(format!(
            "ImageNet has no split '{other}'"
        ))),
    }
}

fn discover_flat_val(
    root: &Path,
    class_map: &[ClassVoxel],
    class_count: u32,
) -> Result<Vec<ClassifiedImage>, TrainerError> {
    let val = root.join("val");
    let ids = read_ground_truth(&ground_truth_file(root)?)?;
    let images_on_disk = flat_images(&val)?;
    if ids.len() != images_on_disk.len() {
        return Err(TrainerError::Parse(format!(
            "validation ground truth has {} lines and '{}' has {} images",
            ids.len(),
            val.display(),
            images_on_disk.len()
        )));
    }
    let mut images = Vec::with_capacity(images_on_disk.len());
    for (path, id) in images_on_disk.into_iter().zip(ids) {
        let label = wnid_for_id(id)?.to_string();
        let class_id = class_id_for_label(&label, class_map)?;
        if class_id >= class_count {
            return Err(TrainerError::Parse(format!(
                "class id {class_id} for '{label}' is outside class_count {class_count}"
            )));
        }
        images.push(ClassifiedImage {
            path,
            label,
            class_id,
            packed: PackedRecord::File,
        });
    }
    Ok(images)
}

fn ground_truth_file(root: &Path) -> Result<PathBuf, TrainerError> {
    let beside = root.join(GROUND_TRUTH_NAME);
    let devkit = root.join(DEVKIT_GROUND_TRUTH);
    match (beside.is_file(), devkit.is_file()) {
        (true, false) => Ok(beside),
        (false, true) => Ok(devkit),
        (true, true) => {
            let left = std::fs::read(&beside).map_err(|error| {
                TrainerError::Parse(format!("cannot read '{}': {error}", beside.display()))
            })?;
            let right = std::fs::read(&devkit).map_err(|error| {
                TrainerError::Parse(format!("cannot read '{}': {error}", devkit.display()))
            })?;
            if left != right {
                return Err(TrainerError::Parse(format!(
                    "'{}' and '{}' are both present and they differ",
                    beside.display(),
                    devkit.display()
                )));
            }
            Ok(beside)
        }
        (false, false) => Err(TrainerError::Parse(format!(
            "flat ImageNet validation needs {GROUND_TRUTH_NAME} in '{}' or in {DEVKIT_GROUND_TRUTH}",
            root.display()
        ))),
    }
}

fn read_ground_truth(path: &Path) -> Result<Vec<u32>, TrainerError> {
    let text = std::fs::read_to_string(path).map_err(|error| {
        TrainerError::Parse(format!("cannot read '{}': {error}", path.display()))
    })?;
    let mut ids = Vec::new();
    for (line_index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let id: u32 = trimmed.parse().map_err(|_| {
            TrainerError::Parse(format!(
                "'{}' line {} is not an ILSVRC2012_ID",
                path.display(),
                line_index + 1
            ))
        })?;
        if id == 0 || id > ILSVRC2012_WNIDS.len() as u32 {
            return Err(TrainerError::Parse(format!(
                "'{}' line {} id {id} is outside 1 through {}",
                path.display(),
                line_index + 1,
                ILSVRC2012_WNIDS.len()
            )));
        }
        ids.push(id);
    }
    if ids.is_empty() {
        return Err(TrainerError::Parse(format!(
            "'{}' has no validation ids",
            path.display()
        )));
    }
    Ok(ids)
}

fn wnid_for_id(id: u32) -> Result<&'static str, TrainerError> {
    ILSVRC2012_WNIDS
        .get(id as usize - 1)
        .copied()
        .ok_or_else(|| TrainerError::Parse(format!("ILSVRC2012_ID {id} is outside 1 through 1000")))
}

fn train_is_wnid_folders(train: &Path) -> Result<bool, TrainerError> {
    let entries = list_dir(train)?;
    let mut folders = 0_usize;
    for path in entries {
        if !path.is_dir() {
            return Ok(false);
        }
        let name = file_name_utf8(&path).map_err(TrainerError::Parse)?;
        if !is_wnid(&name) {
            return Ok(false);
        }
        folders += 1;
    }
    Ok(folders > 0)
}

fn val_is_flat(val: &Path) -> Result<bool, TrainerError> {
    let entries = list_dir(val)?;
    let mut images = 0_usize;
    for path in &entries {
        if path.is_dir() {
            return Ok(false);
        }
        if is_image_file(path) {
            images += 1;
        }
    }
    Ok(images > 0)
}

fn wnid_classes(train: &Path) -> Result<Vec<ClassVoxel>, TrainerError> {
    let mut labels = Vec::new();
    for path in list_dir(train)? {
        if path.is_dir() {
            labels.push(file_name_utf8(&path).map_err(TrainerError::Parse)?);
        }
    }
    labels.sort();
    Ok(labels
        .into_iter()
        .enumerate()
        .map(|(index, label)| ClassVoxel {
            label,
            x: index as u32,
            y: 0,
            z: 0,
        })
        .collect())
}

fn flat_images(val: &Path) -> Result<Vec<PathBuf>, TrainerError> {
    let mut images = Vec::new();
    for path in list_dir(val)? {
        if is_image_file(&path) {
            images.push(path);
        }
    }
    images.sort();
    Ok(images)
}

fn first_image_size(path: Option<&PathBuf>) -> (Option<u32>, Option<u32>) {
    let Some(path) = path else {
        return (None, None);
    };
    match image::image_dimensions(path) {
        Ok((width, height)) => (Some(width), Some(height)),
        Err(_) => (None, None),
    }
}

fn is_wnid(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next() == Some('n') && name.len() == 9 && chars.all(|ch| ch.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_one_is_kit_fox_wnid() {
        assert_eq!(ILSVRC2012_WNIDS[0], "n02119789");
        assert_eq!(ILSVRC2012_WNIDS.len(), 1000);
    }

    #[test]
    fn flat_val_uses_ground_truth_id() {
        let root = tempfile::tempdir().expect("temp");
        let train = root.path().join("train").join("n02119789");
        std::fs::create_dir_all(&train).expect("train");
        let image = image::RgbImage::from_pixel(2, 2, image::Rgb([1, 2, 3]));
        image.save(train.join("a.jpg")).expect("train image");
        let val = root.path().join("val");
        std::fs::create_dir_all(&val).expect("val");
        image
            .save(val.join("ILSVRC2012_val_00000001.jpg"))
            .expect("val image");
        std::fs::write(root.path().join(GROUND_TRUTH_NAME), "1\n").expect("gt");
        let scan = scan_if_flat_val(root.path())
            .expect("scan")
            .expect("imagenet");
        assert_eq!(scan.schema, Some(ImageClassificationSchema::ImageNet));
        assert_eq!(scan.classes.len(), 1);
        assert_eq!(scan.classes[0].label, "n02119789");
        assert!(scan.issues.is_empty(), "{:?}", scan.issues);
        let images = discover_split(root.path(), &Split::Val, &scan.classes, 1).expect("val");
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].label, "n02119789");
        assert_eq!(images[0].class_id, 0);
    }
}
