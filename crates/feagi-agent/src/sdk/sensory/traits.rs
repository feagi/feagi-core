//! Sensory encoder trait definition.

use crate::core::SdkError;
use crate::sdk::types::CorticalID;

/// Trait for encoding sensory inputs into FEAGI-compatible byte payloads.
pub trait SensoryEncoder {
    /// Input type accepted by the encoder.
    type Input;

    /// Encode the input into FEAGI byte container format.
    fn encode(&mut self, input: &Self::Input) -> Result<Vec<u8>, SdkError>;

    /// Cortical IDs affected by this encoder.
    fn cortical_ids(&self) -> &[CorticalID];
}
