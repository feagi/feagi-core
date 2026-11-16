// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Recording state of `memory.capture`, owned by the NPU.
//!
//! A memory area records while the combined signal of its `memory.capture` modulators is above
//! zero. The NPU reads that signal in-burst (right after the modulator commit), so it sees every
//! burst even when the plasticity thread skips some. It cuts a recording session into pieces by
//! burst count and queues ordered boundary events `(session, piece, burst)`. The plasticity
//! service consumes the events and builds the episode links from the states it encodes; it must
//! not assume that two consecutive events or states are one burst apart.
//!
//! Nothing here runs for an area with no capture subscription and no loop: the per-burst tick
//! walks only the areas the modulator engine reports as capture-subscribed.

use ahash::AHashMap;
use feagi_npu_neural::SynapseType;

/// Simulated seconds one recording piece lasts when the area sets no `capture_max_bursts`.
pub const DEFAULT_CAPTURE_PIECE_SECONDS: f64 = 5.0;

/// Upper bound on queued events. Events are consumed every plasticity pass; the bound only
/// protects an NPU that runs with no consumer (the oldest events are dropped).
const MAX_QUEUED_EVENTS: usize = 4096;

/// What a recording event reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureEventKind {
    /// The driver turned on: a session and its first piece start.
    SessionStart,
    /// The driver stayed on and the piece reached its cap: the next piece starts at once.
    PieceStart,
    /// The driver turned off: the session ends.
    SessionEnd,
}

/// One ordered recording boundary of a memory area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureEvent {
    pub area_idx: u32,
    pub kind: CaptureEventKind,
    /// Increases by one per session of the NPU, never reused.
    pub session: u64,
    /// Piece number inside the session, from 0.
    pub piece: u32,
    /// NPU burst the event belongs to: a state encoded at this burst or later is in this piece.
    pub burst: u64,
}

/// Parameters of an area's `sequential_memory` loop mapping, read from the mapping rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SequentialLoopParams {
    pub weight: f32,
    pub psp: f32,
    pub synapse_type: SynapseType,
    pub delay_bursts: u8,
    /// Modulation group of the loop rule (0 = none), stamped on every episode link.
    pub modulation_group: u16,
}

/// What the plasticity service needs from the NPU for one pass.
#[derive(Debug, Clone, Default)]
pub struct CaptureWork {
    /// Boundary events since the last call, oldest first.
    pub events: Vec<CaptureEvent>,
    /// Loop parameters of every area that has a loop mapping.
    pub loops: AHashMap<u32, SequentialLoopParams>,
    /// Capture magnitude (1..=100) of every area that is recording right now.
    pub magnitudes: AHashMap<u32, f32>,
    /// Areas whose episode links changed outside the plasticity service (a loop registered, a
    /// restamp after a mapping edit or a load): the successor ranking must be rebuilt for them.
    pub stale_areas: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
struct AreaRecording {
    session: u64,
    piece: u32,
    piece_start_burst: u64,
}

/// Recording state of every capture-subscribed memory area.
#[derive(Debug, Default)]
pub struct MemoryCaptureState {
    recording: AHashMap<u32, AreaRecording>,
    /// Per-area `capture_max_bursts` from the genome.
    max_bursts: AHashMap<u32, u32>,
    loops: AHashMap<u32, SequentialLoopParams>,
    events: Vec<CaptureEvent>,
    next_session: u64,
    /// Areas whose successor ranking must be rebuilt (see [`CaptureWork::stale_areas`]).
    stale: ahash::AHashSet<u32>,
}

impl MemoryCaptureState {
    pub fn set_max_bursts(&mut self, area_idx: u32, max_bursts: Option<u32>) {
        match max_bursts {
            Some(n) => {
                self.max_bursts.insert(area_idx, n.max(1));
            }
            None => {
                self.max_bursts.remove(&area_idx);
            }
        }
    }

    pub fn set_loop(&mut self, area_idx: u32, params: SequentialLoopParams) {
        self.loops.insert(area_idx, params);
        self.stale.insert(area_idx);
    }

    pub fn clear_loop(&mut self, area_idx: u32) -> bool {
        self.stale.remove(&area_idx);
        self.loops.remove(&area_idx).is_some()
    }

    /// The area's episode links changed outside the plasticity service.
    pub fn mark_stale(&mut self, area_idx: u32) {
        if self.loops.contains_key(&area_idx) {
            self.stale.insert(area_idx);
        }
    }

    pub fn drain_stale(&mut self) -> Vec<u32> {
        self.stale.drain().collect()
    }

    /// Longest a chain replay in the area may run, in bursts: the area's recording-piece cap.
    /// Uncapped (a very large count) when no timestep is known.
    pub fn replay_cap_bursts(&self, area_idx: u32) -> u64 {
        self.piece_bursts(area_idx).unwrap_or(u64::MAX / 4)
    }

    pub fn loop_params(&self, area_idx: u32) -> Option<SequentialLoopParams> {
        self.loops.get(&area_idx).copied()
    }

    pub fn loops(&self) -> &AHashMap<u32, SequentialLoopParams> {
        &self.loops
    }

    /// Bursts one piece lasts: the area's `capture_max_bursts`, else the default seconds
    /// converted with the live simulation timestep (rounded, at least 1). `None` when neither
    /// is known (no burst loop has set a timestep), which leaves the piece uncapped.
    fn piece_bursts(&self, area_idx: u32) -> Option<u64> {
        if let Some(n) = self.max_bursts.get(&area_idx) {
            return Some((*n).max(1) as u64);
        }
        let step = crate::sim_timestep().as_secs_f64();
        if step > 0.0 {
            Some(((DEFAULT_CAPTURE_PIECE_SECONDS / step).round() as u64).max(1))
        } else {
            None
        }
    }

    /// Advance every capture-subscribed area to `burst` given whether its driver is on.
    /// `states` holds `(area, combined capture signal > 0)` for the subscribed areas only.
    /// A recording area that no longer subscribes counts as off.
    pub fn tick(&mut self, burst: u64, states: &[(u32, bool)]) {
        let dropped: Vec<u32> = self
            .recording
            .keys()
            .copied()
            .filter(|area| !states.iter().any(|(a, _)| a == area))
            .collect();
        for area_idx in dropped {
            if let Some(rec) = self.recording.remove(&area_idx) {
                self.push(CaptureEvent {
                    area_idx,
                    kind: CaptureEventKind::SessionEnd,
                    session: rec.session,
                    piece: rec.piece,
                    burst,
                });
            }
        }
        for &(area_idx, on) in states {
            match (on, self.recording.get(&area_idx).copied()) {
                (true, None) => {
                    let session = self.next_session;
                    self.next_session += 1;
                    self.recording.insert(
                        area_idx,
                        AreaRecording {
                            session,
                            piece: 0,
                            piece_start_burst: burst,
                        },
                    );
                    self.push(CaptureEvent {
                        area_idx,
                        kind: CaptureEventKind::SessionStart,
                        session,
                        piece: 0,
                        burst,
                    });
                }
                (true, Some(mut rec)) => {
                    let cap = self.piece_bursts(area_idx);
                    if cap.is_some_and(|cap| burst.saturating_sub(rec.piece_start_burst) >= cap) {
                        rec.piece += 1;
                        rec.piece_start_burst = burst;
                        self.recording.insert(area_idx, rec);
                        self.push(CaptureEvent {
                            area_idx,
                            kind: CaptureEventKind::PieceStart,
                            session: rec.session,
                            piece: rec.piece,
                            burst,
                        });
                    }
                }
                (false, Some(rec)) => {
                    self.recording.remove(&area_idx);
                    self.push(CaptureEvent {
                        area_idx,
                        kind: CaptureEventKind::SessionEnd,
                        session: rec.session,
                        piece: rec.piece,
                        burst,
                    });
                }
                (false, None) => {}
            }
        }
    }

    fn push(&mut self, event: CaptureEvent) {
        if self.events.len() >= MAX_QUEUED_EVENTS {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    /// True while there is nothing a consumer could act on: no loop, no recording area and no
    /// queued event.
    pub fn is_idle(&self) -> bool {
        self.loops.is_empty()
            && self.recording.is_empty()
            && self.events.is_empty()
            && self.stale.is_empty()
    }

    pub fn drain_events(&mut self) -> Vec<CaptureEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn is_recording(&self, area_idx: u32) -> bool {
        self.recording.contains_key(&area_idx)
    }

    pub fn recording_areas(&self) -> impl Iterator<Item = u32> + '_ {
        self.recording.keys().copied()
    }

    /// Forget everything that belongs to the previous genome (loops, caps, sessions).
    pub fn clear_all(&mut self) {
        self.recording.clear();
        self.events.clear();
        self.loops.clear();
        self.max_bursts.clear();
        self.stale.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(state: &mut MemoryCaptureState, from: u64, to: u64, on: bool) {
        for burst in from..to {
            state.tick(burst, &[(3, on)]);
        }
    }

    #[test]
    fn pieces_are_cut_by_burst_count_and_a_new_one_starts_at_once() {
        let mut state = MemoryCaptureState::default();
        state.set_max_bursts(3, Some(12));
        run(&mut state, 0, 30, true);
        run(&mut state, 30, 31, false);
        let events = state.drain_events();
        let kinds: Vec<(CaptureEventKind, u32, u64)> =
            events.iter().map(|e| (e.kind, e.piece, e.burst)).collect();
        assert_eq!(
            kinds,
            vec![
                (CaptureEventKind::SessionStart, 0, 0),
                (CaptureEventKind::PieceStart, 1, 12),
                (CaptureEventKind::PieceStart, 2, 24),
                (CaptureEventKind::SessionEnd, 2, 30),
            ]
        );
        assert!(events.iter().all(|e| e.session == 0));
    }

    #[test]
    fn a_new_driver_burst_starts_a_new_session() {
        let mut state = MemoryCaptureState::default();
        run(&mut state, 0, 2, true);
        run(&mut state, 2, 4, false);
        run(&mut state, 4, 6, true);
        let sessions: Vec<u64> = state.drain_events().iter().map(|e| e.session).collect();
        assert_eq!(sessions, vec![0, 0, 1]);
    }

    #[test]
    fn no_subscription_means_no_state() {
        let mut state = MemoryCaptureState::default();
        state.tick(5, &[]);
        assert!(state.is_idle());
    }
}
