// Sensory stream for receiving sensory data from agents
// Uses PULL socket pattern for receiving data from multiple agents (agents use PUSH)

use feagi_data_serialization::FeagiSerializable;
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;

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
            receive_high_water_mark: 1000,
            linger_ms: 0,
            immediate: false,
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
    npu: Arc<Mutex<Option<Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>>>>,
    /// Statistics
    total_messages: Arc<Mutex<u64>>,
    total_neurons: Arc<Mutex<u64>>,
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
            total_messages: Arc::new(Mutex::new(0)),
            total_neurons: Arc::new(Mutex::new(0)),
        })
    }

    /// Set the Rust NPU reference for direct injection
    pub fn set_npu(&self, npu: Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>) {
        *self.npu.lock() = Some(npu);
        println!("🦀 [SENSORY-STREAM] NPU connected for direct injection");
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

        println!("🦀 [ZMQ-SENSORY] ✅ Listening on {}", self.bind_address);

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
        println!(
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

        println!(
            "🦀 [ZMQ-SENSORY] 🗑️  Draining stale messages (timeout: {}ms)...",
            self.config.startup_drain_timeout_ms
        );

        let sock_guard = self.socket.lock();
        let sock = match sock_guard.as_ref() {
            Some(s) => s,
            None => {
                eprintln!("🦀 [ZMQ-SENSORY] [ERR] Cannot drain - socket not initialized");
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
                    eprintln!("🦀 [ZMQ-SENSORY] [ERR] Drain error: {}", e);
                    break;
                }
            }
        }

        drop(sock_guard);

        if drained_count > 0 {
            println!(
                "🦀 [ZMQ-SENSORY] 🗑️  Drained {} stale messages ({:.1}ms)",
                drained_count,
                drain_start.elapsed().as_secs_f64() * 1000.0
            );
        } else {
            println!("🦀 [ZMQ-SENSORY] ✅ No stale messages found (buffer was clean)");
        }
    }

    /// Start the background processing loop
    fn start_processing_loop(&self) {
        let socket = Arc::clone(&self.socket);
        let running = Arc::clone(&self.running);
        let npu = Arc::clone(&self.npu);
        let total_messages = Arc::clone(&self.total_messages);
        let total_neurons = Arc::clone(&self.total_neurons);
        let config = self.config.clone();

        thread::spawn(move || {
            println!("🦀 [ZMQ-SENSORY] Processing loop started");

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
                    eprintln!("🦀 [ZMQ-SENSORY] [ERR] Poll error: {}", e);
                    continue;
                }

                if !poll_items[0].is_readable() {
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
                        let message_bytes = msg.as_ref();

                        // Try to deserialize as binary XYZP data (using feagi-data-processing)
                        match Self::deserialize_and_inject_xyzp(message_bytes, &npu) {
                            Ok(neuron_count) => {
                                *total_neurons.lock() += neuron_count as u64;

                                // Log periodically
                                if message_count % 100 == 0 {
                                    let total_msg = *total_messages.lock();
                                    let total_n = *total_neurons.lock();
                                    println!(
                                        "🦀 [ZMQ-SENSORY] Stats: {} messages, {} neurons total",
                                        total_msg, total_n
                                    );
                                }
                            }
                            Err(e) => {
                                if message_count <= 5 {
                                    eprintln!(
                                        "🦀 [ZMQ-SENSORY] [ERR] Failed to process sensory data: {}",
                                        e
                                    );
                                    eprintln!(
                                        "🦀 [ZMQ-SENSORY] Message size: {} bytes",
                                        message_bytes.len()
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("🦀 [ZMQ-SENSORY] [ERR] Receive error: {}", e);
                    }
                }
            }

            println!("🦀 [ZMQ-SENSORY] Processing loop stopped");
        });
    }

    /// Deserialize XYZP binary data and inject into NPU
    ///
    /// Receives binary XYZP data from agents, deserializes it using feagi_data_serialization,
    /// and directly injects it into the Rust NPU. Pure Rust path with no Python FFI overhead.
    ///
    /// Returns the number of neurons injected.
    fn deserialize_and_inject_xyzp(
        message_bytes: &[u8],
        npu_mutex: &Arc<Mutex<Option<Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>>>>,
    ) -> Result<usize, String> {
        use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

        // Get NPU reference
        let npu_lock = npu_mutex.lock();
        let npu_arc = match npu_lock.as_ref() {
            Some(n) => Arc::clone(n),
            None => return Err("NPU not connected".to_string()),
        };
        drop(npu_lock); // Release early

        // Validate container header
        if message_bytes.len() < 8 {
            return Err(format!("Message too short: {} bytes", message_bytes.len()));
        }

        let version = message_bytes[0];
        let struct_count = message_bytes[3];
        let data_size = u32::from_le_bytes([
            message_bytes[4],
            message_bytes[5],
            message_bytes[6],
            message_bytes[7],
        ]) as usize;

        if version != 2 {
            return Err(format!("Expected version 2, got {}", version));
        }

        if struct_count != 1 {
            return Err(format!("Expected 1 struct, got {}", struct_count));
        }

        // Struct data starts at byte 8 (after global header + data_size)
        let struct_data_start = 8;
        let struct_data_end = 8 + data_size;

        if message_bytes.len() < struct_data_end {
            return Err(format!(
                "Message too short: {} < {}",
                message_bytes.len(),
                struct_data_end
            ));
        }

        let struct_data = &message_bytes[struct_data_start..struct_data_end];

        // Deserialize CorticalMappedXYZPNeuronVoxels
        let mut cortical_mapped = CorticalMappedXYZPNeuronVoxels::new();
        cortical_mapped
            .try_deserialize_and_update_self_from_byte_slice(struct_data)
            .map_err(|e| format!("Failed to deserialize: {:?}", e))?;

        // Inject XYZP data into NPU
        let mut total_injected = 0;
        let mut npu = npu_arc.lock().unwrap();

        for (cortical_id, neuron_arrays) in &cortical_mapped.mappings {
            // Convert CorticalID to string (use as_ascii_string, not to_string which adds quotes)
            let cortical_name = cortical_id.as_ascii_string();

            // Resolve cortical name to NPU index
            let cortical_idx = match npu.get_cortical_area_id(&cortical_name) {
                Some(idx) => idx,
                None => {
                    eprintln!(
                        "[ZMQ-SENSORY] Warning: Unknown cortical area '{}'",
                        cortical_name
                    );
                    continue;
                }
            };

            // Use borrow_xyzp_vectors to access the coordinate vectors
            let (x_coords, y_coords, z_coords, potentials) = neuron_arrays.borrow_xyzp_vectors();

            // Build neuron ID + potential pairs for direct injection
            let num_neurons = neuron_arrays.len();
            let mut neuron_potential_pairs = Vec::with_capacity(num_neurons);

            // For each XYZ coordinate, find the neuron ID in the NPU
            for i in 0..num_neurons {
                if let Some(neuron_id) = npu.get_neuron_at_coordinates(
                    cortical_idx,
                    x_coords[i],
                    y_coords[i],
                    z_coords[i],
                ) {
                    neuron_potential_pairs.push((neuron_id, potentials[i]));
                }
            }

            // Inject into NPU
            if !neuron_potential_pairs.is_empty() {
                npu.inject_sensory_with_potentials(&neuron_potential_pairs);
                total_injected += neuron_potential_pairs.len();
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
        };

        let stream =
            SensoryStream::new(Arc::clone(&ctx), "tcp://127.0.0.1:5568", config.clone()).unwrap();
        stream.start().unwrap();

        {
            let socket_guard = stream.socket.lock();
            let socket = socket_guard.as_ref().expect("socket must be initialized");
            assert_eq!(socket.get_rcvhwm().unwrap(), config.receive_high_water_mark);
            assert_eq!(socket.get_linger().unwrap(), config.linger_ms);
            assert_eq!(socket.get_immediate().unwrap(), config.immediate);
        }

        stream.stop().unwrap();
    }
}
