// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
 * FEAGI v1 Network API
 *
 * Endpoints for network configuration and status
 * Maps to Python: feagi/api/v1/network.py
 */

use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, State};
use crate::v1::{
    ConnectionInfoApi, ConnectionInfoBluetooth, ConnectionInfoShm, ConnectionInfoStreamStatus,
    ConnectionInfoUdp, ConnectionInfoWebSocket, ConnectionInfoWebSocketEndpoints,
    ConnectionInfoWebSocketPorts, ConnectionInfoZmq, ConnectionInfoZmqEndpoints,
    ConnectionInfoZmqPorts, NetworkConnectionInfo,
};
use serde_json::{json, Value};
use std::collections::HashMap;

// ============================================================================
// NETWORK CONFIGURATION
// ============================================================================

/// Get network configuration status including ZMQ, HTTP, and WebSocket states.
#[utoipa::path(
    get,
    path = "/v1/network/status",
    tag = "network",
    responses(
        (status = 200, description = "Network status", body = HashMap<String, serde_json::Value>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_status(State(_state): State<ApiState>) -> ApiResult<Json<HashMap<String, Value>>> {
    // TODO: Retrieve network status
    let mut response = HashMap::new();
    response.insert("zmq_enabled".to_string(), json!(false));
    response.insert("http_enabled".to_string(), json!(true));
    response.insert("websocket_enabled".to_string(), json!(false));

    Ok(Json(response))
}

/// Configure network parameters including transport protocols and ports.
#[utoipa::path(
    post,
    path = "/v1/network/config",
    tag = "network",
    responses(
        (status = 200, description = "Network configured", body = HashMap<String, String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_config(
    State(_state): State<ApiState>,
    Json(request): Json<HashMap<String, Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // Validate config is provided
    let _config = request
        .get("config")
        .ok_or_else(|| ApiError::invalid_input("Missing 'config' field"))?;

    // TODO: Apply network configuration
    tracing::info!(target: "feagi-api", "Network configuration updated");

    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Network configured successfully".to_string(),
    )])))
}

// ============================================================================
// CONNECTION INFO
// ============================================================================

/// Provider trait for network connection info (implemented by embedders like feagi-rs)
pub trait NetworkConnectionInfoProvider: Send + Sync {
    fn get(&self) -> NetworkConnectionInfo;
}

/// Get all active networking details: API, ZMQ, WebSocket, SHM, UDP (placeholder), Bluetooth (placeholder).
#[utoipa::path(
    get,
    path = "/v1/network/connection_info",
    tag = "network",
    responses(
        (status = 200, description = "Network connection info", body = NetworkConnectionInfo),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_connection_info(
    State(state): State<ApiState>,
) -> ApiResult<Json<NetworkConnectionInfo>> {
    let info = state
        .network_connection_info_provider
        .as_ref()
        .map(|p| p.get())
        .unwrap_or_else(placeholder_connection_info);

    Ok(Json(info))
}

fn placeholder_connection_info() -> NetworkConnectionInfo {
    NetworkConnectionInfo {
        api: ConnectionInfoApi {
            enabled: true,
            base_url: "http://127.0.0.1:8000".to_string(),
            host: "127.0.0.1".to_string(),
            port: 8000,
            swagger_url: "http://127.0.0.1:8000/swagger-ui/".to_string(),
        },
        zmq: ConnectionInfoZmq {
            enabled: false,
            host: "127.0.0.1".to_string(),
            ports: ConnectionInfoZmqPorts {
                registration: 30001,
                sensory: 5558,
                motor: 5564,
                visualization: 5562,
                api_control: 5565,
            },
            endpoints: ConnectionInfoZmqEndpoints {
                registration: "tcp://127.0.0.1:30001".to_string(),
                sensory: "tcp://127.0.0.1:5558".to_string(),
                motor: "tcp://127.0.0.1:5564".to_string(),
                visualization: "tcp://127.0.0.1:5562".to_string(),
            },
        },
        websocket: ConnectionInfoWebSocket {
            enabled: false,
            host: "127.0.0.1".to_string(),
            ports: ConnectionInfoWebSocketPorts {
                registration: 9053,
                sensory: 9051,
                motor: 9052,
                visualization: 9050,
                rest_api: 9054,
            },
            endpoints: ConnectionInfoWebSocketEndpoints {
                registration: "ws://127.0.0.1:9053".to_string(),
                sensory: "ws://127.0.0.1:9051".to_string(),
                motor: "ws://127.0.0.1:9052".to_string(),
                visualization: "ws://127.0.0.1:9050".to_string(),
            },
        },
        shm: ConnectionInfoShm {
            enabled: false,
            base_path: "/tmp".to_string(),
            policy: "auto".to_string(),
            note: "Actual paths are allocated per-agent at registration".to_string(),
        },
        udp: ConnectionInfoUdp {
            enabled: false,
            visualization: None,
            sensory: None,
            note: "Placeholder for future UDP transport support".to_string(),
        },
        bluetooth: ConnectionInfoBluetooth {
            enabled: false,
            relay_port: None,
            note: "Placeholder for future use. Bluetooth relay is provided by feagi-desktop for embodied controllers, not by FEAGI server".to_string(),
        },
        // TODO dont use this struct
        stream_status: ConnectionInfoStreamStatus {
            zmq_control_started: false,
            zmq_data_streams_started: false,
            websocket_started: false,
            note: "Data streams start when genome is loaded and agents with matching capabilities are registered"
                .to_string(),
        },
    }
}
