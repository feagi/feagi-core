// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # Pure Rust Burst Loop Runner
//!
//! Runs the burst processing loop in a dedicated thread with NO Python overhead.
//!
//! ## Design
//! - Runs in native Rust thread (no GIL contention)
//! - Zero FFI crossings in hot path
//! - Adaptive timing for RTOS-like precision
//! - Power neurons injected every burst
//! - Sensory neurons injected by separate threads directly into FCL

use crate::parameter_update_queue::ParameterUpdateQueue;
use crate::sensory::AgentManager;
use crate::update_sim_timestep_from_hz;
#[cfg(feature = "std")]
use crate::{tracing_mutex::TracingMutex, DynamicNPU};
use feagi_npu_neural::types::NeuronId;
use parking_lot::RwLock as ParkingLotRwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Type alias for fire queue sample data structure
type FireQueueSample = ahash::AHashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>;

/// Decoded sensory data: list of (cortical ID, XYZP list per cortical area)
type SensoryXyzpDecoded = Vec<(
    feagi_structures::genomic::cortical_area::CorticalID,
    Vec<(u32, u32, u32, f32)>,
)>;

use tracing::{debug, error, info, trace, warn};

use std::thread;

/// Trait for visualization publishing (abstraction to avoid circular dependency with feagi-io)
/// Any component that can publish visualization data implements this trait.
/// Raw fire queue data for a single cortical area
/// This is the unencoded data that will be serialized by PNS
#[derive(Debug, Clone)]
pub struct RawFireQueueData {
    pub cortical_area_idx: u32,
    pub cortical_id: String, // Cortical ID (base64) - obtained from ConnectomeManager, not NPU
    pub neuron_ids: Vec<u32>,
    pub coords_x: Vec<u32>,
    pub coords_y: Vec<u32>,
    pub coords_z: Vec<u32>,
    pub potentials: Vec<f32>,
}

/// Complete fire queue snapshot for visualization
pub type RawFireQueueSnapshot = ahash::AHashMap<u32, RawFireQueueData>;

pub trait VisualizationPublisher: Send + Sync {
    /// Publish raw fire queue data (PNS will serialize and compress)
    /// This keeps serialization out of the burst engine hot path
    fn publish_raw_fire_queue_for_agent(
        &self,
        agent_id: &str,
        fire_data: RawFireQueueSnapshot,
    ) -> Result<(), String>;
}

pub trait MotorPublisher: Send + Sync {
    /// Publish motor data to a specific agent (XYZP format)
    fn publish_motor(&self, agent_id: &str, data: &[u8]) -> Result<(), String>;
}

/// Returns true when a publish error indicates FEAGI no longer has that agent.
///
/// Upstream error strings currently include a historical typo ("Nonexistant").
/// We match both spellings so stale subscriptions are pruned reliably.
fn is_missing_agent_publish_error(error_message: &str) -> bool {
    error_message.contains("Nonexistant Agent ID") || error_message.contains("Nonexistent Agent ID")
}

/// Transport-agnostic source of sensory bytes (FeagiByteContainer format).
/// Implemented by feagi-io; fed by any transport (ZMQ, WebSocket, SHM, etc.).
pub trait SensoryIntake: Send {
    /// Poll for next sensory payload if available.
    /// Returns serialized FeagiByteContainer bytes.
    fn poll_sensory_data(&mut self) -> Result<Option<Vec<u8>>, String>;
}

/// Burst loop runner - manages the main neural processing loop
///
/// 🦀 Power neurons are stored in RustNPU, not here - 100% Rust!
/// 🦀 Burst count is stored in NPU - single source of truth!
pub struct BurstLoopRunner {
    /// Shared NPU instance (holds power neurons internally + burst count)
    /// DynamicNPU dispatches to either F32 or INT8 variant based on genome
    /// Wrapped in TracingMutex to automatically log all lock acquisitions
    npu: Arc<TracingMutex<DynamicNPU>>,
    /// Target frequency in Hz (shared with burst thread for dynamic updates)
    frequency_hz: Arc<Mutex<f64>>,
    /// Running flag (atomic for thread-safe stop)
    running: Arc<AtomicBool>,
    /// Thread handle (for graceful shutdown)
    thread_handle: Option<thread::JoinHandle<()>>,
    /// Sensory agent manager (per-agent injection threads - SHM-based agents)
    pub sensory_manager: Arc<Mutex<AgentManager>>,
    /// Transport-agnostic sensory intake (feagi-io); fed by any transport, consumed by burst loop
    pub sensory_intake: Option<Arc<Mutex<dyn SensoryIntake>>>,
    /// Visualization SHM writer (optional, None if not configured)
    pub viz_shm_writer: Arc<Mutex<Option<crate::viz_shm_writer::VizSHMWriter>>>,
    /// Motor SHM writer (optional, None if not configured)
    pub motor_shm_writer: Arc<Mutex<Option<crate::motor_shm_writer::MotorSHMWriter>>>,
    /// Visualization publisher for direct Rust-to-Rust publishing (NO PYTHON IN HOT PATH)
    /// Uses trait abstraction to avoid circular dependency with feagi-io
    pub viz_publisher: Option<Arc<dyn VisualizationPublisher>>,
    /// Motor publisher for agent-specific motor command publishing
    pub motor_publisher: Option<Arc<dyn MotorPublisher>>,
    /// Motor area subscriptions: agent_id → Set<cortical_id>
    /// Stores cortical_id strings (e.g., "omot00"), matching sensory stream pattern
    motor_subscriptions: Arc<ParkingLotRwLock<ahash::AHashMap<String, ahash::AHashSet<String>>>>,
    /// Per-agent motor output rates (Hz)
    motor_output_rates_hz: Arc<ParkingLotRwLock<ahash::AHashMap<String, f64>>>,
    /// Per-agent motor output last publish timestamps
    motor_last_publish_time: Arc<ParkingLotRwLock<ahash::AHashMap<String, Instant>>>,
    /// Visualization subscriptions (agent IDs)
    visualization_subscriptions: Arc<ParkingLotRwLock<ahash::AHashSet<String>>>,
    /// Per-agent visualization output rates (Hz)
    visualization_output_rates_hz: Arc<ParkingLotRwLock<ahash::AHashMap<String, f64>>>,
    /// Per-agent visualization last publish timestamps
    visualization_last_publish_time: Arc<ParkingLotRwLock<ahash::AHashMap<String, Instant>>>,
    /// FCL/FQ sampler configuration
    fcl_sampler_frequency: Arc<Mutex<f64>>, // Sampling frequency in Hz
    fcl_sampler_consumer: Arc<Mutex<u32>>, // Consumer type: 1=visualization, 2=motor, 3=both
    /// Cached burst count (shared reference to NPU's atomic) for lock-free reads
    cached_burst_count: Arc<std::sync::atomic::AtomicU64>,
    /// Cached fire queue from last burst (for API queries)
    /// Stored as Arc to avoid cloning when sharing between viz and motor
    cached_fire_queue: Arc<Mutex<Option<Arc<FireQueueSample>>>>,
    /// Parameter update queue (asynchronous, non-blocking)
    /// API pushes updates here, burst loop consumes between bursts
    pub parameter_queue: ParameterUpdateQueue,
    /// Plasticity burst notification callback (called after each burst)
    /// Callback receives the current burst/timestep number
    /// Uses Fn trait object to avoid circular dependency with plasticity crate
    plasticity_notify: Option<Arc<dyn Fn(u64) + Send + Sync>>,
    /// Optional post-burst callback invoked after NPU lock release
    /// Use for external integrations that must run outside the NPU lock
    post_burst_callback: Option<Arc<dyn Fn(u64) + Send + Sync>>,
    /// Cached cortical_idx -> cortical_id mappings (from ConnectomeManager, not NPU)
    /// Refreshed periodically to avoid ConnectomeManager lock contention
    /// This eliminates NPU lock acquisitions that were causing 1-3s delays!
    cached_cortical_id_mappings: Arc<Mutex<ahash::AHashMap<u32, String>>>,
    /// Burst count when mappings were last refreshed
    last_cortical_id_refresh: Arc<Mutex<u64>>,
    /// Cached cortical_idx -> visualization_voxel_granularity mappings (from ConnectomeManager)
    /// Used to determine when to apply aggregated rendering for large areas
    cached_visualization_granularities: Arc<Mutex<VisualizationGranularityCache>>,
}

type VisualizationGranularity = (u32, u32, u32);
type VisualizationGranularityCache = ahash::AHashMap<u32, VisualizationGranularity>;

impl BurstLoopRunner {
    /// Create a new burst loop runner
    ///
    /// # Arguments
    /// * `npu` - The NPU to run bursts on
    /// * `viz_publisher` - Optional visualization publisher (None = no ZMQ visualization)
    /// * `frequency_hz` - Burst frequency in Hz
    pub fn new<V: VisualizationPublisher + 'static, M: MotorPublisher + 'static>(
        npu: Arc<TracingMutex<DynamicNPU>>,
        viz_publisher: Option<Arc<Mutex<V>>>,
        motor_publisher: Option<Arc<Mutex<M>>>,
        frequency_hz: f64,
    ) -> Self {
        // Create FCL injection callback for sensory data
        let npu_for_callback = npu.clone();
        let injection_callback = Arc::new(
            move |cortical_area: u32, xyzp_data: Vec<(u32, u32, u32, f32)>| {
                // 🔍 DEBUG: Log first injection
                static FIRST_CALLBACK_LOGGED: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                if !FIRST_CALLBACK_LOGGED.load(std::sync::atomic::Ordering::Relaxed)
                    && !xyzp_data.is_empty()
                {
                    debug!(
                        "[FCL-INJECT] 🔍 First callback: cortical_area={}, neuron_count={}",
                        cortical_area,
                        xyzp_data.len()
                    );
                    info!(
                        "[FCL-INJECT]    First 3 XYZP: {:?}",
                        &xyzp_data[0..xyzp_data.len().min(3)]
                    );
                    FIRST_CALLBACK_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
                }

                // Convert (x,y,z) to neuron IDs and inject with actual P values
                // 🚀 PERFORMANCE FIX: Do coordinate lookup ONCE using batch API, OUTSIDE the main NPU lock
                // This prevents holding the lock for 1+ seconds while processing 4410 neurons

                let callback_start = std::time::Instant::now();
                info!(
                    "🔍 [SENSORY-CALLBACK] Processing {} XYZP data points for area {}",
                    xyzp_data.len(),
                    cortical_area
                );

                // Step 1: Extract coordinates (no locks needed)
                let coords: Vec<(u32, u32, u32)> =
                    xyzp_data.iter().map(|(x, y, z, _)| (*x, *y, *z)).collect();

                // Step 2: Batch lookup with MINIMAL lock time (only neuron_array read lock, NOT full NPU lock)
                let lookup_start = std::time::Instant::now();
                let neuron_ids = if let Ok(npu_lock) = npu_for_callback.lock() {
                    // Use dispatch method instead of direct neuron_array access
                    let result =
                        npu_lock.batch_get_neuron_ids_from_coordinates(cortical_area, &coords);
                    drop(npu_lock); // Release NPU lock ASAP!
                    result
                } else {
                    warn!("[FCL-INJECT] Failed to acquire NPU lock for coordinate lookup");
                    return;
                };
                let lookup_duration = lookup_start.elapsed();
                info!(
                    "🔍 [SENSORY-CALLBACK] Batch lookup completed in {:?}",
                    lookup_duration
                );

                // 🔍 DEBUG: Log conversion result
                static FIRST_CONVERSION_LOGGED: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                if !FIRST_CONVERSION_LOGGED.load(std::sync::atomic::Ordering::Relaxed)
                    && !neuron_ids.is_empty()
                {
                    info!(
                        "[FCL-INJECT]    Converted {} coords → {} valid neurons",
                        xyzp_data.len(),
                        neuron_ids.len()
                    );
                    info!(
                        "[FCL-INJECT]    First 5 neuron IDs: {:?}",
                        &neuron_ids[0..neuron_ids.len().min(5)]
                    );
                    FIRST_CONVERSION_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
                }

                // Step 3: Build (NeuronId, potential) pairs from batch results (NO LOCKS!)
                let pair_start = std::time::Instant::now();
                let mut neuron_potential_pairs: Vec<(NeuronId, f32)> =
                    Vec::with_capacity(neuron_ids.len());
                for (neuron_id, (_x, _y, _z, p)) in neuron_ids.iter().zip(xyzp_data.iter()) {
                    neuron_potential_pairs.push((*neuron_id, *p));
                }
                let pair_duration = pair_start.elapsed();
                info!(
                    "🔍 [SENSORY-CALLBACK] Built {} pairs in {:?}",
                    neuron_potential_pairs.len(),
                    pair_duration
                );

                // 🔍 DEBUG: Log first few potentials
                static FIRST_POTENTIALS_LOGGED: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                if !FIRST_POTENTIALS_LOGGED.load(std::sync::atomic::Ordering::Relaxed)
                    && !neuron_potential_pairs.is_empty()
                {
                    info!("[FCL-INJECT]    First 5 potentials from data:");
                    for (neuron_id, p) in neuron_potential_pairs.iter().take(5) {
                        info!("[FCL-INJECT]      [{:?}] p={:.3}", neuron_id, p);
                    }
                    FIRST_POTENTIALS_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
                }

                // Step 4: FINAL injection - acquire lock ONLY for this quick operation
                let inject_start = std::time::Instant::now();
                if let Ok(mut npu_lock) = npu_for_callback.lock() {
                    info!(
                        "🔍 [SENSORY-CALLBACK] Acquired NPU lock for injection in {:?}",
                        inject_start.elapsed()
                    );
                    npu_lock.inject_sensory_with_potentials(&neuron_potential_pairs);
                    let inject_duration = inject_start.elapsed();
                    info!(
                        "🔍 [SENSORY-CALLBACK] Injection completed in {:?}",
                        inject_duration
                    );

                    // 🔍 DEBUG: Log injection summary
                    static FIRST_SUMMARY_LOGGED: std::sync::atomic::AtomicBool =
                        std::sync::atomic::AtomicBool::new(false);
                    if !FIRST_SUMMARY_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                        info!(
                            "[FCL-INJECT]    ✅ Injected {} neurons with actual P values from data",
                            neuron_potential_pairs.len()
                        );
                        FIRST_SUMMARY_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                } else {
                    warn!("[FCL-INJECT] Failed to acquire NPU lock for injection");
                }

                let total_duration = callback_start.elapsed();
                info!(
                    "🔍 [SENSORY-CALLBACK] Total callback time: {:?}",
                    total_duration
                );
            },
        );

        let sensory_manager = AgentManager::new(injection_callback);

        // Convert generic publishers to trait objects (if provided)
        let viz_publisher_trait: Option<Arc<dyn VisualizationPublisher>> = viz_publisher.map(|p| {
            // Wrap Arc<Mutex<V>> to implement VisualizationPublisher
            struct VisualizerWrapper<V: VisualizationPublisher>(Arc<Mutex<V>>);
            impl<V: VisualizationPublisher> VisualizationPublisher for VisualizerWrapper<V> {
                fn publish_raw_fire_queue_for_agent(
                    &self,
                    agent_id: &str,
                    fire_data: RawFireQueueSnapshot,
                ) -> Result<(), String> {
                    self.0
                        .lock()
                        .unwrap()
                        .publish_raw_fire_queue_for_agent(agent_id, fire_data)
                }
            }
            Arc::new(VisualizerWrapper(p)) as Arc<dyn VisualizationPublisher>
        });

        let motor_publisher_trait: Option<Arc<dyn MotorPublisher>> = motor_publisher.map(|p| {
            // Wrap Arc<Mutex<M>> to implement MotorPublisher
            struct MotorWrapper<M: MotorPublisher>(Arc<Mutex<M>>);
            impl<M: MotorPublisher> MotorPublisher for MotorWrapper<M> {
                fn publish_motor(&self, agent_id: &str, data: &[u8]) -> Result<(), String> {
                    self.0.lock().unwrap().publish_motor(agent_id, data)
                }
            }
            Arc::new(MotorWrapper(p)) as Arc<dyn MotorPublisher>
        });

        Self {
            npu,
            frequency_hz: Arc::new(Mutex::new(frequency_hz)), // Shared with burst thread for dynamic updates
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            sensory_manager: Arc::new(Mutex::new(sensory_manager)),
            sensory_intake: None, // Can be set later via set_sensory_intake()
            cached_cortical_id_mappings: Arc::new(Mutex::new(ahash::AHashMap::new())),
            last_cortical_id_refresh: Arc::new(Mutex::new(0)),
            cached_visualization_granularities: Arc::new(Mutex::new(ahash::AHashMap::new())),
            viz_shm_writer: Arc::new(Mutex::new(None)), // Initialized later via attach_viz_shm_writer
            motor_shm_writer: Arc::new(Mutex::new(None)), // Initialized later via attach_motor_shm_writer
            viz_publisher: viz_publisher_trait, // Trait object for visualization (NO PYTHON CALLBACKS!)
            motor_publisher: motor_publisher_trait, // Trait object for motor (NO PYTHON CALLBACKS!)
            motor_subscriptions: Arc::new(ParkingLotRwLock::new(ahash::AHashMap::new())),
            motor_output_rates_hz: Arc::new(ParkingLotRwLock::new(ahash::AHashMap::new())),
            motor_last_publish_time: Arc::new(ParkingLotRwLock::new(ahash::AHashMap::new())),
            visualization_subscriptions: Arc::new(ParkingLotRwLock::new(ahash::AHashSet::new())),
            visualization_output_rates_hz: Arc::new(ParkingLotRwLock::new(ahash::AHashMap::new())),
            visualization_last_publish_time: Arc::new(
                ParkingLotRwLock::new(ahash::AHashMap::new()),
            ),
            fcl_sampler_frequency: Arc::new(Mutex::new(30.0)), // Default 30Hz for visualization
            fcl_sampler_consumer: Arc::new(Mutex::new(1)),     // Default: 1 = visualization only
            cached_burst_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            cached_fire_queue: Arc::new(Mutex::new(None)), // Cached fire queue for API (Arc-wrapped to avoid cloning)
            parameter_queue: ParameterUpdateQueue::new(),
            plasticity_notify: None, // Initialized later via set_plasticity_notify_callback
            post_burst_callback: None, // Initialized later via set_post_burst_callback
        }
    }

    /// Set plasticity burst notification callback (called during initialization if plasticity is enabled)
    /// This avoids circular dependency between burst-engine and plasticity crates
    pub fn set_plasticity_notify_callback<F>(&mut self, callback: F)
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        self.plasticity_notify = Some(Arc::new(callback));
        info!("[BURST-RUNNER] Plasticity notification callback attached");
    }

    /// Set a post-burst callback that runs after the NPU lock is released.
    pub fn set_post_burst_callback<F>(&mut self, callback: F)
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        self.post_burst_callback = Some(Arc::new(callback));
        info!("[BURST-RUNNER] Post-burst callback attached");
    }

    /// Return true if a post-burst callback is configured.
    pub fn has_post_burst_callback(&self) -> bool {
        self.post_burst_callback.is_some()
    }

    /// Attach visualization SHM writer (called from Python after registration)
    pub fn attach_viz_shm_writer(
        &mut self,
        shm_path: std::path::PathBuf,
    ) -> Result<(), std::io::Error> {
        // IMPORTANT (macOS SIGBUS avoidance / correctness):
        // Creating a new SHM writer truncates/reinitializes the backing file. If the burst loop is
        // concurrently writing via an existing mmap, truncation can SIGBUS this process.
        //
        // Therefore we must take the writer lock first (blocking burst writes), drop any existing
        // writer (unmaps), and only then create the new writer and install it.
        let mut guard = self.viz_shm_writer.lock().unwrap();
        *guard = None;
        let writer = crate::viz_shm_writer::VizSHMWriter::new(shm_path, None, None)?;
        *guard = Some(writer);
        Ok(())
    }

    /// Attach motor SHM writer (called from Python after registration)
    pub fn attach_motor_shm_writer(
        &mut self,
        shm_path: std::path::PathBuf,
    ) -> Result<(), std::io::Error> {
        // Same race as visualization SHM: motor SHM writer creation truncates the backing file.
        // Take the lock first to prevent concurrent burst writes from using an invalidated mmap.
        let mut guard = self.motor_shm_writer.lock().unwrap();
        *guard = None;
        let writer = crate::motor_shm_writer::MotorSHMWriter::new(shm_path, None, None)?;
        *guard = Some(writer);
        Ok(())
    }

    /// Set transport-agnostic sensory intake (feagi-io; fed by ZMQ, WebSocket, etc.)
    pub fn set_sensory_intake(&mut self, intake: Arc<Mutex<dyn SensoryIntake>>) {
        self.sensory_intake = Some(intake);
        info!("[BURST-RUNNER] Sensory intake attached");
    }

    /// Register an agent's motor subscriptions
    /// Called when an agent registers with motor capability
    /// Stores cortical_id strings (e.g., "omot00"), matching sensory stream pattern
    pub fn register_motor_subscriptions(
        &self,
        agent_id: String,
        cortical_ids: ahash::AHashSet<String>,
    ) {
        self.motor_subscriptions
            .write()
            .insert(agent_id.clone(), cortical_ids.clone());

        info!(
            "[BURST-RUNNER] 🎮 Registered motor subscriptions for agent '{}': {:?}",
            agent_id, cortical_ids
        );
    }

    /// Register an agent's motor subscriptions with a rate limit (Hz).
    ///
    /// # Errors
    /// - Returns Err if the requested rate is invalid or exceeds burst frequency.
    pub fn register_motor_subscriptions_with_rate(
        &self,
        agent_id: String,
        cortical_ids: ahash::AHashSet<String>,
        rate_hz: f64,
    ) -> Result<(), String> {
        if rate_hz <= 0.0 {
            return Err("Motor rate must be greater than 0".to_string());
        }

        let burst_hz = self.get_frequency();
        if rate_hz > burst_hz {
            return Err(format!(
                "Requested motor rate {}Hz exceeds burst frequency {}Hz",
                rate_hz, burst_hz
            ));
        }

        self.motor_subscriptions
            .write()
            .insert(agent_id.clone(), cortical_ids.clone());
        self.motor_output_rates_hz
            .write()
            .insert(agent_id.clone(), rate_hz);
        self.motor_last_publish_time.write().remove(&agent_id);

        info!(
            "[BURST-RUNNER] Registered motor subscriptions for agent '{}' at {:.2}Hz: {:?}",
            agent_id, rate_hz, cortical_ids
        );

        Ok(())
    }

    /// Unregister an agent's motor subscriptions
    /// Called when an agent disconnects
    pub fn unregister_motor_subscriptions(&self, agent_id: &str) {
        if self.motor_subscriptions.write().remove(agent_id).is_some() {
            self.motor_output_rates_hz.write().remove(agent_id);
            self.motor_last_publish_time.write().remove(agent_id);
            info!(
                "[BURST-RUNNER] Removed motor subscriptions for agent '{}'",
                agent_id
            );
        }
    }

    /// Register a visualization agent with a rate limit (Hz).
    ///
    /// # Errors
    /// - Returns Err if the requested rate is invalid or exceeds burst frequency.
    pub fn register_visualization_subscriptions_with_rate(
        &self,
        agent_id: String,
        rate_hz: f64,
    ) -> Result<(), String> {
        if rate_hz <= 0.0 {
            return Err("Visualization rate must be greater than 0".to_string());
        }

        let burst_hz = self.get_frequency();
        if rate_hz > burst_hz {
            return Err(format!(
                "Requested visualization rate {}Hz exceeds burst frequency {}Hz",
                rate_hz, burst_hz
            ));
        }

        self.visualization_subscriptions
            .write()
            .insert(agent_id.clone());
        self.visualization_output_rates_hz
            .write()
            .insert(agent_id.clone(), rate_hz);
        self.visualization_last_publish_time
            .write()
            .remove(&agent_id);
        info!(
            "[BURST-RUNNER] Registered visualization subscription for agent '{}' at {:.2}Hz",
            agent_id, rate_hz
        );
        Ok(())
    }

    /// Unregister a visualization agent subscription.
    pub fn unregister_visualization_subscriptions(&self, agent_id: &str) {
        if self.visualization_subscriptions.write().remove(agent_id) {
            self.visualization_output_rates_hz.write().remove(agent_id);
            self.visualization_last_publish_time
                .write()
                .remove(agent_id);
            info!(
                "[BURST-RUNNER] Removed visualization subscription for agent '{}'",
                agent_id
            );
        }
    }

    // REMOVED: set_viz_zmq_publisher - NO PYTHON CALLBACKS IN HOT PATH!
    // PNS is now passed directly in constructor for pure Rust-to-Rust communication

    /// Set burst frequency (can be called while running - thread-safe)
    pub fn set_frequency(&mut self, frequency_hz: f64) {
        *self.frequency_hz.lock().unwrap() = frequency_hz;
        info!("[BURST-RUNNER] Frequency set to {:.2} Hz", frequency_hz);
    }

    /// Start the burst loop in a background thread
    ///
    /// 🦀 Power neurons are read from RustNPU internally - 100% Rust!
    pub fn start(&mut self) -> Result<(), String> {
        if self.running.load(Ordering::Acquire) {
            return Err("Burst loop already running".to_string());
        }

        let current_freq = *self.frequency_hz.lock().unwrap();
        info!("[BURST-RUNNER] Starting burst loop at {:.2} Hz (power neurons auto-discovered from cortical_idx=1)",
                 current_freq);

        self.running.store(true, Ordering::Release);

        let npu = self.npu.clone();
        let frequency = self.frequency_hz.clone(); // Clone Arc for thread
        let running = self.running.clone();
        let viz_writer = self.viz_shm_writer.clone();
        let motor_writer = self.motor_shm_writer.clone();
        let viz_publisher = self.viz_publisher.clone(); // Direct Rust-to-Rust trait reference (NO PYTHON CALLBACKS!)
        let motor_publisher = self.motor_publisher.clone(); // Direct Rust-to-Rust trait reference (NO PYTHON CALLBACKS!)
        let motor_subs = self.motor_subscriptions.clone();
        let motor_rates = self.motor_output_rates_hz.clone();
        let motor_last_publish = self.motor_last_publish_time.clone();
        let viz_subs = self.visualization_subscriptions.clone();
        let viz_rates = self.visualization_output_rates_hz.clone();
        let viz_last_publish = self.visualization_last_publish_time.clone();
        let cached_burst_count = self.cached_burst_count.clone(); // For lock-free burst count reads
        let cached_fire_queue = self.cached_fire_queue.clone(); // For caching fire queue data
        let param_queue = self.parameter_queue.clone(); // Parameter update queue
        let plasticity_notify = self.plasticity_notify.clone(); // Clone Arc for thread
        let post_burst_callback = self.post_burst_callback.clone(); // Clone Arc for thread
        let cached_cortical_id_mappings = self.cached_cortical_id_mappings.clone();
        let last_cortical_id_refresh = self.last_cortical_id_refresh.clone();
        let cached_visualization_granularities = self.cached_visualization_granularities.clone();
        let sensory_intake = self.sensory_intake.clone();

        self.thread_handle = Some(
            thread::Builder::new()
                .name("feagi-burst-loop".to_string())
                .spawn(move || {
                    burst_loop(
                        npu,
                        frequency,
                        running,
                        viz_writer,
                        motor_writer,
                        viz_publisher,
                        motor_publisher,
                        motor_subs,
                        motor_rates,
                        motor_last_publish,
                        viz_subs,
                        viz_rates,
                        viz_last_publish,
                        cached_burst_count,
                        cached_fire_queue,
                        param_queue,
                        plasticity_notify,
                        post_burst_callback,
                        cached_cortical_id_mappings,
                        last_cortical_id_refresh,
                        cached_visualization_granularities,
                        sensory_intake,
                    );
                })
                .map_err(|e| format!("Failed to spawn burst loop thread: {}", e))?,
        );

        info!("[BURST-RUNNER] ✅ Burst loop started successfully");
        Ok(())
    }

    /// Stop the burst loop gracefully
    ///
    /// This method sets the shutdown flag and waits up to 2 seconds for the thread to finish.
    /// If the thread doesn't finish within the timeout, it's considered non-responsive
    /// and we proceed with shutdown anyway.
    pub fn stop(&mut self) {
        if !self.running.load(Ordering::Acquire) {
            return; // Already stopped
        }

        info!("[BURST-RUNNER] Stopping burst loop...");
        self.running.store(false, Ordering::Release);

        if let Some(handle) = self.thread_handle.take() {
            // Use a timeout to prevent blocking indefinitely
            // The burst loop checks the flag every 50ms, so 2 seconds should be plenty
            let stop_timeout = std::time::Duration::from_secs(2);
            let start = std::time::Instant::now();

            // Poll join with timeout using a simple loop
            // Note: Rust's std::thread::JoinHandle doesn't have a timeout method,
            // so we'll use a different approach: spawn a thread that waits for join
            let handle_clone = handle;
            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let result = handle_clone.join();
                let _ = tx.send(result);
            });

            match rx.recv_timeout(stop_timeout) {
                Ok(Ok(_)) => {
                    info!("[BURST-RUNNER] ✅ Burst loop stopped cleanly");
                }
                Ok(Err(_)) => {
                    warn!("[BURST-RUNNER] ⚠️ Burst loop thread panicked during shutdown");
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    let elapsed = start.elapsed();
                    warn!(
                        "[BURST-RUNNER] ⚠️ Burst loop did not stop within {:?}, proceeding with shutdown",
                        elapsed
                    );
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    warn!("[BURST-RUNNER] ⚠️ Join thread disconnected unexpectedly");
                }
            }
        }
    }

    /// Check if the burst loop is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    /// Get current burst count (lock-free atomic read)
    ///
    /// Reads from cached value that's updated by the burst loop.
    /// Never blocks, even during burst processing.
    ///
    pub fn get_burst_count(&self) -> u64 {
        self.cached_burst_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get configured burst frequency in Hz
    pub fn get_frequency(&self) -> f64 {
        *self.frequency_hz.lock().unwrap()
    }

    /// Get current FCL snapshot for monitoring/debugging
    /// Returns Vec of (NeuronId, potential) pairs
    pub fn get_fcl_snapshot(&self) -> Vec<(NeuronId, f32)> {
        let lock_start = std::time::Instant::now();
        let thread_id = std::thread::current().id();
        debug!(
            "[NPU-LOCK] BURST-RUNNER: Thread {:?} attempting NPU lock for get_fcl_snapshot at {:?}",
            thread_id, lock_start
        );
        let result = {
            let npu_lock = self.npu.lock().unwrap();
            let lock_acquired = std::time::Instant::now();
            let lock_wait = lock_acquired.duration_since(lock_start);
            debug!(
                "[NPU-LOCK] BURST-RUNNER: Thread {:?} acquired lock after {:.2}ms wait for get_fcl_snapshot",
                thread_id,
                lock_wait.as_secs_f64() * 1000.0
            );
            npu_lock.get_last_fcl_snapshot()
        };
        let lock_released = std::time::Instant::now();
        let total_duration = lock_released.duration_since(lock_start);
        debug!("[NPU-LOCK] BURST-RUNNER: Thread {:?} RELEASED NPU lock after get_fcl_snapshot (total: {:.2}ms, returned {} neurons)", 
            thread_id,
            total_duration.as_secs_f64() * 1000.0,
            result.len());
        result
    }

    /// Get current fire queue for monitoring
    /// Returns the last cached fire queue data from previous burst
    pub fn get_fire_queue_sample(&mut self) -> Option<FireQueueSample> {
        let cached = self.cached_fire_queue.lock().unwrap().clone();
        if let Some(ref sample_arc) = cached {
            debug!(
                "[BURST-LOOP-RUNNER] Returning cached fire queue: {} areas",
                sample_arc.len()
            );
        } else {
            debug!("[BURST-LOOP-RUNNER] Cached fire queue is None");
        }
        // Unwrap Arc to return the actual data (API needs owned data)
        cached.map(|arc| (*arc).clone())
    }

    /// Get Fire Ledger window configurations for all cortical areas
    pub fn get_fire_ledger_configs(&self) -> Vec<(u32, usize)> {
        let lock_start = std::time::Instant::now();
        let thread_id = std::thread::current().id();
        debug!("[NPU-LOCK] BURST-RUNNER: Thread {:?} attempting NPU lock for get_fire_ledger_configs at {:?}", thread_id, lock_start);
        let result = {
            let npu_lock = self.npu.lock().unwrap();
            let lock_acquired = std::time::Instant::now();
            let lock_wait = lock_acquired.duration_since(lock_start);
            debug!(
                "[NPU-LOCK] BURST-RUNNER: Thread {:?} acquired lock after {:.2}ms wait for get_fire_ledger_configs",
                thread_id,
                lock_wait.as_secs_f64() * 1000.0
            );
            npu_lock.get_all_fire_ledger_configs()
        };
        let lock_released = std::time::Instant::now();
        let total_duration = lock_released.duration_since(lock_start);
        debug!("[NPU-LOCK] BURST-RUNNER: Thread {:?} RELEASED NPU lock after get_fire_ledger_configs (total: {:.2}ms, returned {} configs)", 
            thread_id,
            total_duration.as_secs_f64() * 1000.0,
            result.len());
        result
    }

    /// Configure Fire Ledger window size for a specific cortical area
    pub fn configure_fire_ledger_window(
        &mut self,
        cortical_idx: u32,
        window_size: usize,
    ) -> Result<(), String> {
        self.npu
            .lock()
            .unwrap()
            .configure_fire_ledger_window(cortical_idx, window_size)
            .map_err(|e| format!("{e}"))
    }

    /// Get FCL/FQ sampler configuration
    pub fn get_fcl_sampler_config(&self) -> (f64, u32) {
        let frequency = *self.fcl_sampler_frequency.lock().unwrap();
        let consumer = *self.fcl_sampler_consumer.lock().unwrap();
        (frequency, consumer)
    }

    /// Update FCL/FQ sampler configuration
    pub fn set_fcl_sampler_config(&self, frequency: Option<f64>, consumer: Option<u32>) {
        if let Some(freq) = frequency {
            *self.fcl_sampler_frequency.lock().unwrap() = freq;
            tracing::info!(target: "feagi-burst-engine", "FCL sampler frequency updated to {}Hz", freq);
        }
        if let Some(cons) = consumer {
            *self.fcl_sampler_consumer.lock().unwrap() = cons;
            tracing::info!(target: "feagi-burst-engine", "FCL sampler consumer updated to {}", cons);
        }
    }

    /// Get FCL sample rate for a specific cortical area
    /// Note: Currently returns global rate as per-area rates not yet implemented
    pub fn get_area_fcl_sample_rate(&self, _area_id: u32) -> f64 {
        *self.fcl_sampler_frequency.lock().unwrap()
    }

    /// Set FCL sample rate for a specific cortical area
    /// Note: Currently sets global rate as per-area rates not yet implemented
    pub fn set_area_fcl_sample_rate(&self, _area_id: u32, sample_rate: f64) {
        *self.fcl_sampler_frequency.lock().unwrap() = sample_rate;
        tracing::info!(target: "feagi-burst-engine", "FCL sampler frequency updated to {}Hz (global)", sample_rate);
    }

    /// Get reference to NPU for direct access (use sparingly)
    pub fn get_npu(&self) -> Arc<TracingMutex<DynamicNPU>> {
        self.npu.clone()
    }

    /// Refresh cortical_idx -> cortical_id mappings from ConnectomeManager
    /// This should be called when cortical areas are created/updated
    /// CRITICAL: This eliminates NPU lock acquisitions that were causing 1-3s delays!
    pub fn refresh_cortical_id_mappings(&self, mappings: ahash::AHashMap<u32, String>) {
        *self.cached_cortical_id_mappings.lock().unwrap() = mappings;
        let current_burst = self
            .cached_burst_count
            .load(std::sync::atomic::Ordering::Relaxed);
        *self.last_cortical_id_refresh.lock().unwrap() = current_burst;
        debug!(
            "[BURST-LOOP] Refreshed cortical_id mappings: {} areas (burst {})",
            self.cached_cortical_id_mappings.lock().unwrap().len(),
            current_burst
        );
    }

    /// Refresh cortical_idx -> visualization_voxel_granularity mappings from ConnectomeManager
    /// This should be called when cortical areas are created/updated
    pub fn refresh_visualization_granularities(
        &self,
        granularities: ahash::AHashMap<u32, (u32, u32, u32)>,
    ) {
        *self.cached_visualization_granularities.lock().unwrap() = granularities;
        debug!(
            "[BURST-LOOP] Refreshed chunk sizes: {} areas",
            self.cached_visualization_granularities
                .lock()
                .unwrap()
                .len()
        );
    }
}

impl Drop for BurstLoopRunner {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Aggregate fire queue data into visualization chunks for large-area rendering
///
/// This function aggregates neuron firing data into coarser spatial chunks to reduce
/// message size for very large cortical areas (>1M neurons). Each chunk represents
/// a spatial region and contains aggregated activity (average potential, count).
///
/// # Arguments
///
/// * `neuron_ids` - Neuron IDs that fired
/// * `coords_x`, `coords_y`, `coords_z` - Neuron coordinates
/// * `potentials` - Membrane potentials
/// * `granularity` - Visualization voxel granularity dimensions (x, y, z)
///
/// # Returns
///
/// Aggregated data: (chunk_coords_x, chunk_coords_y, chunk_coords_z, chunk_potentials, chunk_counts)
type AggregatedVisualizationChunks = (Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>, Vec<u32>);

fn aggregate_into_visualization_chunks(
    neuron_ids: &[u32],
    coords_x: &[u32],
    coords_y: &[u32],
    coords_z: &[u32],
    potentials: &[f32],
    granularity: (u32, u32, u32),
) -> AggregatedVisualizationChunks {
    let (chunk_x, chunk_y, chunk_z) = granularity;

    // Use HashMap to aggregate chunks: chunk_coord -> (sum_potential, count)
    let mut chunk_map: ahash::AHashMap<(u32, u32, u32), (f32, u32)> = ahash::AHashMap::new();

    for i in 0..neuron_ids.len() {
        let x = coords_x[i];
        let y = coords_y[i];
        let z = coords_z[i];
        let p = potentials[i];

        // Calculate chunk coordinates
        let chunk_x_coord = x / chunk_x;
        let chunk_y_coord = y / chunk_y;
        let chunk_z_coord = z / chunk_z;

        let chunk_key = (chunk_x_coord, chunk_y_coord, chunk_z_coord);

        // Aggregate: sum potentials and count neurons
        let entry = chunk_map.entry(chunk_key).or_insert((0.0, 0));
        entry.0 += p;
        entry.1 += 1;
    }

    // Convert aggregated chunks to vectors
    let mut chunk_coords_x = Vec::with_capacity(chunk_map.len());
    let mut chunk_coords_y = Vec::with_capacity(chunk_map.len());
    let mut chunk_coords_z = Vec::with_capacity(chunk_map.len());
    let mut chunk_potentials = Vec::with_capacity(chunk_map.len());
    let mut chunk_counts = Vec::with_capacity(chunk_map.len());

    for ((cx, cy, cz), (sum_p, count)) in chunk_map {
        // Store chunk center coordinates (middle of chunk)
        chunk_coords_x.push(cx * chunk_x + chunk_x / 2);
        chunk_coords_y.push(cy * chunk_y + chunk_y / 2);
        chunk_coords_z.push(cz * chunk_z + chunk_z / 2);

        // Average potential (sum / count)
        chunk_potentials.push(sum_p / count as f32);
        chunk_counts.push(count);
    }

    (
        chunk_coords_x,
        chunk_coords_y,
        chunk_coords_z,
        chunk_potentials,
        chunk_counts,
    )
}

/// Helper function to encode fire queue data to XYZP format
/// Used by both visualization and motor output streams
///
/// 🚀 ZERO-COPY OPTIMIZATION: Takes ownership of fire_data to move (not clone) vectors
/// This eliminates ~1 MB allocation per burst @ 10 Hz = ~10 MB/sec saved
///
/// Filter by cortical_id strings (e.g., "omot00"), matching sensory stream pattern
fn encode_fire_data_to_xyzp(
    fire_data: RawFireQueueSnapshot,
    cortical_id_filter: Option<&ahash::AHashSet<String>>,
) -> Result<Vec<u8>, String> {
    use feagi_structures::genomic::cortical_area::CorticalID;
    use feagi_structures::neuron_voxels::xyzp::{
        CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
    };

    let mut cortical_mapped = CorticalMappedXYZPNeuronVoxels::new();

    for (area_id, area_data) in fire_data {
        let x_vec = area_data.coords_x;
        let y_vec = area_data.coords_y;
        let z_vec = area_data.coords_z;
        let p_vec = area_data.potentials;

        // Skip empty areas or areas with mismatched vector lengths
        if x_vec.is_empty() || y_vec.is_empty() || z_vec.is_empty() || p_vec.is_empty() {
            continue;
        }

        // Sanity check: all vectors should have the same length
        if x_vec.len() != y_vec.len() || x_vec.len() != z_vec.len() || x_vec.len() != p_vec.len() {
            error!(
                "[ENCODE-XYZP] ❌ Vector length mismatch in area {}: x={}, y={}, z={}, p={}",
                area_id,
                x_vec.len(),
                y_vec.len(),
                z_vec.len(),
                p_vec.len()
            );
            continue;
        }

        // Apply cortical_id filter if specified (for motor subscriptions)
        if let Some(filter) = cortical_id_filter {
            debug!(
                "[ENCODE-XYZP] 🎮 Checking area '{}' (bytes: {:02x?}) against filter: {:?}",
                area_data.cortical_id.escape_debug(),
                area_data.cortical_id.as_bytes(),
                filter
                    .iter()
                    .map(|s| format!("{} ({:02x?})", s.escape_debug(), s.as_bytes()))
                    .collect::<Vec<_>>()
            );
            if !filter.contains(&area_data.cortical_id) {
                debug!(
                    "[ENCODE-XYZP] ❌ Area '{}' NOT in filter - skipping",
                    area_data.cortical_id.escape_debug()
                );
                continue; // Skip - not in agent's motor subscriptions
            }
            debug!(
                "[ENCODE-XYZP] ✅ Area '{}' IS in filter - including",
                area_data.cortical_id.escape_debug()
            );
        }

        // Create CorticalID from base64-encoded area name
        let cortical_id = match CorticalID::try_from_base_64(&area_data.cortical_id) {
            Ok(id) => id,
            Err(e) => {
                error!(
                    "[ENCODE-XYZP] ❌ Failed to decode CorticalID from base64 '{}': {:?}",
                    area_data.cortical_id, e
                );
                continue;
            }
        };

        // CRITICAL: Final safety check - ensure we have actual data
        let vec_len = x_vec.len();
        if vec_len == 0 {
            error!(
                "[ENCODE-XYZP] ❌ CRITICAL: Vectors have zero length after all checks! area_id={}",
                area_id
            );
            continue;
        }

        // Create neuron voxel arrays (MOVE vectors for zero-copy)
        match NeuronVoxelXYZPArrays::new_from_vectors(
            x_vec, // ✅ MOVE (no clone)
            y_vec, // ✅ MOVE (no clone)
            z_vec, // ✅ MOVE (no clone)
            p_vec, // ✅ MOVE (no clone)
        ) {
            Ok(arrays) => {
                debug!(
                    "[ENCODE-XYZP] ✅ Created arrays for area {} with {} neurons",
                    area_id, vec_len
                );
                cortical_mapped.mappings.insert(cortical_id, arrays);
            }
            Err(e) => {
                error!(
                    "[ENCODE-XYZP] ❌ Failed to create arrays for area {}: {:?}",
                    area_id, e
                );
                continue;
            }
        }
    }

    // Check if we have any data to send
    if cortical_mapped.mappings.is_empty() {
        // No neurons fired in any subscribed area - return empty buffer
        return Ok(Vec::new());
    }

    // Serialize to FeagiByteContainer (version 2 container format)
    // This ensures proper container wrapping with global header, structure lookup, etc.
    // Note: overwrite_byte_data_with_single_struct_data() already handles efficient allocation internally:
    // - It pre-calculates size via get_number_of_bytes_needed()
    // - Only resizes if current capacity is insufficient
    // - Reuses existing allocation when possible
    use feagi_serialization::FeagiByteContainer;

    let mut byte_container = FeagiByteContainer::new_empty();
    byte_container
        .overwrite_byte_data_with_single_struct_data(&cortical_mapped, 0)
        .map_err(|e| format!("Failed to encode into FeagiByteContainer: {:?}", e))?;

    // Extract bytes from container
    let buffer = byte_container.get_byte_ref().to_vec();

    debug!(
        "[ENCODE-XYZP] ✅ Encoded {} cortical areas into FeagiByteContainer: {} bytes",
        cortical_mapped.mappings.len(),
        buffer.len()
    );

    Ok(buffer)
}

/// Helper to get timestamp string with millisecond precision
fn get_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    let secs = since_epoch.as_secs();
    let millis = since_epoch.subsec_millis();
    use chrono::{DateTime, TimeZone, Utc};
    let dt: DateTime<Utc> = Utc.timestamp_opt(secs as i64, millis * 1_000_000).unwrap();
    dt.format("%Y-%m-%dT%H:%M:%S%.3f").to_string()
}

/// Decode sensory bytes (FeagiByteContainer) into cortical XYZP list.
/// Transport-agnostic; same format whether source is ZMQ, WebSocket, or SHM.
fn decode_sensory_bytes(bytes: &[u8]) -> Result<SensoryXyzpDecoded, String> {
    use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

    let mut byte_container = feagi_serialization::FeagiByteContainer::new_empty();
    let mut data_vec = bytes.to_vec();
    byte_container
        .try_write_data_to_container_and_verify(&mut |container_bytes| {
            std::mem::swap(container_bytes, &mut data_vec);
            Ok(())
        })
        .map_err(|e| format!("FeagiByteContainer load failed: {:?}", e))?;

    let num_structures = byte_container
        .try_get_number_contained_structures()
        .map_err(|e| format!("get_number_contained_structures failed: {:?}", e))?;

    let mut out = Vec::new();
    for struct_idx in 0..num_structures {
        let boxed_struct = byte_container
            .try_create_new_struct_from_index(struct_idx as u8)
            .map_err(|e| {
                format!(
                    "create_new_struct_from_index {} failed: {:?}",
                    struct_idx, e
                )
            })?;

        let cortical_mapped = match boxed_struct
            .as_any()
            .downcast_ref::<CorticalMappedXYZPNeuronVoxels>()
        {
            Some(cm) => cm,
            None => {
                trace!(
                    "[SENSORY-DECODE] Structure {} is not CorticalMappedXYZPNeuronVoxels, skipping",
                    struct_idx
                );
                continue;
            }
        };

        for (cortical_id, neuron_arrays) in &cortical_mapped.mappings {
            let (x_coords, y_coords, z_coords, potentials) = neuron_arrays.borrow_xyzp_vectors();
            let xyzp_data: Vec<(u32, u32, u32, f32)> = x_coords
                .iter()
                .zip(y_coords.iter())
                .zip(z_coords.iter())
                .zip(potentials.iter())
                .map(|(((x, y), z), p)| (*x, *y, *z, *p))
                .collect();
            if !xyzp_data.is_empty() {
                out.push((*cortical_id, xyzp_data));
            }
        }
    }

    static FIRST_DECODE_LOGGED: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);
    if !out.is_empty() && !FIRST_DECODE_LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        let total: usize = out.iter().map(|(_, xyzp)| xyzp.len()).sum();
        info!(
            "[SENSORY-DECODE] First decode: {} areas, {} total neurons",
            out.len(),
            total
        );
        for (cortical_id, xyzp) in &out {
            info!(
                "[SENSORY-DECODE]   area base64={} neurons={}",
                cortical_id.as_base_64(),
                xyzp.len()
            );
        }
    }

    Ok(out)
}

/// Main burst processing loop (runs in dedicated thread)
///
/// This is the HOT PATH - zero Python involvement!
/// Power neurons are read directly from RustNPU's internal state.
/// Burst count is tracked by NPU - single source of truth!
#[allow(clippy::too_many_arguments)]
fn burst_loop(
    npu: Arc<TracingMutex<DynamicNPU>>,
    frequency_hz: Arc<Mutex<f64>>, // Shared frequency - can be updated while running
    running: Arc<AtomicBool>,
    viz_shm_writer: Arc<Mutex<Option<crate::viz_shm_writer::VizSHMWriter>>>,
    motor_shm_writer: Arc<Mutex<Option<crate::motor_shm_writer::MotorSHMWriter>>>,
    viz_publisher: Option<Arc<dyn VisualizationPublisher>>, // Trait object for visualization (NO PYTHON CALLBACKS!)
    motor_publisher: Option<Arc<dyn MotorPublisher>>, // Trait object for motor (NO PYTHON CALLBACKS!)
    motor_subscriptions: Arc<ParkingLotRwLock<ahash::AHashMap<String, ahash::AHashSet<String>>>>,
    motor_output_rates_hz: Arc<ParkingLotRwLock<ahash::AHashMap<String, f64>>>,
    motor_last_publish_time: Arc<ParkingLotRwLock<ahash::AHashMap<String, Instant>>>,
    visualization_subscriptions: Arc<ParkingLotRwLock<ahash::AHashSet<String>>>,
    visualization_output_rates_hz: Arc<ParkingLotRwLock<ahash::AHashMap<String, f64>>>,
    visualization_last_publish_time: Arc<ParkingLotRwLock<ahash::AHashMap<String, Instant>>>,
    cached_burst_count: Arc<std::sync::atomic::AtomicU64>, // For lock-free burst count reads
    cached_fire_queue: Arc<Mutex<Option<Arc<FireQueueSample>>>>, // For caching fire queue data (Arc-wrapped to avoid cloning)
    parameter_queue: ParameterUpdateQueue, // Asynchronous parameter update queue
    plasticity_notify: Option<Arc<dyn Fn(u64) + Send + Sync>>, // Plasticity notification callback
    post_burst_callback: Option<Arc<dyn Fn(u64) + Send + Sync>>, // Post-burst callback
    cached_cortical_id_mappings: Arc<Mutex<ahash::AHashMap<u32, String>>>, // Cached cortical_idx -> cortical_id
    _last_cortical_id_refresh: Arc<Mutex<u64>>, // Burst count when mappings were last refreshed
    cached_visualization_granularities: Arc<Mutex<VisualizationGranularityCache>>, // Cached cortical_idx -> visualization_granularity
    sensory_intake: Option<Arc<Mutex<dyn SensoryIntake>>>, // Transport-agnostic (feagi-io)
) {
    let timestamp = get_timestamp();
    let initial_freq = *frequency_hz.lock().unwrap();
    update_sim_timestep_from_hz(initial_freq);
    info!(
        "[{}] [BURST-LOOP] Starting main loop at {:.2} Hz",
        timestamp, initial_freq
    );

    trace!(
        "[BURST-LOOP] Entering while loop with running={}",
        running.load(Ordering::Acquire)
    );

    let mut burst_num = 0u64;
    let mut last_stats_time = Instant::now();
    let mut total_neurons_fired = 0usize;
    let mut burst_times = Vec::with_capacity(100);
    let mut last_burst_time = None;
    // Tracks agents that temporarily fail publish because transport mapping is not attached yet.
    // This avoids log spam during reconnect races while preserving automatic retry behavior.
    let mut missing_viz_agent_logged: ahash::AHashSet<String> = ahash::AHashSet::new();
    let mut missing_motor_agent_logged: ahash::AHashSet<String> = ahash::AHashSet::new();

    while running.load(Ordering::Acquire) {
        let iteration_start = Instant::now();
        let burst_start = Instant::now();
        // Keep simulation timestep snapshot aligned with runtime frequency.
        // This is used by injection warnings (warn if injection exceeds timestep).
        let current_frequency_hz = *frequency_hz.lock().unwrap();
        update_sim_timestep_from_hz(current_frequency_hz);

        // DIAGNOSTIC: Log that we're alive
        if burst_num % 100 == 0 {
            trace!("[BURST-LOOP] Burst {} starting (loop is alive)", burst_num);
        }

        // Track time since last burst (to detect blocking)
        static LAST_ITERATION_END: std::sync::Mutex<Option<Instant>> = std::sync::Mutex::new(None);
        {
            let mut last_end = LAST_ITERATION_END.lock().unwrap();
            if let Some(last) = *last_end {
                let gap = iteration_start.duration_since(last);
                // Only warn for extreme gaps (>10 seconds) that might indicate system issues
                // Normal batch processing (e.g., MRI data) can have multi-second gaps
                if gap.as_millis() > 10000 {
                    warn!(
                        "[BURST-LOOP] ⚠️ Extremely large gap between bursts: {:.2}ms - burst {} (this may indicate system issues)",
                        gap.as_secs_f64() * 1000.0,
                        burst_num
                    );
                }
            }
            *last_end = Some(iteration_start);
        }

        // Track actual burst interval
        if let Some(last) = last_burst_time {
            let interval = burst_start.duration_since(last);
            burst_times.push(interval);
            if burst_times.len() > 100 {
                burst_times.remove(0);
            }
        }
        last_burst_time = Some(burst_start);

        // CRITICAL: Check shutdown flag immediately after burst processing
        // This ensures we exit as soon as shutdown is requested, even mid-burst
        if !running.load(Ordering::Relaxed) {
            break;
        }

        // Process burst (THE HOT PATH!)
        // 🔋 Power neurons auto-discovered from neuron array - 100% Rust!
        // CRITICAL: Check shutdown flag before acquiring expensive locks
        if !running.load(Ordering::Relaxed) {
            break;
        }

        // Poll transport-agnostic sensory intake (feagi-io) before acquiring NPU lock
        let sensory_xyzp: Option<SensoryXyzpDecoded> = sensory_intake.as_ref().and_then(|intake| {
            let mut guard = intake.lock().ok()?;
            let bytes = guard.poll_sensory_data().ok().flatten()?;
            match decode_sensory_bytes(&bytes) {
                Ok(decoded) => Some(decoded),
                Err(e) => {
                    warn!(
                        "[SENSORY-DECODE] Failed to decode {} bytes: {}",
                        bytes.len(),
                        e
                    );
                    None
                }
            }
        });

        // Track time since last lock release to detect if something held it
        static LAST_LOCK_RELEASE: std::sync::Mutex<Option<Instant>> = std::sync::Mutex::new(None);
        #[allow(dead_code)]
        static LAST_BURST_END: std::sync::Mutex<Option<Instant>> = std::sync::Mutex::new(None);
        static POST_BURST_MISSING_LOGGED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);

        let lock_start = Instant::now();
        if let Ok(last_release) = LAST_LOCK_RELEASE.lock() {
            if let Some(last) = *last_release {
                let gap = lock_start.duration_since(last);
                // Only warn for extreme gaps (>30 seconds) that might indicate deadlock
                // Batch processing (e.g., medical imaging) can hold lock for several seconds legitimately
                if gap.as_millis() > 30000 {
                    warn!(
                        "[NPU-LOCK] Burst {}: Extreme gap since last release: {:.2}ms (possible deadlock or system issue)",
                        burst_num,
                        gap.as_secs_f64() * 1000.0
                    );
                }
            }
        }

        if burst_num < 5 || burst_num % 100 == 0 {
            trace!(
                "[BURST-LOOP-DIAGNOSTIC] Burst {}: Attempting NPU lock...",
                burst_num
            );
        }

        let mut last_process_duration: Option<std::time::Duration> = None;
        let mut last_burst_stats: Option<(usize, usize, usize, usize, usize)> = None;

        // Track lock acquisition time outside block scope for diagnostics
        let lock_acquired = {
            // Log lock attempt with timestamp for correlation
            if lock_start.elapsed().as_millis() == 0 {
                debug!(
                    "[NPU-LOCK] Burst {}: Attempting lock acquisition at {:?}",
                    burst_num, lock_start
                );
            }

            // Check if something else is holding the lock
            let acquisition_start = lock_start;
            let current_thread_id = std::thread::current().id();
            let mut npu_lock = npu.lock().unwrap();
            let acquired = Instant::now();
            let lock_wait_duration = acquired.duration_since(lock_start);

            // Log if lock acquisition took significant time (could indicate contention)
            // Only warn for extreme lock wait times (>30 seconds) - batch processing legitimately holds lock for seconds
            if lock_wait_duration.as_millis() > 30000 {
                // Check if we can see what might have been holding it
                if let Ok(last_release) = LAST_LOCK_RELEASE.lock() {
                    if let Some(last) = *last_release {
                        let time_since_release = acquisition_start.duration_since(last);
                        warn!(
                            "[NPU-LOCK] Extreme lock wait: {:.2}ms wait (burst {}, thread={:?}) | Time since last release: {:.2}ms (possible deadlock)",
                            lock_wait_duration.as_secs_f64() * 1000.0,
                            burst_num,
                            current_thread_id,
                            time_since_release.as_secs_f64() * 1000.0
                        );
                    } else {
                        warn!(
                            "[NPU-LOCK] Extreme lock wait: {:.2}ms (burst {}, thread={:?}) - possible deadlock!",
                            lock_wait_duration.as_secs_f64() * 1000.0,
                            burst_num,
                            current_thread_id
                        );
                    }
                }
                debug!(
                    "[NPU-LOCK] Lock acquired by burst loop thread {:?} after {}ms wait",
                    current_thread_id,
                    lock_wait_duration.as_millis()
                );
            }
            if burst_num < 5 || burst_num % 100 == 0 {
                trace!(
                    "[BURST-TIMING] Burst {}: NPU lock acquired in {:?}",
                    burst_num,
                    lock_wait_duration
                );
            }

            // Check flag again after acquiring lock (in case shutdown happened during lock wait)
            let mut burst_after = npu_lock.get_burst_count();
            let should_exit = if !running.load(Ordering::Relaxed) {
                true // Signal to exit
            } else {
                // APPLY QUEUED PARAMETER UPDATES (before burst processing)
                // This ensures updates take effect immediately in this burst
                let pending_updates = parameter_queue.drain_all();
                if !pending_updates.is_empty() {
                    let update_start = Instant::now();
                    let mut applied_count = 0;

                    info!(
                        "[PARAM-QUEUE] Processing {} queued updates",
                        pending_updates.len()
                    );

                    for update in pending_updates {
                        let count = match update.parameter_name.as_str() {
                            "neuron_fire_threshold" | "firing_threshold" => {
                                if let Some(threshold) = update.value.as_f64() {
                                    npu_lock.update_cortical_area_threshold(
                                        update.cortical_idx,
                                        threshold as f32,
                                    )
                                } else {
                                    0
                                }
                            }
                            // Spatial gradient threshold increments - uses stored neuron positions
                            "neuron_fire_threshold_increment" | "firing_threshold_increment" => {
                                // This is sent as array [x, y, z] from BV
                                if let Some(arr) = update.value.as_array() {
                                    if arr.len() == 3 {
                                        if let (Some(inc_x), Some(inc_y), Some(inc_z)) =
                                            (arr[0].as_f64(), arr[1].as_f64(), arr[2].as_f64())
                                        {
                                            // Get base threshold from update metadata
                                            if let Some(base_threshold) = update.base_threshold {
                                                npu_lock
                                                    .update_cortical_area_threshold_with_gradient(
                                                        update.cortical_idx,
                                                        base_threshold,
                                                        inc_x as f32,
                                                        inc_y as f32,
                                                        inc_z as f32,
                                                    )
                                            } else {
                                                warn!(
                                                    "[PARAM-QUEUE] Spatial gradient update missing base_threshold - skipping"
                                                );
                                                0
                                            }
                                        } else {
                                            0
                                        }
                                    } else {
                                        0
                                    }
                                } else {
                                    0
                                }
                            }
                            // IMPORTANT: firing_threshold_limit is NOT the firing threshold.
                            // Previously this was (incorrectly) routed into update_cortical_area_threshold(),
                            // which could set threshold=0 and make downstream neurons fire trivially.
                            "neuron_firing_threshold_limit" | "firing_threshold_limit" => {
                                if let Some(limit) = update.value.as_f64() {
                                    npu_lock.update_cortical_area_threshold_limit(
                                        update.cortical_idx,
                                        limit as f32,
                                    )
                                } else {
                                    0
                                }
                            }
                            "neuron_refractory_period" | "refractory_period" | "refrac" => {
                                if let Some(period) = update.value.as_u64() {
                                    npu_lock.update_cortical_area_refractory_period(
                                        update.cortical_idx,
                                        period as u16,
                                    )
                                } else {
                                    0
                                }
                            }
                            "leak" | "leak_coefficient" | "neuron_leak_coefficient" => {
                                if let Some(leak) = update.value.as_f64() {
                                    if (0.0..=1.0).contains(&leak) {
                                        npu_lock.update_cortical_area_leak(
                                            update.cortical_idx,
                                            leak as f32,
                                        )
                                    } else {
                                        0
                                    }
                                } else {
                                    0
                                }
                            }
                            "consecutive_fire_cnt_max"
                            | "neuron_consecutive_fire_count"
                            | "consecutive_fire_count" => {
                                if let Some(limit) = update.value.as_u64() {
                                    npu_lock.update_cortical_area_consecutive_fire_limit(
                                        update.cortical_idx,
                                        limit as u16,
                                    )
                                } else {
                                    0
                                }
                            }
                            "snooze_length" | "neuron_snooze_period" | "snooze_period" => {
                                if let Some(snooze) = update.value.as_u64() {
                                    npu_lock.update_cortical_area_snooze_period(
                                        update.cortical_idx,
                                        snooze as u16,
                                    )
                                } else {
                                    0
                                }
                            }
                            "neuron_excitability" => {
                                if let Some(excitability) = update.value.as_f64() {
                                    if (0.0..=1.0).contains(&excitability) {
                                        npu_lock.update_cortical_area_excitability(
                                            update.cortical_idx,
                                            excitability as f32,
                                        )
                                    } else {
                                        0
                                    }
                                } else {
                                    0
                                }
                            }
                            "neuron_mp_charge_accumulation" | "mp_charge_accumulation" => {
                                if let Some(accumulation) = update.value.as_bool() {
                                    npu_lock.update_cortical_area_mp_charge_accumulation(
                                        update.cortical_idx,
                                        accumulation,
                                    )
                                } else {
                                    0
                                }
                            }
                            "postsynaptic_current" | "neuron_post_synaptic_potential" => {
                                if let Some(psp) = update.value.as_f64() {
                                    // PSP is stored in the NPU as u8 (0..=255).
                                    // Clamp deterministically (matches synaptogenesis behavior).
                                    let psp_u8 = psp.clamp(0.0, 255.0) as u8;
                                    let synapses_updated = npu_lock
                                        .update_cortical_area_postsynaptic_current(
                                            update.cortical_idx,
                                            psp_u8,
                                        );
                                    let mappings_updated = npu_lock
                                        .update_stdp_mapping_psp_for_source(
                                            update.cortical_idx,
                                            psp_u8,
                                        );
                                    info!(
                                        target: "feagi-burst-engine",
                                        "Applied PSP update area={} psp={} synapses_updated={} stdp_mappings_updated={}",
                                        update.cortical_id,
                                        psp_u8,
                                        synapses_updated,
                                        mappings_updated
                                    );
                                    synapses_updated
                                } else {
                                    0
                                }
                            }
                            "mp_driven_psp" | "neuron_mp_driven_psp" => {
                                if let Some(enabled) = update.value.as_bool() {
                                    match feagi_structures::genomic::cortical_area::CorticalID::try_from_base_64(
                                        &update.cortical_id,
                                    ) {
                                        Ok(cortical_id) => {
                                            npu_lock.set_mp_driven_psp_flag(cortical_id, enabled);
                                            1
                                        }
                                        Err(_) => 0,
                                    }
                                } else {
                                    0
                                }
                            }
                            "psp_uniform_distribution" | "neuron_psp_uniform_distribution" => {
                                if let Some(enabled) = update.value.as_bool() {
                                    match feagi_structures::genomic::cortical_area::CorticalID::try_from_base_64(
                                        &update.cortical_id,
                                    ) {
                                        Ok(cortical_id) => {
                                            npu_lock.set_psp_uniform_distribution_flag(
                                                cortical_id,
                                                enabled,
                                            );
                                            1
                                        }
                                        Err(_) => 0,
                                    }
                                } else {
                                    0
                                }
                            }
                            _ => 0,
                        };

                        if count > 0 {
                            applied_count += 1;
                            if update.parameter_name == "postsynaptic_current"
                                || update.parameter_name == "neuron_post_synaptic_potential"
                            {
                                debug!(
                                    "[PARAM-QUEUE] Applied {}={} to {} synapses in area {}",
                                    update.parameter_name, update.value, count, update.cortical_id
                                );
                            } else {
                                debug!(
                                    "[PARAM-QUEUE] Applied {}={} to {} neurons in area {}",
                                    update.parameter_name, update.value, count, update.cortical_id
                                );
                            }
                        }
                    }

                    if applied_count > 0 {
                        let update_duration = update_start.elapsed();
                        info!(
                            "[PARAM-QUEUE] ✓ Applied {} parameter updates in {:?}",
                            applied_count, update_duration
                        );
                    }
                }

                // Inject sensory from intake (any transport) into NPU for this burst.
                // Clear pending first so only the latest frame is applied (avoids accumulation).
                if let Some(ref list) = sensory_xyzp {
                    npu_lock.clear_pending_sensory_injections();
                    for (cortical_id, xyzp) in list {
                        npu_lock.inject_sensory_xyzp_by_id(cortical_id, xyzp);
                    }
                }

                let process_start = Instant::now();
                debug!("[BURST-TIMING] Starting process_burst()...");

                let burst_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    npu_lock.process_burst()
                }));

                match burst_result {
                    Ok(Ok(mut result)) => {
                        let process_done = Instant::now();
                        let duration = process_done.duration_since(process_start);
                        last_process_duration = Some(duration);
                        last_burst_stats = Some((
                            result.neuron_count,
                            result.power_injections,
                            result.synaptic_injections,
                            result.neurons_processed,
                            result.neurons_in_refractory,
                        ));

                        if burst_num < 5 || burst_num % 100 == 0 {
                            trace!(
                                "[BURST-TIMING] Burst {}: process_burst() completed in {:?}, {} neurons fired",
                                burst_num,
                                duration,
                                result.neuron_count
                            );
                        }

                        total_neurons_fired += result.neuron_count;
                        // Update cached burst count for lock-free reads
                        let current_burst = npu_lock.get_burst_count();
                        cached_burst_count
                            .store(current_burst, std::sync::atomic::Ordering::Relaxed);

                        // Notify plasticity service of completed burst (while NPU lock still held)
                        // This allows plasticity service to immediately query FireLedger data
                        // Callback is pre-cloned Arc, so this is just a function call (no allocation)
                        if let Some(ref notify_fn) = plasticity_notify {
                            trace!(
                                "[BURST-LOOP] 🧠 Notifying plasticity service of burst {}",
                                current_burst
                            );
                            notify_fn(current_burst);
                        }

                        // CRITICAL PERFORMANCE FIX: Cache fire queue sample from process_burst() result
                        // This avoids needing to acquire NPU lock again (was causing 2-5 second delays!)
                        // The sample is built inside process_burst() while the lock is already held
                        // Store in Arc to avoid cloning when sharing between viz and motor
                        let fq_sample = result.fire_queue_sample.take(); // Move out to avoid clone
                        if fq_sample.is_some() {
                        } else {
                            trace!("[BURST-LOOP] 📸 Fire queue sample is None (no neurons fired this burst)");
                        }
                        // Store as Arc to share without cloning
                        let cache_store_start = Instant::now();
                        *cached_fire_queue.lock().unwrap() = fq_sample.map(Arc::new);
                        let cache_store_duration = cache_store_start.elapsed();
                        if cache_store_duration.as_millis() > 5 {
                            warn!(
                                "[BURST-LOOP] ⚠️ Slow fire queue cache store: {:.2}ms (burst {})",
                                cache_store_duration.as_secs_f64() * 1000.0,
                                burst_num
                            );
                        }

                        burst_after = current_burst;
                        false // Continue processing
                    }
                    Ok(Err(e)) => {
                        let timestamp = get_timestamp();
                        error!(
                            "[{}] [BURST-LOOP] ❌ Burst processing error: {}",
                            timestamp, e
                        );
                        burst_after = npu_lock.get_burst_count();
                        false // Continue despite error
                    }
                    Err(panic_payload) => {
                        error!(
                            "[BURST-LOOP] ❌ process_burst() panicked; rethrowing to preserve crash"
                        );
                        std::panic::resume_unwind(panic_payload);
                    }
                }
            };

            // Return should_exit, lock_acquired time, and burst count
            (should_exit, acquired, burst_after)
        };

        let (should_exit, lock_acquired, burst_after) = lock_acquired;
        let lock_wait_duration = lock_acquired.duration_since(lock_start);
        let npu_lock_release_time = Instant::now();
        let release_thread_id = std::thread::current().id();

        // Update last lock release time
        if let Ok(mut last_release) = LAST_LOCK_RELEASE.lock() {
            *last_release = Some(npu_lock_release_time);
        }

        // Log lock release timing for diagnostics
        let lock_hold_duration = npu_lock_release_time.duration_since(lock_acquired);
        if lock_wait_duration.as_millis() > 50 {
            warn!(
                "[NPU-LOCK] Burst {} waited {:.2}ms to acquire lock",
                burst_num,
                lock_wait_duration.as_secs_f64() * 1000.0
            );
        }
        if lock_hold_duration.as_millis() > 50 {
            if let Some((fired, power, synaptic, processed, refractory)) = last_burst_stats {
                warn!(
                    "[NPU-LOCK] Burst {} held lock {:.2}ms | process_burst {:.2}ms | fired={} power_inj={} syn_inj={} processed={} refractory={}",
                    burst_num,
                    lock_hold_duration.as_secs_f64() * 1000.0,
                    last_process_duration
                        .map(|d| d.as_secs_f64() * 1000.0)
                        .unwrap_or(0.0),
                    fired,
                    power,
                    synaptic,
                    processed,
                    refractory
                );
            } else {
                warn!(
                    "[NPU-LOCK] Burst {} held lock {:.2}ms | process_burst {:.2}ms",
                    burst_num,
                    lock_hold_duration.as_secs_f64() * 1000.0,
                    last_process_duration
                        .map(|d| d.as_secs_f64() * 1000.0)
                        .unwrap_or(0.0)
                );
            }
        }
        if lock_hold_duration.as_millis() > 5 || burst_num < 5 || burst_num % 100 == 0 {
            debug!(
                "[NPU-LOCK] Burst {} (thread={:?}): Lock RELEASED (held for {:.2}ms, total from acquisition: {:.2}ms)",
                burst_num,
                release_thread_id,
                lock_hold_duration.as_secs_f64() * 1000.0,
                npu_lock_release_time.duration_since(lock_start).as_secs_f64() * 1000.0
            );
        }

        if let Some(ref callback) = post_burst_callback {
            callback(burst_after);
        } else if !POST_BURST_MISSING_LOGGED.swap(true, Ordering::Relaxed) {
            tracing::debug!("[BURST-LOOP] Post-burst callback not configured");
        }

        // Exit if shutdown was requested
        if should_exit || !running.load(Ordering::Relaxed) {
            break;
        }

        burst_num += 1;
        // Note: NPU.process_burst() already incremented its internal burst_count

        let post_burst_start = Instant::now();
        let time_between_npu_release_and_post_burst =
            post_burst_start.duration_since(npu_lock_release_time);
        if time_between_npu_release_and_post_burst.as_millis() > 10 {
            warn!(
                "[BURST-LOOP] ⚠️ Slow gap between NPU release and post-burst: {:.2}ms (burst {})",
                time_between_npu_release_and_post_burst.as_secs_f64() * 1000.0,
                burst_num
            );
        }
        // Write visualization data (SHM and/or PNS ZMQ)
        // Check if we need to do ANY visualization (SHM writer OR viz publisher)
        let has_shm_writer = viz_shm_writer.lock().unwrap().is_some();
        let has_viz_publisher = viz_publisher.is_some();

        // Log visualization transport availability (debug info only)
        static DEBUG_LOGGED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        if !DEBUG_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
            debug!(
                "[BURST-LOOP] Visualization transports: SHM writer={}, publisher={}",
                has_shm_writer, has_viz_publisher
            );
            DEBUG_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // CRITICAL FIX: Check throttle BEFORE sampling/encoding (avoid wasted work!)
        // Only sample and encode when we're actually going to publish
        let now = Instant::now();
        let mut viz_due_agents: Vec<String> = Vec::new();
        if has_viz_publisher {
            let subs = visualization_subscriptions.read();
            if !subs.is_empty() {
                let rates = visualization_output_rates_hz.read();
                let last_publish = visualization_last_publish_time.read();
                let burst_hz = *frequency_hz.lock().unwrap();

                for agent_id in subs.iter() {
                    let rate_hz = rates.get(agent_id).copied().unwrap_or(burst_hz);
                    if rate_hz <= 0.0 {
                        warn!(
                            "[BURST-LOOP] Visualization: skipping agent '{}' due to invalid rate {}Hz",
                            agent_id, rate_hz
                        );
                        continue;
                    }

                    let interval = Duration::from_secs_f64(1.0 / rate_hz);
                    if let Some(last_sent) = last_publish.get(agent_id).copied() {
                        if now.duration_since(last_sent) < interval {
                            continue;
                        }
                    }

                    viz_due_agents.push(agent_id.clone());
                }
            }
        }

        let should_publish_viz = has_viz_publisher && !viz_due_agents.is_empty();

        // Sample fire queue ONCE and share between viz and motor using Arc (zero-cost sharing!)
        let has_motor_publisher = motor_publisher.is_some();
        let has_motor_shm = motor_shm_writer.lock().unwrap().is_some();
        let has_motor_subscriptions = !motor_subscriptions.read().is_empty();
        let needs_motor = has_motor_shm || (has_motor_publisher && has_motor_subscriptions);
        let needs_fire_data = has_shm_writer || should_publish_viz || needs_motor;

        if burst_num % 100 == 0 {
            trace!(
                "[BURST-LOOP] Sampling conditions: needs_fire_data={} (shm={}, viz={}, motor={})",
                needs_fire_data,
                has_shm_writer,
                should_publish_viz,
                needs_motor
            );
        }

        // CRITICAL PERFORMANCE FIX: Use fire queue sample from process_burst() result
        // This avoids acquiring NPU lock again (was causing 2-5 second delays with 5.7M neurons!)
        // The sample is already built inside process_burst() while the lock is held
        let shared_fire_data_opt = if needs_fire_data {
            // Get fire queue sample from the last process_burst() result
            // This is stored in cached_fire_queue which was set right after process_burst()
            // CRITICAL PERFORMANCE: Already stored as Arc, so we can clone the Arc (cheap) instead of the data
            let sample_start = Instant::now();
            let fire_data_arc_opt = cached_fire_queue.lock().unwrap().clone(); // Clone Arc, not data!
            let sample_duration = sample_start.elapsed();
            // Lower threshold to catch smaller slowdowns (5ms instead of 10ms)
            if sample_duration.as_millis() > 5 {
                warn!(
                    "[BURST-LOOP] ⚠️ Slow fire queue cache access: {:.2}ms (burst {})",
                    sample_duration.as_secs_f64() * 1000.0,
                    burst_num
                );
            }
            debug!(
                "[BURST-TIMING] Fire queue sample retrieved from cache in {:?}",
                sample_duration
            );

            if burst_num % 100 == 0 {
                trace!(
                    "[BURST-LOOP] Fire queue sample result: has_data={}",
                    fire_data_arc_opt.is_some()
                );
                if let Some(ref data) = fire_data_arc_opt {
                    trace!(
                        "[BURST-LOOP] Fire data contains {} cortical areas",
                        data.len()
                    );
                }
            }

            static FIRST_CHECK_LOGGED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !FIRST_CHECK_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                if has_shm_writer {
                    debug!("[BURST-LOOP] 🔍 Viz SHM writer is attached");
                }
                if has_viz_publisher {
                    debug!("[BURST-LOOP] 🔍 Visualization publisher is attached (Rust-to-Rust, NO PYTHON!)");
                }
                FIRST_CHECK_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
            }

            if let Some(ref fire_data_arc) = fire_data_arc_opt {
                // Convert to RawFireQueueSnapshot for PNS (using Arc for zero-cost sharing)
                let viz_prep_start = Instant::now();
                let mut raw_snapshot = RawFireQueueSnapshot::new();
                let mut total_neurons = 0;

                // CRITICAL PERFORMANCE FIX: Use cached cortical_id mappings from ConnectomeManager
                // This eliminates NPU lock acquisitions that were causing 1-3s delays!
                // Area names are stored in ConnectomeManager, not NPU - we cache them here
                //
                // NOTE: Cache is refreshed externally via refresh_cortical_id_mappings() when areas are created/updated
                // For now, if cache is empty, we'll use fallback (area_{idx}) - this is acceptable as visualization
                // only needs cortical_id once, and it will be refreshed on next area creation/update

                // CRITICAL PERFORMANCE: Clone both maps to release locks immediately
                // This prevents holding locks during expensive visualization aggregation and vector cloning
                let granularities_clone = {
                    let granularities = cached_visualization_granularities.lock().unwrap();
                    if granularities.is_empty() {
                        None
                    } else {
                        Some(granularities.clone())
                    }
                };

                // CRITICAL PERFORMANCE: Clone cortical_id mappings to release lock immediately
                // This prevents lock contention when ConnectomeManager tries to refresh the cache
                let cortical_id_mappings_clone = {
                    let mappings = cached_cortical_id_mappings.lock().unwrap();
                    mappings.clone()
                };

                for (area_id, (neuron_ids, coords_x, coords_y, coords_z, potentials)) in
                    fire_data_arc.iter()
                {
                    if neuron_ids.is_empty() {
                        continue;
                    }

                    // Get cortical_id from cached mappings (no NPU lock needed!)
                    // CRITICAL: For reserved areas (0=_death, 1=_power, 2=_fatigue), use CoreCorticalType
                    // even if cache is empty, so BV can identify them correctly
                    // For other areas, skip if not in cache (cache should be populated from ConnectomeManager)
                    let cortical_id = match cortical_id_mappings_clone.get(area_id) {
                        Some(id) => id.clone(),
                        None => {
                            // Fallback for reserved core areas (BV needs correct cortical_id to identify them)
                            use feagi_structures::genomic::cortical_area::CoreCorticalType;
                            match area_id {
                                0 => CoreCorticalType::Death.to_cortical_id().as_base_64(),
                                1 => CoreCorticalType::Power.to_cortical_id().as_base_64(),
                                2 => CoreCorticalType::Fatigue.to_cortical_id().as_base_64(),
                                _ => {
                                    // Skip areas not in cache (cache should be populated from ConnectomeManager)
                                    // Log warning only once per area to avoid spam
                                    static WARNED_AREAS: std::sync::LazyLock<
                                        std::sync::Mutex<ahash::AHashSet<u32>>,
                                    > = std::sync::LazyLock::new(|| {
                                        std::sync::Mutex::new(ahash::AHashSet::new())
                                    });
                                    let mut warned = WARNED_AREAS.lock().unwrap();
                                    if !warned.contains(area_id) {
                                        warn!(
                                            "[BURST-LOOP] ⚠️ Area {} not in cortical_id cache - skipping visualization. Cache should be refreshed from ConnectomeManager.",
                                            area_id
                                        );
                                        warned.insert(*area_id);
                                    }
                                    continue; // Skip this area - can't visualize without valid cortical_id
                                }
                            }
                        }
                    };

                    // Check if this area should use aggregated rendering
                    // CRITICAL PERFORMANCE: Only clone vectors when NOT using aggregated rendering (aggregated rendering creates new vectors)
                    // For areas without aggregated rendering, we must clone because we're reading from Arc (can't move)
                    // OPTIMIZATION: For small numbers of fired neurons, cloning is fast. For large numbers,
                    // aggregated rendering should be used to reduce data size.
                    let (
                        final_coords_x,
                        final_coords_y,
                        final_coords_z,
                        final_potentials,
                        final_neuron_ids,
                    ) = if let Some(ref granularities) = granularities_clone {
                        if let Some(&granularity) = granularities.get(area_id) {
                            // Apply aggregated rendering for large areas (creates new aggregated vectors)
                            // This reduces data size significantly for areas with many fired neurons
                            let (chunk_x, chunk_y, chunk_z, chunk_p, _chunk_counts) =
                                aggregate_into_visualization_chunks(
                                    neuron_ids,
                                    coords_x,
                                    coords_y,
                                    coords_z,
                                    potentials,
                                    granularity,
                                );
                            // For aggregated rendering, use chunk indices as neuron IDs (or sequential IDs)
                            let chunk_ids: Vec<u32> = (0..chunk_x.len() as u32).collect();
                            (chunk_x, chunk_y, chunk_z, chunk_p, chunk_ids)
                        } else {
                            // No aggregated rendering for this area - must clone because we're reading from Arc (can't move)
                            // NOTE: This is only expensive if many neurons fired. If only a few neurons fired,
                            // the vectors are small and cloning is fast.
                            (
                                coords_x.clone(),
                                coords_y.clone(),
                                coords_z.clone(),
                                potentials.clone(),
                                neuron_ids.clone(),
                            )
                        }
                    } else {
                        // No aggregated rendering configured at all - must clone because we're reading from Arc (can't move)
                        // NOTE: This is only expensive if many neurons fired. If only a few neurons fired,
                        // the vectors are small and cloning is fast.
                        (
                            coords_x.clone(),
                            coords_y.clone(),
                            coords_z.clone(),
                            potentials.clone(),
                            neuron_ids.clone(),
                        )
                    };

                    total_neurons += final_neuron_ids.len();

                    // Minimal memory visualization support:
                    // If this cortical_id is a MEMORY area, BV only needs the area to appear in the Type 11 stream.
                    // We emit a single point at (0,0,0) (memory areas are conceptually 1x1x1) so the client
                    // can trigger its jelly animation without requiring actual per-neuron coordinates.
                    // Detect memory areas by decoding cortical ID bytes (deterministic; no hardcoded IDs).
                    // Memory areas may be encoded as custom IDs prefixed by `cmem...`.
                    let is_memory_area =
                        feagi_structures::genomic::cortical_area::CorticalID::try_from_base_64(
                            &cortical_id,
                        )
                        .ok()
                        .is_some_and(|id| {
                            id.as_bytes().starts_with(b"cmem") || id.as_bytes()[0] == b'm'
                        });

                    // FEAGI-side diagnostics (must be easy to spot in logs):
                    // - If `cortical_id` falls back to "area_{idx}", Type11 serialization may drop the area.
                    // - If memory area is detected, we inject a single (0,0,0) point for BV.

                    // CRITICAL PERFORMANCE: Only clone vectors if needed (memory areas use small vectors)
                    // For normal areas, we must clone because we're reading from Arc (can't move)
                    // For aggregated rendering areas, we already have the aggregated data
                    raw_snapshot.insert(
                        *area_id,
                        RawFireQueueData {
                            cortical_area_idx: *area_id,
                            cortical_id,
                            neuron_ids: if is_memory_area {
                                vec![final_neuron_ids[0]]
                            } else {
                                final_neuron_ids
                            },
                            coords_x: if is_memory_area {
                                vec![0]
                            } else {
                                final_coords_x
                            },
                            coords_y: if is_memory_area {
                                vec![0]
                            } else {
                                final_coords_y
                            },
                            coords_z: if is_memory_area {
                                vec![0]
                            } else {
                                final_coords_z
                            },
                            potentials: if is_memory_area {
                                vec![1.0]
                            } else {
                                final_potentials
                            },
                        },
                    );
                }
                let viz_prep_duration = viz_prep_start.elapsed();
                // Lower threshold to catch smaller slowdowns (10ms instead of 50ms)
                if viz_prep_duration.as_millis() > 10 {
                    warn!(
                        "[BURST-LOOP] ⚠️ Slow viz data prep: {:.2}ms for {} neurons (burst {})",
                        viz_prep_duration.as_secs_f64() * 1000.0,
                        total_neurons,
                        burst_num
                    );
                }

                if total_neurons > 0 {
                    if burst_num % 100 == 0 || total_neurons > 1000 {
                        debug!(
                            "[BURST-LOOP] 🔍 Sampled {} neurons from {} areas for viz",
                            total_neurons,
                            raw_snapshot.len()
                        );
                    }

                    // Minimal, high-signal debugging for BV "no power" issues:
                    // Log whether the outgoing visualization snapshot contains the Power cortical area (core idx=1).
                    // This pinpoints whether the failure is upstream (sampling/packaging) or downstream (BV decode/apply).
                    if burst_num % 30 == 0 {
                        use feagi_structures::genomic::cortical_area::CoreCorticalType;
                        static POWER_ID_B64: std::sync::LazyLock<String> =
                            std::sync::LazyLock::new(|| {
                                CoreCorticalType::Power.to_cortical_id().as_base_64()
                            });

                        let power_neurons = raw_snapshot
                            .values()
                            .find(|d| d.cortical_id == *POWER_ID_B64)
                            .map(|d| d.neuron_ids.len())
                            .unwrap_or(0);

                        info!(
                                "[VIZ-DEBUG] burst={} transports: shm={} publisher={} should_publish_viz={} areas={} total_neurons={} power_neurons={}",
                                burst_num,
                                has_shm_writer,
                                has_viz_publisher,
                                should_publish_viz,
                                raw_snapshot.len(),
                                total_neurons,
                                power_neurons
                            );
                    }

                    // IMPORTANT: Single visualization pipeline per burst.
                    // If SHM is attached, we write to SHM and skip publisher handoff to avoid doing
                    // two independent serialization paths (maintenance + performance nightmare).
                    if has_shm_writer {
                        match encode_fire_data_to_xyzp(raw_snapshot, None) {
                            Ok(buffer) => {
                                let mut viz_writer_lock = viz_shm_writer.lock().unwrap();
                                if let Some(writer) = viz_writer_lock.as_mut() {
                                    if let Err(e) = writer.write_payload(&buffer) {
                                        error!("[BURST-LOOP] ❌ Failed to write viz SHM: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("[BURST-LOOP] ❌ Failed to encode for SHM: {}", e);
                            }
                        }
                    } else if should_publish_viz {
                        // Send raw data to publisher (non-blocking handoff; serialization is off-thread).
                        if let Some(ref publisher) = viz_publisher {
                            static PUBLISH_COUNTER: std::sync::atomic::AtomicU64 =
                                std::sync::atomic::AtomicU64::new(0);

                            let count =
                                PUBLISH_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if count % 30 == 0 {
                                trace!(
                                    "[BURST-LOOP] Viz handoff #{}: {} neurons -> publisher (serialization off-thread)",
                                    count,
                                    total_neurons
                                );
                            }

                            let publish_start = Instant::now();
                            for agent_id in viz_due_agents.iter() {
                                if let Err(e) = publisher.publish_raw_fire_queue_for_agent(
                                    agent_id,
                                    raw_snapshot.clone(),
                                ) {
                                    if is_missing_agent_publish_error(&e) {
                                        if !missing_viz_agent_logged.contains(agent_id) {
                                            warn!(
                                                "[BURST-LOOP] Visualization transport for '{}' not ready yet ({}). Keeping subscription and retrying.",
                                                agent_id, e
                                            );
                                            missing_viz_agent_logged.insert(agent_id.clone());
                                        }
                                    } else {
                                        error!(
                                            "[BURST-LOOP] ❌ VIZ HANDOFF ERROR for '{}': {}",
                                            agent_id, e
                                        );
                                    }
                                    continue;
                                }
                                missing_viz_agent_logged.remove(agent_id);
                                visualization_last_publish_time
                                    .write()
                                    .insert(agent_id.clone(), now);
                            }
                            let publish_duration = publish_start.elapsed();
                            if publish_duration.as_millis() > 5000 {
                                warn!(
                                    "[BURST-LOOP] Very slow viz publish handoff: {:.2}ms (burst {})",
                                    publish_duration.as_secs_f64() * 1000.0,
                                    burst_num
                                );
                            }
                        }
                    }
                }
            } // Close if let Some(fire_data_arc)

            fire_data_arc_opt // Return Arc for motor reuse
        } else {
            if burst_num % 100 == 0 {
                trace!("[BURST-LOOP] Fire queue sampling skipped (no consumers need data)");
            }
            None // No fire data needed
        }; // Assign to shared_fire_data_opt

        // Motor output generation and publishing (per-agent, filtered by subscriptions)
        // NOTE: has_motor_publisher and has_motor_shm already computed above for shared_fire_data_opt

        // CRITICAL: Log motor publisher state every 100 bursts (using INFO to guarantee visibility)
        if burst_num % 100 == 0 {
            trace!(
                "[BURST-LOOP] MOTOR PUBLISHER STATE: has_publisher={}, has_shm={}",
                has_motor_publisher,
                has_motor_shm
            );
        }

        if needs_motor {
            debug!("[BURST-LOOP] 🎮 MOTOR: Starting motor output generation (shared_fire_data available={})", shared_fire_data_opt.is_some());

            // Use shared fire data from above (Arc - zero cost!)
            if let Some(ref fire_data_arc) = shared_fire_data_opt {
                // Read subscriptions first so logs reflect whether motor output is actually in use.
                let subscriptions = motor_subscriptions.read();
                let has_motor_subscriptions = !subscriptions.is_empty();
                debug!(
                    "[BURST-LOOP] 🎮 MOTOR: Fire snapshot has {} cortical areas (all active areas, motor subscriptions active={})",
                    (**fire_data_arc).len(),
                    has_motor_subscriptions
                );

                // CRITICAL PERFORMANCE FIX: Clone mappings to release lock immediately
                // This prevents lock contention when ConnectomeManager tries to refresh the cache
                let cortical_id_mappings_motor = {
                    let mappings = cached_cortical_id_mappings.lock().unwrap();
                    mappings.clone()
                };

                // Convert to RawFireQueueSnapshot (clone data for motor processing)
                let mut motor_snapshot = RawFireQueueSnapshot::new();
                for (area_id, (neuron_ids, coords_x, coords_y, coords_z, potentials)) in
                    fire_data_arc.iter()
                {
                    if neuron_ids.is_empty() {
                        continue;
                    }

                    // Get cortical_id from cached mappings (no NPU lock needed!)
                    // CRITICAL: For reserved areas (0=_death, 1=_power, 2=_fatigue), use CoreCorticalType
                    // even if cache is empty, so BV can identify them correctly
                    // For other areas, skip if not in cache (cache should be populated from ConnectomeManager)
                    let cortical_id = match cortical_id_mappings_motor.get(area_id) {
                        Some(id) => id.clone(),
                        None => {
                            // Fallback for reserved core areas (BV needs correct cortical_id to identify them)
                            use feagi_structures::genomic::cortical_area::CoreCorticalType;
                            match area_id {
                                0 => CoreCorticalType::Death.to_cortical_id().as_base_64(),
                                1 => CoreCorticalType::Power.to_cortical_id().as_base_64(),
                                2 => CoreCorticalType::Fatigue.to_cortical_id().as_base_64(),
                                _ => {
                                    // Skip areas not in cache (cache should be populated from ConnectomeManager)
                                    // Log warning only once per area to avoid spam
                                    static WARNED_AREAS_MOTOR: std::sync::LazyLock<
                                        std::sync::Mutex<ahash::AHashSet<u32>>,
                                    > = std::sync::LazyLock::new(|| {
                                        std::sync::Mutex::new(ahash::AHashSet::new())
                                    });
                                    let mut warned = WARNED_AREAS_MOTOR.lock().unwrap();
                                    if !warned.contains(area_id) {
                                        warn!(
                                            "[BURST-LOOP] ⚠️ Area {} not in cortical_id cache - skipping motor. Cache should be refreshed from ConnectomeManager.",
                                            area_id
                                        );
                                        warned.insert(*area_id);
                                    }
                                    continue; // Skip this area - can't process without valid cortical_id
                                }
                            }
                        }
                    };

                    if has_motor_subscriptions || has_motor_shm {
                        debug!(
                            "[BURST-LOOP] 🎮 MOTOR: Fire snapshot area {} ('{}') has {} neurons firing",
                            area_id,
                            cortical_id.escape_debug(),
                            neuron_ids.len()
                        );
                    }

                    motor_snapshot.insert(
                        *area_id,
                        RawFireQueueData {
                            cortical_area_idx: *area_id,
                            cortical_id,
                            neuron_ids: neuron_ids.clone(),
                            coords_x: coords_x.clone(),
                            coords_y: coords_y.clone(),
                            coords_z: coords_z.clone(),
                            potentials: potentials.clone(),
                        },
                    );
                }

                debug!(
                    "[BURST-LOOP] 🎮 MOTOR: Built snapshot with {} areas",
                    motor_snapshot.len()
                );

                // DEBUG: Log subscription state every 30 bursts
                if burst_num % 30 == 0 {
                    if subscriptions.is_empty() {
                        trace!("[BURST-LOOP] No motor subscriptions");
                    } else {
                        trace!("[BURST-LOOP] {} motor subscriptions", subscriptions.len());
                        for (agent_id, cortical_ids) in subscriptions.iter() {
                            trace!("[BURST-LOOP] Agent '{}' -> {:?}", agent_id, cortical_ids);
                        }
                    }
                }

                if subscriptions.is_empty() {
                    static FIRST_EMPTY_LOG: std::sync::atomic::AtomicBool =
                        std::sync::atomic::AtomicBool::new(false);
                    if !FIRST_EMPTY_LOG.load(std::sync::atomic::Ordering::Relaxed) {
                        debug!("[BURST-LOOP] 🎮 No motor subscriptions (no agents registered with motor capability)");
                        FIRST_EMPTY_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                } else {
                    static FIRST_MOTOR_LOG: std::sync::atomic::AtomicBool =
                        std::sync::atomic::AtomicBool::new(false);
                    if !FIRST_MOTOR_LOG.load(std::sync::atomic::Ordering::Relaxed) {
                        debug!(
                            "[BURST-LOOP] 🎮 Motor output active: {} agents subscribed",
                            subscriptions.len()
                        );
                        FIRST_MOTOR_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
                    }

                    // Generate motor output for each subscribed agent
                    // Note: We clone motor_snapshot for each agent (acceptable overhead for typical 1-2 agents)
                    let now = Instant::now();
                    let burst_hz = *frequency_hz.lock().unwrap();

                    for (agent_id, subscribed_cortical_ids) in subscriptions.iter() {
                        let rate_hz = motor_output_rates_hz
                            .read()
                            .get(agent_id)
                            .copied()
                            .unwrap_or(burst_hz);

                        if rate_hz <= 0.0 {
                            warn!(
                                "[BURST-LOOP] 🎮 MOTOR: Skipping agent '{}' due to invalid rate {}Hz",
                                agent_id, rate_hz
                            );
                            continue;
                        }

                        let interval = Duration::from_secs_f64(1.0 / rate_hz);
                        if let Some(last_sent) =
                            motor_last_publish_time.read().get(agent_id).copied()
                        {
                            if now.duration_since(last_sent) < interval {
                                continue;
                            }
                        }

                        debug!(
                            "[BURST-LOOP] 🎮 MOTOR: Encoding for agent '{}' with filter: {:?}",
                            agent_id,
                            subscribed_cortical_ids
                                .iter()
                                .map(|s| s.escape_debug().to_string())
                                .collect::<Vec<_>>()
                        );

                        // Filter by cortical_id strings (e.g., {"omot00"})
                        let cortical_id_filter = Some(subscribed_cortical_ids);

                        let encode_start = Instant::now();
                        // Clone for each agent (minimal overhead, and allows zero-copy within encode function)
                        match encode_fire_data_to_xyzp(motor_snapshot.clone(), cortical_id_filter) {
                            Ok(motor_bytes) => {
                                debug!(
                                    "[BURST-LOOP] 🎮 MOTOR: Encoded {} bytes for agent '{}'",
                                    motor_bytes.len(),
                                    agent_id
                                );
                                // Skip if no data (no neurons fired in subscribed areas)
                                if motor_bytes.is_empty() {
                                    debug!("[BURST-LOOP] 🎮 MOTOR: Skipping agent '{}' - no matching neurons", agent_id);
                                    continue;
                                }

                                let encode_duration = encode_start.elapsed();

                                static FIRST_ENCODE_LOG: std::sync::atomic::AtomicBool =
                                    std::sync::atomic::AtomicBool::new(false);
                                if !FIRST_ENCODE_LOG.load(std::sync::atomic::Ordering::Relaxed) {
                                    debug!(
                                        "[BURST-LOOP] 🎮 Encoded motor output for '{}': {} bytes in {:?}",
                                        agent_id, motor_bytes.len(), encode_duration
                                    );
                                    FIRST_ENCODE_LOG
                                        .store(true, std::sync::atomic::Ordering::Relaxed);
                                }

                                let mut published = false;

                                // Publish via ZMQ to agent
                                if let Some(ref publisher) = motor_publisher {
                                    match publisher.publish_motor(agent_id, &motor_bytes) {
                                        Ok(_) => {
                                            // Log every motor send (not just first) for debugging
                                            debug!(
                                                "[BURST-LOOP] ✅ PUBLISHED motor data to agent '{}': {} bytes",
                                                agent_id, motor_bytes.len()
                                            );
                                            published = true;
                                        }
                                        Err(e) => {
                                            if is_missing_agent_publish_error(&e) {
                                                if !missing_motor_agent_logged.contains(agent_id) {
                                                    warn!(
                                                        "[BURST-LOOP] Motor transport for '{}' not ready yet ({}). Keeping subscription and retrying.",
                                                        agent_id, e
                                                    );
                                                    missing_motor_agent_logged
                                                        .insert(agent_id.clone());
                                                }
                                            } else {
                                                error!(
                                                    "[BURST-LOOP] ❌ MOTOR PUBLISH ERROR for '{}': {}",
                                                    agent_id, e
                                                );
                                            }
                                        }
                                    }
                                } else {
                                    info!("[BURST-LOOP] 🎮 Motor publisher not available (None)");
                                }

                                // Write to motor SHM if available (for local agents)
                                if let Some(writer) = motor_shm_writer.lock().unwrap().as_mut() {
                                    if let Err(e) = writer.write_payload(&motor_bytes) {
                                        error!("[BURST-LOOP] ❌ Failed to write motor SHM: {}", e);
                                    } else {
                                        published = true;
                                    }
                                }

                                if published {
                                    missing_motor_agent_logged.remove(agent_id);
                                    motor_last_publish_time
                                        .write()
                                        .insert(agent_id.clone(), now);
                                }
                            }
                            Err(e) => {
                                error!(
                                    "[BURST-LOOP] ❌ Failed to encode motor output for '{}': {}",
                                    agent_id, e
                                );
                            }
                        }
                    }
                    drop(subscriptions);
                }
            } else {
                // No fire data available
                static MOTOR_NO_DATA_LOGGED: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                if !MOTOR_NO_DATA_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                    info!("[BURST-LOOP] 🎮 MOTOR: No fire data available (shared_fire_data_opt is None)");
                    MOTOR_NO_DATA_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            }
        } else {
            // Motor publisher not available - log once
            static MOTOR_DISABLED_LOGGED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !MOTOR_DISABLED_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                info!(
                    "[BURST-LOOP] ❌ MOTOR DISABLED: No motor publisher or SHM writer available!"
                );
                MOTOR_DISABLED_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        } // Close motor block

        let post_burst_duration = post_burst_start.elapsed();
        // Only warn for extreme cases (>5 seconds) - batch processing can take time for viz/motor
        if post_burst_duration.as_millis() > 5000 {
            warn!(
                "[BURST-LOOP] Very slow post-burst processing: {:.2}ms (viz+motor, burst {})",
                post_burst_duration.as_secs_f64() * 1000.0,
                burst_num
            );
        }

        let stats_start = Instant::now();
        // Performance logging every 5 seconds
        let now = Instant::now();
        if now.duration_since(last_stats_time).as_secs() >= 5 {
            if !burst_times.is_empty() {
                let avg_interval: Duration =
                    burst_times.iter().sum::<Duration>() / burst_times.len() as u32;
                let actual_hz = 1.0 / avg_interval.as_secs_f64();
                let avg_neurons = total_neurons_fired / burst_times.len();

                let desired_hz = *frequency_hz.lock().unwrap();
                debug!(
                    burst_num,
                    desired_hz,
                    actual_hz,
                    accuracy_percent = (actual_hz / desired_hz * 100.0),
                    avg_neurons,
                    "📊 Burst loop stats"
                );
            }

            last_stats_time = now;
            total_neurons_fired = 0;
        }
        let stats_duration = stats_start.elapsed();
        if stats_duration.as_millis() > 10 {
            warn!(
                "[BURST-LOOP] ⚠️ Slow stats processing: {:.2}ms (burst {})",
                stats_duration.as_secs_f64() * 1000.0,
                burst_num
            );
        }

        // CRITICAL: Check shutdown flag before entering sleep
        // Exit immediately if shutdown was requested during visualization/stats
        if !running.load(Ordering::Relaxed) {
            break;
        }

        // Log total iteration time if it's slow
        // Warn if burst iteration is truly slow (>1 second) - indicates real problems
        let iteration_duration = iteration_start.elapsed();
        if iteration_duration.as_millis() > 1000 {
            // BREAKDOWN: Show where time was spent (use stored duration from process_burst)
            // Note: process_burst_duration is only available in the NPU lock scope, so we approximate
            // The actual breakdown will be logged in the next iteration when we have all timings
            warn!(
                    "[BURST-LOOP] ⚠️ Slow burst iteration: {:.2}ms total (burst {}) | breakdown: gap_before_post={:.2}ms, post_burst={:.2}ms, stats={:.2}ms, unaccounted={:.2}ms | process_burst_ms={:.2} fired={} power_inj={} syn_inj={} processed={} refractory={} lock_wait_ms={:.2}",
                    iteration_duration.as_secs_f64() * 1000.0,
                    burst_num,
                    time_between_npu_release_and_post_burst.as_secs_f64() * 1000.0,
                    post_burst_duration.as_secs_f64() * 1000.0,
                    stats_duration.as_secs_f64() * 1000.0,
                iteration_duration.as_secs_f64() * 1000.0 - time_between_npu_release_and_post_burst.as_secs_f64() * 1000.0 - post_burst_duration.as_secs_f64() * 1000.0 - stats_duration.as_secs_f64() * 1000.0,
                last_process_duration
                    .map(|d| d.as_secs_f64() * 1000.0)
                    .unwrap_or(0.0),
                last_burst_stats.map(|s| s.0).unwrap_or(0),
                last_burst_stats.map(|s| s.1).unwrap_or(0),
                last_burst_stats.map(|s| s.2).unwrap_or(0),
                last_burst_stats.map(|s| s.3).unwrap_or(0),
                last_burst_stats.map(|s| s.4).unwrap_or(0),
                lock_wait_duration.as_secs_f64() * 1000.0
            );
        }

        // Update last iteration end time
        {
            let mut last_end = LAST_ITERATION_END.lock().unwrap();
            *last_end = Some(Instant::now());
        }

        // Adaptive sleep (RTOS-friendly timing)
        // Strategy: <5Hz = chunked sleep, 5-100Hz = hybrid, >100Hz = busy-wait
        // CRITICAL: Break sleep into chunks to allow responsive shutdown
        // Maximum sleep chunk: 50ms to ensure shutdown responds within ~50ms
        // CRITICAL: Read frequency dynamically to allow runtime updates
        let _sleep_start = Instant::now();
        let current_frequency_hz = *frequency_hz.lock().unwrap();
        let interval_sec = 1.0 / current_frequency_hz;
        let target_time = burst_start + Duration::from_secs_f64(interval_sec);
        let now = Instant::now();

        // Log if we're significantly past target (>1 second overshoot) - indicates real problems
        if now > target_time {
            let overshoot = now.duration_since(target_time);
            if overshoot.as_millis() > 1000 {
                warn!(
                    "[BURST-LOOP] Iteration overshoot: {:.2}ms past target (burst {}) - no sleep needed",
                    overshoot.as_secs_f64() * 1000.0,
                    burst_num
                );
            }
        }

        if now < target_time {
            let remaining = target_time - now;

            if current_frequency_hz < 5.0 {
                // Low frequency: sleep in small chunks to allow shutdown
                // Check shutdown flag every 50ms for responsive shutdown (<100ms response time)
                let chunk_size = Duration::from_millis(50);
                let mut remaining_sleep = remaining;
                while remaining_sleep.as_millis() > 0 && running.load(Ordering::Relaxed) {
                    let sleep_duration = remaining_sleep.min(chunk_size);
                    thread::sleep(sleep_duration);
                    // Check flag immediately after sleep
                    if !running.load(Ordering::Relaxed) {
                        break;
                    }
                    let elapsed = Instant::now();
                    remaining_sleep = if elapsed < target_time {
                        target_time - elapsed
                    } else {
                        Duration::ZERO
                    };
                }
            } else if current_frequency_hz > 100.0 {
                // High frequency: pure busy-wait (always responsive)
                while Instant::now() < target_time && running.load(Ordering::Relaxed) {}
            } else {
                // Medium frequency: hybrid (sleep 80%, busy-wait 20%)
                let sleep_duration = remaining.mul_f64(0.8);
                if sleep_duration.as_micros() > 100 {
                    // Break sleep into chunks for responsive shutdown
                    let chunk_size = Duration::from_millis(50); // Check every 50ms for faster shutdown
                    let mut remaining_sleep = sleep_duration;
                    while remaining_sleep.as_millis() > 0 && running.load(Ordering::Relaxed) {
                        let sleep_chunk = remaining_sleep.min(chunk_size);
                        thread::sleep(sleep_chunk);
                        // Check flag immediately after sleep
                        if !running.load(Ordering::Relaxed) {
                            break;
                        }
                        let elapsed = Instant::now();
                        let elapsed_sleep = elapsed.duration_since(burst_start);
                        let target_sleep = Duration::from_secs_f64(interval_sec * 0.8);
                        remaining_sleep = if elapsed_sleep < target_sleep {
                            target_sleep - elapsed_sleep
                        } else {
                            Duration::ZERO
                        };
                    }
                }
                // Busy-wait remainder (responsive to shutdown flag)
                while Instant::now() < target_time && running.load(Ordering::Relaxed) {}
            }
        }
    }

    let timestamp = get_timestamp();
    info!(
        "[{}] [BURST-LOOP] 🛑 Main loop stopped after {} bursts",
        timestamp, burst_num
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_burst_loop_lifecycle() {
        // Use a unit type for the generic parameter since we're not testing visualization/motor
        struct NoViz;
        impl VisualizationPublisher for NoViz {
            fn publish_raw_fire_queue_for_agent(
                &self,
                _agent_id: &str,
                _fire_data: RawFireQueueSnapshot,
            ) -> Result<(), String> {
                Ok(())
            }
        }

        struct NoMotor;
        impl MotorPublisher for NoMotor {
            fn publish_motor(&self, _agent_id: &str, _data: &[u8]) -> Result<(), String> {
                Ok(())
            }
        }

        let rust_npu = <crate::RustNPU<
            feagi_npu_runtime::StdRuntime,
            f32,
            crate::backend::CPUBackend,
        >>::new_cpu_only(1000, 10000, 20);
        let npu = Arc::new(TracingMutex::new(DynamicNPU::F32(rust_npu), "TestNPU"));
        let mut runner = BurstLoopRunner::new::<NoViz, NoMotor>(npu, None, None, 10.0);

        assert!(!runner.is_running());

        // Power neurons are now auto-discovered by cortical_area=1
        runner.start().unwrap();

        assert!(runner.is_running());

        // Let it run for 100ms
        thread::sleep(Duration::from_millis(100));

        // Should have processed ~1 burst at 10Hz
        assert!(runner.get_burst_count() >= 1);

        runner.stop();
        assert!(!runner.is_running());
    }

    #[test]
    fn test_fire_queue_api_cache_uses_non_deduped_snapshot() {
        // Regression test for `/v1/burst_engine/fire_queue`:
        //
        // `RustNPU::process_burst()` already samples the FQ sampler internally (Phase 5).
        // If the burst loop then tries to call `sample_fire_queue()` again, it can get `None`
        // due to deduplication, causing the HTTP endpoint to show an empty fire queue even
        // when neurons fired (while visualization/motor still see activity).

        struct NoViz;
        impl VisualizationPublisher for NoViz {
            fn publish_raw_fire_queue_for_agent(
                &self,
                _agent_id: &str,
                _fire_data: RawFireQueueSnapshot,
            ) -> Result<(), String> {
                Ok(())
            }
        }

        struct NoMotor;
        impl MotorPublisher for NoMotor {
            fn publish_motor(&self, _agent_id: &str, _data: &[u8]) -> Result<(), String> {
                Ok(())
            }
        }

        use feagi_npu_runtime::StdRuntime;
        use feagi_structures::genomic::cortical_area::CoreCorticalType;

        // Build an NPU with one neuron we can deterministically force to fire.
        let mut rust_npu =
            <crate::RustNPU<StdRuntime, f32, crate::backend::CPUBackend>>::new_cpu_only(
                100, 1000, 10,
            );

        // Use a non-core cortical_idx to avoid implicit core neuron creation (0..=2).
        rust_npu.register_cortical_area(3, CoreCorticalType::Death.to_cortical_id().as_base_64());
        // Ensure process_burst() produces a fire queue sample (sampling is typically gated by subscriber flags).
        rust_npu.set_visualization_subscribers(true);

        let neuron = rust_npu
            .add_neuron(
                1.0,      // threshold
                f32::MAX, // threshold_limit (MAX = no limit, SIMD-friendly encoding)
                0.0,      // leak_coefficient
                0.0,      // resting_potential
                0,        // neuron_type
                0,        // refractory_period
                1.0,      // excitability
                0,        // consecutive_fire_limit
                0,        // snooze_period
                true,     // mp_charge_accumulation
                3,        // cortical_area
                0,
                0,
                0,
            )
            .unwrap();

        // Stage a strong sensory injection so it survives Phase-1 FCL clear and fires on burst 1.
        rust_npu.inject_sensory_with_potentials(&[(neuron, 128.0)]);

        let npu = Arc::new(TracingMutex::new(DynamicNPU::F32(rust_npu), "TestNPU"));
        let mut runner = BurstLoopRunner::new::<NoViz, NoMotor>(npu, None, None, 5.0);

        runner.start().unwrap();

        // Wait for first burst to complete (runner executes burst immediately, then sleeps).
        let start = Instant::now();
        while runner.get_burst_count() < 1 && start.elapsed() < Duration::from_secs(1) {
            thread::sleep(Duration::from_millis(5));
        }

        let fq_sample = runner
            .get_fire_queue_sample()
            .expect("Expected cached fire queue sample after first burst");

        let fired_in_area = fq_sample
            .get(&3)
            .map(|(neuron_ids, _, _, _, _)| neuron_ids.contains(&neuron.0))
            .unwrap_or(false);

        runner.stop();

        assert!(
            fired_in_area,
            "Expected neuron {} to appear in cached fire queue for cortical_idx=3",
            neuron.0
        );
    }

    #[test]
    fn test_visualization_rate_validation() {
        struct NoViz;
        impl VisualizationPublisher for NoViz {
            fn publish_raw_fire_queue_for_agent(
                &self,
                _agent_id: &str,
                _fire_data: RawFireQueueSnapshot,
            ) -> Result<(), String> {
                Ok(())
            }
        }

        struct NoMotor;
        impl MotorPublisher for NoMotor {
            fn publish_motor(&self, _agent_id: &str, _data: &[u8]) -> Result<(), String> {
                Ok(())
            }
        }

        let rust_npu = <crate::RustNPU<
            feagi_npu_runtime::StdRuntime,
            f32,
            crate::backend::CPUBackend,
        >>::new_cpu_only(1000, 10000, 20);
        let npu = Arc::new(TracingMutex::new(DynamicNPU::F32(rust_npu), "TestNPU"));
        let runner = BurstLoopRunner::new::<NoViz, NoMotor>(npu, None, None, 10.0);

        assert!(runner
            .register_visualization_subscriptions_with_rate("viz-agent".to_string(), 20.0)
            .is_err());
        assert!(runner
            .register_visualization_subscriptions_with_rate("viz-agent".to_string(), 5.0)
            .is_ok());
    }
}
