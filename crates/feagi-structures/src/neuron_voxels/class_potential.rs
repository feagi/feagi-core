//! Class-id encoding for single-layer (`W×H×1`) class maps.
//!
//! Object segmentation input, object segmentation output, and classifier
//! detection twins carry one voxel per pixel. The class is the potential:
//! `(class_id + 1) / class_count`, always in `(0, 1]`. Zero means unlabeled,
//! so the Misc codec's `[-1, 1]` range and near-zero cutoff both hold.
//!
//! @cursor:ffi-safe - pure arithmetic, no allocation.

use crate::FeagiDataError;

/// Largest class count the encoding supports. Misc drops `|p| <= 1e-4`, so the
/// smallest encoded class `1 / class_count` must stay above that cutoff.
pub const MAX_CLASS_COUNT: u32 = 9_999;

/// Potential for `class_id` in a map of `class_count` classes.
pub fn encode_class_potential(class_id: u32, class_count: u32) -> Result<f32, FeagiDataError> {
    validate_class_count(class_count)?;
    if class_id >= class_count {
        return Err(FeagiDataError::BadParameters(format!(
            "class id {class_id} is outside class count {class_count}"
        )));
    }
    Ok((class_id + 1) as f32 / class_count as f32)
}

/// Class id carried by `potential`, or `None` when it names no class.
///
/// Rounds to the nearest class so values that picked up float error still decode.
pub fn decode_class_potential(potential: f32, class_count: u32) -> Option<u32> {
    if class_count == 0 || class_count > MAX_CLASS_COUNT || !potential.is_finite() {
        return None;
    }
    let level = (potential * class_count as f32).round();
    if level < 1.0 || level > class_count as f32 {
        return None;
    }
    Some(level as u32 - 1)
}

/// Class counts must be in `1..=MAX_CLASS_COUNT`.
pub fn validate_class_count(class_count: u32) -> Result<(), FeagiDataError> {
    if class_count == 0 || class_count > MAX_CLASS_COUNT {
        return Err(FeagiDataError::BadParameters(format!(
            "class count must be in 1..={MAX_CLASS_COUNT}, got {class_count}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_class_round_trips() {
        for class_count in [1_u32, 2, 19, 256, MAX_CLASS_COUNT] {
            for class_id in [0, class_count / 2, class_count - 1] {
                let potential = encode_class_potential(class_id, class_count).unwrap();
                assert!(potential > 0.0 && potential <= 1.0);
                assert_eq!(
                    decode_class_potential(potential, class_count),
                    Some(class_id)
                );
            }
        }
    }

    #[test]
    fn smallest_class_survives_misc_cutoff() {
        let potential = encode_class_potential(0, MAX_CLASS_COUNT).unwrap();
        assert!(potential > 1.0e-4);
    }

    #[test]
    fn zero_and_out_of_range_decode_to_none() {
        assert_eq!(decode_class_potential(0.0, 19), None);
        assert_eq!(decode_class_potential(1.2, 19), None);
        assert_eq!(decode_class_potential(-0.5, 19), None);
        assert_eq!(decode_class_potential(f32::NAN, 19), None);
        assert_eq!(decode_class_potential(0.5, 0), None);
    }

    #[test]
    fn small_drift_still_decodes() {
        let potential = encode_class_potential(13, 19).unwrap();
        assert_eq!(decode_class_potential(potential + 0.01, 19), Some(13));
        assert_eq!(decode_class_potential(potential - 0.01, 19), Some(13));
    }

    #[test]
    fn rejects_bad_inputs() {
        assert!(encode_class_potential(19, 19).is_err());
        assert!(encode_class_potential(0, 0).is_err());
        assert!(encode_class_potential(0, MAX_CLASS_COUNT + 1).is_err());
    }
}
