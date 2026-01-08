// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//
//! Device registration utilities
//!
//! Centralizes the common workflow used by Rust-based controllers:
//! - Export device registrations from `ConnectorAgent`
//! - Count exported input/output/feedback entries
//! - Sync to FEAGI via HTTP (`/v1/agent/{agent_id}/device_registrations`)

use crate::sdk::error::{Result, SdkError};
use std::time::Duration;

#[cfg(feature = "sdk-io")]
use crate::sdk::ConnectorAgent;

#[cfg(feature = "sdk-io")]
use serde_json::Value;

#[cfg(feature = "sdk-io")]
use tracing;

/// Counts of exported device registration sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceRegistrationCounts {
    pub input_units: usize,
    pub output_units: usize,
    pub feedbacks: usize,
}

impl DeviceRegistrationCounts {
    pub const fn empty() -> Self {
        Self {
            input_units: 0,
            output_units: 0,
            feedbacks: 0,
        }
    }
}

#[cfg(feature = "sdk-io")]
pub fn counts_from_exported_json(exported: &Value) -> DeviceRegistrationCounts {
    let input_units = exported
        .get("input_units_and_encoder_properties")
        .and_then(|v| v.as_object())
        .map(|m| m.len())
        .unwrap_or(0);

    let output_units = exported
        .get("output_units_and_decoder_properties")
        .and_then(|v| v.as_object())
        .map(|m| m.len())
        .unwrap_or(0);

    let feedbacks = exported
        .get("feedbacks")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    DeviceRegistrationCounts {
        input_units,
        output_units,
        feedbacks,
    }
}

#[cfg(feature = "sdk-io")]
fn import_url(feagi_host: &str, feagi_api_port: u16, agent_id: &str) -> String {
    format!(
        "http://{}:{}/v1/agent/{}/device_registrations",
        feagi_host, feagi_api_port, agent_id
    )
}

/// Export device registrations and counts from a connector.
#[cfg(feature = "sdk-io")]
pub fn export_with_counts(connector: &ConnectorAgent) -> Result<(Value, DeviceRegistrationCounts)> {
    let exported = connector
        .export_device_registrations_as_config_json()
        .map_err(|e| SdkError::DeviceRegistrationSyncFailed(format!("export failed: {e}")))?;
    let counts = counts_from_exported_json(&exported);
    Ok((exported, counts))
}

/// Export device registrations and sync them to FEAGI using a pre-configured HTTP client.
///
/// Use this when you want to share a `reqwest::Client` across SDK components (TopologyCache + sync).
#[cfg(feature = "sdk-io")]
pub async fn export_and_sync_with_client(
    connector: &ConnectorAgent,
    http_client: &reqwest::Client,
    feagi_host: &str,
    feagi_api_port: u16,
    agent_id: &str,
) -> Result<DeviceRegistrationCounts> {
    let (exported, counts) = export_with_counts(connector)?;

    let url = import_url(feagi_host, feagi_api_port, agent_id);
    let payload = serde_json::json!({ "device_registrations": exported });

    let resp = http_client
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| SdkError::DeviceRegistrationSyncFailed(format!("http request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(SdkError::DeviceRegistrationSyncFailed(format!(
            "http {status}: {body}"
        )));
    }

    // Verify the import worked by immediately calling the export endpoint
    // This helps catch issues where import appears to succeed but data isn't stored
    let export_url = format!(
        "http://{}:{}/v1/agent/{}/device_registrations",
        feagi_host, feagi_api_port, agent_id
    );
    
    if let Ok(verify_resp) = http_client.get(&export_url).send().await {
        if verify_resp.status().is_success() {
            if let Ok(verify_json) = verify_resp.json::<serde_json::Value>().await {
                if let Some(exported_regs) = verify_json.get("device_registrations") {
                    let verify_counts = counts_from_exported_json(exported_regs);
                    if verify_counts.output_units != counts.output_units 
                        || verify_counts.input_units != counts.input_units {
                        tracing::warn!(
                            "⚠️ [SDK] Device registration sync verification mismatch: sent {} input/{} output, but server has {} input/{} output",
                            counts.input_units, counts.output_units,
                            verify_counts.input_units, verify_counts.output_units
                        );
                    } else {
                        tracing::debug!(
                            "✅ [SDK] Device registration sync verified: {} input, {} output, {} feedbacks",
                            verify_counts.input_units, verify_counts.output_units, verify_counts.feedbacks
                        );
                    }
                }
            }
        }
    }

    Ok(counts)
}

/// Export device registrations from a connector and sync them to FEAGI via HTTP.
///
/// This is used by all Rust controllers (video/text/voxel/perception-inspector) to publish their
/// registered capabilities into FEAGI's agent registry.
#[cfg(feature = "sdk-io")]
pub async fn export_and_sync(
    connector: &ConnectorAgent,
    feagi_host: &str,
    feagi_api_port: u16,
    agent_id: &str,
    http_timeout: Duration,
) -> Result<DeviceRegistrationCounts> {
    let client = reqwest::Client::builder()
        .timeout(http_timeout)
        .build()
        .map_err(|e| SdkError::DeviceRegistrationSyncFailed(format!("http client build failed: {e}")))?;
    export_and_sync_with_client(connector, &client, feagi_host, feagi_api_port, agent_id).await
}

