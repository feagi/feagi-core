// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Production plasticity runtime: applies the commands the plasticity service queued and runs
//! the memory/twin fire bridge once per burst.
//!
//! The burst loop calls [`PlasticityRuntime::apply_post_burst`] from its post-burst callback,
//! after the NPU lock is released. All handles are injected; the runtime never reaches for a
//! global connectome manager, so tests drive the same code the production loop runs.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use feagi_brain_development::models::CorticalAreaExt;
use feagi_brain_development::ConnectomeManager;
use feagi_npu_burst_engine::npu::MemoryReplayFrame;
use feagi_npu_burst_engine::{DynamicNPU, TracingMutex};
use feagi_npu_neural::types::NeuronId;
use feagi_npu_plasticity::AsyncPlasticityExecutor;
use feagi_structures::genomic::cortical_area::CorticalID;
use parking_lot::RwLock;
use tracing::{debug, warn};

/// Replay injection payload scheduled for a future burst.
#[derive(Debug, Clone)]
pub struct ReplayInjection {
    pub cortical_id: CorticalID,
    pub coords: Vec<(u32, u32, u32)>,
    pub potential: f32,
}

/// Memory/twin pairing and fire-bridge bookkeeping. One lock guards all of it.
#[derive(Debug, Default)]
struct BridgeState {
    /// memory neuron id -> twin neuron id
    ltm_twin_map: HashMap<u32, u32>,
    /// twin neuron id -> memory neuron id
    ltm_twin_reverse: HashMap<u32, u32>,
    /// memory neuron id -> memory area index
    ltm_memory_area: HashMap<u32, u32>,
    /// twin neuron id -> potential injected when the twin is made to fire
    ltm_twin_fire_potential: HashMap<u32, f32>,
    /// memory neuron id -> burst at which a bridge injection into it is expected back
    bridge_injected_memory: HashMap<u32, u64>,
    /// twin neuron id -> burst at which a bridge injection into it is expected back
    bridge_injected_twin: HashMap<u32, u64>,
}

impl BridgeState {
    /// Forget the twin pairing of one memory neuron. Returns the twin's neuron id if there was one.
    fn forget_twin(&mut self, memory_id: u32) -> Option<u32> {
        self.ltm_memory_area.remove(&memory_id);
        self.bridge_injected_memory.remove(&memory_id);
        let twin_id = self.ltm_twin_map.remove(&memory_id)?;
        self.ltm_twin_reverse.remove(&twin_id);
        self.ltm_twin_fire_potential.remove(&twin_id);
        self.bridge_injected_twin.remove(&twin_id);
        Some(twin_id)
    }

    /// Forget the pairings of memory neurons that are no longer stored. Returns the twin neuron
    /// ids (as connectome neuron ids) that now need deleting.
    fn retire_twins(&mut self, retired_memory_ids: &[u32]) -> Vec<u64> {
        retired_memory_ids
            .iter()
            .filter_map(|&memory_id| self.forget_twin(memory_id))
            .map(u64::from)
            .collect()
    }
}

/// Applies plasticity commands and runs the memory/twin fire bridge after each burst.
pub struct PlasticityRuntime {
    npu: Arc<TracingMutex<DynamicNPU>>,
    manager: Arc<RwLock<ConnectomeManager>>,
    executor: Arc<Mutex<AsyncPlasticityExecutor>>,
    state: Mutex<BridgeState>,
    replay_schedule: Arc<Mutex<BTreeMap<u64, Vec<ReplayInjection>>>>,
}

impl PlasticityRuntime {
    /// Build a runtime over injected handles. `executor` owns the plasticity service the
    /// commands are drained from.
    pub fn new(
        npu: Arc<TracingMutex<DynamicNPU>>,
        manager: Arc<RwLock<ConnectomeManager>>,
        executor: Arc<Mutex<AsyncPlasticityExecutor>>,
    ) -> Self {
        Self {
            npu,
            manager,
            executor,
            state: Mutex::new(BridgeState::default()),
            replay_schedule: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    /// Replay injections scheduled for future bursts.
    pub fn replay_schedule(&self) -> Arc<Mutex<BTreeMap<u64, Vec<ReplayInjection>>>> {
        Arc::clone(&self.replay_schedule)
    }

    /// Delete LTM twin neurons and their synapses in one batch. The batch is logged at debug
    /// level because it can run on many bursts.
    fn delete_twin_neurons(&self, twin_ids: Vec<u64>) {
        if twin_ids.is_empty() {
            return;
        }
        let twin_count = twin_ids.len();
        let deleted = self.manager.read().delete_neurons_batch_quiet(twin_ids);
        match deleted {
            Ok(count) if count == twin_count => {}
            Ok(count) => warn!(
                "[PLASTICITY-LTM] Deleted {} of {} LTM twin neuron(s) for removed memory neurons",
                count, twin_count
            ),
            Err(e) => warn!(
                "[PLASTICITY-LTM] Failed to delete {} LTM twin neuron(s): {}",
                twin_count, e
            ),
        }
    }

    /// Drain and apply queued plasticity commands, then run the memory/twin fire bridge.
    /// Call once per burst, after the NPU lock is released.
    pub fn apply_post_burst(&self, timestep: u64) {
        let callback_start = Instant::now();
        let Ok(exec) = self.executor.lock() else {
            warn!(
                "[PLASTICITY-CMD] Post-burst lock failed at burst {}",
                timestep
            );
            return;
        };
        let (commands, max_ops_per_burst, deferred_commands) = match exec.get_service() {
            Some(service) => {
                let max_ops = service.max_ops_per_burst();
                if max_ops == 0 {
                    warn!(
                        "[PLASTICITY-CMD] max_ops_per_burst is 0 at burst {} - skipping command processing",
                        timestep
                    );
                    (Vec::new(), max_ops, service.pending_command_count())
                } else {
                    let pending_before = service.pending_command_count();
                    let drained = service.dequeue_commands(max_ops);
                    let deferred = pending_before.saturating_sub(drained.len());
                    (drained, max_ops, deferred)
                }
            }
            None => {
                warn!(
                    "[PLASTICITY-CMD] Post-burst service unavailable at burst {}",
                    timestep
                );
                (Vec::new(), 0, 0)
            }
        };
        drop(exec);

        if deferred_commands > 0 {
            debug!(
                "[PLASTICITY-CMD] Burst {} command budget reached ({}/burst), deferred {} command(s)",
                timestep,
                max_ops_per_burst,
                deferred_commands
            );
        }
        let commands_processed = commands.len();
        let command_phase_start = Instant::now();
        let scheduled_replays: Vec<(u64, ReplayInjection)> = Vec::new();
        let mut dropped_unmapped_areas = HashSet::new();
        // Storage-side twins of long-term memory neurons that left the array this burst.
        let mut removed_twin_ids: Vec<u64> = Vec::new();
        if !commands.is_empty() {
            for cmd in commands {
                match cmd {
                    feagi_npu_plasticity::PlasticityCommand::RegisterMemoryNeuron {
                        neuron_id,
                        area_idx,
                        ..
                    } => {
                        // The neuron may have expired or been evicted after this command was
                        // queued. Registering it now would leave a mapping for a dead id.
                        let still_stored = self
                            .executor
                            .lock()
                            .ok()
                            .is_some_and(|exec| exec.is_active_memory_neuron(neuron_id));
                        if !still_stored {
                            continue;
                        }
                        let cortical_id_opt = {
                            let cm = self.manager.read();
                            cm.get_cortical_id(area_idx).cloned()
                        };
                        let Some(cortical_id) = cortical_id_opt else {
                            if dropped_unmapped_areas.insert(area_idx) {
                                warn!(
                                    "[PLASTICITY-CMD] Missing cortical ID for area_idx={}; dropping that memory registration",
                                    area_idx
                                );
                                if let Ok(exec) = self.executor.lock() {
                                    exec.unregister_memory_area(area_idx);
                                }
                            }
                            continue;
                        };
                        let mut npu_lock = self.npu.lock().unwrap();
                        npu_lock.register_dynamic_neuron_mapping(neuron_id, cortical_id);
                        debug!(
                            "[PLASTICITY-CMD] Registered memory neuron mapping id={} area_idx={}",
                            neuron_id, area_idx
                        );
                    }
                    feagi_npu_plasticity::PlasticityCommand::MemoryNeuronConvertedToLtm {
                        neuron_id,
                        area_idx,
                        pattern_hash: _,
                    } => {
                        let still_stored = self
                            .executor
                            .lock()
                            .ok()
                            .is_some_and(|exec| exec.is_active_memory_neuron(neuron_id));
                        if !still_stored {
                            debug!(
                                "[PLASTICITY-LTM] Skipping twin for memory neuron {} after area reset",
                                neuron_id
                            );
                            continue;
                        }
                        let mut state = self.state.lock().unwrap();
                        if state.ltm_twin_map.contains_key(&neuron_id) {
                            continue;
                        }

                        let cortical_id_opt = {
                            let cm = self.manager.read();
                            cm.get_cortical_id(area_idx).cloned()
                        };
                        let Some(cortical_id) = cortical_id_opt else {
                            warn!(
                                "[PLASTICITY-LTM] Missing cortical ID for area_idx={}",
                                area_idx
                            );
                            continue;
                        };

                        let (
                            threshold,
                            threshold_limit,
                            leak,
                            refractory,
                            excitability,
                            fire_limit,
                            snooze,
                            mp_charge,
                        ) = {
                            let cm = self.manager.read();
                            let Some(area) = cm.get_cortical_area(&cortical_id) else {
                                warn!(
                                    "[PLASTICITY-LTM] Missing cortical area for id={}",
                                    cortical_id.as_base_64()
                                );
                                continue;
                            };
                            (
                                area.firing_threshold(),
                                area.firing_threshold_limit(),
                                area.leak_coefficient(),
                                area.refractory_period(),
                                area.neuron_excitability(),
                                area.consecutive_fire_count() as u16,
                                area.snooze_period(),
                                area.mp_charge_accumulation(),
                            )
                        };

                        let twin_id_u64 = {
                            let mut cm = self.manager.write();
                            cm.add_auxiliary_neuron(
                                &cortical_id,
                                0,
                                0,
                                0,
                                threshold,
                                threshold_limit,
                                leak,
                                0.0,
                                0,
                                refractory,
                                excitability,
                                fire_limit,
                                snooze,
                                mp_charge,
                            )
                        };

                        let Ok(twin_id_u64) = twin_id_u64 else {
                            warn!(
                                "[PLASTICITY-LTM] Failed to create twin neuron for memory_id={}",
                                neuron_id
                            );
                            continue;
                        };
                        let Ok(twin_id) = u32::try_from(twin_id_u64) else {
                            warn!("[PLASTICITY-LTM] Twin neuron id overflow: {}", twin_id_u64);
                            continue;
                        };

                        state.ltm_twin_map.insert(neuron_id, twin_id);
                        state.ltm_twin_reverse.insert(twin_id, neuron_id);
                        state.ltm_memory_area.insert(neuron_id, area_idx);
                        state.ltm_twin_fire_potential.insert(twin_id, threshold);
                    }
                    feagi_npu_plasticity::PlasticityCommand::InjectMemoryNeuronToFCL {
                        neuron_id,
                        area_idx,
                        membrane_potential,
                        pattern_hash,
                        is_reactivation: _,
                        replay_frames,
                    } => {
                        let still_stored = self
                            .executor
                            .lock()
                            .ok()
                            .is_some_and(|exec| exec.is_active_memory_neuron(neuron_id));
                        if !still_stored {
                            debug!(
                                "[PLASTICITY-REPLAY] Skipping inject for removed memory neuron {}",
                                neuron_id
                            );
                            continue;
                        }
                        debug!(
                            "[PLASTICITY-REPLAY] Memory neuron inject: area_idx={} pattern={} replay_frames={}",
                            area_idx,
                            pattern_hash,
                            replay_frames.len()
                        );
                        let cortical_id_opt = {
                            let cm = self.manager.read();
                            cm.get_cortical_id(area_idx).cloned()
                        };

                        if let Some(cortical_id) = cortical_id_opt {
                            debug!(
                                "[PLASTICITY-CMD] Injected memory neuron id={} area_idx={} pattern={}",
                                neuron_id, area_idx, pattern_hash
                            );

                            if !replay_frames.is_empty() {
                                let frames: Vec<MemoryReplayFrame> = replay_frames
                                    .iter()
                                    .map(|frame| MemoryReplayFrame {
                                        offset: frame.offset,
                                        upstream_area_idx: frame.upstream_area_idx,
                                        coords: frame.coords.clone(),
                                        membrane_potentials: frame.membrane_potentials.clone(),
                                    })
                                    .collect();
                                let mut npu_lock = self.npu.lock().unwrap();
                                npu_lock.register_memory_replay_frames(neuron_id, frames);
                                debug!(
                                    "[PLASTICITY-REPLAY] Registered replay frames in NPU: neuron_id={} frames={}",
                                    neuron_id,
                                    replay_frames.len()
                                );
                            }
                            let mut npu_lock = self.npu.lock().unwrap();
                            npu_lock.register_dynamic_neuron_mapping(neuron_id, cortical_id);
                            npu_lock.inject_memory_neuron_to_fcl(
                                neuron_id,
                                area_idx,
                                membrane_potential,
                            );
                        } else if dropped_unmapped_areas.insert(area_idx) {
                            warn!(
                                "[PLASTICITY-CMD] Missing cortical ID for area_idx={}; dropping that memory registration",
                                area_idx
                            );
                            if let Ok(exec) = self.executor.lock() {
                                exec.unregister_memory_area(area_idx);
                            }
                        }
                    }
                    feagi_npu_plasticity::PlasticityCommand::UpdateWeightsDelta { .. } => {
                        warn!("[PLASTICITY-CMD] STDP weight updates not yet implemented");
                    }
                    feagi_npu_plasticity::PlasticityCommand::UpdateStateCounters { .. } => {}
                    feagi_npu_plasticity::PlasticityCommand::ResetMemoryNeuronsInArea {
                        cortical_idx,
                    } => {
                        debug!(
                            "[PLASTICITY-CMD] Memory neurons reset in cortical area {}",
                            cortical_idx
                        );
                    }
                    feagi_npu_plasticity::PlasticityCommand::MemoryNeuronRemoved {
                        neuron_id,
                        area_idx,
                        reason,
                    } => {
                        // The plasticity service already removed the memory neuron's synapses,
                        // dynamic mapping, replay frames and activation records. What is left
                        // is the storage-side twin a long-term neuron got, and the bridge
                        // bookkeeping that points at it. The retired-twin sweep may have
                        // forgotten the pair first; it deletes that twin itself, so this
                        // finds nothing then.
                        let twin_id = self.state.lock().unwrap().forget_twin(neuron_id);
                        if let Some(twin_id) = twin_id {
                            removed_twin_ids.push(u64::from(twin_id));
                        }
                        debug!(
                            "[PLASTICITY-CMD] Memory neuron {} removed from area {} ({:?}); twin={:?}",
                            neuron_id, area_idx, reason, twin_id
                        );
                    }
                }
            }
            // One batch per burst deletes the twins and their synapses.
            self.delete_twin_neurons(std::mem::take(&mut removed_twin_ids));
        }
        let command_phase_ms = command_phase_start.elapsed().as_secs_f64() * 1000.0;
        if command_phase_ms > 5.0 {
            debug!(
                "[PLASTICITY-CMD] Burst {} command phase took {:.2}ms",
                timestep, command_phase_ms
            );
        }

        let fire_queue_phase_start = Instant::now();
        let fire_queue_sample = {
            let mut npu_lock = self.npu.lock().unwrap();
            npu_lock.force_sample_fire_queue()
        };
        let fire_queue_phase_ms = fire_queue_phase_start.elapsed().as_secs_f64() * 1000.0;
        if fire_queue_phase_ms > 5.0 {
            debug!(
                "[PLASTICITY-CMD] Burst {} fire-queue sampling phase took {:.2}ms",
                timestep, fire_queue_phase_ms
            );
        }
        if let Some(sample) = fire_queue_sample {
            let ltm_bridge_phase_start = Instant::now();
            let mut state = self.state.lock().unwrap();
            let mut fired_memory: HashMap<u32, f32> = HashMap::new();
            let mut fired_twins: HashMap<u32, f32> = HashMap::new();

            for (_area_idx, (neuron_ids, _xs, _ys, _zs, potentials)) in sample {
                for (idx, neuron_id) in neuron_ids.iter().enumerate() {
                    let Some(potential) = potentials.get(idx).copied() else {
                        continue;
                    };
                    if state.ltm_twin_map.contains_key(neuron_id) {
                        fired_memory.insert(*neuron_id, potential);
                    }
                    if state.ltm_twin_reverse.contains_key(neuron_id) {
                        fired_twins.insert(*neuron_id, potential);
                    }
                }
            }

            let retired_memory_ids: Vec<u32> = match self.executor.lock() {
                Ok(exec) => state
                    .ltm_twin_map
                    .keys()
                    .copied()
                    .filter(|memory_id| !exec.is_active_memory_neuron(*memory_id))
                    .collect(),
                Err(_) => Vec::new(),
            };
            let retired_twins = state.retire_twins(&retired_memory_ids);
            // A memory neuron that left the array usually reaches this sweep before its
            // removal notice is drained. The notice then finds nothing, so the twin has to
            // be deleted here or it leaks with its synapses.
            self.delete_twin_neurons(retired_twins);
            let BridgeState {
                ltm_twin_map,
                ltm_memory_area,
                ltm_twin_fire_potential,
                bridge_injected_memory,
                bridge_injected_twin,
                ..
            } = &mut *state;
            let mut twin_injections: Vec<(NeuronId, f32)> = Vec::new();
            let mut memory_injections: Vec<(u32, u32, f32)> = Vec::new();

            for (memory_id, twin_id) in ltm_twin_map.iter() {
                let memory_fired = fired_memory.get(memory_id);
                let twin_fired = fired_twins.get(twin_id);

                if memory_fired.is_some() && twin_fired.is_some() {
                    continue;
                }

                if let Some(mem_potential) = memory_fired {
                    if bridge_injected_memory.remove(memory_id) == Some(timestep) {
                        continue;
                    }
                    let Some(twin_potential) = ltm_twin_fire_potential.get(twin_id) else {
                        warn!(
                            "[PLASTICITY-LTM] Missing twin potential for twin_id={}",
                            twin_id
                        );
                        continue;
                    };
                    let inject_potential = mem_potential.max(*twin_potential);
                    twin_injections.push((NeuronId(*twin_id), inject_potential));
                    bridge_injected_twin.insert(*twin_id, timestep + 1);
                }

                if let Some(twin_potential) = twin_fired {
                    if bridge_injected_twin.remove(twin_id) == Some(timestep) {
                        continue;
                    }
                    let Some(area_idx) = ltm_memory_area.get(memory_id) else {
                        warn!("[PLASTICITY-LTM] Missing area for memory_id={}", memory_id);
                        continue;
                    };
                    memory_injections.push((*memory_id, *area_idx, *twin_potential));
                    bridge_injected_memory.insert(*memory_id, timestep + 1);
                }
            }

            if !twin_injections.is_empty() || !memory_injections.is_empty() {
                let mut npu_lock = self.npu.lock().unwrap();
                if !twin_injections.is_empty() {
                    npu_lock.inject_sensory_with_potentials(&twin_injections);
                }
                if !memory_injections.is_empty() {
                    for (memory_id, area_idx, potential) in memory_injections {
                        npu_lock.inject_memory_neuron_to_fcl(memory_id, area_idx, potential);
                    }
                }
            }
            let ltm_bridge_phase_ms = ltm_bridge_phase_start.elapsed().as_secs_f64() * 1000.0;
            if ltm_bridge_phase_ms > 5.0 {
                debug!(
                    "[PLASTICITY-CMD] Burst {} LTM bridge phase took {:.2}ms",
                    timestep, ltm_bridge_phase_ms
                );
            }
        }

        if !scheduled_replays.is_empty() {
            let mut schedule = self.replay_schedule.lock().unwrap();
            for (target_burst, injection) in scheduled_replays {
                schedule.entry(target_burst).or_default().push(injection);
            }
        }

        let next_burst = timestep + 1;
        let replay_injections = {
            let mut schedule = self.replay_schedule.lock().unwrap();
            schedule.remove(&next_burst)
        };
        if let Some(injections) = replay_injections {
            debug!(
                "[PLASTICITY-REPLAY] Executing {} replay injections at burst {}",
                injections.len(),
                next_burst
            );
            let mut npu_lock = self.npu.lock().unwrap();
            for injection in injections {
                let xyzp_data: Vec<(u32, u32, u32, f32)> = injection
                    .coords
                    .iter()
                    .map(|(x, y, z)| (*x, *y, *z, injection.potential))
                    .collect();
                debug!(
                    "[PLASTICITY-REPLAY] Injecting coords={} into twin_id={}",
                    xyzp_data.len(),
                    injection.cortical_id.as_base_64()
                );
                npu_lock.inject_sensory_xyzp_by_id(&injection.cortical_id, &xyzp_data);
            }
        } else {
            debug!(
                "[PLASTICITY-REPLAY] No replay injections scheduled for burst {}",
                next_burst
            );
        }

        let callback_elapsed_ms = callback_start.elapsed().as_secs_f64() * 1000.0;
        if callback_elapsed_ms > 20.0 {
            warn!(
                "[PLASTICITY-CMD] Post-burst callback for burst {} took {:.2}ms (commands_processed={}, deferred={})",
                timestep,
                callback_elapsed_ms,
                commands_processed,
                deferred_commands
            );
        } else if callback_elapsed_ms > 5.0 {
            debug!(
                "[PLASTICITY-CMD] Post-burst callback for burst {} took {:.2}ms (commands_processed={}, deferred={})",
                timestep,
                callback_elapsed_ms,
                commands_processed,
                deferred_commands
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_hands_back_the_twin_and_the_later_notice_finds_nothing() {
        let mut s = BridgeState {
            ltm_twin_map: HashMap::from([(50_000_001u32, 7u32), (50_000_002, 8)]),
            ltm_twin_reverse: HashMap::from([(7u32, 50_000_001u32), (8, 50_000_002)]),
            ltm_memory_area: HashMap::from([(50_000_001u32, 3u32), (50_000_002, 3)]),
            ltm_twin_fire_potential: HashMap::from([(7u32, 1.0f32), (8, 1.0)]),
            bridge_injected_memory: HashMap::from([(50_000_001u32, 9u64)]),
            bridge_injected_twin: HashMap::from([(7u32, 9u64)]),
        };

        // The sweep sees memory neuron 50_000_001 gone: its twin must be queued for deletion.
        let twins = s.retire_twins(&[50_000_001]);
        assert_eq!(twins, vec![7u64]);
        assert!(
            !s.ltm_twin_map.contains_key(&50_000_001) && s.ltm_twin_map.contains_key(&50_000_002)
        );
        assert!(!s.ltm_twin_reverse.contains_key(&7) && s.ltm_twin_reverse.contains_key(&8));
        assert!(
            !s.ltm_twin_fire_potential.contains_key(&7) && !s.bridge_injected_twin.contains_key(&7)
        );
        assert!(
            !s.ltm_memory_area.contains_key(&50_000_001)
                && !s.bridge_injected_memory.contains_key(&50_000_001)
        );

        // The removal notice arrives afterwards: no twin left, so no second delete.
        assert_eq!(s.forget_twin(50_000_001), None);

        // A notice for a neuron the sweep has not seen yet still returns its twin.
        assert_eq!(s.forget_twin(50_000_002), Some(8));
        assert!(s.retire_twins(&[50_000_002]).is_empty());
    }
}
