//! Visualization: visualization client registration and neural activity stream status.
//! Converted from the old `visualization` endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;

define_request_message_category! {
    category_name: "Visualization",
    base_path: "v1/visualization",
    category_description: "Visualization clients and neural activity streams",
    read: {
        "Status": {
            path: "status",
            description: "Get visualization system status including active clients and FQ sampler state",
            alloc_only: true,
            response: {
                enabled: bool, "Whether visualization is enabled",
                active_clients: u64, "Number of registered visualization clients",
                fq_sampler_enabled: bool, "Whether the FQ sampler is enabled",
                message: String, "Status message",
            },
        },
    },
    create: {
        "RegisterClient": {
            path: "register_client",
            description: "Register a new visualization client for receiving neural activity streams",
            alloc_only: true,
            payload: {
                client_id: Option<String>, "Client ID to use; generated (UUID) if omitted",
                metadata: Option<HashMap<String, serde_json::Value>>, "Arbitrary client metadata",
            },
            response: {
                client_id: String, "ID of the registered client",
                success: bool, "Whether registration succeeded",
                message: String, "Result message",
            },
        },
        "UnregisterClient": {
            path: "unregister_client",
            description: "Unregister a visualization client to stop receiving neural activity streams",
            alloc_only: true,
            payload: {
                client_id: String, "ID of the client to unregister",
            },
            response: {
                message: String, "Result message",
            },
        },
        "Heartbeat": {
            path: "heartbeat",
            description: "Send heartbeat from visualization client to maintain active connection",
            alloc_only: true,
            payload: {
                client_id: String, "ID of the client sending the heartbeat",
            },
            response: {
                message: String, "Result message",
            },
        },
    },
}

// NOTES:
// RegisterClient.metadata comes from the DTO only; the old handler ignored it. Untyped Value map.
// Heartbeat.client_id taken from VisualizationHeartbeatRequest DTO; the old handler ignored the body.
// UnregisterClient/Heartbeat responses were bare HashMap<String, String> with one "message" key.
// Status.active_clients was usize in the DTO; changed to u64.
