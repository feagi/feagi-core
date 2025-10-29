//! FEAGI Peripheral Nervous System (PNS)
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
//! use feagi_pns::{PNS, PNSConfig};
//!
//! let pns = PNS::new().unwrap();
//! pns.start().unwrap();
//!
//! // Publish visualization data
//! let data = vec![1, 2, 3];
//! pns.publish_visualization(&data).unwrap();
//!
//! pns.stop().unwrap();
//! ```

use feagi_data_structures::FeagiSignal;
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;

#[cfg(feature = "udp-transport")]
use tokio::runtime::Runtime;

// Import NonBlockingTransport trait for UDP transport methods
#[cfg(feature = "udp-transport")]
use crate::nonblocking::transport::NonBlockingTransport;

// Core modules (shared across all transports)
pub mod blocking;
pub mod core;

#[cfg(feature = "udp-transport")]
pub mod nonblocking;

pub mod transports;

// Re-export commonly used types from core
pub use core::{
    AgentCapabilities, AgentDisconnectedEvent, AgentInfo, AgentRegisteredEvent, AgentRegistry,
    AgentTransport, AgentType, HeartbeatTracker, MotorCapability, MotorCommandEvent, PNSConfig,
    PNSError, RegistrationHandler, RegistrationRequest, Result, SensoryDataEvent, SharedFBC,
    StreamType, TransportMode, VisionCapability, VisualizationCapability, VisualizationReadyEvent,
};

// Re-export transport-specific types
#[cfg(feature = "zmq-transport")]
pub use transports::zmq::{
    MotorStream, RestStream, SensoryStream, VisualizationOverflowStrategy, VisualizationSendConfig,
    VisualizationStream, ZmqStreams,
};

#[cfg(feature = "udp-transport")]
pub use transports::udp::{UdpConfig, UdpTransport};

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
/// - `sensory_data_received`: PNS emits when sensory data arrives from agent
/// - `agent_registered`: PNS emits when new agent registers
/// - `agent_disconnected`: PNS emits when agent disconnects/times out
///
/// # Example
/// ```no_run
/// use feagi_pns::PNS;
///
/// let pns = PNS::new().unwrap();
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
pub struct PNS {
    config: PNSConfig,
    agent_registry: Arc<RwLock<AgentRegistry>>,
    registration_handler: Arc<Mutex<RegistrationHandler>>,
    heartbeat_tracker: Arc<Mutex<HeartbeatTracker>>,

    // === Transport Layer ===
    /// ZMQ streams (blocking, TCP-based)
    #[cfg(feature = "zmq-transport")]
    zmq_streams: Arc<Mutex<Option<ZmqStreams>>>,

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
        Arc<Mutex<Option<Arc<std::sync::Mutex<feagi_burst_engine::AgentManager>>>>>,

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

impl PNS {
    /// Create a new PNS with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(PNSConfig::default())
    }

    /// Create a new PNS with custom configuration
    pub fn with_config(config: PNSConfig) -> Result<Self> {
        let agent_registry = Arc::new(RwLock::new(AgentRegistry::with_defaults()));
        let heartbeat_tracker = Arc::new(Mutex::new(HeartbeatTracker::new()));
        let registration_handler = Arc::new(Mutex::new(RegistrationHandler::new(Arc::clone(
            &agent_registry,
        ))));

        Ok(Self {
            config,
            agent_registry,
            registration_handler,
            heartbeat_tracker,
            // Transport layer
            #[cfg(feature = "zmq-transport")]
            zmq_streams: Arc::new(Mutex::new(None)),
            #[cfg(feature = "udp-transport")]
            udp_viz_transport: Arc::new(Mutex::new(None)),
            #[cfg(feature = "udp-transport")]
            udp_sensory_transport: Arc::new(Mutex::new(None)),
            #[cfg(feature = "udp-transport")]
            async_runtime: Arc::new(Mutex::new(None)),
            running: Arc::new(RwLock::new(false)),
            sensory_agent_manager: Arc::new(Mutex::new(None)),
            // Initialize signals
            visualization_ready: Arc::new(Mutex::new(FeagiSignal::new())),
            motor_commands: Arc::new(Mutex::new(FeagiSignal::new())),
            sensory_data_received: Arc::new(Mutex::new(FeagiSignal::new())),
            agent_registered: Arc::new(Mutex::new(FeagiSignal::new())),
            agent_disconnected: Arc::new(Mutex::new(FeagiSignal::new())),
        })
    }

    /// Set the sensory agent manager (for SHM I/O coordination)
    /// Should be called before starting the PNS
    pub fn set_sensory_agent_manager(
        &self,
        manager: Arc<std::sync::Mutex<feagi_burst_engine::AgentManager>>,
    ) {
        *self.sensory_agent_manager.lock() = Some(manager.clone());
        // Also propagate to registration handler
        self.registration_handler
            .lock()
            .set_sensory_agent_manager(manager);
        println!("🦀 [PNS] Sensory agent manager connected for SHM I/O");
    }

    /// Connect the Rust NPU to the sensory stream for direct injection
    /// Should be called after starting the PNS
    #[cfg(feature = "zmq-transport")]
    pub fn connect_npu_to_sensory_stream(
        &self,
        npu: Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>,
    ) {
        if let Some(streams) = self.zmq_streams.lock().as_ref() {
            streams.get_sensory_stream().set_npu(npu);
            println!("🦀 [PNS] NPU connected to sensory stream for direct injection");
        } else {
            eprintln!("🦀 [PNS] [ERR] Cannot connect NPU: ZMQ streams not started");
        }
    }

    /// Connect the Rust NPU to the API control stream for direct queries (zero GIL contention)
    /// Should be called after starting the PNS
    #[cfg(feature = "zmq-transport")]
    pub fn connect_npu_to_api_control_stream(
        &self,
        npu: Arc<std::sync::Mutex<feagi_burst_engine::RustNPU>>,
    ) {
        if let Some(streams) = self.zmq_streams.lock().as_mut() {
            streams.get_api_control_stream_mut().set_npu(npu);
            println!("🦀 [PNS] NPU connected to API control stream for direct queries");
        } else {
            eprintln!("🦀 [PNS] [ERR] Cannot connect NPU: ZMQ streams not started");
        }
    }

    /// Set RPC callback for generic CoreAPIService method calls
    pub fn set_api_rpc_callback<F>(&self, callback: F)
    where
        F: Fn(&str, serde_json::Value) -> std::result::Result<serde_json::Value, String> + Send + Sync + 'static,
    {
        if let Some(streams) = self.zmq_streams.lock().as_mut() {
            // Wrap callback to ensure it matches the expected signature
            streams.get_api_control_stream_mut().set_rpc_callback(move |method, payload| {
                callback(method, payload)
            });
            println!("🦀 [PNS] RPC callback registered for CoreAPIService");
        } else {
            eprintln!("🦀 [PNS] [ERR] Cannot set RPC callback: ZMQ streams not started");
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

    /// Start all PNS services
    /// Start only control streams (REST/registration) - safe before burst engine
    /// 
    /// This starts the REST API for agent registration and heartbeats but does NOT
    /// start sensory/motor/viz streams. Use this during FEAGI startup before the
    /// burst engine is ready.
    pub fn start_control_streams(&self) -> Result<()> {
        if *self.running.read() {
            return Err(PNSError::Agent("PNS already running".to_string()));
        }

        println!("🦀 [PNS] Starting control streams (REST/registration)...");

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
                self.config.visualization_stream.clone(),
                self.config.sensory_stream.clone(),
            )?;

            zmq_streams.start_control_streams()?;
            *self.zmq_streams.lock() = Some(zmq_streams);
        }

        // Start heartbeat monitoring
        self.heartbeat_tracker
            .lock()
            .start(Arc::clone(&self.agent_registry));

        *self.running.write() = true;
        println!("🦀 [PNS] ✅ Control streams started - ready for agent registration");
        println!("🦀 [PNS] ⏸️  Data streams (sensory/motor/viz) NOT started - waiting for burst engine");

        Ok(())
    }

    /// Start data streams (sensory/motor/viz) - requires burst engine running
    /// 
    /// This starts the data processing streams that require an active burst engine.
    /// Call this AFTER the burst engine has been started and is ready to process data.
    pub fn start_data_streams(&self) -> Result<()> {
        if !*self.running.read() {
            return Err(PNSError::Agent(
                "PNS not running - call start_control_streams() first".to_string(),
            ));
        }

        println!("🦀 [PNS] Starting data streams (sensory/motor/viz)...");

        // Initialize async runtime if needed for UDP transports
        #[cfg(feature = "udp-transport")]
        {
            let needs_async = self.config.visualization_transport == TransportMode::Udp
                || self.config.sensory_transport == TransportMode::Udp;

            if needs_async {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(num_cpus::get())
                    .thread_name("feagi-pns-async")
                    .enable_all()
                    .build()
                    .map_err(|e| {
                        PNSError::Transport(format!("Failed to create async runtime: {}", e))
                    })?;
                *self.async_runtime.lock() = Some(Arc::new(runtime));
                println!("🦀 [PNS] Async runtime initialized");
            }
        }

        // Start ZMQ data streams
        #[cfg(feature = "zmq-transport")]
        {
            if let Some(ref zmq_streams) = *self.zmq_streams.lock() {
                zmq_streams.start_data_streams()?;
            } else {
                return Err(PNSError::Agent(
                    "ZMQ streams not initialized - call start_control_streams() first".to_string(),
                ));
            }
        }

        // Start UDP visualization transport if configured
        #[cfg(feature = "udp-transport")]
        {
            if self.config.visualization_transport == TransportMode::Udp {
                if let Some(runtime) = self.async_runtime.lock().as_ref() {
                    let runtime_clone = Arc::clone(runtime);
                    let mut udp_viz = UdpTransport::new(
                        self.config.udp_viz_config.clone(),
                        runtime_clone.clone(),
                    );

                    runtime_clone
                        .block_on(udp_viz.start())
                        .map_err(|e| PNSError::Transport(format!("UDP viz start failed: {}", e)))?;

                    *self.udp_viz_transport.lock() = Some(udp_viz);
                    println!("🦀 [PNS] UDP visualization transport started");
                }
            }

            // Start UDP sensory transport if configured
            if self.config.sensory_transport == TransportMode::Udp {
                if let Some(runtime) = self.async_runtime.lock().as_ref() {
                    let runtime_clone = Arc::clone(runtime);
                    let mut udp_sensory = UdpTransport::new(
                        self.config.udp_sensory_config.clone(),
                        runtime_clone.clone(),
                    );

                    runtime_clone.block_on(udp_sensory.start()).map_err(|e| {
                        PNSError::Transport(format!("UDP sensory start failed: {}", e))
                    })?;

                    *self.udp_sensory_transport.lock() = Some(udp_sensory);
                    println!("🦀 [PNS] UDP sensory transport started");
                }
            }
        }

        println!("🦀 [PNS] ✅ Data streams started - sensory data will now be processed");

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

        println!("🦀 [PNS] Stopping all services...");
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
                        .map_err(|e| PNSError::Transport(format!("UDP viz stop failed: {}", e)))?;
                    println!("🦀 [PNS] UDP visualization transport stopped");
                }

                // Stop UDP sensory transport
                if let Some(mut udp_sensory) = self.udp_sensory_transport.lock().take() {
                    runtime.block_on(udp_sensory.stop()).map_err(|e| {
                        PNSError::Transport(format!("UDP sensory stop failed: {}", e))
                    })?;
                    println!("🦀 [PNS] UDP sensory transport stopped");
                }
            }

            // Shutdown async runtime
            if let Some(runtime_arc) = self.async_runtime.lock().take() {
                // Try to unwrap Arc if we have the only reference, otherwise clone will keep it alive
                match Arc::try_unwrap(runtime_arc) {
                    Ok(runtime) => {
                        runtime.shutdown_timeout(std::time::Duration::from_secs(2));
                        println!("🦀 [PNS] Async runtime shutdown");
                    }
                    Err(_) => {
                        println!(
                            "🦀 [PNS] ⚠️  Async runtime has outstanding references, skipping shutdown"
                        );
                    }
                }
            }
        }

        // Stop heartbeat monitoring
        self.heartbeat_tracker.lock().stop();

        println!("🦀 [PNS] ✅ All services stopped");
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

    /// Publish visualization data to configured transport (UDP or ZMQ)
    /// Called by burst engine after writing FQ data to SHM
    pub fn publish_visualization(&self, data: &[u8]) -> Result<()> {
        static FIRST_LOG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !FIRST_LOG.load(std::sync::atomic::Ordering::Relaxed) {
            eprintln!(
                "[PNS] 🔍 TRACE: publish_visualization() called with {} bytes via {:?}",
                data.len(),
                self.config.visualization_transport
            );
            FIRST_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        match self.config.visualization_transport {
            #[cfg(feature = "zmq-transport")]
            TransportMode::Zmq => {
                if let Some(streams) = self.zmq_streams.lock().as_ref() {
                    streams.publish_visualization(data)?;
                    Ok(())
                } else {
                    eprintln!("[PNS] ❌ CRITICAL: ZMQ streams not started!");
                    Err(PNSError::NotRunning("ZMQ streams not started".to_string()))
                }
            }
            #[cfg(feature = "udp-transport")]
            TransportMode::Udp => {
                // UDP requires async context, bridge via runtime.block_on()
                if let Some(runtime) = self.async_runtime.lock().as_ref() {
                    if let Some(udp_viz) = self.udp_viz_transport.lock().as_ref() {
                        // Create FBC from data for zero-copy
                        use feagi_data_serialization::FeagiByteContainer;
                        let mut fbc = FeagiByteContainer::new_empty();
                        fbc.try_write_data_by_copy_and_verify(data).map_err(|e| {
                            PNSError::Transport(format!("FBC write failed: {:?}", e))
                        })?;
                        let shared_fbc = Arc::new(fbc);

                        // Bridge sync→async via runtime
                        runtime
                            .block_on(udp_viz.publish_visualization(shared_fbc))
                            .map_err(|e| {
                                PNSError::Transport(format!("UDP viz publish failed: {}", e))
                            })?;
                        Ok(())
                    } else {
                        eprintln!("[PNS] ❌ CRITICAL: UDP visualization transport not started!");
                        Err(PNSError::NotRunning(
                            "UDP viz transport not started".to_string(),
                        ))
                    }
                } else {
                    eprintln!("[PNS] ❌ CRITICAL: Async runtime not available for UDP!");
                    Err(PNSError::NotRunning(
                        "Async runtime not available".to_string(),
                    ))
                }
            }
            #[cfg(not(any(feature = "zmq-transport", feature = "udp-transport")))]
            _ => {
                Err(PNSError::Transport(
                    "No visualization transport enabled (enable zmq-transport or udp-transport feature)".to_string(),
                ))
            }
        }
    }
}

/// Implement VisualizationPublisher trait for burst engine integration (NO PYTHON IN HOT PATH!)
impl feagi_burst_engine::VisualizationPublisher for PNS {
    fn publish_visualization(&self, data: &[u8]) -> std::result::Result<(), String> {
        self.publish_visualization(data).map_err(|e| e.to_string())
    }
}

impl Drop for PNS {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

impl Default for PNS {
    fn default() -> Self {
        Self::new().expect("Failed to create default PNS")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pns_creation() {
        let pns = PNS::new();
        assert!(pns.is_ok());
    }

    #[test]
    fn test_pns_lifecycle() {
        let pns = PNS::new().unwrap();
        assert!(!pns.is_running());

        // Note: Can't actually start without conflicting with running FEAGI
        // Real tests require integration testing with Docker
    }

    #[test]
    fn test_udp_viz_config() {
        // Create PNS with UDP visualization transport
        let mut config = PNSConfig::default();
        config.visualization_transport = TransportMode::Udp;
        config.udp_viz_config.bind_address = "127.0.0.1:0".to_string(); // Use port 0 for auto-assign
        config.udp_viz_config.peer_address = "127.0.0.1:9999".to_string();

        let pns = PNS::with_config(config).unwrap();
        assert!(!pns.is_running());

        // Note: Can't test actual start without port conflicts
        // Verifies configuration is accepted and PNS can be created with UDP mode
    }

    #[test]
    fn test_dual_transport_config() {
        // Test that we can configure both UDP and ZMQ
        let mut config = PNSConfig::default();
        config.visualization_transport = TransportMode::Udp;
        config.sensory_transport = TransportMode::Udp;
        config.udp_viz_config.bind_address = "127.0.0.1:0".to_string();
        config.udp_viz_config.peer_address = "127.0.0.1:9998".to_string();
        config.udp_sensory_config.bind_address = "127.0.0.1:0".to_string();
        config.udp_sensory_config.peer_address = "127.0.0.1:9997".to_string();

        let pns = PNS::with_config(config).unwrap();
        assert!(!pns.is_running());

        // Verifies dual UDP transport configuration is valid
    }
}
