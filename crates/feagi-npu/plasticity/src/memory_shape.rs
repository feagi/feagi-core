//! Whether a stored memory neuron still fits its memory area's pattern shape.
//!
//! A memory neuron's pattern hash and replay frames are built from the area's temporal depth,
//! its episodic upstream set and its membrane-potential (MP) encoding mode. When one of these
//! changes, neurons learned under the old shape can no longer be matched or rebuilt. This is the
//! single predicate every layer uses to decide that: the runtime purge on an area update, the
//! save-side validator and the connectome loader.

use crate::mp_change_encoder::MemoryMpMode;

/// Shape-relevant view of one replay frame, independent of where the frame is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayFrameShape {
    /// Timestep offset inside the pattern window (0 = newest).
    pub offset: u32,
    /// Live cortical index of the upstream area the frame was taken from.
    pub upstream_area_idx: u32,
    /// True when the frame carries per-coordinate membrane potentials.
    pub has_membrane_potentials: bool,
}

/// The pattern shape of a memory area.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAreaShape {
    temporal_depth: u32,
    upstream_areas: Vec<u32>,
    mp_mode: MemoryMpMode,
}

impl MemoryAreaShape {
    /// Build a shape. A change encoding with `temporal_depth < 2` runs as pattern-only, exactly
    /// as the plasticity service registers it.
    pub fn new(temporal_depth: u32, mut upstream_areas: Vec<u32>, mp_mode: MemoryMpMode) -> Self {
        upstream_areas.sort_unstable();
        upstream_areas.dedup();
        let mp_mode = if mp_mode.change_encoding().is_some() && temporal_depth < 2 {
            MemoryMpMode::PatternOnly
        } else {
            mp_mode
        };
        Self {
            temporal_depth,
            upstream_areas,
            mp_mode,
        }
    }

    pub fn temporal_depth(&self) -> u32 {
        self.temporal_depth
    }

    pub fn upstream_areas(&self) -> &[u32] {
        &self.upstream_areas
    }

    pub fn mp_mode(&self) -> MemoryMpMode {
        self.mp_mode
    }

    /// True when `other` differs in a key that decides pattern identity: depth, upstream set or
    /// MP mode kind. Capacity, eviction, lifespan and quantization steps are not part of it.
    pub fn pattern_shape_differs(&self, other: &Self) -> bool {
        self.temporal_depth != other.temporal_depth
            || self.upstream_areas != other.upstream_areas
            || self.mp_mode.as_str() != other.mp_mode.as_str()
    }

    /// First reason the neuron's replay frames do not fit this shape, or `None` when they fit.
    /// A neuron with no frames cannot be judged and fits.
    pub fn mismatch(&self, frames: &[ReplayFrameShape]) -> Option<MemoryShapeMismatch> {
        for frame in frames {
            if frame.offset >= self.temporal_depth {
                return Some(MemoryShapeMismatch::OffsetExceedsDepth {
                    offset: frame.offset,
                    temporal_depth: self.temporal_depth,
                });
            }
            if self
                .upstream_areas
                .binary_search(&frame.upstream_area_idx)
                .is_err()
            {
                return Some(MemoryShapeMismatch::UpstreamNotConfigured {
                    upstream_area_idx: frame.upstream_area_idx,
                });
            }
        }
        if !frames.is_empty() {
            // Pattern-only and MP-learning areas share one identity (fired sets), so switching
            // between them keeps every neuron. A change encoding derives identity from MP
            // changes between frames, which frames without any MP cannot supply.
            let any_mp = frames.iter().any(|f| f.has_membrane_potentials);
            if self.mp_mode.change_encoding().is_some() && !any_mp {
                return Some(MemoryShapeMismatch::MpEncodingMismatch {
                    area_mode: self.mp_mode.as_str(),
                    frames_carry_mp: any_mp,
                });
            }
        }
        None
    }
}

/// Why a memory neuron does not fit its area's pattern shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryShapeMismatch {
    OffsetExceedsDepth {
        offset: u32,
        temporal_depth: u32,
    },
    UpstreamNotConfigured {
        upstream_area_idx: u32,
    },
    MpEncodingMismatch {
        area_mode: &'static str,
        frames_carry_mp: bool,
    },
}

impl MemoryShapeMismatch {
    /// Stable key for grouping and reporting counts.
    pub fn key(&self) -> &'static str {
        match self {
            Self::OffsetExceedsDepth { .. } => "replay_offset_exceeds_temporal_depth",
            Self::UpstreamNotConfigured { .. } => "replay_upstream_not_configured",
            Self::MpEncodingMismatch { .. } => "mp_encoding_mismatch",
        }
    }
}

impl core::fmt::Display for MemoryShapeMismatch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OffsetExceedsDepth {
                offset,
                temporal_depth,
            } => write!(
                f,
                "replay offset {offset} exceeds temporal depth {temporal_depth}"
            ),
            Self::UpstreamNotConfigured { upstream_area_idx } => write!(
                f,
                "replay upstream area {upstream_area_idx} is not configured"
            ),
            Self::MpEncodingMismatch {
                area_mode,
                frames_carry_mp,
            } => write!(
                f,
                "membrane-potential encoding '{area_mode}' does not match frames (carry MP: {frames_carry_mp})"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(offset: u32, upstream: u32, mp: bool) -> ReplayFrameShape {
        ReplayFrameShape {
            offset,
            upstream_area_idx: upstream,
            has_membrane_potentials: mp,
        }
    }

    #[test]
    fn offset_beyond_depth_is_a_mismatch() {
        let shape = MemoryAreaShape::new(1, vec![5], MemoryMpMode::PatternOnly);
        assert_eq!(shape.mismatch(&[frame(0, 5, false)]), None);
        assert_eq!(
            shape.mismatch(&[frame(0, 5, false), frame(1, 5, false)]),
            Some(MemoryShapeMismatch::OffsetExceedsDepth {
                offset: 1,
                temporal_depth: 1
            })
        );
    }

    #[test]
    fn unknown_upstream_and_mp_mismatch_are_detected() {
        let shape = MemoryAreaShape::new(2, vec![5], MemoryMpMode::PatternOnly);
        assert!(matches!(
            shape.mismatch(&[frame(0, 9, false)]),
            Some(MemoryShapeMismatch::UpstreamNotConfigured { .. })
        ));
        let enc = crate::MpChangeEncoding::Differential { quantization: 1.0 };
        let change = MemoryAreaShape::new(2, vec![5], MemoryMpMode::Change(enc));
        assert!(matches!(
            change.mismatch(&[frame(0, 5, false)]),
            Some(MemoryShapeMismatch::MpEncodingMismatch { .. })
        ));
        assert!(change.mismatch(&[frame(0, 5, true)]).is_none());
        let mp = MemoryAreaShape::new(2, vec![5], MemoryMpMode::MpLearning);
        assert!(mp.mismatch(&[frame(1, 5, false)]).is_none());
    }

    #[test]
    fn frameless_neuron_fits_and_shape_diff_ignores_non_pattern_keys() {
        let a = MemoryAreaShape::new(2, vec![5, 3], MemoryMpMode::PatternOnly);
        assert!(a.mismatch(&[]).is_none());
        let same = MemoryAreaShape::new(2, vec![3, 5], MemoryMpMode::PatternOnly);
        assert!(!a.pattern_shape_differs(&same));
        assert!(a.pattern_shape_differs(&MemoryAreaShape::new(
            1,
            vec![3, 5],
            MemoryMpMode::PatternOnly
        )));
    }
}
