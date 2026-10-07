// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! # FEAGI Connectome I/O
//!
//! File I/O and serialization for connectome snapshots.
//! Types are defined in `feagi-npu-neural::types::connectome`.
//!
//! This module provides:
//! - File I/O (`save_connectome`, `load_connectome`)
//! - Future: Network transport (ZMQ, WebSocket) for connectome transfer
//!
//! ## Usage
//! ```ignore
//! use feagi_services::connectome::{load_connectome, save_connectome};
//! use feagi_npu_neural::types::connectome::ConnectomeSnapshot;
//!
//! // Save connectome
//! let snapshot = ConnectomeSnapshot { /* ... */ };
//! save_connectome(&snapshot, "brain.connectome")?;
//!
//! // Load connectome
//! let snapshot = load_connectome("brain.connectome")?;
//! ```

use feagi_npu_neural::types::connectome::ConnectomeSnapshot;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use thiserror::Error;

/// Metadata tag holding associative class-channel weights outside schema v1 bincode.
const LTM_CLASS_CHANNEL_WEIGHTS_TAG: &str = "ltm_class_channel_weights_v1";

/// Metadata tag holding per-synapse modulation groups outside schema v1 bincode.
const SYNAPSE_MODULATION_GROUPS_TAG: &str = "synapse_modulation_groups_v1";

#[derive(Debug, Serialize, Deserialize)]
struct ClassChannelWeightRecord {
    neuron_id: u32,
    weights: Vec<(u32, f32)>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModulationGroupRecord {
    index: u64,
    group: u16,
}

/// Connectome I/O errors
#[derive(Error, Debug)]
pub enum ConnectomeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Version mismatch: file version {file_version}, expected {expected_version}")]
    VersionMismatch {
        file_version: u32,
        expected_version: u32,
    },

    #[error("Invalid magic number: expected FEAGI, got {0:?}")]
    InvalidMagic([u8; 5]),

    #[error("Checksum mismatch: file may be corrupted")]
    ChecksumMismatch,

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Brain artifact compatibility error: {0}")]
    BrainArtifact(String),
}

pub type Result<T> = std::result::Result<T, ConnectomeError>;

/// Magic number for connectome files: "FEAGI"
const MAGIC: &[u8; 5] = b"FEAGI";

/// Current format version (increment when format changes)
/// Version 1: Original format without compression
/// Version 2: Added flags byte for compression support
const FORMAT_VERSION: u32 = 3;

/// Return the binary container format version without deserializing its body.
pub fn connectome_container_version(bytes: &[u8]) -> Result<u32> {
    if bytes.len() < MAGIC.len() + std::mem::size_of::<u32>() {
        return Err(ConnectomeError::BrainArtifact(
            "connectome header is truncated".to_string(),
        ));
    }
    if &bytes[..MAGIC.len()] != MAGIC {
        let mut actual = [0u8; 5];
        actual.copy_from_slice(&bytes[..MAGIC.len()]);
        return Err(ConnectomeError::InvalidMagic(actual));
    }
    let mut version = [0u8; 4];
    version.copy_from_slice(&bytes[MAGIC.len()..MAGIC.len() + 4]);
    Ok(u32::from_le_bytes(version))
}

/// Save a connectome to a file with optional LZ4 compression
///
/// # Arguments
/// * `snapshot` - The connectome snapshot to save
/// * `path` - File path to write to
///
/// # Format
/// ```text
/// [Header]
/// - Magic: "FEAGI" (5 bytes)
/// - Version: u32 (4 bytes)
/// - Flags: u8 (1 byte) - bit 0: compressed
/// - Uncompressed Size: u64 (8 bytes, original size before compression)
/// - Checksum: u64 (8 bytes, CRC64 of data)
/// [Data]
/// - Bincode-serialized ConnectomeSnapshot (optionally LZ4 compressed)
/// ```
pub fn save_connectome<P: AsRef<Path>>(snapshot: &ConnectomeSnapshot, path: P) -> Result<()> {
    let mut file = File::create(path)?;
    write_connectome_to_writer(snapshot, &mut file)
}

/// Serialize a connectome into `.connectome` binary bytes.
pub fn save_connectome_to_bytes(snapshot: &ConnectomeSnapshot) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    write_connectome_to_writer(snapshot, &mut bytes)?;
    Ok(bytes)
}

fn write_connectome_to_writer<W: Write>(
    snapshot: &ConnectomeSnapshot,
    writer: &mut W,
) -> Result<()> {
    let mut export_snapshot = snapshot.clone();
    crate::brain_artifact::embed_manifest_for_current_export(&mut export_snapshot)?;
    persist_class_channel_weights_for_schema_v1(&mut export_snapshot)?;
    persist_modulation_groups_for_schema_v1(&mut export_snapshot)?;
    let manifest = crate::brain_artifact::encoded_manifest(&export_snapshot)?;
    let manifest_bytes = manifest.as_bytes();
    let manifest_len = u32::try_from(manifest_bytes.len()).map_err(|_| {
        ConnectomeError::BrainArtifact("artifact manifest exceeds maximum length".to_string())
    })?;

    // Write header
    writer.write_all(MAGIC)?;
    writer.write_all(&FORMAT_VERSION.to_le_bytes())?;

    // Serialize data
    let data = bincode::serialize(&export_snapshot)
        .map_err(|e| ConnectomeError::Serialization(e.to_string()))?;

    // Compress if feature enabled
    #[cfg(feature = "connectome-compression")]
    let (final_data, flags, uncompressed_size) = {
        let original_size = data.len();
        let compressed = lz4::block::compress(&data, None, false)
            .map_err(|e| ConnectomeError::Compression(e.to_string()))?;
        (compressed, 1u8, original_size as u64) // Flag bit 0 = compressed
    };

    #[cfg(not(feature = "connectome-compression"))]
    let (final_data, flags, uncompressed_size) = (data, 0u8, 0u64);

    // Write flags
    writer.write_all(&[flags])?;

    // Write uncompressed size (only meaningful if compressed)
    writer.write_all(&uncompressed_size.to_le_bytes())?;

    // Container v3 stores compatibility metadata before the opaque bincode body
    // so readers can select an immutable historical DTO before deserialization.
    writer.write_all(&manifest_len.to_le_bytes())?;
    writer.write_all(manifest_bytes)?;

    // Calculate checksum
    let checksum = calculate_checksum(&final_data);
    writer.write_all(&checksum.to_le_bytes())?;

    // Write data
    writer.write_all(&final_data)?;

    Ok(())
}

/// Load a connectome from a file with automatic LZ4 decompression
///
/// # Arguments
/// * `path` - File path to read from
///
/// # Returns
/// The deserialized connectome snapshot
pub fn load_connectome<P: AsRef<Path>>(path: P) -> Result<ConnectomeSnapshot> {
    let mut file = File::open(path)?;
    load_connectome_from_reader(&mut file)
}

/// Load a connectome from an in-memory `.connectome` file payload.
pub fn load_connectome_from_bytes(bytes: &[u8]) -> Result<ConnectomeSnapshot> {
    load_connectome_from_reader(&mut std::io::Cursor::new(bytes))
}

fn load_connectome_from_reader<R: Read>(reader: &mut R) -> Result<ConnectomeSnapshot> {
    // Read and verify magic number
    let mut magic = [0u8; 5];
    reader.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(ConnectomeError::InvalidMagic(magic));
    }

    // Read and verify version
    let mut version_bytes = [0u8; 4];
    reader.read_exact(&mut version_bytes)?;
    let version = u32::from_le_bytes(version_bytes);

    // Versions 1 and 2 are legacy containers. Version 3 adds a manifest
    // before the opaque serialized body.
    if version != 1 && version != 2 && version != 3 {
        return Err(ConnectomeError::VersionMismatch {
            file_version: version,
            expected_version: FORMAT_VERSION,
        });
    }

    // Read flags (versions 2 and 3)
    let (is_compressed, uncompressed_size) = if version >= 2 {
        let mut flags = [0u8; 1];
        reader.read_exact(&mut flags)?;
        let compressed = (flags[0] & 1) != 0;

        // Read uncompressed size
        let mut size_bytes = [0u8; 8];
        reader.read_exact(&mut size_bytes)?;
        let size = u64::from_le_bytes(size_bytes);

        (compressed, size as usize)
    } else {
        (false, 0) // Version 1 files are never compressed
    };

    let envelope_manifest = if version == 3 {
        let mut manifest_len_bytes = [0u8; 4];
        reader.read_exact(&mut manifest_len_bytes)?;
        let manifest_len = u32::from_le_bytes(manifest_len_bytes) as usize;
        let mut manifest_bytes = vec![0u8; manifest_len];
        reader.read_exact(&mut manifest_bytes)?;
        Some(String::from_utf8(manifest_bytes).map_err(|error| {
            ConnectomeError::BrainArtifact(format!("artifact manifest is not UTF-8: {error}"))
        })?)
    } else {
        None
    };

    // Read checksum
    let mut checksum_bytes = [0u8; 8];
    reader.read_exact(&mut checksum_bytes)?;
    let expected_checksum = u64::from_le_bytes(checksum_bytes);

    // Read data
    let mut compressed_data = Vec::new();
    reader.read_to_end(&mut compressed_data)?;

    // Verify checksum
    let actual_checksum = calculate_checksum(&compressed_data);
    if actual_checksum != expected_checksum {
        return Err(ConnectomeError::ChecksumMismatch);
    }

    // Decompress if needed
    let data = if is_compressed {
        #[cfg(feature = "connectome-compression")]
        {
            lz4::block::decompress(&compressed_data, Some(uncompressed_size as i32))
                .map_err(|e| ConnectomeError::Compression(format!("Decompression failed: {}", e)))?
        }
        #[cfg(not(feature = "connectome-compression"))]
        {
            return Err(ConnectomeError::Compression(
                "File is compressed but compression feature is not enabled".to_string(),
            ));
        }
    } else {
        compressed_data
    };

    // Deserialize
    let mut snapshot: ConnectomeSnapshot =
        bincode::deserialize(&data).map_err(|e| ConnectomeError::Deserialization(e.to_string()))?;
    restore_class_channel_weights_for_schema_v1(&mut snapshot)?;
    restore_modulation_groups_for_schema_v1(&mut snapshot)?;

    if let Some(envelope_manifest) = envelope_manifest {
        let snapshot_manifest = crate::brain_artifact::encoded_manifest(&snapshot)?;
        if snapshot_manifest != envelope_manifest {
            return Err(ConnectomeError::BrainArtifact(
                "envelope manifest does not match serialized snapshot manifest".to_string(),
            ));
        }
    }

    Ok(snapshot)
}

/// Calculate a simple checksum (CRC64-like)
pub(crate) fn calculate_checksum(data: &[u8]) -> u64 {
    // Simple FNV-1a hash for now (can upgrade to proper CRC64 later)
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Keep associative class-channel weights out of the schema v1 neuron record.
///
/// Published connectome readers still use that positional layout. An extra
/// field is read as the next neuron and fails with an invalid option tag.
fn persist_class_channel_weights_for_schema_v1(snapshot: &mut ConnectomeSnapshot) -> Result<()> {
    let mut seen_ids = HashSet::new();
    let mut records = Vec::new();
    for neuron in &snapshot.long_term_memory_neurons {
        if neuron.class_channel_weights.is_empty() {
            continue;
        }
        if !seen_ids.insert(neuron.neuron_id) {
            return Err(ConnectomeError::Serialization(format!(
                "duplicate long-term memory neuron id {} in class channel weights",
                neuron.neuron_id
            )));
        }
        records.push(ClassChannelWeightRecord {
            neuron_id: neuron.neuron_id,
            weights: neuron.class_channel_weights.clone(),
        });
    }
    if records.is_empty() {
        snapshot.metadata.tags.remove(LTM_CLASS_CHANNEL_WEIGHTS_TAG);
        return Ok(());
    }
    let encoded = serde_json::to_string(&records).map_err(|error| {
        ConnectomeError::Serialization(format!(
            "failed to encode long-term memory class channel weights: {error}"
        ))
    })?;
    snapshot
        .metadata
        .tags
        .insert(LTM_CLASS_CHANNEL_WEIGHTS_TAG.to_string(), encoded);
    Ok(())
}

fn restore_class_channel_weights_for_schema_v1(snapshot: &mut ConnectomeSnapshot) -> Result<()> {
    let Some(encoded) = snapshot.metadata.tags.get(LTM_CLASS_CHANNEL_WEIGHTS_TAG) else {
        return Ok(());
    };
    let records: Vec<ClassChannelWeightRecord> =
        serde_json::from_str(encoded).map_err(|error| {
            ConnectomeError::BrainArtifact(format!(
                "long-term memory class channel weights tag is invalid: {error}"
            ))
        })?;
    let mut by_id = std::collections::HashMap::new();
    for record in records {
        if record.weights.is_empty() {
            return Err(ConnectomeError::BrainArtifact(format!(
                "long-term memory neuron {} has an empty class channel weight record",
                record.neuron_id
            )));
        }
        if by_id.insert(record.neuron_id, record.weights).is_some() {
            return Err(ConnectomeError::BrainArtifact(format!(
                "duplicate class channel weight record for long-term memory neuron {}",
                record.neuron_id
            )));
        }
    }
    for neuron in &mut snapshot.long_term_memory_neurons {
        if let Some(weights) = by_id.remove(&neuron.neuron_id) {
            neuron.class_channel_weights = weights;
        }
    }
    if !by_id.is_empty() {
        let mut missing: Vec<u32> = by_id.into_keys().collect();
        missing.sort_unstable();
        return Err(ConnectomeError::BrainArtifact(format!(
            "class channel weights refer to unknown long-term memory neurons: {missing:?}"
        )));
    }
    Ok(())
}

fn persist_modulation_groups_for_schema_v1(snapshot: &mut ConnectomeSnapshot) -> Result<()> {
    let records: Vec<ModulationGroupRecord> = snapshot
        .synapses
        .modulation_groups
        .iter()
        .enumerate()
        .filter(|(_, group)| **group != 0)
        .map(|(index, group)| ModulationGroupRecord {
            index: index as u64,
            group: *group,
        })
        .collect();
    if records.is_empty() {
        snapshot.metadata.tags.remove(SYNAPSE_MODULATION_GROUPS_TAG);
        return Ok(());
    }
    let encoded = serde_json::to_string(&records).map_err(|error| {
        ConnectomeError::Serialization(format!(
            "failed to encode synapse modulation groups: {error}"
        ))
    })?;
    snapshot
        .metadata
        .tags
        .insert(SYNAPSE_MODULATION_GROUPS_TAG.to_string(), encoded);
    Ok(())
}

fn restore_modulation_groups_for_schema_v1(snapshot: &mut ConnectomeSnapshot) -> Result<()> {
    let Some(encoded) = snapshot.metadata.tags.get(SYNAPSE_MODULATION_GROUPS_TAG) else {
        return Ok(());
    };
    let records: Vec<ModulationGroupRecord> = serde_json::from_str(encoded).map_err(|error| {
        ConnectomeError::BrainArtifact(format!("synapse modulation groups tag is invalid: {error}"))
    })?;
    let count = snapshot.synapses.count;
    let mut groups = vec![0u16; count];
    for record in records {
        if record.group == 0 {
            return Err(ConnectomeError::BrainArtifact(
                "synapse modulation group 0 is not stored".to_string(),
            ));
        }
        let index = usize::try_from(record.index).map_err(|_| {
            ConnectomeError::BrainArtifact(format!(
                "synapse modulation group index {} does not fit this platform",
                record.index
            ))
        })?;
        if index >= count {
            return Err(ConnectomeError::BrainArtifact(format!(
                "synapse modulation group index {index} is outside synapse count {count}"
            )));
        }
        if groups[index] != 0 {
            return Err(ConnectomeError::BrainArtifact(format!(
                "duplicate synapse modulation group at index {index}"
            )));
        }
        groups[index] = record.group;
    }
    snapshot.synapses.modulation_groups = groups;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_npu_neural::types::connectome::{
        ConnectomeMetadata, SerializableLongTermMemoryNeuron, SerializableMemoryReplayFrame,
        SerializableNeuronArray, SerializableNeuronReference, SerializableSemanticSynapse,
        SerializableSynapseArray,
    };
    use serde::Serialize;
    use tempfile::NamedTempFile;

    #[test]
    fn pre_modulator_synapse_layout_deserializes() {
        #[derive(Serialize)]
        struct LegacySynapseArray {
            count: usize,
            capacity: usize,
            source_neurons: Vec<u32>,
            target_neurons: Vec<u32>,
            weights: Vec<f32>,
            postsynaptic_potentials: Vec<f32>,
            types: Vec<u8>,
            delay_bursts: Vec<u8>,
            valid_mask: Vec<bool>,
            source_index: ahash::AHashMap<u32, Vec<usize>>,
            edge_flags: Vec<u8>,
            eligibility_traces: Vec<f32>,
        }

        let legacy = LegacySynapseArray {
            count: 1,
            capacity: 1,
            source_neurons: vec![1],
            target_neurons: vec![2],
            weights: vec![0.5],
            postsynaptic_potentials: vec![1.0],
            types: vec![0],
            delay_bursts: vec![1],
            valid_mask: vec![true],
            source_index: ahash::AHashMap::new(),
            edge_flags: vec![0],
            eligibility_traces: vec![0.0],
        };
        let bytes = bincode::serialize(&legacy).unwrap();
        let loaded: SerializableSynapseArray = bincode::deserialize(&bytes).unwrap();
        assert_eq!(loaded.count, 1);
        assert!(loaded.modulation_groups.is_empty());
    }

    #[test]
    fn modulation_groups_roundtrip_outside_schema_v1_layout() {
        let mut snapshot = ConnectomeSnapshot {
            version: 1,
            neurons: SerializableNeuronArray::default(),
            synapses: SerializableSynapseArray {
                count: 2,
                ..SerializableSynapseArray::default()
            },
            cortical_area_names: ahash::AHashMap::new(),
            burst_count: 3,
            power_amount: 1.0,
            fire_ledger_window: 20,
            metadata: ConnectomeMetadata::default(),
            persist_mode: feagi_npu_neural::types::connectome::ConnectomePersistMode::Full,
            genome_json: None,
            memory_area_ids: Vec::new(),
            plastic_mappings: Vec::new(),
            brain_region_ids: Vec::new(),
            long_term_memory_neurons: Vec::new(),
            long_term_memory_replay_frames: Vec::new(),
            lite_synapses: Vec::new(),
        };
        snapshot.synapses.modulation_groups = vec![0, 4];
        let bytes = save_connectome_to_bytes(&snapshot).unwrap();
        let loaded = load_connectome_from_bytes(&bytes).unwrap();
        assert_eq!(loaded.synapses.modulation_groups, vec![0, 4]);
        assert!(loaded
            .metadata
            .tags
            .contains_key(SYNAPSE_MODULATION_GROUPS_TAG));
    }

    #[test]
    fn test_save_load_roundtrip() {
        // Create a minimal snapshot
        let snapshot = ConnectomeSnapshot {
            version: 1,
            neurons: SerializableNeuronArray::default(),
            synapses: SerializableSynapseArray::default(),
            cortical_area_names: ahash::AHashMap::new(),
            burst_count: 42,
            power_amount: 1.0,
            fire_ledger_window: 20,
            metadata: ConnectomeMetadata::default(),
            persist_mode: feagi_npu_neural::types::connectome::ConnectomePersistMode::Full,
            genome_json: None,
            memory_area_ids: Vec::new(),
            plastic_mappings: Vec::new(),
            brain_region_ids: Vec::new(),
            long_term_memory_neurons: Vec::new(),
            long_term_memory_replay_frames: Vec::new(),
            lite_synapses: Vec::new(),
        };

        // Save to temp file
        let temp_file = NamedTempFile::new().unwrap();
        save_connectome(&snapshot, temp_file.path()).unwrap();

        // Load back
        let loaded = load_connectome(temp_file.path()).unwrap();

        // Verify
        assert_eq!(loaded.version, snapshot.version);
        assert_eq!(loaded.burst_count, snapshot.burst_count);
        assert_eq!(loaded.power_amount, snapshot.power_amount);

        let bytes = std::fs::read(temp_file.path()).unwrap();
        let from_bytes = load_connectome_from_bytes(&bytes).unwrap();
        assert_eq!(from_bytes.burst_count, snapshot.burst_count);
        let serialized = save_connectome_to_bytes(&snapshot).unwrap();
        let from_serialized = load_connectome_from_bytes(&serialized).unwrap();
        assert_eq!(from_serialized.burst_count, snapshot.burst_count);
    }

    #[test]
    fn test_save_load_roundtrip_lite_with_memory_and_plasticity_payload() {
        let (current_genome, _) = feagi_evolutionary::migrate_genome_json_to_current(
            feagi_evolutionary::BAREBONES_GENOME_JSON,
        )
        .unwrap();
        let snapshot = ConnectomeSnapshot {
            version: 1,
            neurons: SerializableNeuronArray::default(),
            synapses: SerializableSynapseArray::default(),
            cortical_area_names: ahash::AHashMap::new(),
            burst_count: 0,
            power_amount: 1.0,
            fire_ledger_window: 20,
            metadata: ConnectomeMetadata {
                tags: ahash::AHashMap::from_iter([(
                    feagi_npu_neural::types::connectome::LITE_EDGE_ENCODING_TAG.to_string(),
                    feagi_npu_neural::types::connectome::LITE_EDGE_ENCODING_SEMANTIC_V1.to_string(),
                )]),
                ..ConnectomeMetadata::default()
            },
            persist_mode: feagi_npu_neural::types::connectome::ConnectomePersistMode::Lite,
            genome_json: Some(serde_json::to_string(&current_genome).unwrap()),
            memory_area_ids: vec!["mmem0001".to_string()],
            plastic_mappings: vec![("csrc0001".to_string(), "cdst0001".to_string())],
            brain_region_ids: vec!["region-1".to_string()],
            long_term_memory_neurons: vec![SerializableLongTermMemoryNeuron {
                neuron_id: 50_000_000,
                cortical_area_idx: 13,
                cortical_id: Some("bXZpc3VhbO4=".to_string()),
                pattern_hash: Some(0xBEEF),
                is_longterm_memory: true,
                is_active: true,
                lifespan_current: 100,
                lifespan_initial: 20,
                lifespan_growth_rate: 2.0,
                creation_burst: 1,
                last_activation_burst: 2,
                activation_count: 3,
                spatial_signature: None,
                class_channels: Vec::new(),
                class_channel_weights: Vec::new(),
            }],
            long_term_memory_replay_frames: vec![(
                50_000_000,
                vec![SerializableMemoryReplayFrame {
                    offset: 0,
                    upstream_area_idx: 7,
                    upstream_cortical_id: Some("csrc0001".to_string()),
                    coords: vec![(0, 0, 0)],
                    membrane_potentials: Some(vec![0.42]),
                }],
            )],
            lite_synapses: vec![
                SerializableSemanticSynapse {
                    source: SerializableNeuronReference::Regular {
                        cortical_id: "csrc0001".to_string(),
                        x: 0,
                        y: 0,
                        z: 0,
                        neuron_index: 0,
                    },
                    target: SerializableNeuronReference::Regular {
                        cortical_id: "cdst0001".to_string(),
                        x: 0,
                        y: 0,
                        z: 0,
                        neuron_index: 0,
                    },
                    weight: 0.95,
                    postsynaptic_potential: 1.1,
                    synapse_type: 0,
                    delay_bursts: 1,
                    edge_flags: 1,
                    eligibility_trace: 0.45,
                },
                SerializableSemanticSynapse {
                    source: SerializableNeuronReference::LongTermMemory {
                        snapshot_neuron_id: 50_000_000,
                        cortical_id: "mmem0001".to_string(),
                    },
                    target: SerializableNeuronReference::Regular {
                        cortical_id: "cdst0001".to_string(),
                        x: 0,
                        y: 0,
                        z: 0,
                        neuron_index: 0,
                    },
                    weight: 662.0,
                    postsynaptic_potential: 500.0,
                    synapse_type: 0,
                    delay_bursts: 1,
                    edge_flags: 0,
                    eligibility_trace: 0.0,
                },
            ],
        };

        let temp_file = NamedTempFile::new().unwrap();
        save_connectome(&snapshot, temp_file.path()).unwrap();
        let loaded = load_connectome(temp_file.path()).unwrap();
        assert!(loaded.is_lite_mode());
        assert_eq!(loaded.synapses.count, 0);
        assert_eq!(loaded.lite_synapses.len(), 2);
        assert_eq!(loaded.long_term_memory_neurons.len(), 1);
        assert_eq!(
            loaded.long_term_memory_neurons[0].cortical_id.as_deref(),
            Some("bXZpc3VhbO4=")
        );
        assert_eq!(loaded.long_term_memory_replay_frames.len(), 1);
    }

    #[test]
    fn test_invalid_magic() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut file = File::create(temp_file.path()).unwrap();
        file.write_all(b"WRONG").unwrap();

        let result = load_connectome(temp_file.path());
        assert!(matches!(result, Err(ConnectomeError::InvalidMagic(_))));
    }

    #[test]
    fn test_checksum() {
        let data1 = b"hello world";
        let data2 = b"hello world";
        let data3 = b"hello worlD";

        assert_eq!(calculate_checksum(data1), calculate_checksum(data2));
        assert_ne!(calculate_checksum(data1), calculate_checksum(data3));
    }

    /// Published schema v1 readers do not know `class_channel_weights`.
    /// Neuron id 50_002_623 begins with byte 191, which those readers report
    /// as an invalid option tag when the field is inserted into the record.
    #[test]
    fn class_channel_weights_round_trip_outside_schema_v1_layout() {
        #[derive(Debug, serde::Deserialize)]
        #[allow(dead_code)]
        struct SchemaV1LongTermMemoryNeuron {
            neuron_id: u32,
            cortical_area_idx: u32,
            #[serde(default)]
            cortical_id: Option<String>,
            pattern_hash: Option<u64>,
            is_longterm_memory: bool,
            is_active: bool,
            lifespan_current: u32,
            lifespan_initial: u32,
            lifespan_growth_rate: f32,
            creation_burst: u64,
            last_activation_burst: u64,
            activation_count: u32,
            #[serde(default)]
            spatial_signature: Option<u64>,
            #[serde(default)]
            class_channels: Vec<u32>,
        }

        #[derive(Debug, serde::Deserialize)]
        #[allow(dead_code)]
        struct SchemaV1Snapshot {
            version: u32,
            neurons: SerializableNeuronArray,
            synapses: SerializableSynapseArray,
            cortical_area_names: ahash::AHashMap<u32, String>,
            burst_count: u64,
            power_amount: f32,
            fire_ledger_window: usize,
            metadata: ConnectomeMetadata,
            persist_mode: feagi_npu_neural::types::connectome::ConnectomePersistMode,
            genome_json: Option<String>,
            memory_area_ids: Vec<String>,
            plastic_mappings: Vec<(String, String)>,
            brain_region_ids: Vec<String>,
            long_term_memory_neurons: Vec<SchemaV1LongTermMemoryNeuron>,
            long_term_memory_replay_frames: Vec<(u32, Vec<SerializableMemoryReplayFrame>)>,
            lite_synapses: Vec<SerializableSemanticSynapse>,
        }

        let weights = vec![(4u32, 0.25f32), (9u32, 1.5f32)];
        let snapshot = ConnectomeSnapshot {
            version: 1,
            neurons: SerializableNeuronArray::default(),
            synapses: SerializableSynapseArray::default(),
            cortical_area_names: ahash::AHashMap::new(),
            burst_count: 7,
            power_amount: 1.0,
            fire_ledger_window: 20,
            metadata: ConnectomeMetadata::default(),
            persist_mode: feagi_npu_neural::types::connectome::ConnectomePersistMode::Full,
            genome_json: None,
            memory_area_ids: Vec::new(),
            plastic_mappings: Vec::new(),
            brain_region_ids: Vec::new(),
            long_term_memory_neurons: vec![
                SerializableLongTermMemoryNeuron {
                    neuron_id: 50_000_000,
                    cortical_area_idx: 1,
                    cortical_id: Some("bU1OSVNUXyA=".to_string()),
                    pattern_hash: None,
                    is_longterm_memory: true,
                    is_active: true,
                    lifespan_current: 10,
                    lifespan_initial: 10,
                    lifespan_growth_rate: 1.0,
                    creation_burst: 1,
                    last_activation_burst: 1,
                    activation_count: 1,
                    spatial_signature: None,
                    class_channels: Vec::new(),
                    class_channel_weights: Vec::new(),
                },
                SerializableLongTermMemoryNeuron {
                    neuron_id: 50_002_623,
                    cortical_area_idx: 2,
                    cortical_id: Some("bU1OSVNUXx8=".to_string()),
                    pattern_hash: Some(3),
                    is_longterm_memory: true,
                    is_active: true,
                    lifespan_current: 10,
                    lifespan_initial: 10,
                    lifespan_growth_rate: 1.0,
                    creation_burst: 1,
                    last_activation_burst: 2,
                    activation_count: 2,
                    spatial_signature: None,
                    class_channels: vec![4, 9],
                    class_channel_weights: weights.clone(),
                },
            ],
            long_term_memory_replay_frames: Vec::new(),
            lite_synapses: Vec::new(),
        };

        let bytes = save_connectome_to_bytes(&snapshot).unwrap();
        let loaded = load_connectome_from_bytes(&bytes).unwrap();
        assert!(loaded
            .metadata
            .tags
            .contains_key(LTM_CLASS_CHANNEL_WEIGHTS_TAG));
        assert_eq!(
            loaded.long_term_memory_neurons[0].class_channel_weights,
            Vec::new()
        );
        assert_eq!(
            loaded.long_term_memory_neurons[1].class_channel_weights,
            weights
        );
        assert_eq!(loaded.long_term_memory_neurons[1].neuron_id, 50_002_623);

        let mut export_snapshot = snapshot.clone();
        persist_class_channel_weights_for_schema_v1(&mut export_snapshot).unwrap();
        let body = bincode::serialize(&export_snapshot).unwrap();
        let legacy: SchemaV1Snapshot = bincode::deserialize(&body).unwrap();
        assert_eq!(legacy.long_term_memory_neurons.len(), 2);
        assert_eq!(legacy.long_term_memory_neurons[0].neuron_id, 50_000_000);
        assert_eq!(legacy.long_term_memory_neurons[1].neuron_id, 50_002_623);
        assert_eq!(
            legacy.long_term_memory_neurons[1].class_channels,
            vec![4, 9]
        );
        assert!(legacy
            .metadata
            .tags
            .contains_key(LTM_CLASS_CHANNEL_WEIGHTS_TAG));
    }
}
