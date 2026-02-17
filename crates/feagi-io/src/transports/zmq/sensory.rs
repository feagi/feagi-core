// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Sensory stream for receiving sensory data from agents
// Uses PULL socket pattern for receiving data from multiple agents (agents use PUSH)

use feagi_structures::FeagiDataError;
use futures_util::FutureExt;
use parking_lot::{Mutex, RwLock};
use std::future::Future;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Handle;
use tokio::runtime::Runtime;
use tokio::task::block_in_place;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use zeromq::{PullSocket, Socket, SocketRecv};

fn block_on_runtime<T>(runtime: &Runtime, future: impl Future<Output = T>) -> T {
    if Handle::try_current().is_ok() {
        block_in_place(|| Handle::current().block_on(future))
    } else {
        runtime.block_on(future)
    }
}

/// Runtime configuration for the ZMQ sensory receiver.
#[derive(Clone, Debug)]
pub struct SensoryReceiveConfig {
    pub receive_high_water_mark: i32,
    pub linger_ms: i32,
    pub immediate: bool,
    pub poll_timeout_ms: i64,
    /// Duration in milliseconds to drain stale messages on startup
    /// Real-time systems MUST discard buffered sensory data from previous sessions
    pub startup_drain_timeout_ms: u64,
}

impl Default for SensoryReceiveConfig {
    fn default() -> Self {
        Self {
            // REAL-TIME: HWM=1 ensures only latest sensory data is kept
            // Old data is automatically dropped by ZMQ if not consumed fast enough
            receive_high_water_mark: 1,
            linger_ms: 0,
            // REAL-TIME: immediate=true disables Nagle's algorithm for lowest latency
            immediate: true,
            poll_timeout_ms: 1000,
            startup_drain_timeout_ms: 500, // 500ms drain on startup
        }
    }
}

impl SensoryReceiveConfig {
    pub fn validate(&self) -> Result<(), FeagiDataError> {
        if self.receive_high_water_mark < 0 {
            return Err(FeagiDataError::BadParameters(
                "receive_high_water_mark must be >= 0".to_string(),
            ));
        }
        if self.poll_timeout_ms < 0 {
            return Err(FeagiDataError::BadParameters(
                "poll_timeout_ms must be >= 0".to_string(),
            ));
        }
        if self.startup_drain_timeout_ms > 10000 {
            return Err(FeagiDataError::BadParameters(
                "startup_drain_timeout_ms must be <= 10000ms (10 seconds)".to_string(),
            ));
        }
        Ok(())
    }
}

/// Sensory stream for receiving sensory data from agents
#[derive(Clone)]
pub struct SensoryStream {
    runtime: Arc<Runtime>,
    bind_address: String,
    socket: Arc<Mutex<Option<PullSocket>>>,
    running: Arc<Mutex<bool>>,
    config: SensoryReceiveConfig,
    /// Reference to Rust NPU for direct injection (no FFI overhead!)
    npu: Arc<
        Mutex<
            Option<Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>>,
        >,
    >,
    /// Reference to AgentRegistry for security gating
    agent_registry: Arc<Mutex<Option<Arc<RwLock<crate::core::AgentRegistry>>>>>,
    /// Statistics
    total_messages: Arc<Mutex<u64>>,
    total_neurons: Arc<Mutex<u64>>,
    /// Security stats (rejected messages)
    rejected_no_genome: Arc<Mutex<u64>>,
    rejected_no_agents: Arc<Mutex<u64>>,
}

impl SensoryStream {
    /// Create a new sensory stream
    pub fn new(
        runtime: Arc<Runtime>,
        bind_address: &str,
        config: SensoryReceiveConfig,
    ) -> Result<Self, FeagiDataError> {
        config.validate()?;
        Ok(Self {
            runtime,
            bind_address: bind_address.to_string(),
            socket: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            config,
            npu: Arc::new(Mutex::new(None)),
            agent_registry: Arc::new(Mutex::new(None)),
            total_messages: Arc::new(Mutex::new(0)),
            total_neurons: Arc::new(Mutex::new(0)),
            rejected_no_genome: Arc::new(Mutex::new(0)),
            rejected_no_agents: Arc::new(Mutex::new(0)),
        })
    }

    /// Set the Rust NPU reference for direct injection
    pub fn set_npu(
        &self,
        npu: Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
    ) {
        *self.npu.lock() = Some(npu);
        info!("🦀 [SENSORY-STREAM] NPU connected for direct injection");
    }

    /// Set the AgentRegistry reference for security gating
    pub fn set_agent_registry(&self, registry: Arc<RwLock<crate::core::AgentRegistry>>) {
        *self.agent_registry.lock() = Some(registry);
        info!("🦀 [SENSORY-STREAM] AgentRegistry connected for security gating");
    }

    /// Start the sensory stream
    pub fn start(&self) -> Result<(), FeagiDataError> {
        if *self.running.lock() {
            return Err(FeagiDataError::BadParameters(
                "Sensory stream already running".to_string(),
            ));
        }

        let defaults = SensoryReceiveConfig::default();
        if self.config.receive_high_water_mark != defaults.receive_high_water_mark
            || self.config.linger_ms != defaults.linger_ms
            || self.config.immediate != defaults.immediate
        {
            return Err(FeagiDataError::BadParameters(format!(
                "zeromq transport does not support custom socket options (receive_high_water_mark={}, linger_ms={}, immediate={})",
                self.config.receive_high_water_mark,
                self.config.linger_ms,
                self.config.immediate
            )));
        }

        // Create PULL socket for receiving sensory data
        let mut socket = PullSocket::new();
        block_on_runtime(self.runtime.as_ref(), socket.bind(&self.bind_address))
            .map_err(|e| super::bind_error_to_feagi_error(&self.bind_address, e))?;

        *self.socket.lock() = Some(socket);
        *self.running.lock() = true;

        info!("🦀 [ZMQ-SENSORY] ✅ Listening on {}", self.bind_address);

        // CRITICAL: Drain stale buffered messages before processing real-time data
        // Real-time systems must discard residual sensory data from previous sessions
        self.drain_stale_messages();

        // Start processing loop
        self.start_processing_loop();

        Ok(())
    }

    /// Stop the sensory stream
    pub fn stop(&self) -> Result<(), FeagiDataError> {
        *self.running.lock() = false;

        // Log final statistics
        let total_msg = *self.total_messages.lock();
        let total_neurons = *self.total_neurons.lock();
        info!(
            "🦀 [ZMQ-SENSORY] Stopped. Total: {} messages, {} neurons",
            total_msg, total_neurons
        );

        *self.socket.lock() = None;
        Ok(())
    }

    /// Drain all stale buffered messages on startup (real-time requirement)
    ///
    /// CRITICAL for real-time systems: Residual sensory data from previous sessions
    /// or disconnected agents is garbage and must be discarded before processing
    /// begins. This method drains the ZMQ receive buffer using non-blocking reads
    /// until the configured timeout expires.
    fn drain_stale_messages(&self) {
        let drain_start = std::time::Instant::now();
        let drain_timeout = std::time::Duration::from_millis(self.config.startup_drain_timeout_ms);
        let mut drained_count = 0u64;

        info!(
            "🦀 [ZMQ-SENSORY] 🗑️  Draining stale messages (timeout: {}ms)...",
            self.config.startup_drain_timeout_ms
        );

        let mut sock_guard = self.socket.lock();
        let sock = match sock_guard.as_mut() {
            Some(s) => s,
            None => {
                warn!("🦀 [ZMQ-SENSORY] [ERR] Cannot drain - socket not initialized");
                return;
            }
        };

        // Drain loop: read and discard all buffered messages until timeout
        loop {
            // Check timeout
            if drain_start.elapsed() >= drain_timeout {
                break;
            }

            let result =
                block_on_runtime(self.runtime.as_ref(), async { sock.recv().now_or_never() });
            match result {
                None => break,
                Some(Ok(_)) => {
                    drained_count += 1;
                }
                Some(Err(e)) => {
                    error!("🦀 [ZMQ-SENSORY] [ERR] Drain error: {}", e);
                    break;
                }
            }
        }

        drop(sock_guard);

        if drained_count > 0 {
            info!(
                "🦀 [ZMQ-SENSORY] 🗑️  Drained {} stale messages ({:.1}ms)",
                drained_count,
                drain_start.elapsed().as_secs_f64() * 1000.0
            );
        } else {
            info!("🦀 [ZMQ-SENSORY] ✅ No stale messages found (buffer was clean)");
        }
    }

    /// Start the background processing loop
    fn start_processing_loop(&self) {
        let socket = Arc::clone(&self.socket);
        let running = Arc::clone(&self.running);
        let npu = Arc::clone(&self.npu);
        let agent_registry = Arc::clone(&self.agent_registry);
        let total_messages = Arc::clone(&self.total_messages);
        let total_neurons = Arc::clone(&self.total_neurons);
        let rejected_no_genome = Arc::clone(&self.rejected_no_genome);
        let rejected_no_agents = Arc::clone(&self.rejected_no_agents);
        let config = self.config.clone();
        let runtime = Arc::clone(&self.runtime);

        thread::spawn(move || {
            info!("🦀 [ZMQ-SENSORY] Processing loop started");

            let mut message_count = 0u64;

            while *running.lock() {
                let mut sock_guard = socket.lock();
                let sock = match sock_guard.as_mut() {
                    Some(s) => s,
                    None => {
                        drop(sock_guard);
                        thread::sleep(std::time::Duration::from_millis(100));
                        continue;
                    }
                };

                let recv_result = block_on_runtime(runtime.as_ref(), async {
                    timeout(
                        std::time::Duration::from_millis(config.poll_timeout_ms as u64),
                        sock.recv(),
                    )
                    .await
                });

                let message = match recv_result {
                    Ok(Ok(message)) => message,
                    Ok(Err(e)) => {
                        error!("🦀 [ZMQ-SENSORY] [ERR] Receive error: {}", e);
                        drop(sock_guard);
                        continue;
                    }
                    Err(_) => {
                        if message_count == 0 || message_count % 1000 == 0 {
                            debug!(
                                "🦀 [ZMQ-SENSORY] 🔍 Polling for messages (no data yet, message_count: {})",
                                message_count
                            );
                        }
                        drop(sock_guard);
                        continue;
                    }
                };

                let mut newest_bytes = match message.into_vec().into_iter().next() {
                    Some(frame) => frame.to_vec(),
                    None => {
                        drop(sock_guard);
                        continue;
                    }
                };

                let mut drained_newer: u64 = 0;
                loop {
                    let drain_result =
                        block_on_runtime(runtime.as_ref(), async { sock.recv().now_or_never() });
                    match drain_result {
                        Some(Ok(next_message)) => {
                            if let Some(frame) = next_message.into_vec().into_iter().next() {
                                newest_bytes = frame.to_vec();
                                drained_newer += 1;
                            }
                        }
                        Some(Err(e)) => {
                            warn!("🦀 [ZMQ-SENSORY] [WARN] Drain recv error: {}", e);
                            break;
                        }
                        None => break,
                    }
                }

                drop(sock_guard); // Release lock before processing

                *total_messages.lock() += 1;
                message_count += 1;

                // Process the binary data
                let message_bytes: &[u8] = newest_bytes.as_slice();
                let t_zmq_receive_start = std::time::Instant::now();
                let receive_timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                debug!(
                    "🦀 [ZMQ-SENSORY] 📥 Received message #{}: {} bytes, timestamp: {}",
                    message_count,
                    message_bytes.len(),
                    receive_timestamp
                );

                // Try to deserialize as binary XYZP data (using feagi-data-processing)
                let t_deserialize_start = std::time::Instant::now();
                match Self::deserialize_and_inject_xyzp(
                    message_bytes,
                    &npu,
                    &agent_registry,
                    &rejected_no_genome,
                    &rejected_no_agents,
                ) {
                    Ok(neuron_count) => {
                        *total_neurons.lock() += neuron_count as u64;
                        let t_deserialize_ms = t_deserialize_start.elapsed().as_secs_f64() * 1000.0;
                        let t_zmq_total = t_zmq_receive_start.elapsed();
                        let processing_time_ms = t_zmq_total.as_secs_f64() * 1000.0;

                        // Log detailed performance metrics (first 10, then every 50th for better visibility)
                        if message_count <= 10 || message_count % 50 == 0 {
                            let total_msg = *total_messages.lock();
                            let total_n = *total_neurons.lock();
                            let avg_neurons_per_msg = if total_msg > 0 {
                                total_n / total_msg
                            } else {
                                0
                            };
                            info!(
                                "[PERF][FEAGI-ZMQ] Message #{}: {} bytes → {} neurons, deserialize+inject={:.2}ms, total={:.2}ms, avg_neurons={}, drained_newer={}",
                                message_count, message_bytes.len(), neuron_count, t_deserialize_ms, processing_time_ms, avg_neurons_per_msg, drained_newer
                            );
                        }

                        // Log performance warning if processing takes too long (affects frame rate)
                        if processing_time_ms > 33.0
                            && (message_count <= 10 || message_count % 100 == 0)
                        {
                            warn!(
                                "[PERF][FEAGI-ZMQ] ⚠️ Slow processing: {:.2}ms for {} neurons (target: <33ms for 30fps)",
                                processing_time_ms, neuron_count
                            );
                        }
                    }
                    Err(e) => {
                        // Always log first few errors, then periodically
                        if message_count <= 10 || message_count % 100 == 0 {
                            error!(
                                "🦀 [ZMQ-SENSORY] [ERR] Failed to process sensory data (message #{}): {}",
                                message_count, e
                            );
                            warn!(
                                "🦀 [ZMQ-SENSORY] Message size: {} bytes",
                                message_bytes.len()
                            );
                        }
                    }
                }
            }

            info!("🦀 [ZMQ-SENSORY] Processing loop stopped");
        });
    }

    /// Deserialize XYZP binary data and inject into NPU
    ///
    /// Receives binary XYZP data from agents, deserializes it using FeagiByteContainer
    /// (version 2 container format), and directly injects it into the Rust NPU.
    /// Pure Rust path with no Python FFI overhead.
    ///
    /// ## Security Gating
    /// This method rejects data if:
    /// 1. No genome is loaded (NPU has no neurons)
    /// 2. No agents with sensory capability are registered
    ///
    /// This prevents malicious agents from sending data when FEAGI is not ready.
    ///
    /// Returns the number of neurons injected.
    fn deserialize_and_inject_xyzp(
        message_bytes: &[u8],
        npu_mutex: &Arc<
            Mutex<
                Option<
                    Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
                >,
            >,
        >,
        agent_registry_mutex: &Arc<Mutex<Option<Arc<RwLock<crate::core::AgentRegistry>>>>>,
        rejected_no_genome: &Arc<Mutex<u64>>,
        rejected_no_agents: &Arc<Mutex<u64>>,
    ) -> Result<usize, FeagiDataError> {
        use feagi_serialization::FeagiByteContainer;
        use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

        // Get NPU reference
        let npu_lock = npu_mutex.lock();
        let npu_arc = match npu_lock.as_ref() {
            Some(n) => Arc::clone(n),
            None => {
                return Err(FeagiDataError::InternalError(
                    "NPU not connected".to_string(),
                ))
            }
        };
        drop(npu_lock); // Release early

        // SECURITY GATE 1: Check if genome is loaded
        {
            let lock_start = std::time::Instant::now();
            debug!("[NPU-LOCK] ZMQ-SENSORY: Acquiring lock for genome check");
            let npu = npu_arc.lock().unwrap();
            let lock_wait = lock_start.elapsed();
            debug!(
                "[NPU-LOCK] ZMQ-SENSORY: Lock acquired for genome check (waited {:.2}ms)",
                lock_wait.as_secs_f64() * 1000.0
            );
            if !npu.is_genome_loaded() {
                *rejected_no_genome.lock() += 1;
                let count = *rejected_no_genome.lock();
                if count == 1 || count % 100 == 0 {
                    warn!("🚫 [ZMQ-SENSORY] [SECURITY] Rejected sensory data: No genome loaded (rejected {} total)", count);
                }
                return Err(FeagiDataError::BadParameters(
                    "Security: No genome loaded".to_string(),
                ));
            }
        }

        // SECURITY GATE 2: Check if any sensory agents are registered
        {
            let registry_lock = agent_registry_mutex.lock();
            if let Some(registry_arc) = registry_lock.as_ref() {
                let registry = registry_arc.read();
                if !registry.has_sensory_agents() {
                    *rejected_no_agents.lock() += 1;
                    let count = *rejected_no_agents.lock();
                    if count == 1 || count % 100 == 0 {
                        warn!("🚫 [ZMQ-SENSORY] [SECURITY] Rejected sensory data: No registered sensory agents (rejected {} total)", count);
                    }
                    return Err(FeagiDataError::BadParameters(
                        "Security: No registered sensory agents".to_string(),
                    ));
                }
            } else {
                // AgentRegistry not connected yet - reject for safety
                return Err(FeagiDataError::InternalError(
                    "Security: AgentRegistry not connected".to_string(),
                ));
            }
        }

        // Deserialize using FeagiByteContainer (proper container format)
        let mut byte_container = FeagiByteContainer::new_empty();
        let mut data_vec = message_bytes.to_vec();

        // Load bytes into container
        byte_container
            .try_write_data_to_container_and_verify(&mut |bytes| {
                std::mem::swap(bytes, &mut data_vec);
                Ok(())
            })
            .map_err(|e| {
                FeagiDataError::DeserializationError(format!(
                    "Failed to load FeagiByteContainer: {:?}",
                    e
                ))
            })?;

        // Verify container structure count
        let num_structures = byte_container
            .try_get_number_contained_structures()
            .map_err(|e| {
                FeagiDataError::DeserializationError(format!(
                    "Failed to get structure count: {:?}",
                    e
                ))
            })?;

        if num_structures == 0 {
            return Err(FeagiDataError::BadParameters(
                "FeagiByteContainer has no structures".to_string(),
            ));
        }

        // Extract first structure (should be CorticalMappedXYZPNeuronVoxels)
        let boxed_struct = byte_container.try_create_new_struct_from_index(0)?;

        // Downcast to CorticalMappedXYZPNeuronVoxels using as_any().downcast_ref()
        let cortical_mapped = boxed_struct
            .as_any()
            .downcast_ref::<CorticalMappedXYZPNeuronVoxels>()
            .ok_or_else(|| {
                FeagiDataError::BadParameters(
                    "Structure is not CorticalMappedXYZPNeuronVoxels".to_string(),
                )
            })?;

        // ✅ CLEAN ARCHITECTURE: IOSystem just transports XYZP, NPU handles all neural logic
        // The NPU owns coordinate-to-ID conversion and does it efficiently in batch

        let t_inject_start = std::time::Instant::now();
        let mut total_injected = 0;

        // Temporal smoothing parameters
        const SMOOTHING_RAMP_FACTOR: f32 = 0.85; // Apply 85% of target potential per frame
        const LARGE_FRAME_THRESHOLD: usize = 5000; // Only smooth very large frames (>5000 neurons)

        // Calculate total neurons for batching decision
        let total_neurons: usize = cortical_mapped
            .mappings
            .values()
            .map(|arrays| arrays.len())
            .sum();

        // CRITICAL FIX: For very large injections (>100k neurons), hold NPU lock for entire injection
        // to ensure atomicity. This prevents the sensory frame from being split across multiple bursts.
        //
        // Previous implementation released lock between batches, allowing burst loop to run and process
        // only partial data, causing correctness issues where one sensory frame was processed across
        // multiple bursts instead of a single burst.
        if total_neurons > 100_000 {
            let injection_start = std::time::Instant::now();

            // Hold NPU lock for entire injection to ensure all data is added to pending_sensory_injections
            // before any burst can process it. This ensures the entire sensory frame is processed in ONE burst.
            let lock_start = std::time::Instant::now();
            debug!(
                "[NPU-LOCK] ZMQ-SENSORY: Acquiring lock for LARGE injection ({} neurons) - THIS CAN BLOCK BURST LOOP!",
                total_neurons
            );
            let mut npu = npu_arc.lock().unwrap();
            let lock_wait = lock_start.elapsed();
            debug!(
                "[NPU-LOCK] ZMQ-SENSORY: Lock acquired for large injection (waited {:.2}ms, holding for {} neurons)",
                lock_wait.as_secs_f64() * 1000.0,
                total_neurons
            );
            npu.clear_pending_sensory_injections();

            // Process all cortical areas while holding the lock
            for (cortical_id, neuron_arrays) in &cortical_mapped.mappings {
                let (x_coords, y_coords, z_coords, potentials) =
                    neuron_arrays.borrow_xyzp_vectors();
                let num_neurons = x_coords.len();

                if num_neurons == 0 {
                    continue;
                }

                // Apply smoothing if needed (pre-compute once for entire area)
                let smoothed_potentials: Option<Vec<f32>> = if num_neurons > LARGE_FRAME_THRESHOLD {
                    Some(
                        potentials
                            .iter()
                            .map(|&p| p * SMOOTHING_RAMP_FACTOR)
                            .collect(),
                    )
                } else {
                    None
                };

                // Inject entire area at once (ensures atomicity - all neurons from this area
                // are added to pending_sensory_injections before lock is released)
                let injected = if let Some(ref smoothed) = smoothed_potentials {
                    npu.inject_sensory_xyzp_arrays_by_id(
                        cortical_id,
                        x_coords,
                        y_coords,
                        z_coords,
                        smoothed,
                    )
                } else {
                    npu.inject_sensory_xyzp_arrays_by_id(
                        cortical_id,
                        x_coords,
                        y_coords,
                        z_coords,
                        potentials,
                    )
                };
                total_injected += injected;
            }

            // Lock released here - all injections are now in pending_sensory_injections
            // The next burst will process ALL of them atomically
            let lock_hold_duration = lock_start.elapsed();
            drop(npu);
            debug!(
                "[NPU-LOCK] ZMQ-SENSORY: Lock released after large injection (held for {:.2}ms)",
                lock_hold_duration.as_secs_f64() * 1000.0
            );

            let total_duration = injection_start.elapsed();
            // Only warn for extreme cases (>30 seconds) - batch processing can legitimately take several seconds
            if total_duration.as_millis() > 30000 {
                warn!(
                    "[ZMQ-SENSORY] Extremely slow injection: {} neurons in {:.2}ms (possible performance issue)",
                    total_injected,
                    total_duration.as_secs_f64() * 1000.0
                );
            } else if total_injected > 1_000_000 {
                // Info log for large batch injections (medical imaging, analysis, etc.)
                info!(
                    "[ZMQ-SENSORY] Batch injection: {} neurons in {:.2}ms (batch processing mode)",
                    total_injected,
                    total_duration.as_secs_f64() * 1000.0
                );
            }
        } else {
            // Small injection: process normally (single lock acquisition)
            let lock_start = std::time::Instant::now();
            debug!(
                "[NPU-LOCK] ZMQ-SENSORY: Acquiring lock for small injection ({} neurons)",
                total_neurons
            );
            let mut npu = npu_arc.lock().unwrap();
            let lock_wait = lock_start.elapsed();
            debug!(
                "[NPU-LOCK] ZMQ-SENSORY: Lock acquired for small injection (waited {:.2}ms)",
                lock_wait.as_secs_f64() * 1000.0
            );

            // Clear pending injections for large frames
            if total_neurons > LARGE_FRAME_THRESHOLD {
                npu.clear_pending_sensory_injections();
            }

            for (cortical_id, neuron_arrays) in &cortical_mapped.mappings {
                let (x_coords, y_coords, z_coords, potentials) =
                    neuron_arrays.borrow_xyzp_vectors();

                let injected = if potentials.len() > LARGE_FRAME_THRESHOLD {
                    let smoothed_potentials: Vec<f32> = potentials
                        .iter()
                        .map(|&p| p * SMOOTHING_RAMP_FACTOR)
                        .collect();

                    npu.inject_sensory_xyzp_arrays_by_id(
                        cortical_id,
                        x_coords,
                        y_coords,
                        z_coords,
                        &smoothed_potentials,
                    )
                } else {
                    npu.inject_sensory_xyzp_arrays_by_id(
                        cortical_id,
                        x_coords,
                        y_coords,
                        z_coords,
                        potentials,
                    )
                };
                total_injected += injected;

                if injected == 0 {
                    debug!(
                        "[ZMQ-SENSORY] No neurons injected for area '{}' ({} coords)",
                        cortical_id.as_base_64(),
                        x_coords.len()
                    );
                }
            }
            let lock_hold_duration = lock_start.elapsed();
            drop(npu);
            debug!(
                "[NPU-LOCK] ZMQ-SENSORY: Lock released after small injection (held for {:.2}ms)",
                lock_hold_duration.as_secs_f64() * 1000.0
            );
        }

        // Log injection timing only for slow injections (>50ms) when debug logging is enabled
        // Zero injections are normal in event-driven systems - no need to log
        if tracing::enabled!(tracing::Level::DEBUG) {
            let t_inject_ms = t_inject_start.elapsed().as_secs_f64() * 1000.0;
            if t_inject_ms > 50.0 {
                debug!(
                    "[PERF][FEAGI-INJECT] Injected {} neurons in {:.2}ms",
                    total_injected, t_inject_ms
                );
            }
        }

        Ok(total_injected)
    }

    /// Check if stream is running
    pub fn is_running(&self) -> bool {
        *self.running.lock()
    }

    /// Get statistics
    pub fn get_stats(&self) -> (u64, u64) {
        (*self.total_messages.lock(), *self.total_neurons.lock())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensory_stream_creation() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let stream = SensoryStream::new(
            runtime,
            "tcp://127.0.0.1:5558",
            SensoryReceiveConfig::default(),
        );
        assert!(stream.is_ok());
    }

    #[test]
    fn test_sensory_stream_applies_socket_config() {
        let runtime = Arc::new(Runtime::new().unwrap());
        let config = SensoryReceiveConfig {
            receive_high_water_mark: 3,
            linger_ms: 0,
            immediate: true,
            poll_timeout_ms: 10,
            startup_drain_timeout_ms: 500,
        };

        let stream = SensoryStream::new(runtime, "tcp://127.0.0.1:5568", config.clone()).unwrap();
        assert!(stream.start().is_err());
    }
}
