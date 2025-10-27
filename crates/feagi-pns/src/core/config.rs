//! PNS configuration

use crate::transports::zmq::VisualizationSendConfig;

/// Configuration for PNS
#[derive(Debug, Clone)]
pub struct PNSConfig {
    pub zmq_rest_address: String,
    pub zmq_motor_address: String,
    pub zmq_viz_address: String,
    pub zmq_sensory_address: String,
    pub shm_base_path: String,
    pub visualization_stream: VisualizationSendConfig,
}

impl Default for PNSConfig {
    fn default() -> Self {
        Self {
            zmq_rest_address: "tcp://0.0.0.0:5563".to_string(), // REST/registration port
            zmq_motor_address: "tcp://0.0.0.0:30005".to_string(), // Motor output port
            zmq_viz_address: "tcp://0.0.0.0:5562".to_string(),  // Visualization output port
            zmq_sensory_address: "tcp://0.0.0.0:5558".to_string(), // Sensory input port (PULL socket)
            shm_base_path: "/tmp".to_string(),
            visualization_stream: VisualizationSendConfig::default(),
        }
    }
}

