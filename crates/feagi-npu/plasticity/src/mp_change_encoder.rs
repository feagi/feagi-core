// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Change-based membrane potential encoding for episodic memory areas.
//!
//! In a change mode, a memory neuron stands for how upstream membrane
//! potentials moved across the temporal window, not for their absolute values.
//! A window of depth D yields D-1 steps; each step pairs a neuron's MP in one
//! frame with its MP in the next and reduces the pair to an integer bucket.
//!
//! - Differential: `round((current - prior) / quantization)`. A neuron missing
//!   from a frame counts as MP 0, so starting or stopping to fire is a change.
//! - Ratio: `round(ln(current / prior) / ln(1 + quantization% / 100))`, i.e.
//!   compounding buckets, so doubling and halving are equally far from 0.
//!   Only steps where both MPs are positive take part.
//!
//! @cursor:critical-path - runs once per memory area per burst.
//! Math goes through `libm` so buckets are identical on every platform and in
//! `no_std` builds; the standard library defers `ln` to the platform libm.

use ahash::AHashMap;
use xxhash_rust::xxh64::xxh64;

/// Fired-neuron membrane potentials for one upstream area in one frame.
pub type MpFrame = AHashMap<u32, f32>;

/// Bucketed changes for one upstream area in one step, sorted by neuron id.
pub type ChangeStep = Vec<(u32, i64)>;

/// Hash domain tag for differential patterns (fired-set patterns use no tag).
const DIFFERENTIAL_HASH_TAG: u8 = 1;
/// Hash domain tag for ratio patterns.
const RATIO_HASH_TAG: u8 = 2;

/// How a memory area uses upstream membrane potentials.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryMpMode {
    /// Identity from fired sets; replay force-fires stored coordinates.
    PatternOnly,
    /// Identity from fired sets; replay frames carry per-coordinate MPs.
    MpLearning,
    /// Identity from MP changes between consecutive frames; replay is disabled.
    Change(MpChangeEncoding),
}

impl MemoryMpMode {
    /// Stable name for API reporting: "pattern_only", "mp_learning", "mp_differential", "mp_ratio".
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryMpMode::PatternOnly => "pattern_only",
            MemoryMpMode::MpLearning => "mp_learning",
            MemoryMpMode::Change(MpChangeEncoding::Differential { .. }) => "mp_differential",
            MemoryMpMode::Change(MpChangeEncoding::Ratio { .. }) => "mp_ratio",
        }
    }

    /// True when upstream areas must archive membrane potentials.
    pub fn requires_mp_archival(&self) -> bool {
        !matches!(self, MemoryMpMode::PatternOnly)
    }

    /// Change encoding, when the mode is change-based.
    pub fn change_encoding(&self) -> Option<MpChangeEncoding> {
        match self {
            MemoryMpMode::Change(encoding) => Some(*encoding),
            _ => None,
        }
    }
}

/// Change-based encoding and its rounding granularity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MpChangeEncoding {
    /// Quantized MP difference; `quantization` is in membrane potential units.
    Differential { quantization: f32 },
    /// Compounding percentage change; `quantization_percent` is the growth per bucket.
    Ratio { quantization_percent: f32 },
}

impl MpChangeEncoding {
    /// Reject non-finite or non-positive quantization levels.
    pub fn validate(&self) -> Result<(), String> {
        let (name, q) = match self {
            MpChangeEncoding::Differential { quantization } => {
                ("mp_delta_quantization", *quantization)
            }
            MpChangeEncoding::Ratio {
                quantization_percent,
            } => ("mp_ratio_quantization", *quantization_percent),
        };
        if q.is_finite() && q > 0.0 {
            Ok(())
        } else {
            Err(format!(
                "{name} must be a finite number greater than 0, got {q}"
            ))
        }
    }

    /// Bucket for one neuron across one step, or `None` if it does not take part.
    ///
    /// `None` MPs mean the neuron did not fire in that frame.
    pub fn bucket(&self, prior: Option<f32>, current: Option<f32>) -> Option<i64> {
        match self {
            MpChangeEncoding::Differential { quantization } => {
                if prior.is_none() && current.is_none() {
                    return None;
                }
                let delta = f64::from(current.unwrap_or(0.0)) - f64::from(prior.unwrap_or(0.0));
                Some(libm::round(delta / f64::from(*quantization)) as i64)
            }
            MpChangeEncoding::Ratio {
                quantization_percent,
            } => {
                let (p, c) = (prior?, current?);
                if p <= 0.0 || c <= 0.0 {
                    return None;
                }
                let step = libm::log1p(f64::from(*quantization_percent) / 100.0);
                let ratio = f64::from(c) / f64::from(p);
                Some(libm::round(libm::log(ratio) / step) as i64)
            }
        }
    }

    /// Encode one step of one upstream area into sorted `(neuron_id, bucket)` pairs.
    pub fn encode_step(&self, prior: &MpFrame, current: &MpFrame) -> ChangeStep {
        let mut ids: Vec<u32> = prior.keys().chain(current.keys()).copied().collect();
        ids.sort_unstable();
        ids.dedup();
        ids.into_iter()
            .filter_map(|id| {
                self.bucket(prior.get(&id).copied(), current.get(&id).copied())
                    .map(|b| (id, b))
            })
            .collect()
    }

    fn hash_tag(&self) -> u8 {
        match self {
            MpChangeEncoding::Differential { .. } => DIFFERENTIAL_HASH_TAG,
            MpChangeEncoding::Ratio { .. } => RATIO_HASH_TAG,
        }
    }

    /// Deterministic xxHash64 of an encoded window.
    ///
    /// `steps` are ordered oldest to newest, and within a step by upstream area
    /// in sorted order. Empty steps are kept so the step position stays part of
    /// the identity. A mode tag keeps differential, ratio, and fired-set hashes
    /// apart in the shared pattern map.
    pub fn pattern_hash(&self, steps: &[ChangeStep]) -> u64 {
        let mut buffer = vec![self.hash_tag()];
        for step in steps {
            buffer.extend_from_slice(&(step.len() as u32).to_le_bytes());
            for (id, bucket) in step {
                buffer.extend_from_slice(&id.to_le_bytes());
                buffer.extend_from_slice(&bucket.to_le_bytes());
            }
        }
        xxh64(&buffer, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(entries: &[(u32, f32)]) -> MpFrame {
        entries.iter().copied().collect()
    }

    const DIFF: MpChangeEncoding = MpChangeEncoding::Differential { quantization: 1.0 };
    const RATIO: MpChangeEncoding = MpChangeEncoding::Ratio {
        quantization_percent: 20.0,
    };

    #[test]
    fn mode_names_are_distinct_and_stable() {
        assert_eq!(MemoryMpMode::PatternOnly.as_str(), "pattern_only");
        assert_eq!(MemoryMpMode::MpLearning.as_str(), "mp_learning");
        assert_eq!(MemoryMpMode::Change(DIFF).as_str(), "mp_differential");
        assert_eq!(MemoryMpMode::Change(RATIO).as_str(), "mp_ratio");
    }

    #[test]
    fn differential_ignores_absolute_level() {
        let a = DIFF.encode_step(&frame(&[(1, 2.0)]), &frame(&[(1, 4.0)]));
        let b = DIFF.encode_step(&frame(&[(1, 7.0)]), &frame(&[(1, 9.0)]));
        assert_eq!(a, vec![(1, 2)]);
        assert_eq!(DIFF.pattern_hash(&[a]), DIFF.pattern_hash(&[b]));
    }

    #[test]
    fn differential_distinguishes_direction_and_size() {
        let up = DIFF.encode_step(&frame(&[(1, 2.0)]), &frame(&[(1, 4.0)]));
        let down = DIFF.encode_step(&frame(&[(1, 4.0)]), &frame(&[(1, 2.0)]));
        let smaller = DIFF.encode_step(&frame(&[(1, 2.0)]), &frame(&[(1, 3.0)]));
        let h = |s: ChangeStep| DIFF.pattern_hash(&[s]);
        assert_ne!(h(up.clone()), h(down));
        assert_ne!(h(up), h(smaller));
    }

    #[test]
    fn differential_missing_neuron_counts_as_zero() {
        assert_eq!(DIFF.bucket(None, Some(3.0)), Some(3));
        assert_eq!(DIFF.bucket(Some(3.0), None), Some(-3));
        assert_eq!(DIFF.bucket(None, None), None);
        let step = DIFF.encode_step(&frame(&[(1, 2.0)]), &frame(&[(2, 5.0)]));
        assert_eq!(step, vec![(1, -2), (2, 5)]);
    }

    #[test]
    fn differential_rounds_to_nearest_level() {
        let q = MpChangeEncoding::Differential { quantization: 0.5 };
        assert_eq!(q.bucket(Some(1.0), Some(1.7)), Some(1));
        assert_eq!(q.bucket(Some(1.0), Some(1.8)), Some(2));
        assert_eq!(q.bucket(Some(1.0), Some(1.0)), Some(0));
        assert_eq!(q.bucket(Some(1.0), Some(0.2)), Some(-2));
    }

    #[test]
    fn ratio_ignores_scale() {
        let a = RATIO.encode_step(&frame(&[(1, 2.0)]), &frame(&[(1, 4.0)]));
        let b = RATIO.encode_step(&frame(&[(1, 7.0)]), &frame(&[(1, 14.0)]));
        assert_eq!(a, b);
        let c = RATIO.encode_step(&frame(&[(1, 7.0)]), &frame(&[(1, 9.0)]));
        assert_ne!(a, c);
    }

    #[test]
    fn ratio_buckets_compound_symmetrically() {
        // ln(2) / ln(1.2) = 3.80..., so doubling and halving are 4 buckets from 0.
        assert_eq!(RATIO.bucket(Some(2.0), Some(4.0)), Some(4));
        assert_eq!(RATIO.bucket(Some(4.0), Some(2.0)), Some(-4));
        assert_eq!(RATIO.bucket(Some(5.0), Some(6.0)), Some(1));
        assert_eq!(RATIO.bucket(Some(5.0), Some(5.0)), Some(0));
    }

    #[test]
    fn ratio_only_tracks_positive_to_positive() {
        assert_eq!(RATIO.bucket(None, Some(3.0)), None);
        assert_eq!(RATIO.bucket(Some(3.0), None), None);
        assert_eq!(RATIO.bucket(Some(0.0), Some(3.0)), None);
        assert_eq!(RATIO.bucket(Some(-1.0), Some(3.0)), None);
        assert_eq!(RATIO.bucket(Some(3.0), Some(-1.0)), None);
        let step = RATIO.encode_step(&frame(&[(1, 2.0), (2, 3.0)]), &frame(&[(1, 4.0)]));
        assert_eq!(step, vec![(1, 4)]);
    }

    #[test]
    fn empty_step_position_is_part_of_identity() {
        let change = DIFF.encode_step(&frame(&[(1, 2.0)]), &frame(&[(1, 4.0)]));
        let early = [change.clone(), Vec::new()];
        let late = [Vec::new(), change];
        assert_ne!(DIFF.pattern_hash(&early), DIFF.pattern_hash(&late));
    }

    #[test]
    fn modes_do_not_share_hashes() {
        let steps = [vec![(1u32, 4i64)]];
        assert_ne!(DIFF.pattern_hash(&steps), RATIO.pattern_hash(&steps));
    }

    #[test]
    fn hashing_is_deterministic_and_order_independent() {
        let a = DIFF.encode_step(
            &frame(&[(9, 1.0), (3, 2.0), (5, 3.0)]),
            &frame(&[(5, 1.0), (9, 4.0), (3, 2.0)]),
        );
        let b = DIFF.encode_step(
            &frame(&[(3, 2.0), (5, 3.0), (9, 1.0)]),
            &frame(&[(3, 2.0), (9, 4.0), (5, 1.0)]),
        );
        assert_eq!(a, vec![(3, 0), (5, -2), (9, 3)]);
        assert_eq!(DIFF.pattern_hash(&[a]), DIFF.pattern_hash(&[b]));
    }

    #[test]
    fn validate_rejects_bad_quantization() {
        for q in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(MpChangeEncoding::Differential { quantization: q }
                .validate()
                .is_err());
            assert!(MpChangeEncoding::Ratio {
                quantization_percent: q
            }
            .validate()
            .is_err());
        }
        assert!(DIFF.validate().is_ok());
        assert!(RATIO.validate().is_ok());
    }

    #[test]
    fn mode_helpers() {
        assert!(!MemoryMpMode::PatternOnly.requires_mp_archival());
        assert!(MemoryMpMode::MpLearning.requires_mp_archival());
        assert!(MemoryMpMode::Change(DIFF).requires_mp_archival());
        assert_eq!(MemoryMpMode::Change(RATIO).change_encoding(), Some(RATIO));
        assert_eq!(MemoryMpMode::MpLearning.change_encoding(), None);
    }
}
