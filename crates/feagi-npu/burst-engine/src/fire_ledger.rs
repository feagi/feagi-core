// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! FireLedger - Dense, burst-aligned firing history for memory + STDP.
//!
//! Key semantics:
//! - Dense: tracked areas receive a frame every burst (explicit empty frames when silent).
//! - Burst-aligned: windows are defined by timestep range, not "last N firing events".
//! - Tracked-only: history is stored only for explicitly tracked cortical areas.
//! - Deterministic: no implicit defaults; errors are explicit.

use ahash::AHashMap;
use roaring::RoaringBitmap;
use std::collections::VecDeque;

use crate::fire_structures::FireQueue;

#[derive(Debug, Clone, thiserror::Error)]
pub enum FireLedgerError {
    #[error("window size must be > 0")]
    InvalidWindowSize,

    #[error("depth must be > 0")]
    InvalidDepth,

    #[error("non-monotonic timestep: current={current}, requested={requested}")]
    NonMonotonicTimestep { current: u64, requested: u64 },

    #[error("area {cortical_idx} is not tracked")]
    AreaNotTracked { cortical_idx: u32 },

    #[error("requested end_timestep={end_timestep} exceeds current_timestep={current_timestep}")]
    EndTimestepInFuture {
        end_timestep: u64,
        current_timestep: u64,
    },

    #[error(
        "insufficient history for area {cortical_idx}: need [{start}..{end}], but have [{have_start}..{have_end}]"
    )]
    InsufficientHistory {
        cortical_idx: u32,
        start: u64,
        end: u64,
        have_start: u64,
        have_end: u64,
    },

    #[error("requested depth {depth} exceeds tracked window size {window_size} for area {cortical_idx}")]
    DepthExceedsWindow {
        cortical_idx: u32,
        depth: usize,
        window_size: usize,
    },
}

/// Dense, tracked-area firing history.
#[derive(Debug, Clone)]
pub struct FireLedger {
    tracked: AHashMap<u32, TrackedAreaHistory>,
    current_timestep: u64,
    capacity_hint: usize,
}

#[derive(Debug, Clone)]
struct TrackedAreaHistory {
    window_size: usize,
    frames: VecDeque<(u64, RoaringBitmap)>, // oldest -> newest
}

impl FireLedger {
    /// Create a new FireLedger.
    ///
    /// `capacity_hint` is used only for internal allocations.
    pub fn new(capacity_hint: usize) -> Self {
        Self {
            tracked: AHashMap::new(),
            current_timestep: 0,
            capacity_hint,
        }
    }

    pub fn current_timestep(&self) -> u64 {
        self.current_timestep
    }

    /// Track a cortical area with an explicit window size.
    ///
    /// This is an exact setting (not max/merge). If multiple subsystems depend on the same area,
    /// the caller must pass the final resolved requirement.
    pub fn track_area(&mut self, cortical_idx: u32, window_size: usize) -> Result<(), FireLedgerError> {
        if window_size == 0 {
            return Err(FireLedgerError::InvalidWindowSize);
        }

        match self.tracked.get_mut(&cortical_idx) {
            Some(hist) => hist.resize_window(window_size),
            None => {
                let mut hist = TrackedAreaHistory::new(window_size, self.capacity_hint);
                // Deterministic initialization: if we already have a current_timestep, prefill a dense
                // empty window ending at current_timestep so queries can succeed immediately.
                if self.current_timestep > 0 {
                    let start = self
                        .current_timestep
                        .saturating_sub(window_size as u64)
                        .saturating_add(1);
                    for t in start..=self.current_timestep {
                        hist.push_frame(t, RoaringBitmap::new());
                    }
                }
                self.tracked.insert(cortical_idx, hist);
            }
        }

        Ok(())
    }

    pub fn untrack_area(&mut self, cortical_idx: u32) -> bool {
        self.tracked.remove(&cortical_idx).is_some()
    }

    /// Get the tracked window size for a cortical area.
    pub fn get_tracked_window(&self, cortical_idx: u32) -> Result<usize, FireLedgerError> {
        self.tracked
            .get(&cortical_idx)
            .map(|h| h.window_size)
            .ok_or(FireLedgerError::AreaNotTracked { cortical_idx })
    }

    /// Tracked windows (sorted for deterministic output).
    pub fn get_tracked_windows(&self) -> Vec<(u32, usize)> {
        let mut out: Vec<(u32, usize)> = self
            .tracked
            .iter()
            .map(|(&idx, hist)| (idx, hist.window_size))
            .collect();
        out.sort_unstable_by_key(|(idx, _)| *idx);
        out
    }

    /// Archive firing data for a burst.
    ///
    /// Dense semantics:
    /// - For each tracked area, a frame is written for every timestep.
    /// - Gaps in timesteps are filled with empty frames.
    pub fn archive_burst(&mut self, timestep: u64, fire_queue: &FireQueue) -> Result<(), FireLedgerError> {
        if self.current_timestep != 0 && timestep <= self.current_timestep {
            return Err(FireLedgerError::NonMonotonicTimestep {
                current: self.current_timestep,
                requested: timestep,
            });
        }

        if self.tracked.is_empty() {
            self.current_timestep = timestep;
            return Ok(());
        }

        // Build bitmaps only for tracked areas that fired this timestep.
        let mut fired_bitmaps: AHashMap<u32, RoaringBitmap> =
            AHashMap::with_capacity(self.tracked.len());
        for (&cortical_idx, neurons) in &fire_queue.neurons_by_area {
            if self.tracked.contains_key(&cortical_idx) {
                let bitmap: RoaringBitmap = neurons.iter().map(|n| n.neuron_id.0).collect();
                fired_bitmaps.insert(cortical_idx, bitmap);
            }
        }

        // Fill missing timesteps with empty frames for all tracked areas.
        if self.current_timestep > 0 && timestep > self.current_timestep + 1 {
            for missing_t in (self.current_timestep + 1)..timestep {
                for hist in self.tracked.values_mut() {
                    hist.push_frame(missing_t, RoaringBitmap::new());
                }
            }
        }

        // Write this timestep.
        for (&cortical_idx, hist) in self.tracked.iter_mut() {
            let bitmap = fired_bitmaps
                .remove(&cortical_idx)
                .unwrap_or_default();
            hist.push_frame(timestep, bitmap);
        }

        self.current_timestep = timestep;
        Ok(())
    }

    /// Get a dense, burst-aligned window of bitmaps for a tracked area.
    ///
    /// Returns exactly `depth` frames covering `[end_timestep - depth + 1 .. end_timestep]`.
    pub fn get_dense_window_bitmaps(
        &self,
        cortical_idx: u32,
        end_timestep: u64,
        depth: usize,
    ) -> Result<Vec<(u64, RoaringBitmap)>, FireLedgerError> {
        if depth == 0 {
            return Err(FireLedgerError::InvalidDepth);
        }
        if end_timestep > self.current_timestep {
            return Err(FireLedgerError::EndTimestepInFuture {
                end_timestep,
                current_timestep: self.current_timestep,
            });
        }

        let hist = self
            .tracked
            .get(&cortical_idx)
            .ok_or(FireLedgerError::AreaNotTracked { cortical_idx })?;

        if depth > hist.window_size {
            return Err(FireLedgerError::DepthExceedsWindow {
                cortical_idx,
                depth,
                window_size: hist.window_size,
            });
        }

        let start = end_timestep.saturating_sub(depth as u64).saturating_add(1);
        let (have_start, have_end) = hist.range_bounds().ok_or(FireLedgerError::InsufficientHistory {
            cortical_idx,
            start,
            end: end_timestep,
            have_start: 0,
            have_end: 0,
        })?;

        if start < have_start || end_timestep > have_end {
            return Err(FireLedgerError::InsufficientHistory {
                cortical_idx,
                start,
                end: end_timestep,
                have_start,
                have_end,
            });
        }

        let start_idx = (start - have_start) as usize;
        let end_idx = start_idx + depth - 1;

        let mut out = Vec::with_capacity(depth);
        for idx in start_idx..=end_idx {
            let Some((t, bm)) = hist.frames.get(idx) else {
                return Err(FireLedgerError::InsufficientHistory {
                    cortical_idx,
                    start,
                    end: end_timestep,
                    have_start,
                    have_end,
                });
            };
            out.push((*t, bm.clone()));
        }
        Ok(out)
    }
}

impl TrackedAreaHistory {
    fn new(window_size: usize, capacity_hint: usize) -> Self {
        let cap = window_size.max(1).min(capacity_hint.max(1));
        Self {
            window_size,
            frames: VecDeque::with_capacity(cap),
        }
    }

    fn resize_window(&mut self, new_size: usize) {
        self.window_size = new_size;
        while self.frames.len() > new_size {
            self.frames.pop_front();
        }
    }

    fn range_bounds(&self) -> Option<(u64, u64)> {
        let (start, _) = self.frames.front()?;
        let (end, _) = self.frames.back()?;
        Some((*start, *end))
    }

    fn push_frame(&mut self, timestep: u64, bitmap: RoaringBitmap) {
        self.frames.push_back((timestep, bitmap));
        while self.frames.len() > self.window_size {
            self.frames.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fire_structures::{FireQueue, FiringNeuron};
    use feagi_npu_neural::types::NeuronId;

    #[test]
    fn test_dense_history_includes_silence() {
        let mut ledger = FireLedger::new(16);
        ledger.track_area(1, 5).unwrap();

        // t=1: fired {100,200}
        let mut fq1 = FireQueue::new();
        fq1.add_neuron(FiringNeuron {
            neuron_id: NeuronId(100),
            membrane_potential: 1.5,
            cortical_idx: 1,
            x: 0,
            y: 0,
            z: 0,
        });
        fq1.add_neuron(FiringNeuron {
            neuron_id: NeuronId(200),
            membrane_potential: 1.2,
            cortical_idx: 1,
            x: 1,
            y: 0,
            z: 0,
        });
        ledger.archive_burst(1, &fq1).unwrap();

        // t=2: silent (empty fire queue)
        let fq2 = FireQueue::new();
        ledger.archive_burst(2, &fq2).unwrap();

        // t=3: fired {200}
        let mut fq3 = FireQueue::new();
        fq3.add_neuron(FiringNeuron {
            neuron_id: NeuronId(200),
            membrane_potential: 1.0,
            cortical_idx: 1,
            x: 0,
            y: 0,
            z: 0,
        });
        ledger.archive_burst(3, &fq3).unwrap();

        let window = ledger.get_dense_window_bitmaps(1, 3, 3).unwrap();
        assert_eq!(window.len(), 3);
        assert_eq!(window[0].0, 1);
        assert_eq!(window[1].0, 2);
        assert_eq!(window[2].0, 3);
        assert_eq!(window[0].1.len(), 2);
        assert_eq!(window[1].1.len(), 0);
        assert_eq!(window[2].1.len(), 1);
    }

    #[test]
    fn test_gap_fill_with_empty_frames() {
        let mut ledger = FireLedger::new(16);
        ledger.track_area(1, 5).unwrap();

        let mut fq1 = FireQueue::new();
        fq1.add_neuron(FiringNeuron {
            neuron_id: NeuronId(1),
            membrane_potential: 1.0,
            cortical_idx: 1,
            x: 0,
            y: 0,
            z: 0,
        });
        ledger.archive_burst(1, &fq1).unwrap();

        // Jump to t=4 (gap fill t=2..3)
        let fq4 = FireQueue::new();
        ledger.archive_burst(4, &fq4).unwrap();

        let window = ledger.get_dense_window_bitmaps(1, 4, 4).unwrap();
        assert_eq!(
            window.iter().map(|(t, _)| *t).collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );
        assert_eq!(window[1].1.len(), 0);
        assert_eq!(window[2].1.len(), 0);
    }

    #[test]
    fn test_insufficient_history_errors() {
        let mut ledger = FireLedger::new(16);
        ledger.track_area(1, 3).unwrap();
        let fq1 = FireQueue::new();
        ledger.archive_burst(1, &fq1).unwrap();

        let err = ledger.get_dense_window_bitmaps(1, 1, 3).unwrap_err();
        assert!(matches!(err, FireLedgerError::InsufficientHistory { .. }));
    }
}
