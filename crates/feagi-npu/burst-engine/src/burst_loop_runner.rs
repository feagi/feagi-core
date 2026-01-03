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
#[cfg(feature = "std")]
use crate::DynamicNPU;
use feagi_npu_neural::types::NeuronId;
use parking_lot::RwLock as ParkingLotRwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Type alias for fire queue sample data structure
type FireQueueSample = ahash::AHashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>;
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
    fn publish_raw_fire_queue(&self, fire_data: RawFireQueueSnapshot) -> Result<(), String>;
}

pub trait MotorPublisher: Send + Sync {
    /// Publish motor data to a specific agent (XYZP format)
    fn publish_motor(&self, agent_id: &str, data: &[u8]) -> Result<(), String>;
}

/// Burst loop runner - manages the main neural processing loop
///
/// 🦀 Power neurons are stored in RustNPU, not here - 100% Rust!
/// 🦀 Burst count is stored in NPU - single source of truth!
pub struct BurstLoopRunner {
    /// Shared NPU instance (holds power neurons internally + burst count)
    /// DynamicNPU dispatches to either F32 or INT8 variant based on genome
    npu: Arc<Mutex<DynamicNPU>>,
    /// Target frequency in Hz (shared with burst thread for dynamic updates)
    frequency_hz: Arc<Mutex<f64>>,
    /// Running flag (atomic for thread-safe stop)
    running: Arc<AtomicBool>,
    /// Thread handle (for graceful shutdown)
    thread_handle: Option<thread::JoinHandle<()>>,
    /// Sensory agent manager (per-agent injection threads)
    pub sensory_manager: Arc<Mutex<AgentManager>>,
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
    /// Cached cortical_idx -> cortical_id mappings (from ConnectomeManager, not NPU)
    /// Refreshed periodically to avoid ConnectomeManager lock contention
    /// This eliminates NPU lock acquisitions that were causing 1-3s delays!
    cached_cortical_id_mappings: Arc<Mutex<ahash::AHashMap<u32, String>>>,
    /// Burst count when mappings were last refreshed
    last_cortical_id_refresh: Arc<Mutex<u64>>,
    /// Cached cortical_idx -> heatmap_chunk_size mappings (from ConnectomeManager)
    /// Used to determine when to apply heatmap aggregation for large areas
    cached_chunk_sizes: Arc<Mutex<ahash::AHashMap<u32, (u32, u32, u32)>>>,
}

impl BurstLoopRunner {
    /// Create a new burst loop runner
    ///
    /// # Arguments
    /// * `npu` - The NPU to run bursts on
    /// * `viz_publisher` - Optional visualization publisher (None = no ZMQ visualization)
    /// * `frequency_hz` - Burst frequency in Hz
    pub fn new<V: VisualizationPublisher + 'static, M: MotorPublisher + 'static>(
        npu: Arc<Mutex<DynamicNPU>>,
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
                fn publish_raw_fire_queue(
                    &self,
                    fire_data: RawFireQueueSnapshot,
                ) -> Result<(), String> {
                    self.0.lock().unwrap().publish_raw_fire_queue(fire_data)
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
            cached_cortical_id_mappings: Arc::new(Mutex::new(ahash::AHashMap::new())),
            last_cortical_id_refresh: Arc::new(Mutex::new(0)),
            cached_chunk_sizes: Arc::new(Mutex::new(ahash::AHashMap::new())),
            viz_shm_writer: Arc::new(Mutex::new(None)), // Initialized later via attach_viz_shm_writer
            motor_shm_writer: Arc::new(Mutex::new(None)), // Initialized later via attach_motor_shm_writer
            viz_publisher: viz_publisher_trait, // Trait object for visualization (NO PYTHON CALLBACKS!)
            motor_publisher: motor_publisher_trait, // Trait object for motor (NO PYTHON CALLBACKS!)
            motor_subscriptions: Arc::new(ParkingLotRwLock::new(ahash::AHashMap::new())),
            fcl_sampler_frequency: Arc::new(Mutex::new(30.0)), // Default 30Hz for visualization
            fcl_sampler_consumer: Arc::new(Mutex::new(1)),     // Default: 1 = visualization only
            cached_burst_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            cached_fire_queue: Arc::new(Mutex::new(None)), // Cached fire queue for API (Arc-wrapped to avoid cloning)
            parameter_queue: ParameterUpdateQueue::new(),
            plasticity_notify: None, // Initialized later via set_plasticity_notify_callback
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

    /// Attach visualization SHM writer (called from Python after registration)
    pub fn attach_viz_shm_writer(
        &mut self,
        shm_path: std::path::PathBuf,
    ) -> Result<(), std::io::Error> {
        let writer = crate::viz_shm_writer::VizSHMWriter::new(shm_path, None, None)?;
        let mut guard = self.viz_shm_writer.lock().unwrap();
        *guard = Some(writer);
        Ok(())
    }

    /// Attach motor SHM writer (called from Python after registration)
    pub fn attach_motor_shm_writer(
        &mut self,
        shm_path: std::path::PathBuf,
    ) -> Result<(), std::io::Error> {
        let writer = crate::motor_shm_writer::MotorSHMWriter::new(shm_path, None, None)?;
        let mut guard = self.motor_shm_writer.lock().unwrap();
        *guard = Some(writer);
        Ok(())
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

    /// Unregister an agent's motor subscriptions
    /// Called when an agent disconnects
    pub fn unregister_motor_subscriptions(&self, agent_id: &str) {
        if self.motor_subscriptions.write().remove(agent_id).is_some() {
            info!(
                "[BURST-RUNNER] Removed motor subscriptions for agent '{}'",
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
        let cached_burst_count = self.cached_burst_count.clone(); // For lock-free burst count reads
        let cached_fire_queue = self.cached_fire_queue.clone(); // For caching fire queue data
        let param_queue = self.parameter_queue.clone(); // Parameter update queue
        let plasticity_notify = self.plasticity_notify.clone(); // Clone Arc for thread
        let cached_cortical_id_mappings = self.cached_cortical_id_mappings.clone();
        let last_cortical_id_refresh = self.last_cortical_id_refresh.clone();
        let cached_chunk_sizes = self.cached_chunk_sizes.clone();

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
                        cached_burst_count,
                        cached_fire_queue,
                        param_queue,
                        plasticity_notify,
                        cached_cortical_id_mappings,
                        last_cortical_id_refresh,
                        cached_chunk_sizes,
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
        self.npu.lock().unwrap().get_last_fcl_snapshot()
    }

    /// Get current fire queue for monitoring
    /// Returns the last cached fire queue data from previous burst
    pub fn get_fire_queue_sample(&mut self) -> Option<FireQueueSample> {
        let cached = self.cached_fire_queue.lock().unwrap().clone();
        if let Some(ref sample_arc) = cached {
            debug!("[BURST-LOOP-RUNNER] Returning cached fire queue: {} areas", sample_arc.len());
        } else {
            debug!("[BURST-LOOP-RUNNER] Cached fire queue is None");
        }
        // Unwrap Arc to return the actual data (API needs owned data)
        cached.map(|arc| (*arc).clone())
    }

    /// Get Fire Ledger window configurations for all cortical areas
    pub fn get_fire_ledger_configs(&self) -> Vec<(u32, usize)> {
        self.npu.lock().unwrap().get_all_fire_ledger_configs()
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
    pub fn get_npu(&self) -> Arc<Mutex<DynamicNPU>> {
        self.npu.clone()
    }

    /// Refresh cortical_idx -> cortical_id mappings from ConnectomeManager
    /// This should be called when cortical areas are created/updated
    /// CRITICAL: This eliminates NPU lock acquisitions that were causing 1-3s delays!
    pub fn refresh_cortical_id_mappings(&self, mappings: ahash::AHashMap<u32, String>) {
        *self.cached_cortical_id_mappings.lock().unwrap() = mappings;
        let current_burst = self.cached_burst_count.load(std::sync::atomic::Ordering::Relaxed);
        *self.last_cortical_id_refresh.lock().unwrap() = current_burst;
        debug!(
            "[BURST-LOOP] Refreshed cortical_id mappings: {} areas (burst {})",
            self.cached_cortical_id_mappings.lock().unwrap().len(),
            current_burst
        );
    }

    /// Refresh cortical_idx -> heatmap_chunk_size mappings from ConnectomeManager
    /// This should be called when cortical areas are created/updated
    pub fn refresh_chunk_sizes(&self, chunk_sizes: ahash::AHashMap<u32, (u32, u32, u32)>) {
        *self.cached_chunk_sizes.lock().unwrap() = chunk_sizes;
        debug!(
            "[BURST-LOOP] Refreshed chunk sizes: {} areas",
            self.cached_chunk_sizes.lock().unwrap().len()
        );
    }
}

impl Drop for BurstLoopRunner {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Aggregate fire queue data into heatmap chunks for large-area visualization
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
/// * `chunk_size` - Chunk dimensions (x, y, z)
///
/// # Returns
///
/// Aggregated data: (chunk_coords_x, chunk_coords_y, chunk_coords_z, chunk_potentials, chunk_counts)
fn aggregate_into_heatmap_chunks(
    neuron_ids: &[u32],
    coords_x: &[u32],
    coords_y: &[u32],
    coords_z: &[u32],
    potentials: &[f32],
    chunk_size: (u32, u32, u32),
) -> (Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>, Vec<u32>) {
    let (chunk_x, chunk_y, chunk_z) = chunk_size;
    
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
    
    (chunk_coords_x, chunk_coords_y, chunk_coords_z, chunk_potentials, chunk_counts)
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

/// Main burst processing loop (runs in dedicated thread)
///
/// This is the HOT PATH - zero Python involvement!
/// Power neurons are read directly from RustNPU's internal state.
/// Burst count is tracked by NPU - single source of truth!
#[allow(clippy::too_many_arguments)]
fn burst_loop(
    npu: Arc<Mutex<DynamicNPU>>,
    frequency_hz: Arc<Mutex<f64>>, // Shared frequency - can be updated while running
    running: Arc<AtomicBool>,
    viz_shm_writer: Arc<Mutex<Option<crate::viz_shm_writer::VizSHMWriter>>>,
    motor_shm_writer: Arc<Mutex<Option<crate::motor_shm_writer::MotorSHMWriter>>>,
    viz_publisher: Option<Arc<dyn VisualizationPublisher>>, // Trait object for visualization (NO PYTHON CALLBACKS!)
    motor_publisher: Option<Arc<dyn MotorPublisher>>, // Trait object for motor (NO PYTHON CALLBACKS!)
    motor_subscriptions: Arc<ParkingLotRwLock<ahash::AHashMap<String, ahash::AHashSet<String>>>>,
    cached_burst_count: Arc<std::sync::atomic::AtomicU64>, // For lock-free burst count reads
    cached_fire_queue: Arc<Mutex<Option<Arc<FireQueueSample>>>>, // For caching fire queue data (Arc-wrapped to avoid cloning)
    parameter_queue: ParameterUpdateQueue,                 // Asynchronous parameter update queue
    plasticity_notify: Option<Arc<dyn Fn(u64) + Send + Sync>>, // Plasticity notification callback
    cached_cortical_id_mappings: Arc<Mutex<ahash::AHashMap<u32, String>>>, // Cached cortical_idx -> cortical_id
    last_cortical_id_refresh: Arc<Mutex<u64>>, // Burst count when mappings were last refreshed
    cached_chunk_sizes: Arc<Mutex<ahash::AHashMap<u32, (u32, u32, u32)>>>, // Cached cortical_idx -> chunk_size
) {
    let timestamp = get_timestamp();
    let initial_freq = *frequency_hz.lock().unwrap();
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

    while running.load(Ordering::Acquire) {
        let iteration_start = Instant::now();
        let burst_start = Instant::now();

        // DIAGNOSTIC: Log that we're alive
        if burst_num.is_multiple_of(100) {
            trace!("[BURST-LOOP] Burst {} starting (loop is alive)", burst_num);
        }
        
        // Track time since last burst (to detect blocking)
        static LAST_ITERATION_END: std::sync::Mutex<Option<Instant>> = std::sync::Mutex::new(None);
        {
            let mut last_end = LAST_ITERATION_END.lock().unwrap();
            if let Some(last) = *last_end {
                let gap = iteration_start.duration_since(last);
                if gap.as_millis() > 100 {
                    warn!(
                        "[BURST-LOOP] ⚠️ Large gap between bursts: {:.2}ms (expected ~66ms at 15Hz) - burst {}",
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

        let lock_start = Instant::now();
        if burst_num < 5 || burst_num.is_multiple_of(100) {
            trace!(
                "[BURST-LOOP-DIAGNOSTIC] Burst {}: Attempting NPU lock...",
                burst_num
            );
        }

        let should_exit = {
            let mut npu_lock = npu.lock().unwrap();
            let lock_acquired = Instant::now();
            let lock_wait_duration = lock_acquired.duration_since(lock_start);
            // Log if lock acquisition took significant time (could indicate contention)
            if lock_wait_duration.as_millis() > 10 {
                warn!(
                    "[BURST-LOOP] ⚠️ Slow NPU lock acquisition: {:.2}ms (burst {}) - possible lock contention!",
                    lock_wait_duration.as_secs_f64() * 1000.0,
                    burst_num
                );
            }
            if burst_num < 5 || burst_num.is_multiple_of(100) {
                trace!(
                    "[BURST-TIMING] Burst {}: NPU lock acquired in {:?}",
                    burst_num,
                    lock_wait_duration
                );
            }

            // Check flag again after acquiring lock (in case shutdown happened during lock wait)
            if !running.load(Ordering::Relaxed) {
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
                            "neuron_fire_threshold"
                            | "firing_threshold"
                            => {
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
                                        if let (Some(inc_x), Some(inc_y), Some(inc_z)) = (
                                            arr[0].as_f64(),
                                            arr[1].as_f64(),
                                            arr[2].as_f64(),
                                        ) {
                                            // Get base threshold from update metadata
                                            if let Some(base_threshold) = update.base_threshold {
                                                npu_lock.update_cortical_area_threshold_with_gradient(
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
                            debug!(
                                "[PARAM-QUEUE] Applied {}={} to {} neurons in area {}",
                                update.parameter_name, update.value, count, update.cortical_id
                            );
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

                let process_start = Instant::now();
                debug!("[BURST-TIMING] Starting process_burst()...");

                match npu_lock.process_burst() {
                    Ok(mut result) => {
                        let process_done = Instant::now();
                        let duration = process_done.duration_since(process_start);

                        if burst_num < 5 || burst_num.is_multiple_of(100) {
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
                        cached_burst_count.store(
                            current_burst,
                            std::sync::atomic::Ordering::Relaxed,
                        );
                        
                        // Notify plasticity service of completed burst (while NPU lock still held)
                        // This allows plasticity service to immediately query FireLedger data
                        // Callback is pre-cloned Arc, so this is just a function call (no allocation)
                        if let Some(ref notify_fn) = plasticity_notify {
                            trace!("[BURST-LOOP] 🧠 Notifying plasticity service of burst {}", current_burst);
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
                        
                        false // Continue processing
                    }
                    Err(e) => {
                        let timestamp = get_timestamp();
                        error!(
                            "[{}] [BURST-LOOP] ❌ Burst processing error: {}",
                            timestamp, e
                        );
                        false // Continue despite error
                    }
                }
            }
        };

        let npu_lock_release_time = Instant::now();
        if burst_num < 5 || burst_num.is_multiple_of(100) {
            trace!("[BURST-TIMING] Burst {}: NPU lock RELEASED", burst_num);
        }

        // Exit if shutdown was requested
        if should_exit || !running.load(Ordering::Relaxed) {
            break;
        }

        burst_num += 1;
        // Note: NPU.process_burst() already incremented its internal burst_count

        let post_burst_start = Instant::now();
        let time_between_npu_release_and_post_burst = post_burst_start.duration_since(npu_lock_release_time);
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
        static LAST_VIZ_PUBLISH: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let last_viz = LAST_VIZ_PUBLISH.load(std::sync::atomic::Ordering::Relaxed);
        // Use saturating_sub to prevent panic on clock adjustments (NTP sync, suspend/resume, etc.)
        let should_publish_viz = (now_ms.saturating_sub(last_viz) >= 33) && has_viz_publisher;

        // Sample fire queue ONCE and share between viz and motor using Arc (zero-cost sharing!)
        let has_motor_publisher = motor_publisher.is_some();
        let has_motor_shm = motor_shm_writer.lock().unwrap().is_some();
        let needs_motor = has_motor_publisher || has_motor_shm;
        let needs_fire_data = has_shm_writer || should_publish_viz || needs_motor;

        if burst_num.is_multiple_of(100) {
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

            if burst_num.is_multiple_of(100) {
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
                let cortical_id_mappings = cached_cortical_id_mappings.lock().unwrap();
                let chunk_sizes = cached_chunk_sizes.lock().unwrap();

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
                    let cortical_id = match cortical_id_mappings.get(area_id) {
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
                                    static WARNED_AREAS: std::sync::LazyLock<std::sync::Mutex<ahash::AHashSet<u32>>> = 
                                        std::sync::LazyLock::new(|| std::sync::Mutex::new(ahash::AHashSet::new()));
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

                    // Check if this area should use heatmap aggregation
                    let (final_coords_x, final_coords_y, final_coords_z, final_potentials, final_neuron_ids) = 
                        if let Some(&chunk_size) = chunk_sizes.get(area_id) {
                            // Apply heatmap aggregation for large areas
                            let (chunk_x, chunk_y, chunk_z, chunk_p, _chunk_counts) = 
                                aggregate_into_heatmap_chunks(
                                    neuron_ids,
                                    coords_x,
                                    coords_y,
                                    coords_z,
                                    potentials,
                                    chunk_size,
                                );
                            // For heatmap, use chunk indices as neuron IDs (or sequential IDs)
                            let chunk_ids: Vec<u32> = (0..chunk_x.len() as u32).collect();
                            (chunk_x, chunk_y, chunk_z, chunk_p, chunk_ids)
                        } else {
                            // No heatmap - use original data
                            (coords_x.clone(), coords_y.clone(), coords_z.clone(), potentials.clone(), neuron_ids.clone())
                        };

                    total_neurons += final_neuron_ids.len();

                    // Minimal memory visualization support:
                    // If this cortical_id is a MEMORY area, BV only needs the area to appear in the Type 11 stream.
                    // We emit a single point at (0,0,0) (memory areas are conceptually 1x1x1) so the client
                    // can trigger its jelly animation without requiring actual per-neuron coordinates.
                    // Detect memory areas by decoding cortical ID bytes (deterministic; no hardcoded IDs).
                    // Memory areas may be encoded as custom IDs prefixed by `cmem...`.
                    let is_memory_area = feagi_structures::genomic::cortical_area::CorticalID::try_from_base_64(&cortical_id)
                        .ok()
                        .is_some_and(|id| id.as_bytes().starts_with(b"cmem") || id.as_bytes()[0] == b'm');

                    // FEAGI-side diagnostics (must be easy to spot in logs):
                    // - If `cortical_id` falls back to "area_{idx}", Type11 serialization may drop the area.
                    // - If memory area is detected, we inject a single (0,0,0) point for BV.

                    // CRITICAL PERFORMANCE: Only clone vectors if needed (memory areas use small vectors)
                    // For normal areas, we must clone because we're reading from Arc (can't move)
                    // For heatmap areas, we already have the aggregated data
                    raw_snapshot.insert(
                        *area_id,
                        RawFireQueueData {
                            cortical_area_idx: *area_id,
                            cortical_id: cortical_id,
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
                    if burst_num.is_multiple_of(100) || total_neurons > 1000 {
                        debug!(
                            "[BURST-LOOP] 🔍 Sampled {} neurons from {} areas for viz",
                            total_neurons,
                            raw_snapshot.len()
                        );
                    }

                    // Send raw data to PNS (non-blocking handoff, PNS will serialize on its own thread)
                    if let Some(ref publisher) = viz_publisher {
                        static PUBLISH_COUNTER: std::sync::atomic::AtomicU64 =
                            std::sync::atomic::AtomicU64::new(0);

                        // Update shared timestamp (used for throttle check above)
                        LAST_VIZ_PUBLISH.store(now_ms, std::sync::atomic::Ordering::Relaxed);

                        let count =
                            PUBLISH_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if count.is_multiple_of(30) {
                            trace!(
                                "[BURST-LOOP] Viz handoff #{}: {} neurons -> PNS (serialization off-thread)",
                                count,
                                total_neurons
                            );
                        }

                        // CRITICAL PERFORMANCE: Move raw_snapshot instead of cloning (we don't need it after this)
                        // PNS will serialize on its own thread, so we can give it ownership
                        let publish_start = Instant::now();
                        if let Err(e) = publisher.publish_raw_fire_queue(raw_snapshot) {
                            error!("[BURST-LOOP] ❌ VIZ HANDOFF ERROR: {}", e);
                        }
                        let publish_duration = publish_start.elapsed();
                        // Lower threshold to catch smaller slowdowns (10ms instead of 100ms)
                        if publish_duration.as_millis() > 10 {
                            warn!(
                                "[BURST-LOOP] ⚠️ Slow viz publish handoff: {:.2}ms (burst {})",
                                publish_duration.as_secs_f64() * 1000.0,
                                burst_num
                            );
                        }
                    }

                    // SHM writer still needs serialized data (for local visualization)
                    // This is acceptable since SHM is local IPC, not network-bound
                    // NOTE: raw_snapshot was moved to publisher above, so we need to rebuild it for SHM
                    // This is acceptable since SHM is typically not used when PNS publisher is available
                    if has_shm_writer {
                        // Rebuild raw_snapshot for SHM (only if SHM is actually being used)
                        // TODO: Share raw_snapshot between publisher and SHM to avoid rebuilding
                        warn!("[BURST-LOOP] ⚠️ SHM writer requires rebuilding raw_snapshot (performance impact)");
                        // For now, skip SHM encoding if we already published to PNS
                        // SHM is typically only used for local visualization without PNS
                        // Rebuild raw_snapshot from fire_data_arc for SHM
                        let mut shm_snapshot = RawFireQueueSnapshot::new();
                        let cortical_id_mappings = cached_cortical_id_mappings.lock().unwrap();
                        for (area_id, (neuron_ids, coords_x, coords_y, coords_z, potentials)) in
                            fire_data_arc.iter()
                        {
                            if neuron_ids.is_empty() {
                                continue;
                            }
                            let cortical_id = match cortical_id_mappings.get(area_id) {
                                Some(id) => id.clone(),
                                None => {
                                    use feagi_structures::genomic::cortical_area::CoreCorticalType;
                                    match area_id {
                                        0 => CoreCorticalType::Death.to_cortical_id().as_base_64(),
                                        1 => CoreCorticalType::Power.to_cortical_id().as_base_64(),
                                        2 => CoreCorticalType::Fatigue.to_cortical_id().as_base_64(),
                                        _ => continue,
                                    }
                                }
                            };
                            shm_snapshot.insert(
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
                        match encode_fire_data_to_xyzp(shm_snapshot, None) {
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
                    }
                }
            } // Close if let Some(fire_data_arc)

            fire_data_arc_opt // Return Arc for motor reuse
        } else {
            if burst_num.is_multiple_of(100) {
                trace!("[BURST-LOOP] Fire queue sampling skipped (no consumers need data)");
            }
            None // No fire data needed
        }; // Assign to shared_fire_data_opt

        // Motor output generation and publishing (per-agent, filtered by subscriptions)
        // NOTE: has_motor_publisher and has_motor_shm already computed above for shared_fire_data_opt

        // CRITICAL: Log motor publisher state every 100 bursts (using INFO to guarantee visibility)
        if burst_num.is_multiple_of(100) {
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
                debug!(
                    "[BURST-LOOP] 🎮 MOTOR: Processing fire data with {} cortical areas",
                    (**fire_data_arc).len()
                );
                
                // CRITICAL PERFORMANCE FIX: Use cached cortical_id mappings (no NPU lock needed!)
                let cortical_id_mappings = cached_cortical_id_mappings.lock().unwrap();
                
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
                    let cortical_id = match cortical_id_mappings.get(area_id) {
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
                                    static WARNED_AREAS_MOTOR: std::sync::LazyLock<std::sync::Mutex<ahash::AHashSet<u32>>> = 
                                        std::sync::LazyLock::new(|| std::sync::Mutex::new(ahash::AHashSet::new()));
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

                    debug!(
                        "[BURST-LOOP] 🎮 MOTOR: Area {} ('{}') has {} neurons firing",
                        area_id,
                        cortical_id.escape_debug(),
                        neuron_ids.len()
                    );

                    motor_snapshot.insert(
                        *area_id,
                        RawFireQueueData {
                            cortical_area_idx: *area_id,
                            cortical_id: cortical_id,
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

                // Get motor subscriptions
                let subscriptions = motor_subscriptions.read();

                // DEBUG: Log subscription state every 30 bursts
                if burst_num.is_multiple_of(30) {
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
                    for (agent_id, subscribed_cortical_ids) in subscriptions.iter() {
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

                                // Publish via ZMQ to agent
                                if let Some(ref publisher) = motor_publisher {
                                    match publisher.publish_motor(agent_id, &motor_bytes) {
                                        Ok(_) => {
                                            // Log every motor send (not just first) for debugging
                                            debug!(
                                                "[BURST-LOOP] ✅ PUBLISHED motor data to agent '{}': {} bytes",
                                                agent_id, motor_bytes.len()
                                            );
                                        }
                                        Err(e) => {
                                            error!(
                                                "[BURST-LOOP] ❌ MOTOR PUBLISH ERROR for '{}': {}",
                                                agent_id, e
                                            );
                                        }
                                    }
                                } else {
                                    info!("[BURST-LOOP] 🎮 Motor publisher not available (None)");
                                }

                                // Write to motor SHM if available (for local agents)
                                if let Some(writer) = motor_shm_writer.lock().unwrap().as_mut() {
                                    if let Err(e) = writer.write_payload(&motor_bytes) {
                                        error!("[BURST-LOOP] ❌ Failed to write motor SHM: {}", e);
                                    }
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
        // Lower threshold to catch smaller slowdowns (10ms instead of 100ms)
        // This will help us identify where the 100-600ms is coming from
        if post_burst_duration.as_millis() > 10 {
            warn!(
                "[BURST-LOOP] ⚠️ Slow post-burst processing: {:.2}ms (viz+motor, burst {})",
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
        let iteration_duration = iteration_start.elapsed();
        if iteration_duration.as_millis() > 100 {
            // BREAKDOWN: Show where time was spent (use stored duration from process_burst)
            // Note: process_burst_duration is only available in the NPU lock scope, so we approximate
            // The actual breakdown will be logged in the next iteration when we have all timings
            warn!(
                "[BURST-LOOP] ⚠️ Slow burst iteration: {:.2}ms total (burst {}) | breakdown: gap_before_post={:.2}ms, post_burst={:.2}ms, stats={:.2}ms, unaccounted={:.2}ms",
                iteration_duration.as_secs_f64() * 1000.0,
                burst_num,
                time_between_npu_release_and_post_burst.as_secs_f64() * 1000.0,
                post_burst_duration.as_secs_f64() * 1000.0,
                stats_duration.as_secs_f64() * 1000.0,
                iteration_duration.as_secs_f64() * 1000.0 - time_between_npu_release_and_post_burst.as_secs_f64() * 1000.0 - post_burst_duration.as_secs_f64() * 1000.0 - stats_duration.as_secs_f64() * 1000.0
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
        let sleep_start = Instant::now();
        let current_frequency_hz = *frequency_hz.lock().unwrap();
        let interval_sec = 1.0 / current_frequency_hz;
        let target_time = burst_start + Duration::from_secs_f64(interval_sec);
        let now = Instant::now();
        
        // Log if we're already past target (iteration took too long)
        if now > target_time {
            let overshoot = now.duration_since(target_time);
            if overshoot.as_millis() > 50 {
                warn!(
                    "[BURST-LOOP] ⚠️ Iteration overshoot: {:.2}ms past target (burst {}) - no sleep needed",
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
            fn publish_raw_fire_queue(
                &self,
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
        let npu = Arc::new(Mutex::new(DynamicNPU::F32(rust_npu)));
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
            fn publish_raw_fire_queue(&self, _fire_data: RawFireQueueSnapshot) -> Result<(), String> {
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
            <crate::RustNPU<StdRuntime, f32, crate::backend::CPUBackend>>::new_cpu_only(100, 1000, 10);

        // Avoid power auto-injection by NOT using cortical_area index 1.
        rust_npu.register_cortical_area(
            2,
            CoreCorticalType::Death.to_cortical_id().as_base_64(),
        );

        let neuron = rust_npu
            .add_neuron(
                1.0,  // threshold
                0.0,  // threshold_limit
                0.0,  // leak_coefficient
                0.0,  // resting_potential
                0,    // neuron_type
                0,    // refractory_period
                1.0,  // excitability
                0,    // consecutive_fire_limit
                0,    // snooze_period
                true, // mp_charge_accumulation
                2,    // cortical_area
                0,
                0,
                0,
            )
            .unwrap();

        // Stage a strong sensory injection so it survives Phase-1 FCL clear and fires on burst 1.
        rust_npu.inject_sensory_with_potentials(&[(neuron, 128.0)]);

        let npu = Arc::new(Mutex::new(DynamicNPU::F32(rust_npu)));
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
            .get(&2)
            .map(|(neuron_ids, _, _, _, _)| neuron_ids.iter().any(|&id| id == neuron.0))
            .unwrap_or(false);

        runner.stop();

        assert!(
            fired_in_area,
            "Expected neuron {} to appear in cached fire queue for cortical_idx=2",
            neuron.0
        );
    }
}
