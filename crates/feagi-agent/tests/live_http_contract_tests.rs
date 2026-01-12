// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Live HTTP contract tests for FEAGI API compatibility.
//!
//! These tests are intended to be run in CI with a FEAGI HTTP API server running.
//! They validate end-to-end behavior of the API endpoints the SDK relies on.

#![cfg(feature = "sdk-io")]

use std::time::Duration;

use feagi_agent::sdk::types::{
    CorticalChannelCount, CorticalUnitIndex, FrameChangeHandling, MiscDataDimensions,
};

use feagi_agent::sdk::ConnectorAgent;

fn feagi_api_host() -> String {
    std::env::var("FEAGI_API_HOST")
        .expect("FEAGI_API_HOST must be set (no default). CI should set it.")
}

fn feagi_api_port() -> u16 {
    let raw = std::env::var("FEAGI_API_PORT")
        .expect("FEAGI_API_PORT must be set (no default). CI should set it.");
    raw.parse::<u16>()
        .expect("FEAGI_API_PORT must be a valid u16")
}

fn base_url() -> String {
    format!("http://{}:{}", feagi_api_host(), feagi_api_port())
}

#[tokio::test]
async fn device_registrations_import_then_export_roundtrip() {
    // Use a unique agent_id per run.
    let agent_id = format!(
        "sdk_live_contract_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    // Build a simple non-empty device registration export using the SDK connector.
    let connector = ConnectorAgent::new();
    {
        let mut sensor_cache = connector.get_sensor_cache();
        sensor_cache
            .misc_data_register(
                CorticalUnitIndex::from(0u8),
                CorticalChannelCount::new(1).unwrap(),
                FrameChangeHandling::Absolute,
                MiscDataDimensions::new(1, 1, 1).unwrap(),
            )
            .unwrap();
    }

    let exported = connector
        .export_device_registrations_as_config_json()
        .unwrap();

    // POST import
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let import_url = format!("{}/v1/agent/{}/device_registrations", base_url(), agent_id);
    let import_payload = serde_json::json!({ "device_registrations": exported });

    let resp = client
        .post(&import_url)
        .json(&import_payload)
        .send()
        .await
        .expect("import request failed");
    assert!(
        resp.status().is_success(),
        "import status expected success, got {}",
        resp.status()
    );

    // GET export (server should now have the registrations)
    let export_url = import_url;
    let resp = client
        .get(&export_url)
        .send()
        .await
        .expect("export request failed");
    assert!(
        resp.status().is_success(),
        "export status expected success, got {}",
        resp.status()
    );

    let body: serde_json::Value = resp.json().await.expect("export response not JSON");
    let device_regs = body
        .get("device_registrations")
        .expect("export response missing device_registrations");

    // Minimal shape checks: the sections we rely on must exist and be objects/arrays.
    assert!(
        device_regs
            .get("input_units_and_encoder_properties")
            .is_some(),
        "missing input_units_and_encoder_properties"
    );
    assert!(
        device_regs
            .get("output_units_and_decoder_properties")
            .is_some(),
        "missing output_units_and_decoder_properties"
    );
    assert!(device_regs.get("feedbacks").is_some(), "missing feedbacks");
}
