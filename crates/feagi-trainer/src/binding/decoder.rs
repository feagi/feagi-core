//! Decoder axis — a *selector* over FEAGI's native coders (ADR-002, Appendix B.1).

use crate::binding::profile::DecoderBindingProfile;
use crate::contracts::common::PluginRef;
use crate::contracts::prediction_record::{ClassReadout, TypedPrediction};
use crate::error::TrainerError;

/// The readout of a class score vector: the highest score, which must be above 0.
///
/// No score above 0 (or an empty slice) is [`ClassReadout::NoAnswer`]. A top score shared by two
/// or more classes (exact `f64` equality) is [`ClassReadout::Tied`]; otherwise the winning class.
pub fn class_from_scores(scores: &[f64]) -> ClassReadout {
    let mut best: Option<(usize, f64)> = None;
    let mut tied = false;
    for (index, &score) in scores.iter().enumerate() {
        if score <= 0.0 {
            continue;
        }
        match best {
            Some((_, top)) if score == top => tied = true,
            Some((_, top)) if score < top => {}
            _ => {
                best = Some((index, score));
                tied = false;
            }
        }
    }
    match best {
        None => ClassReadout::NoAnswer,
        Some(_) if tied => ClassReadout::Tied,
        Some((index, _)) => ClassReadout::Class(index as u32),
    }
}

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

#[cfg(test)]
mod tests {
    use super::class_from_scores;
    use crate::contracts::prediction_record::ClassReadout;

    #[test]
    fn ties_for_the_top_score_are_tied() {
        assert_eq!(class_from_scores(&[0.5, 0.5, 0.0]), ClassReadout::Tied);
        assert_eq!(class_from_scores(&[0.0, 0.2, 0.7, 0.7]), ClassReadout::Tied);
    }

    #[test]
    fn a_tie_below_the_top_score_is_not_a_tie() {
        assert_eq!(class_from_scores(&[0.2, 0.2, 0.7]), ClassReadout::Class(2));
        assert_eq!(class_from_scores(&[0.2, 0.7, 0.3]), ClassReadout::Class(1));
    }

    #[test]
    fn silent_or_negative_scores_are_no_answer() {
        assert_eq!(class_from_scores(&[0.0, 0.0, 0.0]), ClassReadout::NoAnswer);
        assert_eq!(class_from_scores(&[-1.0, 0.0]), ClassReadout::NoAnswer);
        assert_eq!(class_from_scores(&[]), ClassReadout::NoAnswer);
    }
}
