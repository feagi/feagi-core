//! Motor decoder trait definition.

use crate::core::SdkError;

/// Trait for decoding FEAGI motor bytes or voxels into higher-level frames.
pub trait MotorDecoder {
    /// Input type accepted by the decoder.
    type Input;
    /// Output type produced by the decoder.
    type Output;

    /// Decode motor input into a higher-level representation.
    fn decode(&self, input: &Self::Input) -> Result<Self::Output, SdkError>;
}
