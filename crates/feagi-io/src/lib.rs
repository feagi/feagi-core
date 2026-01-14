// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! FEAGI I/O System
//!
//! Handles all agent I/O: registration, ZMQ, SHM, heartbeat tracking.
//!
//! # Architecture
//!
//! This crate follows a hybrid module structure:
//! - **`core/`**: Shared types, agent registry, configuration
//! - **`blocking/`**: Infrastructure for blocking I/O transports (threads, channels, compression)
//! - **`nonblocking/`**: Infrastructure for async/await transports (tokio, async channels)
//! - **`transports/`**: Specific transport implementations (ZMQ, UDP, SHM, WebSocket, RTOS)
//!
//! # Example
//!
//! ```no_run
//! use feagi_io::{IOSystem, IOConfig};
//!
//! let io_system = IOSystem::new().unwrap();
//! io_system.start().unwrap();
//!
//! // Publish visualization data
//! let data = vec![1, 2, 3];
//! io_system.publish_visualization(&data).unwrap();
//!
//! io_system.stop().unwrap();
//! ```

/// Crate version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use feagi_structures::FeagiSignal;
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;

#[cfg(feature = "udp-transport")]
use tokio::runtime::Runtime;

// Import NonBlockingTransport trait for UDP transport methods
#[cfg(feature = "udp-transport")]
use crate::nonblocking::transport::NonBlockingTransport;
use tracing::{debug, error, info, trace, warn};

/// Stream state for dynamic start/stop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StreamState {
    /// Stream is stopped (socket closed, thread terminated)
    Stopped,
    /// Stream is starting (transitioning to Running)
    Starting,
    /// Stream is running (socket bound, thread active)
    Running,
    /// Stream is stopping (transitioning to Stopped)
    Stopping,
}

/// Minimal PNS clone for callbacks (only the Arc fields needed for dynamic gating)
#[derive(Clone)]
struct IOSystemForCallbacks {
    npu_ref: Arc<
        Mutex<
            Option<Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>>,
        >,
    >,
    agent_registry: Arc<RwLock<AgentRegistry>>,
    #[cfg(feature = "zmq-transport")]
    zmq_streams: Arc<Mutex<Option<ZmqStreams>>>,
    #[cfg(feature = "websocket-transport")]
    websocket_streams: Arc<Mutex<Option<WebSocketStreams>>>,
    #[allow(dead_code)]
    websocket_enabled: bool,
    #[allow(dead_code)]
    websocket_viz_port: u16,
    sensory_stream_state: Arc<Mutex<StreamState>>,
    motor_stream_state: Arc<Mutex<StreamState>>,
    viz_stream_state: Arc<Mutex<StreamState>>,
}

impl IOSystemForCallbacks {
    fn is_genome_loaded(&self) -> bool {
        if let Some(npu_arc) = self.npu_ref.lock().as_ref() {
            npu_arc.lock().unwrap().is_genome_loaded()
        } else {
            false
        }
    }

    fn should_sensory_stream_run(&self) -> bool {
        self.is_genome_loaded() && self.agent_registry.read().has_sensory_agents()
    }

    fn should_motor_stream_run(&self) -> bool {
        self.is_genome_loaded() && self.agent_registry.read().has_motor_agents()
    }

    fn should_viz_stream_run(&self) -> bool {
        self.is_genome_loaded() && self.agent_registry.read().has_visualization_agents()
    }

    fn get_active_viz_transports(&self) -> Vec<String> {
        let registry = self.agent_registry.read();
        let mut transports = std::collections::HashSet::new();

        for agent in registry.get_all() {
            if matches!(
                agent.agent_type,
                AgentType::Visualization | AgentType::Infrastructure
            ) {
                if let Some(ref chosen) = agent.chosen_transport {
                    transports.insert(chosen.clone());
                } else {
                    // Legacy: if no chosen_transport, assume ZMQ
                    transports.insert("zmq".to_string());
                }
            }
        }

        transports.into_iter().collect()
    }

    fn try_start_sensory_stream(&self) {
        let mut state = self.sensory_stream_state.lock();

        // Check if already starting/running
        if *state != StreamState::Stopped {
            debug!(
                "[PNS-DYNAMIC] Sensory stream already {:?}, not starting",
                *state
            );
            return;
        }

        // Check conditions and log WHY if not met
        let genome_loaded = self.is_genome_loaded();
        let has_agents = self.agent_registry.read().has_sensory_agents();
        let agent_count = self.agent_registry.read().count_sensory_agents();

        if !genome_loaded || !has_agents {
            warn!("⚠️  [PNS-DYNAMIC] Cannot start sensory stream:");
            warn!("    - Genome loaded: {} (has neurons)", genome_loaded);
            warn!(
                "    - Sensory agents registered: {} (count: {})",
                has_agents, agent_count
            );
            return;
        }

        *state = StreamState::Starting;
        drop(state);

        // Double-check before actually starting
        if !self.should_sensory_stream_run() {
            *self.sensory_stream_state.lock() = StreamState::Stopped;
            warn!("⚠️  [PNS-DYNAMIC] Sensory stream conditions changed during startup, aborting");
            return;
        }

        #[cfg(feature = "zmq-transport")]
        {
            if let Some(streams) = self.zmq_streams.lock().as_ref() {
                match streams.start_sensory_stream() {
                    Ok(()) => {
                        *self.sensory_stream_state.lock() = StreamState::Running;
                        let count = self.agent_registry.read().count_sensory_agents();
                        info!("🟢 [PNS-DYNAMIC] Sensory stream started: {} agents", count);
                    }
                    Err(e) => {
                        error!("❌ [PNS-DYNAMIC] Failed to start sensory: {}", e);
                        *self.sensory_stream_state.lock() = StreamState::Stopped;
                    }
                }
            }
        }
    }

    fn try_stop_sensory_stream(&self) {
        let mut state = self.sensory_stream_state.lock();
        if *state != StreamState::Running {
            return;
        }

        *state = StreamState::Stopping;
        drop(state);

        #[cfg(feature = "zmq-transport")]
        {
            if let Some(streams) = self.zmq_streams.lock().as_ref() {
                match streams.stop_sensory_stream() {
                    Ok(()) => {
                        *self.sensory_stream_state.lock() = StreamState::Stopped;
                        warn!("🔴 [PNS-DYNAMIC] Sensory stream stopped");
                    }
                    Err(e) => {
                        error!("❌ [PNS-DYNAMIC] Failed to stop sensory: {}", e);
                        *self.sensory_stream_state.lock() = StreamState::Running;
                    }
                }
            }
        }
    }

    fn try_start_motor_stream(&self) {
        let mut state = self.motor_stream_state.lock();

        if *state != StreamState::Stopped {
            debug!(
                "[PNS-DYNAMIC] Motor stream already {:?}, not starting",
                *state
            );
            return;
        }

        let genome_loaded = self.is_genome_loaded();
        let has_agents = self.agent_registry.read().has_motor_agents();
        let agent_count = self.agent_registry.read().count_motor_agents();

        if !genome_loaded || !has_agents {
            warn!("⚠️  [PNS-DYNAMIC] Cannot start motor stream:");
            warn!("    - Genome loaded: {} (has neurons)", genome_loaded);
            warn!(
                "    - Motor agents registered: {} (count: {})",
                has_agents, agent_count
            );
            return;
        }

        *state = StreamState::Starting;
        drop(state);

        if !self.should_motor_stream_run() {
            *self.motor_stream_state.lock() = StreamState::Stopped;
            warn!("⚠️  [PNS-DYNAMIC] Motor stream conditions changed during startup, aborting");
            return;
        }

        #[cfg(feature = "zmq-transport")]
        {
            if let Some(streams) = self.zmq_streams.lock().as_ref() {
                match streams.start_motor_stream() {
                    Ok(()) => {
                        *self.motor_stream_state.lock() = StreamState::Running;
                        let count = self.agent_registry.read().count_motor_agents();
                        info!("🟢 [PNS-DYNAMIC] Motor stream started: {} agents", count);
                    }
                    Err(e) => {
                        error!("❌ [PNS-DYNAMIC] Failed to start motor: {}", e);
                        *self.motor_stream_state.lock() = StreamState::Stopped;
                    }
                }
            }
        }
    }

    fn try_stop_motor_stream(&self) {
        let mut state = self.motor_stream_state.lock();
        if *state != StreamState::Running {
            return;
        }

        *state = StreamState::Stopping;
        drop(state);

        #[cfg(feature = "zmq-transport")]
        {
            if let Some(streams) = self.zmq_streams.lock().as_ref() {
                match streams.stop_motor_stream() {
                    Ok(()) => {
                        *self.motor_stream_state.lock() = StreamState::Stopped;
                        warn!("🔴 [PNS-DYNAMIC] Motor stream stopped");
                    }
                    Err(e) => {
                        error!("❌ [PNS-DYNAMIC] Failed to stop motor: {}", e);
                        *self.motor_stream_state.lock() = StreamState::Running;
                    }
                }
            }
        }
    }

    fn try_start_viz_stream(&self) {
        let mut state = self.viz_stream_state.lock();

        // Allow restart if stuck in Starting (recovery from failed startup)
        if *state == StreamState::Running {
            debug!("[PNS-DYNAMIC] Viz stream already Running, not restarting");
            return;
        }

        if *state == StreamState::Starting {
            warn!("⚠️ [PNS-DYNAMIC] Viz stream stuck in Starting state - forcing restart");
            *state = StreamState::Stopped; // Reset to allow restart
        }

        let genome_loaded = self.is_genome_loaded();
        let has_agents = self.agent_registry.read().has_visualization_agents();
        let agent_count = self.agent_registry.read().count_visualization_agents();

        if !genome_loaded || !has_agents {
            warn!("⚠️  [PNS-DYNAMIC] Cannot start visualization stream:");
            warn!("    - Genome loaded: {} (has neurons)", genome_loaded);
            warn!(
                "    - Visualization agents registered: {} (count: {})",
                has_agents, agent_count
            );
            return;
        }

        *state = StreamState::Starting;
        drop(state);

        if !self.should_viz_stream_run() {
            *self.viz_stream_state.lock() = StreamState::Stopped;
            warn!("⚠️  [PNS-DYNAMIC] Viz stream conditions changed during startup, aborting");
            return;
        }

        // Determine which transports need to be started based on agent preferences
        let active_transports = self.get_active_viz_transports();
        let needs_zmq = active_transports.contains(&"zmq".to_string())
            || active_transports.contains(&"shm".to_string())
            || active_transports.is_empty(); // Default to ZMQ if no preference
        let needs_websocket = active_transports.contains(&"websocket".to_string());

        info!(
            "🔍 [PNS-DYNAMIC] Starting viz streams for transports: {:?} (ZMQ: {}, WebSocket: {})",
            active_transports, needs_zmq, needs_websocket
        );

        // Start ZMQ viz stream only if agents are using it
        #[cfg(feature = "zmq-transport")]
        {
            if needs_zmq {
                info!("🔍 [PNS-DYNAMIC] Starting ZMQ viz stream (agents using ZMQ/SHM)...");
                if let Some(streams) = self.zmq_streams.lock().as_ref() {
                    match streams.start_viz_stream() {
                        Ok(()) => {
                            info!("🟢 [PNS-DYNAMIC] ZMQ viz stream started");
                        }
                        Err(e) => {
                            error!("❌ [PNS-DYNAMIC] Failed to start ZMQ viz: {}", e);
                        }
                    }
                }
            } else {
                info!("⏭️ [PNS-DYNAMIC] Skipping ZMQ viz stream (no agents using ZMQ/SHM)");
            }
        }

        // Start WebSocket viz stream only if agents are using it
        #[cfg(feature = "websocket-transport")]
        {
            if needs_websocket && self.websocket_enabled {
                info!("🔍 [PNS-DYNAMIC] Starting WebSocket viz stream (agents using WebSocket)...");
                let ws_lock = self.websocket_streams.lock();
                if let Some(ref streams) = *ws_lock {
                    match streams.start_data_streams() {
                        Ok(()) => {
                            info!(
                                "🟢 [PNS-DYNAMIC] WebSocket viz stream started on port {}",
                                self.websocket_viz_port
                            );
                        }
                        Err(e) => {
                            error!("❌ [PNS-DYNAMIC] Failed to start WebSocket viz: {}", e);
                        }
                    }
                    drop(ws_lock);
                } else {
                    warn!("⚠️ [PNS-DYNAMIC] WebSocket enabled but streams not initialized!");
                }
            } else if needs_websocket && !self.websocket_enabled {
                error!("❌ [PNS-DYNAMIC] Agent requested WebSocket but it's disabled in config!");
            } else {
                info!("⏭️ [PNS-DYNAMIC] Skipping WebSocket viz stream (no agents using WebSocket)");
            }
        }

        // Mark as running after starting all enabled transports
        info!("🔍 [PNS-DYNAMIC] Setting viz stream state to Running...");
        *self.viz_stream_state.lock() = StreamState::Running;
        let count = self.agent_registry.read().count_visualization_agents();
        info!("🟢 [PNS-DYNAMIC] Viz streams started for {} agents", count);
    }

    fn try_stop_viz_stream(&self) {
        let mut state = self.viz_stream_state.lock();
        if *state != StreamState::Running {
            return;
        }

        *state = StreamState::Stopping;
        drop(state);

        #[cfg(feature = "zmq-transport")]
        {
            if let Some(streams) = self.zmq_streams.lock().as_ref() {
                match streams.stop_viz_stream() {
                    Ok(()) => {
                        *self.viz_stream_state.lock() = StreamState::Stopped;
                        warn!("🔴 [PNS-DYNAMIC] Viz stream stopped");
                    }
                    Err(e) => {
                        error!("❌ [PNS-DYNAMIC] Failed to stop viz: {}", e);
                        *self.viz_stream_state.lock() = StreamState::Running;
                    }
                }
            }
        }
    }

    fn evaluate_all_stream_states(&self) {
        if self.should_sensory_stream_run() {
            self.try_start_sensory_stream();
        } else {
            self.try_stop_sensory_stream();
        }

        if self.should_motor_stream_run() {
            self.try_start_motor_stream();
        } else {
            self.try_stop_motor_stream();
        }

        if self.should_viz_stream_run() {
            self.try_start_viz_stream();
        } else {
            self.try_stop_viz_stream();
        }
    }

    fn on_agent_registered_dynamic(&self, agent_id: &str) {
        info!(
            "🔄 [PNS-DYNAMIC] Agent '{}' registered, evaluating stream conditions...",
            agent_id
        );

        // Log current state BEFORE evaluation
        let genome_loaded = self.is_genome_loaded();
        let sensory_count = self.agent_registry.read().count_sensory_agents();
        let motor_count = self.agent_registry.read().count_motor_agents();
        let viz_count = self.agent_registry.read().count_visualization_agents();

        info!("🔍 [PNS-DYNAMIC] Current state:");
        info!("    - Genome loaded: {}", genome_loaded);
        info!("    - Sensory agents: {}", sensory_count);
        info!("    - Motor agents: {}", motor_count);
        info!("    - Visualization agents: {}", viz_count);
        info!(
            "    - Sensory stream state: {:?}",
            *self.sensory_stream_state.lock()
        );
        info!(
            "    - Motor stream state: {:?}",
            *self.motor_stream_state.lock()
        );
        info!(
            "    - Viz stream state: {:?}",
            *self.viz_stream_state.lock()
        );

        self.evaluate_all_stream_states();
    }

    fn on_agent_deregistered_dynamic(&self, agent_id: &str) {
        info!(
            "🔄 [PNS-DYNAMIC] Agent '{}' deregistered, evaluating stream conditions...",
            agent_id
        );
        self.evaluate_all_stream_states();
    }
}

// Core modules (shared across all transports)
pub mod blocking;
pub mod core;

#[cfg(feature = "udp-transport")]
pub mod nonblocking;

pub mod transports;

// Connectome I/O (file I/O and future network transport)
// Types are in feagi-npu-neural::types::connectome
// File I/O functions are here in feagi-io
#[cfg(feature = "connectome-serialization")]
pub mod connectome;

// Re-export commonly used types from core
pub use core::{
    AgentCapabilities, AgentDisconnectedEvent, AgentInfo, AgentRegisteredEvent, AgentRegistry,
    AgentTransport, AgentType, HeartbeatTracker, IOConfig, IOError, MotorCapability,
    MotorCommandEvent, MotorUnit, MotorUnitSpec, RegistrationHandler, RegistrationRequest,
    RegistrationResponse, Result, SensoryCapability, SensoryDataEvent, SensoryUnit, SharedFBC,
    StreamType, TransportConfig, TransportMode, VisionCapability, VisualizationCapability,
    VisualizationReadyEvent, WebSocketConfig,
};

// Re-export transport-specific types
#[cfg(feature = "zmq-transport")]
pub use transports::zmq::{
    MotorStream, RestStream, SensoryStream, VisualizationOverflowStrategy, VisualizationSendConfig,
    VisualizationStream, ZmqStreams,
};

#[cfg(feature = "udp-transport")]
pub use transports::udp::{UdpConfig, UdpTransport};

#[cfg(feature = "websocket-transport")]
pub use transports::websocket::WebSocketStreams;

// Keep shm module at root for now (will be moved to transports/ in future)
pub mod shm;

/// Main PNS - manages all agent I/O
///
/// # Event-Driven Architecture
///
/// PNS uses FeagiSignal for decoupled communication:
///
/// **Incoming Signals (Burst Engine → PNS)**:
/// - `visualization_ready`: Burst engine emits when neural activity is ready
/// - `motor_commands`: Burst engine emits when motor outputs are computed
///
/// **Outgoing Signals (PNS → Burst Engine)**:
/// - `sensory_data_received`: IOSystem emits when sensory data arrives from agent
/// - `agent_registered`: IOSystem emits when new agent registers
/// - `agent_disconnected`: IOSystem emits when agent disconnects/times out
///
/// # Example
/// ```no_run
/// use feagi_io::IOSystem;
///
/// let pns = IOSystem::new().unwrap();
///
/// // Burst engine subscribes to PNS outgoing signals
/// pns.sensory_data_received.lock().connect(|event| {
///     println!("Received sensory data from {}", event.agent_id);
/// });
///
/// // PNS subscribes to burst engine incoming signals
/// // (burst engine would call pns.visualization_ready.lock().connect(...))
///
/// pns.start().unwrap();
/// ```
pub struct IOSystem {
    config: IOConfig,
    agent_registry: Arc<RwLock<AgentRegistry>>,
    registration_handler: Arc<Mutex<RegistrationHandler>>,
    heartbeat_tracker: Arc<Mutex<HeartbeatTracker>>,

    // === Transport Layer ===
    /// ZMQ streams (blocking, TCP-based)
    #[cfg(feature = "zmq-transport")]
    zmq_streams: Arc<Mutex<Option<ZmqStreams>>>,

    /// WebSocket streams (async, web-compatible)
    #[cfg(feature = "websocket-transport")]
    websocket_streams: Arc<Mutex<Option<WebSocketStreams>>>,

    /// UDP visualization transport (async, best-effort)
    #[cfg(feature = "udp-transport")]
    udp_viz_transport: Arc<Mutex<Option<UdpTransport>>>,
    /// UDP sensory transport (async, best-effort)
    #[cfg(feature = "udp-transport")]
    udp_sensory_transport: Arc<Mutex<Option<UdpTransport>>>,
    /// Tokio runtime for async transports (Arc-wrapped for sharing with UDP transports)
    #[cfg(feature = "udp-transport")]
    async_runtime: Arc<Mutex<Option<Arc<Runtime>>>>,

    running: Arc<RwLock<bool>>,
    /// Optional reference to burst engine's sensory agent manager for SHM I/O
    sensory_agent_manager:
        Arc<Mutex<Option<Arc<std::sync::Mutex<feagi_npu_burst_engine::AgentManager>>>>>,

    // === Dynamic Stream Gating ===
    /// NPU reference for genome state checking (dynamic gating)
    npu_ref: Arc<
        Mutex<
            Option<Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>>,
        >,
    >,
    /// Sensory stream state
    sensory_stream_state: Arc<Mutex<StreamState>>,
    /// Motor stream state
    motor_stream_state: Arc<Mutex<StreamState>>,
    /// Visualization stream state
    viz_stream_state: Arc<Mutex<StreamState>>,

    // === Incoming Signals (Burst Engine → PNS) ===
    /// Signal for visualization data ready to be published
    pub visualization_ready: Arc<Mutex<FeagiSignal<VisualizationReadyEvent>>>,
    /// Signal for motor commands ready to be sent
    pub motor_commands: Arc<Mutex<FeagiSignal<MotorCommandEvent>>>,

    // === Outgoing Signals (PNS → Burst Engine) ===
    /// Signal emitted when sensory data is received from an agent
    pub sensory_data_received: Arc<Mutex<FeagiSignal<SensoryDataEvent>>>,
    /// Signal emitted when a new agent registers
    pub agent_registered: Arc<Mutex<FeagiSignal<AgentRegisteredEvent>>>,
    /// Signal emitted when an agent disconnects
    pub agent_disconnected: Arc<Mutex<FeagiSignal<AgentDisconnectedEvent>>>,
}

impl IOSystem {
    /// Create a new PNS with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(IOConfig::default())
    }

    /// Create a new I/O system with custom configuration
    pub fn with_config(config: IOConfig) -> Result<Self> {
        let agent_registry = Arc::new(RwLock::new(AgentRegistry::with_defaults()));
        let heartbeat_tracker = Arc::new(Mutex::new(HeartbeatTracker::new()));

        // Extract ports from config addresses (e.g., "tcp://0.0.0.0:5564" -> 5564)
        let motor_port = config
            .zmq_motor_address
            .split(':')
            .next_back()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(5564); // @architecture:acceptable - emergency fallback
        let viz_port = config
            .zmq_viz_address
            .split(':')
            .next_back()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(5562); // @architecture:acceptable - emergency fallback

        // Extract sensory port from config
        let sensory_port = config
            .zmq_sensory_address
            .split(':')
            .next_back()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(5558); // @architecture:acceptable - emergency fallback

        // Extract registration/rest port from config (used by AgentClient registration channel)
        let registration_port = config
            .zmq_rest_address
            .split(':')
            .next_back()
            .and_then(|s| s.parse::<u16>().ok())
            .ok_or_else(|| {
                core::types::IOError::Config(format!(
                    "Invalid zmq_rest_address (expected host:port): {}",
                    config.zmq_rest_address
                ))
            })?;

        info!(
            "🦀 [PNS] Port configuration: registration={}, sensory={}, motor={}, viz={}",
            registration_port, sensory_port, motor_port, viz_port
        );

        let mut registration_handler_instance = RegistrationHandler::new(
            Arc::clone(&agent_registry),
            registration_port,
            sensory_port,
            motor_port,
            viz_port,
        );

        // Configure WebSocket transport
        registration_handler_instance.set_websocket_config(
            config.websocket.enabled,
            config.websocket.host.clone(),
            config.websocket.sensory_port,
            config.websocket.motor_port,
            config.websocket.visualization_port,
            config.websocket.registration_port,
        );

        let registration_handler = Arc::new(Mutex::new(registration_handler_instance));

        Ok(Self {
            config,
            agent_registry,
            registration_handler,
            heartbeat_tracker,
            // Transport layer
            #[cfg(feature = "zmq-transport")]
            zmq_streams: Arc::new(Mutex::new(None)),
            #[cfg(feature = "websocket-transport")]
            websocket_streams: Arc::new(Mutex::new(None)),
            #[cfg(feature = "udp-transport")]
            udp_viz_transport: Arc::new(Mutex::new(None)),
            #[cfg(feature = "udp-transport")]
            udp_sensory_transport: Arc::new(Mutex::new(None)),
            #[cfg(feature = "udp-transport")]
            async_runtime: Arc::new(Mutex::new(None)),
            running: Arc::new(RwLock::new(false)),
            sensory_agent_manager: Arc::new(Mutex::new(None)),
            // Dynamic stream gating
            npu_ref: Arc::new(Mutex::new(None)),
            sensory_stream_state: Arc::new(Mutex::new(StreamState::Stopped)),
            motor_stream_state: Arc::new(Mutex::new(StreamState::Stopped)),
            viz_stream_state: Arc::new(Mutex::new(StreamState::Stopped)),
            // Initialize signals
            visualization_ready: Arc::new(Mutex::new(FeagiSignal::new())),
            motor_commands: Arc::new(Mutex::new(FeagiSignal::new())),
            sensory_data_received: Arc::new(Mutex::new(FeagiSignal::new())),
            agent_registered: Arc::new(Mutex::new(FeagiSignal::new())),
            agent_disconnected: Arc::new(Mutex::new(FeagiSignal::new())),
        })
    }

    /// Wire dynamic gating callbacks to registration handler
    /// Must be called after PNS is wrapped in Arc (in main.rs)
    pub fn wire_dynamic_gating_callbacks(pns: &Arc<Self>) {
        let pns_weak = Arc::downgrade(pns);

        let on_registered = {
            let pns_weak = pns_weak.clone();
            move |agent_id: String| {
                if let Some(pns_ref) = pns_weak.upgrade() {
                    pns_ref.on_agent_registered_dynamic(&agent_id);
                }
            }
        };
        let on_deregistered = {
            let pns_weak = pns_weak.clone();
            move |agent_id: String| {
                if let Some(pns_ref) = pns_weak.upgrade() {
                    pns_ref.on_agent_deregistered_dynamic(&agent_id);
                }
            }
        };

        // Register callbacks using the setter methods
        pns.registration_handler
            .lock()
            .set_on_agent_registered_dynamic(on_registered);
        pns.registration_handler
            .lock()
            .set_on_agent_deregistered_dynamic(on_deregistered);
        info!("🦀 [PNS] Dynamic gating callbacks registered with RegistrationHandler");
    }

    /// Set the sensory agent manager (for SHM I/O coordination)
    /// Should be called before starting the PNS
    pub fn set_sensory_agent_manager(
        &self,
        manager: Arc<std::sync::Mutex<feagi_npu_burst_engine::AgentManager>>,
    ) {
        *self.sensory_agent_manager.lock() = Some(manager.clone());
        // Also propagate to registration handler
        self.registration_handler
            .lock()
            .set_sensory_agent_manager(manager);
        info!("🦀 [PNS] Sensory agent manager connected for SHM I/O");
    }

    /// Set the burst runner (for motor subscription tracking)
    /// Should be called after creating BurstLoopRunner
    pub fn set_burst_runner(
        &self,
        runner: Arc<parking_lot::RwLock<feagi_npu_burst_engine::BurstLoopRunner>>,
    ) {
        // Propagate to registration handler for motor subscription management
        self.registration_handler.lock().set_burst_runner(runner);
        info!("🦀 [PNS] Burst runner connected for motor subscriptions");
    }

    /// Set NPU reference for dynamic stream gating
    /// Should be called during initialization, before starting streams
    pub fn set_npu_for_gating(
        &self,
        npu: Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
    ) {
        *self.npu_ref.lock() = Some(Arc::clone(&npu));
        info!("🦀 [PNS] NPU connected for dynamic stream gating (with lock tracing)");
    }

    /// Connect the Rust NPU to the sensory stream for direct injection
    /// Should be called after starting the PNS
    #[cfg(feature = "zmq-transport")]
    pub fn connect_npu_to_sensory_stream(
        &self,
        npu: Arc<feagi_npu_burst_engine::TracingMutex<feagi_npu_burst_engine::DynamicNPU>>,
    ) {
        if let Some(streams) = self.zmq_streams.lock().as_ref() {
            streams.get_sensory_stream().set_npu(npu);
            info!("🦀 [PNS] NPU connected to sensory stream for direct injection");
        } else {
            info!("🦀 [PNS] [ERR] Cannot connect NPU: ZMQ streams not started");
        }
    }

    /// Connect the Rust NPU to the API control stream for direct queries (zero GIL contention)
    /// Should be called after starting the PNS
    #[cfg(feature = "zmq-transport")]
    pub fn connect_npu_to_api_control_stream(
        &self,
        npu: Arc<std::sync::Mutex<feagi_npu_burst_engine::DynamicNPU>>,
    ) {
        if let Some(streams) = self.zmq_streams.lock().as_mut() {
            streams.get_api_control_stream_mut().set_npu(npu);
            info!("🦀 [PNS] NPU connected to API control stream for direct queries");
        } else {
            info!("🦀 [PNS] [ERR] Cannot connect NPU: ZMQ streams not started");
        }
    }

    /// Set RPC callback for generic CoreAPIService method calls
    pub fn set_api_rpc_callback<F>(&self, callback: F)
    where
        F: Fn(&str, serde_json::Value) -> std::result::Result<serde_json::Value, String>
            + Send
            + Sync
            + 'static,
    {
        if let Some(streams) = self.zmq_streams.lock().as_mut() {
            // Wrap callback to ensure it matches the expected signature
            streams
                .get_api_control_stream_mut()
                .set_rpc_callback(move |method, payload| callback(method, payload));
            info!("🦀 [PNS] RPC callback registered for CoreAPIService");
        } else {
            info!("🦀 [PNS] [ERR] Cannot set RPC callback: ZMQ streams not started");
        }
    }

    /// Set callback for agent registration events (for Python integration)
    pub fn set_on_agent_registered<F>(&self, callback: F)
    where
        F: Fn(String, String, String) + Send + Sync + 'static,
    {
        self.registration_handler
            .lock()
            .set_on_agent_registered(callback);
    }

    /// Set callback for agent deregistration events (for Python integration)
    pub fn set_on_agent_deregistered<F>(&self, callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.registration_handler
            .lock()
            .set_on_agent_deregistered(callback);
    }

    // === Dynamic Stream Gating - Condition Checking ===

    /// Check if genome is loaded in NPU
    fn is_genome_loaded(&self) -> bool {
        if let Some(npu_arc) = self.npu_ref.lock().as_ref() {
            npu_arc.lock().unwrap().is_genome_loaded()
        } else {
            false
        }
    }

    /// Check if sensory stream should be running
    fn should_sensory_stream_run(&self) -> bool {
        self.is_genome_loaded() && self.agent_registry.read().has_sensory_agents()
    }

    /// Check if motor stream should be running
    fn should_motor_stream_run(&self) -> bool {
        self.is_genome_loaded() && self.agent_registry.read().has_motor_agents()
    }

    /// Check if visualization stream should be running
    fn should_viz_stream_run(&self) -> bool {
        self.is_genome_loaded() && self.agent_registry.read().has_visualization_agents()
    }

    // === Dynamic Stream Gating - Stream Control ===

    /// Dynamically start sensory stream if conditions are met
    fn try_start_sensory_stream(&self) {
        let mut state = self.sensory_stream_state.lock();
        match *state {
            StreamState::Stopped => {
                if !self.should_sensory_stream_run() {
                    return; // Conditions not met
                }

                *state = StreamState::Starting;
                drop(state);

                // Double-check conditions after state transition (race protection)
                if !self.should_sensory_stream_run() {
                    *self.sensory_stream_state.lock() = StreamState::Stopped;
                    return;
                }

                // Start stream
                #[cfg(feature = "zmq-transport")]
                {
                    if let Some(streams) = self.zmq_streams.lock().as_ref() {
                        match streams.start_sensory_stream() {
                            Ok(()) => {
                                *self.sensory_stream_state.lock() = StreamState::Running;
                                let agent_count = self.agent_registry.read().count_sensory_agents();
                                info!("🟢 [PNS-DYNAMIC] Sensory stream started: genome loaded, {} agents registered", agent_count);
                            }
                            Err(e) => {
                                error!("❌ [PNS-DYNAMIC] Failed to start sensory stream: {}", e);
                                *self.sensory_stream_state.lock() = StreamState::Stopped;
                            }
                        }
                    }
                }
            }
            _ => {
                debug!("[PNS-DYNAMIC] Sensory stream already starting/running");
            }
        }
    }

    /// Dynamically stop sensory stream
    fn try_stop_sensory_stream(&self) {
        let mut state = self.sensory_stream_state.lock();
        match *state {
            StreamState::Running => {
                *state = StreamState::Stopping;
                drop(state);

                #[cfg(feature = "zmq-transport")]
                {
                    if let Some(streams) = self.zmq_streams.lock().as_ref() {
                        match streams.stop_sensory_stream() {
                            Ok(()) => {
                                *self.sensory_stream_state.lock() = StreamState::Stopped;
                                warn!("🔴 [PNS-DYNAMIC] Sensory stream stopped: conditions no longer met");
                            }
                            Err(e) => {
                                error!("❌ [PNS-DYNAMIC] Failed to stop sensory stream: {}", e);
                                *self.sensory_stream_state.lock() = StreamState::Running;
                            }
                        }
                    }
                }
            }
            _ => {
                debug!("[PNS-DYNAMIC] Sensory stream not running");
            }
        }
    }

    /// Dynamically start motor stream if conditions are met
    fn try_start_motor_stream(&self) {
        let mut state = self.motor_stream_state.lock();
        if *state == StreamState::Stopped {
            if !self.should_motor_stream_run() {
                return;
            }

            *state = StreamState::Starting;
            drop(state);

            if !self.should_motor_stream_run() {
                *self.motor_stream_state.lock() = StreamState::Stopped;
                return;
            }

            #[cfg(feature = "zmq-transport")]
            {
                if let Some(streams) = self.zmq_streams.lock().as_ref() {
                    match streams.start_motor_stream() {
                        Ok(()) => {
                            *self.motor_stream_state.lock() = StreamState::Running;
                            let agent_count = self.agent_registry.read().count_motor_agents();
                            info!("🟢 [PNS-DYNAMIC] Motor stream started: genome loaded, {} agents registered", agent_count);
                        }
                        Err(e) => {
                            error!("❌ [PNS-DYNAMIC] Failed to start motor stream: {}", e);
                            *self.motor_stream_state.lock() = StreamState::Stopped;
                        }
                    }
                }
            }
        }
    }

    /// Dynamically stop motor stream
    fn try_stop_motor_stream(&self) {
        let mut state = self.motor_stream_state.lock();
        if *state == StreamState::Running {
            *state = StreamState::Stopping;
            drop(state);

            #[cfg(feature = "zmq-transport")]
            {
                if let Some(streams) = self.zmq_streams.lock().as_ref() {
                    match streams.stop_motor_stream() {
                        Ok(()) => {
                            *self.motor_stream_state.lock() = StreamState::Stopped;
                            warn!(
                                "🔴 [PNS-DYNAMIC] Motor stream stopped: conditions no longer met"
                            );
                        }
                        Err(e) => {
                            error!("❌ [PNS-DYNAMIC] Failed to stop motor stream: {}", e);
                            *self.motor_stream_state.lock() = StreamState::Running;
                        }
                    }
                }
            }
        }
    }

    /// Dynamically start visualization stream if conditions are met
    fn try_start_viz_stream(&self) {
        let mut state = self.viz_stream_state.lock();
        if *state == StreamState::Stopped {
            if !self.should_viz_stream_run() {
                return;
            }

            *state = StreamState::Starting;
            drop(state);

            if !self.should_viz_stream_run() {
                *self.viz_stream_state.lock() = StreamState::Stopped;
                return;
            }

            #[cfg(feature = "zmq-transport")]
            {
                if let Some(streams) = self.zmq_streams.lock().as_ref() {
                    match streams.start_viz_stream() {
                        Ok(()) => {
                            *self.viz_stream_state.lock() = StreamState::Running;
                            let agent_count =
                                self.agent_registry.read().count_visualization_agents();
                            info!("🟢 [PNS-DYNAMIC] Visualization stream started: genome loaded, {} agents registered", agent_count);
                        }
                        Err(e) => {
                            error!("❌ [PNS-DYNAMIC] Failed to start viz stream: {}", e);
                            *self.viz_stream_state.lock() = StreamState::Stopped;
                        }
                    }
                }
            }
        }
    }

    /// Dynamically stop visualization stream
    fn try_stop_viz_stream(&self) {
        let mut state = self.viz_stream_state.lock();
        if *state == StreamState::Running {
            *state = StreamState::Stopping;
            drop(state);

            #[cfg(feature = "zmq-transport")]
            {
                if let Some(streams) = self.zmq_streams.lock().as_ref() {
                    match streams.stop_viz_stream() {
                        Ok(()) => {
                            *self.viz_stream_state.lock() = StreamState::Stopped;
                            warn!("🔴 [PNS-DYNAMIC] Visualization stream stopped: conditions no longer met");
                        }
                        Err(e) => {
                            error!("❌ [PNS-DYNAMIC] Failed to stop viz stream: {}", e);
                            *self.viz_stream_state.lock() = StreamState::Running;
                        }
                    }
                }
            }
        }
    }

    /// Evaluate all stream conditions and start/stop as needed
    fn evaluate_all_stream_states(&self) {
        // Sensory
        if self.should_sensory_stream_run() {
            self.try_start_sensory_stream();
        } else {
            self.try_stop_sensory_stream();
        }

        // Motor
        if self.should_motor_stream_run() {
            self.try_start_motor_stream();
        } else {
            self.try_stop_motor_stream();
        }

        // Visualization
        if self.should_viz_stream_run() {
            self.try_start_viz_stream();
        } else {
            self.try_stop_viz_stream();
        }
    }

    // === Lifecycle Hooks ===

    /// Called when genome is loaded - triggers stream evaluation
    pub fn on_genome_loaded(&self) {
        info!("🧬 [PNS-DYNAMIC] Genome loaded - evaluating stream conditions");
        self.evaluate_all_stream_states();
    }

    /// Called when genome is unloaded - stops all data streams immediately
    pub fn on_genome_unloaded(&self) {
        warn!("🧬 [PNS-DYNAMIC] Genome unloaded - stopping all data streams");
        self.try_stop_sensory_stream();
        self.try_stop_motor_stream();
        self.try_stop_viz_stream();
    }

    /// Called when an agent registers - triggers stream evaluation
    pub fn on_agent_registered_dynamic(&self, _agent_id: &str) {
        debug!("[PNS-DYNAMIC] Agent registered - evaluating stream conditions");
        self.evaluate_all_stream_states();
    }

    /// Called when an agent deregisters - triggers stream evaluation
    pub fn on_agent_deregistered_dynamic(&self, _agent_id: &str) {
        debug!("[PNS-DYNAMIC] Agent deregistered - evaluating stream conditions");
        self.evaluate_all_stream_states();
    }

    /// Start all PNS services
    /// Start only control streams (REST/registration) - safe before burst engine
    ///
    /// This starts the REST API for agent registration and heartbeats but does NOT
    /// start sensory/motor/viz streams. Use this during FEAGI startup before the
    /// burst engine is ready.
    pub fn start_control_streams(&self) -> Result<()> {
        if *self.running.read() {
            return Err(IOError::Agent("PNS already running".to_string()));
        }

        info!("🦀 [PNS] Starting control streams (REST/registration)...");

        // Initialize ZMQ streams but only start control streams
        #[cfg(feature = "zmq-transport")]
        {
            let zmq_streams = ZmqStreams::new(
                &self.config.zmq_rest_address,
                &self.config.zmq_api_control_address,
                &self.config.zmq_motor_address,
                &self.config.zmq_viz_address,
                &self.config.zmq_sensory_address,
                Arc::clone(&self.registration_handler),
                Arc::clone(&self.agent_registry),
                self.config.visualization_stream.clone(),
                self.config.sensory_stream.clone(),
            )?;

            zmq_streams.start_control_streams()?;
            *self.zmq_streams.lock() = Some(zmq_streams);
        }

        // Initialize WebSocket streams if enabled
        #[cfg(feature = "websocket-transport")]
        {
            if self.config.websocket.enabled {
                info!("🦀 [PNS] Initializing WebSocket streams...");
                let ws_streams = WebSocketStreams::new(self.config.clone())?;
                ws_streams.start_control_streams()?;
                *self.websocket_streams.lock() = Some(ws_streams);
                info!("🦀 [PNS] ✅ WebSocket control streams initialized");
            } else {
                info!("🦀 [PNS] WebSocket transport disabled in configuration");
            }
        }

        // Start heartbeat monitoring
        self.heartbeat_tracker
            .lock()
            .start(Arc::clone(&self.agent_registry));

        *self.running.write() = true;

        // Wire up dynamic gating callbacks
        info!("🦀 [PNS] Wiring dynamic stream gating callbacks...");
        let pns_self = self.clone_for_callbacks();
        self.registration_handler
            .lock()
            .set_on_agent_registered_dynamic(move |agent_id: String| {
                info!(
                    "🔔 [PNS-DYNAMIC-CALLBACK] Registration callback fired for agent: {}",
                    agent_id
                );
                pns_self.on_agent_registered_dynamic(&agent_id);
            });

        let pns_self = self.clone_for_callbacks();
        self.registration_handler
            .lock()
            .set_on_agent_deregistered_dynamic(move |agent_id: String| {
                info!(
                    "🔔 [PNS-DYNAMIC-CALLBACK] Deregistration callback fired for agent: {}",
                    agent_id
                );
                pns_self.on_agent_deregistered_dynamic(&agent_id);
            });

        info!("🦀 [PNS] ✅ Dynamic gating callbacks wired");
        info!("🦀 [PNS] ✅ Control streams started - ready for agent registration");
        info!("🦀 [PNS] ⏸️  Data streams (sensory/motor/viz) will start dynamically when conditions are met");

        Ok(())
    }

    /// Clone PNS for callbacks (only clone the Arc fields needed)
    fn clone_for_callbacks(&self) -> IOSystemForCallbacks {
        IOSystemForCallbacks {
            npu_ref: Arc::clone(&self.npu_ref),
            agent_registry: Arc::clone(&self.agent_registry),
            #[cfg(feature = "zmq-transport")]
            zmq_streams: Arc::clone(&self.zmq_streams),
            #[cfg(feature = "websocket-transport")]
            websocket_streams: Arc::clone(&self.websocket_streams),
            websocket_enabled: self.config.websocket.enabled,
            websocket_viz_port: self.config.websocket.visualization_port,
            sensory_stream_state: Arc::clone(&self.sensory_stream_state),
            motor_stream_state: Arc::clone(&self.motor_stream_state),
            viz_stream_state: Arc::clone(&self.viz_stream_state),
        }
    }

    /// Start data streams (sensory/motor/viz) - requires burst engine running
    ///
    /// This starts the data processing streams that require an active burst engine.
    /// Call this AFTER the burst engine has been started and is ready to process data.
    pub fn start_data_streams(&self) -> Result<()> {
        if !*self.running.read() {
            return Err(IOError::Agent(
                "PNS not running - call start_control_streams() first".to_string(),
            ));
        }

        info!("🦀 [PNS] Starting data streams (sensory/motor/viz)...");

        // Initialize async runtime if needed for UDP transports
        #[cfg(feature = "udp-transport")]
        {
            let needs_async = self.config.visualization_transport == TransportMode::Udp
                || self.config.sensory_transport == TransportMode::Udp;

            if needs_async {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(num_cpus::get())
                    .thread_name("feagi-io-async")
                    .enable_all()
                    .build()
                    .map_err(|e| {
                        IOError::Transport(format!("Failed to create async runtime: {}", e))
                    })?;
                *self.async_runtime.lock() = Some(Arc::new(runtime));
                info!("🦀 [PNS] Async runtime initialized");
            }
        }

        // Start ZMQ data streams
        #[cfg(feature = "zmq-transport")]
        {
            let streams_lock = self.zmq_streams.lock();
            if let Some(ref zmq_streams) = *streams_lock {
                match zmq_streams.start_data_streams() {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                }
            } else {
                return Err(IOError::Agent(
                    "ZMQ streams not initialized - call start_control_streams() first".to_string(),
                ));
            }
        }

        // Start WebSocket data streams if enabled
        #[cfg(feature = "websocket-transport")]
        {
            if self.config.websocket.enabled {
                let streams_lock = self.websocket_streams.lock();
                if let Some(ref ws_streams) = *streams_lock {
                    match ws_streams.start_data_streams() {
                        Ok(()) => {
                            info!("🦀 [PNS] ✅ WebSocket data streams started");
                            info!("🦀 [PNS] 🌐 Brain Visualizer can now connect to: ws://{}:{}",
                                  self.config.websocket.host, self.config.websocket.visualization_port);
                        }
                        Err(e) => {
                            warn!(
                                "⚠️ [PNS] WebSocket streams start failed: {} (continuing with ZMQ)",
                                e
                            );
                        }
                    }
                } else {
                    info!("🦀 [PNS] WebSocket streams not initialized (disabled in config)");
                }
            }
        }

        // Start UDP visualization transport if configured
        #[cfg(feature = "udp-transport")]
        {
            if self.config.visualization_transport == TransportMode::Udp {
                if let Some(runtime) = self.async_runtime.lock().as_ref() {
                    let runtime_clone: Arc<tokio::runtime::Runtime> = Arc::clone(runtime);
                    let mut udp_viz = UdpTransport::new(
                        self.config.udp_viz_config.clone(),
                        runtime_clone.clone(),
                    );

                    runtime_clone
                        .block_on(udp_viz.start())
                        .map_err(|e| IOError::Transport(format!("UDP viz start failed: {}", e)))?;

                    *self.udp_viz_transport.lock() = Some(udp_viz);
                    info!("🦀 [PNS] UDP visualization transport started");
                }
            }

            // Start UDP sensory transport if configured
            if self.config.sensory_transport == TransportMode::Udp {
                if let Some(runtime) = self.async_runtime.lock().as_ref() {
                    let runtime_clone: Arc<tokio::runtime::Runtime> = Arc::clone(runtime);
                    let mut udp_sensory = UdpTransport::new(
                        self.config.udp_sensory_config.clone(),
                        runtime_clone.clone(),
                    );

                    runtime_clone.block_on(udp_sensory.start()).map_err(|e| {
                        IOError::Transport(format!("UDP sensory start failed: {}", e))
                    })?;

                    *self.udp_sensory_transport.lock() = Some(udp_sensory);
                    info!("🦀 [PNS] UDP sensory transport started");
                }
            }
        }

        info!("🦀 [PNS] ✅ Data streams started - sensory data will now be processed");

        Ok(())
    }

    /// Start all streams at once (legacy method for backward compatibility)
    ///
    /// Equivalent to calling start_control_streams() followed by start_data_streams().
    /// Prefer using the split methods during FEAGI startup for proper sequencing.
    pub fn start(&self) -> Result<()> {
        self.start_control_streams()?;
        self.start_data_streams()?;
        Ok(())
    }

    /// Stop all PNS services
    pub fn stop(&self) -> Result<()> {
        if !*self.running.read() {
            return Ok(());
        }

        info!("🦀 [PNS] Stopping all services...");
        *self.running.write() = false;

        // Stop ZMQ streams
        #[cfg(feature = "zmq-transport")]
        {
            if let Some(streams) = self.zmq_streams.lock().take() {
                streams.stop()?;
            }
        }

        // Stop UDP transports
        #[cfg(feature = "udp-transport")]
        {
            if let Some(runtime) = self.async_runtime.lock().as_ref() {
                if let Some(mut udp_viz) = self.udp_viz_transport.lock().take() {
                    runtime
                        .block_on(udp_viz.stop())
                        .map_err(|e| IOError::Transport(format!("UDP viz stop failed: {}", e)))?;
                    info!("🦀 [PNS] UDP visualization transport stopped");
                }

                // Stop UDP sensory transport
                if let Some(mut udp_sensory) = self.udp_sensory_transport.lock().take() {
                    runtime.block_on(udp_sensory.stop()).map_err(|e| {
                        IOError::Transport(format!("UDP sensory stop failed: {}", e))
                    })?;
                    info!("🦀 [PNS] UDP sensory transport stopped");
                }
            }

            // Shutdown async runtime
            if let Some(runtime_arc) = self.async_runtime.lock().take() {
                // Try to unwrap Arc if we have the only reference, otherwise clone will keep it alive
                match Arc::try_unwrap(runtime_arc) {
                    Ok(runtime) => {
                        runtime.shutdown_timeout(std::time::Duration::from_secs(2));
                        info!("🦀 [PNS] Async runtime shutdown");
                    }
                    Err(_) => {
                        warn!(
                            "🦀 [PNS] ⚠️  Async runtime has outstanding references, skipping shutdown"
                        );
                    }
                }
            }
        }

        // Stop heartbeat monitoring
        self.heartbeat_tracker.lock().stop();

        info!("🦀 [PNS] ✅ All services stopped");
        Ok(())
    }

    /// Check if PNS is running
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    /// Get agent registry (for external access)
    pub fn get_agent_registry(&self) -> Arc<RwLock<AgentRegistry>> {
        Arc::clone(&self.agent_registry)
    }

    /// Get registration handler (for full transport negotiation)
    pub fn get_registration_handler(&self) -> Arc<parking_lot::Mutex<RegistrationHandler>> {
        Arc::clone(&self.registration_handler)
    }

    /// Check which transports have active visualization agents
    fn get_active_viz_transports(&self) -> Vec<String> {
        let registry = self.agent_registry.read();
        let mut transports = std::collections::HashSet::new();

        for agent in registry.get_all() {
            if matches!(
                agent.agent_type,
                AgentType::Visualization | AgentType::Infrastructure
            ) {
                if let Some(ref chosen) = agent.chosen_transport {
                    transports.insert(chosen.clone());
                } else {
                    // Legacy: if no chosen_transport, assume ZMQ
                    transports.insert("zmq".to_string());
                }
            }
        }

        transports.into_iter().collect()
    }

    /// Publish raw fire queue data (NEW ARCHITECTURE - serialization off burst thread)
    /// Called by burst engine with raw fire queue data
    /// PNS will serialize on its own thread to avoid blocking burst engine
    ///
    /// **PER-AGENT TRANSPORT:** Only publishes to transports that have active agents
    pub fn publish_raw_fire_queue(
        &self,
        fire_data: feagi_npu_burst_engine::RawFireQueueSnapshot,
    ) -> Result<()> {
        static FIRST_LOG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !FIRST_LOG.load(std::sync::atomic::Ordering::Relaxed) {
            info!(
                "[PNS] 🔍 ARCHITECTURE: publish_raw_fire_queue() called with {} areas (serialization will happen on PNS thread)",
                fire_data.len()
            );
            FIRST_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        let active_transports = self.get_active_viz_transports();

        static TRANSPORT_LOG_COUNTER: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);
        let log_count = TRANSPORT_LOG_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if log_count.is_multiple_of(100) {
            trace!("[PNS] Active viz transports: {:?}", active_transports);
        }

        let mut published_to = Vec::new();
        let mut errors = Vec::new();

        // Publish to ZMQ if any agent is using it
        #[cfg(feature = "zmq-transport")]
        {
            if active_transports.contains(&"zmq".to_string())
                || active_transports.contains(&"shm".to_string())
                || active_transports.is_empty()
            {
                if let Some(streams) = self.zmq_streams.lock().as_ref() {
                    match streams.publish_raw_fire_queue(fire_data.clone()) {
                        Ok(()) => published_to.push("ZMQ"),
                        Err(e) => errors.push(format!("ZMQ: {}", e)),
                    }
                }
            }
        }

        // Publish to WebSocket if any agent chose it OR if WebSocket servers are running
        // (allowing clients that don't send chosen_transport to still receive data)
        #[cfg(feature = "websocket-transport")]
        {
            let should_publish_ws = active_transports.contains(&"websocket".to_string())
                || (self.config.websocket.enabled
                    && self.agent_registry.read().has_visualization_agents());

            if should_publish_ws {
                if let Some(streams) = self.websocket_streams.lock().as_ref() {
                    // Publish regardless of the internal `running` flag.
                    //
                    // The WS visualization publisher may be started eagerly (for BV handshake)
                    // before `start_data_streams()` is called. Gating on `is_running()` can
                    // incorrectly suppress all WS visualization output.
                    match streams.publish_raw_fire_queue(fire_data.clone()) {
                        Ok(()) => published_to.push("WebSocket"),
                        Err(e) => errors.push(format!("WebSocket: {}", e)),
                    }
                } else {
                    if log_count.is_multiple_of(100) {
                        warn!("[PNS] ⚠️ WebSocket enabled but streams not initialized!");
                    }
                }
            }
        }

        // Log results
        if !published_to.is_empty() {
            static LOG_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let count = LOG_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if count.is_multiple_of(100) {
                // Log every 100th frame to avoid spam
                trace!("[PNS] Published visualization to: {:?}", published_to);
            }
            Ok(())
        } else if !errors.is_empty() {
            warn!("[PNS] ⚠️ Visualization publish failed: {:?}", errors);
            Err(IOError::Transport(format!(
                "All transports failed: {:?}",
                errors
            )))
        } else {
            // No transports running - this is OK during startup
            Ok(())
        }
    }

    /// Publish motor data to a specific agent
    /// Called by burst engine to send motor commands
    ///
    /// **MULTI-TRANSPORT:** Publishes to ALL enabled transports (ZMQ, WebSocket)
    pub fn publish_motor(&self, agent_id: &str, data: &[u8]) -> Result<()> {
        static FIRST_LOG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !FIRST_LOG.load(std::sync::atomic::Ordering::Relaxed) {
            info!(
                "[PNS] 🎮 publish_motor() called for agent '{}': {} bytes",
                agent_id,
                data.len()
            );
            FIRST_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        let mut published_to = Vec::new();
        let mut errors = Vec::new();

        // Publish to ZMQ if enabled
        #[cfg(feature = "zmq-transport")]
        {
            if let Some(streams) = self.zmq_streams.lock().as_ref() {
                match streams.publish_motor(agent_id, data) {
                    Ok(()) => published_to.push("ZMQ"),
                    Err(e) => errors.push(format!("ZMQ: {}", e)),
                }
            }
        }

        // Publish to WebSocket if enabled
        #[cfg(feature = "websocket-transport")]
        {
            if let Some(streams) = self.websocket_streams.lock().as_ref() {
                // Same rationale as visualization: WS publishers can be started before the
                // data-stream lifecycle flips.
                match streams.publish_motor(agent_id, data) {
                    Ok(()) => published_to.push("WebSocket"),
                    Err(e) => errors.push(format!("WebSocket: {}", e)),
                }
            }
        }

        // Return success if published to at least one transport
        if !published_to.is_empty() {
            Ok(())
        } else if !errors.is_empty() {
            warn!(
                "[PNS] ⚠️ Motor publish failed for '{}': {:?}",
                agent_id, errors
            );
            Err(IOError::Transport(format!(
                "All transports failed: {:?}",
                errors
            )))
        } else {
            // No transports running
            Err(IOError::NotRunning(
                "No motor transports available".to_string(),
            ))
        }
    }
}

/// Implement VisualizationPublisher trait for burst engine integration (NO PYTHON IN HOT PATH!)
impl feagi_npu_burst_engine::VisualizationPublisher for IOSystem {
    fn publish_raw_fire_queue(
        &self,
        fire_data: feagi_npu_burst_engine::RawFireQueueSnapshot,
    ) -> std::result::Result<(), String> {
        self.publish_raw_fire_queue(fire_data)
            .map_err(|e| e.to_string())
    }
}

impl feagi_npu_burst_engine::MotorPublisher for IOSystem {
    fn publish_motor(&self, agent_id: &str, data: &[u8]) -> std::result::Result<(), String> {
        self.publish_motor(agent_id, data)
            .map_err(|e| e.to_string())
    }
}

impl Drop for IOSystem {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

impl Default for IOSystem {
    fn default() -> Self {
        Self::new().expect("Failed to create default PNS")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pns_creation() {
        let pns = IOSystem::new();
        assert!(pns.is_ok());
    }

    #[test]
    fn test_pns_lifecycle() {
        let pns = IOSystem::new().unwrap();
        assert!(!pns.is_running());

        // Note: Can't actually start without conflicting with running FEAGI
        // Real tests require integration testing with Docker
    }

    #[test]
    fn test_udp_viz_config() {
        // Create PNS with UDP visualization transport
        let udp_viz = UdpConfig {
            bind_address: "127.0.0.1:0".to_string(), // Use port 0 for auto-assign
            peer_address: "127.0.0.1:9999".to_string(),
            ..Default::default()
        };
        let config = IOConfig {
            visualization_transport: TransportMode::Udp,
            udp_viz_config: udp_viz,
            ..Default::default()
        };

        let pns = IOSystem::with_config(config).unwrap();
        assert!(!pns.is_running());

        // Note: Can't test actual start without port conflicts
        // Verifies configuration is accepted and PNS can be created with UDP mode
    }

    #[test]
    fn test_dual_transport_config() {
        // Test that we can configure both UDP and ZMQ
        let udp_viz = UdpConfig {
            bind_address: "127.0.0.1:0".to_string(),
            peer_address: "127.0.0.1:9998".to_string(),
            ..Default::default()
        };
        let udp_sensory = UdpConfig {
            bind_address: "127.0.0.1:0".to_string(),
            peer_address: "127.0.0.1:9997".to_string(),
            ..Default::default()
        };
        let config = IOConfig {
            visualization_transport: TransportMode::Udp,
            sensory_transport: TransportMode::Udp,
            udp_viz_config: udp_viz,
            udp_sensory_config: udp_sensory,
            ..Default::default()
        };

        let pns = IOSystem::with_config(config).unwrap();
        assert!(!pns.is_running());

        // Verifies dual UDP transport configuration is valid
    }
}
