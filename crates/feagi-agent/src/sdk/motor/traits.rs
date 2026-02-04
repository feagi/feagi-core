//! Motor decoder trait. SDK surface for controllers.

use crate::sdk::types::CorticalMappedXYZPNeuronVoxels;

use super::perception::PerceptionFrame;

/// Trait for motor decoders (perception, etc.).
pub trait MotorDecoder {
    /// Decode motor data into a perception frame.
    fn decode(
        &mut self,
        motor_data: &CorticalMappedXYZPNeuronVoxels,
    ) -> Result<Option<PerceptionFrame>, crate::FeagiAgentClientError>;
}
