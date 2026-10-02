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

use crate::classifier_reward::{
    scanning_instance_affect, AnswerObservation, ChannelAffect, RecordedDecision,
};
use crate::episodic_scan::{
    class_channel_index, collect_active_scan_windows, mask_channels_in_window, should_skip_scan,
    spatial_signature_hash, ScanKernel,
};
use crate::log_rate_limiter::BurstLogRateLimiter;
use crate::memory_neuron_array::{
    MemoryNeuronArray, MemoryNeuronDetail, MemoryNeuronLifecycleConfig,
};
use crate::memory_stats_cache::{self, MemoryStatsCache};
use crate::mp_change_encoder::{ChangeStep, MemoryMpMode, MpChangeEncoding};
use crate::pattern_detector::{BatchPatternDetector, PatternConfig};
use crate::stdp::STDPConfig;
use ahash::AHashSet;
use feagi_structures::neuron_voxels::class_potential::{
    decode_class_potential, encode_class_potential,
};
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

fn command_targets_area(command: &PlasticityCommand, area_idx: u32) -> bool {
    match command {
        PlasticityCommand::RegisterMemoryNeuron {
            area_idx: command_area,
            ..
        }
        | PlasticityCommand::MemoryNeuronConvertedToLtm {
            area_idx: command_area,
            ..
        }
        | PlasticityCommand::InjectMemoryNeuronToFCL {
            area_idx: command_area,
            ..
        }
        | PlasticityCommand::UpdateStateCounters {
            area_idx: command_area,
            ..
        } => *command_area == area_idx,
        PlasticityCommand::ResetMemoryNeuronsInArea { cortical_idx } => *cortical_idx == area_idx,
        PlasticityCommand::UpdateWeightsDelta { .. } => false,
    }
}

/// Replay frame describing a single temporal slice for an upstream area.
#[derive(Debug, Clone)]
pub struct ReplayFrame {
    pub offset: u32,
    pub upstream_area_idx: u32,
    pub coords: Vec<(u32, u32, u32)>,
    /// Per-coordinate membrane potential at encoding time.
    /// Present only when the memory area uses `MemoryMpMode::MpLearning`.
    /// Length matches `coords` when present.
    pub membrane_potentials: Option<Vec<f32>>,
}

/// One field mapped into a kernel memory area with `episodic_scan`.
#[derive(Debug, Clone)]
pub struct MemoryScanSource {
    pub field_area_idx: u32,
    pub twin_area_idx: u32,
    pub field_width: u32,
    pub field_height: u32,
    pub field_depth: u32,
}

/// Single-layer mask sampled in XY during scanner training.
///
/// Each labeled pixel fires once with potential `(class_id + 1) / class_channel_count`.
#[derive(Debug, Clone)]
pub struct ScannerMaskSource {
    pub mask_area_idx: u32,
    pub mask_width: u32,
    pub mask_height: u32,
}

/// Where the correct-answer area is read for one scanning instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerFeedbackLayout {
    /// Every firing voxel is a class channel. Used when feedback matches the class area.
    ClassVolume,
    /// The class is the potential at the window origin. Used when feedback matches the detection twin.
    OutputColumn,
    /// One class for the whole image. Every window of this burst shares these channels.
    ImageClass,
}

/// Reward-training settings for one classifier associative mapping.
///
/// Steps come from that mapping's shared plasticity constant, LTP multiplier,
/// LTD multiplier, and plasticity eta. Each scanning instance applies them
/// only to its own decision.
#[derive(Debug, Clone)]
pub struct ClassifierRewardConfig {
    pub pain_area_idx: u32,
    pub pleasure_area_idx: u32,
    pub feedback_area_idx: Option<u32>,
    pub feedback_layout: AnswerFeedbackLayout,
    pub feedback_width: u32,
    pub feedback_height: u32,
    /// Bursts between the decision and the answer that grades it. Zero grades the current burst.
    pub answer_latency_bursts: u32,
    /// When set, weight changes run only on bursts this area fires.
    pub learn_area_idx: Option<u32>,
    /// OPU that receives one surplus PSP per class channel.
    pub confidence_area_idx: Option<u32>,
    pub confidence_width: u32,
    pub confidence_height: u32,
    pub confidence_depth: u32,
    /// Class-memory firing threshold. Surplus confidence is weight minus this value.
    pub firing_threshold: f32,
    pub pleasure_step: f32,
    pub pain_step: f32,
    pub max_weight: f32,
}

/// Scan assembly attached to a kernel memory area.
///
/// Absent unless a kernel area, class area, associative Mem1→Mem2 mapping,
/// and at least one `episodic_scan` source are all present. Scanner mode
/// replaces the kernel and class areas with `kernel` size and `scanner_mask`.
/// Detection twins are single-layer: each detected pixel fires once with
/// potential `(class_id + 1) / class_channel_count`.
#[derive(Debug, Clone)]
pub struct MemoryScanConfig {
    pub kernel: ScanKernel,
    pub min_window_activity: u32,
    pub scan_skip_density: f32,
    pub class_channel_count: u32,
    pub class_area_width: u32,
    pub class_area_height: u32,
    pub class_memory_area_idx: u32,
    pub sources: Vec<MemoryScanSource>,
    /// Present only in scanner training mode.
    pub scanner_mask: Option<ScannerMaskSource>,
    /// Present only while classifier reward training is on.
    pub reward: Option<ClassifierRewardConfig>,
}

/// Memory area configuration
#[derive(Debug, Clone)]
pub struct MemoryAreaConfig {
    pub temporal_depth: u32,
    pub upstream_areas: Vec<u32>,
    pub mp_mode: MemoryMpMode,
    pub scan: Option<MemoryScanConfig>,
}

/// Runtime counts for a memory cortical area (plasticity layer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCorticalAreaRuntimeInfo {
    pub short_term_neuron_count: usize,
    pub long_term_neuron_count: usize,
    pub upstream_pattern_cache_size: usize,
    /// Mode the plasticity layer actually runs (after depth < 2 auto-disable);
    /// `None` when the area is not registered.
    pub effective_mp_mode: Option<String>,
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
        let mut encoded_this_burst: Vec<(u32, usize, Vec<ReplayFrame>)> = Vec::new();

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

        // Step 2: Detect patterns for all memory areas
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
                            // The newest frame must be the neurons that actually fired on this
                            // burst. A ledger frame can still hold an older pattern after the
                            // upstream area has gone silent, which reactivates memory neurons
                            // that have no live source.
                            let newest_frame = reference_timesteps.len().saturating_sub(1);
                            let mut live_newest: HashMap<u32, Vec<u32>> = HashMap::new();
                            for &upstream_area_idx in &upstream_sorted {
                                if let Some(ids) = npu_lock.fired_neuron_ids_if_queue_timestep(
                                    upstream_area_idx,
                                    current_timestep,
                                ) {
                                    live_newest.insert(upstream_area_idx, ids);
                                }
                            }
                            // Flatten as: for each timestep (oldest->newest), for each upstream area (sorted),
                            // append that area's fired-neuron set at that timestep.
                            let mut out: Vec<HashSet<u32>> = Vec::with_capacity(
                                reference_timesteps.len() * upstream_sorted.len(),
                            );
                            for frame_i in 0..reference_timesteps.len() {
                                for (area_idx, w) in &windows {
                                    let (_t, bitmap) = &w[frame_i];
                                    let live_ids = if frame_i == newest_frame {
                                        live_newest.get(area_idx).map(Vec::as_slice)
                                    } else {
                                        None
                                    };
                                    let neuron_set =
                                        Self::pattern_bitmap_for_frame(bitmap.iter(), live_ids);
                                    out.push(neuron_set);
                                }
                            }

                            // Fetch MP windows for MP learning and change encoding
                            let mp_data = if area_config.mp_mode.requires_mp_archival() {
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

            let change_encoding = area_config.mp_mode.change_encoding();
            let detected = match change_encoding {
                Some(encoding) => Self::encode_change_steps(
                    &encoding,
                    area_config.upstream_areas.len(),
                    mp_windows.as_deref(),
                )
                .and_then(|steps| {
                    detector.detect_change_pattern(
                        *memory_area_idx,
                        &area_config.upstream_areas,
                        &encoding,
                        &steps,
                        Some(area_config.temporal_depth),
                    )
                }),
                None => detector.detect_pattern(
                    *memory_area_idx,
                    &area_config.upstream_areas,
                    current_timestep,
                    timestep_bitmaps,
                    Some(area_config.temporal_depth),
                ),
            };

            if let Some(pattern) = detected {
                // Change-encoded memories have no absolute MPs to reconstruct, so they never replay.
                let replay_frames = if change_encoding.is_some() {
                    Vec::new()
                } else {
                    Self::build_replay_frames(npu, &windows, mp_windows.as_deref())
                };
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
                    array.find_neuron_by_pattern(*memory_area_idx, &pattern.pattern_hash)
                {
                    // Reactivate existing neuron
                    if array.reactivate_memory_neuron(existing_neuron_idx, current_timestep) {
                        let mut s = stats.lock().unwrap();
                        s.memory_neurons_reactivated += 1;
                        drop(s);

                        let neuron_id = array.get_neuron_id(existing_neuron_idx).unwrap();

                        // EMA averaging of membrane potentials on reactivation
                        let final_replay_frames = if area_config.mp_mode == MemoryMpMode::MpLearning
                        {
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

                        if final_replay_frames.is_empty() && change_encoding.is_none() {
                            tracing::warn!(
                                target: "plasticity",
                                "[PLASTICITY] Burst {} reactivation area={} neuron_id={} has empty replay frames",
                                current_timestep,
                                memory_area_idx,
                                neuron_id
                            );
                        }
                        array.set_spatial_signature(
                            existing_neuron_idx,
                            Self::spatial_signature_from_replay(&final_replay_frames),
                        );
                        encoded_this_burst.push((
                            *memory_area_idx,
                            existing_neuron_idx,
                            final_replay_frames.clone(),
                        ));
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

                        if replay_frames.is_empty() && change_encoding.is_none() {
                            tracing::warn!(
                                target: "plasticity",
                                "[PLASTICITY] Burst {} new memory neuron area={} neuron_id={} has empty replay frames",
                                current_timestep,
                                memory_area_idx,
                                neuron_id
                            );
                        }
                        array.set_spatial_signature(
                            neuron_idx,
                            Self::spatial_signature_from_replay(&replay_frames),
                        );
                        encoded_this_burst.push((
                            *memory_area_idx,
                            neuron_idx,
                            replay_frames.clone(),
                        ));
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

        Self::bind_classifier_class_channels(
            &mut array,
            &memory_areas_snapshot,
            &encoded_this_burst,
        );
        Self::run_scanner_training(
            npu,
            &mut array,
            &memory_areas_snapshot,
            memory_lifecycle_configs,
            memory_area_names,
            memory_stats_cache,
            current_timestep,
            &mut commands,
            stats,
        );
        Self::run_episodic_scan(npu, &mut array, &memory_areas_snapshot, current_timestep);

        // Step 3: Age short-term neurons. Runs after every create and reactivate so a
        // neuron matched this burst is not aged out before it can be matched.
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

    /// Neuron set hashed for one upstream frame.
    ///
    /// `live_ids` is the fire queue for this burst. When it is present, it replaces the
    /// ledger bitmap so a stale ledger pattern cannot recall a memory neuron after its
    /// upstream area has stopped firing. `None` keeps the ledger frame because the queue
    /// belongs to a different burst.
    fn pattern_bitmap_for_frame(
        ledger_ids: impl IntoIterator<Item = u32>,
        live_ids: Option<&[u32]>,
    ) -> HashSet<u32> {
        match live_ids {
            Some(live) => live.iter().copied().collect(),
            None => ledger_ids.into_iter().collect(),
        }
    }

    /// Encode dense upstream MP windows into D-1 change steps.
    ///
    /// Steps run oldest to newest; within a step, upstream areas keep the sorted
    /// order `mp_windows` was fetched in. Returns `None` when any upstream MP
    /// window is missing or the windows are misaligned, since a partial window
    /// would hash to a different pattern than the full one.
    fn encode_change_steps(
        encoding: &MpChangeEncoding,
        upstream_count: usize,
        mp_windows: Option<&[PerAreaMpWindow]>,
    ) -> Option<Vec<ChangeStep>> {
        let windows = mp_windows?;
        if windows.len() != upstream_count || windows.is_empty() {
            tracing::trace!(
                target: "plasticity",
                "[PLASTICITY] Change encoding skipped: {} of {} upstream MP windows available",
                windows.len(),
                upstream_count
            );
            return None;
        }
        let frame_count = windows[0].1.len();
        if windows.iter().any(|(_, w)| w.len() != frame_count) {
            tracing::warn!(
                target: "plasticity",
                "[PLASTICITY] Change encoding skipped: misaligned upstream MP windows"
            );
            return None;
        }
        let mut steps = Vec::with_capacity(frame_count.saturating_sub(1) * windows.len());
        for frame_i in 1..frame_count {
            for (_, window) in windows {
                steps.push(encoding.encode_step(&window[frame_i - 1].1, &window[frame_i].1));
            }
        }
        Some(steps)
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

    fn spatial_signature_from_replay(frames: &[ReplayFrame]) -> u64 {
        let mut by_offset: std::collections::BTreeMap<u32, Vec<(u32, u32, u32)>> =
            std::collections::BTreeMap::new();
        for frame in frames {
            by_offset
                .entry(frame.offset)
                .or_default()
                .extend(frame.coords.iter().copied());
        }
        let stacked: Vec<Vec<(u32, u32, u32)>> = by_offset.into_values().collect();
        spatial_signature_hash(&stacked)
    }

    fn class_channels_from_replay(
        frames: &[ReplayFrame],
        class_width: u32,
        class_height: u32,
        class_channel_count: u32,
    ) -> Vec<u32> {
        let mut channels = HashSet::new();
        for frame in frames {
            for &(x, y, z) in &frame.coords {
                let channel = class_channel_index(x, y, z, class_width, class_height);
                if channel < class_channel_count {
                    channels.insert(channel);
                }
            }
        }
        let mut out: Vec<u32> = channels.into_iter().collect();
        out.sort_unstable();
        out
    }

    /// Bind class-memory encode hits onto the kernel-memory neurons encoded in the same burst.
    fn bind_classifier_class_channels(
        array: &mut MemoryNeuronArray,
        memory_areas: &HashMap<u32, MemoryAreaConfig>,
        encoded_this_burst: &[(u32, usize, Vec<ReplayFrame>)],
    ) {
        for (kernel_area_idx, kernel_cfg) in memory_areas {
            let Some(scan) = kernel_cfg.scan.as_ref() else {
                continue;
            };
            let class_hits: Vec<u32> = encoded_this_burst
                .iter()
                .filter(|(area_idx, _, _)| *area_idx == scan.class_memory_area_idx)
                .flat_map(|(_, _, frames)| {
                    Self::class_channels_from_replay(
                        frames,
                        scan.class_area_width,
                        scan.class_area_height,
                        scan.class_channel_count,
                    )
                })
                .collect();
            if class_hits.is_empty() {
                continue;
            }
            if scan.scanner_mask.is_some() {
                continue;
            }
            // Only kernel patterns encoded with this label learn it; binding every stored
            // pattern would give all of them every class and make recall a tie.
            let co_encoded = encoded_this_burst
                .iter()
                .filter(|(area_idx, _, _)| area_idx == kernel_area_idx)
                .map(|(_, neuron_idx, _)| *neuron_idx);
            for neuron_idx in co_encoded {
                for channel in &class_hits {
                    array.bind_class_channel(neuron_idx, *channel);
                }
            }
        }
    }

    /// Learn one long-term pattern per active image window, labeled by mask Z.
    #[allow(clippy::too_many_arguments)]
    fn run_scanner_training(
        npu: &Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
        array: &mut MemoryNeuronArray,
        memory_areas: &HashMap<u32, MemoryAreaConfig>,
        memory_lifecycle_configs: &Arc<Mutex<HashMap<u32, MemoryNeuronLifecycleConfig>>>,
        memory_area_names: &Arc<Mutex<HashMap<u32, String>>>,
        memory_stats_cache: &crate::memory_stats_cache::MemoryStatsCache,
        current_timestep: u64,
        commands: &mut Vec<PlasticityCommand>,
        stats: &Arc<Mutex<PlasticityStats>>,
    ) {
        let lifecycle_configs = memory_lifecycle_configs.lock().unwrap();
        for (kernel_area_idx, kernel_cfg) in memory_areas {
            let Some(scan) = kernel_cfg.scan.as_ref() else {
                continue;
            };
            let Some(mask) = scan.scanner_mask.as_ref() else {
                continue;
            };
            if scan.kernel.voxel_count() == 0
                || scan.sources.is_empty()
                || scan.class_channel_count == 0
            {
                continue;
            }
            let temporal_depth = kernel_cfg.temporal_depth.max(1) as usize;
            let lifecycle_config = lifecycle_configs
                .get(kernel_area_idx)
                .copied()
                .unwrap_or_default();
            let Some(mask_newest) = Self::newest_mask_classes(
                npu,
                mask.mask_area_idx,
                scan.class_channel_count,
                current_timestep,
                temporal_depth,
            ) else {
                continue;
            };
            if mask_newest.is_empty() {
                continue;
            }
            for source in &scan.sources {
                let frames = {
                    let npu_lock = npu.lock().unwrap();
                    let window = match npu_lock.get_fire_ledger_dense_window_bitmaps(
                        source.field_area_idx,
                        current_timestep,
                        temporal_depth,
                    ) {
                        Ok(w) => w,
                        Err(_) => continue,
                    };
                    let mut coord_frames: Vec<Vec<(u32, u32, u32)>> = Vec::new();
                    for (_t, bitmap) in &window {
                        let mut coords: Vec<(u32, u32, u32)> = bitmap
                            .iter()
                            .filter_map(|neuron_id| npu_lock.get_neuron_coordinates(neuron_id))
                            .collect();
                        coords.sort_unstable();
                        coord_frames.push(coords);
                    }
                    coord_frames
                };
                let windows = collect_active_scan_windows(
                    &frames,
                    scan.kernel,
                    source.field_width,
                    source.field_height,
                    source.field_depth,
                    scan.min_window_activity,
                );
                for window in windows {
                    let channels = mask_channels_in_window(
                        &mask_newest,
                        window.origin.0,
                        window.origin.1,
                        scan.kernel.width,
                        scan.kernel.height,
                        scan.class_channel_count,
                    );
                    if channels.is_empty() {
                        continue;
                    }
                    let Some(neuron_idx) = array.create_memory_neuron(
                        window.spatial_hash,
                        *kernel_area_idx,
                        current_timestep,
                        &lifecycle_config,
                    ) else {
                        continue;
                    };
                    let created = array
                        .get_activation_count(neuron_idx)
                        .is_some_and(|count| count == 1);
                    array.set_spatial_signature(neuron_idx, window.spatial_hash);
                    for channel in channels {
                        if channel < scan.class_channel_count {
                            array.bind_class_channel(neuron_idx, channel);
                        }
                    }
                    let Some(neuron_id) = array.get_neuron_id(neuron_idx) else {
                        continue;
                    };
                    if created {
                        if let Some(area_name) =
                            memory_area_names.lock().unwrap().get(kernel_area_idx)
                        {
                            crate::memory_stats_cache::on_neuron_created(
                                memory_stats_cache,
                                area_name,
                            );
                        }
                        let mut recorded = stats.lock().unwrap();
                        recorded.memory_neurons_created += 1;
                        drop(recorded);
                    }
                    commands.push(PlasticityCommand::RegisterMemoryNeuron {
                        neuron_id,
                        area_idx: *kernel_area_idx,
                        threshold: 1.0,
                        membrane_potential: 0.0,
                    });
                }
            }
        }
    }

    /// Class channels in the correct-answer area for one scanning instance.
    ///
    /// A connected area with no firing is [`AnswerObservation::Silent`]: the
    /// gap before a label arrives is not pain.
    #[allow(clippy::too_many_arguments)]
    fn feedback_channels_for_window(
        npu: &Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
        reward: &ClassifierRewardConfig,
        origin: (u32, u32, u32),
        class_channel_count: u32,
        class_area_width: u32,
        class_area_height: u32,
        current_timestep: u64,
        temporal_depth: usize,
    ) -> AnswerObservation {
        let Some(feedback_area_idx) = reward.feedback_area_idx else {
            return AnswerObservation::Absent;
        };
        if reward.feedback_layout == AnswerFeedbackLayout::OutputColumn {
            let classes = Self::newest_mask_classes(
                npu,
                feedback_area_idx,
                class_channel_count,
                current_timestep,
                temporal_depth,
            )
            .unwrap_or_default();
            let mut channels: Vec<u32> = classes
                .into_iter()
                .filter(|(x, y, _)| *x == origin.0 && *y == origin.1)
                .map(|(_, _, class_id)| class_id)
                .collect();
            channels.sort_unstable();
            channels.dedup();
            return if channels.is_empty() {
                AnswerObservation::Silent
            } else {
                AnswerObservation::Present(channels)
            };
        }
        let Ok(npu_lock) = npu.lock() else {
            return AnswerObservation::Silent;
        };
        let Ok(window) = npu_lock.get_fire_ledger_dense_window_bitmaps(
            feedback_area_idx,
            current_timestep,
            temporal_depth,
        ) else {
            return AnswerObservation::Silent;
        };
        let Some((_, newest)) = window.last() else {
            return AnswerObservation::Silent;
        };
        let mut channels = Vec::new();
        for neuron_id in newest.iter() {
            let Some((x, y, z)) = npu_lock.get_neuron_coordinates(neuron_id) else {
                continue;
            };
            let channel = match reward.feedback_layout {
                AnswerFeedbackLayout::OutputColumn => continue,
                AnswerFeedbackLayout::ClassVolume => {
                    class_channel_index(x, y, z, class_area_width, class_area_height)
                }
                AnswerFeedbackLayout::ImageClass => {
                    class_channel_index(x, y, z, reward.feedback_width, reward.feedback_height)
                }
            };
            if channel < class_channel_count {
                channels.push(channel);
            }
        }
        channels.sort_unstable();
        channels.dedup();
        if channels.is_empty() {
            AnswerObservation::Silent
        } else {
            AnswerObservation::Present(channels)
        }
    }

    /// `(x, y, class_id)` for every voxel of a single-layer class map that fired on the newest burst.
    ///
    /// The class is decoded from the firing-time potential. Voxels whose potential names
    /// no class are dropped. `None` when the area's fire ledger or MP window is unavailable.
    fn newest_mask_classes(
        npu: &Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
        area_idx: u32,
        class_count: u32,
        current_timestep: u64,
        temporal_depth: usize,
    ) -> Option<Vec<(u32, u32, u32)>> {
        let npu_lock = npu.lock().ok()?;
        let window = npu_lock
            .get_fire_ledger_dense_window_mp(area_idx, current_timestep, temporal_depth)
            .ok()?;
        let (_, newest) = window.last()?;
        let mut classes: Vec<(u32, u32, u32)> = newest
            .iter()
            .filter_map(|(neuron_id, potential)| {
                let (x, y, _) = npu_lock.get_neuron_coordinates(*neuron_id)?;
                let class_id = decode_class_potential(*potential, class_count)?;
                Some((x, y, class_id))
            })
            .collect();
        classes.sort_unstable();
        Some(classes)
    }

    /// Spike written on the winning depth of a kernel-mode class output.
    const KERNEL_CLASS_OUTPUT_POTENTIAL: f32 = 1.0;

    /// Kernel mode reports one class for the whole field at depth `z`.
    fn kernel_class_output_coord(
        votes: &HashMap<(u32, u32), HashMap<u32, f32>>,
        class_count: u32,
    ) -> Option<(u32, u32, u32)> {
        let mut totals: HashMap<u32, f32> = HashMap::new();
        for pixel in votes.values() {
            for (class_id, score) in pixel {
                if *class_id >= class_count {
                    continue;
                }
                *totals.entry(*class_id).or_insert(0.0) += *score;
            }
        }
        let winner =
            totals.iter().fold(
                None,
                |best: Option<(u32, f32)>, (class_id, score)| match best {
                    Some((best_id, best_score))
                        if best_score > *score || (best_score == *score && best_id < *class_id) =>
                    {
                        Some((best_id, best_score))
                    }
                    _ => Some((*class_id, *score)),
                },
            )?;
        Some((0, 0, winner.0))
    }

    /// One class per detected pixel, and the potential that encodes it.
    ///
    /// Votes are summed per class; the highest total wins and ties go to the lower class id,
    /// so overlapping windows write each pixel exactly once.
    fn detection_twin_writes(
        votes: &HashMap<(u32, u32), HashMap<u32, f32>>,
        class_count: u32,
    ) -> (Vec<(u32, u32, u32)>, Vec<f32>) {
        let mut pixels: Vec<&(u32, u32)> = votes.keys().collect();
        pixels.sort_unstable();
        let mut coords = Vec::with_capacity(pixels.len());
        let mut potentials = Vec::with_capacity(pixels.len());
        for pixel in pixels {
            let winner =
                votes[pixel]
                    .iter()
                    .fold(
                        None,
                        |best: Option<(u32, f32)>, (class_id, score)| match best {
                            Some((best_id, best_score))
                                if best_score > *score
                                    || (best_score == *score && best_id < *class_id) =>
                            {
                                Some((best_id, best_score))
                            }
                            _ => Some((*class_id, *score)),
                        },
                    );
            let Some((class_id, _)) = winner else {
                continue;
            };
            let Ok(potential) = encode_class_potential(class_id, class_count) else {
                continue;
            };
            coords.push((pixel.0, pixel.1, 0));
            potentials.push(potential);
        }
        (coords, potentials)
    }

    fn confidence_coord(
        channel: u32,
        width: u32,
        height: u32,
        depth: u32,
    ) -> Option<(u32, u32, u32)> {
        if width > 1 && height == 1 && depth == 1 && channel < width {
            return Some((channel, 0, 0));
        }
        if height > 1 && width == 1 && depth == 1 && channel < height {
            return Some((0, channel, 0));
        }
        if depth > 1 && width == 1 && height == 1 && channel < depth {
            return Some((0, 0, channel));
        }
        let plane = width.saturating_mul(height);
        if plane == 0 || depth == 0 {
            return None;
        }
        let volume = plane.saturating_mul(depth);
        if channel >= volume {
            return None;
        }
        let z = channel / plane;
        let rest = channel % plane;
        Some((rest % width, rest / width, z))
    }

    fn area_fired_this_burst(
        npu: &Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
        area_idx: u32,
        current_timestep: u64,
        temporal_depth: usize,
    ) -> bool {
        let Ok(npu_lock) = npu.lock() else {
            return false;
        };
        let Ok(window) = npu_lock.get_fire_ledger_dense_window_bitmaps(
            area_idx,
            current_timestep,
            temporal_depth,
        ) else {
            return false;
        };
        window.last().is_some_and(|(_, newest)| !newest.is_empty())
    }

    fn run_episodic_scan(
        npu: &Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
        array: &mut MemoryNeuronArray,
        memory_areas: &HashMap<u32, MemoryAreaConfig>,
        current_timestep: u64,
    ) {
        for (kernel_area_idx, kernel_cfg) in memory_areas {
            let Some(scan) = kernel_cfg.scan.as_ref() else {
                continue;
            };
            if scan.kernel.voxel_count() == 0 || scan.sources.is_empty() {
                continue;
            }
            let temporal_depth = kernel_cfg.temporal_depth.max(1) as usize;
            for source in &scan.sources {
                let field_volume = u64::from(source.field_width)
                    .saturating_mul(u64::from(source.field_height))
                    .saturating_mul(u64::from(source.field_depth));
                let (frames, fired_count) = {
                    let npu_lock = npu.lock().unwrap();
                    let window = match npu_lock.get_fire_ledger_dense_window_bitmaps(
                        source.field_area_idx,
                        current_timestep,
                        temporal_depth,
                    ) {
                        Ok(w) => w,
                        Err(_) => continue,
                    };
                    let mut coord_frames: Vec<Vec<(u32, u32, u32)>> = Vec::new();
                    let mut newest_count = 0usize;
                    for (i, (_t, bitmap)) in window.iter().enumerate() {
                        let mut coords: Vec<(u32, u32, u32)> = bitmap
                            .iter()
                            .filter_map(|neuron_id| npu_lock.get_neuron_coordinates(neuron_id))
                            .collect();
                        coords.sort_unstable();
                        if i + 1 == window.len() {
                            newest_count = coords.len();
                        }
                        coord_frames.push(coords);
                    }
                    (coord_frames, newest_count)
                };
                if should_skip_scan(fired_count, field_volume, scan.scan_skip_density) {
                    continue;
                }
                let windows = collect_active_scan_windows(
                    &frames,
                    scan.kernel,
                    source.field_width,
                    source.field_height,
                    source.field_depth,
                    scan.min_window_activity,
                );
                if windows.is_empty() {
                    if scan.reward.is_some() {
                        array
                            .presentation_ledger_mut()
                            .expire_area(*kernel_area_idx);
                    }
                    continue;
                }
                let learn_open = scan.reward.as_ref().is_none_or(|reward| {
                    reward.learn_area_idx.is_none_or(|learn_area_idx| {
                        Self::area_fired_this_burst(
                            npu,
                            learn_area_idx,
                            current_timestep,
                            temporal_depth,
                        )
                    })
                });
                let mut votes: HashMap<(u32, u32), HashMap<u32, f32>> = HashMap::new();
                let mut confidence: HashMap<u32, f32> = HashMap::new();
                let mut pain_this_source = false;
                let mut pleasure_this_source = false;
                for window in windows {
                    let observation = scan.reward.as_ref().map(|reward| {
                        Self::feedback_channels_for_window(
                            npu,
                            reward,
                            window.origin,
                            scan.class_channel_count,
                            scan.class_area_width,
                            scan.class_area_height,
                            current_timestep,
                            temporal_depth,
                        )
                    });
                    let matches =
                        array.find_ltm_by_spatial_signature(*kernel_area_idx, window.spatial_hash);
                    for neuron_idx in matches {
                        if let Some(reward) = scan.reward.as_ref() {
                            let current_channels = array.get_class_channels(neuron_idx);
                            let spatial_hash = window.spatial_hash;
                            array.presentation_ledger_mut().record(RecordedDecision {
                                burst: current_timestep,
                                area_idx: *kernel_area_idx,
                                neuron_idx,
                                origin: window.origin,
                                spatial_hash,
                                channels: current_channels.clone(),
                            });
                            array
                                .presentation_ledger_mut()
                                .prune(current_timestep, reward.answer_latency_bursts);
                            let graded = if reward.answer_latency_bursts == 0 {
                                Some(current_channels)
                            } else {
                                array.presentation_ledger_mut().channels_at(
                                    current_timestep,
                                    reward.answer_latency_bursts,
                                    *kernel_area_idx,
                                    neuron_idx,
                                    window.origin,
                                )
                            };
                            let skip = !learn_open
                                || matches!(observation, Some(AnswerObservation::Silent))
                                || graded.is_none()
                                || array
                                    .presentation_ledger_mut()
                                    .already_corrected(*kernel_area_idx, spatial_hash);
                            if !skip {
                                if let Some(decision) = graded {
                                    let feedback = match observation
                                        .clone()
                                        .unwrap_or(AnswerObservation::Absent)
                                    {
                                        AnswerObservation::Absent => None,
                                        AnswerObservation::Silent => None,
                                        AnswerObservation::Present(channels) => Some(channels),
                                    };
                                    if !matches!(observation, Some(AnswerObservation::Silent)) {
                                        for channel in &decision {
                                            array.ensure_class_channel_weight(
                                                neuron_idx,
                                                *channel,
                                                reward.pleasure_step,
                                            );
                                        }
                                        let updates = scanning_instance_affect(
                                            &decision,
                                            feedback.as_deref(),
                                        );
                                        if !updates.is_empty() {
                                            for update in updates {
                                                let delta = match update.affect {
                                                    ChannelAffect::Pleasure => reward.pleasure_step,
                                                    ChannelAffect::Pain => -reward.pain_step,
                                                };
                                                let initial = if update.bind {
                                                    0.0
                                                } else {
                                                    reward.pleasure_step
                                                };
                                                array.apply_class_channel_delta(
                                                    neuron_idx,
                                                    update.channel,
                                                    delta,
                                                    reward.max_weight,
                                                    initial,
                                                );
                                                match update.affect {
                                                    ChannelAffect::Pleasure => {
                                                        pleasure_this_source = true
                                                    }
                                                    ChannelAffect::Pain => pain_this_source = true,
                                                }
                                            }
                                            array
                                                .presentation_ledger_mut()
                                                .mark_corrected(*kernel_area_idx, spatial_hash);
                                        }
                                    }
                                }
                            }
                        }
                        for class_z in array.get_class_channels(neuron_idx) {
                            if class_z >= scan.class_channel_count {
                                continue;
                            }
                            let vote = array
                                .class_channel_weight(neuron_idx, class_z)
                                .unwrap_or(1.0);
                            *votes
                                .entry((window.origin.0, window.origin.1))
                                .or_default()
                                .entry(class_z)
                                .or_insert(0.0) += vote;
                            if let Some(reward) = scan.reward.as_ref() {
                                if let Some(weight) =
                                    array.class_channel_weight(neuron_idx, class_z)
                                {
                                    let surplus = weight - reward.firing_threshold;
                                    if surplus > 0.0 {
                                        let entry = confidence.entry(class_z).or_insert(0.0);
                                        if surplus > *entry {
                                            *entry = surplus;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(reward) = scan.reward.as_ref() {
                    if let Some(confidence_area_idx) = reward.confidence_area_idx {
                        if !confidence.is_empty() {
                            let mut coords = Vec::new();
                            let mut potentials = Vec::new();
                            let mut channels: Vec<u32> = confidence.keys().copied().collect();
                            channels.sort_unstable();
                            for channel in channels {
                                let Some(coord) = Self::confidence_coord(
                                    channel,
                                    reward.confidence_width,
                                    reward.confidence_height,
                                    reward.confidence_depth,
                                ) else {
                                    continue;
                                };
                                coords.push(coord);
                                potentials.push(confidence[&channel]);
                            }
                            if !coords.is_empty() {
                                if let Ok(npu_lock) = npu.lock() {
                                    npu_lock.schedule_replay_injection(
                                        current_timestep.saturating_add(1),
                                        confidence_area_idx,
                                        coords,
                                        Some(potentials),
                                    );
                                }
                            }
                        }
                    }
                }
                if let Some(reward) = scan.reward.as_ref() {
                    if let Ok(npu_lock) = npu.lock() {
                        if pain_this_source {
                            npu_lock.schedule_replay_injection(
                                current_timestep.saturating_add(1),
                                reward.pain_area_idx,
                                vec![(0, 0, 0)],
                                None,
                            );
                        }
                        if pleasure_this_source {
                            npu_lock.schedule_replay_injection(
                                current_timestep.saturating_add(1),
                                reward.pleasure_area_idx,
                                vec![(0, 0, 0)],
                                None,
                            );
                        }
                    }
                }
                if votes.is_empty() {
                    continue;
                }
                let (coords, potentials) = if scan.scanner_mask.is_none() {
                    match Self::kernel_class_output_coord(&votes, scan.class_channel_count) {
                        Some(coord) => (vec![coord], vec![Self::KERNEL_CLASS_OUTPUT_POTENTIAL]),
                        None => continue,
                    }
                } else {
                    Self::detection_twin_writes(&votes, scan.class_channel_count)
                };
                if coords.is_empty() {
                    continue;
                }
                if let Ok(npu_lock) = npu.lock() {
                    if let Err(error) = npu_lock.schedule_valued_force_fire(
                        current_timestep.saturating_add(1),
                        source.twin_area_idx,
                        coords,
                        potentials,
                    ) {
                        tracing::warn!(
                            target: "plasticity",
                            "[PLASTICITY] Detection twin {} write rejected: {}",
                            source.twin_area_idx,
                            error
                        );
                    }
                }
            }
        }
    }

    /// Register a memory area for pattern detection
    pub fn register_memory_area(
        &self,
        area_idx: u32,
        area_name: String,
        temporal_depth: u32,
        upstream_areas: Vec<u32>,
        lifecycle_config: Option<MemoryNeuronLifecycleConfig>,
        mp_mode: MemoryMpMode,
    ) -> bool {
        let Some(mp_mode) = Self::resolve_mp_mode(area_idx, temporal_depth, mp_mode) else {
            return false;
        };
        let upstream_len = upstream_areas.len();
        let upstream_clone = upstream_areas.clone();
        let mut areas = self.memory_areas.lock().unwrap();
        let existing_scan = areas.get(&area_idx).and_then(|cfg| cfg.scan.clone());
        areas.insert(
            area_idx,
            MemoryAreaConfig {
                temporal_depth,
                upstream_areas,
                mp_mode,
                scan: existing_scan,
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

            // Enable MP archival on upstream areas for MP learning and change encoding
            if mp_mode.requires_mp_archival() {
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
                    "[PLASTICITY] MP mode {:?} for memory area {} - upstream MP archival active",
                    mp_mode,
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

    /// Resolve the MP mode a memory area actually runs with.
    ///
    /// Change encoding needs two frames, so windows shallower than 2 run as
    /// `PatternOnly`. Invalid quantization returns `None` and the area is not
    /// registered.
    fn resolve_mp_mode(
        area_idx: u32,
        temporal_depth: u32,
        mp_mode: MemoryMpMode,
    ) -> Option<MemoryMpMode> {
        let Some(encoding) = mp_mode.change_encoding() else {
            return Some(mp_mode);
        };
        if let Err(e) = encoding.validate() {
            tracing::error!(
                target: "plasticity",
                "[PLASTICITY] Memory area {} not registered: {}",
                area_idx,
                e
            );
            return None;
        }
        if temporal_depth < 2 {
            tracing::warn!(
                target: "plasticity",
                "[PLASTICITY] Memory area {} change encoding disabled: temporal_depth={} (needs >= 2)",
                area_idx,
                temporal_depth
            );
            return Some(MemoryMpMode::PatternOnly);
        }
        Some(mp_mode)
    }

    /// MP mode a registered memory area runs with, after depth and validation checks.
    pub fn memory_area_mp_mode(&self, area_idx: u32) -> Option<MemoryMpMode> {
        self.memory_areas
            .lock()
            .unwrap()
            .get(&area_idx)
            .map(|cfg| cfg.mp_mode)
    }

    /// Attach or replace scan configuration for a registered kernel memory area.
    ///
    /// Re-registering the area via [`Self::register_memory_area`] keeps this
    /// config. Passing `None` clears scan for the area.
    pub fn configure_memory_scan(&self, area_idx: u32, scan: Option<MemoryScanConfig>) -> bool {
        let mut areas = self.memory_areas.lock().unwrap();
        let Some(cfg) = areas.get_mut(&area_idx) else {
            return false;
        };
        if let Some(ref scan_cfg) = scan {
            if let Ok(mut npu) = self.npu.lock() {
                let desired = cfg.temporal_depth as usize;
                let existing_configs = npu.get_all_fire_ledger_configs();
                for source in &scan_cfg.sources {
                    let existing = existing_configs
                        .iter()
                        .find(|(idx, _)| *idx == source.field_area_idx)
                        .map(|(_, w)| *w)
                        .unwrap_or(0);
                    let resolved = existing.max(desired);
                    if resolved != existing {
                        if let Err(e) =
                            npu.configure_fire_ledger_window(source.field_area_idx, resolved)
                        {
                            tracing::warn!(
                                target: "plasticity",
                                "[PLASTICITY] Failed to configure FireLedger window for scan field {} (requested={}): {}",
                                source.field_area_idx,
                                resolved,
                                e
                            );
                        }
                    }
                }
                if let Some(reward) = &scan_cfg.reward {
                    for area_idx in [reward.feedback_area_idx, reward.learn_area_idx]
                        .into_iter()
                        .flatten()
                    {
                        let existing = existing_configs
                            .iter()
                            .find(|(idx, _)| *idx == area_idx)
                            .map(|(_, w)| *w)
                            .unwrap_or(0);
                        let resolved = existing.max(desired);
                        if resolved != existing {
                            if let Err(e) = npu.configure_fire_ledger_window(area_idx, resolved) {
                                tracing::warn!(
                                    target: "plasticity",
                                    "[PLASTICITY] Failed to configure FireLedger window for classifier reward area {} (requested={}): {}",
                                    area_idx,
                                    resolved,
                                    e
                                );
                            }
                        }
                    }
                    if reward.feedback_layout == AnswerFeedbackLayout::OutputColumn {
                        if let Some(feedback_area_idx) = reward.feedback_area_idx {
                            if let Err(e) = npu.enable_fire_ledger_mp_archival(feedback_area_idx) {
                                tracing::warn!(
                                    target: "plasticity",
                                    "[PLASTICITY] Failed to enable MP archival for classifier answer area {}: {}",
                                    feedback_area_idx,
                                    e
                                );
                            }
                        }
                    }
                }
                if let Some(mask) = &scan_cfg.scanner_mask {
                    let existing = existing_configs
                        .iter()
                        .find(|(idx, _)| *idx == mask.mask_area_idx)
                        .map(|(_, w)| *w)
                        .unwrap_or(0);
                    let resolved = existing.max(desired);
                    if resolved != existing {
                        if let Err(e) =
                            npu.configure_fire_ledger_window(mask.mask_area_idx, resolved)
                        {
                            tracing::warn!(
                                target: "plasticity",
                                "[PLASTICITY] Failed to configure FireLedger window for scan mask {} (requested={}): {}",
                                mask.mask_area_idx,
                                resolved,
                                e
                            );
                        }
                    }
                    if let Err(e) = npu.enable_fire_ledger_mp_archival(mask.mask_area_idx) {
                        tracing::warn!(
                            target: "plasticity",
                            "[PLASTICITY] Failed to enable MP archival for scan mask {}: {}",
                            mask.mask_area_idx,
                            e
                        );
                    }
                }
            }
        }
        cfg.scan = scan;
        true
    }

    /// Clear cortical-index keyed registrations before a connectome rebuild.
    ///
    /// Memory neurons are restored separately from the snapshot; this only
    /// removes runtime registrations and queued commands that refer to the
    /// previous connectome's numeric cortical indexes.
    pub fn clear_memory_area_registrations(&self) {
        self.memory_areas.lock().unwrap().clear();
        self.memory_lifecycle_configs.lock().unwrap().clear();
        self.memory_area_names.lock().unwrap().clear();
        self.pattern_detector.detectors.lock().unwrap().clear();
        self.command_queue.lock().unwrap().clear();
    }

    /// Discard every memory neuron along with all cortical-index keyed registrations.
    ///
    /// Genome load renumbers every non-core cortical index, so memory neurons carried
    /// over from the previous brain keep indexes that now resolve to unrelated cortical
    /// areas. Those rows are unusable at runtime and poison exported connectomes, so the
    /// entire memory state is dropped before the new genome is built.
    ///
    /// Returns the number of memory neurons discarded.
    pub fn reset_all_memory_state(&self) -> usize {
        let area_names = self.memory_area_names.lock().unwrap().clone();
        let mut attributed_per_area: Vec<(String, usize)> = Vec::with_capacity(area_names.len());
        let discarded = {
            let mut array = self.memory_neuron_array.lock().unwrap();
            for (area_idx, area_name) in area_names.iter() {
                let active = array.get_active_neurons_by_area(*area_idx).len();
                if active > 0 {
                    attributed_per_area.push((area_name.clone(), active));
                }
            }
            let discarded = array.get_stats().active_neurons;
            array.reset();
            discarded
        };

        let mut attributed = 0usize;
        for (area_name, active) in attributed_per_area {
            attributed += active;
            for _ in 0..active {
                memory_stats_cache::on_neuron_deleted(&self.memory_stats_cache, &area_name);
            }
        }
        if discarded > attributed {
            tracing::warn!(
                target: "plasticity",
                "[PLASTICITY] Discarded {} memory neuron(s) whose cortical index is no longer registered; per-area stats could not be attributed",
                discarded - attributed
            );
        }

        self.clear_memory_area_registrations();
        tracing::info!(
            target: "plasticity",
            "[PLASTICITY] Memory state reset for new genome: {} memory neuron(s) discarded",
            discarded
        );
        discarded
    }

    /// Stop pattern detection for one cortical index and delete its memory neurons.
    ///
    /// Deleting a cortical area removes it from the connectome index map. A
    /// registration left behind keeps detecting patterns and queues injections
    /// the burst loop cannot apply, because that index no longer has an id.
    pub fn unregister_memory_area(&self, area_idx: u32) {
        let was_registered = self.memory_areas.lock().unwrap().contains_key(&area_idx);
        if !was_registered {
            return;
        }
        self.reset_memory_neurons_in_area(area_idx);
        self.memory_areas.lock().unwrap().remove(&area_idx);
        self.memory_lifecycle_configs
            .lock()
            .unwrap()
            .remove(&area_idx);
        self.memory_area_names.lock().unwrap().remove(&area_idx);
        self.pattern_detector
            .detectors
            .lock()
            .unwrap()
            .remove(&area_idx);
        self.command_queue
            .lock()
            .unwrap()
            .retain(|command| !command_targets_area(command, area_idx));
        tracing::info!(
            target: "plasticity",
            "[PLASTICITY] Unregistered memory area idx={}",
            area_idx
        );
    }

    /// Return sorted registered memory-area indexes for diagnostics and verification.
    pub fn registered_memory_area_indexes(&self) -> Vec<u32> {
        let mut indexes: Vec<u32> = self.memory_areas.lock().unwrap().keys().copied().collect();
        indexes.sort_unstable();
        indexes
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

        // Delete associative and ordinary synapses on these memory neurons and on any
        // storage neurons that live in the same cortical area (LTM bridge twins).
        {
            let mut npu_lock = self.npu.lock().unwrap();
            let mut synapse_endpoints = memory_neuron_ids.clone();
            synapse_endpoints.extend(npu_lock.get_neurons_in_cortical_area(cortical_idx));
            if !synapse_endpoints.is_empty() {
                let scrub_ids: AHashSet<u32> = synapse_endpoints.iter().copied().collect();
                npu_lock.scrub_synaptic_arrival_schedule_for_neuron_targets(&scrub_ids);
                npu_lock.remove_synapses_touching_neuron_ids(&synapse_endpoints);
            }
            if !memory_neuron_ids.is_empty() {
                npu_lock.clear_memory_replay_frames(&memory_neuron_ids);
                npu_lock.clear_sparse_memory_associative_state(&memory_neuron_ids);
            }
        }
        self.pattern_detector.forget_area(cortical_idx);

        // Delete the memory neurons themselves so the next pattern allocates a new one.
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
        let effective_mp_mode = self
            .memory_area_mp_mode(cortical_idx)
            .map(|mode| mode.as_str().to_string());
        let array = self.memory_neuron_array.lock().unwrap();
        let upstream_pattern_cache_size = self
            .pattern_detector
            .cached_pattern_count_for_area(cortical_idx);
        MemoryCorticalAreaRuntimeInfo {
            short_term_neuron_count: array.count_short_term_in_area(cortical_idx),
            long_term_neuron_count: array.count_long_term_in_area(cortical_idx),
            upstream_pattern_cache_size,
            effective_mp_mode,
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
    fn silent_upstream_does_not_keep_a_stale_ledger_pattern() {
        let stale_ledger = [11_u32, 12, 13];
        let live_fires: &[u32] = &[];
        let bitmap = PlasticityService::pattern_bitmap_for_frame(stale_ledger, Some(live_fires));
        assert!(
            bitmap.is_empty(),
            "a quiet upstream burst must not hash the leftover ledger pattern"
        );

        let still_firing = PlasticityService::pattern_bitmap_for_frame([87], Some(&[87]));
        assert_eq!(still_firing, HashSet::from([87]));

        let late_reader = PlasticityService::pattern_bitmap_for_frame(stale_ledger, None);
        assert_eq!(late_reader, HashSet::from([11, 12, 13]));
    }

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

        let result = service.register_memory_area(
            100,
            "mem_00".to_string(),
            3,
            vec![1, 2],
            None,
            MemoryMpMode::PatternOnly,
        );
        assert!(result);

        let areas = service.memory_areas.lock().unwrap();
        assert!(areas.contains_key(&100));
    }

    #[test]
    fn clear_memory_area_registrations_removes_stale_indexes_only() {
        let config = PlasticityConfig::default();
        let cache = create_memory_stats_cache();
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 16, 16, 8).unwrap(),
            "plasticity-registration-reset-test-npu",
        ));
        let service = PlasticityService::new(config, cache, npu);
        service.register_memory_area(
            16,
            "stale-memory".to_string(),
            1,
            vec![9],
            None,
            MemoryMpMode::PatternOnly,
        );
        service.pattern_detector.get_detector(16, 1);
        service.enqueue_commands_for_test(vec![PlasticityCommand::ResetMemoryNeuronsInArea {
            cortical_idx: 16,
        }]);

        service.clear_memory_area_registrations();

        assert!(service.memory_areas.lock().unwrap().is_empty());
        assert!(service.memory_lifecycle_configs.lock().unwrap().is_empty());
        assert!(service.memory_area_names.lock().unwrap().is_empty());
        assert!(service
            .pattern_detector
            .detectors
            .lock()
            .unwrap()
            .is_empty());
        assert!(service.command_queue.lock().unwrap().is_empty());
    }

    #[test]
    fn unregister_memory_area_stops_that_index_only() {
        let config = PlasticityConfig::default();
        let cache = create_memory_stats_cache();
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 16, 16, 8).unwrap(),
            "plasticity-unregister-area-test-npu",
        ));
        let service = PlasticityService::new(config, cache, npu);
        service.register_memory_area(
            23,
            "deleted-memory".to_string(),
            1,
            vec![16],
            None,
            MemoryMpMode::PatternOnly,
        );
        service.register_memory_area(
            35,
            "live-memory".to_string(),
            1,
            vec![16],
            None,
            MemoryMpMode::PatternOnly,
        );
        service.pattern_detector.get_detector(23, 1);
        {
            let mut array = service.memory_neuron_array.lock().unwrap();
            let lifecycle = MemoryNeuronLifecycleConfig::default();
            array
                .create_memory_neuron(41, 23, 0, &lifecycle)
                .expect("memory neuron in the deleted area");
            array
                .create_memory_neuron(42, 35, 0, &lifecycle)
                .expect("memory neuron in the live area");
        }
        service.enqueue_commands_for_test(vec![PlasticityCommand::InjectMemoryNeuronToFCL {
            neuron_id: 41,
            area_idx: 23,
            membrane_potential: 1.5,
            pattern_hash: 7,
            is_reactivation: false,
            replay_frames: Vec::new(),
        }]);

        service.unregister_memory_area(23);

        let indexes = service.registered_memory_area_indexes();
        assert_eq!(indexes, vec![35]);
        assert!(service
            .pattern_detector
            .detectors
            .lock()
            .unwrap()
            .get(&23)
            .is_none());
        assert!(service.command_queue.lock().unwrap().is_empty());
        let array = service.memory_neuron_array.lock().unwrap();
        assert!(array.get_active_neurons_by_area(23).is_empty());
        assert_eq!(array.get_active_neurons_by_area(35).len(), 1);
    }

    #[test]
    fn reset_all_memory_state_discards_neurons_and_registrations() {
        let config = PlasticityConfig::default();
        let cache = create_memory_stats_cache();
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 16, 16, 8).unwrap(),
            "plasticity-memory-state-reset-test-npu",
        ));
        let service = PlasticityService::new(config, cache, npu);
        service.register_memory_area(
            8,
            "mem_reset".to_string(),
            1,
            vec![7],
            None,
            MemoryMpMode::PatternOnly,
        );
        {
            let mut array = service.memory_neuron_array.lock().unwrap();
            let lifecycle = MemoryNeuronLifecycleConfig::default();
            array
                .create_memory_neuron(11, 8, 0, &lifecycle)
                .expect("registered-area memory neuron");
            // A neuron whose cortical index belongs to the previous genome's index map.
            array
                .create_memory_neuron(22, 99, 0, &lifecycle)
                .expect("stale-area memory neuron");
            assert_eq!(array.get_stats().active_neurons, 2);
        }

        let discarded = service.reset_all_memory_state();

        assert_eq!(discarded, 2);
        assert_eq!(
            service
                .memory_neuron_array
                .lock()
                .unwrap()
                .get_stats()
                .active_neurons,
            0
        );
        assert!(service.export_long_term_memory_neurons().is_empty());
        assert!(service.memory_areas.lock().unwrap().is_empty());
        assert!(service.memory_area_names.lock().unwrap().is_empty());
        assert_eq!(
            memory_stats_cache::get_area_stats(&service.memory_stats_cache, "mem_reset")
                .expect("registered area keeps a stats entry")
                .neuron_count,
            0,
            "per-area neuron count must be settled for registered areas"
        );
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
        service.register_memory_area(
            100,
            "mem_ltm_test".to_string(),
            1,
            vec![1],
            None,
            MemoryMpMode::PatternOnly,
        );

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
        assert_eq!(runtime.effective_mp_mode.as_deref(), Some("pattern_only"));
        assert_eq!(
            service
                .memory_cortical_area_runtime_info(999)
                .effective_mp_mode,
            None
        );
    }

    #[test]
    fn test_runtime_info_reports_change_mode_disabled_at_depth_one() {
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 16, 16, 8).unwrap(),
            "plasticity-test-npu",
        ));
        let service = PlasticityService::new(
            PlasticityConfig::default(),
            create_memory_stats_cache(),
            npu,
        );
        let diff = MemoryMpMode::Change(MpChangeEncoding::Differential { quantization: 1.0 });
        assert!(service.register_memory_area(100, "d1".into(), 1, vec![1], None, diff));
        assert!(service.register_memory_area(101, "d2".into(), 2, vec![1], None, diff));
        let effective = |idx| {
            service
                .memory_cortical_area_runtime_info(idx)
                .effective_mp_mode
        };
        assert_eq!(effective(100).as_deref(), Some("pattern_only"));
        assert_eq!(effective(101).as_deref(), Some("mp_differential"));
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
        service.register_memory_area(
            100,
            area_name.to_string(),
            1,
            vec![1],
            None,
            MemoryMpMode::PatternOnly,
        );

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

        let memory_neuron_id = {
            let array = service.memory_neuron_array.lock().unwrap();
            array.get_active_neurons_by_area(100)[0]
        };
        {
            let mut npu = service.npu.lock().unwrap();
            npu.add_synapse(
                feagi_npu_neural::types::NeuronId(7),
                feagi_npu_neural::types::NeuronId(memory_neuron_id),
                feagi_npu_neural::types::SynapticWeight(900.0),
                feagi_npu_neural::types::SynapticPsp(500.0),
                feagi_npu_neural::types::SynapseType::Excitatory,
                feagi_npu_neural::synapse::SYNAPSE_EDGE_ASSOCIATIVE_MEMORY,
                1,
            )
            .expect("associative synapse");
            npu.rebuild_synapse_index();
            assert_eq!(npu.get_incoming_synapses(memory_neuron_id).len(), 1);
        }

        let reset_count = service.reset_memory_neurons_in_area(100);
        assert_eq!(reset_count, 3);
        assert!(
            service
                .npu
                .lock()
                .unwrap()
                .get_incoming_synapses(memory_neuron_id)
                .is_empty(),
            "reset must delete associative synapses onto the memory neuron"
        );

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
            MemoryMpMode::PatternOnly,
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

    /// A kernel-mode match fires depth `z` of the class output. A different
    /// signature does not light it.
    #[test]
    fn episodic_scan_injects_match_into_bound_twin_only() {
        use std::collections::HashMap;

        const FIELD_IDX: u32 = 10;
        const KERNEL_IDX: u32 = 11;
        const TWIN_IDX: u32 = 12;
        const CLASS_CHANNEL: u32 = 1;
        const CLASS_COUNT: u32 = 3;

        let cache = create_memory_stats_cache();
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 64, 64, 8).unwrap(),
            "classifier-scan-injection-npu",
        ));
        let _service = PlasticityService::new(PlasticityConfig::default(), cache, Arc::clone(&npu));

        let (field_neuron, twin_match, twin_decoy, timestep) = {
            let mut guard = npu.lock().unwrap();
            guard.register_cortical_area(FIELD_IDX, "Y2ZpZWxkMDE=".to_string());
            guard.register_cortical_area(KERNEL_IDX, "bWttZW0wMDE=".to_string());
            guard.register_cortical_area(TWIN_IDX, "Y3R3aW4wMDE=".to_string());
            guard.configure_fire_ledger_window(FIELD_IDX, 1).unwrap();
            guard.configure_fire_ledger_window(TWIN_IDX, 1).unwrap();
            guard.enable_fire_ledger_mp_archival(TWIN_IDX).unwrap();
            let field_neuron = guard
                .add_neuron(
                    1.0,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    false,
                    FIELD_IDX,
                    0,
                    0,
                    0,
                )
                .unwrap();
            // The old one-hot location of the class. Single-layer twins never write it.
            let twin_decoy = guard
                .add_neuron(
                    1.0,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    false,
                    TWIN_IDX,
                    0,
                    0,
                    CLASS_CHANNEL,
                )
                .unwrap();
            // Threshold far above the class potential: the write must bypass LIF.
            let twin_match = guard
                .add_neuron(
                    50.0,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    false,
                    TWIN_IDX,
                    0,
                    0,
                    0,
                )
                .unwrap();
            guard.inject_sensory_with_potentials(&[(field_neuron, 2.0)]);
            let burst = guard.process_burst().unwrap();
            assert!(
                burst.fired_neurons.contains(&field_neuron),
                "the field must fire before the scan can match it"
            );
            (field_neuron, twin_match, twin_decoy, burst.burst)
        };

        let signature = super::spatial_signature_hash(&[vec![(0, 0, 0)]]);
        let mut array = MemoryNeuronArray::new(16);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 100,
            longterm_threshold: 100,
            ..Default::default()
        };
        let neuron_idx = array
            .create_memory_neuron(0xC1A5, KERNEL_IDX, 0, &config)
            .unwrap();
        array.set_spatial_signature(neuron_idx, signature);
        array.bind_class_channel(neuron_idx, CLASS_CHANNEL);
        assert_eq!(array.check_longterm_conversion(100), vec![neuron_idx]);

        let scan = MemoryScanConfig {
            kernel: super::ScanKernel {
                width: 1,
                height: 1,
                depth: 1,
            },
            min_window_activity: 1,
            scan_skip_density: 1.0,
            class_channel_count: CLASS_COUNT,
            class_area_width: 1,
            class_area_height: 1,
            class_memory_area_idx: 99,
            sources: vec![MemoryScanSource {
                field_area_idx: FIELD_IDX,
                twin_area_idx: TWIN_IDX,
                field_width: 1,
                field_height: 1,
                field_depth: 1,
            }],
            scanner_mask: None,
            reward: None,
        };
        let mut areas = HashMap::new();
        areas.insert(
            KERNEL_IDX,
            MemoryAreaConfig {
                temporal_depth: 1,
                upstream_areas: vec![FIELD_IDX],
                mp_mode: MemoryMpMode::PatternOnly,
                scan: Some(scan),
            },
        );
        PlasticityService::run_episodic_scan(&npu, &mut array, &areas, timestep);
        let matched = npu.lock().unwrap().process_burst().unwrap();
        assert!(
            matched.fired_neurons.contains(&twin_decoy),
            "kernel mode fires the class depth, matching the class input"
        );
        assert!(
            !matched.fired_neurons.contains(&twin_match),
            "kernel mode does not stamp a potential on z 0"
        );
        assert!(
            !matched.fired_neurons.contains(&field_neuron),
            "the scan must not re-fire the field"
        );

        array.set_spatial_signature(neuron_idx, signature.wrapping_add(1));
        let missed_timestep = {
            let mut guard = npu.lock().unwrap();
            guard.inject_sensory_with_potentials(&[(field_neuron, 2.0)]);
            guard.process_burst().unwrap().burst
        };
        PlasticityService::run_episodic_scan(&npu, &mut array, &areas, missed_timestep);
        let missed = npu.lock().unwrap().process_burst().unwrap();
        assert!(
            !missed.fired_neurons.contains(&twin_match),
            "a signature that does not match long-term memory must not light the twin"
        );
    }

    /// A class label binds only to kernel patterns encoded in the same burst, so earlier
    /// patterns keep their own labels and recall does not tie across every class.
    #[test]
    fn class_label_binds_only_to_co_encoded_kernel_patterns() {
        const KERNEL_IDX: u32 = 11;
        const CLASS_MEM_IDX: u32 = 12;
        const CLASS_COUNT: u32 = 10;

        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 100,
            longterm_threshold: 100,
            ..Default::default()
        };
        let mut array = MemoryNeuronArray::new(16);
        let earlier = array
            .create_memory_neuron(0xA, KERNEL_IDX, 0, &config)
            .unwrap();
        let current = array
            .create_memory_neuron(0xB, KERNEL_IDX, 1, &config)
            .unwrap();
        let class_neuron = array
            .create_memory_neuron(0xC, CLASS_MEM_IDX, 1, &config)
            .unwrap();
        assert_eq!(array.check_longterm_conversion(100).len(), 3);
        array.bind_class_channel(earlier, 3);

        let mut areas = HashMap::new();
        areas.insert(
            KERNEL_IDX,
            MemoryAreaConfig {
                temporal_depth: 1,
                upstream_areas: vec![],
                mp_mode: MemoryMpMode::PatternOnly,
                scan: Some(MemoryScanConfig {
                    kernel: super::ScanKernel {
                        width: 1,
                        height: 1,
                        depth: 1,
                    },
                    min_window_activity: 1,
                    scan_skip_density: 1.0,
                    class_channel_count: CLASS_COUNT,
                    class_area_width: 1,
                    class_area_height: 1,
                    class_memory_area_idx: CLASS_MEM_IDX,
                    sources: vec![],
                    scanner_mask: None,
                    reward: None,
                }),
            },
        );
        let label_seven = ReplayFrame {
            offset: 0,
            upstream_area_idx: 0,
            coords: vec![(0, 0, 7)],
            membrane_potentials: None,
        };
        let encoded = vec![
            (KERNEL_IDX, current, Vec::new()),
            (CLASS_MEM_IDX, class_neuron, vec![label_seven]),
        ];
        PlasticityService::bind_classifier_class_channels(&mut array, &areas, &encoded);

        assert_eq!(array.get_class_channels(current), vec![7]);
        assert_eq!(
            array.get_class_channels(earlier),
            vec![3],
            "a pattern not encoded this burst must keep only its own label"
        );
    }

    /// Scanner training writes one memory neuron per labeled window and does not
    /// inject that window back into the kernel input.
    #[test]
    fn scanner_training_learns_mask_channel_without_kernel_injection() {
        const FIELD_IDX: u32 = 10;
        const KERNEL_IDX: u32 = 11;
        const MASK_IDX: u32 = 13;
        const CLASS_CHANNEL: u32 = 1;
        const CLASS_COUNT: u32 = 3;

        let cache = create_memory_stats_cache();
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 64, 64, 8).unwrap(),
            "classifier-scanner-training-npu",
        ));
        let service = PlasticityService::new(PlasticityConfig::default(), cache, Arc::clone(&npu));
        let lifecycle = MemoryNeuronLifecycleConfig {
            initial_lifespan: 100,
            longterm_threshold: 100,
            ..Default::default()
        };
        assert!(service.register_memory_area(
            KERNEL_IDX,
            "kernel_mem".to_string(),
            1,
            vec![FIELD_IDX],
            Some(lifecycle),
            MemoryMpMode::PatternOnly,
        ));

        let (_labeled_neuron, _unlabeled_neuron, timestep) = {
            let mut guard = npu.lock().unwrap();
            guard.register_cortical_area(FIELD_IDX, "Y2ZpZWxkMDE=".to_string());
            guard.register_cortical_area(MASK_IDX, "Y21hc2swMDE=".to_string());
            guard.configure_fire_ledger_window(FIELD_IDX, 1).unwrap();
            guard.configure_fire_ledger_window(MASK_IDX, 1).unwrap();
            guard.enable_fire_ledger_mp_archival(MASK_IDX).unwrap();
            let labeled_neuron = guard
                .add_neuron(
                    1.0,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    false,
                    FIELD_IDX,
                    0,
                    0,
                    0,
                )
                .unwrap();
            let unlabeled_neuron = guard
                .add_neuron(
                    1.0,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    false,
                    FIELD_IDX,
                    1,
                    0,
                    0,
                )
                .unwrap();
            // Single-layer mask pixel: the class rides on the potential.
            let mask_neuron = guard
                .add_neuron(
                    0.0001,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    false,
                    MASK_IDX,
                    0,
                    0,
                    0,
                )
                .unwrap();
            let class_potential = encode_class_potential(CLASS_CHANNEL, CLASS_COUNT).unwrap();
            guard.inject_sensory_with_potentials(&[
                (labeled_neuron, 2.0),
                (unlabeled_neuron, 2.0),
                (mask_neuron, class_potential),
            ]);
            let burst = guard.process_burst().unwrap();
            assert!(burst.fired_neurons.contains(&labeled_neuron));
            assert!(burst.fired_neurons.contains(&mask_neuron));
            (labeled_neuron, unlabeled_neuron, burst.burst)
        };

        let scan = MemoryScanConfig {
            kernel: super::ScanKernel {
                width: 1,
                height: 1,
                depth: 1,
            },
            min_window_activity: 1,
            scan_skip_density: 0.0,
            class_channel_count: CLASS_COUNT,
            class_area_width: 1,
            class_area_height: 1,
            class_memory_area_idx: 99,
            sources: vec![MemoryScanSource {
                field_area_idx: FIELD_IDX,
                twin_area_idx: 12,
                field_width: 2,
                field_height: 1,
                field_depth: 1,
            }],
            scanner_mask: Some(ScannerMaskSource {
                mask_area_idx: MASK_IDX,
                mask_width: 2,
                mask_height: 1,
            }),
            reward: None,
        };
        assert!(service.configure_memory_scan(KERNEL_IDX, Some(scan)));
        let areas = service.memory_areas.lock().unwrap().clone();
        let mut array = service.memory_neuron_array.lock().unwrap();
        let mut commands = Vec::new();
        PlasticityService::run_scanner_training(
            &service.npu,
            &mut array,
            &areas,
            &service.memory_lifecycle_configs,
            &service.memory_area_names,
            &service.memory_stats_cache,
            timestep,
            &mut commands,
            &service.stats,
        );

        let expected_hash = super::spatial_signature_hash(&[vec![(0, 0, 0)]]);
        assert_eq!(
            array.get_stats().active_neurons,
            1,
            "only the masked window is a training sample"
        );
        let learned = array
            .find_neuron_by_pattern(KERNEL_IDX, &expected_hash)
            .expect("the labeled window is stored under its spatial hash");
        assert_eq!(array.get_cortical_area_id(learned), Some(KERNEL_IDX));
        assert_eq!(array.get_class_channels(learned), vec![CLASS_CHANNEL]);
        assert!(commands.iter().any(|command| matches!(
            command,
            PlasticityCommand::RegisterMemoryNeuron { area_idx, .. } if *area_idx == KERNEL_IDX
        )));
        assert!(!commands
            .iter()
            .any(|command| matches!(command, PlasticityCommand::InjectMemoryNeuronToFCL { .. })));
        drop(array);
        drop(areas);
    }

    /// An ambiguous decision weakens only that kernel neuron's class channels.
    /// A single-channel decision, and a kernel that did not match, stay put.
    #[test]
    fn reward_training_pain_is_limited_to_the_ambiguous_decision() {
        const FIELD_IDX: u32 = 10;
        const KERNEL_IDX: u32 = 11;
        const TWIN_IDX: u32 = 12;
        const PAIN_IDX: u32 = 20;
        const PLEASURE_IDX: u32 = 21;

        let cache = create_memory_stats_cache();
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 64, 64, 8).unwrap(),
            "classifier-reward-pain-npu",
        ));
        let _service = PlasticityService::new(PlasticityConfig::default(), cache, Arc::clone(&npu));
        let (timestep, pain_neuron) = {
            let mut guard = npu.lock().unwrap();
            guard.register_cortical_area(FIELD_IDX, "Y2ZpZWxkMDE=".to_string());
            guard.register_cortical_area(KERNEL_IDX, "bWttZW0wMDE=".to_string());
            guard.register_cortical_area(TWIN_IDX, "Y3R3aW4wMDE=".to_string());
            guard.register_cortical_area(PAIN_IDX, "Y3BhaW4wMDE=".to_string());
            guard.register_cortical_area(PLEASURE_IDX, "Y3BsZWEwMDE=".to_string());
            guard.configure_fire_ledger_window(FIELD_IDX, 1).unwrap();
            let field_neuron = guard
                .add_neuron(
                    1.0,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    false,
                    FIELD_IDX,
                    0,
                    0,
                    0,
                )
                .unwrap();
            guard
                .add_neuron(
                    1.0,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    false,
                    TWIN_IDX,
                    0,
                    0,
                    1,
                )
                .unwrap();
            guard
                .add_neuron(
                    1.0,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    false,
                    TWIN_IDX,
                    0,
                    0,
                    2,
                )
                .unwrap();
            let pain_neuron = guard
                .add_neuron(
                    1.0,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    false,
                    PAIN_IDX,
                    0,
                    0,
                    0,
                )
                .unwrap();
            guard
                .add_neuron(
                    1.0,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    false,
                    PLEASURE_IDX,
                    0,
                    0,
                    0,
                )
                .unwrap();
            guard.inject_sensory_with_potentials(&[(field_neuron, 2.0)]);
            (guard.process_burst().unwrap().burst, pain_neuron)
        };

        let signature = super::spatial_signature_hash(&[vec![(0, 0, 0)]]);
        let mut array = MemoryNeuronArray::new(16);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 100,
            longterm_threshold: 100,
            ..Default::default()
        };
        let ambiguous = array
            .create_memory_neuron(0xA11, KERNEL_IDX, 0, &config)
            .unwrap();
        array.set_spatial_signature(ambiguous, signature);
        array.bind_class_channel(ambiguous, 1);
        array.bind_class_channel(ambiguous, 2);
        let quiet = array
            .create_memory_neuron(0xB22, KERNEL_IDX, 0, &config)
            .unwrap();
        array.set_spatial_signature(quiet, signature.wrapping_add(1));
        array.bind_class_channel(quiet, 1);
        assert_eq!(array.check_longterm_conversion(100), vec![ambiguous, quiet]);

        let scan = MemoryScanConfig {
            kernel: super::ScanKernel {
                width: 1,
                height: 1,
                depth: 1,
            },
            min_window_activity: 1,
            scan_skip_density: 1.0,
            class_channel_count: 3,
            class_area_width: 1,
            class_area_height: 1,
            class_memory_area_idx: 99,
            sources: vec![MemoryScanSource {
                field_area_idx: FIELD_IDX,
                twin_area_idx: TWIN_IDX,
                field_width: 1,
                field_height: 1,
                field_depth: 1,
            }],
            scanner_mask: None,
            reward: Some(ClassifierRewardConfig {
                pain_area_idx: PAIN_IDX,
                pleasure_area_idx: PLEASURE_IDX,
                feedback_area_idx: None,
                feedback_layout: AnswerFeedbackLayout::OutputColumn,
                feedback_width: 1,
                feedback_height: 1,
                answer_latency_bursts: 0,
                learn_area_idx: None,
                confidence_area_idx: None,
                confidence_width: 0,
                confidence_height: 0,
                confidence_depth: 0,
                firing_threshold: 1.0,
                pleasure_step: 1.0,
                pain_step: 1.0,
                max_weight: f32::INFINITY,
            }),
        };
        let mut areas = HashMap::new();
        areas.insert(
            KERNEL_IDX,
            MemoryAreaConfig {
                temporal_depth: 1,
                upstream_areas: vec![FIELD_IDX],
                mp_mode: MemoryMpMode::PatternOnly,
                scan: Some(scan),
            },
        );
        PlasticityService::run_episodic_scan(&npu, &mut array, &areas, timestep);
        assert!(array.get_class_channels(ambiguous).is_empty());
        assert_eq!(array.get_class_channels(quiet), vec![1]);
        let fired = npu.lock().unwrap().process_burst().unwrap();
        assert!(
            fired.fired_neurons.contains(&pain_neuron),
            "ambiguous decision stimulates the classifier pain area"
        );
    }

    /// Overlapping windows that disagree on a pixel write it once, with the
    /// strongest class. Ties go to the lower class id.
    #[test]
    fn detection_twin_writes_one_winner_per_pixel() {
        let mut votes: HashMap<(u32, u32), HashMap<u32, f32>> = HashMap::new();
        votes.entry((0, 0)).or_default().insert(4, 1.0);
        votes.entry((0, 0)).or_default().insert(7, 2.5);
        votes.entry((3, 1)).or_default().insert(5, 1.0);
        votes.entry((3, 1)).or_default().insert(2, 1.0);
        let (coords, potentials) = PlasticityService::detection_twin_writes(&votes, 19);
        assert_eq!(coords, vec![(0, 0, 0), (3, 1, 0)]);
        assert_eq!(
            potentials
                .iter()
                .map(|p| decode_class_potential(*p, 19))
                .collect::<Vec<_>>(),
            vec![Some(7), Some(2)]
        );
    }

    /// A correct-answer column confirms one channel and pains the other.
    /// The missed correct channel is bound under pleasure.
    #[test]
    fn reward_training_feedback_splits_pain_and_pleasure_on_one_instance() {
        const FIELD_IDX: u32 = 10;
        const KERNEL_IDX: u32 = 11;
        const TWIN_IDX: u32 = 12;
        const PAIN_IDX: u32 = 20;
        const PLEASURE_IDX: u32 = 21;
        const FEEDBACK_IDX: u32 = 22;

        let cache = create_memory_stats_cache();
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 64, 64, 8).unwrap(),
            "classifier-reward-feedback-npu",
        ));
        let _service = PlasticityService::new(PlasticityConfig::default(), cache, Arc::clone(&npu));
        let timestep = {
            let mut guard = npu.lock().unwrap();
            guard.register_cortical_area(FIELD_IDX, "Y2ZpZWxkMDE=".to_string());
            guard.register_cortical_area(KERNEL_IDX, "bWttZW0wMDE=".to_string());
            guard.register_cortical_area(TWIN_IDX, "Y3R3aW4wMDE=".to_string());
            guard.register_cortical_area(PAIN_IDX, "Y3BhaW4wMDE=".to_string());
            guard.register_cortical_area(PLEASURE_IDX, "Y3BsZWEwMDE=".to_string());
            guard.register_cortical_area(FEEDBACK_IDX, "Y2ZlZWQwMDE=".to_string());
            guard.configure_fire_ledger_window(FIELD_IDX, 1).unwrap();
            guard.configure_fire_ledger_window(FEEDBACK_IDX, 1).unwrap();
            guard.enable_fire_ledger_mp_archival(FEEDBACK_IDX).unwrap();
            let field_neuron = guard
                .add_neuron(
                    1.0,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    false,
                    FIELD_IDX,
                    0,
                    0,
                    0,
                )
                .unwrap();
            // Twin-shaped answer: one layer, class 2 of 3 carried as potential.
            let feedback_neuron = guard
                .add_neuron(
                    0.0001,
                    f32::MAX,
                    0.0,
                    0.0,
                    0,
                    0,
                    1.0,
                    u16::MAX,
                    0,
                    false,
                    FEEDBACK_IDX,
                    0,
                    0,
                    0,
                )
                .unwrap();
            let answer = encode_class_potential(2, 3).unwrap();
            guard.inject_sensory_with_potentials(&[(field_neuron, 2.0), (feedback_neuron, answer)]);
            let burst = guard.process_burst().unwrap();
            assert!(burst.fired_neurons.contains(&feedback_neuron));
            burst.burst
        };

        let signature = super::spatial_signature_hash(&[vec![(0, 0, 0)]]);
        let mut array = MemoryNeuronArray::new(16);
        let config = MemoryNeuronLifecycleConfig {
            initial_lifespan: 100,
            longterm_threshold: 100,
            ..Default::default()
        };
        let neuron_idx = array
            .create_memory_neuron(0xC33, KERNEL_IDX, 0, &config)
            .unwrap();
        array.set_spatial_signature(neuron_idx, signature);
        array.bind_class_channel(neuron_idx, 1);
        assert_eq!(array.check_longterm_conversion(100), vec![neuron_idx]);

        let scan = MemoryScanConfig {
            kernel: super::ScanKernel {
                width: 1,
                height: 1,
                depth: 1,
            },
            min_window_activity: 1,
            scan_skip_density: 1.0,
            class_channel_count: 3,
            class_area_width: 1,
            class_area_height: 1,
            class_memory_area_idx: 99,
            sources: vec![MemoryScanSource {
                field_area_idx: FIELD_IDX,
                twin_area_idx: TWIN_IDX,
                field_width: 1,
                field_height: 1,
                field_depth: 1,
            }],
            scanner_mask: None,
            reward: Some(ClassifierRewardConfig {
                pain_area_idx: PAIN_IDX,
                pleasure_area_idx: PLEASURE_IDX,
                feedback_area_idx: Some(FEEDBACK_IDX),
                feedback_layout: AnswerFeedbackLayout::OutputColumn,
                feedback_width: 1,
                feedback_height: 1,
                answer_latency_bursts: 0,
                learn_area_idx: None,
                confidence_area_idx: None,
                confidence_width: 0,
                confidence_height: 0,
                confidence_depth: 0,
                firing_threshold: 1.0,
                pleasure_step: 1.0,
                pain_step: 1.0,
                max_weight: f32::INFINITY,
            }),
        };
        let mut areas = HashMap::new();
        areas.insert(
            KERNEL_IDX,
            MemoryAreaConfig {
                temporal_depth: 1,
                upstream_areas: vec![FIELD_IDX],
                mp_mode: MemoryMpMode::PatternOnly,
                scan: Some(scan),
            },
        );
        PlasticityService::run_episodic_scan(&npu, &mut array, &areas, timestep);
        assert_eq!(array.get_class_channels(neuron_idx), vec![2]);
        assert_eq!(array.class_channel_weight(neuron_idx, 2), Some(1.0));
        assert_eq!(array.class_channel_weight(neuron_idx, 1), None);
    }
}
