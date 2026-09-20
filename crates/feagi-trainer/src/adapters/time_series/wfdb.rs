//! WFDB format codec (header, format 16/212 signal files, MIT annotations).
//!
//! This is a *format* reader, not a MIT-BIH-specific adapter. Any WFDB archive whose
//! signals and annotations are fully mapped by [`TimeSeriesPackageConfig`] can be ingested.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::adapters::time_series::config::{TimeSeriesPackageConfig, WfdbChannelMap};
use crate::adapters::time_series::corpus::{AnalogEpisode, AnalogEvent};
use crate::error::TrainerError;

/// Loads every WFDB record in `root` that has a `.hea` file.
pub fn load_wfdb_directory(
    root: &Path,
    config: &TimeSeriesPackageConfig,
) -> Result<Vec<AnalogEpisode>, TrainerError> {
    let suffix = config.annotation_suffix.as_deref().ok_or_else(|| {
        TrainerError::Config("wfdb ingest requires annotation_suffix".to_string())
    })?;
    let channel_map = config
        .channel_map
        .as_ref()
        .ok_or_else(|| TrainerError::Config("wfdb ingest requires channel_map".to_string()))?;

    let mut headers: Vec<PathBuf> = fs::read_dir(root)
        .map_err(|e| TrainerError::Io(format!("cannot read '{}': {e}", root.display())))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("hea"))
        })
        .collect();
    headers.sort();
    if headers.is_empty() {
        return Err(TrainerError::Parse(format!(
            "no .hea records under '{}'",
            root.display()
        )));
    }

    let mut episodes = Vec::with_capacity(headers.len());
    for header_path in headers {
        let stem = header_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                TrainerError::Parse(format!(
                    "WFDB header '{}' has no utf-8 stem",
                    header_path.display()
                ))
            })?;
        episodes.push(load_wfdb_record(
            header_path.parent().unwrap_or(root),
            stem,
            suffix,
            channel_map,
        )?);
    }
    Ok(episodes)
}

fn load_wfdb_record(
    dir: &Path,
    record: &str,
    annotation_suffix: &str,
    channel_map: &[WfdbChannelMap],
) -> Result<AnalogEpisode, TrainerError> {
    let header = parse_header(
        &fs::read_to_string(dir.join(format!("{record}.hea")))
            .map_err(|e| TrainerError::Io(format!("cannot read {record}.hea: {e}")))?,
    )?;
    if header.signals.len() != channel_map.len() {
        return Err(TrainerError::Parse(format!(
            "record '{record}': header has {} signal(s) but channel_map has {}",
            header.signals.len(),
            channel_map.len()
        )));
    }
    let mut expected: Vec<usize> = (0..header.signals.len()).collect();
    let mut mapped: Vec<usize> = channel_map.iter().map(|m| m.source_index).collect();
    expected.sort_unstable();
    mapped.sort_unstable();
    if expected != mapped {
        return Err(TrainerError::Parse(format!(
            "record '{record}': channel_map must list every signal index exactly once"
        )));
    }

    let dat_name = &header.signals[0].file_name;
    for signal in &header.signals {
        if signal.file_name != *dat_name {
            return Err(TrainerError::Unsupported(format!(
                "record '{record}': split signal files are not supported"
            )));
        }
        if signal.format != header.signals[0].format {
            return Err(TrainerError::Parse(format!(
                "record '{record}': mixed signal formats are not supported"
            )));
        }
    }

    let raw = fs::read(dir.join(dat_name))
        .map_err(|e| TrainerError::Io(format!("cannot read '{dat_name}': {e}")))?;
    let adu = decode_signal_file(
        &raw,
        header.signals[0].format,
        header.sample_count,
        header.signals.len(),
    )?;
    if adu.len() != header.sample_count * header.signals.len() {
        return Err(TrainerError::Parse(format!(
            "record '{record}': decoded {} values, expected {}",
            adu.len(),
            header.sample_count * header.signals.len()
        )));
    }

    let mut streams = BTreeMap::new();
    for mapping in channel_map {
        let spec = &header.signals[mapping.source_index];
        if spec.gain == 0.0 {
            return Err(TrainerError::Parse(format!(
                "record '{record}': signal {} has gain 0",
                mapping.source_index
            )));
        }
        let mut physical = Vec::with_capacity(header.sample_count);
        for sample_i in 0..header.sample_count {
            let adu_value = adu[sample_i * header.signals.len() + mapping.source_index];
            physical.push((f64::from(adu_value) - spec.baseline) / spec.gain);
        }
        let as_f32: Vec<f32> = physical.iter().map(|v| *v as f32).collect();
        streams.insert(mapping.stream_id.clone(), as_f32);
    }

    let atr_path = dir.join(format!("{record}.{annotation_suffix}"));
    let atr_bytes = fs::read(&atr_path)
        .map_err(|e| TrainerError::Io(format!("cannot read '{}': {e}", atr_path.display())))?;
    let events = read_mit_annotations(&atr_bytes).map_err(|e| match e {
        TrainerError::Parse(message) => {
            TrainerError::Parse(format!("record '{record}': {message}"))
        }
        other => other,
    })?;

    Ok(AnalogEpisode {
        episode_id: record.to_string(),
        sample_rate_hz: header.sampling_frequency,
        streams,
        events,
    })
}

#[derive(Debug, Clone)]
struct WfdbSignalSpec {
    file_name: String,
    format: u32,
    gain: f64,
    baseline: f64,
}

#[derive(Debug, Clone)]
struct WfdbHeader {
    sampling_frequency: f64,
    sample_count: usize,
    signals: Vec<WfdbSignalSpec>,
}

fn parse_header(text: &str) -> Result<WfdbHeader, TrainerError> {
    let mut lines = text.lines().filter(|line| !line.starts_with('#'));
    let record_line = lines
        .next()
        .ok_or_else(|| TrainerError::Parse("WFDB header is empty".to_string()))?;
    let rec_fields: Vec<&str> = record_line.split_whitespace().collect();
    if rec_fields.len() < 3 {
        return Err(TrainerError::Parse(format!(
            "WFDB record line is incomplete: '{record_line}'"
        )));
    }
    let n_signals: usize = rec_fields[1]
        .parse()
        .map_err(|_| TrainerError::Parse(format!("invalid signal count '{}'", rec_fields[1])))?;
    let freq_token = rec_fields[2].split('/').next().unwrap_or(rec_fields[2]);
    let sampling_frequency: f64 = freq_token
        .parse()
        .map_err(|_| TrainerError::Parse(format!("invalid sampling frequency '{freq_token}'")))?;
    if sampling_frequency <= 0.0 || !sampling_frequency.is_finite() {
        return Err(TrainerError::Parse(format!(
            "sampling frequency must be a positive finite value, got {sampling_frequency}"
        )));
    }
    let sample_count: usize = rec_fields
        .get(3)
        .ok_or_else(|| TrainerError::Parse("WFDB record line missing sample count".to_string()))?
        .parse()
        .map_err(|_| TrainerError::Parse(format!("invalid sample count '{}'", rec_fields[3])))?;

    let mut signals = Vec::with_capacity(n_signals);
    for _ in 0..n_signals {
        let line = lines.next().ok_or_else(|| {
            TrainerError::Parse("WFDB header has fewer signal lines than declared".to_string())
        })?;
        signals.push(parse_signal_line(line)?);
    }
    Ok(WfdbHeader {
        sampling_frequency,
        sample_count,
        signals,
    })
}

fn parse_signal_line(line: &str) -> Result<WfdbSignalSpec, TrainerError> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 5 {
        return Err(TrainerError::Parse(format!(
            "WFDB signal line is incomplete: '{line}'"
        )));
    }
    let format: u32 = fields[1]
        .parse()
        .map_err(|_| TrainerError::Parse(format!("invalid WFDB format '{}'", fields[1])))?;
    let gain: f64 = fields[2]
        .split('/')
        .next()
        .unwrap_or(fields[2])
        .parse()
        .map_err(|_| TrainerError::Parse(format!("invalid gain '{}'", fields[2])))?;
    let baseline: f64 = fields[4]
        .parse()
        .map_err(|_| TrainerError::Parse(format!("invalid baseline '{}'", fields[4])))?;
    Ok(WfdbSignalSpec {
        file_name: fields[0].to_string(),
        format,
        gain,
        baseline,
    })
}

fn decode_signal_file(
    bytes: &[u8],
    format: u32,
    sample_count: usize,
    n_signals: usize,
) -> Result<Vec<i32>, TrainerError> {
    match format {
        16 => decode_format_16(bytes, sample_count, n_signals),
        212 => decode_format_212(bytes, sample_count, n_signals),
        other => Err(TrainerError::Unsupported(format!(
            "WFDB signal format {other} is not supported (supported: 16, 212)"
        ))),
    }
}

fn decode_format_16(
    bytes: &[u8],
    sample_count: usize,
    n_signals: usize,
) -> Result<Vec<i32>, TrainerError> {
    let expected = sample_count * n_signals * 2;
    if bytes.len() < expected {
        return Err(TrainerError::Parse(format!(
            "format 16 file is {0} bytes, expected at least {expected}",
            bytes.len()
        )));
    }
    let mut out = Vec::with_capacity(sample_count * n_signals);
    for chunk in bytes[..expected].chunks_exact(2) {
        out.push(i16::from_le_bytes([chunk[0], chunk[1]]) as i32);
    }
    Ok(out)
}

fn decode_format_212(
    bytes: &[u8],
    sample_count: usize,
    n_signals: usize,
) -> Result<Vec<i32>, TrainerError> {
    let n_values = sample_count * n_signals;
    let n_pairs = n_values.div_ceil(2);
    let expected = n_pairs * 3;
    if bytes.len() < expected {
        return Err(TrainerError::Parse(format!(
            "format 212 file is {0} bytes, expected at least {expected}",
            bytes.len()
        )));
    }
    let mut out = Vec::with_capacity(n_values);
    for pair_i in 0..n_pairs {
        let base = pair_i * 3;
        let b0 = bytes[base] as i32;
        let b1 = bytes[base + 1] as i32;
        let b2 = bytes[base + 2] as i32;
        let s0 = sign_extend_12(b0 | ((b2 & 0x0f) << 8));
        let s1 = sign_extend_12(b1 | ((b2 & 0xf0) << 4));
        out.push(s0);
        if out.len() < n_values {
            out.push(s1);
        }
    }
    Ok(out)
}

fn sign_extend_12(raw: i32) -> i32 {
    let masked = raw & 0x0fff;
    if masked & 0x0800 != 0 {
        masked | !0x0fff
    } else {
        masked
    }
}

/// MIT annotation type codes used by WFDB `annstr`.
fn annotation_symbol(code: u8) -> Result<&'static str, TrainerError> {
    match code {
        1 => Ok("N"),
        2 => Ok("L"),
        3 => Ok("R"),
        4 => Ok("a"),
        5 => Ok("V"),
        6 => Ok("F"),
        7 => Ok("J"),
        8 => Ok("A"),
        9 => Ok("S"),
        10 => Ok("E"),
        11 => Ok("j"),
        12 => Ok("/"),
        13 => Ok("Q"),
        14 => Ok("~"),
        16 => Ok("|"),
        18 => Ok("s"),
        19 => Ok("T"),
        20 => Ok("*"),
        21 => Ok("D"),
        22 => Ok("\""),
        23 => Ok("="),
        24 => Ok("p"),
        25 => Ok("B"),
        26 => Ok("^"),
        27 => Ok("t"),
        28 => Ok("+"),
        29 => Ok("u"),
        30 => Ok("?"),
        31 => Ok("!"),
        32 => Ok("["),
        33 => Ok("]"),
        34 => Ok("e"),
        35 => Ok("n"),
        36 => Ok("@"),
        37 => Ok("x"),
        38 => Ok("f"),
        39 => Ok("("),
        40 => Ok(")"),
        41 => Ok("r"),
        other => Err(TrainerError::Parse(format!(
            "unsupported WFDB annotation code {other}"
        ))),
    }
}

const SKIP: u8 = 59;
const NUM: u8 = 60;
const SUB: u8 = 61;
const CHN: u8 = 62;
const AUX: u8 = 63;

fn read_mit_annotations(bytes: &[u8]) -> Result<Vec<AnalogEvent>, TrainerError> {
    let mut events = Vec::new();
    let mut i = 0;
    let mut time: i64 = 0;
    while i + 1 < bytes.len() {
        let word = u16::from_le_bytes([bytes[i], bytes[i + 1]]);
        i += 2;
        let code = (word >> 10) as u8;
        let delta = (word & 0x03ff) as i64;
        if code == SKIP {
            if i + 4 > bytes.len() {
                return Err(TrainerError::Parse(
                    "truncated WFDB SKIP annotation".to_string(),
                ));
            }
            let extra = i32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
            i += 4;
            time = time.checked_add(extra as i64).ok_or_else(|| {
                TrainerError::Parse("WFDB SKIP annotation time overflow".to_string())
            })?;
            continue;
        }
        if code == NUM || code == SUB || code == CHN {
            continue;
        }
        if code == AUX {
            // WFDB `getann`: length is the low 8 bits of the AUX word. The
            // following `(len + 1) & !1` bytes are payload (even-padded).
            // The first payload byte is data, not a second length.
            let aux_len = (word & 0x00ff) as usize;
            let padded = (aux_len + 1) & !1;
            if i + padded > bytes.len() {
                return Err(TrainerError::Parse(
                    "truncated WFDB AUX annotation".to_string(),
                ));
            }
            i += padded;
            continue;
        }
        if word == 0 {
            break;
        }
        if code == 0 {
            continue;
        }
        time = time
            .checked_add(delta)
            .ok_or_else(|| TrainerError::Parse("WFDB annotation time overflow".to_string()))?;
        if time < 0 {
            return Err(TrainerError::Parse(format!(
                "WFDB annotation time is negative ({time})"
            )));
        }
        events.push(AnalogEvent {
            sample_index: time as u64,
            label: annotation_symbol(code)?.to_string(),
        });
    }
    Ok(events)
}

/// Writes a minimal format-212 record for tests and integration fixtures.
pub fn write_test_record(
    dir: &Path,
    record: &str,
    samples: &[i16],
    n_signals: usize,
    annotations: &[(u64, u8)],
) -> Result<(), TrainerError> {
    if !samples.len().is_multiple_of(n_signals) {
        return Err(TrainerError::Config(
            "test sample count is not divisible by n_signals".to_string(),
        ));
    }
    let sample_count = samples.len() / n_signals;
    let mut hea = format!("{record} {n_signals} 360 {sample_count}\n");
    for signal_i in 0..n_signals {
        hea.push_str(&format!(
            "{record}.dat 212 200 11 0 0 0 {signal_i} lead{signal_i}\n"
        ));
    }
    fs::write(dir.join(format!("{record}.hea")), hea)
        .map_err(|e| TrainerError::Io(e.to_string()))?;
    fs::write(
        dir.join(format!("{record}.dat")),
        encode_format_212(samples),
    )
    .map_err(|e| TrainerError::Io(e.to_string()))?;
    fs::write(
        dir.join(format!("{record}.atr")),
        encode_mit_annotations(annotations)?,
    )
    .map_err(|e| TrainerError::Io(e.to_string()))?;
    Ok(())
}

fn encode_format_212(samples: &[i16]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < samples.len() {
        let s0 = samples[i] as i32;
        let s1 = if i + 1 < samples.len() {
            samples[i + 1] as i32
        } else {
            0
        };
        out.push((s0 & 0xff) as u8);
        out.push((s1 & 0xff) as u8);
        out.push((((s0 >> 8) & 0x0f) | ((s1 >> 4) & 0xf0)) as u8);
        i += 2;
    }
    out
}

fn encode_mit_annotations(annotations: &[(u64, u8)]) -> Result<Vec<u8>, TrainerError> {
    let mut out = Vec::new();
    let mut prev = 0u64;
    for &(time, code) in annotations {
        if time < prev {
            return Err(TrainerError::Config(
                "test annotations must be non-decreasing".to_string(),
            ));
        }
        let delta = time - prev;
        if delta > 0x03ff {
            let skip = (SKIP as u16) << 10;
            out.extend_from_slice(&skip.to_le_bytes());
            let extra = i32::try_from(delta)
                .map_err(|_| TrainerError::Config("annotation delta exceeds i32".to_string()))?;
            out.extend_from_slice(&extra.to_le_bytes());
            let word = (code as u16) << 10;
            out.extend_from_slice(&word.to_le_bytes());
        } else {
            let word = (delta as u16) | ((code as u16) << 10);
            out.extend_from_slice(&word.to_le_bytes());
        }
        prev = time;
    }
    out.extend_from_slice(&0u16.to_le_bytes());
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::time_series::config::{
        IncompleteWindowPolicy, TimeSeriesNormalize, TimeSeriesSourceKind, TimeSeriesWindowConfig,
        UnknownLabelPolicy,
    };
    use crate::contracts::{Split, SplitId};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("feagi-wfdb-{nanos}"));
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn config(map: Vec<WfdbChannelMap>) -> TimeSeriesPackageConfig {
        TimeSeriesPackageConfig {
            dataset_name: "wfdb".to_string(),
            source_kind: TimeSeriesSourceKind::Wfdb,
            window: TimeSeriesWindowConfig {
                pre_samples: 2,
                post_samples: 2,
                stream_ids: vec!["lead_0".to_string()],
                feature_count: 5,
            },
            class_labels: vec!["N".to_string(), "V".to_string()],
            split: Split::Train,
            split_id: SplitId("train".to_string()),
            normalize: TimeSeriesNormalize::MinMaxPerWindow,
            incomplete_window_policy: IncompleteWindowPolicy::Exclude,
            unknown_label_policy: UnknownLabelPolicy::Exclude,
            annotation_suffix: Some("atr".to_string()),
            channel_map: Some(map),
            presentation: crate::adapters::time_series::config::TimeSeriesPresentation::Snapshot,
        }
    }

    #[test]
    fn format_212_round_trip_two_signals() {
        let samples = [10i16, -20, 30, -40, 50, -60];
        let encoded = encode_format_212(&samples);
        let decoded = decode_format_212(&encoded, 3, 2).expect("decode");
        assert_eq!(decoded, vec![10, -20, 30, -40, 50, -60]);
    }

    #[test]
    fn loads_mapped_record() {
        let dir = temp_dir();
        // 8 samples, 2 signals, interleaved.
        let mut samples = Vec::new();
        for t in 0..8 {
            samples.push(t as i16);
            samples.push(-(t as i16));
        }
        write_test_record(&dir, "100", &samples, 2, &[(3, 1), (5, 5)]).expect("write");
        let cfg = config(vec![
            WfdbChannelMap {
                source_index: 0,
                stream_id: "lead_0".to_string(),
            },
            WfdbChannelMap {
                source_index: 1,
                stream_id: "lead_1".to_string(),
            },
        ]);
        let episodes = load_wfdb_directory(&dir, &cfg).expect("load");
        assert_eq!(episodes.len(), 1);
        assert_eq!(episodes[0].events.len(), 2);
        assert_eq!(episodes[0].events[1].label, "V");
        assert_eq!(episodes[0].streams["lead_0"].len(), 8);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn aux_payload_does_not_desync_following_beats() {
        // Official MIT layout: beat, AUX (length in the word), beat.
        // A first-payload-byte length would skip the second beat or invent codes.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(((1u16) << 10) | 10).to_le_bytes());
        bytes.extend_from_slice(&(((AUX as u16) << 10) | 2).to_le_bytes());
        bytes.extend_from_slice(b"xy");
        bytes.extend_from_slice(&(((5u16) << 10) | 10).to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        let events = read_mit_annotations(&bytes).expect("annotations");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].label, "N");
        assert_eq!(events[0].sample_index, 10);
        assert_eq!(events[1].label, "V");
        assert_eq!(events[1].sample_index, 20);
    }

    #[test]
    fn odd_length_aux_consumes_pad_byte() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(((1u16) << 10) | 4).to_le_bytes());
        bytes.extend_from_slice(&(((AUX as u16) << 10) | 3).to_le_bytes());
        bytes.extend_from_slice(b"ab");
        bytes.push(b'c');
        bytes.push(0);
        bytes.extend_from_slice(&(((5u16) << 10) | 6).to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        let events = read_mit_annotations(&bytes).expect("annotations");
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].label, "V");
        assert_eq!(events[1].sample_index, 10);
    }

    #[test]
    fn rejects_incomplete_channel_map() {
        let dir = temp_dir();
        write_test_record(&dir, "100", &[1, 2, 3, 4], 2, &[(1, 1)]).expect("write");
        let cfg = config(vec![WfdbChannelMap {
            source_index: 0,
            stream_id: "lead_0".to_string(),
        }]);
        let err = load_wfdb_directory(&dir, &cfg).unwrap_err();
        assert!(matches!(err, TrainerError::Parse(_)));
        let _ = fs::remove_dir_all(&dir);
    }
}
