//! Decoder axis — a *selector* over FEAGI's native coders (ADR-002, Appendix B.1).

use crate::binding::profile::DecoderBindingProfile;
use crate::contracts::common::PluginRef;
use crate::contracts::prediction_record::TypedPrediction;
use crate::error::TrainerError;

/// Selects and applies a FEAGI coder to decode a motor frame into a typed prediction.
pub trait DecoderPlugin {
    /// The runtime motor-frame type this decoder consumes.
    type Frame;

    /// Versioned identity of this decoder selector (recorded in provenance).
    fn plugin_ref(&self) -> PluginRef;

    /// Decodes one motor frame according to `profile`.
    fn decode(
        &mut self,
        motor: Self::Frame,
        profile: &DecoderBindingProfile,
    ) -> Result<TypedPrediction, TrainerError>;
}

/// Decodes one motor frame into one prediction per parallel X slot.
pub trait SlotDecoder {
    /// The runtime motor-frame type this decoder consumes.
    type Frame;

    /// Versioned identity of this decoder selector (recorded in provenance).
    fn plugin_ref(&self) -> PluginRef;

    /// Returns `slot_count` class predictions, one per X column of the Misc OPU.
    fn decode_slots(
        &mut self,
        motor: Self::Frame,
        profile: &DecoderBindingProfile,
        slot_count: u32,
    ) -> Result<Vec<TypedPrediction>, TrainerError>;
}
