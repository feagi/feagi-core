//! PNS configuration

use crate::transports::zmq::VisualizationSendConfig;
use crate::transports::udp::UdpConfig;

/// Transport mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    /// ZMQ (TCP-based, reliable, blocking)
    Zmq,
    /// UDP (best-effort, high-throughput, nonblocking)
    Udp,
}

/// Configuration for PNS
#[derive(Debug, Clone)]
pub struct PNSConfig {
    // === ZMQ Configuration ===
    pub zmq_rest_address: String,
    pub zmq_motor_address: String,
    pub zmq_viz_address: String,
    pub zmq_sensory_address: String,
    pub visualization_stream: VisualizationSendConfig,

    // === UDP Configuration ===
    pub udp_viz_config: UdpConfig,
    pub udp_sensory_config: UdpConfig,

    // === Transport Mode Selection ===
    /// Transport for visualization data (Zmq or Udp)
    pub visualization_transport: TransportMode,
    /// Transport for sensory data (Zmq or Udp)
    pub sensory_transport: TransportMode,

    // === Shared Configuration ===
    pub shm_base_path: String,
}

impl Default for PNSConfig {
    fn default() -> Self {
        Self {
            // ZMQ defaults
            zmq_rest_address: "tcp://0.0.0.0:5563".to_string(), // REST/registration port
            zmq_motor_address: "tcp://0.0.0.0:30005".to_string(), // Motor output port
            zmq_viz_address: "tcp://0.0.0.0:5562".to_string(),  // Visualization output port
            zmq_sensory_address: "tcp://0.0.0.0:5558".to_string(), // Sensory input port
            visualization_stream: VisualizationSendConfig::default(),

            // UDP defaults
            udp_viz_config: UdpConfig {
                bind_address: "0.0.0.0:5565".to_string(),
                peer_address: "127.0.0.1:5565".to_string(),
                compress: true,
                max_message_size: 262144, // 256KB
            },
            udp_sensory_config: UdpConfig {
                bind_address: "0.0.0.0:5566".to_string(),
                peer_address: "127.0.0.1:5566".to_string(),
                compress: true,
                max_message_size: 65536, // 64KB
            },

            // Default to ZMQ for compatibility
            visualization_transport: TransportMode::Zmq,
            sensory_transport: TransportMode::Zmq,

            // Shared
            shm_base_path: "/tmp".to_string(),
        }
    }
}

