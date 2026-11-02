// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Burst-indexed stimulus schedules.
//!
//! A schedule is a list of entries, each pinned to an ABSOLUTE burst index. The burst loop
//! drains every entry that is due at the start of a burst (inside the NPU lock, before
//! `process_burst`) and injects it through the manual-stimulation (force-fire) path, so all
//! areas of one entry land on the same burst.

use feagi_structures::genomic::cortical_area::CorticalID;
use std::sync::Mutex;

/// Force-fire voxels for one cortical area: `(x, y, z, potential)`.
pub type StimulusVoxels = Vec<(u32, u32, u32, f32)>;

/// One scheduled injection: every area fires on `burst`.
#[derive(Debug, Clone)]
pub struct ScheduledStimulus {
    /// Absolute burst index the entry fires on.
    pub burst: u64,
    pub areas: Vec<(CorticalID, StimulusVoxels)>,
}

/// Summary of one active schedule (for listing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StimulusScheduleSummary {
    pub schedule_id: u64,
    pub start_burst: u64,
    /// Absolute burst of the last entry; `None` for a repeating schedule (runs until cancelled).
    pub end_burst: Option<u64>,
    pub remaining_entries: usize,
    pub next_burst: Option<u64>,
    /// Replay period in bursts for a repeating schedule.
    pub repeat_period_bursts: Option<u64>,
}

#[derive(Debug)]
struct ActiveSchedule {
    start_burst: u64,
    end_burst: Option<u64>,
    /// When set, fired entries are re-queued `period` bursts later until cancelled.
    repeat_period: Option<u64>,
    /// Remaining entries, sorted by ascending burst.
    entries: Vec<ScheduledStimulus>,
}

#[derive(Debug, Default)]
struct Inner {
    next_id: u64,
    schedules: Vec<(u64, ActiveSchedule)>,
}

/// Shared store of active stimulus schedules.
#[derive(Debug, Default)]
pub struct StimulusScheduleStore {
    inner: Mutex<Inner>,
}

impl StimulusScheduleStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a one-shot schedule. Returns `(schedule_id, start_burst, end_burst)` (first and
    /// last entry burst), or `None` when `entries` is empty.
    pub fn add(&self, entries: Vec<ScheduledStimulus>) -> Option<(u64, u64, u64)> {
        let (id, start, end) = self.add_with_repeat(entries, None)?;
        Some((id, start, end.unwrap_or(start)))
    }

    /// Register a schedule that, when `repeat_period` is `Some(p)`, replays every `p` bursts
    /// until cancelled (`end_burst` is then `None`). Returns `(schedule_id, start_burst,
    /// end_burst)`, or `None` when `entries` is empty or `p` is 0 or not greater than the span
    /// between the first and last entry (cycles must not overlap).
    pub fn add_with_repeat(
        &self,
        mut entries: Vec<ScheduledStimulus>,
        repeat_period: Option<u64>,
    ) -> Option<(u64, u64, Option<u64>)> {
        if entries.is_empty() {
            return None;
        }
        entries.sort_by_key(|e| e.burst);
        let start_burst = entries[0].burst;
        let last_burst = entries[entries.len() - 1].burst;
        if let Some(p) = repeat_period {
            if p == 0 || p <= last_burst - start_burst {
                return None;
            }
        }
        let end_burst = if repeat_period.is_some() {
            None
        } else {
            Some(last_burst)
        };
        let mut inner = self.inner.lock().unwrap();
        inner.next_id += 1;
        let id = inner.next_id;
        inner.schedules.push((
            id,
            ActiveSchedule {
                start_burst,
                end_burst,
                repeat_period,
                entries,
            },
        ));
        Some((id, start_burst, end_burst))
    }

    /// Cancel a schedule; returns whether it existed.
    pub fn cancel(&self, schedule_id: u64) -> bool {
        let mut inner = self.inner.lock().unwrap();
        let before = inner.schedules.len();
        inner.schedules.retain(|(id, _)| *id != schedule_id);
        inner.schedules.len() != before
    }

    /// Summaries of all active schedules.
    pub fn list(&self) -> Vec<StimulusScheduleSummary> {
        let inner = self.inner.lock().unwrap();
        inner
            .schedules
            .iter()
            .map(|(id, s)| StimulusScheduleSummary {
                schedule_id: *id,
                start_burst: s.start_burst,
                end_burst: s.end_burst,
                remaining_entries: s.entries.len(),
                next_burst: s.entries.first().map(|e| e.burst),
                repeat_period_bursts: s.repeat_period,
            })
            .collect()
    }

    /// Remove and return every entry with `burst <= burst_index`. Finished one-shot schedules
    /// are dropped; repeating schedules re-queue each fired entry at its next slot
    /// (`burst + k * period`, the first one after `burst_index`). An entry that was enqueued too
    /// late fires on the next burst, never earlier.
    pub fn take_due(&self, burst_index: u64) -> Vec<ScheduledStimulus> {
        let mut inner = self.inner.lock().unwrap();
        if inner.schedules.is_empty() {
            return Vec::new();
        }
        let mut due = Vec::new();
        for (_, s) in inner.schedules.iter_mut() {
            let split = s.entries.partition_point(|e| e.burst <= burst_index);
            if split == 0 {
                continue;
            }
            let fired: Vec<ScheduledStimulus> = s.entries.drain(..split).collect();
            if let Some(period) = s.repeat_period {
                for e in &fired {
                    let behind = burst_index - e.burst;
                    let k = behind / period + 1;
                    let mut next = e.clone();
                    next.burst = e.burst.saturating_add(k.saturating_mul(period));
                    s.entries.push(next);
                }
                s.entries.sort_by_key(|e| e.burst);
            }
            due.extend(fired);
        }
        inner.schedules.retain(|(_, s)| !s.entries.is_empty());
        due
    }

    /// Cancel every schedule (genome reload / connectome reset).
    pub fn clear_all(&self) {
        self.inner.lock().unwrap().schedules.clear();
    }

    /// Drop every scheduled injection that targets `area` (area deleted). Entries left without
    /// areas are removed, and schedules left without entries are cancelled.
    pub fn clear_area(&self, area: &CorticalID) {
        let mut inner = self.inner.lock().unwrap();
        for (_, s) in inner.schedules.iter_mut() {
            for e in s.entries.iter_mut() {
                e.areas.retain(|(id, _)| id != area);
            }
            s.entries.retain(|e| !e.areas.is_empty());
        }
        inner.schedules.retain(|(_, s)| !s.entries.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(burst: u64) -> ScheduledStimulus {
        ScheduledStimulus {
            burst,
            areas: Vec::new(),
        }
    }

    #[test]
    fn drains_due_entries_and_drops_finished_schedules() {
        let store = StimulusScheduleStore::new();
        let (id, start, end) = store.add(vec![entry(7), entry(5)]).unwrap();
        assert_eq!((start, end), (5, 7));
        assert!(store.take_due(4).is_empty());
        assert_eq!(store.take_due(5).len(), 1);
        assert_eq!(store.list()[0].remaining_entries, 1);
        assert_eq!(store.take_due(9).len(), 1);
        assert!(store.list().is_empty());
        assert!(!store.cancel(id));
    }

    #[test]
    fn cancel_removes_remaining_entries() {
        let store = StimulusScheduleStore::new();
        let (id, _, _) = store.add(vec![entry(1), entry(2)]).unwrap();
        assert!(store.cancel(id));
        assert!(store.take_due(10).is_empty());
    }

    fn area_entry(burst: u64, id: &CorticalID) -> ScheduledStimulus {
        ScheduledStimulus {
            burst,
            areas: vec![(*id, vec![(0, 0, 0, 1.0)])],
        }
    }

    #[test]
    fn clear_all_drops_every_schedule() {
        let store = StimulusScheduleStore::new();
        store.add(vec![entry(1)]).unwrap();
        store.add_with_repeat(vec![entry(1)], Some(5)).unwrap();
        store.clear_all();
        assert!(store.list().is_empty());
        assert!(store.take_due(100).is_empty());
    }

    #[test]
    fn clear_area_drops_only_that_area_and_empty_entries() {
        let a = CorticalID::try_from_bytes(b"cstm0001").unwrap();
        let b = CorticalID::try_from_bytes(b"cstm0002").unwrap();
        let store = StimulusScheduleStore::new();
        store.add(vec![area_entry(3, &a)]).unwrap();
        store
            .add(vec![ScheduledStimulus {
                burst: 4,
                areas: vec![(a, vec![(0, 0, 0, 1.0)]), (b, vec![(0, 0, 0, 1.0)])],
            }])
            .unwrap();
        store.clear_area(&a);
        assert_eq!(store.list().len(), 1);
        let due = store.take_due(10);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].areas.len(), 1);
        assert_eq!(due[0].areas[0].0, b);
    }

    #[test]
    fn repeating_schedule_fires_every_period_until_cancelled() {
        let store = StimulusScheduleStore::new();
        let (id, start, end) = store
            .add_with_repeat(vec![entry(10), entry(12)], Some(5))
            .unwrap();
        assert_eq!((start, end), (10, None));
        let mut fired = Vec::new();
        for burst in 10..=26 {
            for d in store.take_due(burst) {
                fired.push((burst, d.burst));
            }
        }
        let bursts: Vec<u64> = fired.iter().map(|(b, _)| *b).collect();
        assert_eq!(bursts, vec![10, 12, 15, 17, 20, 22, 25]);
        assert!(fired.iter().all(|(b, e)| b == e));
        let info = &store.list()[0];
        assert_eq!(info.end_burst, None);
        assert_eq!(info.repeat_period_bursts, Some(5));
        assert_eq!(info.next_burst, Some(27));
        assert!(store.cancel(id));
        assert!(store.take_due(40).is_empty());
    }

    #[test]
    fn repeating_late_entry_skips_missed_slots() {
        let store = StimulusScheduleStore::new();
        store.add_with_repeat(vec![entry(10)], Some(5)).unwrap();
        assert_eq!(store.take_due(23).len(), 1);
        assert_eq!(store.list()[0].next_burst, Some(25));
    }

    #[test]
    fn repeat_period_must_exceed_entry_span() {
        let store = StimulusScheduleStore::new();
        assert!(store
            .add_with_repeat(vec![entry(0), entry(4)], Some(0))
            .is_none());
        assert!(store
            .add_with_repeat(vec![entry(0), entry(4)], Some(4))
            .is_none());
        assert!(store
            .add_with_repeat(vec![entry(0), entry(4)], Some(5))
            .is_some());
    }
}
