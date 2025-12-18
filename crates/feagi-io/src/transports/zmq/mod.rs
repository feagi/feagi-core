// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// ZMQ Streams - manages all ZMQ communication

mod api_control;
mod motor;
mod rest;
mod sensory;
mod visualization;

pub use api_control::ApiControlStream;
pub use motor::MotorStream;
pub use rest::RestStream;
pub use sensory::{SensoryReceiveConfig, SensoryStream};
pub use visualization::{
    VisualizationOverflowStrategy, VisualizationSendConfig, VisualizationStream,
};

use crate::core::{IOError, RegistrationHandler, AgentRegistry};
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;
use tracing::{debug, info};

/// ZMQ Streams coordinator
pub struct ZmqStreams {
    rest_stream: RestStream,
    api_control_stream: ApiControlStream,
    motor_stream: MotorStream,
    viz_stream: VisualizationStream,
    sensory_stream: SensoryStream,
}

impl ZmqStreams {
    pub fn new(
        rest_address: &str,
        api_control_address: &str,
        motor_address: &str,
        viz_address: &str,
        sensory_address: &str,
        registration_handler: Arc<Mutex<RegistrationHandler>>,
        agent_registry: Arc<RwLock<AgentRegistry>>,
        viz_config: VisualizationSendConfig,
        sensory_config: SensoryReceiveConfig,
    ) -> Result<Self, IOError> {
        let context = Arc::new(zmq::Context::new());

        let mut rest_stream = RestStream::new(Arc::clone(&context), rest_address)
            .map_err(|e| IOError::Zmq(format!("REST stream: {}", e)))?;

        // Set registration handler
        rest_stream.set_registration_handler(registration_handler);

        let api_control_stream = ApiControlStream::new(Arc::clone(&context), api_control_address)
            .map_err(|e| IOError::Zmq(format!("API control stream: {}", e)))?;

        let motor_stream = MotorStream::new(Arc::clone(&context), motor_address)
            .map_err(|e| IOError::Zmq(format!("Motor stream: {}", e)))?;

        let viz_stream = VisualizationStream::new(Arc::clone(&context), viz_address, viz_config)
            .map_err(|e| IOError::Zmq(format!("Viz stream: {}", e)))?;

        let sensory_stream =
            SensoryStream::new(Arc::clone(&context), sensory_address, sensory_config)
                .map_err(|e| IOError::Zmq(format!("Sensory stream: {}", e)))?;
        
        // Wire up agent registry for security gating
        sensory_stream.set_agent_registry(Arc::clone(&agent_registry));

        Ok(Self {
            rest_stream,
            api_control_stream,
            motor_stream,
            viz_stream,
            sensory_stream,
        })
    }

    /// Start only control streams (REST/registration + API control) - safe before burst engine
    pub fn start_control_streams(&self) -> Result<(), IOError> {
        self.rest_stream
            .start()
            .map_err(|e| IOError::Zmq(format!("REST start: {}", e)))?;
        self.api_control_stream
            .start()
            .map_err(|e| IOError::Zmq(format!("API control start: {}", e)))?;
        info!("🦀 [ZMQ-STREAMS] ✅ Control streams started (REST + API Control)");
        Ok(())
    }

    /// Start data streams (sensory/motor/viz) - requires burst engine running
    pub fn start_data_streams(&self) -> Result<(), IOError> {
        self.motor_stream
            .start()
            .map_err(|e| IOError::Zmq(format!("Motor start: {}", e)))?;
        self.viz_stream
            .start()
            .map_err(|e| IOError::Zmq(format!("Viz start: {}", e)))?;
        self.sensory_stream
            .start()
            .map_err(|e| IOError::Zmq(format!("Sensory start: {}", e)))?;
        info!("🦀 [ZMQ-STREAMS] ✅ Data streams started (sensory/motor/viz)");
        Ok(())
    }
    
    // === Individual Stream Control (for dynamic gating) ===
    
    /// Start sensory stream only
    pub fn start_sensory_stream(&self) -> Result<(), IOError> {
        self.sensory_stream
            .start()
            .map_err(|e| IOError::Zmq(format!("Sensory start: {}", e)))
    }
    
    /// Stop sensory stream only
    pub fn stop_sensory_stream(&self) -> Result<(), IOError> {
        self.sensory_stream
            .stop()
            .map_err(|e| IOError::Zmq(format!("Sensory stop: {}", e)))
    }
    
    /// Start motor stream only
    pub fn start_motor_stream(&self) -> Result<(), IOError> {
        self.motor_stream
            .start()
            .map_err(|e| IOError::Zmq(format!("Motor start: {}", e)))
    }
    
    /// Stop motor stream only
    pub fn stop_motor_stream(&self) -> Result<(), IOError> {
        self.motor_stream
            .stop()
            .map_err(|e| IOError::Zmq(format!("Motor stop: {}", e)))
    }
    
    /// Start visualization stream only
    pub fn start_viz_stream(&self) -> Result<(), IOError> {
        self.viz_stream
            .start()
            .map_err(|e| IOError::Zmq(format!("Viz start: {}", e)))
    }
    
    /// Stop visualization stream only
    pub fn stop_viz_stream(&self) -> Result<(), IOError> {
        self.viz_stream
            .stop()
            .map_err(|e| IOError::Zmq(format!("Viz stop: {}", e)))
    }

    /// Start all streams at once (legacy method for backward compatibility)
    pub fn start(&self) -> Result<(), IOError> {
        self.start_control_streams()?;
        self.start_data_streams()?;
        Ok(())
    }

    pub fn stop(&self) -> Result<(), IOError> {
        self.rest_stream
            .stop()
            .map_err(|e| IOError::Zmq(format!("REST stop: {}", e)))?;
        self.api_control_stream
            .stop()
            .map_err(|e| IOError::Zmq(format!("API control stop: {}", e)))?;
        self.motor_stream
            .stop()
            .map_err(|e| IOError::Zmq(format!("Motor stop: {}", e)))?;
        self.viz_stream
            .stop()
            .map_err(|e| IOError::Zmq(format!("Viz stop: {}", e)))?;
        self.sensory_stream
            .stop()
            .map_err(|e| IOError::Zmq(format!("Sensory stop: {}", e)))?;
        Ok(())
    }

    /// Get reference to sensory stream (for NPU connection)
    pub fn get_sensory_stream(&self) -> &SensoryStream {
        &self.sensory_stream
    }

    /// Get mutable reference to API control stream (for NPU connection)
    pub fn get_api_control_stream_mut(&mut self) -> &mut ApiControlStream {
        &mut self.api_control_stream
    }

    /// Publish raw fire queue data (NEW ARCHITECTURE - serialization in PNS thread)
    pub fn publish_raw_fire_queue(&self, fire_data: feagi_npu_burst_engine::RawFireQueueSnapshot) -> Result<(), IOError> {
        static FIRST_LOG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !FIRST_LOG.load(std::sync::atomic::Ordering::Relaxed) {
            info!(
                "[ZMQ-STREAMS] 🏗️ ARCHITECTURE: Forwarding raw fire queue ({} areas) to viz_stream (serialization will happen on worker thread)",
                fire_data.len()
            );
            FIRST_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        self.viz_stream
            .publish_raw_fire_queue(fire_data)
            .map_err(|e| IOError::Zmq(format!("Viz publish: {}", e)))
    }
    
    /// Publish motor data to a specific agent via ZMQ
    pub fn publish_motor(&self, agent_id: &str, data: &[u8]) -> Result<(), IOError> {
        // Log every motor publish for debugging
        debug!(
            "[ZMQ-STREAMS] 🎮 Publishing motor data to '{}': {} bytes via ZMQ",
            agent_id, data.len()
        );

        // Publish with agent_id as topic prefix for agent-specific delivery
        // ZMQ PUB/SUB uses multipart messages: [topic, data]
        self.motor_stream
            .publish_with_topic(agent_id.as_bytes(), data)
            .map_err(|e| IOError::Zmq(format!("Motor publish: {}", e)))
    }
}
