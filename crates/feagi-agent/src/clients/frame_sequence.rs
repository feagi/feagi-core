//! Client-side helpers for the 16-bit burst id carried in every FEAGI frame header.
//!
//! FEAGI writes `burst_number & 0xFFFF` into the container header (bytes 1..3, little endian)
//! of each motor frame. A receiver that reads it per frame can tell a frame lost in transport
//! (gap in ids) from a frame that arrived and simply held no activity.

use feagi_serialization::FeagiByteContainer;

use crate::FeagiAgentError;

/// Read the 16-bit burst id from a received frame container.
///
/// Fails if the container does not hold a valid frame.
pub fn burst_id_of_frame(container: &FeagiByteContainer) -> Result<u16, FeagiAgentError> {
    Ok(container.get_increment_counter()?)
}

/// Result of observing the next frame id; see [`FrameSequenceTracker::observe`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameSequenceStep {
    /// First id seen by this tracker.
    First,
    /// Exactly the expected next id.
    Next,
    /// `missed` frames were skipped between the previous id and this one.
    Gap { missed: u16 },
    /// Same id as the previous frame.
    Repeated,
    /// An older id (more than half the id space behind). The tracker did not advance.
    OutOfOrder,
}

/// Tracks consecutive 16-bit frame ids with wraparound. Pure: no timing, no I/O.
#[derive(Debug, Clone, Copy, Default)]
pub struct FrameSequenceTracker {
    last: Option<u16>,
}

impl FrameSequenceTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Last id the tracker advanced to, if any.
    pub fn last_id(&self) -> Option<u16> {
        self.last
    }

    /// Record `id` and classify it against the previous one.
    pub fn observe(&mut self, id: u16) -> FrameSequenceStep {
        let Some(last) = self.last else {
            self.last = Some(id);
            return FrameSequenceStep::First;
        };
        let delta = id.wrapping_sub(last);
        match delta {
            0 => FrameSequenceStep::Repeated,
            1 => {
                self.last = Some(id);
                FrameSequenceStep::Next
            }
            2..=32767 => {
                self.last = Some(id);
                FrameSequenceStep::Gap { missed: delta - 1 }
            }
            _ => FrameSequenceStep::OutOfOrder,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

    fn frame_with_burst_number(burst_number: u32) -> FeagiByteContainer {
        // Same masking and encoder call the burst engine uses.
        let burst_id = (burst_number & 0xFFFF) as u16;
        let mut container = FeagiByteContainer::new_empty();
        container
            .overwrite_byte_data_with_single_struct_data(
                &CorticalMappedXYZPNeuronVoxels::new(),
                burst_id,
            )
            .unwrap();
        // Round trip through bytes like a receiver does.
        let mut received = FeagiByteContainer::new_empty();
        received
            .try_write_data_by_copy_and_verify(container.get_byte_ref())
            .unwrap();
        received
    }

    #[test]
    fn burst_id_of_frame_matches_low_16_bits_small() {
        assert_eq!(burst_id_of_frame(&frame_with_burst_number(5)).unwrap(), 5);
    }

    #[test]
    fn burst_id_of_frame_matches_low_16_bits_wrapped() {
        assert_eq!(
            burst_id_of_frame(&frame_with_burst_number(70000)).unwrap(),
            (70000u32 & 0xFFFF) as u16
        );
    }

    #[test]
    fn tracker_first_then_next() {
        let mut t = FrameSequenceTracker::new();
        assert_eq!(t.observe(10), FrameSequenceStep::First);
        assert_eq!(t.observe(11), FrameSequenceStep::Next);
        assert_eq!(t.observe(12), FrameSequenceStep::Next);
    }

    #[test]
    fn tracker_reports_gaps() {
        let mut t = FrameSequenceTracker::new();
        t.observe(10);
        assert_eq!(t.observe(12), FrameSequenceStep::Gap { missed: 1 });
    }

    #[test]
    fn tracker_wraps_next_and_gap() {
        let mut t = FrameSequenceTracker::new();
        t.observe(65535);
        assert_eq!(t.observe(0), FrameSequenceStep::Next);
        let mut t = FrameSequenceTracker::new();
        t.observe(65534);
        assert_eq!(t.observe(1), FrameSequenceStep::Gap { missed: 2 });
    }

    #[test]
    fn tracker_repeated_and_out_of_order_do_not_advance() {
        let mut t = FrameSequenceTracker::new();
        t.observe(100);
        assert_eq!(t.observe(100), FrameSequenceStep::Repeated);
        assert_eq!(t.observe(90), FrameSequenceStep::OutOfOrder);
        assert_eq!(t.last_id(), Some(100));
        assert_eq!(t.observe(101), FrameSequenceStep::Next);
    }
}
