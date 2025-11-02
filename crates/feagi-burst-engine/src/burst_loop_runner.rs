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

use crate::sensory::AgentManager;
use crate::RustNPU;
use feagi_types::NeuronId;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn, error};

use std::thread;

/// Trait for visualization publishing (abstraction to avoid circular dependency with feagi-pns)
/// Any component that can publish visualization data implements this trait.
pub trait VisualizationPublisher: Send + Sync {
    /// Publish visualization data (LZ4-compressed Type 11 format)
    fn publish_visualization(&self, data: &[u8]) -> Result<(), String>;
}

/// Burst loop runner - manages the main neural processing loop
///
/// 🦀 Power neurons are stored in RustNPU, not here - 100% Rust!
/// 🦀 Burst count is stored in NPU - single source of truth!
pub struct BurstLoopRunner {
    /// Shared NPU instance (holds power neurons internally + burst count)
    npu: Arc<Mutex<RustNPU>>,
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
    /// Uses trait abstraction to avoid circular dependency with feagi-pns
    pub viz_publisher: Option<Arc<dyn VisualizationPublisher>>,
    /// FCL/FQ sampler configuration
    fcl_sampler_frequency: Arc<Mutex<f64>>,  // Sampling frequency in Hz
    fcl_sampler_consumer: Arc<Mutex<u32>>,   // Consumer type: 1=visualization, 2=motor, 3=both
    /// Cached burst count (shared reference to NPU's atomic) for lock-free reads
    cached_burst_count: Arc<std::sync::atomic::AtomicU64>,
}

impl BurstLoopRunner {
    /// Create a new burst loop runner
    ///
    /// # Arguments
    /// * `npu` - The NPU to run bursts on
    /// * `viz_publisher` - Optional visualization publisher (None = no ZMQ visualization)
    /// * `frequency_hz` - Burst frequency in Hz
    pub fn new<P: VisualizationPublisher + 'static>(
        npu: Arc<Mutex<RustNPU>>,
        viz_publisher: Option<Arc<Mutex<P>>>,
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
                info!("🔍 [SENSORY-CALLBACK] Processing {} XYZP data points for area {}", xyzp_data.len(), cortical_area);
                
                // Step 1: Extract coordinates (no locks needed)
                let coords: Vec<(u32, u32, u32)> =
                    xyzp_data.iter().map(|(x, y, z, _)| (*x, *y, *z)).collect();

                // Step 2: Batch lookup with MINIMAL lock time (only neuron_array read lock, NOT full NPU lock)
                let lookup_start = std::time::Instant::now();
                let neuron_ids = if let Ok(npu_lock) = npu_for_callback.lock() {
                    let result = npu_lock
                        .neuron_array
                        .read().unwrap()
                        .batch_coordinate_lookup(cortical_area, &coords);
                    drop(npu_lock); // Release NPU lock ASAP!
                    result
                } else {
                    warn!("[FCL-INJECT] Failed to acquire NPU lock for coordinate lookup");
                    return;
                };
                let lookup_duration = lookup_start.elapsed();
                info!("🔍 [SENSORY-CALLBACK] Batch lookup completed in {:?}", lookup_duration);

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
                info!("🔍 [SENSORY-CALLBACK] Built {} pairs in {:?}", neuron_potential_pairs.len(), pair_duration);

                // 🔍 DEBUG: Log first few potentials
                static FIRST_POTENTIALS_LOGGED: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                if !FIRST_POTENTIALS_LOGGED.load(std::sync::atomic::Ordering::Relaxed)
                    && !neuron_potential_pairs.is_empty()
                {
                    info!("[FCL-INJECT]    First 5 potentials from data:");
                    for (_idx, (neuron_id, p)) in
                        neuron_potential_pairs.iter().take(5).enumerate()
                    {
                        info!("[FCL-INJECT]      [{:?}] p={:.3}", neuron_id, p);
                    }
                    FIRST_POTENTIALS_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
                }

                // Step 4: FINAL injection - acquire lock ONLY for this quick operation
                let inject_start = std::time::Instant::now();
                if let Ok(mut npu_lock) = npu_for_callback.lock() {
                    info!("🔍 [SENSORY-CALLBACK] Acquired NPU lock for injection in {:?}", inject_start.elapsed());
                    npu_lock.inject_sensory_with_potentials(&neuron_potential_pairs);
                    let inject_duration = inject_start.elapsed();
                    info!("🔍 [SENSORY-CALLBACK] Injection completed in {:?}", inject_duration);

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
                info!("🔍 [SENSORY-CALLBACK] Total callback time: {:?}", total_duration);
            },
        );

        let sensory_manager = AgentManager::new(injection_callback);

        // Convert generic publisher to trait object (if provided)
        let viz_publisher_trait: Option<Arc<dyn VisualizationPublisher>> = viz_publisher.map(|p| {
            // Wrap Arc<Mutex<P>> to implement VisualizationPublisher
            struct VisualizerWrapper<P: VisualizationPublisher>(Arc<Mutex<P>>);
            impl<P: VisualizationPublisher> VisualizationPublisher for VisualizerWrapper<P> {
                fn publish_visualization(&self, data: &[u8]) -> Result<(), String> {
                    self.0.lock().unwrap().publish_visualization(data)
                }
            }
            Arc::new(VisualizerWrapper(p)) as Arc<dyn VisualizationPublisher>
        });

        Self {
            npu,
            frequency_hz: Arc::new(Mutex::new(frequency_hz)),  // Shared with burst thread for dynamic updates
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            sensory_manager: Arc::new(Mutex::new(sensory_manager)),
            viz_shm_writer: Arc::new(Mutex::new(None)), // Initialized later via attach_viz_shm_writer
            motor_shm_writer: Arc::new(Mutex::new(None)), // Initialized later via attach_motor_shm_writer
            viz_publisher: viz_publisher_trait, // Trait object for visualization (NO PYTHON CALLBACKS!)
            fcl_sampler_frequency: Arc::new(Mutex::new(30.0)), // Default 30Hz for visualization
            fcl_sampler_consumer: Arc::new(Mutex::new(1)), // Default: 1 = visualization only
            cached_burst_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
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
        let frequency = self.frequency_hz.clone();  // Clone Arc for thread
        let running = self.running.clone();
        let viz_writer = self.viz_shm_writer.clone();
        let viz_publisher = self.viz_publisher.clone(); // Direct Rust-to-Rust trait reference (NO PYTHON CALLBACKS!)
        let cached_burst_count = self.cached_burst_count.clone(); // For lock-free burst count reads

        self.thread_handle = Some(
            thread::Builder::new()
                .name("feagi-burst-loop".to_string())
                .spawn(move || {
                    burst_loop(npu, frequency, running, viz_writer, viz_publisher, cached_burst_count);
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
        self.cached_burst_count.load(std::sync::atomic::Ordering::Relaxed)
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
    /// Returns the last sampled fire queue data
    pub fn get_fire_queue_sample(&mut self) -> Option<ahash::AHashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>> {
        self.npu.lock().unwrap().sample_fire_queue()
    }
    
    /// Get Fire Ledger window configurations for all cortical areas
    pub fn get_fire_ledger_configs(&self) -> Vec<(u32, usize)> {
        self.npu.lock().unwrap().get_all_fire_ledger_configs()
    }
    
    /// Configure Fire Ledger window size for a specific cortical area
    pub fn configure_fire_ledger_window(&mut self, cortical_idx: u32, window_size: usize) {
        self.npu.lock().unwrap().configure_fire_ledger_window(cortical_idx, window_size);
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
    pub fn get_npu(&self) -> Arc<Mutex<RustNPU>> {
        self.npu.clone()
    }
}

impl Drop for BurstLoopRunner {
    fn drop(&mut self) {
        self.stop();
    }
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
fn burst_loop(
    npu: Arc<Mutex<RustNPU>>,
    frequency_hz: Arc<Mutex<f64>>,  // Shared frequency - can be updated while running
    running: Arc<AtomicBool>,
    viz_shm_writer: Arc<Mutex<Option<crate::viz_shm_writer::VizSHMWriter>>>,
    viz_publisher: Option<Arc<dyn VisualizationPublisher>>, // Trait object for visualization (NO PYTHON CALLBACKS!)
    cached_burst_count: Arc<std::sync::atomic::AtomicU64>, // For lock-free burst count reads
) {
    let timestamp = get_timestamp();
    let initial_freq = *frequency_hz.lock().unwrap();
    info!(
        "[{}] [BURST-LOOP] 🚀 Starting main loop at {:.2} Hz",
        timestamp, initial_freq
    );

    let mut burst_num = 0u64;
    let mut last_stats_time = Instant::now();
    let mut total_neurons_fired = 0usize;
    let mut burst_times = Vec::with_capacity(100);
    let mut last_burst_time = None;

    while running.load(Ordering::Acquire) {
        let burst_start = Instant::now();

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
        info!("🔍 [BURST-TIMING] Attempting to acquire NPU lock for burst...");
        
        let should_exit = {
            let npu_lock = npu.lock().unwrap();
            let lock_acquired = Instant::now();
            info!("🔍 [BURST-TIMING] NPU lock acquired in {:?}", lock_acquired.duration_since(lock_start));
            
            // Check flag again after acquiring lock (in case shutdown happened during lock wait)
            if !running.load(Ordering::Relaxed) {
                true // Signal to exit
            } else {
                let process_start = Instant::now();
                info!("🔍 [BURST-TIMING] Starting process_burst()...");
                
                match npu_lock.process_burst() {
                    Ok(result) => {
                        let process_done = Instant::now();
                        info!("🔍 [BURST-TIMING] process_burst() completed in {:?}, {} neurons fired", 
                            process_done.duration_since(process_start), result.neuron_count);
                        
                        total_neurons_fired += result.neuron_count;
                        // Update cached burst count for lock-free reads
                        cached_burst_count.store(npu_lock.get_burst_count(), std::sync::atomic::Ordering::Relaxed);
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
        
        // Exit if shutdown was requested
        if should_exit || !running.load(Ordering::Relaxed) {
            break;
        }

        burst_num += 1;
        // Note: NPU.process_burst() already incremented its internal burst_count

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

        if has_shm_writer || has_viz_publisher {
            // Force sample FQ on every burst (bypasses rate limiting)
            let viz_lock_start = Instant::now();
            info!("🔍 [BURST-TIMING] Attempting to acquire NPU lock for viz sampling...");
            let fire_data_opt = npu.lock().unwrap().force_sample_fire_queue();
            let viz_lock_done = Instant::now();
            info!("🔍 [BURST-TIMING] Viz sampling completed in {:?}", viz_lock_done.duration_since(viz_lock_start));

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

            if let Some(fire_data) = fire_data_opt {
                // Debug: Log first successful viz write (for migration debugging)
                // Warning about unused static is expected - kept for development debugging
                #[allow(dead_code)]  // In development - used for debugging first viz write
                static FIRST_VIZ_WRITE_LOGGED: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);

                // Encode as raw Type 11 structure (no container wrapper, like rust-py-libs pattern)
                use feagi_data_serialization::FeagiSerializable;
                use feagi_data_structures::genomic::CorticalID;
                use feagi_data_structures::neuron_voxels::xyzp::{
                    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
                };

                // Convert FQ data to CorticalMappedXYZPNeuronVoxels
                let mut cortical_mapped = CorticalMappedXYZPNeuronVoxels::new();
                let mut _total_neurons = 0;

                // 🚀 ZERO-COPY OPTIMIZATION: Consume fire_data to avoid cloning vectors
                // This eliminates ~1 MB allocation per burst @ 10 Hz = ~10 MB/sec saved
                for (area_id, (_id_vec, x_vec, y_vec, z_vec, p_vec)) in fire_data {
                    _total_neurons += _id_vec.len();

                    // Debug: Log what areas we're encoding
                    static FIRST_AREAS_LOGGED: std::sync::atomic::AtomicBool =
                        std::sync::atomic::AtomicBool::new(false);
                    if !FIRST_AREAS_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                        let area_name = npu
                            .lock()
                            .unwrap()
                            .get_cortical_area_name(area_id)
                            .map(|s| s.to_string());
                        debug!(
                            "[BURST-LOOP] 🔍 Area {} ({}): {} neurons",
                            area_id,
                            area_name.as_deref().unwrap_or("unknown"),
                            _id_vec.len()
                        );
                        FIRST_AREAS_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
                    }

                    // Get cortical area name from NPU mapping
                    let cortical_name_opt = npu
                        .lock()
                        .unwrap()
                        .get_cortical_area_name(area_id)
                        .map(|s| s.to_string());
                    let cortical_id = match cortical_name_opt {
                        Some(name) => {
                            let mut bytes = [b' '; 6];
                            let name_bytes = name.as_bytes();
                            let copy_len = name_bytes.len().min(6);
                            bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

                            match CorticalID::from_bytes(&bytes) {
                                Ok(id) => id,
                                Err(e) => {
                                    error!("[BURST-LOOP] ❌ Failed to create CorticalID for '{}': {:?}", name, e);
                                    continue;
                                }
                            }
                        }
                        None => {
                            error!(
                                "[BURST-LOOP] ❌ No cortical area name registered for area_id {}",
                                area_id
                            );
                            continue;
                        }
                    };

                    // MOVED: Transfer ownership instead of cloning (zero-copy)
                    match NeuronVoxelXYZPArrays::new_from_vectors(
                        x_vec,  // ✅ MOVE (no clone)
                        y_vec,  // ✅ MOVE (no clone)
                        z_vec,  // ✅ MOVE (no clone)
                        p_vec,  // ✅ MOVE (no clone)
                    ) {
                        Ok(neuron_arrays) => {
                            cortical_mapped.insert(cortical_id, neuron_arrays);
                        }
                        Err(e) => {
                            error!("[BURST-LOOP] ❌ Failed to create neuron arrays: {:?}", e);
                            continue;
                        }
                    }
                }

                if cortical_mapped.len() > 0 {
                    // Use raw FeagiSerializable API (like rust-py-libs pattern)
                    let bytes_needed = cortical_mapped.get_number_of_bytes_needed();
                    let mut buffer = vec![0u8; bytes_needed];

                    static FIRST_SIZE_LOG: std::sync::atomic::AtomicBool =
                        std::sync::atomic::AtomicBool::new(false);
                    if !FIRST_SIZE_LOG.load(std::sync::atomic::Ordering::Relaxed) {
                        debug!(
                            "[BURST-LOOP] 🔍 SERIALIZE: bytes_needed={}, buffer.len()={}",
                            bytes_needed,
                            buffer.len()
                        );
                        FIRST_SIZE_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
                    }

                    if let Ok(_) = cortical_mapped.try_serialize_struct_to_byte_slice(&mut buffer) {
                        static FIRST_ACTUAL_SIZE_LOG: std::sync::atomic::AtomicBool =
                            std::sync::atomic::AtomicBool::new(false);
                        if !FIRST_ACTUAL_SIZE_LOG.load(std::sync::atomic::Ordering::Relaxed) {
                            debug!("[BURST-LOOP] 🔍 SERIALIZE: Actual serialized buffer.len()={}, first 8 bytes: {:02x?}", buffer.len(), &buffer[..8.min(buffer.len())]);
                            FIRST_ACTUAL_SIZE_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                        // Write to SHM if writer is attached (uncompressed - local IPC)
                        if has_shm_writer {
                            let mut viz_writer_lock = viz_shm_writer.lock().unwrap();
                            if let Some(writer) = viz_writer_lock.as_mut() {
                                match writer.write_payload(&buffer) {
                                    Ok(_) => {
                                        // Visualization data written successfully to SHM
                                    }
                                    Err(e) => {
                                        let timestamp = get_timestamp();
                                        error!(
                                            "[{}] [BURST-LOOP] ❌ Failed to write viz SHM: {}",
                                            timestamp, e
                                        );
                                    }
                                }
                            }
                        }

                        // Publish to ZMQ via trait object (direct Rust-to-Rust call, NO PYTHON!)
                        // Publisher (PNS) handles LZ4 compression and ZMQ publishing
                        if let Some(ref publisher) = viz_publisher {
                            static FIRST_PUBLISH_LOGGED: std::sync::atomic::AtomicBool =
                                std::sync::atomic::AtomicBool::new(false);
                            if !FIRST_PUBLISH_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                                debug!("[BURST-LOOP] First visualization publish: {} bytes", buffer.len());
                                FIRST_PUBLISH_LOGGED
                                    .store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            
                            // DIAGNOSTIC: Track publish rate and data volume
                            static PUBLISH_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                            
                            let count = PUBLISH_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if count % 30 == 0 {  // Log every 30 publishes (~1 second at 30Hz)
                                debug!("[BURST-LOOP] Viz publish #{}: {} bytes/frame", count, buffer.len());
                            }

                            match publisher.publish_visualization(&buffer) {
                                Ok(_) => {
                                    static SUCCESS_LOGGED: std::sync::atomic::AtomicBool =
                                        std::sync::atomic::AtomicBool::new(false);
                                    if !SUCCESS_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                                        debug!("[BURST-LOOP] Visualization publisher connected and working");
                                        SUCCESS_LOGGED
                                            .store(true, std::sync::atomic::Ordering::Relaxed);
                                    }
                                }
                                Err(e) => {
                                    error!("[BURST-LOOP] ❌ VIZ PUBLISH ERROR: {}", e);
                                }
                            }
                        } else {
                            static NO_PUBLISHER_LOGGED: std::sync::atomic::AtomicBool =
                                std::sync::atomic::AtomicBool::new(false);
                            if !NO_PUBLISHER_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                                error!("[BURST-LOOP] ⚠️ CRITICAL: viz_publisher is None!");
                                NO_PUBLISHER_LOGGED
                                    .store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    } else {
                        static SERIALIZE_ERROR_LOGGED: std::sync::atomic::AtomicBool =
                            std::sync::atomic::AtomicBool::new(false);
                        if !SERIALIZE_ERROR_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                            let timestamp = get_timestamp();
                            error!(
                                "[{}] [BURST-LOOP] ❌ Failed to serialize - skipping viz writes",
                                timestamp
                            );
                            SERIALIZE_ERROR_LOGGED
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                } else {
                    static SKIP_LOGGED: std::sync::atomic::AtomicBool =
                        std::sync::atomic::AtomicBool::new(false);
                    if !SKIP_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                        debug!("[BURST-LOOP] Skipping visualization - no cortical areas yet (genome not loaded)");
                        SKIP_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            } // Close if let Some(fire_data)
        } // Close visualization block

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

        // CRITICAL: Check shutdown flag before entering sleep
        // Exit immediately if shutdown was requested during visualization/stats
        if !running.load(Ordering::Relaxed) {
            break;
        }

        // Adaptive sleep (RTOS-friendly timing)
        // Strategy: <5Hz = chunked sleep, 5-100Hz = hybrid, >100Hz = busy-wait
        // CRITICAL: Break sleep into chunks to allow responsive shutdown
        // Maximum sleep chunk: 50ms to ensure shutdown responds within ~50ms
        // CRITICAL: Read frequency dynamically to allow runtime updates
        let current_frequency_hz = *frequency_hz.lock().unwrap();
        let interval_sec = 1.0 / current_frequency_hz;
        let target_time = burst_start + Duration::from_secs_f64(interval_sec);
        let now = Instant::now();

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
        // Use a unit type for the generic parameter since we're not testing visualization
        struct NoViz;
        impl VisualizationPublisher for NoViz {
            fn publish_visualization(&self, _data: &[u8]) -> Result<(), String> {
                Ok(())
            }
        }
        
        let npu = Arc::new(Mutex::new(RustNPU::new_cpu_only(1000, 10000, 20)));
        let mut runner = BurstLoopRunner::new::<NoViz>(npu, None, 10.0);

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
}
