//! Canonical analog package reader (minimal Experience-package subset for time series).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::adapters::time_series::corpus::{AnalogEpisode, AnalogEvent};
use crate::error::TrainerError;

const MANIFEST_NAME: &str = "manifest.json";
const STREAM_SCHEMA_NAME: &str = "stream_schema.json";
const EPISODES_NAME: &str = "episodes.json";
const EVENTS_NAME: &str = "events.jsonl";

#[derive(Debug, Deserialize)]
struct PackageManifest {
    schema_version: u32,
    sample_rate_hz: f64,
}

#[derive(Debug, Deserialize)]
struct StreamSchemaFile {
    schema_version: u32,
    streams: Vec<StreamEntry>,
}

#[derive(Debug, Deserialize)]
struct StreamEntry {
    stream_id: String,
    dtype: String,
}

#[derive(Debug, Deserialize)]
struct EpisodeFile {
    schema_version: u32,
    episodes: Vec<EpisodeEntry>,
}

#[derive(Debug, Deserialize)]
struct EpisodeEntry {
    episode_id: String,
    sample_count: u64,
    stream_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EventLine {
    episode_id: String,
    sample_index: u64,
    label: String,
}

/// Loads a directory-shaped analog package.
pub fn load_package(root: &Path) -> Result<Vec<AnalogEpisode>, TrainerError> {
    let manifest: PackageManifest = read_json(root, MANIFEST_NAME)?;
    if manifest.schema_version != 1 {
        return Err(TrainerError::Parse(format!(
            "unsupported analog package manifest schema_version {}",
            manifest.schema_version
        )));
    }
    if manifest.sample_rate_hz <= 0.0 || !manifest.sample_rate_hz.is_finite() {
        return Err(TrainerError::Parse(
            "manifest.sample_rate_hz must be a positive finite value".to_string(),
        ));
    }

    let schema: StreamSchemaFile = read_json(root, STREAM_SCHEMA_NAME)?;
    if schema.schema_version != 1 {
        return Err(TrainerError::Parse(format!(
            "unsupported stream_schema schema_version {}",
            schema.schema_version
        )));
    }
    if schema.streams.is_empty() {
        return Err(TrainerError::Parse(
            "stream_schema.streams must not be empty".to_string(),
        ));
    }
    for stream in &schema.streams {
        if stream.dtype != "f32le" {
            return Err(TrainerError::Unsupported(format!(
                "stream '{}' dtype '{}' is not supported (supported: f32le)",
                stream.stream_id, stream.dtype
            )));
        }
    }

    let episodes_file: EpisodeFile = read_json(root, EPISODES_NAME)?;
    if episodes_file.schema_version != 1 {
        return Err(TrainerError::Parse(format!(
            "unsupported episodes schema_version {}",
            episodes_file.schema_version
        )));
    }
    if episodes_file.episodes.is_empty() {
        return Err(TrainerError::Parse(
            "episodes.json must list at least one episode".to_string(),
        ));
    }

    let events = read_events(&root.join(EVENTS_NAME))?;
    let mut events_by_episode: BTreeMap<String, Vec<AnalogEvent>> = BTreeMap::new();
    for event in events {
        events_by_episode
            .entry(event.episode_id.clone())
            .or_default()
            .push(AnalogEvent {
                sample_index: event.sample_index,
                label: event.label,
            });
    }

    let mut episodes = Vec::with_capacity(episodes_file.episodes.len());
    for entry in episodes_file.episodes {
        let mut streams = BTreeMap::new();
        for stream_id in &entry.stream_ids {
            if !schema.streams.iter().any(|s| s.stream_id == *stream_id) {
                return Err(TrainerError::Parse(format!(
                    "episode '{}': stream '{stream_id}' is not declared in stream_schema",
                    entry.episode_id
                )));
            }
            let path = root
                .join("streams")
                .join(&entry.episode_id)
                .join(format!("{stream_id}.f32le"));
            let bytes = fs::read(&path)
                .map_err(|e| TrainerError::Io(format!("cannot read '{}': {e}", path.display())))?;
            if bytes.len() % 4 != 0 {
                return Err(TrainerError::Parse(format!(
                    "stream file '{}' length is not a multiple of 4",
                    path.display()
                )));
            }
            let values: Vec<f32> = bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|chunk| f32::from_le_bytes(*chunk))
                .collect();
            if values.len() as u64 != entry.sample_count {
                return Err(TrainerError::Parse(format!(
                    "episode '{}': stream '{stream_id}' has {} samples, expected {}",
                    entry.episode_id,
                    values.len(),
                    entry.sample_count
                )));
            }
            streams.insert(stream_id.clone(), values);
        }
        episodes.push(AnalogEpisode {
            episode_id: entry.episode_id.clone(),
            sample_rate_hz: manifest.sample_rate_hz,
            streams,
            events: events_by_episode
                .remove(&entry.episode_id)
                .unwrap_or_default(),
        });
    }
    if !events_by_episode.is_empty() {
        let unknown: Vec<_> = events_by_episode.keys().cloned().collect();
        return Err(TrainerError::Parse(format!(
            "events.jsonl references unknown episode(s): {}",
            unknown.join(", ")
        )));
    }
    Ok(episodes)
}

fn read_json<T: for<'de> Deserialize<'de>>(root: &Path, name: &str) -> Result<T, TrainerError> {
    let path = root.join(name);
    let text = fs::read_to_string(&path)
        .map_err(|e| TrainerError::Io(format!("cannot read '{}': {e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| TrainerError::Parse(format!("invalid {name}: {e}")))
}

fn read_events(path: &Path) -> Result<Vec<EventLine>, TrainerError> {
    let text = fs::read_to_string(path)
        .map_err(|e| TrainerError::Io(format!("cannot read '{}': {e}", path.display())))?;
    let mut events = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event: EventLine = serde_json::from_str(line)
            .map_err(|e| TrainerError::Parse(format!("events.jsonl line {}: {e}", idx + 1)))?;
        events.push(event);
    }
    Ok(events)
}

/// Writes a package used by tests.
#[cfg(test)]
pub fn write_test_package(
    root: &Path,
    sample_rate_hz: f64,
    episode_id: &str,
    stream_id: &str,
    samples: &[f32],
    events: &[(u64, &str)],
) -> Result<(), TrainerError> {
    use std::io::Write;

    fs::create_dir_all(root.join("streams").join(episode_id))
        .map_err(|e| TrainerError::Io(e.to_string()))?;
    fs::write(
        root.join(MANIFEST_NAME),
        serde_json::json!({
            "schema_version": 1,
            "sample_rate_hz": sample_rate_hz
        })
        .to_string(),
    )
    .map_err(|e| TrainerError::Io(e.to_string()))?;
    fs::write(
        root.join(STREAM_SCHEMA_NAME),
        serde_json::json!({
            "schema_version": 1,
            "streams": [{ "stream_id": stream_id, "dtype": "f32le" }]
        })
        .to_string(),
    )
    .map_err(|e| TrainerError::Io(e.to_string()))?;
    fs::write(
        root.join(EPISODES_NAME),
        serde_json::json!({
            "schema_version": 1,
            "episodes": [{
                "episode_id": episode_id,
                "sample_count": samples.len() as u64,
                "stream_ids": [stream_id]
            }]
        })
        .to_string(),
    )
    .map_err(|e| TrainerError::Io(e.to_string()))?;

    let mut events_file =
        fs::File::create(root.join(EVENTS_NAME)).map_err(|e| TrainerError::Io(e.to_string()))?;
    for (sample_index, label) in events {
        writeln!(
            events_file,
            "{{\"episode_id\":\"{episode_id}\",\"sample_index\":{sample_index},\"label\":\"{label}\"}}"
        )
        .map_err(|e| TrainerError::Io(e.to_string()))?;
    }

    let mut bytes = Vec::with_capacity(samples.len() * 4);
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(
        root.join("streams")
            .join(episode_id)
            .join(format!("{stream_id}.f32le")),
        bytes,
    )
    .map_err(|e| TrainerError::Io(e.to_string()))?;
    Ok(())
}
