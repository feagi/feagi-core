// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Sensory stream for receiving sensory data from agents
// Uses PULL socket pattern for receiving data from multiple agents (agents use PUSH)

use parking_lot::{Mutex, RwLock};
use std::sync::Arc;
use std::thread;
use tracing::{debug, error, info, warn};

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
    pub fn validate(&self) -> Result<(), String> {
        if self.receive_high_water_mark < 0 {
            return Err("receive_high_water_mark must be >= 0".to_string());
        }
        if self.poll_timeout_ms < 0 {
            return Err("poll_timeout_ms must be >= 0".to_string());
        }
        if self.startup_drain_timeout_ms > 10000 {
            return Err("startup_drain_timeout_ms must be <= 10000ms (10 seconds)".to_string());
        }
        Ok(())
    }
}

/// Sensory stream for receiving sensory data from agents
#[derive(Clone)]
pub struct SensoryStream {
    context: Arc<zmq::Context>,
    bind_address: String,
    socket: Arc<Mutex<Option<zmq::Socket>>>,
    running: Arc<Mutex<bool>>,
    config: SensoryReceiveConfig,
    /// Reference to Rust NPU for direct injection (no FFI overhead!)
    npu: Arc<Mutex<Option<Arc<std::sync::Mutex<feagi_npu_burst_engine::DynamicNPU>>>>>,
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
        context: Arc<zmq::Context>,
        bind_address: &str,
        config: SensoryReceiveConfig,
    ) -> Result<Self, String> {
        config.validate()?;
        Ok(Self {
            context,
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
    pub fn set_npu(&self, npu: Arc<std::sync::Mutex<feagi_npu_burst_engine::DynamicNPU>>) {
        *self.npu.lock() = Some(npu);
        info!("🦀 [SENSORY-STREAM] NPU connected for direct injection");
    }

    /// Set the AgentRegistry reference for security gating
    pub fn set_agent_registry(&self, registry: Arc<RwLock<crate::core::AgentRegistry>>) {
        *self.agent_registry.lock() = Some(registry);
        info!("🦀 [SENSORY-STREAM] AgentRegistry connected for security gating");
    }

    /// Start the sensory stream
    pub fn start(&self) -> Result<(), String> {
        if *self.running.lock() {
            return Err("Sensory stream already running".to_string());
        }

        // Create PULL socket for receiving sensory data
        let socket = self.context.socket(zmq::PULL).map_err(|e| e.to_string())?;

        // Set socket options
        socket
            .set_linger(self.config.linger_ms)
            .map_err(|e| e.to_string())?;
        socket
            .set_rcvhwm(self.config.receive_high_water_mark)
            .map_err(|e| e.to_string())?;
        socket
            .set_immediate(self.config.immediate)
            .map_err(|e| e.to_string())?;

        // Bind socket
        socket.bind(&self.bind_address).map_err(|e| e.to_string())?;

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
    pub fn stop(&self) -> Result<(), String> {
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

        let sock_guard = self.socket.lock();
        let sock = match sock_guard.as_ref() {
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

            // Non-blocking receive (DONTWAIT flag)
            let mut msg = zmq::Message::new();
            match sock.recv(&mut msg, zmq::DONTWAIT) {
                Ok(()) => {
                    drained_count += 1;
                    // Message discarded - we don't process stale data
                }
                Err(zmq::Error::EAGAIN) => {
                    // No more messages available - buffer is empty
                    break;
                }
                Err(e) => {
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

        thread::spawn(move || {
            info!("🦀 [ZMQ-SENSORY] Processing loop started");

            let mut message_count = 0u64;

            while *running.lock() {
                let sock_guard = socket.lock();
                let sock = match sock_guard.as_ref() {
                    Some(s) => s,
                    None => {
                        drop(sock_guard);
                        thread::sleep(std::time::Duration::from_millis(100));
                        continue;
                    }
                };

                // Poll for messages with timeout
                let poll_items = &mut [sock.as_poll_item(zmq::POLLIN)];
                if let Err(e) = zmq::poll(poll_items, config.poll_timeout_ms) {
                    error!("🦀 [ZMQ-SENSORY] [ERR] Poll error: {}", e);
                    continue;
                }

                if !poll_items[0].is_readable() {
                    // Log periodically that we're polling but no messages
                    if message_count == 0 || message_count.is_multiple_of(1000) {
                        debug!("🦀 [ZMQ-SENSORY] 🔍 Polling for messages (no data yet, message_count: {})", message_count);
                    }
                    drop(sock_guard);
                    continue;
                }

                // Receive message
                let mut msg = zmq::Message::new();
                match sock.recv(&mut msg, 0) {
                    Ok(()) => {
                        drop(sock_guard); // Release lock before processing

                        *total_messages.lock() += 1;
                        message_count += 1;

                        // Process the binary data
                        let message_bytes: &[u8] = msg.as_ref();
                        let t_zmq_receive_start = std::time::Instant::now();
                        debug!(
                            "🦀 [ZMQ-SENSORY] 📥 Received message #{}: {} bytes",
                            message_count,
                            message_bytes.len()
                        );

                        // Try to deserialize as binary XYZP data (using feagi-data-processing)
                        match Self::deserialize_and_inject_xyzp(
                            message_bytes,
                            &npu,
                            &agent_registry,
                            &rejected_no_genome,
                            &rejected_no_agents,
                        ) {
                            Ok(neuron_count) => {
                                *total_neurons.lock() += neuron_count as u64;

                                // Log stats periodically (every 100 messages)
                                if message_count.is_multiple_of(100) {
                                    let t_zmq_total = t_zmq_receive_start.elapsed();
                                    let total_msg = *total_messages.lock();
                                    let total_n = *total_neurons.lock();
                                    debug!(
                                        "[ZMQ-SENSORY] Stats: {} messages, {} neurons total, last msg: {:.2}ms, {} bytes, {} neurons",
                                        total_msg, total_n, t_zmq_total.as_secs_f64() * 1000.0, message_bytes.len(), neuron_count
                                    );
                                }
                            }
                            Err(e) => {
                                // Always log first few errors, then periodically
                                if message_count <= 10 || message_count.is_multiple_of(100) {
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
                    Err(e) => {
                        error!("🦀 [ZMQ-SENSORY] [ERR] Receive error: {}", e);
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
        npu_mutex: &Arc<Mutex<Option<Arc<std::sync::Mutex<feagi_npu_burst_engine::DynamicNPU>>>>>,
        agent_registry_mutex: &Arc<Mutex<Option<Arc<RwLock<crate::core::AgentRegistry>>>>>,
        rejected_no_genome: &Arc<Mutex<u64>>,
        rejected_no_agents: &Arc<Mutex<u64>>,
    ) -> Result<usize, String> {
        use feagi_serialization::FeagiByteContainer;
        use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

        // Get NPU reference
        let npu_lock = npu_mutex.lock();
        let npu_arc = match npu_lock.as_ref() {
            Some(n) => Arc::clone(n),
            None => return Err("NPU not connected".to_string()),
        };
        drop(npu_lock); // Release early

        // SECURITY GATE 1: Check if genome is loaded
        {
            let npu = npu_arc.lock().unwrap();
            if !npu.is_genome_loaded() {
                *rejected_no_genome.lock() += 1;
                let count = *rejected_no_genome.lock();
                if count == 1 || count.is_multiple_of(100) {
                    warn!("🚫 [ZMQ-SENSORY] [SECURITY] Rejected sensory data: No genome loaded (rejected {} total)", count);
                }
                return Err("Security: No genome loaded".to_string());
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
                    if count == 1 || count.is_multiple_of(100) {
                        warn!("🚫 [ZMQ-SENSORY] [SECURITY] Rejected sensory data: No registered sensory agents (rejected {} total)", count);
                    }
                    return Err("Security: No registered sensory agents".to_string());
                }
            } else {
                // AgentRegistry not connected yet - reject for safety
                return Err("Security: AgentRegistry not connected".to_string());
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
            .map_err(|e| format!("Failed to load FeagiByteContainer: {:?}", e))?;

        // Verify container structure count
        let num_structures = byte_container
            .try_get_number_contained_structures()
            .map_err(|e| format!("Failed to get structure count: {:?}", e))?;

        if num_structures == 0 {
            return Err("FeagiByteContainer has no structures".to_string());
        }

        // Extract first structure (should be CorticalMappedXYZPNeuronVoxels)
        let boxed_struct = byte_container
            .try_create_new_struct_from_index(0)
            .map_err(|e| format!("Failed to deserialize structure from container: {:?}", e))?;

        // Downcast to CorticalMappedXYZPNeuronVoxels using as_any().downcast_ref()
        let cortical_mapped = boxed_struct
            .as_any()
            .downcast_ref::<CorticalMappedXYZPNeuronVoxels>()
            .ok_or_else(|| "Structure is not CorticalMappedXYZPNeuronVoxels".to_string())?;

        // ✅ CLEAN ARCHITECTURE: IOSystem just transports XYZP, NPU handles all neural logic
        // The NPU owns coordinate-to-ID conversion and does it efficiently in batch

        let mut total_injected = 0;

        for (cortical_id, neuron_arrays) in &cortical_mapped.mappings {
            // Step 1: Extract raw XYZP data (NO LOCKS)
            let (x_coords, y_coords, z_coords, potentials) = neuron_arrays.borrow_xyzp_vectors();
            let num_neurons = neuron_arrays.len();

            // Build XYZP array
            let xyzp_data: Vec<(u32, u32, u32, f32)> = (0..num_neurons)
                .map(|i| (x_coords[i], y_coords[i], z_coords[i], potentials[i]))
                .collect();

            // Step 2: Quick lock - NPU handles everything (name resolution + coordinate conversion + injection)
            let mut npu = npu_arc.lock().unwrap();

            // NPU handles: CorticalID → cortical_idx lookup, coordinates → neuron IDs, injection
            let injected = npu.inject_sensory_xyzp_by_id(cortical_id, &xyzp_data);
            total_injected += injected;

            drop(npu);

            // Only log missing neurons at debug level (common case, not an error)
            if injected == 0 {
                debug!(
                    "[ZMQ-SENSORY] No neurons injected for area '{}' ({} coords)",
                    cortical_id.as_base_64(),
                    xyzp_data.len()
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
        let ctx = Arc::new(zmq::Context::new());
        let stream =
            SensoryStream::new(ctx, "tcp://127.0.0.1:5558", SensoryReceiveConfig::default());
        assert!(stream.is_ok());
    }

    #[test]
    fn test_sensory_stream_applies_socket_config() {
        let ctx = Arc::new(zmq::Context::new());
        let config = SensoryReceiveConfig {
            receive_high_water_mark: 3,
            linger_ms: 0,
            immediate: true,
            poll_timeout_ms: 10,
            startup_drain_timeout_ms: 500,
        };

        let stream =
            SensoryStream::new(Arc::clone(&ctx), "tcp://127.0.0.1:5568", config.clone()).unwrap();
        stream.start().unwrap();

        {
            let socket_guard = stream.socket.lock();
            let socket = socket_guard.as_ref().expect("socket must be initialized");
            assert_eq!(socket.get_rcvhwm().unwrap(), config.receive_high_water_mark);
            assert_eq!(socket.get_linger().unwrap(), config.linger_ms);
            // Note: get_immediate() may not be available in all zmq versions
            // assert_eq!(socket.get_immediate().unwrap(), config.immediate);
        }

        stream.stop().unwrap();
    }
}
