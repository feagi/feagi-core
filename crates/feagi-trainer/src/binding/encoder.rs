//! Encoder axis — a *selector* over FEAGI's native coders (ADR-002, Appendix B.1).
//!
//! An encoder turns an [`IRSample`] into a runtime sensory frame. It implements no
//! spike-coding math; it selects/configures a FEAGI coder (via the resolved
//! [`EncodingScheme`](crate::binding::EncodingScheme)) and records the choice.

use crate::binding::profile::EncoderBindingProfile;
use crate::contracts::common::PluginRef;
use crate::contracts::ir_sample::IRSample;
use crate::error::TrainerError;

/// Selects and applies a FEAGI coder to encode samples into a runtime sensory frame.
///
/// `Frame` is the [`FeagiRuntime`](crate::binding::FeagiRuntime) `SensoryFrame` the produced
/// payload feeds into; keeping it associated decouples encoders from the concrete neuron
/// representation (forward-compatible with the NPU/quantization refactor).
pub trait EncoderPlugin {
    /// The runtime sensory-frame type this encoder produces.
    type Frame;

    /// Versioned identity of this encoder selector (recorded in provenance).
    fn plugin_ref(&self) -> PluginRef;

    /// Encodes one sample into a runtime sensory frame according to `profile`.
    fn encode(
        &mut self,
        sample: &IRSample,
        profile: &EncoderBindingProfile,
    ) -> Result<Self::Frame, TrainerError>;
}
