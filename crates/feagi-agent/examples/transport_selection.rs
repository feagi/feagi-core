// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Transport Selection Example
//!
//! Demonstrates how agents can query available transports and choose
//! which one to use based on their requirements.
//!
//! This example shows:
//! 1. Agent registration
//! 2. Parsing transport options from response
//! 3. Selecting appropriate transport
//! 4. Connecting using chosen transport
//!
//! Run with:
//! ```bash
//! cargo run --example transport_selection
//! ```

use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct TransportOption {
    transport_type: String,
    enabled: bool,
    ports: HashMap<String, u16>,
    host: String,
}

#[derive(Debug)]
struct RegistrationInfo {
    status: String,
    message: Option<String>,
    #[allow(dead_code)]
    zmq_ports: Option<HashMap<String, u16>>,
    transports: Vec<TransportOption>,
    recommended_transport: Option<String>,
}

fn parse_registration_response(response: &Value) -> Result<RegistrationInfo, String> {
    let body = response.get("body").ok_or("Missing body in response")?;

    // Parse transports
    let mut transports = Vec::new();
    if let Some(transport_array) = body.get("transports").and_then(|t| t.as_array()) {
        for transport in transport_array {
            let transport_type = transport
                .get("transport_type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown")
                .to_string();

            let enabled = transport
                .get("enabled")
                .and_then(|e| e.as_bool())
                .unwrap_or(false);

            let mut ports = HashMap::new();
            if let Some(ports_obj) = transport.get("ports").and_then(|p| p.as_object()) {
                for (key, value) in ports_obj {
                    if let Some(port) = value.as_u64() {
                        ports.insert(key.clone(), port as u16);
                    }
                }
            }

            let host = transport
                .get("host")
                .and_then(|h| h.as_str())
                .unwrap_or("0.0.0.0")
                .to_string();

            transports.push(TransportOption {
                transport_type,
                enabled,
                ports,
                host,
            });
        }
    }

    // Parse legacy ZMQ ports
    let zmq_ports = body
        .get("zmq_ports")
        .and_then(|p| p.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_u64().map(|port| (k.clone(), port as u16)))
                .collect()
        });

    Ok(RegistrationInfo {
        status: body
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string(),
        message: body
            .get("message")
            .and_then(|m| m.as_str())
            .map(|s| s.to_string()),
        zmq_ports,
        transports,
        recommended_transport: body
            .get("recommended_transport")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string()),
    })
}

fn choose_transport(
    reg_info: &RegistrationInfo,
    preference: Option<&str>,
) -> Option<TransportOption> {
    // Filter enabled transports
    let available: Vec<&TransportOption> =
        reg_info.transports.iter().filter(|t| t.enabled).collect();

    if available.is_empty() {
        return None;
    }

    // If preference specified, try that first
    if let Some(pref) = preference {
        if let Some(transport) = available.iter().find(|t| t.transport_type == pref) {
            return Some((*transport).clone());
        }
    }

    // Fall back to recommended
    if let Some(recommended) = &reg_info.recommended_transport {
        if let Some(transport) = available.iter().find(|t| &t.transport_type == recommended) {
            return Some((*transport).clone());
        }
    }

    // Last resort: first available
    available.first().map(|t| (*t).clone())
}

fn main() {
    println!("🦀 FEAGI Agent Transport Selection Example\n");

    // Simulate registration response from FEAGI
    let registration_response = serde_json::json!({
        "status": 200,
        "body": {
            "status": "success",
            "message": "Agent robot_01 registered successfully",
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
                        "visualization": 9050,
                        "registration": 9053
                    },
                    "host": "0.0.0.0"
                }
            ],
            "recommended_transport": "zmq"
        }
    });

    // Parse registration info
    let reg_info =
        parse_registration_response(&registration_response).expect("Failed to parse response");

    println!("📋 Registration Info:");
    println!("   Status: {}", reg_info.status);
    println!(
        "   Message: {}",
        reg_info.message.as_deref().unwrap_or("N/A")
    );
    println!(
        "   Recommended: {}",
        reg_info.recommended_transport.as_deref().unwrap_or("N/A")
    );
    println!();

    println!("🌐 Available Transports:");
    for (i, transport) in reg_info.transports.iter().enumerate() {
        println!(
            "   {}. {} (enabled: {})",
            i + 1,
            transport.transport_type,
            transport.enabled
        );
        println!("      Host: {}", transport.host);
        println!("      Ports:");
        for (stream, port) in &transport.ports {
            println!("        - {}: {}", stream, port);
        }
        println!();
    }

    // Example 1: Auto-select (use recommended)
    println!("🎯 Example 1: Auto-select (recommended)");
    let auto_transport = choose_transport(&reg_info, None).expect("No transports available");
    println!(
        "   Selected: {} on {}",
        auto_transport.transport_type, auto_transport.host
    );
    println!(
        "   Sensory port: {}",
        auto_transport.ports.get("sensory").unwrap_or(&0)
    );
    println!();

    // Example 2: Prefer WebSocket
    println!("🎯 Example 2: Prefer WebSocket");
    let ws_transport =
        choose_transport(&reg_info, Some("websocket")).expect("WebSocket not available");
    println!(
        "   Selected: {} on {}",
        ws_transport.transport_type, ws_transport.host
    );
    println!(
        "   Sensory port: {}",
        ws_transport.ports.get("sensory").unwrap_or(&0)
    );
    println!();

    // Example 3: Force ZMQ
    println!("🎯 Example 3: Force ZMQ");
    let zmq_transport = choose_transport(&reg_info, Some("zmq")).expect("ZMQ not available");
    println!(
        "   Selected: {} on {}",
        zmq_transport.transport_type, zmq_transport.host
    );
    println!(
        "   Sensory port: {}",
        zmq_transport.ports.get("sensory").unwrap_or(&0)
    );
    println!();

    // Example 4: Check what would happen in browser (WebSocket only capable)
    println!("🎯 Example 4: Browser scenario (WebSocket only)");
    let browser_transport = choose_transport(&reg_info, Some("websocket"))
        .expect("Browser requires WebSocket but it's not available");
    println!("   Browser will use: {}", browser_transport.transport_type);
    println!(
        "   Viz WebSocket: ws://{}:{}/visualization",
        browser_transport.host,
        browser_transport.ports.get("visualization").unwrap_or(&0)
    );

    println!("\n✅ Transport selection demonstration complete!");
}
