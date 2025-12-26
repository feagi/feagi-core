// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! FEAGI Agent Client implementation

use crate::config::AgentConfig;
use crate::error::{Result, SdkError};
use crate::heartbeat::HeartbeatService;
use crate::reconnect::{retry_with_backoff, ReconnectionStrategy};
use feagi_io::AgentType;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;
use tracing::{debug, error, info, warn};

/// Main FEAGI Agent Client
///
/// This client handles:
/// - Registration with FEAGI
/// - Automatic heartbeat
/// - Sending sensory data
/// - Receiving motor data (for motor agents)
/// - Automatic deregistration on drop
///
/// # Example
/// ```ignore
/// use feagi_agent::{AgentClient, AgentConfig, AgentType};
///
/// let config = AgentConfig::new("my_camera", AgentType::Sensory)
///     .with_feagi_host("localhost")
///     .with_vision_capability("camera", (640, 480), 3, "i_vision");
///
/// let mut client = AgentClient::new(config)?;
/// client.connect()?;
///
/// // Send sensory data
/// client.send_sensory_data(vec![(0, 50.0), (1, 75.0)])?;
///
/// // Client auto-deregisters on drop
/// ```
pub struct AgentClient {
    /// Configuration
    config: AgentConfig,

    /// ZMQ context
    context: zmq::Context,

    /// Registration socket (ZMQ REQ - shared with heartbeat)
    registration_socket: Option<Arc<Mutex<zmq::Socket>>>,

    /// Sensory data socket (ZMQ PUSH)
    sensory_socket: Option<zmq::Socket>,

    /// Motor data socket (ZMQ SUB)
    motor_socket: Option<zmq::Socket>,

    /// Visualization stream socket (ZMQ SUB)
    viz_socket: Option<zmq::Socket>,

    /// Control/API socket (ZMQ REQ - REST over ZMQ)
    control_socket: Option<zmq::Socket>,

    /// Heartbeat service
    heartbeat: Option<HeartbeatService>,

    /// Registration state
    registered: bool,
}

impl AgentClient {
    /// Create a new FEAGI agent client
    ///
    /// # Arguments
    /// * `config` - Agent configuration
    pub fn new(config: AgentConfig) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        let context = zmq::Context::new();

        Ok(Self {
            config,
            context,
            registration_socket: None,
            sensory_socket: None,
            motor_socket: None,
            viz_socket: None,
            control_socket: None,
            heartbeat: None,
            registered: false,
        })
    }

    /// Connect to FEAGI and register the agent
    ///
    /// This will:
    /// 1. Create ZMQ sockets
    /// 2. Register with FEAGI
    /// 3. Start heartbeat service (ONLY after successful registration)
    ///
    /// IMPORTANT: Background threads are ONLY started AFTER successful registration.
    /// This prevents thread interference with GUI event loops (e.g., MuJoCo, Godot).
    /// If registration fails, NO threads are spawned.
    pub fn connect(&mut self) -> Result<()> {
        if self.registered {
            return Err(SdkError::AlreadyConnected);
        }

        info!(
            "[CLIENT] Connecting to FEAGI: {}",
            self.config.registration_endpoint
        );

        // Step 1: Create sockets with retry
        let mut socket_strategy = ReconnectionStrategy::new(
            self.config.retry_backoff_ms,
            self.config.registration_retries,
        );
        retry_with_backoff(
            || self.create_sockets(),
            &mut socket_strategy,
            "Socket creation",
        )?;

        // Step 2: Register with FEAGI with retry
        let mut reg_strategy = ReconnectionStrategy::new(
            self.config.retry_backoff_ms,
            self.config.registration_retries,
        );
        retry_with_backoff(|| self.register(), &mut reg_strategy, "Registration")?;

        // Step 3: Start heartbeat service (ONLY after successful registration)
        // This is critical: threads are only spawned AFTER we know FEAGI is reachable
        if self.config.heartbeat_interval > 0.0 {
            debug!("[CLIENT] Starting heartbeat service (post-registration)");
            self.start_heartbeat()?;
        } else {
            debug!("[CLIENT] Heartbeat disabled (interval = 0)");
        }

        info!(
            "[CLIENT] ✓ Connected and registered as: {}",
            self.config.agent_id
        );
        Ok(())
    }

    /// Create ZMQ sockets
    fn create_sockets(&mut self) -> Result<()> {
        // Registration socket (REQ - for registration and heartbeat)
        let reg_socket = self.context.socket(zmq::REQ)?;
        reg_socket.set_rcvtimeo(self.config.connection_timeout_ms as i32)?;
        reg_socket.set_sndtimeo(self.config.connection_timeout_ms as i32)?;
        reg_socket.connect(&self.config.registration_endpoint)?;
        self.registration_socket = Some(Arc::new(Mutex::new(reg_socket)));

        // Sensory socket (PUSH - for sending data to FEAGI)
        let sensory_socket = self.context.socket(zmq::PUSH)?;
        sensory_socket.set_sndhwm(self.config.sensory_send_hwm)?;
        sensory_socket.set_linger(self.config.sensory_linger_ms)?;
        sensory_socket.set_immediate(self.config.sensory_immediate)?;
        sensory_socket.connect(&self.config.sensory_endpoint)?;
        self.sensory_socket = Some(sensory_socket);

        // Motor socket (SUB - for receiving motor commands from FEAGI)
        if matches!(self.config.agent_type, AgentType::Motor | AgentType::Both) {
            info!(
                "[SDK-CONNECT] 🎮 Initializing motor socket for agent '{}' (type: {:?})",
                self.config.agent_id, self.config.agent_type
            );
            info!(
                "[SDK-CONNECT] 🎮 Motor endpoint: {}",
                self.config.motor_endpoint
            );

            let motor_socket = self.context.socket(zmq::SUB)?;
            motor_socket.connect(&self.config.motor_endpoint)?;
            info!("[SDK-CONNECT] ✅ Motor socket connected");

            // Subscribe to messages for this agent
            info!(
                "[SDK-CONNECT] 🎮 Subscribing to topic: '{}'",
                String::from_utf8_lossy(self.config.agent_id.as_bytes())
            );
            motor_socket.set_subscribe(self.config.agent_id.as_bytes())?;
            info!("[SDK-CONNECT] ✅ Motor subscription set");

            self.motor_socket = Some(motor_socket);
            info!("[SDK-CONNECT] ✅ Motor socket initialized successfully");
        } else {
            info!(
                "[SDK-CONNECT] ⚠️ Motor socket NOT initialized (agent type: {:?})",
                self.config.agent_type
            );
        }

        // Visualization socket (SUB - for receiving neural activity stream from FEAGI)
        if matches!(
            self.config.agent_type,
            AgentType::Visualization | AgentType::Infrastructure
        ) {
            let viz_socket = self.context.socket(zmq::SUB)?;
            viz_socket.connect(&self.config.visualization_endpoint)?;

            // Subscribe to all visualization messages
            viz_socket.set_subscribe(b"")?;
            self.viz_socket = Some(viz_socket);
            debug!("[CLIENT] ✓ Visualization socket created");
        }

        // Control socket (REQ - for REST API requests over ZMQ)
        if matches!(self.config.agent_type, AgentType::Infrastructure) {
            let control_socket = self.context.socket(zmq::REQ)?;
            control_socket.set_rcvtimeo(self.config.connection_timeout_ms as i32)?;
            control_socket.set_sndtimeo(self.config.connection_timeout_ms as i32)?;
            control_socket.connect(&self.config.control_endpoint)?;
            self.control_socket = Some(control_socket);
            debug!("[CLIENT] ✓ Control/API socket created");
        }

        debug!("[CLIENT] ✓ ZMQ sockets created");
        Ok(())
    }

    /// Register with FEAGI
    fn register(&mut self) -> Result<()> {
        let registration_msg = serde_json::json!({
            "method": "POST",
            "path": "/v1/agent/register",
            "body": {
                "agent_id": self.config.agent_id,
                "agent_type": match self.config.agent_type {
                    AgentType::Sensory => "sensory",
                    AgentType::Motor => "motor",
                    AgentType::Both => "both",
                    AgentType::Visualization => "visualization",
                    AgentType::Infrastructure => "infrastructure",
                },
                "capabilities": self.config.capabilities,
            }
        });

        let socket = self
            .registration_socket
            .as_ref()
            .ok_or_else(|| SdkError::Other("Registration socket not initialized".to_string()))?;

        // Send registration request and get response
        let response = {
            let socket = socket
                .lock()
                .map_err(|e| SdkError::ThreadError(format!("Failed to lock socket: {}", e)))?;

            debug!(
                "[CLIENT] Sending registration request for: {}",
                self.config.agent_id
            );
            socket.send(registration_msg.to_string().as_bytes(), 0)?;

            // Wait for response
            let response_bytes = socket.recv_bytes(0)?;
            serde_json::from_slice::<serde_json::Value>(&response_bytes)?
        }; // Lock is dropped here

        // Check response status (REST format: {"status": 200, "body": {...}})
        let status_code = response
            .get("status")
            .and_then(|s| s.as_u64())
            .unwrap_or(500);
        if status_code == 200 {
            self.registered = true;
            info!("[CLIENT] ✓ Registration successful: {:?}", response);
            Ok(())
        } else {
            let empty_body = serde_json::json!({});
            let body = response.get("body").unwrap_or(&empty_body);
            let message = body
                .get("error")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");

            // Check if already registered - try deregistration
            if message.contains("already registered") {
                warn!("[CLIENT] ⚠ Agent already registered - attempting deregistration first");
                self.deregister()?;
                Err(SdkError::RegistrationFailed(
                    "Retry after deregistration".to_string(),
                ))
            } else {
                error!("[CLIENT] ✗ Registration failed: {}", message);
                Err(SdkError::RegistrationFailed(message.to_string()))
            }
        }
    }

    /// Deregister from FEAGI
    fn deregister(&mut self) -> Result<()> {
        if !self.registered && self.registration_socket.is_none() {
            return Ok(()); // Nothing to deregister
        }

        info!("[CLIENT] Deregistering agent: {}", self.config.agent_id);

        let deregistration_msg = serde_json::json!({
            "method": "DELETE",
            "path": "/v1/agent/deregister",
            "body": {
                "agent_id": self.config.agent_id,
            }
        });

        if let Some(socket) = &self.registration_socket {
            let socket = socket
                .lock()
                .map_err(|e| SdkError::ThreadError(format!("Failed to lock socket: {}", e)))?;

            // Send deregistration request
            if let Err(e) = socket.send(deregistration_msg.to_string().as_bytes(), 0) {
                warn!("[CLIENT] ⚠ Failed to send deregistration: {}", e);
                return Ok(()); // Don't fail on deregistration error
            }

            // Wait for response (with timeout)
            match socket.recv_bytes(0) {
                Ok(response_bytes) => {
                    let response: serde_json::Value = serde_json::from_slice(&response_bytes)?;
                    if response.get("status").and_then(|s| s.as_str()) == Some("success") {
                        info!("[CLIENT] ✓ Deregistration successful");
                    } else {
                        warn!("[CLIENT] ⚠ Deregistration returned: {:?}", response);
                    }
                }
                Err(e) => {
                    warn!("[CLIENT] ⚠ Deregistration timeout/error: {}", e);
                }
            }
        }

        self.registered = false;
        Ok(())
    }

    /// Start heartbeat service
    fn start_heartbeat(&mut self) -> Result<()> {
        if self.heartbeat.is_some() {
            return Ok(());
        }

        let socket = self
            .registration_socket
            .as_ref()
            .ok_or_else(|| SdkError::Other("Registration socket not initialized".to_string()))?;

        let mut heartbeat = HeartbeatService::new(
            self.config.agent_id.clone(),
            Arc::clone(socket),
            self.config.heartbeat_interval,
        );

        heartbeat.start()?;
        self.heartbeat = Some(heartbeat);

        debug!(
            "[CLIENT] ✓ Heartbeat service started (interval: {}s)",
            self.config.heartbeat_interval
        );
        Ok(())
    }

    /// Send sensory data to FEAGI
    ///
    /// # Arguments
    /// * `neuron_pairs` - Vector of (neuron_id, potential) pairs
    ///
    /// # Example
    /// ```ignore
    /// client.send_sensory_data(vec![
    ///     (0, 50.0),
    ///     (1, 75.0),
    ///     (2, 30.0),
    /// ])?;
    /// ```
    pub fn send_sensory_data(&self, neuron_pairs: Vec<(i32, f64)>) -> Result<()> {
        if !self.registered {
            return Err(SdkError::NotRegistered);
        }

        let socket = self
            .sensory_socket
            .as_ref()
            .ok_or_else(|| SdkError::Other("Sensory socket not initialized".to_string()))?;

        // ARCHITECTURE COMPLIANCE: Use binary XYZP format, NOT JSON
        // This serializes data using feagi_data_structures for cross-platform compatibility
        use feagi_structures::genomic::cortical_area::CorticalID;
        use feagi_structures::neuron_voxels::xyzp::{
            CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
        };

        // Get cortical area and dimensions from vision capability
        let vision_cap = self
            .config
            .capabilities
            .vision
            .as_ref()
            .ok_or_else(|| SdkError::Other("No vision capability configured".to_string()))?;

        let (width, _height) = vision_cap.dimensions;
        let cortical_area = &vision_cap.target_cortical_area;

        // Create CorticalID from area name
        let mut bytes = [b' '; 8];
        let name_bytes = cortical_area.as_bytes();
        let copy_len = name_bytes.len().min(8);
        bytes[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        let cortical_id = CorticalID::try_from_bytes(&bytes).map_err(|e| {
            SdkError::Other(format!("Invalid cortical ID '{}': {:?}", cortical_area, e))
        })?;

        // Convert flat neuron IDs to XYZP format
        let mut x_coords = Vec::with_capacity(neuron_pairs.len());
        let mut y_coords = Vec::with_capacity(neuron_pairs.len());
        let mut z_coords = Vec::with_capacity(neuron_pairs.len());
        let mut potentials = Vec::with_capacity(neuron_pairs.len());

        for (neuron_id, potential) in neuron_pairs {
            let neuron_id = neuron_id as u32;
            x_coords.push(neuron_id % (width as u32));
            y_coords.push(neuron_id / (width as u32));
            z_coords.push(0); // Single channel grayscale
            potentials.push(potential as f32);
        }

        let _neuron_count = x_coords.len(); // Reserved for future validation

        // Create neuron arrays from vectors
        let neuron_arrays =
            NeuronVoxelXYZPArrays::new_from_vectors(x_coords, y_coords, z_coords, potentials)
                .map_err(|e| SdkError::Other(format!("Failed to create neuron arrays: {:?}", e)))?;

        // Create cortical mapped data
        let mut cortical_mapped = CorticalMappedXYZPNeuronVoxels::new();
        cortical_mapped.insert(cortical_id, neuron_arrays);

        // Serialize to binary using FeagiByteContainer (version 2 container format)
        let mut byte_container = feagi_serialization::FeagiByteContainer::new_empty();
        byte_container
            .overwrite_byte_data_with_single_struct_data(&cortical_mapped, 0)
            .map_err(|e| SdkError::Other(format!("Failed to serialize to container: {:?}", e)))?;

        let buffer = byte_container.get_byte_ref().to_vec();

        // Send binary XYZP data (version 2 container format)
        socket.send(&buffer, 0)?;

        debug!(
            "[CLIENT] Sent {} bytes XYZP binary to {}",
            buffer.len(),
            cortical_area
        );
        Ok(())
    }

    /// Send pre-serialized sensory bytes to FEAGI (real-time semantics).
    ///
    /// This is intended for high-performance clients (e.g., Python SDK brain_input)
    /// that already produce FeagiByteContainer bytes via Rust-side encoding caches.
    ///
    /// Real-time policy:
    /// - Uses ZMQ DONTWAIT to avoid blocking the caller.
    /// - On backpressure (EAGAIN), the message is dropped (latest-only semantics).
    pub fn send_sensory_bytes(&self, bytes: Vec<u8>) -> Result<()> {
        if !self.registered {
            return Err(SdkError::NotRegistered);
        }

        let socket = self
            .sensory_socket
            .as_ref()
            .ok_or_else(|| SdkError::Other("Sensory socket not initialized".to_string()))?;

        match socket.send(&bytes, zmq::DONTWAIT) {
            Ok(()) => {
                debug!("[CLIENT] Sent {} bytes sensory (raw)", bytes.len());
                Ok(())
            }
            Err(zmq::Error::EAGAIN) => {
                // REAL-TIME: Drop on pressure (do not block and do not buffer history)
                static DROPPED: AtomicU64 = AtomicU64::new(0);
                static LAST_LOG_MS: AtomicU64 = AtomicU64::new(0);

                let dropped = DROPPED.fetch_add(1, Ordering::Relaxed) + 1;
                let now_ms = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                let last_ms = LAST_LOG_MS.load(Ordering::Relaxed);
                // Rate-limit warnings (max once per 5s) to avoid log spam on sustained pressure.
                if now_ms.saturating_sub(last_ms) >= 5_000
                    && LAST_LOG_MS
                        .compare_exchange(last_ms, now_ms, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok()
                {
                    warn!(
                        "[CLIENT] Sensory backpressure: dropped_messages={} last_payload_bytes={}",
                        dropped,
                        bytes.len()
                    );
                }

                Ok(())
            }
            Err(e) => Err(SdkError::Zmq(e)),
        }
    }

    /// Receive motor data from FEAGI (non-blocking)
    ///
    /// Returns None if no data is available.
    /// Motor data is in binary XYZP format (CorticalMappedXYZPNeuronVoxels).
    ///
    /// # Example
    /// ```ignore
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// if let Some(motor_data) = client.receive_motor_data()? {
    ///     // Process binary motor data
    ///     for (cortical_id, neurons) in motor_data.iter() {
    ///         println!("Motor area {:?}: {} neurons", cortical_id, neurons.len());
    ///     }
    /// }
    /// ```
    pub fn receive_motor_data(
        &self,
    ) -> Result<Option<feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels>> {
        use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;

        if !self.registered {
            return Err(SdkError::NotRegistered);
        }

        let socket = self.motor_socket.as_ref().ok_or_else(|| {
            info!("[CLIENT] ❌ receive_motor_data() called but motor_socket is None!");
            SdkError::Other("Motor socket not initialized (not a motor agent?)".to_string())
        })?;

        // Non-blocking receive (multipart: [topic/agent_id, data])
        // First frame is the topic (agent_id), second frame is the motor data
        match socket.recv_bytes(zmq::DONTWAIT) {
            Ok(topic) => {
                info!(
                    "[CLIENT] 📥 Received first frame: {} bytes: '{}'",
                    topic.len(),
                    String::from_utf8_lossy(&topic)
                );

                // Verify topic matches our agent_id (redundant due to SUB filter, but safe)
                if topic != self.config.agent_id.as_bytes() {
                    info!(
                        "[CLIENT] ⚠️ Received motor data for different agent: expected '{}', got '{}'",
                        self.config.agent_id,
                        String::from_utf8_lossy(&topic)
                    );
                    return Ok(None);
                }

                // Check if more frames are available (should be for multipart)
                let data = if socket.get_rcvmore().map_err(SdkError::Zmq)? {
                    info!(
                        "[CLIENT] 📥 More frames available, receiving second frame (motor data)..."
                    );
                    // Receive second frame (actual motor data)
                    let data = socket.recv_bytes(0).map_err(|e| {
                        info!("[CLIENT] ❌ Failed to receive second frame: {}", e);
                        SdkError::Zmq(e)
                    })?;
                    info!(
                        "[CLIENT] 📥 Received motor data frame: {} bytes",
                        data.len()
                    );
                    data
                } else {
                    info!("[CLIENT] ⚠️ NO MORE FRAMES! Old FEAGI (single-part message)");
                    info!(
                        "[CLIENT] 📥 Using first frame as motor data ({} bytes)",
                        topic.len()
                    );
                    // Fallback: treat first frame as data (backward compatibility with old FEAGI)
                    topic
                };

                // ARCHITECTURE COMPLIANCE: Deserialize binary XYZP motor data using FeagiByteContainer
                let mut byte_container = feagi_serialization::FeagiByteContainer::new_empty();
                let mut data_vec = data.to_vec();

                // Load bytes into container
                byte_container
                    .try_write_data_to_container_and_verify(&mut |bytes| {
                        std::mem::swap(bytes, &mut data_vec);
                        Ok(())
                    })
                    .map_err(|e| {
                        SdkError::Other(format!("Failed to load motor data bytes: {:?}", e))
                    })?;

                // Get number of structures (should be 1 for motor data)
                let num_structures = byte_container
                    .try_get_number_contained_structures()
                    .map_err(|e| {
                        SdkError::Other(format!("Failed to get structure count: {:?}", e))
                    })?;

                if num_structures == 0 {
                    return Ok(None);
                }

                // Extract first structure
                let boxed_struct =
                    byte_container
                        .try_create_new_struct_from_index(0)
                        .map_err(|e| {
                            SdkError::Other(format!("Failed to extract motor structure: {:?}", e))
                        })?;

                // Downcast to CorticalMappedXYZPNeuronVoxels
                let motor_data = boxed_struct
                    .as_any()
                    .downcast_ref::<CorticalMappedXYZPNeuronVoxels>()
                    .ok_or_else(|| {
                        SdkError::Other(
                            "Motor data is not CorticalMappedXYZPNeuronVoxels".to_string(),
                        )
                    })?
                    .clone();

                debug!(
                    "[CLIENT] ✓ Received motor data ({} bytes, {} areas)",
                    data.len(),
                    motor_data.len()
                );
                Ok(Some(motor_data))
            }
            Err(zmq::Error::EAGAIN) => {
                // No data available (FEAGI not publishing OR slow joiner syndrome)
                Ok(None)
            }
            Err(e) => {
                error!("[CLIENT] ❌ ZMQ error on motor receive: {}", e);
                Err(SdkError::Zmq(e))
            }
        }
    }

    /// Receive visualization data from FEAGI (non-blocking)
    ///
    /// Returns None if no data is available.
    ///
    /// # Example
    /// ```ignore
    /// if let Some(viz_data) = client.receive_visualization_data()? {
    ///     // Process neural activity data
    ///     println!("Visualization data size: {} bytes", viz_data.len());
    /// }
    /// ```
    pub fn receive_visualization_data(&self) -> Result<Option<Vec<u8>>> {
        if !self.registered {
            return Err(SdkError::NotRegistered);
        }

        let socket = self.viz_socket.as_ref().ok_or_else(|| {
            SdkError::Other(
                "Visualization socket not initialized (not a visualization/infrastructure agent?)"
                    .to_string(),
            )
        })?;

        // Non-blocking receive
        match socket.recv_bytes(zmq::DONTWAIT) {
            Ok(data) => {
                debug!(
                    "[CLIENT] ✓ Received visualization data ({} bytes)",
                    data.len()
                );
                Ok(Some(data))
            }
            Err(zmq::Error::EAGAIN) => Ok(None), // No data available
            Err(e) => Err(SdkError::Zmq(e)),
        }
    }

    /// Make a REST API request to FEAGI over ZMQ
    ///
    /// # Arguments
    /// * `method` - HTTP method (GET, POST, PUT, DELETE)
    /// * `route` - API route (e.g., "/v1/system/health_check")
    /// * `data` - Optional request body for POST/PUT requests
    ///
    /// # Example
    /// ```ignore
    /// // GET request
    /// let health = client.control_request("GET", "/v1/system/health_check", None)?;
    ///
    /// // POST request
    /// let data = serde_json::json!({"key": "value"});
    /// let response = client.control_request("POST", "/v1/some/endpoint", Some(data))?;
    /// ```
    pub fn control_request(
        &self,
        method: &str,
        route: &str,
        data: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        if !self.registered {
            return Err(SdkError::NotRegistered);
        }

        let socket = self.control_socket.as_ref().ok_or_else(|| {
            SdkError::Other(
                "Control socket not initialized (not an infrastructure agent?)".to_string(),
            )
        })?;

        // Prepare REST-over-ZMQ request
        let mut request = serde_json::json!({
            "method": method,
            "route": route,
            "headers": {"content-type": "application/json"},
        });

        if let Some(body) = data {
            request["body"] = body;
        }

        // Send request
        socket.send(request.to_string().as_bytes(), 0)?;

        // Wait for response
        let response_bytes = socket.recv_bytes(0)?;
        let response: serde_json::Value = serde_json::from_slice(&response_bytes)?;

        debug!("[CLIENT] ✓ Control request {} {} completed", method, route);
        Ok(response)
    }

    /// Check if agent is registered
    pub fn is_registered(&self) -> bool {
        self.registered
    }

    /// Get agent ID
    pub fn agent_id(&self) -> &str {
        &self.config.agent_id
    }
}

impl Drop for AgentClient {
    fn drop(&mut self) {
        debug!("[CLIENT] Dropping AgentClient: {}", self.config.agent_id);

        // Step 1: Stop heartbeat service first (this stops background threads)
        if let Some(mut heartbeat) = self.heartbeat.take() {
            debug!("[CLIENT] Stopping heartbeat service before cleanup");
            heartbeat.stop();
            debug!("[CLIENT] Heartbeat service stopped");
        }

        // Step 2: Deregister from FEAGI (after threads stopped)
        if self.registered {
            debug!("[CLIENT] Deregistering agent: {}", self.config.agent_id);
            if let Err(e) = self.deregister() {
                warn!("[CLIENT] Deregistration failed during drop: {}", e);
                // Continue cleanup even if deregistration fails
            }
        }

        // Step 3: Sockets will be dropped automatically
        debug!(
            "[CLIENT] AgentClient dropped cleanly: {}",
            self.config.agent_id
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_io::AgentType;

    #[test]
    fn test_client_creation() {
        let config = AgentConfig::new("test_agent", AgentType::Sensory)
            .with_vision_capability("camera", (640, 480), 3, "i_vision")
            .with_registration_endpoint("tcp://localhost:8000")
            .with_sensory_endpoint("tcp://localhost:5558");

        let client = AgentClient::new(config);
        assert!(client.is_ok());

        let client = client.unwrap();
        assert!(!client.is_registered());
        assert_eq!(client.agent_id(), "test_agent");
    }

    // Note: Full integration tests require a running FEAGI instance
    // and should be in separate integration test files
}
