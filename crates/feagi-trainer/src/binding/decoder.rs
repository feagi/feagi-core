//! Decoder axis — a *selector* over FEAGI's native coders (ADR-002, Appendix B.1).
//!
//! A decoder turns a runtime motor frame into a [`TypedPrediction`]. Like the encoder it
//! implements no decoding math beyond selecting/configuring a FEAGI coder and mapping its
//! output into the typed contract.

use crate::binding::profile::DecoderBindingProfile;
use crate::contracts::common::PluginRef;
use crate::contracts::prediction_record::TypedPrediction;
use crate::error::TrainerError;

/// Selects and applies a FEAGI coder to decode a runtime motor frame into a prediction.
///
/// `Frame` is the [`FeagiRuntime`](crate::binding::FeagiRuntime) `MotorFrame` consumed.
pub trait DecoderPlugin {
    /// The runtime motor-frame type this decoder consumes.
    type Frame;

    /// Versioned identity of this decoder selector (recorded in provenance).
    fn plugin_ref(&self) -> PluginRef;

    /// Decodes one motor frame into a typed prediction according to `profile`.
    fn decode(
        &mut self,
        motor: Self::Frame,
        profile: &DecoderBindingProfile,
    ) -> Result<TypedPrediction, TrainerError>;
}
