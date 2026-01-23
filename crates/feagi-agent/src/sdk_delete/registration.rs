// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//
//! High-level registration workflow helpers
//!
//! Goal: minimize controller boilerplate while keeping behavior explicit and deterministic.
//! Controllers often need to:
//! - connect/register via `AgentClient`
//! - export device registrations from `ConnectorAgent`
//! - sync those registrations to FEAGI's agent registry via HTTP

use crate::sdk::error::{Result, SdkError};
use std::time::Duration;

#[cfg(feature = "sdk-io")]
use crate::sdk::util::device_registrations::{self, DeviceRegistrationCounts};

/// HTTP API configuration for FEAGI.
///
/// This is intentionally narrow (HTTP-only) so it can be reused across:
/// - `TopologyCache`
/// - device registration sync (`/v1/agent/{agent_id}/device_registrations`)
#[derive(Debug, Clone)]
pub struct FeagiApiConfig {
    pub host: String,
    pub port: u16,
    pub http_timeout: Duration,
}

impl FeagiApiConfig {
    pub fn new(host: impl Into<String>, port: u16, http_timeout: Duration) -> Self {
        Self {
            host: host.into(),
            port,
            http_timeout,
        }
    }
}

/// High-level registrar that centralizes common controller registration workflow.
///
/// Design goals:
/// - minimal controller boilerplate
/// - consistent HTTP timeout behavior
/// - ability to share a single `reqwest::Client` across topology + sync calls
#[cfg(feature = "sdk-io")]
#[derive(Clone)]
pub struct AgentRegistrar {
    api: FeagiApiConfig,
    http_client: reqwest::Client,
}

#[cfg(feature = "sdk-io")]
impl AgentRegistrar {
    /// Create a registrar with a configured reqwest client (timeout enforced).
    pub fn new(api: FeagiApiConfig) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(api.http_timeout)
            .build()
            .map_err(|e| {
                SdkError::DeviceRegistrationSyncFailed(format!("http client build failed: {e}"))
            })?;
        Ok(Self { api, http_client })
    }

    /// Create a `TopologyCache` that shares the registrar's HTTP client.
    pub fn topology_cache(&self) -> crate::sdk::base::TopologyCache {
        crate::sdk::base::TopologyCache::with_http_client(
            self.api.host.clone(),
            self.api.port,
            self.http_client.clone(),
        )
    }

    /// Sync device registrations without performing ZMQ registration.
    pub async fn sync_device_registrations(
        &self,
        connector: &crate::sdk::ConnectorAgent,
        agent_id: &str,
    ) -> Result<DeviceRegistrationCounts> {
        device_registrations::export_and_sync_with_client(
            connector,
            &self.http_client,
            &self.api.host,
            self.api.port,
            agent_id,
        )
        .await
    }

    /// Connect the client (ZMQ registration) and then sync device registrations (HTTP).
    ///
    /// This does **not** register any devices for you; controllers should register their devices
    /// on the `ConnectorAgent` before calling this function.
    pub async fn connect_and_sync(
        &self,
        client: &mut crate::core::AgentClient,
        connector: &crate::sdk::ConnectorAgent,
        agent_id: &str,
    ) -> Result<DeviceRegistrationCounts> {
        client.connect()?;
        self.sync_device_registrations(connector, agent_id).await
    }
}

/// Connect the client (ZMQ registration) and then sync device registrations (HTTP).
///
/// This does **not** register any devices for you; controllers should register their devices
/// on the `ConnectorAgent` before calling this function.
#[cfg(feature = "sdk-io")]
pub async fn connect_and_sync_device_registrations(
    client: &mut crate::core::AgentClient,
    connector: &crate::sdk::ConnectorAgent,
    feagi_host: &str,
    feagi_api_port: u16,
    agent_id: &str,
    http_timeout: Duration,
) -> Result<DeviceRegistrationCounts> {
    client.connect()?;
    device_registrations::export_and_sync(
        connector,
        feagi_host,
        feagi_api_port,
        agent_id,
        http_timeout,
    )
    .await
}

/// Sync device registrations without performing ZMQ registration.
///
/// Useful when the controller manages connection separately but wants a single call to publish
/// registrations into FEAGI's agent registry.
#[cfg(feature = "sdk-io")]
pub async fn sync_device_registrations(
    connector: &crate::sdk::ConnectorAgent,
    feagi_host: &str,
    feagi_api_port: u16,
    agent_id: &str,
    http_timeout: Duration,
) -> Result<DeviceRegistrationCounts> {
    device_registrations::export_and_sync(
        connector,
        feagi_host,
        feagi_api_port,
        agent_id,
        http_timeout,
    )
    .await
}
