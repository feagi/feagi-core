// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Plasticity Service - orchestrates STDP and memory formation
//!
//! RTOS-friendly design:
//! - No sleeps/timeouts; uses condition variables
//! - Read-only access to firing history
//! - Mutations are enqueued as commands

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crate::log_rate_limiter::BurstLogRateLimiter;
use crate::memory_neuron_array::{
    MemoryNeuronArray, MemoryNeuronDetail, MemoryNeuronLifecycleConfig,
};
use crate::memory_stats_cache::{self, MemoryStatsCache};
use crate::pattern_detector::{BatchPatternDetector, PatternConfig};
use crate::stdp::STDPConfig;
use ahash::AHashSet;
use feagi_npu_neural::types::NeuronId;
use serde::{Deserialize, Serialize};

/// Default burst gap between repeated MP-unavailable warnings.
pub const DEFAULT_MP_UNAVAILABLE_WARN_PERIOD_BURSTS: u64 = 100;

type MpWindowFrame = (u64, ahash::AHashMap<u32, f32>);
type PerAreaMpWindow = (u32, Vec<MpWindowFrame>);

// State manager access for fatigue reporting
// TODO: Add feagi_state_manager dependency when wiring up state manager access
// #[cfg(feature = "std")]
// use feagi_state_manager::MemoryMappedState;

/// Plasticity configuration
#[derive(Debug, Clone)]
pub struct PlasticityConfig {
    /// Queue capacity for commands
    pub queue_capacity: usize,

    /// Maximum operations per burst
    pub max_ops_per_burst: usize,

    /// Memory neuron array capacity
    pub memory_array_capacity: usize,

    /// STDP configuration
    pub stdp: Option<STDPConfig>,

    /// Pattern detection configuration
    pub pattern_config: PatternConfig,

    /// Memory neuron lifecycle configuration
    pub memory_lifecycle_config: MemoryNeuronLifecycleConfig,

    /// Minimum burst gap between repeated "MP window unavailable" warnings
    /// for the same upstream area. `0` logs every burst.
    pub mp_unavailable_warn_period_bursts: u64,
}

impl Default for PlasticityConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 1000,
            max_ops_per_burst: 100,
            memory_array_capacity: 50000,
            stdp: Some(STDPConfig::default()),
            pattern_config: PatternConfig::default(),
            memory_lifecycle_config: MemoryNeuronLifecycleConfig::default(),
            mp_unavailable_warn_period_bursts: DEFAULT_MP_UNAVAILABLE_WARN_PERIOD_BURSTS,
        }
    }
}

/// Plasticity command types
#[derive(Debug, Clone)]
pub enum PlasticityCommand {
    /// Update synaptic weights with deltas
    UpdateWeightsDelta {
        synapse_indices: Vec<usize>,
        deltas: Vec<f32>,
    },

    /// Notification that a memory neuron was created/reactivated in MemoryNeuronArray
    /// Memory neurons are stored separately from regular neurons (not in NPU neuron array)
    /// This command is for logging/stats only
    RegisterMemoryNeuron {
        neuron_id: u32,
        area_idx: u32,
        threshold: f32,
        membrane_potential: f32,
    },

    /// Notification that a memory neuron has converted to long-term memory (LTM).
    /// Used to create a persistent associative twin in the standard neuron array.
    MemoryNeuronConvertedToLtm {
        neuron_id: u32,
        area_idx: u32,
        pattern_hash: u64,
    },

    /// Inject memory neuron to Fire Candidate List for immediate firing
    /// Memory neurons bypass threshold checks and fire when their pattern is detected
    InjectMemoryNeuronToFCL {
        neuron_id: u32,
        area_idx: u32,
        membrane_potential: f32,
        pattern_hash: u64,
        is_reactivation: bool,
        replay_frames: Vec<ReplayFrame>,
    },

    /// Update state counters
    UpdateStateCounters {
        memory_neurons_created: usize,
        current_memory_neuron_count: usize,
        area_idx: u32,
        neuron_id: u32,
    },

    /// Reset (delete) all memory neurons and their synapses in a cortical area
    ResetMemoryNeuronsInArea { cortical_idx: u32 },
}

/// Replay frame describing a single temporal slice for an upstream area.
#[derive(Debug, Clone)]
pub struct ReplayFrame {
    pub offset: u32,
    pub upstream_area_idx: u32,
    pub coords: Vec<(u32, u32, u32)>,
    /// Per-coordinate membrane potential at encoding time.
    /// Present only when mp_learning_enabled=true for the memory area.
    /// Length matches `coords` when present.
    pub membrane_potentials: Option<Vec<f32>>,
}

/// Memory area configuration
#[derive(Debug, Clone)]
pub struct MemoryAreaConfig {
    pub temporal_depth: u32,
    pub upstream_areas: Vec<u32>,
    pub mp_learning_enabled: bool,
}

/// Runtime counts for a memory cortical area (plasticity layer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCorticalAreaRuntimeInfo {
    pub short_term_neuron_count: usize,
    pub long_term_neuron_count: usize,
    pub upstream_pattern_cache_size: usize,
}

impl MemoryCorticalAreaRuntimeInfo {
    /// Total active memory neurons in the plasticity layer (ST + LT).
    pub fn active_memory_neuron_count(&self) -> usize {
        self.short_term_neuron_count + self.long_term_neuron_count
    }
}

/// Plasticity service statistics
#[derive(Debug, Clone, Default)]
pub struct PlasticityStats {
    pub memory_patterns_detected: usize,
    pub memory_neurons_created: usize,
    pub memory_neurons_reactivated: usize,
    pub memory_neurons_aged: usize,
    pub memory_neurons_converted_ltm: usize,
    pub plasticity_commands_enqueued: usize,
    pub plasticity_commands_dropped: usize,
}

/// Plasticity service - independent thread that computes plasticity every burst
#[derive(Clone)]
pub struct PlasticityService {
    config: PlasticityConfig,

    // NPU reference (for querying CPU-resident FireLedger)
    npu: Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,

    // Pattern detection
    pattern_detector: BatchPatternDetector,

    // Memory neuron array
    memory_neuron_array: Arc<Mutex<MemoryNeuronArray>>,

    // Memory area tracking
    memory_areas: Arc<Mutex<HashMap<u32, MemoryAreaConfig>>>,
    memory_lifecycle_configs: Arc<Mutex<HashMap<u32, MemoryNeuronLifecycleConfig>>>,
    memory_area_names: Arc<Mutex<HashMap<u32, String>>>, // area_idx -> area_name

    // Thread synchronization
    cv: Arc<(Mutex<(bool, u64)>, Condvar)>, // (running, latest_timestep)

    // Command queue
    command_queue: Arc<Mutex<Vec<PlasticityCommand>>>,

    // Statistics
    stats: Arc<Mutex<PlasticityStats>>,

    // Memory area stats cache (for health check)
    memory_stats_cache: MemoryStatsCache,

    /// Rate-limits repeating MP-unavailable warnings (keyed by upstream area idx).
    mp_window_warn_limiter: Arc<Mutex<BurstLogRateLimiter>>,
}

impl PlasticityService {
    /// Create a new plasticity service with stats cache and NPU reference
    pub fn new(
        config: PlasticityConfig,
        memory_stats_cache: MemoryStatsCache,
        npu: Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
    ) -> Self {
        let pattern_detector = BatchPatternDetector::new(config.pattern_config.clone());
        let memory_array_capacity = config.memory_array_capacity;

        let memory_neuron_array =
            Arc::new(Mutex::new(MemoryNeuronArray::new(memory_array_capacity)));

        {
            let mem_assoc = Arc::clone(&memory_neuron_array);
            let mem_ltm = Arc::clone(&memory_neuron_array);
            let guard = npu.lock().unwrap();
            guard.set_memory_neuron_assoc_predicate(Some(Arc::new(move |id: u32| {
                mem_assoc
                    .lock()
                    .unwrap()
                    .get_memory_neuron_detail(id)
                    .map(|d| d.is_active)
                    .unwrap_or(false)
            })));
            guard.set_memory_neuron_longterm_predicate(Some(Arc::new(move |id: u32| {
                mem_ltm
                    .lock()
                    .unwrap()
                    .get_memory_neuron_detail(id)
                    .map(|d| d.is_longterm_memory)
                    .unwrap_or(false)
            })));
        }

        let mp_warn_period = config.mp_unavailable_warn_period_bursts;
        Self {
            config,
            npu,
            pattern_detector,
            memory_neuron_array,
            memory_areas: Arc::new(Mutex::new(HashMap::new())),
            memory_lifecycle_configs: Arc::new(Mutex::new(HashMap::new())),
            memory_area_names: Arc::new(Mutex::new(HashMap::new())),
            cv: Arc::new((Mutex::new((false, 0)), Condvar::new())),
            command_queue: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(PlasticityStats::default())),
            memory_stats_cache,
            mp_window_warn_limiter: Arc::new(Mutex::new(BurstLogRateLimiter::new(mp_warn_period))),
        }
    }

    /// Get the memory stats cache (for wiring to health check)
    pub fn get_memory_stats_cache(&self) -> MemoryStatsCache {
        self.memory_stats_cache.clone()
    }

    /// Notify service of new burst
    pub fn notify_burst(&self, timestep: u64) {
        // trace!("[PLASTICITY-SVC] 🔔 Burst {} notification received, waking compute thread", timestep);
        let (lock, cvar) = &*self.cv;
        let mut data = lock.lock().unwrap();
        data.0 = true; // ✅ Set flag to true so thread wakes up!
        data.1 = timestep;
        cvar.notify_all();
    }

    /// Start the plasticity service thread
    pub fn start(&self) -> thread::JoinHandle<()> {
        let cv = Arc::clone(&self.cv);
        let command_queue = Arc::clone(&self.command_queue);
        let memory_neuron_array = Arc::clone(&self.memory_neuron_array);
        let memory_areas = Arc::clone(&self.memory_areas);
        let memory_lifecycle_configs = Arc::clone(&self.memory_lifecycle_configs);
        let memory_area_names = Arc::clone(&self.memory_area_names);
        let pattern_detector = self.pattern_detector.clone();
        let stats = Arc::clone(&self.stats);
        let config = self.config.clone();
        let memory_stats_cache = self.memory_stats_cache.clone();
        let npu = Arc::clone(&self.npu); // Clone NPU reference for thread
        let mp_window_warn_limiter = Arc::clone(&self.mp_window_warn_limiter);

        tracing::info!(target: "plasticity", "🧠 Starting PlasticityService background thread...");

        thread::spawn(move || {
            tracing::info!(target: "plasticity", "✓ PlasticityService thread started - waiting for burst notifications");

            let (lock, cvar) = &*cv;

            loop {
                let timestep = {
                    let mut data = lock.lock().unwrap();
                    while !data.0 {
                        data = cvar.wait(data).unwrap();
                    }
                    data.0 = false; // ✅ Reset flag so we wait again after processing
                    data.1
                };

                // trace!("[PLASTICITY-THREAD] 💤➡️🏃 Woke up for burst {}, starting compute_plasticity", timestep);

                // Compute plasticity
                Self::compute_plasticity(
                    timestep,
                    &npu,
                    &memory_neuron_array,
                    &memory_areas,
                    &memory_lifecycle_configs,
                    &memory_area_names,
                    &pattern_detector,
                    &command_queue,
                    &stats,
                    &config,
                    &memory_stats_cache,
                    &mp_window_warn_limiter,
                );
            }
        })
    }

    /// Stop the plasticity service
    pub fn stop(&self) {
        let (lock, cvar) = &*self.cv;
        let mut data = lock.lock().unwrap();
        data.0 = false;
        cvar.notify_all();
    }

    /// Compute plasticity for current burst
    #[allow(clippy::too_many_arguments)]
    fn compute_plasticity(
        current_timestep: u64,
        npu: &Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
        memory_neuron_array: &Arc<Mutex<MemoryNeuronArray>>,
        memory_areas: &Arc<Mutex<HashMap<u32, MemoryAreaConfig>>>,
        memory_lifecycle_configs: &Arc<Mutex<HashMap<u32, MemoryNeuronLifecycleConfig>>>,
        memory_area_names: &Arc<Mutex<HashMap<u32, String>>>,
        pattern_detector: &BatchPatternDetector,
        command_queue: &Arc<Mutex<Vec<PlasticityCommand>>>,
        stats: &Arc<Mutex<PlasticityStats>>,
        config: &PlasticityConfig,
        memory_stats_cache: &MemoryStatsCache,
        mp_window_warn_limiter: &Arc<Mutex<BurstLogRateLimiter>>,
    ) {
        let memory_areas_snapshot = memory_areas.lock().unwrap().clone();

        // Log plasticity status every 100 bursts
        if current_timestep % 100 == 0 {
            if memory_areas_snapshot.is_empty() {
                // This is normal if plasticity isn't being used - log at debug level instead of warn
                tracing::debug!(target: "plasticity",
                    "[PLASTICITY] Burst {} - No memory areas registered (plasticity not in use)",
                    current_timestep
                );
            } else {
                tracing::debug!(target: "plasticity",
                    "[PLASTICITY] ✓ Burst {} - Monitoring {} memory area(s)",
                    current_timestep,
                    memory_areas_snapshot.len()
                );
            }
        }

        if memory_areas_snapshot.is_empty() {
            // Early return - plasticity service is running but no memory areas registered
            // This means plasticity will NEVER acquire NPU lock, so it's not the cause of lock contention
            return;
        }

        let mut commands = Vec::new();
        let mut array = memory_neuron_array.lock().unwrap();

        // Step 1: Check for long-term memory conversion BEFORE aging.
        //
        // Rationale:
        // If a neuron’s lifespan is already at/above the long-term threshold (e.g., init=100, threshold=100),
        // we must convert it before decrementing lifespan; otherwise it becomes 99 and never qualifies.
        let converted_neurons = {
            let lifecycle_configs = memory_lifecycle_configs.lock().unwrap();
            array.check_longterm_conversion_by_area(
                &lifecycle_configs,
                config.memory_lifecycle_config.longterm_threshold,
            )
        };
        if !converted_neurons.is_empty() {
            let mut s = stats.lock().unwrap();
            s.memory_neurons_converted_ltm += converted_neurons.len();
            drop(s);

            for neuron_idx in converted_neurons {
                let neuron_id = array.get_neuron_id(neuron_idx);
                let area_idx = array.get_cortical_area_id(neuron_idx);
                let pattern_hash = array.get_pattern_hash(neuron_idx);
                if let (Some(neuron_id), Some(area_idx), Some(pattern_hash)) =
                    (neuron_id, area_idx, pattern_hash)
                {
                    commands.push(PlasticityCommand::MemoryNeuronConvertedToLtm {
                        neuron_id,
                        area_idx,
                        pattern_hash,
                    });
                } else {
                    tracing::warn!(
                        target: "plasticity",
                        "[PLASTICITY] LTM conversion missing metadata for idx={}",
                        neuron_idx
                    );
                }
            }
        }

        // Step 2: Age all memory neurons (non-long-term only)
        let died_neurons = array.age_memory_neurons(current_timestep);
        if !died_neurons.is_empty() {
            let mut s = stats.lock().unwrap();
            s.memory_neurons_aged += died_neurons.len();
            drop(s);

            // Update memory stats cache for deleted neurons (group by area)
            let area_names_map = memory_area_names.lock().unwrap();
            let mut area_death_counts: HashMap<u32, usize> = HashMap::new();

            for died_idx in died_neurons {
                if let Some(area_idx) = array.get_cortical_area_id(died_idx) {
                    *area_death_counts.entry(area_idx).or_insert(0) += 1;
                }
            }

            for (area_idx, count) in area_death_counts {
                if let Some(area_name) = area_names_map.get(&area_idx) {
                    for _ in 0..count {
                        memory_stats_cache::on_neuron_deleted(memory_stats_cache, area_name);
                    }
                }
            }

            // Update memory utilization in state manager after deletions
            Self::update_memory_utilization_in_state_manager(&array, config);
        }

        // Step 3: Detect patterns for all memory areas
        // Query CPU-resident FireLedger for upstream area firing history
        for (memory_area_idx, area_config) in memory_areas_snapshot.iter() {
            tracing::trace!(
                target: "plasticity",
                "Burst {} - Processing memory area {} with {} upstream areas: {:?}",
                current_timestep,
                memory_area_idx,
                area_config.upstream_areas.len(),
                area_config.upstream_areas
            );

            // Query FireLedger for upstream firing history (CPU-resident, dense burst-aligned windows)
            let plasticity_lock_start = std::time::Instant::now();
            tracing::debug!(
                "[NPU-LOCK] PLASTICITY: Acquiring NPU lock for FireLedger query (burst {}, area {})",
                current_timestep,
                memory_area_idx
            );
            let (timestep_bitmaps, windows, mp_windows) = {
                let temporal_depth = area_config.temporal_depth as usize;

                // Brief lock to query FireLedger - data is already CPU-resident from burst processing
                let npu_lock = npu.lock().unwrap();
                let plasticity_lock_wait = plasticity_lock_start.elapsed();
                tracing::debug!(
                    "[NPU-LOCK] PLASTICITY: Lock acquired (waited {:.2}ms, burst {}, area {})",
                    plasticity_lock_wait.as_secs_f64() * 1000.0,
                    current_timestep,
                    memory_area_idx
                );
                tracing::debug!(
                    "[NPU-LOCK] PLASTICITY: Slow lock acquisition: {:.2}ms (burst {})",
                    plasticity_lock_wait.as_secs_f64() * 1000.0,
                    current_timestep
                );

                let result = if temporal_depth == 0 || area_config.upstream_areas.is_empty() {
                    (Vec::new(), Vec::new(), None)
                } else {
                    // Deterministic: upstream areas are processed in sorted order so hashing is stable.
                    let mut upstream_sorted = area_config.upstream_areas.clone();
                    upstream_sorted.sort_unstable();

                    // Fetch a dense window for each upstream area (same [t-D+1..t] range).
                    let mut windows: Vec<(u32, Vec<(u64, roaring::RoaringBitmap)>)> = Vec::new();
                    let mut windows_ok = true;
                    for &upstream_area_idx in &upstream_sorted {
                        let window = match npu_lock.get_fire_ledger_dense_window_bitmaps(
                            upstream_area_idx,
                            current_timestep,
                            temporal_depth,
                        ) {
                            Ok(w) => w,
                            Err(e) => {
                                tracing::trace!(
                                    target: "plasticity",
                                    "Burst {} - Upstream area {} dense window unavailable (depth={}): {}",
                                    current_timestep,
                                    upstream_area_idx,
                                    temporal_depth,
                                    e
                                );
                                windows_ok = false;
                                break;
                            }
                        };
                        let frame_counts: Vec<u64> =
                            window.iter().map(|(_, bm)| bm.len()).collect();
                        let total_fired: u64 = frame_counts.iter().sum();
                        tracing::trace!(
                            target: "plasticity",
                            "Burst {} - Upstream area {} window covers {}..{} ({} frames) fired_counts={:?} total_fired={}",
                            current_timestep,
                            upstream_area_idx,
                            window.first().map(|(t, _)| *t).unwrap_or(0),
                            window.last().map(|(t, _)| *t).unwrap_or(0),
                            window.len(),
                            frame_counts,
                            total_fired
                        );
                        windows.push((upstream_area_idx, window));
                    }

                    if !windows_ok || windows.is_empty() {
                        (Vec::new(), Vec::new(), None)
                    } else {
                        // Validate alignment: all upstream windows must share the same timesteps.
                        let reference_timesteps: Vec<u64> =
                            windows[0].1.iter().map(|(t, _)| *t).collect();
                        let mut aligned = true;
                        for (area_idx, w) in &windows[1..] {
                            let ts: Vec<u64> = w.iter().map(|(t, _)| *t).collect();
                            if ts != reference_timesteps {
                                aligned = false;
                                tracing::warn!(target: "plasticity",
                                    "[PLASTICITY] Misaligned FireLedger windows for memory area {} at burst {}: upstream {} timesteps {:?} != {:?}",
                                    memory_area_idx, current_timestep, area_idx, ts, reference_timesteps
                                );
                                break;
                            }
                        }

                        if !aligned {
                            (Vec::new(), Vec::new(), None)
                        } else {
                            // Flatten as: for each timestep (oldest->newest), for each upstream area (sorted),
                            // append that area's fired-neuron set at that timestep.
                            let mut out: Vec<HashSet<u32>> = Vec::with_capacity(
                                reference_timesteps.len() * upstream_sorted.len(),
                            );
                            for frame_i in 0..reference_timesteps.len() {
                                for (_area_idx, w) in &windows {
                                    let (_t, bitmap) = &w[frame_i];
                                    let neuron_set: HashSet<u32> = bitmap.iter().collect();
                                    out.push(neuron_set);
                                }
                            }

                            // Fetch MP windows if mp_learning_enabled
                            let mp_data = if area_config.mp_learning_enabled {
                                let mut mp_wins: Vec<PerAreaMpWindow> = Vec::new();
                                for &upstream_idx in &upstream_sorted {
                                    match npu_lock.get_fire_ledger_dense_window_mp(
                                        upstream_idx,
                                        current_timestep,
                                        temporal_depth,
                                    ) {
                                        Ok(mp_window) => mp_wins.push((upstream_idx, mp_window)),
                                        Err(e) => {
                                            let emit = mp_window_warn_limiter
                                                .lock()
                                                .unwrap()
                                                .should_emit(upstream_idx, current_timestep);
                                            if let Some(suppressed) = emit {
                                                if suppressed == 0 {
                                                    tracing::warn!(
                                                        target: "plasticity",
                                                        "[PLASTICITY] MP window unavailable for upstream {}: {}",
                                                        upstream_idx,
                                                        e
                                                    );
                                                } else {
                                                    tracing::warn!(
                                                        target: "plasticity",
                                                        "[PLASTICITY] MP window unavailable for upstream {}: {} (suppressed {} repeats)",
                                                        upstream_idx,
                                                        e,
                                                        suppressed
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                                if mp_wins.is_empty() {
                                    None
                                } else {
                                    Some(mp_wins)
                                }
                            } else {
                                None
                            };

                            (out, windows, mp_data)
                        }
                    }
                };

                // Log lock hold time before release
                let plasticity_lock_hold = plasticity_lock_start.elapsed();
                tracing::debug!(
                    "[NPU-LOCK] PLASTICITY: Lock held for {:.2}ms (burst {}) - releasing now",
                    plasticity_lock_hold.as_secs_f64() * 1000.0,
                    current_timestep
                );
                // Lock is released here when npu_lock goes out of scope
                drop(npu_lock);
                tracing::debug!(
                    "[NPU-LOCK] PLASTICITY: Lock RELEASED (burst {}, total hold: {:.2}ms)",
                    current_timestep,
                    plasticity_lock_hold.as_secs_f64() * 1000.0
                );
                result
            };

            if timestep_bitmaps.is_empty() {
                tracing::trace!(
                    target: "plasticity",
                    "Burst {} - Memory area {} has no firing history from upstream areas - skipping",
                    current_timestep,
                    memory_area_idx
                );
                // No firing history available for upstream areas - skip pattern detection
                continue;
            }

            tracing::trace!(
                target: "plasticity",
                "Burst {} - Memory area {} has {} timestep bitmaps for pattern detection",
                current_timestep,
                memory_area_idx,
                timestep_bitmaps.len()
            );

            let detector =
                pattern_detector.get_detector(*memory_area_idx, area_config.temporal_depth);

            if let Some(pattern) = detector.detect_pattern(
                *memory_area_idx,
                &area_config.upstream_areas,
                current_timestep,
                timestep_bitmaps,
                Some(area_config.temporal_depth),
            ) {
                let replay_frames = Self::build_replay_frames(npu, &windows, mp_windows.as_deref());
                tracing::debug!(
                    target: "plasticity",
                    "[PLASTICITY] Burst {} pattern detected area={} hash={} upstream={} replay_frames={}",
                    current_timestep,
                    memory_area_idx,
                    pattern.pattern_hash,
                    area_config.upstream_areas.len(),
                    replay_frames.len()
                );
                let mut s = stats.lock().unwrap();
                s.memory_patterns_detected += 1;
                drop(s);

                // Check if pattern already has a memory neuron
                if let Some(existing_neuron_idx) =
                    array.find_neuron_by_pattern(&pattern.pattern_hash)
                {
                    // Reactivate existing neuron
                    if array.reactivate_memory_neuron(existing_neuron_idx, current_timestep) {
                        let mut s = stats.lock().unwrap();
                        s.memory_neurons_reactivated += 1;
                        drop(s);

                        let neuron_id = array.get_neuron_id(existing_neuron_idx).unwrap();

                        // EMA averaging of membrane potentials on reactivation
                        let final_replay_frames = if area_config.mp_learning_enabled {
                            Self::average_replay_frame_mps(npu, neuron_id, &replay_frames)
                        } else {
                            replay_frames.clone()
                        };

                        // Register and inject reactivated neuron
                        commands.push(PlasticityCommand::RegisterMemoryNeuron {
                            neuron_id,
                            area_idx: *memory_area_idx,
                            threshold: 1.5,
                            membrane_potential: 0.0,
                        });

                        if final_replay_frames.is_empty() {
                            tracing::warn!(
                                target: "plasticity",
                                "[PLASTICITY] Burst {} reactivation area={} neuron_id={} has empty replay frames",
                                current_timestep,
                                memory_area_idx,
                                neuron_id
                            );
                        }
                        commands.push(PlasticityCommand::InjectMemoryNeuronToFCL {
                            neuron_id,
                            area_idx: *memory_area_idx,
                            membrane_potential: 1.5,
                            pattern_hash: pattern.pattern_hash,
                            is_reactivation: true,
                            replay_frames: final_replay_frames,
                        });

                        let total_memory = array.get_stats().active_neurons;
                        commands.push(PlasticityCommand::UpdateStateCounters {
                            memory_neurons_created: 0,
                            current_memory_neuron_count: total_memory,
                            area_idx: *memory_area_idx,
                            neuron_id,
                        });
                    }
                } else {
                    // Create new memory neuron
                    tracing::debug!(target: "plasticity",
                        "[PLASTICITY] 🧠 Creating NEW memory neuron for pattern {} in area {}",
                        pattern.pattern_hash, memory_area_idx
                    );

                    let lifecycle_config = memory_lifecycle_configs
                        .lock()
                        .unwrap()
                        .get(memory_area_idx)
                        .copied()
                        .unwrap_or_default();

                    if let Some(neuron_idx) = array.create_memory_neuron(
                        pattern.pattern_hash,
                        *memory_area_idx,
                        current_timestep,
                        &lifecycle_config,
                    ) {
                        tracing::debug!(target: "plasticity",
                            "[PLASTICITY] ✓ Memory neuron created: idx={}, pattern={}",
                            neuron_idx, pattern.pattern_hash
                        );

                        let mut s = stats.lock().unwrap();
                        s.memory_neurons_created += 1;
                        drop(s);

                        // Update memory stats cache
                        if let Some(area_name) =
                            memory_area_names.lock().unwrap().get(memory_area_idx)
                        {
                            memory_stats_cache::on_neuron_created(memory_stats_cache, area_name);
                        }

                        let neuron_id = array.get_neuron_id(neuron_idx).unwrap();

                        // Register and inject new neuron
                        tracing::trace!(target: "plasticity",
                            "[PLASTICITY] 📤 Queueing commands: RegisterMemoryNeuron(id={}) + InjectMemoryNeuronToFCL(id={}, potential=1.5)",
                            neuron_id, neuron_id
                        );

                        commands.push(PlasticityCommand::RegisterMemoryNeuron {
                            neuron_id,
                            area_idx: *memory_area_idx,
                            threshold: 1.0,
                            membrane_potential: 0.0,
                        });

                        if replay_frames.is_empty() {
                            tracing::warn!(
                                target: "plasticity",
                                "[PLASTICITY] Burst {} new memory neuron area={} neuron_id={} has empty replay frames",
                                current_timestep,
                                memory_area_idx,
                                neuron_id
                            );
                        }
                        commands.push(PlasticityCommand::InjectMemoryNeuronToFCL {
                            neuron_id,
                            area_idx: *memory_area_idx,
                            membrane_potential: 1.5,
                            pattern_hash: pattern.pattern_hash,
                            is_reactivation: false,
                            replay_frames,
                        });

                        commands.push(PlasticityCommand::UpdateStateCounters {
                            memory_neurons_created: 1,
                            current_memory_neuron_count: array.get_stats().active_neurons,
                            area_idx: *memory_area_idx,
                            neuron_id,
                        });

                        // Update memory utilization in state manager after creation
                        Self::update_memory_utilization_in_state_manager(&array, config);
                    } else {
                        // Get diagnostic information to understand failure cause
                        let array_stats = array.get_stats();
                        let id_stats = array.get_id_allocation_stats();
                        tracing::warn!(target: "plasticity",
                            "[PLASTICITY] ⚠️  Failed to create memory neuron for pattern {} in area {} - Array: {}/{} active ({} LTM, {} reusable), ID: {}/{} allocated",
                            pattern.pattern_hash,
                            memory_area_idx,
                            array_stats.active_neurons,
                            array_stats.total_capacity,
                            array_stats.longterm_neurons,
                            array_stats.reusable_indices,
                            id_stats.memory_allocated,
                            id_stats.memory_capacity
                        );
                    }
                }
            } else {
                tracing::trace!(
                    target: "plasticity",
                    "Burst {} - No pattern detected for memory area {}",
                    current_timestep,
                    memory_area_idx
                );
            }
        }

        // Enqueue commands
        if !commands.is_empty() {
            let cmd_count = commands.len();
            let mut queue = command_queue.lock().unwrap();
            let mut s = stats.lock().unwrap();

            if queue.len() + cmd_count <= config.queue_capacity {
                queue.extend(commands);
                s.plasticity_commands_enqueued += cmd_count;
            } else {
                s.plasticity_commands_dropped += cmd_count;
            }
        }
    }

    /// Build replay frames from dense upstream windows for pattern reconstruction.
    /// When `mp_windows` is Some, membrane potentials are captured per coordinate.
    fn build_replay_frames(
        npu: &Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
        windows: &[(u32, Vec<(u64, roaring::RoaringBitmap)>)],
        mp_windows: Option<&[PerAreaMpWindow]>,
    ) -> Vec<ReplayFrame> {
        if windows.is_empty() {
            return Vec::new();
        }

        let npu_lock = npu.lock().unwrap();
        let mut frames = Vec::new();
        let mut empty_bitmaps = 0usize;
        let mut missing_coords = 0usize;
        for (upstream_area_idx, window) in windows {
            let mp_window_for_area = mp_windows.and_then(|mw| {
                mw.iter()
                    .find(|(idx, _)| idx == upstream_area_idx)
                    .map(|(_, w)| w)
            });

            for (offset, (_timestep, bitmap)) in window.iter().enumerate() {
                if bitmap.is_empty() {
                    empty_bitmaps += 1;
                    continue;
                }

                let mp_map = mp_window_for_area
                    .and_then(|w| w.get(offset))
                    .map(|(_, mp)| mp);

                let mut entries: Vec<((u32, u32, u32), Option<f32>)> = bitmap
                    .iter()
                    .filter_map(|neuron_id| {
                        let coord = npu_lock.get_neuron_coordinates(neuron_id)?;
                        let mp = mp_map.and_then(|m| m.get(&neuron_id).copied());
                        Some((coord, mp))
                    })
                    .collect();
                if entries.is_empty() {
                    missing_coords += 1;
                    continue;
                }
                entries.sort_unstable_by_key(|(coord, _)| *coord);

                let coords: Vec<(u32, u32, u32)> = entries.iter().map(|(c, _)| *c).collect();
                let membrane_potentials = if mp_map.is_some() {
                    Some(entries.iter().map(|(_, mp)| mp.unwrap_or(0.0)).collect())
                } else {
                    None
                };

                frames.push(ReplayFrame {
                    offset: offset as u32,
                    upstream_area_idx: *upstream_area_idx,
                    coords,
                    membrane_potentials,
                });
            }
        }
        tracing::debug!(
            target: "plasticity",
            "[PLASTICITY] Replay frames built frames={} empty_bitmaps={} missing_coords={}",
            frames.len(),
            empty_bitmaps,
            missing_coords
        );

        frames
    }

    /// Average membrane potentials between stored replay frames and new ones (EMA alpha=0.5).
    /// Retrieves existing frames from NPU, averages with new frames, returns merged result.
    fn average_replay_frame_mps(
        npu: &Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
        neuron_id: u32,
        new_frames: &[ReplayFrame],
    ) -> Vec<ReplayFrame> {
        let npu_lock = npu.lock().unwrap();
        let stored = match npu_lock.get_memory_replay_frames(neuron_id) {
            Some(arc) => arc,
            None => return new_frames.to_vec(),
        };
        drop(npu_lock);

        if stored.len() != new_frames.len() {
            return new_frames.to_vec();
        }

        new_frames
            .iter()
            .zip(stored.iter())
            .map(|(new_frame, old_frame)| {
                let membrane_potentials = match (
                    &new_frame.membrane_potentials,
                    &old_frame.membrane_potentials,
                ) {
                    (Some(new_mps), Some(old_mps)) if new_mps.len() == old_mps.len() => Some(
                        new_mps
                            .iter()
                            .zip(old_mps.iter())
                            .map(|(new_mp, old_mp)| (old_mp + new_mp) / 2.0)
                            .collect(),
                    ),
                    (Some(new_mps), None) => Some(new_mps.clone()),
                    _ => new_frame.membrane_potentials.clone(),
                };
                ReplayFrame {
                    offset: new_frame.offset,
                    upstream_area_idx: new_frame.upstream_area_idx,
                    coords: new_frame.coords.clone(),
                    membrane_potentials,
                }
            })
            .collect()
    }

    /// Register a memory area for pattern detection
    pub fn register_memory_area(
        &self,
        area_idx: u32,
        area_name: String,
        temporal_depth: u32,
        upstream_areas: Vec<u32>,
        lifecycle_config: Option<MemoryNeuronLifecycleConfig>,
        mp_learning_enabled: bool,
    ) -> bool {
        let upstream_len = upstream_areas.len();
        let upstream_clone = upstream_areas.clone();
        let mut areas = self.memory_areas.lock().unwrap();
        areas.insert(
            area_idx,
            MemoryAreaConfig {
                temporal_depth,
                upstream_areas,
                mp_learning_enabled,
            },
        );

        let mut names = self.memory_area_names.lock().unwrap();
        names.insert(area_idx, area_name.clone());

        if let Some(config) = lifecycle_config {
            let mut configs = self.memory_lifecycle_configs.lock().unwrap();
            configs.insert(area_idx, self.sanitize_lifecycle_config(config));
        }

        // Ensure FireLedger tracks upstream areas for the requested temporal depth (STDP ledger).
        // Also track the memory cortical area on the episodic memory FireLedger (pattern-injection fires).
        if let Ok(mut npu) = self.npu.lock() {
            let desired = temporal_depth as usize;
            let existing_configs = npu.get_all_fire_ledger_configs();
            for upstream_idx in upstream_clone {
                let existing = existing_configs
                    .iter()
                    .find(|(idx, _)| *idx == upstream_idx)
                    .map(|(_, w)| *w)
                    .unwrap_or(0);
                let resolved = existing.max(desired);
                if resolved != existing {
                    if let Err(e) = npu.configure_fire_ledger_window(upstream_idx, resolved) {
                        tracing::warn!(
                            target: "plasticity",
                            "[PLASTICITY] Failed to configure FireLedger window for upstream {} (requested={}): {}",
                            upstream_idx,
                            resolved,
                            e
                        );
                    }
                }
            }

            let existing_episodic = npu.get_all_episodic_memory_fire_ledger_configs();
            let existing_mem = existing_episodic
                .iter()
                .find(|(idx, _)| *idx == area_idx)
                .map(|(_, w)| *w)
                .unwrap_or(0);
            let resolved_mem = existing_mem.max(desired);
            if resolved_mem != existing_mem {
                if let Err(e) =
                    npu.configure_episodic_memory_fire_ledger_window(area_idx, resolved_mem)
                {
                    tracing::warn!(
                        target: "plasticity",
                        "[PLASTICITY] Failed to configure episodic memory FireLedger for area {} (requested={}): {}",
                        area_idx,
                        resolved_mem,
                        e
                    );
                }
            }

            // Enable MP archival on upstream areas when mp_learning_enabled is true
            if mp_learning_enabled {
                let upstream_for_mp = areas
                    .get(&area_idx)
                    .map(|c| c.upstream_areas.clone())
                    .unwrap_or_default();
                for upstream_idx in upstream_for_mp {
                    if let Err(e) = npu.enable_fire_ledger_mp_archival(upstream_idx) {
                        tracing::warn!(
                            target: "plasticity",
                            "[PLASTICITY] Failed to enable MP archival for upstream {} (memory area {}): {}",
                            upstream_idx,
                            area_idx,
                            e
                        );
                    }
                }
                tracing::info!(
                    target: "plasticity",
                    "[PLASTICITY] MP learning enabled for memory area {} - upstream MP archival active",
                    area_idx
                );
            }
        } else {
            tracing::warn!(
                target: "plasticity",
                "[PLASTICITY] Failed to lock NPU for FireLedger configuration"
            );
        }

        // Initialize cache entry for this area
        memory_stats_cache::init_memory_area(&self.memory_stats_cache, &area_name);

        tracing::info!(
            target: "plasticity",
            "[PLASTICITY] Registered memory area: idx={} name={} depth={} upstream={}",
            area_idx,
            area_name,
            temporal_depth,
            upstream_len
        );

        true
    }

    /// Normalize lifecycle values for runtime safety.
    ///
    /// Zero values can leak in from legacy or partially populated memory-area payloads.
    /// In those cases, fall back to the active plasticity config defaults so ST/LT
    /// lifecycle remains deterministic and visible.
    fn sanitize_lifecycle_config(
        &self,
        mut config: MemoryNeuronLifecycleConfig,
    ) -> MemoryNeuronLifecycleConfig {
        let defaults = self.config.memory_lifecycle_config;
        if config.initial_lifespan == 0 {
            config.initial_lifespan = defaults.initial_lifespan;
        }
        if config.lifespan_growth_rate <= 0.0 {
            config.lifespan_growth_rate = defaults.lifespan_growth_rate;
        }
        if config.longterm_threshold == 0 {
            config.longterm_threshold = defaults.longterm_threshold;
        }
        if config.max_reactivations == 0 {
            config.max_reactivations = defaults.max_reactivations;
        }
        config
    }

    /// Dequeue plasticity commands
    pub fn dequeue_commands(&self, max_count: usize) -> Vec<PlasticityCommand> {
        let mut queue = self.command_queue.lock().unwrap();
        let count = queue.len().min(max_count);
        queue.drain(..count).collect()
    }

    /// Return number of pending commands in the plasticity queue.
    pub fn pending_command_count(&self) -> usize {
        self.command_queue.lock().unwrap().len()
    }

    /// Return configured per-burst command processing budget.
    pub fn max_ops_per_burst(&self) -> usize {
        self.config.max_ops_per_burst
    }

    /// Get statistics
    pub fn get_stats(&self) -> PlasticityStats {
        self.stats.lock().unwrap().clone()
    }

    /// Drain all pending commands from the queue
    /// This should be called after each burst to process plasticity commands
    pub fn drain_commands(&self) -> Vec<PlasticityCommand> {
        let mut queue = self.command_queue.lock().unwrap();
        let drained = queue.drain(..).collect::<Vec<_>>();
        if !drained.is_empty() {
            tracing::debug!(
                target: "plasticity",
                "[PLASTICITY-SVC] Drained {} command(s) for execution",
                drained.len()
            );
            for command in &drained {
                match command {
                    PlasticityCommand::RegisterMemoryNeuron {
                        neuron_id,
                        area_idx,
                        ..
                    } => {
                        tracing::debug!(
                            target: "plasticity",
                            "[PLASTICITY-SVC] RegisterMemoryNeuron area={} neuron_id={}",
                            area_idx,
                            neuron_id
                        );
                    }
                    PlasticityCommand::MemoryNeuronConvertedToLtm {
                        neuron_id,
                        area_idx,
                        ..
                    } => {
                        tracing::info!(
                            target: "plasticity",
                            "[PLASTICITY-SVC] MemoryNeuronConvertedToLtm area={} neuron_id={}",
                            area_idx,
                            neuron_id
                        );
                    }
                    PlasticityCommand::InjectMemoryNeuronToFCL {
                        neuron_id,
                        area_idx,
                        is_reactivation,
                        replay_frames,
                        ..
                    } => {
                        tracing::debug!(
                            target: "plasticity",
                            "[PLASTICITY-SVC] InjectMemoryNeuronToFCL area={} neuron_id={} reactivation={} replay_frames={}",
                            area_idx,
                            neuron_id,
                            is_reactivation,
                            replay_frames.len()
                        );
                        if replay_frames.is_empty() {
                            tracing::warn!(
                                target: "plasticity",
                                "[PLASTICITY-SVC] InjectMemoryNeuronToFCL area={} neuron_id={} has empty replay frames",
                                area_idx,
                                neuron_id
                            );
                        }
                    }
                    PlasticityCommand::UpdateWeightsDelta { .. } => {}
                    PlasticityCommand::UpdateStateCounters { .. } => {}
                    PlasticityCommand::ResetMemoryNeuronsInArea { cortical_idx } => {
                        tracing::debug!(
                            target: "plasticity",
                            "[PLASTICITY-SVC] ResetMemoryNeuronsInArea cortical_idx={}",
                            cortical_idx
                        );
                    }
                }
            }
        }
        drained
    }

    pub fn enqueue_commands_for_test(&self, commands: Vec<PlasticityCommand>) {
        let mut queue = self.command_queue.lock().unwrap();
        queue.extend(commands);
    }

    /// Get memory neuron array reference
    pub fn get_memory_neuron_array(&self) -> Arc<Mutex<MemoryNeuronArray>> {
        Arc::clone(&self.memory_neuron_array)
    }

    /// Active long-term memory neurons only (STM is excluded).
    pub fn export_long_term_memory_neurons(&self) -> Vec<MemoryNeuronDetail> {
        self.memory_neuron_array
            .lock()
            .unwrap()
            .export_long_term_memory_neurons()
    }

    /// Replace in-memory STM/LTM state with restored long-term memory neurons.
    pub fn restore_long_term_memory_neurons(
        &self,
        neurons: &[MemoryNeuronDetail],
    ) -> Result<usize, String> {
        self.memory_neuron_array
            .lock()
            .unwrap()
            .restore_long_term_memory_neurons(neurons)
    }

    /// Reset (delete) all memory neurons and their synapses in a cortical area.
    ///
    /// Returns the number of memory neurons deleted.
    pub fn reset_memory_neurons_in_area(&self, cortical_idx: u32) -> usize {
        let area_name = self
            .memory_area_names
            .lock()
            .unwrap()
            .get(&cortical_idx)
            .cloned();

        let mut array = self.memory_neuron_array.lock().unwrap();

        // Get all memory neuron IDs in this area before deleting
        let memory_neuron_ids = array.get_active_neurons_by_area(cortical_idx);

        tracing::info!(
            target: "plasticity",
            "[PLASTICITY] Resetting {} memory neurons in cortical area {}",
            memory_neuron_ids.len(),
            cortical_idx
        );

        // Delete all synapses from/to these memory neurons
        if !memory_neuron_ids.is_empty() {
            let mut npu_lock = self.npu.lock().unwrap();
            let scrub_ids: AHashSet<u32> = memory_neuron_ids.iter().copied().collect();
            npu_lock.scrub_synaptic_arrival_schedule_for_neuron_targets(&scrub_ids);
            for &neuron_id in &memory_neuron_ids {
                // Delete outgoing synapses
                let outgoing = npu_lock.get_outgoing_synapses(neuron_id);
                for (target_id, _, _, _) in outgoing {
                    npu_lock.remove_synapse(NeuronId(neuron_id), NeuronId(target_id));
                }

                // Delete incoming synapses
                let incoming = npu_lock.get_incoming_synapses(neuron_id);
                for (source_id, _, _, _) in incoming {
                    npu_lock.remove_synapse(NeuronId(source_id), NeuronId(neuron_id));
                }
            }
        }

        // Delete the memory neurons themselves
        let reset_count = array.reset_cortical_area(cortical_idx);
        drop(array);

        if reset_count > 0 {
            if let Some(area_name) = area_name {
                for _ in 0..reset_count {
                    memory_stats_cache::on_neuron_deleted(&self.memory_stats_cache, &area_name);
                }
            }
            let array = self.memory_neuron_array.lock().unwrap();
            Self::update_memory_utilization_in_state_manager(&array, &self.config);
        }

        tracing::info!(
            target: "plasticity",
            "[PLASTICITY] Reset complete: deleted {} memory neurons and their synapses from area {}",
            reset_count,
            cortical_idx
        );

        reset_count
    }

    /// ST/LTM counts and pattern-detector cache size for a memory cortical area index.
    pub fn memory_cortical_area_runtime_info(
        &self,
        cortical_idx: u32,
    ) -> MemoryCorticalAreaRuntimeInfo {
        let array = self.memory_neuron_array.lock().unwrap();
        let upstream_pattern_cache_size = self
            .pattern_detector
            .cached_pattern_count_for_area(cortical_idx);
        MemoryCorticalAreaRuntimeInfo {
            short_term_neuron_count: array.count_short_term_in_area(cortical_idx),
            long_term_neuron_count: array.count_long_term_in_area(cortical_idx),
            upstream_pattern_cache_size,
        }
    }

    /// Lookup plasticity-layer detail for a memory neuron id.
    pub fn memory_neuron_detail(&self, neuron_id: u32) -> Option<MemoryNeuronDetail> {
        let array = self.memory_neuron_array.lock().unwrap();
        array.get_memory_neuron_detail(neuron_id)
    }

    /// Update memory neuron utilization in state manager
    ///
    /// Calculates memory neuron utilization percentage and updates the state manager.
    /// This should be called after memory neuron creation/deletion operations.
    ///
    /// # Arguments
    ///
    /// * `array` - Reference to the memory neuron array
    /// * `config` - Plasticity configuration containing memory array capacity
    #[cfg(feature = "std")]
    fn update_memory_utilization_in_state_manager(
        array: &MemoryNeuronArray,
        config: &PlasticityConfig,
    ) {
        let stats = array.get_stats();
        let memory_neuron_count = stats.active_neurons;
        let memory_neuron_capacity = config.memory_array_capacity;

        let memory_neuron_util = if memory_neuron_capacity > 0 {
            ((memory_neuron_count as f64 / memory_neuron_capacity as f64) * 100.0).round() as u8
        } else {
            0
        };

        // Update state manager with memory neuron utilization
        // Note: ConnectomeManager will read this value when it recalculates fatigue index
        // (triggered by neuron/synapse operations, not directly from here to avoid circular dependency)
        #[cfg(feature = "feagi-state-manager")]
        {
            use feagi_state_manager::StateManager;
            if let Some(state_manager) = StateManager::instance().try_write() {
                state_manager
                    .get_core_state()
                    .set_memory_neuron_util(memory_neuron_util);
            }
        }

        tracing::trace!(
            target: "plasticity",
            "[FATIGUE] Memory neuron utilization: {}% ({}/{} active)",
            memory_neuron_util, memory_neuron_count, memory_neuron_capacity
        );
    }
}

// BatchPatternDetector Clone is implemented in pattern_detector.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_stats_cache::create_memory_stats_cache;
    use feagi_npu_burst_engine::backend::CPUBackend;
    use feagi_npu_burst_engine::DynamicNPU;
    use feagi_npu_burst_engine::TracingMutex;
    use feagi_npu_runtime::StdRuntime;
    use std::sync::Arc;

    #[test]
    fn test_plasticity_service_creation() {
        let config = PlasticityConfig::default();
        let cache = create_memory_stats_cache();
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 16, 16, 8).unwrap(),
            "plasticity-test-npu",
        ));
        let service = PlasticityService::new(config, cache, npu);

        let stats = service.get_stats();
        assert_eq!(stats.memory_neurons_created, 0);
    }

    #[test]
    fn test_mp_unavailable_warn_period_default_is_configured() {
        let config = PlasticityConfig::default();
        assert_eq!(
            config.mp_unavailable_warn_period_bursts,
            DEFAULT_MP_UNAVAILABLE_WARN_PERIOD_BURSTS
        );
        assert!(config.mp_unavailable_warn_period_bursts > 0);
    }

    #[test]
    fn test_register_memory_area() {
        let config = PlasticityConfig::default();
        let cache = create_memory_stats_cache();
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 16, 16, 8).unwrap(),
            "plasticity-test-npu",
        ));
        let service = PlasticityService::new(config, cache, npu);

        let result =
            service.register_memory_area(100, "mem_00".to_string(), 3, vec![1, 2], None, false);
        assert!(result);

        let areas = service.memory_areas.lock().unwrap();
        assert!(areas.contains_key(&100));
    }

    #[test]
    fn test_ltm_conversion_preserves_active_memory_neuron_count() {
        let config = PlasticityConfig::default();
        let cache = create_memory_stats_cache();
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 16, 16, 8).unwrap(),
            "plasticity-test-npu",
        ));
        let service = PlasticityService::new(config, cache, npu);
        service.register_memory_area(100, "mem_ltm_test".to_string(), 1, vec![1], None, false);

        {
            let mut array = service.memory_neuron_array.lock().unwrap();
            let lifecycle = MemoryNeuronLifecycleConfig {
                longterm_threshold: 2,
                initial_lifespan: 2,
                ..Default::default()
            };
            assert!(array.create_memory_neuron(1, 100, 0, &lifecycle).is_some());
            assert!(array.create_memory_neuron(2, 100, 0, &lifecycle).is_some());
            let converted = array.check_longterm_conversion(2);
            assert_eq!(converted.len(), 2);
        }

        let runtime = service.memory_cortical_area_runtime_info(100);
        assert_eq!(runtime.short_term_neuron_count, 0);
        assert_eq!(runtime.long_term_neuron_count, 2);
        assert_eq!(runtime.active_memory_neuron_count(), 2);
    }

    #[test]
    fn test_reset_memory_neurons_syncs_stats_cache() {
        let config = PlasticityConfig::default();
        let cache = create_memory_stats_cache();
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 16, 16, 8).unwrap(),
            "plasticity-test-npu",
        ));
        let service = PlasticityService::new(config, cache.clone(), npu);
        let area_name = "mem_reset_test";
        service.register_memory_area(100, area_name.to_string(), 1, vec![1], None, false);

        memory_stats_cache::on_neuron_created(&cache, area_name);
        memory_stats_cache::on_neuron_created(&cache, area_name);
        memory_stats_cache::on_neuron_created(&cache, area_name);

        {
            let mut array = service.memory_neuron_array.lock().unwrap();
            let lifecycle = MemoryNeuronLifecycleConfig::default();
            for pattern in 1..=3 {
                assert!(array
                    .create_memory_neuron(pattern, 100, 0, &lifecycle)
                    .is_some());
            }
        }

        let reset_count = service.reset_memory_neurons_in_area(100);
        assert_eq!(reset_count, 3);

        let runtime = service.memory_cortical_area_runtime_info(100);
        assert_eq!(runtime.active_memory_neuron_count(), 0);
        assert_eq!(
            memory_stats_cache::get_area_stats(&cache, area_name)
                .map(|s| s.neuron_count)
                .unwrap_or(0),
            0
        );
    }

    #[test]
    fn test_register_memory_area_sanitizes_zero_lifecycle_values() {
        let config = PlasticityConfig::default();
        let cache = create_memory_stats_cache();
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 16, 16, 8).unwrap(),
            "plasticity-test-npu",
        ));
        let service = PlasticityService::new(config.clone(), cache, npu);
        let zeroed = MemoryNeuronLifecycleConfig {
            initial_lifespan: 0,
            lifespan_growth_rate: 0.0,
            longterm_threshold: 0,
            max_reactivations: 0,
        };
        assert!(service.register_memory_area(
            100,
            "mem_sanitized".to_string(),
            1,
            vec![1],
            Some(zeroed),
            false,
        ));

        let lifecycle = service
            .memory_lifecycle_configs
            .lock()
            .unwrap()
            .get(&100)
            .copied()
            .expect("missing lifecycle config");
        assert_eq!(
            lifecycle.initial_lifespan,
            config.memory_lifecycle_config.initial_lifespan
        );
        assert_eq!(
            lifecycle.lifespan_growth_rate,
            config.memory_lifecycle_config.lifespan_growth_rate
        );
        assert_eq!(
            lifecycle.longterm_threshold,
            config.memory_lifecycle_config.longterm_threshold
        );
        assert_eq!(
            lifecycle.max_reactivations,
            config.memory_lifecycle_config.max_reactivations
        );
    }
}
