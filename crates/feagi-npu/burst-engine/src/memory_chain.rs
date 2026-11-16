// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Chain replay state of a `sequential_memory` loop: lockout, chain head, visited set and the
//! ordered successor table.
//!
//! Everything here is transient or derived (it is never persisted):
//!
//! - **Lockout and chain** (`chains`): while a chain replay runs in an area, no other replay
//!   starts in it. A chain is started only by a recall fire of a state that has a successor.
//!   Ordinary window echoes never start one.
//! - **Successor table** (`order`): per source state, its successors best first. The plasticity
//!   service ranks them (it owns the neuron columns) and stamps the order here; the NPU only
//!   reads it. A successor missing from the table ranks last; a successor whose link is gone is
//!   never offered, because the live synapses are the authority on which links exist.
//!
//! Nothing runs for an area with no loop and no recording: `engaged` is false for a brain that
//! has neither, and every per-burst check starts with it.

use ahash::{AHashMap, AHashSet};

/// One running chain replay in one memory area.
#[derive(Debug, Clone)]
pub(crate) struct ChainRun {
    /// Burst the cue fired at.
    pub start_burst: u64,
    /// Longest the replay may run, in bursts.
    pub cap_bursts: u64,
    /// The state that fired last.
    pub head: u32,
    /// The successor chosen at `head`, awaited through the flagged link.
    pub next: Option<u32>,
    /// Last burst the awaited arrival can land at; a chain whose arrival is lost ends after it.
    pub deadline: u64,
    /// Every state that played (each plays at most once per replay).
    pub visited: AHashSet<u32>,
    /// A chain started from an episode neuron follows its member list exactly (the list bounds
    /// it, so it is exempt from `visited`). `None` for a chain started from a state.
    pub episode: Option<Vec<u32>>,
    /// Position in `episode` of the member awaited next.
    pub pos: usize,
    /// Q1 candidates of a state-cued chain: `(episode neuron, index of the member that played
    /// last)`. Each is advanced by one compare when the head moves and dropped when its next
    /// member is not the new head. They never steer the path (D12 picks it).
    pub candidates: Vec<(u32, usize)>,
}

/// Open the Q1 candidates of episodes that begin at the state `head`. A one-state episode is
/// complete at once and goes to `completed`; a longer one becomes a live candidate.
pub(crate) fn open_candidates(
    first_index: &AHashMap<u32, Vec<u32>>,
    episodes: &AHashMap<u32, Vec<u32>>,
    head: u32,
    candidates: &mut Vec<(u32, usize)>,
    completed: &mut Vec<u32>,
) {
    let Some(ids) = first_index.get(&head) else {
        return;
    };
    for e in ids {
        match episodes.get(e).map(Vec::len) {
            Some(1) => completed.push(*e),
            Some(n) if n > 1 => candidates.push((*e, 0)),
            _ => {}
        }
    }
}

/// Move every candidate to the new `head`: one compare each. A candidate whose next member is
/// `head` advances (and completes at its last member); any other drops.
pub(crate) fn advance_candidates(
    episodes: &AHashMap<u32, Vec<u32>>,
    head: u32,
    candidates: &mut Vec<(u32, usize)>,
    completed: &mut Vec<u32>,
) {
    candidates.retain_mut(|(e, pos)| match episodes.get(e) {
        Some(list) if list.get(*pos + 1) == Some(&head) => {
            *pos += 1;
            if *pos + 1 == list.len() {
                completed.push(*e);
                false
            } else {
                true
            }
        }
        _ => false,
    });
}

/// Position of `target` in the ranked successors of `source` (lower is stronger); a successor
/// missing from the table ranks last.
pub(crate) fn rank_in(order: &AHashMap<u32, Vec<u32>>, source: u32, target: u32) -> usize {
    order
        .get(&source)
        .and_then(|ranked| ranked.iter().position(|t| *t == target))
        .unwrap_or(usize::MAX)
}

/// Replay-chain state of the NPU. Lives in the fire structures.
#[derive(Debug, Default)]
pub(crate) struct ReplayChains {
    /// Some area has a loop or records. False means the per-burst checks return at once.
    pub engaged: bool,
    /// Memory areas inside a recording session (mirror of the capture state).
    pub recording: AHashSet<u32>,
    /// Memory areas that have a `sequential_memory` loop.
    pub loop_areas: AHashSet<u32>,
    /// Successors of each source state, best first (stamped by the plasticity service).
    pub order: AHashMap<u32, Vec<u32>>,
    /// Ordered member list of each episode neuron (stamped by the plasticity service; derived,
    /// rebuilt after a load). An episode neuron is never a state.
    pub episodes: AHashMap<u32, Vec<u32>>,
    /// First state of each episode to its episode neurons (Q1 index; derived from `episodes`
    /// on every stamp, so it is rebuilt with them after a load).
    pub first_index: AHashMap<u32, Vec<u32>>,
    /// Running chain replay per area.
    pub chains: AHashMap<u32, ChainRun>,
    /// Chain heads that drive a step this burst, `(area, head)`; read after propagation.
    pub stepping: Vec<(u32, u32)>,
    /// Fires whose own replay is blocked this burst (lockout, recording, same-burst loser).
    pub suppressed: AHashSet<u32>,
    /// Replays blocked because a chain was running in the area (since the NPU was built).
    pub blocked_by_lockout: u64,
    /// Replays blocked because the area was recording.
    pub blocked_by_recording: u64,
    /// The same two counts per memory area, `(by a running chain, by recording)`.
    pub blocked_by_area: AHashMap<u32, (u64, u64)>,
}

impl ReplayChains {
    pub fn refresh_engaged(&mut self) {
        self.engaged =
            !self.loop_areas.is_empty() || !self.recording.is_empty() || !self.chains.is_empty();
    }

    /// Count `n` replays of `area_idx` blocked by a running chain.
    pub fn note_lockout(&mut self, area_idx: u32, n: u64) {
        self.blocked_by_lockout += n;
        self.blocked_by_area.entry(area_idx).or_default().0 += n;
    }

    /// Count `n` replays of `area_idx` blocked because the area records.
    pub fn note_recording_block(&mut self, area_idx: u32, n: u64) {
        self.blocked_by_recording += n;
        self.blocked_by_area.entry(area_idx).or_default().1 += n;
    }

    /// Is the area one of the engaged ones (has a loop, records, or runs a chain)?
    pub fn area_engaged(&self, area_idx: u32) -> bool {
        self.loop_areas.contains(&area_idx)
            || self.recording.contains(&area_idx)
            || self.chains.contains_key(&area_idx)
    }

    /// An area lost its loop: its chain and loop registration go; its recording mirror stays.
    pub fn forget_area(&mut self, area_idx: u32) {
        self.chains.remove(&area_idx);
        self.loop_areas.remove(&area_idx);
        self.refresh_engaged();
    }

    /// Mirror the NPU's recording areas.
    pub fn sync_recording(&mut self, recording: impl Iterator<Item = u32>) {
        self.recording.clear();
        self.recording.extend(recording);
        self.refresh_engaged();
    }

    /// Add or replace episode neurons, and forget the listed ones.
    pub fn stamp_episodes(&mut self, upserts: Vec<(u32, Vec<u32>)>, removed: &[u32]) {
        for id in removed {
            self.episodes.remove(id);
        }
        for (id, members) in upserts {
            self.episodes.insert(id, members);
        }
        self.first_index.clear();
        for (id, members) in &self.episodes {
            if let Some(first) = members.first() {
                self.first_index.entry(*first).or_default().push(*id);
            }
        }
        for ids in self.first_index.values_mut() {
            ids.sort_unstable();
        }
    }

    /// Replace the ranked successors of the listed sources.
    pub fn stamp_order(&mut self, ranked: Vec<(u32, Vec<u32>)>) {
        for (source, targets) in ranked {
            if targets.is_empty() {
                self.order.remove(&source);
            } else {
                self.order.insert(source, targets);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_engaged_by_default() {
        let chains = ReplayChains::default();
        assert!(!chains.engaged);
        assert!(!chains.area_engaged(3));
    }

    #[test]
    fn a_loop_or_a_recording_engages_the_area() {
        let mut chains = ReplayChains::default();
        chains.loop_areas.insert(3);
        chains.refresh_engaged();
        assert!(chains.engaged && chains.area_engaged(3) && !chains.area_engaged(4));
        chains.sync_recording([4].into_iter());
        assert!(chains.area_engaged(4));
        chains.forget_area(3);
        assert!(chains.area_engaged(4));
        chains.sync_recording(std::iter::empty());
        assert!(!chains.engaged);
    }

    #[test]
    fn candidates_open_advance_complete_and_drop() {
        let mut chains = ReplayChains::default();
        chains.stamp_episodes(
            vec![(100, vec![1, 2, 3]), (101, vec![1]), (102, vec![1, 5])],
            &[],
        );
        assert_eq!(chains.first_index[&1], vec![100, 101, 102]);
        let (mut cands, mut done) = (Vec::new(), Vec::new());
        open_candidates(
            &chains.first_index,
            &chains.episodes,
            1,
            &mut cands,
            &mut done,
        );
        assert_eq!(done, vec![101]);
        assert_eq!(cands, vec![(100, 0), (102, 0)]);
        advance_candidates(&chains.episodes, 2, &mut cands, &mut done);
        assert_eq!(cands, vec![(100, 1)]);
        advance_candidates(&chains.episodes, 3, &mut cands, &mut done);
        assert!(cands.is_empty());
        assert_eq!(done, vec![101, 100]);
        chains.stamp_episodes(vec![], &[100, 101, 102]);
        assert!(chains.first_index.is_empty());
    }

    #[test]
    fn rank_is_position_and_unknown_ranks_last() {
        let mut chains = ReplayChains::default();
        chains.stamp_order(vec![(1, vec![7, 5, 9])]);
        assert_eq!(rank_in(&chains.order, 1, 5), 1);
        assert_eq!(rank_in(&chains.order, 1, 42), usize::MAX);
        assert_eq!(rank_in(&chains.order, 2, 7), usize::MAX);
        chains.stamp_order(vec![(1, vec![])]);
        assert_eq!(rank_in(&chains.order, 1, 7), usize::MAX);
    }
}
