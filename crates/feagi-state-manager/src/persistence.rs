// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! State persistence (save/load)
//!
//! Uses `serde` + bincode for fast binary serialization on `std` targets.
//! For `no_std` and WASM, persistence is delegated to external storage handlers.

use crate::{Result, StateError};

/// Persistent state snapshot (serializable)
#[cfg(feature = "std")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateSnapshot {
    pub genome_state: u8,
    pub connectome_state: u8,
    pub burst_engine_state: u8,
    pub agent_count: u32,
    pub burst_frequency: f32,
    pub neuron_count: u32,
    pub synapse_count: u32,
    pub cortical_area_count: u32,
    pub version: u64,
    pub timestamp: u64,
}

#[cfg(feature = "std")]
impl StateSnapshot {
    /// Save snapshot to file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let encoded = bincode::serialize(self)
            .map_err(|e| StateError::PersistenceError(format!("Serialize failed: {}", e)))?;

        std::fs::write(path, encoded)
            .map_err(|e| StateError::PersistenceError(format!("Write failed: {}", e)))?;

        Ok(())
    }

    /// Load snapshot from file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self> {
        let data = std::fs::read(path)
            .map_err(|e| StateError::PersistenceError(format!("Read failed: {}", e)))?;

        let snapshot = bincode::deserialize(&data)
            .map_err(|e| StateError::PersistenceError(format!("Deserialize failed: {}", e)))?;

        Ok(snapshot)
    }
}

// For no_std and WASM, provide stubs
#[cfg(not(feature = "std"))]
pub struct StateSnapshot;

#[cfg(not(feature = "std"))]
impl StateSnapshot {
    pub fn save_to_file(&self, _path: &str) -> Result<()> {
        Err(StateError::PersistenceError(
            "Persistence not available on no_std".to_string(),
        ))
    }

    pub fn load_from_file(_path: &str) -> Result<Self> {
        Err(StateError::PersistenceError(
            "Persistence not available on no_std".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn test_snapshot_roundtrip() {
        let snapshot = StateSnapshot {
            genome_state: 2,
            connectome_state: 3,
            burst_engine_state: 3,
            agent_count: 5,
            burst_frequency: 30.0,
            neuron_count: 1_000_000,
            synapse_count: 50_000_000,
            cortical_area_count: 100,
            version: 42,
            timestamp: 123456789,
        };

        let temp_path = std::path::Path::new("/tmp/feagi_test_snapshot.bin");
        snapshot.save_to_file(temp_path).unwrap();

        let loaded = StateSnapshot::load_from_file(temp_path).unwrap();
        assert_eq!(loaded.genome_state, 2);
        assert_eq!(loaded.agent_count, 5);
        assert_eq!(loaded.neuron_count, 1_000_000);

        std::fs::remove_file(temp_path).ok();
    }
}
