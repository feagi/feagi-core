//! Sensor cadence hold: re-inject an IPU's previous frame for exactly one burst when no
//! new frame arrived for that area.
//!
//! Sensory injection is per burst: `inject_sensory_xyzp_by_id` stages fire candidates
//! that Phase 1 drains into the FCL for the current burst only, so an input area that
//! receives no payload in a burst has no candidates and stays silent. A controller
//! streaming a scalar sensor slightly slower than the burst rate therefore shows gaps
//! every other burst. For areas with `sensory_hold_last_frame` on, [`SensoryHold`]
//! bridges a single missing burst with the frame from the immediately preceding one;
//! a longer gap goes silent on the burst after.

use feagi_structures::genomic::cortical_area::CorticalID;

/// XYZP list for one cortical area as decoded from the sensory wire format.
pub type HeldXyzp = Vec<(u32, u32, u32, f32)>;

/// One decoded sensory batch: `(cortical_id, xyzp list)` per area.
pub type SensoryBatch = Vec<(CorticalID, HeldXyzp)>;

#[derive(Debug, Default)]
pub struct SensoryHold {
    /// Areas with `sensory_hold_last_frame` enabled (pushed from the services layer).
    enabled: ahash::AHashSet<CorticalID>,
    /// Last non-empty frame per enabled area: `(burst_num it was injected at, xyzp)`.
    last_frame: ahash::AHashMap<CorticalID, (u64, HeldXyzp)>,
}

impl SensoryHold {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the set of hold-enabled areas. Stored frames of areas no longer enabled
    /// are dropped.
    pub fn set_enabled(&mut self, enabled: ahash::AHashSet<CorticalID>) {
        self.last_frame.retain(|id, _| enabled.contains(id));
        self.enabled = enabled;
    }

    pub fn is_enabled(&self, cortical_id: &CorticalID) -> bool {
        self.enabled.contains(cortical_id)
    }

    pub fn enabled_len(&self) -> usize {
        self.enabled.len()
    }

    /// Record this burst's frames for enabled areas and append a held frame for every
    /// enabled area that got nothing this burst but had a frame at `burst_num - 1`.
    ///
    /// Returns the number of areas whose frame was re-injected.
    pub fn apply(&mut self, burst_num: u64, batches: &mut Vec<SensoryBatch>) -> usize {
        if self.enabled.is_empty() {
            return 0;
        }

        // Remember the newest non-empty frame per enabled area seen this burst.
        for batch in batches.iter() {
            for (cortical_id, xyzp) in batch {
                if xyzp.is_empty() || !self.enabled.contains(cortical_id) {
                    continue;
                }
                self.last_frame
                    .insert(*cortical_id, (burst_num, xyzp.clone()));
            }
        }

        // Re-inject frames from exactly the preceding burst for areas silent this burst.
        let mut held: SensoryBatch = Vec::new();
        for (cortical_id, (frame_burst, xyzp)) in self.last_frame.iter() {
            if *frame_burst + 1 == burst_num {
                held.push((*cortical_id, xyzp.clone()));
            }
        }
        let count = held.len();
        if count > 0 {
            batches.push(held);
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(raw: &[u8; 8]) -> CorticalID {
        CorticalID::try_from_bytes(raw).unwrap()
    }

    fn frame(cortical_id: CorticalID, p: f32) -> SensoryBatch {
        vec![(cortical_id, vec![(0, 0, 0, p)])]
    }

    fn frames_for(batches: &[SensoryBatch], cortical_id: &CorticalID) -> Vec<f32> {
        batches
            .iter()
            .flatten()
            .filter(|(cid, _)| cid == cortical_id)
            .map(|(_, xyzp)| xyzp[0].3)
            .collect()
    }

    fn hold_for(ids: &[CorticalID]) -> SensoryHold {
        let mut hold = SensoryHold::new();
        hold.set_enabled(ids.iter().copied().collect());
        hold
    }

    #[test]
    fn frame_at_n_is_reinjected_at_n_plus_1_only() {
        let imu = id(b"iHLD0001");
        let mut hold = hold_for(&[imu]);

        let mut b10 = vec![frame(imu, 0.7)];
        assert_eq!(hold.apply(10, &mut b10), 0);
        assert_eq!(frames_for(&b10, &imu), vec![0.7]);

        let mut b11: Vec<SensoryBatch> = Vec::new();
        assert_eq!(hold.apply(11, &mut b11), 1);
        assert_eq!(frames_for(&b11, &imu), vec![0.7]);

        let mut b12: Vec<SensoryBatch> = Vec::new();
        assert_eq!(hold.apply(12, &mut b12), 0);
        assert!(b12.is_empty());
    }

    #[test]
    fn new_frame_replaces_old_one() {
        let imu = id(b"iHLD0001");
        let mut hold = hold_for(&[imu]);

        let mut b1 = vec![frame(imu, 0.2)];
        hold.apply(1, &mut b1);
        let mut b2 = vec![frame(imu, 0.9)];
        assert_eq!(hold.apply(2, &mut b2), 0);
        assert_eq!(frames_for(&b2, &imu), vec![0.9]);

        let mut b3: Vec<SensoryBatch> = Vec::new();
        hold.apply(3, &mut b3);
        assert_eq!(frames_for(&b3, &imu), vec![0.9]);
    }

    #[test]
    fn option_off_never_reinjects_and_never_stores() {
        let imu = id(b"iHLD0001");
        let cam = id(b"iHLD0002");
        let mut hold = hold_for(&[imu]);

        let mut b1 = vec![frame(cam, 0.5)];
        hold.apply(1, &mut b1);
        assert!(hold.last_frame.is_empty());
        let mut b2: Vec<SensoryBatch> = Vec::new();
        assert_eq!(hold.apply(2, &mut b2), 0);
        assert!(b2.is_empty());

        let mut none = SensoryHold::new();
        let mut b3 = vec![frame(imu, 0.5)];
        none.apply(3, &mut b3);
        let mut b4: Vec<SensoryBatch> = Vec::new();
        assert_eq!(none.apply(4, &mut b4), 0);
        assert!(none.last_frame.is_empty());
    }

    #[test]
    fn disabling_an_area_drops_its_stored_frame() {
        let imu = id(b"iHLD0001");
        let mut hold = hold_for(&[imu]);
        let mut b1 = vec![frame(imu, 0.3)];
        hold.apply(1, &mut b1);
        hold.set_enabled(ahash::AHashSet::new());
        let mut b2: Vec<SensoryBatch> = Vec::new();
        assert_eq!(hold.apply(2, &mut b2), 0);
        assert!(b2.is_empty());
    }

    #[test]
    fn empty_xyzp_is_not_a_frame() {
        let imu = id(b"iHLD0001");
        let mut hold = hold_for(&[imu]);
        let mut b1 = vec![vec![(imu, Vec::new())]];
        hold.apply(1, &mut b1);
        let mut b2: Vec<SensoryBatch> = Vec::new();
        assert_eq!(hold.apply(2, &mut b2), 0);
    }
}
