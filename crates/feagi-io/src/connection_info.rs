// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! PNS connection info for API discovery
//!
//! Provides snapshot of active transports and stream status for GET /v1/network/connection_info.

/// Parse host and port from address string like "tcp://127.0.0.1:30001"
pub(crate) fn parse_address(addr: &str) -> (String, u16) {
    let s = addr
        .strip_prefix("tcp://")
        .or_else(|| addr.strip_prefix("ws://"))
        .unwrap_or(addr);
    let parts: Vec<&str> = s.split(':').collect();
    let host = parts.first().map(|h| (*h).to_string()).unwrap_or_else(|| "127.0.0.1".to_string());
    let port = parts
        .get(1)
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(0);
    (host, port)
}

/// Stream runtime status
#[derive(Debug, Clone)]
pub struct StreamStatus {
    pub zmq_control_started: bool,
    pub zmq_data_streams_started: bool,
    pub websocket_started: bool,
}

/// Connection config snapshot (for building API response)
#[derive(Debug, Clone)]
pub struct PnsConnectionConfigSnapshot {
    pub zmq_host: String,
    pub zmq_registration_port: u16,
    pub zmq_sensory_port: u16,
    pub zmq_motor_port: u16,
    pub zmq_viz_port: u16,
    pub zmq_api_control_port: u16,
    pub ws_enabled: bool,
    pub ws_host: String,
    pub ws_registration_port: u16,
    pub ws_sensory_port: u16,
    pub ws_motor_port: u16,
    pub ws_viz_port: u16,
    pub ws_rest_api_port: u16,
    pub shm_base_path: String,
}
