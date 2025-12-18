// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Transport selection and configuration
//!
//! Provides types and utilities for parsing FEAGI's transport
//! information from registration responses.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transport configuration from FEAGI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    pub transport_type: String,
    pub enabled: bool,
    pub ports: HashMap<String, u16>,
    pub host: String,
}

impl TransportConfig {
    /// Get port for a specific stream
    pub fn get_port(&self, stream: &str) -> Option<u16> {
        self.ports.get(stream).copied()
    }

    /// Check if transport supports a stream
    pub fn supports_stream(&self, stream: &str) -> bool {
        self.ports.contains_key(stream)
    }
}

/// Registration response from FEAGI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub status: String,
    pub message: Option<String>,
    pub shm_paths: Option<HashMap<String, String>>,
    pub zmq_ports: Option<HashMap<String, u16>>,
    pub transports: Option<Vec<TransportConfig>>,
    pub recommended_transport: Option<String>,
    /// Cortical area availability status for agent operations
    pub cortical_areas: serde_json::Value,
}

impl RegistrationResponse {
    /// Parse from JSON value
    pub fn from_json(value: &serde_json::Value) -> Result<Self, String> {
        serde_json::from_value(value.clone())
            .map_err(|e| format!("Failed to parse registration response: {}", e))
    }

    /// Get all available (enabled) transports
    pub fn available_transports(&self) -> Vec<&TransportConfig> {
        self.transports
            .as_ref()
            .map(|transports| transports.iter().filter(|t| t.enabled).collect())
            .unwrap_or_default()
    }

    /// Get transport by type
    pub fn get_transport(&self, transport_type: &str) -> Option<&TransportConfig> {
        self.transports.as_ref().and_then(|transports| {
            transports
                .iter()
                .find(|t| t.transport_type == transport_type && t.enabled)
        })
    }

    /// Choose best transport based on preference
    pub fn choose_transport(&self, preference: Option<&str>) -> Option<&TransportConfig> {
        let available = self.available_transports();

        if available.is_empty() {
            return None;
        }

        // Try preference first
        if let Some(pref) = preference {
            if let Some(transport) = self.get_transport(pref) {
                return Some(transport);
            }
        }

        // Fall back to recommended
        if let Some(recommended) = &self.recommended_transport {
            if let Some(transport) = self.get_transport(recommended) {
                return Some(transport);
            }
        }

        // Last resort: first available
        available.first().copied()
    }

    /// Get ZMQ ports (legacy support)
    pub fn get_zmq_ports(&self) -> HashMap<String, u16> {
        self.zmq_ports.clone().unwrap_or_default()
    }

    /// Check if transport is available
    pub fn has_transport(&self, transport_type: &str) -> bool {
        self.get_transport(transport_type).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_registration_response() {
        let json = serde_json::json!({
            "status": "success",
            "message": "Agent registered successfully",
            "zmq_ports": {
                "sensory": 5558,
                "motor": 5564,
                "visualization": 5562
            },
            "transports": [
                {
                    "transport_type": "zmq",
                    "enabled": true,
                    "ports": {
                        "sensory": 5558,
                        "motor": 5564,
                        "visualization": 5562
                    },
                    "host": "0.0.0.0"
                },
                {
                    "transport_type": "websocket",
                    "enabled": true,
                    "ports": {
                        "sensory": 9051,
                        "motor": 9052,
                        "visualization": 9050
                    },
                    "host": "0.0.0.0"
                }
            ],
            "recommended_transport": "zmq",
            "cortical_areas": {}
        });

        let response = RegistrationResponse::from_json(&json).unwrap();

        assert_eq!(response.status, "success");
        assert_eq!(response.available_transports().len(), 2);
        assert!(response.has_transport("zmq"));
        assert!(response.has_transport("websocket"));

        let zmq = response.get_transport("zmq").unwrap();
        assert_eq!(zmq.get_port("sensory"), Some(5558));

        let ws = response.get_transport("websocket").unwrap();
        assert_eq!(ws.get_port("sensory"), Some(9051));
    }

    #[test]
    fn test_choose_transport() {
        let json = serde_json::json!({
            "status": "success",
            "transports": [
                {
                    "transport_type": "zmq",
                    "enabled": true,
                    "ports": {"sensory": 5558},
                    "host": "0.0.0.0"
                },
                {
                    "transport_type": "websocket",
                    "enabled": true,
                    "ports": {"sensory": 9051},
                    "host": "0.0.0.0"
                }
            ],
            "recommended_transport": "zmq",
            "cortical_areas": {}
        });

        let response = RegistrationResponse::from_json(&json).unwrap();

        // Auto-select (uses recommended)
        let auto = response.choose_transport(None).unwrap();
        assert_eq!(auto.transport_type, "zmq");

        // Prefer WebSocket
        let ws = response.choose_transport(Some("websocket")).unwrap();
        assert_eq!(ws.transport_type, "websocket");

        // Prefer ZMQ
        let zmq = response.choose_transport(Some("zmq")).unwrap();
        assert_eq!(zmq.transport_type, "zmq");
    }
}
