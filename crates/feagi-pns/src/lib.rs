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

use parking_lot::{Mutex, RwLock};
use std::sync::Arc;

// Core modules (shared across all transports)
pub mod core;
pub mod blocking;
pub mod nonblocking;
pub mod transports;

// Re-export commonly used types from core
pub use core::{
    AgentCapabilities, AgentInfo, AgentRegistry, AgentType, HeartbeatTracker, PNSConfig,
    PNSError, RegistrationHandler, Result, SharedFBC, StreamType,
};

// Re-export transport-specific types
pub use transports::zmq::{
    MotorStream, RestStream, SensoryStream, VisualizationOverflowStrategy, VisualizationSendConfig,
    VisualizationStream, ZmqStreams,
};

// Keep shm module at root for now (will be moved to transports/ in future)
pub mod shm;

/// Main PNS - manages all agent I/O
pub struct PNS {
    config: PNSConfig,
    agent_registry: Arc<RwLock<AgentRegistry>>,
    registration_handler: Arc<Mutex<RegistrationHandler>>,
    heartbeat_tracker: Arc<Mutex<HeartbeatTracker>>,
    zmq_streams: Arc<Mutex<Option<ZmqStreams>>>,
    running: Arc<RwLock<bool>>,
    /// Optional reference to burst engine's sensory agent manager for SHM I/O
    sensory_agent_manager:
        Arc<Mutex<Option<Arc<std::sync::Mutex<feagi_burst_engine::AgentManager>>>>>,
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
            zmq_streams: Arc::new(Mutex::new(None)),
            running: Arc::new(RwLock::new(false)),
            sensory_agent_manager: Arc::new(Mutex::new(None)),
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
    pub fn start(&self) -> Result<()> {
        if *self.running.read() {
            return Err(PNSError::Agent("PNS already running".to_string()));
        }

        println!("🦀 [PNS] Starting FEAGI Peripheral Nervous System...");

        // Start ZMQ streams
        let zmq_streams = ZmqStreams::new(
            &self.config.zmq_rest_address,
            &self.config.zmq_motor_address,
            &self.config.zmq_viz_address,
            &self.config.zmq_sensory_address,
            Arc::clone(&self.registration_handler),
            self.config.visualization_stream.clone(),
        )?;

        zmq_streams.start()?;
        *self.zmq_streams.lock() = Some(zmq_streams);

        // Start heartbeat monitoring
        self.heartbeat_tracker
            .lock()
            .start(Arc::clone(&self.agent_registry));

        *self.running.write() = true;
        println!("🦀 [PNS] ✅ All services started successfully");

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
        if let Some(streams) = self.zmq_streams.lock().take() {
            streams.stop()?;
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

    /// Publish visualization data to all ZMQ subscribers
    /// Called by burst engine after writing FQ data to SHM
    pub fn publish_visualization(&self, data: &[u8]) -> Result<()> {
        static FIRST_LOG: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        if !FIRST_LOG.load(std::sync::atomic::Ordering::Relaxed) {
            eprintln!(
                "[PNS] 🔍 TRACE: publish_visualization() called with {} bytes",
                data.len()
            );
            FIRST_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        if let Some(streams) = self.zmq_streams.lock().as_ref() {
            streams.publish_visualization(data)?;
            Ok(())
        } else {
            eprintln!("[PNS] ❌ CRITICAL: ZMQ streams not started!");
            Err(PNSError::NotRunning("ZMQ streams not started".to_string()))
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
}
