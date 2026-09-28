//! SVHN cropped digits, MAT v5 numeric arrays.
//!
//! `train_32x32.mat` and `test_32x32.mat` hold `X` as 32×32×3×N and `y` as labels 1 through 10.
//! Label 10 is digit 0. MATLAB stores `X` column-major. Compressed MAT arrays and MATLAB itself
//! are not used: the pixel payload must be an uncompressed numeric element so one frame can be read
//! by seeking.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::contracts::Split;
use crate::error::TrainerError;

use super::{
    class_id_for_label, locate_single_child, resized_rgb_png, split_dir_name, ClassVoxel,
    ClassifiedImage, ImageClassificationScan, ImageClassificationSchema, PackedRecord,
};

const WIDTH: usize = 32;
const HEIGHT: usize = 32;
const PIXELS: usize = WIDTH * HEIGHT;
const TRAIN_FILE: &str = "train_32x32.mat";
const TEST_FILE: &str = "test_32x32.mat";
const EXTRA_FILE: &str = "extra_32x32.mat";

const MI_INT8: u32 = 1;
const MI_UINT8: u32 = 2;
const MI_INT16: u32 = 3;
const MI_UINT16: u32 = 4;
const MI_INT32: u32 = 5;
const MI_UINT32: u32 = 6;
const MI_SINGLE: u32 = 7;
const MI_DOUBLE: u32 = 9;
const MI_INT64: u32 = 12;
const MI_UINT64: u32 = 13;
const MI_MATRIX: u32 = 14;
const MI_COMPRESSED: u32 = 15;

const MX_DOUBLE: u8 = 6;
const MX_SINGLE: u8 = 7;
const MX_INT8: u8 = 8;
const MX_UINT8: u8 = 9;
const MX_INT16: u8 = 10;
const MX_UINT16: u8 = 11;
const MX_INT32: u8 = 12;
const MX_UINT32: u8 = 13;

struct NumericArray {
    name: String,
    dims: Vec<usize>,
    /// Byte offset of the raw numeric payload inside the MAT file.
    data_offset: u64,
    data_type: u32,
    little_endian: bool,
}

pub(super) fn scan_if_present(
    root: &Path,
) -> Result<Option<ImageClassificationScan>, TrainerError> {
    let Some(dir) = locate_single_child(root, directory_has_svhn, "SVHN")? else {
        if root.join(EXTRA_FILE).is_file() {
            return Err(TrainerError::Parse(format!(
                "'{EXTRA_FILE}' is not a train or test split. Use {TRAIN_FILE} and {TEST_FILE}."
            )));
        }
        return Ok(None);
    };
    let mut splits = Vec::new();
    let mut issues = Vec::new();
    for (name, split) in [(TRAIN_FILE, "train"), (TEST_FILE, "test")] {
        let path = dir.join(name);
        if !path.is_file() {
            continue;
        }
        let arrays = read_header_arrays(&path)?;
        let x = require_array(&arrays, "X", &path)?;
        let y = require_array(&arrays, "y", &path)?;
        let count = svhn_count(x)?;
        let labels = svhn_count(y)?;
        if count != labels {
            issues.push(format!(
                "'{}' X has {count} frames and y has {labels} labels",
                path.display()
            ));
        }
        splits.push(split.to_string());
    }
    let classes = (0..10)
        .map(|index| ClassVoxel {
            label: index.to_string(),
            x: index,
            y: 0,
            z: 0,
        })
        .collect();
    Ok(Some(ImageClassificationScan {
        schema: Some(ImageClassificationSchema::SvhnMat),
        classes,
        splits,
        split_stats: Vec::new(),
        image_width: Some(WIDTH as u32),
        image_height: Some(HEIGHT as u32),
        issues,
    }))
}

pub(super) fn discover_split(
    root: &Path,
    split: &Split,
    class_map: &[ClassVoxel],
    class_count: u32,
) -> Result<Vec<ClassifiedImage>, TrainerError> {
    let dir = locate_single_child(root, directory_has_svhn, "SVHN")?.ok_or_else(|| {
        TrainerError::Parse(format!(
            "SVHN MAT files were not found under '{}'",
            root.display()
        ))
    })?;
    let split_name = split_dir_name(split)?;
    let file_name = match split_name {
        "train" => TRAIN_FILE,
        "test" => TEST_FILE,
        "val" => {
            return Err(TrainerError::Parse(
                "SVHN has no validation split".to_string(),
            ));
        }
        other => {
            return Err(TrainerError::Parse(format!("SVHN has no split '{other}'")));
        }
    };
    let path = dir.join(file_name);
    if !path.is_file() {
        return Err(TrainerError::Parse(format!(
            "SVHN split '{split_name}' is missing under '{}'",
            dir.display()
        )));
    }
    let arrays = read_header_arrays(&path)?;
    let x = require_array(&arrays, "X", &path)?;
    let y = require_array(&arrays, "y", &path)?;
    let count = svhn_count(x)?;
    if svhn_count(y)? != count {
        return Err(TrainerError::Parse(format!(
            "'{}' X and y lengths differ",
            path.display()
        )));
    }
    let labels = read_labels(&path, y, count)?;
    let mut images = Vec::with_capacity(count);
    for (record, value) in labels.into_iter().enumerate() {
        let label = digit_label(value)
            .map_err(|message| TrainerError::Parse(format!("{message} in '{}'", path.display())))?;
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
            packed: PackedRecord::Svhn {
                record: record as u32,
                data_offset: x.data_offset,
            },
        });
    }
    Ok(images)
}

pub(super) fn load_svhn_png(
    path: &Path,
    record: u32,
    data_offset: u64,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, TrainerError> {
    let mut file = File::open(path).map_err(|error| {
        TrainerError::Parse(format!("cannot read '{}': {error}", path.display()))
    })?;
    let frame_bytes = PIXELS * 3;
    let start = data_offset + u64::from(record) * frame_bytes as u64;
    file.seek(SeekFrom::Start(start)).map_err(|error| {
        TrainerError::Parse(format!(
            "cannot seek to SVHN frame {record} in '{}': {error}",
            path.display()
        ))
    })?;
    let mut raw = vec![0_u8; frame_bytes];
    file.read_exact(&mut raw).map_err(|error| {
        TrainerError::Parse(format!(
            "cannot read SVHN frame {record} from '{}': {error}",
            path.display()
        ))
    })?;
    // MATLAB dimension order is row, column, channel, sample. Column-major.
    let mut rgb = Vec::with_capacity(frame_bytes);
    for row in 0..HEIGHT {
        for col in 0..WIDTH {
            for channel in 0..3 {
                let index = row + HEIGHT * col + PIXELS * channel;
                rgb.push(raw[index]);
            }
        }
    }
    let image = image::RgbImage::from_raw(WIDTH as u32, HEIGHT as u32, rgb).ok_or_else(|| {
        TrainerError::Parse(format!(
            "cannot build an SVHN frame from '{}'",
            path.display()
        ))
    })?;
    resized_rgb_png(image, width, height)
}

fn directory_has_svhn(dir: &Path) -> Result<bool, TrainerError> {
    Ok(dir.join(TRAIN_FILE).is_file() || dir.join(TEST_FILE).is_file())
}

fn digit_label(value: f64) -> Result<String, String> {
    if value.fract() != 0.0 {
        return Err(format!("SVHN label {value} is not an integer"));
    }
    let index = value as i32;
    match index {
        10 => Ok("0".to_string()),
        1..=9 => Ok(index.to_string()),
        _ => Err(format!("SVHN label {value} is outside 1 through 10")),
    }
}

fn svhn_count(array: &NumericArray) -> Result<usize, TrainerError> {
    let product: usize = array.dims.iter().copied().product();
    if array.dims.len() == 4
        && array.dims[0] == HEIGHT
        && array.dims[1] == WIDTH
        && array.dims[2] == 3
    {
        return Ok(array.dims[3]);
    }
    if array.dims.len() == 2 && (array.dims[0] == 1 || array.dims[1] == 1) {
        return Ok(product);
    }
    Err(TrainerError::Parse(format!(
        "SVHN array '{}' has dimensions {:?}, expected 32x32x3xN or a label vector",
        array.name, array.dims
    )))
}

fn require_array<'a>(
    arrays: &'a [NumericArray],
    name: &str,
    path: &Path,
) -> Result<&'a NumericArray, TrainerError> {
    arrays
        .iter()
        .find(|array| array.name == name)
        .ok_or_else(|| {
            TrainerError::Parse(format!(
                "'{}' has no numeric array '{name}'",
                path.display()
            ))
        })
}

fn read_labels(path: &Path, array: &NumericArray, count: usize) -> Result<Vec<f64>, TrainerError> {
    let mut file = File::open(path).map_err(|error| {
        TrainerError::Parse(format!("cannot read '{}': {error}", path.display()))
    })?;
    file.seek(SeekFrom::Start(array.data_offset))
        .map_err(|error| {
            TrainerError::Parse(format!(
                "cannot seek labels in '{}': {error}",
                path.display()
            ))
        })?;
    let width = numeric_width(array.data_type)?;
    let mut raw = vec![0_u8; width * count];
    file.read_exact(&mut raw).map_err(|error| {
        TrainerError::Parse(format!(
            "cannot read labels from '{}': {error}",
            path.display()
        ))
    })?;
    let mut values = Vec::with_capacity(count);
    for chunk in raw.chunks(width) {
        values.push(decode_number(array.data_type, chunk, array.little_endian)?);
    }
    Ok(values)
}

fn read_header_arrays(path: &Path) -> Result<Vec<NumericArray>, TrainerError> {
    let bytes = std::fs::read(path).map_err(|error| {
        TrainerError::Parse(format!("cannot read '{}': {error}", path.display()))
    })?;
    if bytes.len() < 128 {
        return Err(TrainerError::Parse(format!(
            "'{}' is shorter than a MAT v5 header",
            path.display()
        )));
    }
    let little = match &bytes[126..128] {
        b"IM" => true,
        b"MI" => false,
        other => {
            return Err(TrainerError::Parse(format!(
                "'{}' is not a MAT v5 file (endian marker {other:?})",
                path.display()
            )));
        }
    };
    parse_arrays(&bytes, 128, little, path)
}

fn parse_arrays(
    bytes: &[u8],
    mut offset: usize,
    little: bool,
    path: &Path,
) -> Result<Vec<NumericArray>, TrainerError> {
    let mut arrays = Vec::new();
    while offset + 8 <= bytes.len() {
        let (kind, payload_start, payload_end, next) = read_tag(bytes, offset, little, path)?;
        if kind == MI_COMPRESSED {
            return Err(TrainerError::Parse(format!(
                "'{}' stores a compressed MAT array. SVHN X and y must be uncompressed so frames can be read by seeking.",
                path.display()
            )));
        }
        if kind == MI_MATRIX {
            arrays.push(parse_matrix(
                bytes,
                payload_start,
                payload_end,
                little,
                path,
            )?);
        }
        offset = next;
    }
    Ok(arrays)
}

fn parse_matrix(
    bytes: &[u8],
    start: usize,
    _end: usize,
    little: bool,
    path: &Path,
) -> Result<NumericArray, TrainerError> {
    let mut offset = start;
    let (_, flags_start, flags_end, next) = read_tag(bytes, offset, little, path)?;
    offset = next;
    if flags_end - flags_start < 8 {
        return Err(TrainerError::Parse(format!(
            "'{}' matrix flags are shorter than 8 bytes",
            path.display()
        )));
    }
    let class = if little {
        bytes[flags_start]
    } else {
        bytes[flags_start + 3]
    };
    let (_, dim_start, dim_end, next) = read_tag(bytes, offset, little, path)?;
    offset = next;
    let mut dims = Vec::new();
    let mut dim_offset = dim_start;
    while dim_offset + 4 <= dim_end {
        let value = read_i32(&bytes[dim_offset..dim_offset + 4], little);
        if value < 0 {
            return Err(TrainerError::Parse(format!(
                "'{}' has a negative matrix dimension",
                path.display()
            )));
        }
        dims.push(value as usize);
        dim_offset += 4;
    }
    let (_, name_start, name_end, next) = read_tag(bytes, offset, little, path)?;
    offset = next;
    let name = String::from_utf8_lossy(&bytes[name_start..name_end])
        .trim_end_matches('\0')
        .to_string();
    let (data_type, data_start, data_end, _) = read_tag(bytes, offset, little, path)?;
    if !matches!(
        class,
        MX_DOUBLE | MX_SINGLE | MX_INT8 | MX_UINT8 | MX_INT16 | MX_UINT16 | MX_INT32 | MX_UINT32
    ) {
        return Err(TrainerError::Parse(format!(
            "'{}' array '{name}' uses MATLAB class {class}, which this reader does not load",
            path.display()
        )));
    }
    let _ = data_end;
    Ok(NumericArray {
        name,
        dims,
        data_offset: data_start as u64,
        data_type,
        little_endian: little,
    })
}

fn read_tag(
    bytes: &[u8],
    offset: usize,
    little: bool,
    path: &Path,
) -> Result<(u32, usize, usize, usize), TrainerError> {
    if offset + 4 > bytes.len() {
        return Err(TrainerError::Parse(format!(
            "'{}' ended inside a MAT tag",
            path.display()
        )));
    }
    let first = read_u32(&bytes[offset..offset + 4], little);
    if first > 0xFFFF {
        let kind = first & 0xFFFF;
        let size = (first >> 16) as usize;
        let data_start = offset + 4;
        let data_end = data_start + size;
        if data_end > bytes.len() || offset + 8 > bytes.len() {
            return Err(TrainerError::Parse(format!(
                "'{}' small MAT element runs past the file",
                path.display()
            )));
        }
        return Ok((kind, data_start, data_end, offset + 8));
    }
    if offset + 8 > bytes.len() {
        return Err(TrainerError::Parse(format!(
            "'{}' ended inside a MAT tag",
            path.display()
        )));
    }
    let size = read_u32(&bytes[offset + 4..offset + 8], little) as usize;
    let data_start = offset + 8;
    let data_end = data_start + size;
    let padded = (size + 7) & !7;
    let next = data_start + padded;
    if data_end > bytes.len() || next > bytes.len() {
        return Err(TrainerError::Parse(format!(
            "'{}' MAT element runs past the file",
            path.display()
        )));
    }
    Ok((first, data_start, data_end, next))
}

fn numeric_width(data_type: u32) -> Result<usize, TrainerError> {
    match data_type {
        MI_INT8 | MI_UINT8 => Ok(1),
        MI_INT16 | MI_UINT16 => Ok(2),
        MI_INT32 | MI_UINT32 | MI_SINGLE => Ok(4),
        MI_DOUBLE | MI_INT64 | MI_UINT64 => Ok(8),
        other => Err(TrainerError::Parse(format!(
            "MAT numeric type {other} is not a supported SVHN array"
        ))),
    }
}

fn decode_number(data_type: u32, bytes: &[u8], little: bool) -> Result<f64, TrainerError> {
    let value = match data_type {
        MI_UINT8 => f64::from(bytes[0]),
        MI_INT8 => f64::from(bytes[0] as i8),
        MI_UINT16 => f64::from(read_u16(bytes, little)),
        MI_INT16 => f64::from(read_u16(bytes, little) as i16),
        MI_UINT32 => f64::from(read_u32(bytes, little)),
        MI_INT32 => f64::from(read_u32(bytes, little) as i32),
        MI_SINGLE => f64::from(f32::from_bits(read_u32(bytes, little))),
        MI_DOUBLE => f64::from_bits(read_u64(bytes, little)),
        other => {
            return Err(TrainerError::Parse(format!(
                "MAT numeric type {other} is not a supported SVHN label"
            )));
        }
    };
    Ok(value)
}

fn read_u16(bytes: &[u8], little: bool) -> u16 {
    if little {
        u16::from_le_bytes([bytes[0], bytes[1]])
    } else {
        u16::from_be_bytes([bytes[0], bytes[1]])
    }
}

fn read_u32(bytes: &[u8], little: bool) -> u32 {
    if little {
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    } else {
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}

fn read_u64(bytes: &[u8], little: bool) -> u64 {
    if little {
        u64::from_le_bytes(bytes[..8].try_into().expect("8 bytes"))
    } else {
        u64::from_be_bytes(bytes[..8].try_into().expect("8 bytes"))
    }
}

fn read_i32(bytes: &[u8], little: bool) -> i32 {
    read_u32(bytes, little) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_u32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    fn tag(kind: u32, payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        push_u32(&mut out, kind);
        push_u32(&mut out, payload.len() as u32);
        out.extend_from_slice(payload);
        while out.len() % 8 != 0 {
            out.push(0);
        }
        out
    }

    /// One 32×32×3×1 uint8 frame and a double label of 10.
    fn write_one_frame(path: &Path, label: f64) {
        let mut header = vec![b' '; 116];
        header.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]);
        header.extend_from_slice(&0x0100_u16.to_le_bytes());
        header.extend_from_slice(b"IM");
        let mut x_payload = vec![0_u8; PIXELS * 3];
        x_payload[0] = 11;
        let mut body = Vec::new();
        body.extend(tag(MI_UINT32, &[MX_UINT8, 0, 0, 0, 0, 0, 0, 0]));
        let dims = {
            let mut bytes = Vec::new();
            for value in [32_i32, 32, 3, 1] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            bytes
        };
        body.extend(tag(MI_INT32, &dims));
        body.extend(tag(MI_INT8, b"X"));
        body.extend(tag(MI_UINT8, &x_payload));
        let matrix = tag(MI_MATRIX, &body);
        let mut y_body = Vec::new();
        y_body.extend(tag(MI_UINT32, &[MX_DOUBLE, 0, 0, 0, 0, 0, 0, 0]));
        let y_dims = {
            let mut bytes = Vec::new();
            for value in [1_i32, 1] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            bytes
        };
        y_body.extend(tag(MI_INT32, &y_dims));
        y_body.extend(tag(MI_INT8, b"y"));
        y_body.extend(tag(MI_DOUBLE, &label.to_le_bytes()));
        let y_matrix = tag(MI_MATRIX, &y_body);
        let mut file = header;
        file.extend(matrix);
        file.extend(y_matrix);
        std::fs::write(path, file).expect("mat");
    }

    #[test]
    fn label_ten_is_digit_zero() {
        let root = tempfile::tempdir().expect("temp");
        let path = root.path().join(TRAIN_FILE);
        write_one_frame(&path, 10.0);
        let scan = scan_if_present(root.path()).expect("scan").expect("svhn");
        assert_eq!(scan.schema, Some(ImageClassificationSchema::SvhnMat));
        assert_eq!(scan.splits, vec!["train".to_string()]);
        assert!(scan.issues.is_empty(), "{:?}", scan.issues);
        let images =
            discover_split(root.path(), &Split::Train, &scan.classes, 10).expect("discover");
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].label, "0");
        assert_eq!(images[0].class_id, 0);
        let PackedRecord::Svhn {
            record,
            data_offset,
        } = images[0].packed
        else {
            panic!("svhn record");
        };
        let png = load_svhn_png(&path, record, data_offset, 32, 32).expect("png");
        assert!(png.starts_with(&[137, 80, 78, 71]));
    }
}
