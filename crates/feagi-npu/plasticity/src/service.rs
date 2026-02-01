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

use crate::memory_neuron_array::{MemoryNeuronArray, MemoryNeuronLifecycleConfig};
use crate::memory_stats_cache::{self, MemoryStatsCache};
use crate::pattern_detector::{BatchPatternDetector, PatternConfig};
use crate::stdp::STDPConfig;

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
}

/// Replay frame describing a single temporal slice for an upstream area.
#[derive(Debug, Clone)]
pub struct ReplayFrame {
    pub offset: u32,
    pub upstream_area_idx: u32,
    pub coords: Vec<(u32, u32, u32)>,
}

/// Memory area configuration
#[derive(Debug, Clone)]
pub struct MemoryAreaConfig {
    pub temporal_depth: u32,
    pub upstream_areas: Vec<u32>,
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

        Self {
            config,
            npu,
            pattern_detector,
            memory_neuron_array: Arc::new(Mutex::new(MemoryNeuronArray::new(
                memory_array_capacity,
            ))),
            memory_areas: Arc::new(Mutex::new(HashMap::new())),
            memory_lifecycle_configs: Arc::new(Mutex::new(HashMap::new())),
            memory_area_names: Arc::new(Mutex::new(HashMap::new())),
            cv: Arc::new((Mutex::new((false, 0)), Condvar::new())),
            command_queue: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(PlasticityStats::default())),
            memory_stats_cache,
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
    ) {
        let memory_areas_snapshot = memory_areas.lock().unwrap().clone();

        // Log plasticity status every 100 bursts
        if current_timestep.is_multiple_of(100) {
            if memory_areas_snapshot.is_empty() {
                // This is normal if plasticity isn't being used - log at debug level instead of warn
                tracing::debug!(target: "plasticity",
                    "[PLASTICITY] Burst {} - No memory areas registered (plasticity not in use)",
                    current_timestep
                );
            } else {
                tracing::info!(target: "plasticity",
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
            tracing::debug!(target: "plasticity",
                "[PLASTICITY-DEBUG] Burst {} - Processing memory area {} with {} upstream areas: {:?}",
                current_timestep, memory_area_idx, area_config.upstream_areas.len(), area_config.upstream_areas
            );

            // Query FireLedger for upstream firing history (CPU-resident, dense burst-aligned windows)
            let plasticity_lock_start = std::time::Instant::now();
            tracing::debug!(
                "[NPU-LOCK] PLASTICITY: Acquiring NPU lock for FireLedger query (burst {}, area {})",
                current_timestep,
                memory_area_idx
            );
            let (timestep_bitmaps, windows) = {
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
                    (Vec::new(), Vec::new())
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
                                tracing::debug!(target: "plasticity",
                                    "[PLASTICITY-DEBUG] Burst {} - Upstream area {} dense window unavailable (depth={}): {}",
                                    current_timestep, upstream_area_idx, temporal_depth, e
                                );
                                windows_ok = false;
                                break;
                            }
                        };
                        let frame_counts: Vec<u64> =
                            window.iter().map(|(_, bm)| bm.len()).collect();
                        let total_fired: u64 = frame_counts.iter().sum();
                        tracing::debug!(target: "plasticity",
                            "[PLASTICITY-DEBUG] Burst {} - Upstream area {} window covers {}..{} ({} frames) fired_counts={:?} total_fired={}",
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
                        (Vec::new(), Vec::new())
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
                            (Vec::new(), Vec::new())
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
                            (out, windows)
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
                tracing::debug!(target: "plasticity",
                    "[PLASTICITY-DEBUG] Burst {} - Memory area {} has NO firing history from upstream areas - skipping",
                    current_timestep, memory_area_idx
                );
                // No firing history available for upstream areas - skip pattern detection
                continue;
            }

            tracing::debug!(target: "plasticity",
                "[PLASTICITY-DEBUG] Burst {} - Memory area {} has {} timestep bitmaps for pattern detection",
                current_timestep, memory_area_idx, timestep_bitmaps.len()
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
                let replay_frames = Self::build_replay_frames(npu, &windows);
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

                        // Register and inject reactivated neuron
                        commands.push(PlasticityCommand::RegisterMemoryNeuron {
                            neuron_id,
                            area_idx: *memory_area_idx,
                            threshold: 1.5,
                            membrane_potential: 0.0,
                        });

                        if replay_frames.is_empty() {
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
                            replay_frames: replay_frames.clone(),
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
                tracing::debug!(target: "plasticity",
                    "[PLASTICITY-DEBUG] Burst {} - No pattern detected for memory area {}",
                    current_timestep, memory_area_idx
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
    fn build_replay_frames(
        npu: &Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
        windows: &[(u32, Vec<(u64, roaring::RoaringBitmap)>)],
    ) -> Vec<ReplayFrame> {
        if windows.is_empty() {
            return Vec::new();
        }

        let npu_lock = npu.lock().unwrap();
        let mut frames = Vec::new();
        let mut empty_bitmaps = 0usize;
        let mut missing_coords = 0usize;
        for (upstream_area_idx, window) in windows {
            for (offset, (_timestep, bitmap)) in window.iter().enumerate() {
                if bitmap.is_empty() {
                    empty_bitmaps += 1;
                    continue;
                }

                let mut coords: Vec<(u32, u32, u32)> = bitmap
                    .iter()
                    .filter_map(|neuron_id| npu_lock.get_neuron_coordinates(neuron_id))
                    .collect();
                if coords.is_empty() {
                    missing_coords += 1;
                    continue;
                }
                coords.sort_unstable();

                frames.push(ReplayFrame {
                    offset: offset as u32,
                    upstream_area_idx: *upstream_area_idx,
                    coords,
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

    /// Register a memory area for pattern detection
    pub fn register_memory_area(
        &self,
        area_idx: u32,
        area_name: String,
        temporal_depth: u32,
        upstream_areas: Vec<u32>,
        lifecycle_config: Option<MemoryNeuronLifecycleConfig>,
    ) -> bool {
        let upstream_len = upstream_areas.len();
        let upstream_clone = upstream_areas.clone();
        let mut areas = self.memory_areas.lock().unwrap();
        areas.insert(
            area_idx,
            MemoryAreaConfig {
                temporal_depth,
                upstream_areas,
            },
        );

        let mut names = self.memory_area_names.lock().unwrap();
        names.insert(area_idx, area_name.clone());

        if let Some(config) = lifecycle_config {
            let mut configs = self.memory_lifecycle_configs.lock().unwrap();
            configs.insert(area_idx, config);
        }

        // Ensure FireLedger tracks upstream areas for the requested temporal depth.
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

    /// Dequeue plasticity commands
    pub fn dequeue_commands(&self, max_count: usize) -> Vec<PlasticityCommand> {
        let mut queue = self.command_queue.lock().unwrap();
        let count = queue.len().min(max_count);
        queue.drain(..count).collect()
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
                        neuron_id, area_idx, ..
                    } => {
                        tracing::debug!(
                            target: "plasticity",
                            "[PLASTICITY-SVC] RegisterMemoryNeuron area={} neuron_id={}",
                            area_idx,
                            neuron_id
                        );
                    }
                    PlasticityCommand::MemoryNeuronConvertedToLtm {
                        neuron_id, area_idx, ..
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
    fn test_register_memory_area() {
        let config = PlasticityConfig::default();
        let cache = create_memory_stats_cache();
        let npu = Arc::new(TracingMutex::new(
            DynamicNPU::new_f32(StdRuntime::new(), CPUBackend::new(), 16, 16, 8).unwrap(),
            "plasticity-test-npu",
        ));
        let service = PlasticityService::new(config, cache, npu);

        let result = service.register_memory_area(100, "mem_00".to_string(), 3, vec![1, 2], None);
        assert!(result);

        let areas = service.memory_areas.lock().unwrap();
        assert!(areas.contains_key(&100));
    }
}
