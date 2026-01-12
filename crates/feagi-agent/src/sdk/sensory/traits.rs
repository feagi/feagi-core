// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Sensory encoder traits

use crate::sdk::error::Result;
use feagi_structures::genomic::cortical_area::CorticalID;
use std::collections::HashMap;

/// Trait for encoding sensory data into FEAGI-compatible format
///
/// Implement this trait to create custom sensory encoders. The SDK provides
/// pre-built encoders for common modalities (video, text, audio).
///
/// # Example
/// ```ignore
/// use feagi_agent::sdk::sensory::SensoryEncoder;
///
/// struct MyEncoder { /* ... */ }
///
/// impl SensoryEncoder for MyEncoder {
///     type Input = MyDataType;
///
///     fn encode(&mut self, input: &Self::Input) -> Result<Vec<u8>> {
///         // Encode to FEAGI binary format
///         Ok(encoded_bytes)
///     }
///
///     fn cortical_ids(&self) -> &[CorticalID] {
///         &self.cortical_ids
///     }
///
///     fn cortical_id_mappings(&self) -> HashMap<String, u32> {
///         // Map cortical IDs to indices
///         HashMap::new()
///     }
/// }
/// ```
pub trait SensoryEncoder: Send + Sync {
    /// Input data type for this encoder
    type Input;

    /// Encode input data into FEAGI binary format
    ///
    /// Returns serialized XYZP voxel data ready to send to FEAGI.
    /// Takes `&mut self` to allow internal state updates (e.g., frame differencing).
    fn encode(&mut self, input: &Self::Input) -> Result<Vec<u8>>;

    /// Get cortical IDs this encoder produces
    fn cortical_ids(&self) -> &[CorticalID];

    /// Get cortical ID mappings for agent registration
    ///
    /// Maps cortical ID (base64) to index. Used during agent registration
    /// to tell FEAGI which cortical areas this agent will send data to.
    fn cortical_id_mappings(&self) -> HashMap<String, u32> {
        let mut mappings = HashMap::new();
        for (idx, id) in self.cortical_ids().iter().enumerate() {
            mappings.insert(id.as_base_64(), idx as u32);
        }
        mappings
    }
}
