//! Common types and utilities for WebSocket transport

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WsMessage {
    /// Sensory data from agent to FEAGI
    Sensory {
        agent_id: String,
        timestamp: u64,
        data: Vec<u8>,
        compressed: bool,
        sequence: u64,
    },
    
    /// Motor command from FEAGI to agent
    Motor {
        agent_id: String,
        timestamp: u64,
        command: String,
        data: Vec<u8>,
        sequence: u64,
    },
    
    /// Visualization data (binary only, no JSON wrapper)
    Visualization {
        data: Vec<u8>,
        compressed: bool,
    },
    
    /// Control command (request/response)
    ControlRequest {
        request_id: String,
        agent_id: String,
        method: String,
        path: String,
        body: Option<serde_json::Value>,
    },
    
    /// Control response
    ControlResponse {
        request_id: String,
        status: u16,
        body: Option<serde_json::Value>,
    },
    
    /// Heartbeat/ping
    Ping {
        timestamp: u64,
    },
    
    /// Heartbeat response
    Pong {
        timestamp: u64,
    },
}

impl WsMessage {
    /// Create a sensory message
    pub fn sensory(agent_id: String, data: Vec<u8>, compressed: bool, sequence: u64) -> Self {
        Self::Sensory {
            agent_id,
            timestamp: current_timestamp(),
            data,
            compressed,
            sequence,
        }
    }
    
    /// Create a motor message
    pub fn motor(agent_id: String, command: String, data: Vec<u8>, sequence: u64) -> Self {
        Self::Motor {
            agent_id,
            timestamp: current_timestamp(),
            command,
            data,
            sequence,
        }
    }
    
    /// Create a visualization message
    pub fn visualization(data: Vec<u8>, compressed: bool) -> Self {
        Self::Visualization { data, compressed }
    }
    
    /// Create a control request
    pub fn control_request(
        request_id: String,
        agent_id: String,
        method: String,
        path: String,
        body: Option<serde_json::Value>,
    ) -> Self {
        Self::ControlRequest {
            request_id,
            agent_id,
            method,
            path,
            body,
        }
    }
    
    /// Create a control response
    pub fn control_response(
        request_id: String,
        status: u16,
        body: Option<serde_json::Value>,
    ) -> Self {
        Self::ControlResponse {
            request_id,
            status,
            body,
        }
    }
    
    /// Create a ping message
    pub fn ping() -> Self {
        Self::Ping {
            timestamp: current_timestamp(),
        }
    }
    
    /// Create a pong message
    pub fn pong() -> Self {
        Self::Pong {
            timestamp: current_timestamp(),
        }
    }
}

/// Get current timestamp in milliseconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// WebSocket endpoint paths
pub mod endpoints {
    pub const SENSORY: &str = "/sensory";
    pub const MOTOR: &str = "/motor";
    pub const MOTOR_AGENT: &str = "/motor/{agent_id}";
    pub const VISUALIZATION: &str = "/visualization";
    pub const CONTROL: &str = "/control/{agent_id}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensory_message() {
        let msg = WsMessage::sensory(
            "robot_1".to_string(),
            vec![1, 2, 3],
            false,
            100,
        );
        
        match msg {
            WsMessage::Sensory { agent_id, data, compressed, sequence, .. } => {
                assert_eq!(agent_id, "robot_1");
                assert_eq!(data, vec![1, 2, 3]);
                assert!(!compressed);
                assert_eq!(sequence, 100);
            }
            _ => panic!("Wrong message type"),
        }
    }
    
    #[test]
    fn test_control_request() {
        let msg = WsMessage::control_request(
            "req_123".to_string(),
            "robot_1".to_string(),
            "GET".to_string(),
            "/genome/cortical_areas".to_string(),
            None,
        );
        
        match msg {
            WsMessage::ControlRequest { request_id, agent_id, method, path, .. } => {
                assert_eq!(request_id, "req_123");
                assert_eq!(agent_id, "robot_1");
                assert_eq!(method, "GET");
                assert_eq!(path, "/genome/cortical_areas");
            }
            _ => panic!("Wrong message type"),
        }
    }
}

