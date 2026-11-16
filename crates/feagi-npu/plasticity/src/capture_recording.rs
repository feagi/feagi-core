// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Plasticity-service side of `memory.capture` recording.
//!
//! The NPU owns the recording state and queues ordered boundary events (see
//! `feagi_npu_burst_engine::memory_capture`). This module turns those events and the states the
//! service encodes into episode links:
//!
//! - a **state** is a memory neuron that fired by encoding (live input) while the area records;
//!   recall, replay and scanner fires are never passed in;
//! - a held input re-encodes the same neuron every burst and counts as one state;
//! - silent bursts encode nothing, so they neither add a state nor end the piece;
//! - each new state (a neuron different from the last one) links `last -> new`, so the chain is
//!   continuous across skipped bursts and across piece boundaries;
//! - the boost (lifespan toward long-term) is applied once per neuron per session.
//!
//! Replay ranking lives here too: the service keeps each loop area's episode links as
//! `source -> successors`, ranks the successors of a state by the neuron columns it owns
//! (long-term first, then the larger lifespan, then the activation count, then the one that
//! fired most recently) and stamps the order into the NPU's successor table. The table is
//! derived: it is rebuilt from the live links after a load or a mapping edit.
//!
//! The service can skip bursts, so nothing here assumes `t - 1` adjacency: events are applied
//! in order up to the burst being processed, and later ones wait.

use std::collections::{HashMap, HashSet};

use feagi_npu_burst_engine::{CaptureEvent, CaptureEventKind};

/// Read-only view of one area's recording, for tests and tooling.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CaptureAreaInfo {
    /// A session is open (the driver is on).
    pub recording: bool,
    /// Session id of the open or last session.
    pub session: Option<u64>,
    /// Pieces started since the service began (first piece of each session included).
    pub pieces: u32,
    /// Episode links the service asked the NPU to insert.
    pub links: u64,
    /// Of those, links from the last state of one piece to the first state of the next.
    pub boundary_links: u64,
    /// Neurons whose lifespan was boosted (once per neuron per session).
    pub boosted: u64,
    /// Episode neurons created at a piece end (a new ordered member list).
    pub episodes_created: u64,
    /// Piece ends whose member list already had an episode neuron (reactivated, none added).
    pub episodes_reactivated: u64,
    /// Piece ends that made no episode neuron: a member was gone, the hash matched another
    /// list, or there was no room.
    pub episodes_skipped: u64,
}

/// One finished recording piece, waiting for the service to make its episode neuron.
#[derive(Debug, Clone, PartialEq)]
pub struct ClosedPiece {
    pub area_idx: u32,
    /// The piece's states in the order they were encoded (a held input counts once).
    pub members: Vec<u32>,
    /// Capture magnitude of the session, in percent (the boost of the episode neuron).
    pub magnitude: f32,
}

#[derive(Debug, Default)]
struct AreaRecorder {
    recording: bool,
    session: Option<u64>,
    /// Last encoded neuron id and the burst it was encoded at.
    last_state: Option<(u32, u64)>,
    /// A piece boundary was crossed and no state was seen since.
    boundary_pending: bool,
    boosted_this_session: HashSet<u32>,
    /// States of the open piece, in encoding order. An episode neuron is never one of them.
    piece_members: Vec<u32>,
    /// A member of the open piece left the array: the piece makes no episode neuron.
    piece_broken: bool,
    info: CaptureAreaInfo,
}

/// What recording asks for after one encoded state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StateOutcome {
    /// Apply the once-per-session lifespan boost to this neuron now.
    pub boost: bool,
    /// Insert a link from this neuron to the state (both ids are memory neuron ids).
    pub link_from: Option<u32>,
    /// The link crosses a piece boundary (last state of one piece to the first of the next).
    pub boundary: bool,
}

/// Service-side recording state of every capture-subscribed memory area.
#[derive(Debug, Default)]
pub struct CaptureRecording {
    areas: HashMap<u32, AreaRecorder>,
    /// Events not yet reached by the burst being processed.
    pending: Vec<CaptureEvent>,
    /// Areas that have a `sequential_memory` loop, as of the last pass.
    loops: HashSet<u32>,
    /// Capture magnitude of each recording area, as of the last pass.
    magnitudes: HashMap<u32, f32>,
    /// Episode links of each loop area as `source -> successors` (the ranking's input).
    edges: HashMap<u32, HashMap<u32, Vec<u32>>>,
    /// Areas whose ranking must be recomputed at the end of this pass.
    dirty: HashSet<u32>,
    /// Last order stamped into the NPU per source: only changes are sent again.
    stamped: HashMap<u32, Vec<u32>>,
    /// Capture magnitude last seen per area (the signal is gone by the pass that sees a session
    /// end, and the episode neuron is boosted by the session's magnitude).
    last_magnitudes: HashMap<u32, f32>,
    /// Pieces finished by a boundary event, in order, waiting for their episode neuron.
    closed: Vec<ClosedPiece>,
    /// Episode neuron ids the service made or restored (so a removal pass finds them quickly).
    episode_ids: HashSet<u32>,
}

impl CaptureRecording {
    /// True when there is nothing to record and nothing that could start recording.
    pub fn is_idle(&self) -> bool {
        self.pending.is_empty() && self.areas.values().all(|a| !a.recording)
    }

    /// Take the NPU's work for this pass and apply every event up to `timestep`.
    pub fn begin_pass(
        &mut self,
        timestep: u64,
        events: Vec<CaptureEvent>,
        loops: impl IntoIterator<Item = u32>,
        magnitudes: HashMap<u32, f32>,
    ) {
        self.pending.extend(events);
        self.loops = loops.into_iter().collect();
        for (area, magnitude) in &magnitudes {
            self.last_magnitudes.insert(*area, *magnitude);
        }
        self.magnitudes = magnitudes;
        let split = self
            .pending
            .iter()
            .position(|e| e.burst > timestep)
            .unwrap_or(self.pending.len());
        let due: Vec<CaptureEvent> = self.pending.drain(..split).collect();
        for event in due {
            self.apply(event);
        }
    }

    fn apply(&mut self, event: CaptureEvent) {
        let magnitude = self
            .last_magnitudes
            .get(&event.area_idx)
            .copied()
            .unwrap_or(0.0);
        let area = self.areas.entry(event.area_idx).or_default();
        match event.kind {
            CaptureEventKind::SessionStart => {
                area.recording = true;
                area.session = Some(event.session);
                area.last_state = None;
                area.boundary_pending = false;
                area.boosted_this_session.clear();
                area.piece_members.clear();
                area.piece_broken = false;
                area.info.recording = true;
                area.info.session = Some(event.session);
                area.info.pieces += 1;
            }
            CaptureEventKind::PieceStart => {
                area.boundary_pending = true;
                area.info.pieces += 1;
                Self::close_piece(&mut self.closed, event.area_idx, area, magnitude);
            }
            CaptureEventKind::SessionEnd => {
                Self::close_piece(&mut self.closed, event.area_idx, area, magnitude);
                area.recording = false;
                area.last_state = None;
                area.boundary_pending = false;
                area.boosted_this_session.clear();
                area.info.recording = false;
            }
        }
    }

    /// The open piece ends: its member list is handed to the service, and a new list starts.
    fn close_piece(
        closed: &mut Vec<ClosedPiece>,
        area_idx: u32,
        area: &mut AreaRecorder,
        magnitude: f32,
    ) {
        let members = std::mem::take(&mut area.piece_members);
        let broken = std::mem::take(&mut area.piece_broken);
        if members.is_empty() {
            return;
        }
        if broken {
            area.info.episodes_skipped += 1;
            tracing::debug!(
                target: "plasticity",
                "[PLASTICITY] memory.capture area {}: a member left the array before the piece ended; no episode neuron",
                area_idx
            );
            return;
        }
        closed.push(ClosedPiece {
            area_idx,
            members,
            magnitude,
        });
    }

    /// Pieces that ended since the last call, oldest first.
    pub fn take_closed(&mut self) -> Vec<ClosedPiece> {
        std::mem::take(&mut self.closed)
    }

    /// What happened to a piece's episode neuron, for the area's summary.
    pub fn note_episode(&mut self, area_idx: u32, created: bool, reactivated: bool) {
        if let Some(area) = self.areas.get_mut(&area_idx) {
            if created {
                area.info.episodes_created += 1;
            } else if reactivated {
                area.info.episodes_reactivated += 1;
            } else {
                area.info.episodes_skipped += 1;
            }
        }
    }

    /// Once per neuron per session: should this neuron get the session boost now? Used for the
    /// episode neuron (states use [`Self::note_state`]).
    pub fn boost_once(&mut self, area_idx: u32, neuron_id: u32) -> bool {
        match self.areas.get_mut(&area_idx) {
            Some(area) => {
                let first = area.boosted_this_session.insert(neuron_id);
                if first {
                    area.info.boosted += 1;
                }
                first
            }
            None => false,
        }
    }

    /// The service made or restored an episode neuron.
    pub fn track_episode(&mut self, neuron_id: u32) {
        self.episode_ids.insert(neuron_id);
    }

    /// Episode neurons among `removed`, forgotten here (the NPU's table forgets them too).
    pub fn take_removed_episodes(&mut self, removed: &HashSet<u32>) -> Vec<u32> {
        if self.episode_ids.is_empty() || removed.is_empty() {
            return Vec::new();
        }
        let gone: Vec<u32> = removed
            .iter()
            .copied()
            .filter(|id| self.episode_ids.remove(id))
            .collect();
        gone
    }

    pub fn is_recording(&self, area_idx: u32) -> bool {
        self.areas.get(&area_idx).is_some_and(|a| a.recording)
    }

    pub fn has_loop(&self, area_idx: u32) -> bool {
        self.loops.contains(&area_idx)
    }

    /// Capture magnitude of a recording area, in percent.
    pub fn magnitude(&self, area_idx: u32) -> f32 {
        self.magnitudes.get(&area_idx).copied().unwrap_or(0.0)
    }

    /// The area is held as a full snapshot (`memory.freeze`): nothing is recorded, and the next
    /// state starts a fresh chain.
    pub fn break_chain(&mut self, area_idx: u32) {
        if let Some(area) = self.areas.get_mut(&area_idx) {
            area.last_state = None;
            area.piece_members.clear();
            area.piece_broken = false;
        }
    }

    /// One encoded state of a recording area at `burst`.
    pub fn note_state(&mut self, area_idx: u32, neuron_id: u32, burst: u64) -> StateOutcome {
        let has_loop = self.loops.contains(&area_idx);
        let Some(area) = self.areas.get_mut(&area_idx) else {
            return StateOutcome::default();
        };
        if !area.recording {
            return StateOutcome::default();
        }
        let mut out = StateOutcome::default();
        if area.boosted_this_session.insert(neuron_id) {
            area.info.boosted += 1;
            out.boost = true;
        }
        let boundary = std::mem::take(&mut area.boundary_pending);
        // The piece's ordered member list: a held input is one entry, and a state held across a
        // piece boundary is the first entry of the new piece.
        if boundary || area.last_state.map(|(last, _)| last) != Some(neuron_id) {
            area.piece_members.push(neuron_id);
        }
        if let Some((last, _)) = area.last_state {
            if last != neuron_id && has_loop {
                out.link_from = Some(last);
                out.boundary = boundary;
            }
        }
        area.last_state = Some((neuron_id, burst));
        out
    }

    /// Links the NPU actually inserted (a duplicate pair is not counted).
    pub fn record_inserted(&mut self, area_idx: u32, links: u64, boundary_links: u64) {
        if let Some(area) = self.areas.get_mut(&area_idx) {
            area.info.links += links;
            area.info.boundary_links += boundary_links;
        }
    }

    /// The successors of every source state of a loop area, replaced from the NPU's live links.
    pub fn replace_edges(&mut self, area_idx: u32, pairs: impl IntoIterator<Item = (u32, u32)>) {
        let mut edges: HashMap<u32, Vec<u32>> = HashMap::new();
        for (source, target) in pairs {
            let list = edges.entry(source).or_default();
            if !list.contains(&target) {
                list.push(target);
            }
        }
        // Forget the old stamps of this area's sources so the rebuilt order is sent in full.
        if let Some(old) = self.edges.get(&area_idx) {
            for source in old.keys() {
                self.stamped.remove(source);
            }
        }
        self.edges.insert(area_idx, edges);
        self.dirty.insert(area_idx);
    }

    /// One new episode link `source -> target`.
    pub fn add_edge(&mut self, area_idx: u32, source: u32, target: u32) {
        let list = self
            .edges
            .entry(area_idx)
            .or_default()
            .entry(source)
            .or_default();
        if !list.contains(&target) {
            list.push(target);
        }
        self.dirty.insert(area_idx);
    }

    /// The ranking of the area changes with its neurons (activation, conversion, removal).
    pub fn mark_dirty(&mut self, area_idx: u32) {
        if self.loops.contains(&area_idx) {
            self.dirty.insert(area_idx);
        }
    }

    /// Loop areas to rank now; clears the set.
    pub fn take_dirty(&mut self) -> Vec<u32> {
        let areas: Vec<u32> = self
            .dirty
            .drain()
            .filter(|area| self.loops.contains(area))
            .collect();
        areas
    }

    /// Successors of every source of the area.
    pub fn edges_of(&self, area_idx: u32) -> Option<&HashMap<u32, Vec<u32>>> {
        self.edges.get(&area_idx)
    }

    /// Keep only the sources whose ranked order differs from the last stamp, and remember it.
    pub fn changed_orders(&mut self, ranked: Vec<(u32, Vec<u32>)>) -> Vec<(u32, Vec<u32>)> {
        ranked
            .into_iter()
            .filter(|(source, order)| {
                if self.stamped.get(source) == Some(order) {
                    false
                } else {
                    if order.is_empty() {
                        self.stamped.remove(source);
                    } else {
                        self.stamped.insert(*source, order.clone());
                    }
                    true
                }
            })
            .collect()
    }

    /// Neurons left the memory array this pass: they are no longer a chain end and may be
    /// boosted again if their id is reused.
    pub fn forget_neurons(&mut self, removed: &HashSet<u32>) {
        if removed.is_empty() {
            return;
        }
        for area in self.areas.values_mut() {
            if area.last_state.is_some_and(|(id, _)| removed.contains(&id)) {
                area.last_state = None;
            }
            area.boosted_this_session.retain(|id| !removed.contains(id));
            if area.piece_members.iter().any(|id| removed.contains(id)) {
                area.piece_broken = true;
            }
        }
        self.closed
            .retain(|piece| !piece.members.iter().any(|id| removed.contains(id)));
        for (area_idx, edges) in self.edges.iter_mut() {
            let mut touched = false;
            for id in removed {
                if edges.remove(id).is_some() {
                    touched = true;
                }
                self.stamped.remove(id);
            }
            for targets in edges.values_mut() {
                let before = targets.len();
                targets.retain(|t| !removed.contains(t));
                touched |= targets.len() != before;
            }
            edges.retain(|_, targets| !targets.is_empty());
            if touched {
                self.dirty.insert(*area_idx);
            }
        }
    }

    pub fn info(&self, area_idx: u32) -> Option<CaptureAreaInfo> {
        self.areas.get(&area_idx).map(|a| a.info.clone())
    }

    /// Drop all state (brain reset).
    pub fn clear(&mut self) {
        self.areas.clear();
        self.pending.clear();
        self.loops.clear();
        self.magnitudes.clear();
        self.edges.clear();
        self.dirty.clear();
        self.stamped.clear();
        self.last_magnitudes.clear();
        self.closed.clear();
        self.episode_ids.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: CaptureEventKind, piece: u32, burst: u64) -> CaptureEvent {
        CaptureEvent {
            area_idx: 1,
            kind,
            session: 0,
            piece,
            burst,
        }
    }

    #[test]
    fn events_wait_until_their_burst_is_reached() {
        let mut rec = CaptureRecording::default();
        rec.begin_pass(
            3,
            vec![
                event(CaptureEventKind::SessionStart, 0, 2),
                event(CaptureEventKind::SessionEnd, 0, 6),
            ],
            [1],
            HashMap::new(),
        );
        assert!(rec.is_recording(1));
        rec.begin_pass(7, vec![], [1], HashMap::new());
        assert!(!rec.is_recording(1));
    }

    #[test]
    fn a_held_state_is_one_state_and_links_follow_the_last_state() {
        let mut rec = CaptureRecording::default();
        rec.begin_pass(
            0,
            vec![event(CaptureEventKind::SessionStart, 0, 0)],
            [1],
            HashMap::new(),
        );
        let a = rec.note_state(1, 10, 0);
        assert!(a.boost && a.link_from.is_none());
        let held = rec.note_state(1, 10, 1);
        assert!(!held.boost && held.link_from.is_none());
        let b = rec.note_state(1, 11, 5);
        assert!(b.boost);
        assert_eq!(b.link_from, Some(10));
        // Links are counted when the NPU reports them inserted, not when they are asked for.
        rec.record_inserted(1, 1, 0);
        let info = rec.info(1).unwrap();
        assert_eq!((info.links, info.boosted), (1, 2));
    }

    #[test]
    fn ranking_changes_are_sent_once_and_removed_neurons_leave_the_edges() {
        let mut rec = CaptureRecording::default();
        rec.begin_pass(0, vec![], [1], HashMap::new());
        rec.add_edge(1, 10, 11);
        rec.add_edge(1, 10, 12);
        rec.add_edge(1, 10, 12);
        assert_eq!(rec.edges_of(1).unwrap()[&10], vec![11, 12]);
        assert_eq!(rec.take_dirty(), vec![1]);
        assert!(rec.take_dirty().is_empty());
        let first = rec.changed_orders(vec![(10, vec![12, 11])]);
        assert_eq!(first.len(), 1);
        assert!(rec.changed_orders(vec![(10, vec![12, 11])]).is_empty());
        rec.forget_neurons(&[12].into_iter().collect());
        assert_eq!(rec.edges_of(1).unwrap()[&10], vec![11]);
        assert_eq!(rec.take_dirty(), vec![1]);
    }

    #[test]
    fn no_loop_means_no_link_but_still_a_boost() {
        let mut rec = CaptureRecording::default();
        rec.begin_pass(
            0,
            vec![event(CaptureEventKind::SessionStart, 0, 0)],
            [],
            HashMap::new(),
        );
        rec.note_state(1, 10, 0);
        let out = rec.note_state(1, 11, 1);
        assert!(out.boost);
        assert_eq!(out.link_from, None);
    }

    #[test]
    fn a_piece_boundary_closes_the_member_list_and_a_held_state_opens_the_next() {
        let mut rec = CaptureRecording::default();
        rec.begin_pass(
            0,
            vec![event(CaptureEventKind::SessionStart, 0, 0)],
            [1],
            [(1, 40.0)].into_iter().collect(),
        );
        rec.note_state(1, 10, 0);
        rec.note_state(1, 10, 1); // held: still one entry
        rec.note_state(1, 11, 2);
        rec.begin_pass(
            5,
            vec![event(CaptureEventKind::PieceStart, 1, 5)],
            [1],
            HashMap::new(),
        );
        let closed = rec.take_closed();
        assert_eq!(closed.len(), 1);
        assert_eq!(closed[0].members, vec![10, 11]);
        assert_eq!(closed[0].magnitude, 40.0);
        // State 11 stays on across the boundary: it opens the next piece.
        rec.note_state(1, 11, 5);
        rec.note_state(1, 12, 6);
        rec.begin_pass(
            9,
            vec![event(CaptureEventKind::SessionEnd, 1, 9)],
            [1],
            HashMap::new(),
        );
        let closed = rec.take_closed();
        assert_eq!(closed[0].members, vec![11, 12]);
        // The magnitude is still known once the signal is gone.
        assert_eq!(closed[0].magnitude, 40.0);
    }

    #[test]
    fn a_member_that_left_the_array_breaks_its_piece() {
        let mut rec = CaptureRecording::default();
        rec.begin_pass(
            0,
            vec![event(CaptureEventKind::SessionStart, 0, 0)],
            [1],
            HashMap::new(),
        );
        rec.note_state(1, 10, 0);
        rec.note_state(1, 11, 1);
        rec.forget_neurons(&[10].into_iter().collect());
        rec.begin_pass(
            3,
            vec![event(CaptureEventKind::SessionEnd, 0, 3)],
            [1],
            HashMap::new(),
        );
        assert!(rec.take_closed().is_empty());
        assert_eq!(rec.info(1).unwrap().episodes_skipped, 1);
    }
}
