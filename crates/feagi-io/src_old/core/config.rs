// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! PNS configuration

#[cfg(feature = "zmq-transport")]
use crate::transports::zmq::{SensoryReceiveConfig, VisualizationSendConfig};

#[cfg(feature = "udp-transport")]
use crate::transports::udp::UdpConfig;

/// Transport mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    /// ZMQ (TCP-based, reliable, blocking)
    #[cfg(feature = "zmq-transport")]
    Zmq,
    /// UDP (best-effort, high-throughput, nonblocking)
    #[cfg(feature = "udp-transport")]
    Udp,
}

/// WebSocket transport configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub enabled: bool,
    pub host: String,
    pub sensory_port: u16,
    pub motor_port: u16,
    pub visualization_port: u16,
    pub registration_port: u16,
    pub rest_api_port: u16,
    pub connection_timeout_ms: u64,
    pub ping_interval_ms: u64,
    pub ping_timeout_ms: u64,
    pub close_timeout_ms: u64,
    pub max_message_size: usize,
    pub max_connections: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default, enabled via config
            host: "0.0.0.0".to_string(),
            sensory_port: 9051,
            motor_port: 9052,
            visualization_port: 9050,
            registration_port: 9053,
            rest_api_port: 9054,
            connection_timeout_ms: 5000,
            ping_interval_ms: 60000,
            ping_timeout_ms: 60000,
            close_timeout_ms: 10000,
            max_message_size: 10485760, // 10MB
            max_connections: 100,
        }
    }
}

/// Configuration for I/O System
#[derive(Debug, Clone)]
pub struct IOConfig {
    // === ZMQ Configuration ===
    #[cfg(feature = "zmq-transport")]
    pub zmq_rest_address: String,
    #[cfg(feature = "zmq-transport")]
    pub zmq_api_control_address: String,
    #[cfg(feature = "zmq-transport")]
    pub zmq_motor_address: String,
    #[cfg(feature = "zmq-transport")]
    pub zmq_viz_address: String,
    #[cfg(feature = "zmq-transport")]
    pub zmq_sensory_address: String,
    #[cfg(feature = "zmq-transport")]
    pub visualization_stream: VisualizationSendConfig,
    #[cfg(feature = "zmq-transport")]
    pub sensory_stream: SensoryReceiveConfig,

    // === UDP Configuration ===
    #[cfg(feature = "udp-transport")]
    pub udp_viz_config: UdpConfig,
    #[cfg(feature = "udp-transport")]
    pub udp_sensory_config: UdpConfig,

    // === WebSocket Configuration (NEW) ===
    pub websocket: WebSocketConfig,

    // === Transport Mode Selection ===
    /// Transport for visualization data (Zmq or Udp)
    pub visualization_transport: TransportMode,
    /// Transport for sensory data (Zmq or Udp)
    pub sensory_transport: TransportMode,

    // === Shared Configuration ===
    pub shm_base_path: String,
}

impl Default for IOConfig {
    fn default() -> Self {
        Self {
            // ZMQ defaults
            #[cfg(feature = "zmq-transport")]
            zmq_rest_address: "tcp://0.0.0.0:5563".to_string(), // REST/registration port
            #[cfg(feature = "zmq-transport")]
            zmq_api_control_address: "tcp://0.0.0.0:5565".to_string(), // API control port
            #[cfg(feature = "zmq-transport")]
            zmq_motor_address: "tcp://0.0.0.0:5564".to_string(), // Motor output port (standard FEAGI port)
            #[cfg(feature = "zmq-transport")]
            zmq_viz_address: "tcp://0.0.0.0:5562".to_string(), // Visualization output port
            #[cfg(feature = "zmq-transport")]
            zmq_sensory_address: "tcp://0.0.0.0:5558".to_string(), // Sensory input port
            #[cfg(feature = "zmq-transport")]
            visualization_stream: VisualizationSendConfig::default(),
            #[cfg(feature = "zmq-transport")]
            sensory_stream: SensoryReceiveConfig::default(),

            // UDP defaults
            #[cfg(feature = "udp-transport")]
            udp_viz_config: UdpConfig {
                bind_address: "0.0.0.0:5565".to_string(),
                peer_address: "127.0.0.1:5565".to_string(),
                compress: true,
                max_message_size: 262144, // 256KB
            },
            #[cfg(feature = "udp-transport")]
            udp_sensory_config: UdpConfig {
                bind_address: "0.0.0.0:5566".to_string(),
                peer_address: "127.0.0.1:5566".to_string(),
                compress: true,
                max_message_size: 65536, // 64KB
            },

            // Default to ZMQ for compatibility (or UDP if ZMQ is not enabled)
            #[cfg(feature = "zmq-transport")]
            visualization_transport: TransportMode::Zmq,
            #[cfg(all(feature = "udp-transport", not(feature = "zmq-transport")))]
            visualization_transport: TransportMode::Udp,

            #[cfg(feature = "zmq-transport")]
            sensory_transport: TransportMode::Zmq,
            #[cfg(all(feature = "udp-transport", not(feature = "zmq-transport")))]
            sensory_transport: TransportMode::Udp,

            // WebSocket defaults
            websocket: WebSocketConfig::default(),

            // Shared
            shm_base_path: "/tmp".to_string(),
        }
    }
}
