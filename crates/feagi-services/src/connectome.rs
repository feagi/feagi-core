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
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use thiserror::Error;

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
}

pub type Result<T> = std::result::Result<T, ConnectomeError>;

/// Magic number for connectome files: "FEAGI"
const MAGIC: &[u8; 5] = b"FEAGI";

/// Current format version (increment when format changes)
/// Version 1: Original format without compression
/// Version 2: Added flags byte for compression support
const FORMAT_VERSION: u32 = 2;

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

    // Write header
    file.write_all(MAGIC)?;
    file.write_all(&FORMAT_VERSION.to_le_bytes())?;

    // Serialize data
    let data =
        bincode::serialize(snapshot).map_err(|e| ConnectomeError::Serialization(e.to_string()))?;

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
    file.write_all(&[flags])?;

    // Write uncompressed size (only meaningful if compressed)
    file.write_all(&uncompressed_size.to_le_bytes())?;

    // Calculate checksum
    let checksum = calculate_checksum(&final_data);
    file.write_all(&checksum.to_le_bytes())?;

    // Write data
    file.write_all(&final_data)?;

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

    // Read and verify magic number
    let mut magic = [0u8; 5];
    file.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(ConnectomeError::InvalidMagic(magic));
    }

    // Read and verify version
    let mut version_bytes = [0u8; 4];
    file.read_exact(&mut version_bytes)?;
    let version = u32::from_le_bytes(version_bytes);

    // Support version 1 (no compression) and version 2 (with compression)
    if version != 1 && version != 2 {
        return Err(ConnectomeError::VersionMismatch {
            file_version: version,
            expected_version: FORMAT_VERSION,
        });
    }

    // Read flags (only in version 2)
    let (is_compressed, uncompressed_size) = if version == 2 {
        let mut flags = [0u8; 1];
        file.read_exact(&mut flags)?;
        let compressed = (flags[0] & 1) != 0;

        // Read uncompressed size
        let mut size_bytes = [0u8; 8];
        file.read_exact(&mut size_bytes)?;
        let size = u64::from_le_bytes(size_bytes);

        (compressed, size as usize)
    } else {
        (false, 0) // Version 1 files are never compressed
    };

    // Read checksum
    let mut checksum_bytes = [0u8; 8];
    file.read_exact(&mut checksum_bytes)?;
    let expected_checksum = u64::from_le_bytes(checksum_bytes);

    // Read data
    let mut compressed_data = Vec::new();
    file.read_to_end(&mut compressed_data)?;

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
    let snapshot: ConnectomeSnapshot =
        bincode::deserialize(&data).map_err(|e| ConnectomeError::Deserialization(e.to_string()))?;

    Ok(snapshot)
}

/// Calculate a simple checksum (CRC64-like)
fn calculate_checksum(data: &[u8]) -> u64 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_npu_neural::types::connectome::{
        ConnectomeMetadata, SerializableNeuronArray, SerializableSynapseArray,
    };
    use tempfile::NamedTempFile;

    #[test]
    fn test_save_load_roundtrip() {
        // Create a minimal snapshot
        let snapshot = ConnectomeSnapshot {
            version: FORMAT_VERSION,
            neurons: SerializableNeuronArray::default(),
            synapses: SerializableSynapseArray::default(),
            cortical_area_names: ahash::AHashMap::new(),
            burst_count: 42,
            power_amount: 1.0,
            fire_ledger_window: 20,
            metadata: ConnectomeMetadata::default(),
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
}
