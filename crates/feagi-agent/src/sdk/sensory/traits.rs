//! Sensory encoder trait. SDK surface for controllers.

/// Trait for sensory encoders (video, text, etc.).
pub trait SensoryEncoder {
    /// Encode frame and return bytes to send. Stub for SDK surface.
    fn encode(
        &mut self,
        _frame: &feagi_sensorimotor::data_types::ImageFrame,
    ) -> Result<Vec<u8>, crate::FeagiAgentClientError>;
}
