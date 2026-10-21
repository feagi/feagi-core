//! Network: transport configuration and connection details (API, ZMQ, WebSocket, SHM, ...).
//! Converted from the old `network` endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;

/// API transport section
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfoApi {
    pub enabled: bool,
    pub base_url: String,
    pub host: String,
    pub port: u16,
    pub swagger_url: String,
}

/// ZMQ ports
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfoZmqPorts {
    pub registration: u16,
    pub sensory: u16,
    pub motor: u16,
    pub visualization: u16,
    pub api_control: u16,
}

/// ZMQ endpoint URLs
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfoZmqEndpoints {
    pub registration: String,
    pub sensory: String,
    pub motor: String,
    pub visualization: String,
}

/// ZMQ transport section
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfoZmq {
    pub enabled: bool,
    pub host: String,
    pub ports: ConnectionInfoZmqPorts,
    pub endpoints: ConnectionInfoZmqEndpoints,
}

/// WebSocket ports
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfoWebSocketPorts {
    pub registration: u16,
    pub sensory: u16,
    pub motor: u16,
    pub visualization: u16,
    pub rest_api: u16,
}

/// WebSocket endpoint URLs
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfoWebSocketEndpoints {
    pub registration: String,
    pub sensory: String,
    pub motor: String,
    pub visualization: String,
}

/// WebSocket transport section
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfoWebSocket {
    pub enabled: bool,
    pub host: String,
    pub ports: ConnectionInfoWebSocketPorts,
    pub endpoints: ConnectionInfoWebSocketEndpoints,
}

/// Shared memory transport section
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfoShm {
    pub enabled: bool,
    pub base_path: String,
    pub policy: String,
    pub note: String,
}

/// UDP transport section (placeholder)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfoUdp {
    pub enabled: bool,
    pub visualization: Option<serde_json::Value>,
    pub sensory: Option<serde_json::Value>,
    pub note: String,
}

/// Bluetooth transport section (placeholder)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfoBluetooth {
    pub enabled: bool,
    pub relay_port: Option<u16>,
    pub note: String,
}

/// Stream runtime status
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfoStreamStatus {
    pub zmq_control_started: bool,
    pub zmq_data_streams_started: bool,
    pub websocket_started: bool,
    pub note: String,
}

define_request_message_category! {
    category_name: "Network",
    base_path: "v1/network",
    category_description: "Network configuration, transport status and connection details",
    read: {
        "Status": {
            path: "status",
            description: "Get network configuration status including ZMQ, HTTP, and WebSocket states",
            response: {
                zmq_enabled: bool, "Whether the ZMQ transport is enabled",
                http_enabled: bool, "Whether the HTTP API is enabled",
                websocket_enabled: bool, "Whether the WebSocket transport is enabled",
            },
        },
        "ConnectionInfo": {
            path: "connection_info",
            description: "Get all active networking details: API, ZMQ, WebSocket, SHM, UDP (placeholder), Bluetooth (placeholder)",
            alloc_only: true,
            response: {
                api: ConnectionInfoApi, "HTTP API transport",
                zmq: ConnectionInfoZmq, "ZMQ transport",
                websocket: ConnectionInfoWebSocket, "WebSocket transport",
                shm: ConnectionInfoShm, "Shared memory transport",
                udp: ConnectionInfoUdp, "UDP transport (placeholder)",
                bluetooth: ConnectionInfoBluetooth, "Bluetooth relay (placeholder)",
                stream_status: ConnectionInfoStreamStatus, "Runtime state of the data streams",
            },
        },
    },
    create: {
        "Config": {
            path: "config",
            description: "Configure network parameters including transport protocols and ports",
            alloc_only: true,
            payload: {
                config: HashMap<String, serde_json::Value>, "Network configuration to apply",
            },
            response: {
                message: String, "Result message",
            },
        },
    },
}

// NOTES:
// Config.config is an untyped map of serde_json::Value; the old handler only checked it was present.
// Config response was a bare HashMap<String, String> with a single "message" key; mapped to `message`.
// Status: old handler returned HashMap<String, Value>; fields taken from NetworkStatusResponse DTO.
// ConnectionInfoUdp.visualization/sensory are untyped Option<Value> placeholders in the old DTO.
