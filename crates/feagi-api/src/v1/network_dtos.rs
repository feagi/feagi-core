// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Network API DTOs
//!
//! Request/response types for network configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Network status response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NetworkStatusResponse {
    pub zmq_enabled: bool,
    pub http_enabled: bool,
    pub websocket_enabled: bool,
}

/// Network configuration request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NetworkConfigRequest {
    pub config: HashMap<String, serde_json::Value>,
}

/// Success response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NetworkSuccessResponse {
    pub message: String,
    pub success: bool,
}

// ============================================================================
// Connection Info (GET /v1/network/connection_info)
// ============================================================================

/// API transport section
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectionInfoApi {
    pub enabled: bool,
    pub base_url: String,
    pub host: String,
    pub port: u16,
    pub swagger_url: String,
}

/// ZMQ transport section
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectionInfoZmq {
    pub enabled: bool,
    pub host: String,
    pub ports: ConnectionInfoZmqPorts,
    pub endpoints: ConnectionInfoZmqEndpoints,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectionInfoZmqPorts {
    pub registration: u16,
    pub sensory: u16,
    pub motor: u16,
    pub visualization: u16,
    pub api_control: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectionInfoZmqEndpoints {
    pub registration: String,
    pub sensory: String,
    pub motor: String,
    pub visualization: String,
}

/// WebSocket transport section
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectionInfoWebSocket {
    pub enabled: bool,
    pub host: String,
    pub ports: ConnectionInfoWebSocketPorts,
    pub endpoints: ConnectionInfoWebSocketEndpoints,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectionInfoWebSocketPorts {
    pub registration: u16,
    pub sensory: u16,
    pub motor: u16,
    pub visualization: u16,
    pub rest_api: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectionInfoWebSocketEndpoints {
    pub registration: String,
    pub sensory: String,
    pub motor: String,
    pub visualization: String,
}

/// SHM transport section
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectionInfoShm {
    pub enabled: bool,
    pub base_path: String,
    pub policy: String,
    pub note: String,
}

/// UDP transport section (placeholder)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectionInfoUdp {
    pub enabled: bool,
    pub visualization: Option<serde_json::Value>,
    pub sensory: Option<serde_json::Value>,
    pub note: String,
}

/// Bluetooth transport section (placeholder)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectionInfoBluetooth {
    pub enabled: bool,
    pub relay_port: Option<u16>,
    pub note: String,
}

/// Stream runtime status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConnectionInfoStreamStatus {
    pub zmq_control_started: bool,
    pub zmq_data_streams_started: bool,
    pub websocket_started: bool,
    pub note: String,
}

/// Full network connection info response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NetworkConnectionInfo {
    pub api: ConnectionInfoApi,
    pub zmq: ConnectionInfoZmq,
    pub websocket: ConnectionInfoWebSocket,
    pub shm: ConnectionInfoShm,
    pub udp: ConnectionInfoUdp,
    pub bluetooth: ConnectionInfoBluetooth,
    pub stream_status: ConnectionInfoStreamStatus,
}
