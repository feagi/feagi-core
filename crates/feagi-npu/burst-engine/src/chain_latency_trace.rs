// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Optional tracing: pairwise chain latency and/or **all neurons that fire** in a traced cortical area.
//!
//! @npu-debug-instrumentation: remove or fold into observability API after root-cause.
//!
//! Enable with:
//! - `FEAGI_NPU_TRACE_CHAIN_UPSTREAM=<u32>` and/or `FEAGI_NPU_TRACE_CHAIN_DOWNSTREAM=<u32>`
//! - `FEAGI_NPU_TRACE_AREA_FIRE_IDS=1` — per burst, log every neuron id that fired in the same
//!   cortical index as FCL/dynamics trace ([`crate::neural_dynamics::trace_fcl_cortical_idx_for_logging`]).
//!   Includes `deltas_bursts_since_last_self_fire` for each id (no coordinates; post-neurogenesis ids).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Once, OnceLock};

use ahash::AHashMap;
use parking_lot::Mutex;

use crate::fire_structures::FireQueue;
use crate::neural_dynamics::trace_fcl_cortical_idx_for_logging;

fn last_self_fire_map() -> &'static Mutex<AHashMap<u32, u64>> {
    static LAST: OnceLock<Mutex<AHashMap<u32, u64>>> = OnceLock::new();
    LAST.get_or_init(|| Mutex::new(AHashMap::new()))
}

const UNSET_BURST: u64 = u64::MAX;

struct ChainTraceCfg {
    upstream: Option<u32>,
    downstream: Option<u32>,
}

fn chain_trace_cfg() -> &'static ChainTraceCfg {
    static CFG: OnceLock<ChainTraceCfg> = OnceLock::new();
    CFG.get_or_init(|| {
        let upstream = std::env::var("FEAGI_NPU_TRACE_CHAIN_UPSTREAM")
            .ok()
            .and_then(|s| s.parse().ok());
        let downstream = std::env::var("FEAGI_NPU_TRACE_CHAIN_DOWNSTREAM")
            .ok()
            .and_then(|s| s.parse().ok());

        static WARN_ONCE: Once = Once::new();
        if upstream.is_some() ^ downstream.is_some() {
            WARN_ONCE.call_once(|| {
                tracing::warn!(
                    target: "feagi-npu-trace",
                    "Set both FEAGI_NPU_TRACE_CHAIN_UPSTREAM and FEAGI_NPU_TRACE_CHAIN_DOWNSTREAM for full chain latency metrics."
                );
            });
        }

        ChainTraceCfg {
            upstream,
            downstream,
        }
    })
}

static LAST_UPSTREAM_BURST: AtomicU64 = AtomicU64::new(UNSET_BURST);
static LAST_DOWNSTREAM_BURST: AtomicU64 = AtomicU64::new(UNSET_BURST);

fn timestep_secs_per_burst() -> f64 {
    let ns = crate::sim_timestep().as_nanos();
    if ns == 0 {
        return 0.0;
    }
    (ns as f64) / 1e9
}

fn area_fire_ids_enabled() -> bool {
    static CFG: OnceLock<bool> = OnceLock::new();
    *CFG.get_or_init(|| {
        std::env::var("FEAGI_NPU_TRACE_AREA_FIRE_IDS")
            .ok()
            .as_deref()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    })
}

/// One line per burst when any neuron in the traced cortical area fired: ids + burst deltas since
/// each neuron's previous fire (same area trace as FCL / dynamics cortical_idx filter).
pub(crate) fn emit_area_cortical_fire_detail(burst_count: u64, fire_queue: &FireQueue) {
    if !area_fire_ids_enabled() {
        return;
    }
    let Some(area_idx) = trace_fcl_cortical_idx_for_logging() else {
        return;
    };
    let Some(neurons) = fire_queue.neurons_by_area.get(&area_idx) else {
        return;
    };
    if neurons.is_empty() {
        return;
    }

    let mut ids: Vec<u32> = neurons.iter().map(|n| n.neuron_id.0).collect();
    ids.sort_unstable();

    let mut deltas: Vec<Option<u64>> = Vec::with_capacity(ids.len());
    {
        let mut map = last_self_fire_map().lock();
        for nid in &ids {
            let prev = map.insert(*nid, burst_count);
            deltas.push(prev.map(|p| burst_count.saturating_sub(p)));
        }
    }

    tracing::debug!(
        target: "feagi-npu-trace",
        burst = burst_count,
        cortical_idx = area_idx,
        neuron_ids = ?ids,
        deltas_bursts_since_last_self_fire = ?deltas,
        "AREA_FIRES"
    );
}

/// Emit at most two debug lines (upstream fire, downstream fire) for this burst.
pub(crate) fn emit_chain_latency_trace(burst_count: u64, fire_queue: &FireQueue) {
    let cfg = chain_trace_cfg();
    if cfg.upstream.is_none() && cfg.downstream.is_none() {
        return;
    }

    let mut saw_upstream = false;
    let mut saw_downstream = false;
    for n in fire_queue.neurons_by_area.values().flatten() {
        if Some(n.neuron_id.0) == cfg.upstream {
            saw_upstream = true;
        }
        if Some(n.neuron_id.0) == cfg.downstream {
            saw_downstream = true;
        }
    }

    // Apply upstream before downstream so same-burst fires get delta_bursts=0.
    if saw_upstream {
        if let Some(uid) = cfg.upstream {
            LAST_UPSTREAM_BURST.store(burst_count, Ordering::Release);
            tracing::debug!(
                target: "feagi-npu-trace",
                burst = burst_count,
                neuron = uid,
                "CHAIN upstream_fired"
            );
        }
    }

    if saw_downstream {
        if let Some(did) = cfg.downstream {
            let last_u = LAST_UPSTREAM_BURST.load(Ordering::Acquire);
            let delta_bursts = if last_u != UNSET_BURST {
                Some(burst_count.saturating_sub(last_u))
            } else {
                None
            };
            let delta_seconds_since_upstream_approx = delta_bursts.map(|d| {
                let t = timestep_secs_per_burst();
                (d as f64) * t
            });

            let prev_dn = LAST_DOWNSTREAM_BURST.swap(burst_count, Ordering::AcqRel);
            let delta_bursts_since_prev_downstream = if prev_dn != UNSET_BURST {
                Some(burst_count.saturating_sub(prev_dn))
            } else {
                None
            };

            tracing::debug!(
                target: "feagi-npu-trace",
                burst = burst_count,
                neuron = did,
                delta_bursts_since_upstream = ?delta_bursts,
                delta_seconds_since_upstream_approx = ?delta_seconds_since_upstream_approx,
                delta_bursts_since_prev_downstream = ?delta_bursts_since_prev_downstream,
                "CHAIN downstream_fired"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fire_structures::{FireQueue, FiringNeuron, FIRE_KIND_STDP_ELIGIBLE};
    use feagi_npu_neural::types::NeuronId;

    #[test]
    fn emit_empty_queue_no_panic() {
        let fq = FireQueue::new();
        emit_chain_latency_trace(0, &fq);
        emit_area_cortical_fire_detail(0, &fq);
    }

    #[test]
    fn emit_with_neurons_no_panic_without_env() {
        let mut fq = FireQueue::new();
        fq.add_neuron(FiringNeuron {
            neuron_id: NeuronId(1),
            membrane_potential: 1.0,
            cortical_idx: 0,
            x: 0,
            y: 0,
            z: 0,
            fire_kind: FIRE_KIND_STDP_ELIGIBLE,
        });
        emit_chain_latency_trace(1, &fq);
        emit_area_cortical_fire_detail(1, &fq);
    }
}
