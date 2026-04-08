// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Sparse schedule of PSP contributions keyed by **arrival burst index**.
//!
//! See `docs/SYNAPTIC_DELAY_ARCHITECTURE.md`.

use ahash::{AHashMap, AHashSet};
use feagi_npu_neural::types::{FireCandidateList, NeuronId};

/// Pending FCL-style PSP and associative-memory PSP keyed by future burst number.
#[derive(Debug, Default, Clone)]
pub struct SynapticArrivalSchedule {
    /// `arrival_burst` -> per-neuron accumulated PSP delta (same merge rules as [`FireCandidateList`]).
    pub fcl_by_arrival_burst: AHashMap<u64, FireCandidateList>,
    /// `arrival_burst` -> memory-neuron-id -> associative PSP sum for that burst.
    pub memory_associative_by_arrival_burst: AHashMap<u64, AHashMap<u32, f32>>,
}

impl SynapticArrivalSchedule {
    /// Merge propagation output from fires at `fire_burst` into future buckets (`fire_burst + d`).
    pub fn schedule_from_delayed_propagation(
        &mut self,
        fire_burst: u64,
        fcl_by_delay: &AHashMap<u32, crate::synaptic_propagation::PropagationResult>,
        memory_by_delay: &AHashMap<u32, AHashMap<u32, f32>>,
    ) {
        for (&delay_bursts, area_map) in fcl_by_delay {
            let arrival = fire_burst.saturating_add(u64::from(delay_bursts));
            let bucket = self.fcl_by_arrival_burst.entry(arrival).or_default();
            for targets in area_map.values() {
                for &(nid, c) in targets {
                    bucket.add_candidate(nid, c.0);
                }
            }
        }
        for (&delay_bursts, mmap) in memory_by_delay {
            let arrival = fire_burst.saturating_add(u64::from(delay_bursts));
            let entry = self
                .memory_associative_by_arrival_burst
                .entry(arrival)
                .or_default();
            for (&k, &v) in mmap {
                *entry.entry(k).or_insert(0.0) += v;
            }
        }
    }

    /// Drain scheduled arrivals for `arrival_burst` into the live FCL and associative map.
    pub fn drain_into_phase1(
        &mut self,
        arrival_burst: u64,
        fcl: &mut FireCandidateList,
        memory_associative: &mut AHashMap<u32, f32>,
    ) {
        if let Some(scheduled_fcl) = self.fcl_by_arrival_burst.remove(&arrival_burst) {
            for (nid, pot) in scheduled_fcl.iter() {
                fcl.add_candidate(nid, pot);
            }
        }
        if let Some(m) = self
            .memory_associative_by_arrival_burst
            .remove(&arrival_burst)
        {
            for (k, v) in m {
                *memory_associative.entry(k).or_insert(0.0) += v;
            }
        }
    }

    /// Remove all pending PSP contributions whose **targets** are in `neuron_ids`.
    ///
    /// Used when resetting a cortical area so delayed synaptic arrivals (see
    /// `docs/SYNAPTIC_DELAY_ARCHITECTURE.md`) and associative-memory buckets do not inject
    /// residual drive on later bursts. `neuron_ids` is typically every regular neuron in the
    /// area and/or memory neuron ids being cleared by plasticity.
    pub fn remove_pending_for_neuron_targets(&mut self, neuron_ids: &AHashSet<u32>) {
        if neuron_ids.is_empty() {
            return;
        }
        for fcl in self.fcl_by_arrival_burst.values_mut() {
            for &nid in neuron_ids.iter() {
                fcl.remove_candidate(NeuronId(nid));
            }
        }
        self.fcl_by_arrival_burst.retain(|_, fcl| !fcl.is_empty());

        for mmap in self.memory_associative_by_arrival_burst.values_mut() {
            mmap.retain(|k, _| !neuron_ids.contains(k));
        }
        self.memory_associative_by_arrival_burst
            .retain(|_, m| !m.is_empty());
    }
}
