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

use crate::core::{PNSError, RegistrationHandler};
use parking_lot::Mutex;
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
        viz_config: VisualizationSendConfig,
        sensory_config: SensoryReceiveConfig,
    ) -> Result<Self, PNSError> {
        let context = Arc::new(zmq::Context::new());

        let mut rest_stream = RestStream::new(Arc::clone(&context), rest_address)
            .map_err(|e| PNSError::Zmq(format!("REST stream: {}", e)))?;

        // Set registration handler
        rest_stream.set_registration_handler(registration_handler);

        let api_control_stream = ApiControlStream::new(Arc::clone(&context), api_control_address)
            .map_err(|e| PNSError::Zmq(format!("API control stream: {}", e)))?;

        let motor_stream = MotorStream::new(Arc::clone(&context), motor_address)
            .map_err(|e| PNSError::Zmq(format!("Motor stream: {}", e)))?;

        let viz_stream = VisualizationStream::new(Arc::clone(&context), viz_address, viz_config)
            .map_err(|e| PNSError::Zmq(format!("Viz stream: {}", e)))?;

        let sensory_stream =
            SensoryStream::new(Arc::clone(&context), sensory_address, sensory_config)
                .map_err(|e| PNSError::Zmq(format!("Sensory stream: {}", e)))?;

        Ok(Self {
            rest_stream,
            api_control_stream,
            motor_stream,
            viz_stream,
            sensory_stream,
        })
    }

    /// Start only control streams (REST/registration + API control) - safe before burst engine
    pub fn start_control_streams(&self) -> Result<(), PNSError> {
        self.rest_stream
            .start()
            .map_err(|e| PNSError::Zmq(format!("REST start: {}", e)))?;
        self.api_control_stream
            .start()
            .map_err(|e| PNSError::Zmq(format!("API control start: {}", e)))?;
        info!("🦀 [ZMQ-STREAMS] ✅ Control streams started (REST + API Control)");
        Ok(())
    }

    /// Start data streams (sensory/motor/viz) - requires burst engine running
    pub fn start_data_streams(&self) -> Result<(), PNSError> {
        self.motor_stream
            .start()
            .map_err(|e| PNSError::Zmq(format!("Motor start: {}", e)))?;
        self.viz_stream
            .start()
            .map_err(|e| PNSError::Zmq(format!("Viz start: {}", e)))?;
        self.sensory_stream
            .start()
            .map_err(|e| PNSError::Zmq(format!("Sensory start: {}", e)))?;
        info!("🦀 [ZMQ-STREAMS] ✅ Data streams started (sensory/motor/viz)");
        Ok(())
    }

    /// Start all streams at once (legacy method for backward compatibility)
    pub fn start(&self) -> Result<(), PNSError> {
        self.start_control_streams()?;
        self.start_data_streams()?;
        Ok(())
    }

    pub fn stop(&self) -> Result<(), PNSError> {
        self.rest_stream
            .stop()
            .map_err(|e| PNSError::Zmq(format!("REST stop: {}", e)))?;
        self.api_control_stream
            .stop()
            .map_err(|e| PNSError::Zmq(format!("API control stop: {}", e)))?;
        self.motor_stream
            .stop()
            .map_err(|e| PNSError::Zmq(format!("Motor stop: {}", e)))?;
        self.viz_stream
            .stop()
            .map_err(|e| PNSError::Zmq(format!("Viz stop: {}", e)))?;
        self.sensory_stream
            .stop()
            .map_err(|e| PNSError::Zmq(format!("Sensory stop: {}", e)))?;
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

    /// Publish visualization data to ZMQ subscribers
    pub fn publish_visualization(&self, data: &[u8]) -> Result<(), PNSError> {
        static FIRST_LOG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !FIRST_LOG.load(std::sync::atomic::Ordering::Relaxed) {
            debug!(
                "[ZMQ-STREAMS] 🔍 TRACE: Forwarding {} bytes to viz_stream.publish()",
                data.len()
            );
            FIRST_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        self.viz_stream
            .publish(data)
            .map_err(|e| PNSError::Zmq(format!("Viz publish: {}", e)))
    }
    
    /// Publish motor data to a specific agent via ZMQ
    pub fn publish_motor(&self, agent_id: &str, data: &[u8]) -> Result<(), PNSError> {
        // Log every motor publish for debugging
        debug!(
            "[ZMQ-STREAMS] 🎮 Publishing motor data to '{}': {} bytes via ZMQ",
            agent_id, data.len()
        );

        // TODO: Add topic prefix for agent-specific delivery
        // For now, publish to all (agents filter by agent_id in the data)
        self.motor_stream
            .publish(data)
            .map_err(|e| PNSError::Zmq(format!("Motor publish: {}", e)))
    }
}
