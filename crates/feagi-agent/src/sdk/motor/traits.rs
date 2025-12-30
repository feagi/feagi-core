// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Motor decoder traits

use crate::sdk::error::Result;
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

/// Trait for decoding motor data from FEAGI
///
/// Implement this trait to create custom motor decoders.
///
/// # Example
/// ```ignore
/// use feagi_agent::sdk::motor::MotorDecoder;
///
/// struct MyDecoder { /* ... */ }
///
/// impl MotorDecoder for MyDecoder {
///     type Output = MyCommandType;
///
///     fn decode(&self, data: &CorticalMappedXYZPNeuronVoxels) -> Result<Self::Output> {
///         // Decode FEAGI motor data
///         Ok(commands)
///     }
///
///     fn cortical_ids(&self) -> &[CorticalID] {
///         &self.cortical_ids
///     }
/// }
/// ```
pub trait MotorDecoder: Send + Sync {
    /// Output type for this decoder
    type Output;

    /// Decode motor data from FEAGI
    fn decode(&self, data: &CorticalMappedXYZPNeuronVoxels) -> Result<Self::Output>;

    /// Get cortical IDs this decoder consumes
    fn cortical_ids(&self) -> &[CorticalID];
}

