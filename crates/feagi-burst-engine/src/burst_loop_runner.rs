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
    /// Target frequency in Hz
    frequency_hz: f64,
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
                    println!(
                        "[FCL-INJECT] 🔍 First callback: cortical_area={}, neuron_count={}",
                        cortical_area,
                        xyzp_data.len()
                    );
                    println!(
                        "[FCL-INJECT]    First 3 XYZP: {:?}",
                        &xyzp_data[0..xyzp_data.len().min(3)]
                    );
                    FIRST_CALLBACK_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
                }

                // Convert (x,y,z) to neuron IDs and inject with actual P values
                if let Ok(mut npu_lock) = npu_for_callback.lock() {
                    // Extract coordinates for batch lookup
                    let coords: Vec<(u32, u32, u32)> =
                        xyzp_data.iter().map(|(x, y, z, _)| (*x, *y, *z)).collect();

                    // Batch coordinate lookup
                    let neuron_ids = npu_lock
                        .neuron_array
                        .read().unwrap()
                        .batch_coordinate_lookup(cortical_area, &coords);

                    // 🔍 DEBUG: Log conversion result
                    static FIRST_CONVERSION_LOGGED: std::sync::atomic::AtomicBool =
                        std::sync::atomic::AtomicBool::new(false);
                    if !FIRST_CONVERSION_LOGGED.load(std::sync::atomic::Ordering::Relaxed)
                        && !neuron_ids.is_empty()
                    {
                        println!(
                            "[FCL-INJECT]    Converted {} coords → {} valid neurons",
                            xyzp_data.len(),
                            neuron_ids.len()
                        );
                        println!(
                            "[FCL-INJECT]    First 5 neuron IDs: {:?}",
                            &neuron_ids[0..neuron_ids.len().min(5)]
                        );
                        FIRST_CONVERSION_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
                    }

                    // Build (NeuronId, potential) pairs from XYZP data
                    let mut neuron_potential_pairs: Vec<(NeuronId, f32)> =
                        Vec::with_capacity(xyzp_data.len());
                    for (x, y, z, p) in xyzp_data.iter() {
                        if let Some(neuron_id) = npu_lock.neuron_array.read().unwrap().get_neuron_at_coordinate(
                            cortical_area,
                            *x,
                            *y,
                            *z,
                        ) {
                            neuron_potential_pairs.push((neuron_id, *p));
                        }
                    }

                    // 🔍 DEBUG: Log first few potentials
                    static FIRST_POTENTIALS_LOGGED: std::sync::atomic::AtomicBool =
                        std::sync::atomic::AtomicBool::new(false);
                    if !FIRST_POTENTIALS_LOGGED.load(std::sync::atomic::Ordering::Relaxed)
                        && !neuron_potential_pairs.is_empty()
                    {
                        println!("[FCL-INJECT]    First 5 potentials from data:");
                        for (_idx, (neuron_id, p)) in
                            neuron_potential_pairs.iter().take(5).enumerate()
                        {
                            println!("[FCL-INJECT]      [{:?}] p={:.3}", neuron_id, p);
                        }
                        FIRST_POTENTIALS_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
                    }

                    // Inject with individual potentials
                    npu_lock.inject_sensory_with_potentials(&neuron_potential_pairs);

                    // 🔍 DEBUG: Log injection summary
                    static FIRST_SUMMARY_LOGGED: std::sync::atomic::AtomicBool =
                        std::sync::atomic::AtomicBool::new(false);
                    if !FIRST_SUMMARY_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                        println!(
                            "[FCL-INJECT]    ✅ Injected {} neurons with actual P values from data",
                            neuron_potential_pairs.len()
                        );
                        FIRST_SUMMARY_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                }
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
            frequency_hz,
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            sensory_manager: Arc::new(Mutex::new(sensory_manager)),
            viz_shm_writer: Arc::new(Mutex::new(None)), // Initialized later via attach_viz_shm_writer
            motor_shm_writer: Arc::new(Mutex::new(None)), // Initialized later via attach_motor_shm_writer
            viz_publisher: viz_publisher_trait, // Trait object for visualization (NO PYTHON CALLBACKS!)
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

    /// Set burst frequency (can be called while running)
    pub fn set_frequency(&mut self, frequency_hz: f64) {
        self.frequency_hz = frequency_hz;
        println!("[BURST-RUNNER] Frequency set to {:.2} Hz", frequency_hz);
    }

    /// Start the burst loop in a background thread
    ///
    /// 🦀 Power neurons are read from RustNPU internally - 100% Rust!
    pub fn start(&mut self) -> Result<(), String> {
        if self.running.load(Ordering::Acquire) {
            return Err("Burst loop already running".to_string());
        }

        println!("[BURST-RUNNER] Starting burst loop at {:.2} Hz (power neurons auto-discovered from cortical_idx=1)", 
                 self.frequency_hz);

        self.running.store(true, Ordering::Release);

        let npu = self.npu.clone();
        let frequency = self.frequency_hz;
        let running = self.running.clone();
        let viz_writer = self.viz_shm_writer.clone();
        let viz_publisher = self.viz_publisher.clone(); // Direct Rust-to-Rust trait reference (NO PYTHON CALLBACKS!)

        self.thread_handle = Some(
            thread::Builder::new()
                .name("feagi-burst-loop".to_string())
                .spawn(move || {
                    burst_loop(npu, frequency, running, viz_writer, viz_publisher);
                })
                .map_err(|e| format!("Failed to spawn burst loop thread: {}", e))?,
        );

        println!("[BURST-RUNNER] ✅ Burst loop started successfully");
        Ok(())
    }

    /// Stop the burst loop gracefully
    pub fn stop(&mut self) {
        if !self.running.load(Ordering::Acquire) {
            return; // Already stopped
        }

        println!("[BURST-RUNNER] Stopping burst loop...");
        self.running.store(false, Ordering::Release);

        if let Some(handle) = self.thread_handle.take() {
            if handle.join().is_err() {
                eprintln!("[BURST-RUNNER] ⚠️ Burst loop thread panicked during shutdown");
            } else {
                println!("[BURST-RUNNER] ✅ Burst loop stopped cleanly");
            }
        }
    }

    /// Check if the burst loop is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    /// Get current burst count (from NPU - single source of truth)
    pub fn get_burst_count(&self) -> u64 {
        self.npu.lock().unwrap().get_burst_count()
    }

    /// Get configured burst frequency in Hz
    pub fn get_frequency(&self) -> f64 {
        self.frequency_hz
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
    frequency_hz: f64,
    running: Arc<AtomicBool>,
    viz_shm_writer: Arc<Mutex<Option<crate::viz_shm_writer::VizSHMWriter>>>,
    viz_publisher: Option<Arc<dyn VisualizationPublisher>>, // Trait object for visualization (NO PYTHON CALLBACKS!)
) {
    let timestamp = get_timestamp();
    println!(
        "[{}] [BURST-LOOP] 🚀 Starting main loop at {:.2} Hz",
        timestamp, frequency_hz
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
        let _fired_count = {
            let mut npu_lock = npu.lock().unwrap();
            match npu_lock.process_burst() {
                Ok(result) => {
                    total_neurons_fired += result.neuron_count;
                    result.neuron_count
                }
                Err(e) => {
                    let timestamp = get_timestamp();
                    eprintln!(
                        "[{}] [BURST-LOOP] ❌ Burst processing error: {}",
                        timestamp, e
                    );
                    0
                }
            }
        }; // Drop lock immediately

        // CRITICAL: Check shutdown flag immediately after burst processing
        // Exit early if shutdown was requested during burst processing
        if !running.load(Ordering::Relaxed) {
            break;
        }

        burst_num += 1;
        // Note: NPU.process_burst() already incremented its internal burst_count

        // Write visualization data (SHM and/or PNS ZMQ)
        // Check if we need to do ANY visualization (SHM writer OR viz publisher)
        let has_shm_writer = viz_shm_writer.lock().unwrap().is_some();
        let has_viz_publisher = viz_publisher.is_some();

        // 🔍 CRITICAL DEBUG: Log what's available
        static DEBUG_LOGGED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        if !DEBUG_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
            eprintln!(
                "[BURST-LOOP] 🔍 CRITICAL: has_shm_writer={}, has_viz_publisher={}",
                has_shm_writer, has_viz_publisher
            );
            DEBUG_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        if has_shm_writer || has_viz_publisher {
            // Force sample FQ on every burst (bypasses rate limiting)
            let fire_data_opt = npu.lock().unwrap().force_sample_fire_queue();

            static FIRST_CHECK_LOGGED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !FIRST_CHECK_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                if has_shm_writer {
                    println!("[BURST-LOOP] 🔍 Viz SHM writer is attached");
                }
                if has_viz_publisher {
                    println!("[BURST-LOOP] 🔍 Visualization publisher is attached (Rust-to-Rust, NO PYTHON!)");
                }
                FIRST_CHECK_LOGGED.store(true, std::sync::atomic::Ordering::Relaxed);
            }

            if let Some(fire_data) = fire_data_opt {
                // Debug: Log first successful viz write (for migration debugging)
                // Warning about unused static is expected - kept for development debugging
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
                        eprintln!(
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
                                    eprintln!("[BURST-LOOP] ❌ Failed to create CorticalID for '{}': {:?}", name, e);
                                    continue;
                                }
                            }
                        }
                        None => {
                            eprintln!(
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
                            eprintln!("[BURST-LOOP] ❌ Failed to create neuron arrays: {:?}", e);
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
                        eprintln!(
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
                            eprintln!("[BURST-LOOP] 🔍 SERIALIZE: Actual serialized buffer.len()={}, first 8 bytes: {:02x?}", buffer.len(), &buffer[..8.min(buffer.len())]);
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
                                        eprintln!(
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
                                eprintln!("[BURST-LOOP] 🔍 CRITICAL: Calling publisher.publish_visualization() with {} bytes", buffer.len());
                                FIRST_PUBLISH_LOGGED
                                    .store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            
                            // DIAGNOSTIC: Track publish rate and data volume
                            static PUBLISH_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                            
                            let count = PUBLISH_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if count % 30 == 0 {  // Log every 30 publishes (~1 second at 30Hz)
                                eprintln!("[BURST-LOOP] 📊 VIZ PUBLISH #{}: {} bytes/frame", count, buffer.len());
                            }

                            match publisher.publish_visualization(&buffer) {
                                Ok(_) => {
                                    static SUCCESS_LOGGED: std::sync::atomic::AtomicBool =
                                        std::sync::atomic::AtomicBool::new(false);
                                    if !SUCCESS_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                                        eprintln!("[BURST-LOOP] ✅ CRITICAL: publish_visualization() SUCCESS!");
                                        SUCCESS_LOGGED
                                            .store(true, std::sync::atomic::Ordering::Relaxed);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[BURST-LOOP] ❌ VIZ PUBLISH ERROR: {}", e);
                                }
                            }
                        } else {
                            static NO_PUBLISHER_LOGGED: std::sync::atomic::AtomicBool =
                                std::sync::atomic::AtomicBool::new(false);
                            if !NO_PUBLISHER_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                                eprintln!("[BURST-LOOP] ⚠️ CRITICAL: viz_publisher is None!");
                                NO_PUBLISHER_LOGGED
                                    .store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    } else {
                        static SERIALIZE_ERROR_LOGGED: std::sync::atomic::AtomicBool =
                            std::sync::atomic::AtomicBool::new(false);
                        if !SERIALIZE_ERROR_LOGGED.load(std::sync::atomic::Ordering::Relaxed) {
                            let timestamp = get_timestamp();
                            eprintln!(
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
                        eprintln!("[BURST-LOOP] ⚠️ Skipping viz - no cortical areas yet");
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
                let timestamp = get_timestamp();

                println!(
                    "[{}] [BURST-LOOP] 📊 Stats: Burst #{} | Desired: {:.2} Hz | Actual: {:.2} Hz ({:.1}%) | Avg neurons: {}",
                    timestamp,
                    burst_num,
                    frequency_hz,
                    actual_hz,
                    (actual_hz / frequency_hz * 100.0),
                    avg_neurons
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
        let interval_sec = 1.0 / frequency_hz;
        let target_time = burst_start + Duration::from_secs_f64(interval_sec);
        let now = Instant::now();

        if now < target_time {
            let remaining = target_time - now;

            if frequency_hz < 5.0 {
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
            } else if frequency_hz > 100.0 {
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
    println!(
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
        let npu = Arc::new(Mutex::new(RustNPU::new(1000, 10000, 20)));
        let mut runner = BurstLoopRunner::new(npu, 10.0);

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
