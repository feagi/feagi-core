//! Ordered ingest for sensory streams where every frame must get its own burst.
//!
//! The burst loop keeps only the newest packet per source each burst. That is
//! right for vision and other state-like inputs, where an older frame is stale.
//! Audio is different: each frame is 1/burst_rate seconds of sound. When a
//! sender and FEAGI both run at the burst rate on separate clocks, jitter near
//! the drain boundary lands two audio frames in one burst and none in the next.
//! Newest-wins then drops a slice of audio every time.
//!
//! Audio input areas (`iaud`) are ordered by unit type: their frames queue per
//! (source, area) in arrival order and one frame leaves each queue per burst.
//! Every other area keeps newest-wins.
//!
//! @cursor:critical-path Runs once per burst inside the burst loop thread.

use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::SensoryCorticalUnit;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Voxels of one cortical area: (x, y, z, potential).
pub type AreaXyzp = Vec<(u32, u32, u32, f32)>;
/// Areas decoded from one sensory payload.
pub type DecodedAreas = Vec<(CorticalID, AreaXyzp)>;

/// Source key for payloads that carry no agent id.
const UNKNOWN_SOURCE: &str = "";

/// True for input areas whose frames must each get their own burst.
///
/// Decided from the cortical id alone: direction byte `i` plus the audio unit code.
pub fn is_sequential_area(cortical_id: &CorticalID) -> bool {
    let bytes = cortical_id.as_bytes();
    bytes[0] == b'i'
        && bytes[1..4] == SensoryCorticalUnit::AudioInput.get_cortical_id_unit_reference()
}

/// Per-(source, area) FIFO queues for ordered areas.
#[derive(Debug)]
pub struct SequentialIngest {
    max_frames: usize,
    /// Keyed by (source id, cortical id bytes) so iteration order is deterministic.
    queues: BTreeMap<(String, [u8; 8]), (CorticalID, VecDeque<AreaXyzp>)>,
    /// Sources seen sending an ordered area. Every payload they send is decoded.
    ordered_sources: BTreeSet<String>,
}

impl SequentialIngest {
    /// `max_frames` bounds each queue. A full queue drops its oldest frame.
    ///
    /// # Errors
    ///
    /// Returns an error when `max_frames` is 0.
    pub fn new(max_frames: usize) -> Result<Self, String> {
        if max_frames == 0 {
            return Err("burst_engine.sequential_ingest_max_frames must be at least 1".to_string());
        }
        Ok(Self {
            max_frames,
            queues: BTreeMap::new(),
            ordered_sources: BTreeSet::new(),
        })
    }

    /// True when `source` has sent an ordered area before, so none of its payloads
    /// may be collapsed to the newest one before decoding.
    pub fn carries_ordered_areas(&self, source: Option<&str>) -> bool {
        self.ordered_sources
            .contains(source.unwrap_or(UNKNOWN_SOURCE))
    }

    /// Move ordered areas out of `decoded` into their queues, in call order.
    ///
    /// Returns how many queued frames were dropped because a queue was full.
    pub fn absorb(&mut self, source: Option<&str>, decoded: &mut DecodedAreas) -> usize {
        let source_key = source.unwrap_or(UNKNOWN_SOURCE);
        let mut dropped = 0usize;
        let mut saw_ordered = false;
        let mut index = 0;
        while index < decoded.len() {
            if !is_sequential_area(&decoded[index].0) {
                index += 1;
                continue;
            }
            saw_ordered = true;
            let (cortical_id, xyzp) = decoded.swap_remove(index);
            let (_, queue) = self
                .queues
                .entry((source_key.to_string(), *cortical_id.as_bytes()))
                .or_insert_with(|| (cortical_id, VecDeque::new()));
            while queue.len() >= self.max_frames {
                queue.pop_front();
                dropped += 1;
            }
            queue.push_back(xyzp);
        }
        if saw_ordered {
            self.ordered_sources.insert(source_key.to_string());
        }
        dropped
    }

    /// The oldest frame from every non-empty queue: one frame per (source, area) this burst.
    pub fn next_burst(&mut self) -> DecodedAreas {
        let mut out = DecodedAreas::new();
        self.queues.retain(|_, (cortical_id, queue)| {
            if let Some(xyzp) = queue.pop_front() {
                out.push((*cortical_id, xyzp));
            }
            !queue.is_empty()
        });
        out
    }

    /// Frames waiting across all queues.
    pub fn queued_frames(&self) -> usize {
        self.queues.values().map(|(_, queue)| queue.len()).sum()
    }

    /// Drop every queued frame and forget ordered sources (genome reload, shutdown).
    pub fn clear(&mut self) {
        self.queues.clear();
        self.ordered_sources.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::FrameChangeHandling;

    fn audio_input(unit: u16) -> CorticalID {
        SensoryCorticalUnit::get_cortical_ids_array_for_audio_input_with_parameters(
            FrameChangeHandling::Absolute,
            CorticalUnitIndex::from(unit),
        )[0]
    }

    fn other_input() -> CorticalID {
        CorticalID::try_from_base_64("aXRlbgoAAAA=").expect("text input id")
    }

    fn frame(tag: u32) -> AreaXyzp {
        vec![(tag, 0, 0, 0.5)]
    }

    #[test]
    fn audio_input_is_ordered_and_other_inputs_are_not() {
        assert!(is_sequential_area(&audio_input(0)));
        assert!(is_sequential_area(&audio_input(1)));
        assert!(!is_sequential_area(&other_input()));
        let audio_output = CorticalID::try_from_base_64("b2F1ZAoAAAA=").expect("audio output id");
        assert!(!is_sequential_area(&audio_output));
    }

    #[test]
    fn two_frames_in_one_burst_leave_on_two_bursts_in_order() {
        let mut ingest = SequentialIngest::new(8).expect("valid depth");
        let area = audio_input(0);
        let mut first = vec![(area, frame(1))];
        let mut second = vec![(area, frame(2))];
        assert_eq!(ingest.absorb(Some("agent"), &mut first), 0);
        assert_eq!(ingest.absorb(Some("agent"), &mut second), 0);
        assert!(first.is_empty() && second.is_empty());

        assert_eq!(ingest.next_burst(), vec![(area, frame(1))]);
        assert_eq!(ingest.next_burst(), vec![(area, frame(2))]);
        assert!(ingest.next_burst().is_empty());
        assert_eq!(ingest.queued_frames(), 0);
    }

    #[test]
    fn non_ordered_areas_stay_in_the_payload() {
        let mut ingest = SequentialIngest::new(8).expect("valid depth");
        let mut decoded = vec![(other_input(), frame(7)), (audio_input(0), frame(1))];
        ingest.absorb(Some("agent"), &mut decoded);
        assert_eq!(decoded, vec![(other_input(), frame(7))]);
    }

    #[test]
    fn a_full_queue_drops_its_oldest_frame() {
        let mut ingest = SequentialIngest::new(2).expect("valid depth");
        let area = audio_input(0);
        for tag in 1..=3 {
            let mut decoded = vec![(area, frame(tag))];
            let dropped = ingest.absorb(Some("agent"), &mut decoded);
            assert_eq!(dropped, usize::from(tag == 3));
        }
        assert_eq!(ingest.next_burst(), vec![(area, frame(2))]);
        assert_eq!(ingest.next_burst(), vec![(area, frame(3))]);
    }

    #[test]
    fn sources_and_channels_queue_separately() {
        let mut ingest = SequentialIngest::new(8).expect("valid depth");
        let left = audio_input(0);
        let right = audio_input(1);
        ingest.absorb(Some("a"), &mut vec![(left, frame(1)), (right, frame(2))]);
        ingest.absorb(Some("b"), &mut vec![(left, frame(3))]);
        let mut burst = ingest.next_burst();
        burst.sort_by_key(|(_, xyzp)| xyzp[0].0);
        assert_eq!(
            burst,
            vec![(left, frame(1)), (right, frame(2)), (left, frame(3))]
        );
    }

    #[test]
    fn a_source_is_remembered_once_it_sends_audio() {
        let mut ingest = SequentialIngest::new(8).expect("valid depth");
        assert!(!ingest.carries_ordered_areas(Some("agent")));
        ingest.absorb(Some("agent"), &mut vec![(audio_input(0), frame(1))]);
        assert!(ingest.carries_ordered_areas(Some("agent")));
        assert!(!ingest.carries_ordered_areas(Some("camera")));
        ingest.clear();
        assert!(!ingest.carries_ordered_areas(Some("agent")));
        assert_eq!(ingest.queued_frames(), 0);
    }

    #[test]
    fn zero_depth_is_rejected() {
        assert!(SequentialIngest::new(0).is_err());
    }
}
