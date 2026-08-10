//! HTTP helpers for fetching FEAGI health snapshots.
//!
//! Both blocking and async variants are provided behind the
//! `agent-client-asynchelper-tokio` feature, which already pulls `reqwest`
//! into the dependency graph. Host, port, and timeout are caller-supplied:
//! no defaults are read from the environment, matching the centralized
//! configuration policy.

use crate::clients::recovery::health_watcher::HealthSnapshot;
use crate::command_and_control::health_check_message::HealthCheckResponse;
use crate::FeagiAgentError;
use std::time::Duration;

/// Caller-supplied configuration for fetching FEAGI health.
#[derive(Debug, Clone)]
pub struct HealthFetchConfig {
    pub feagi_api_host: String,
    pub feagi_api_port: u16,
    pub timeout: Duration,
}

impl HealthFetchConfig {
    pub fn new(host: impl Into<String>, port: u16, timeout: Duration) -> Self {
        Self {
            feagi_api_host: host.into(),
            feagi_api_port: port,
            timeout,
        }
    }

    pub fn url(&self) -> String {
        format!("http://{}:{}/v1/system/health_check", self.feagi_api_host, self.feagi_api_port)
    }
}

/// Async fetch of `/v1/system/health_check` -> [`HealthSnapshot`].
#[cfg(feature = "agent-client-asynchelper-tokio")]
pub async fn fetch_health_snapshot(config: &HealthFetchConfig) -> Result<HealthSnapshot, FeagiAgentError> {
    let client = reqwest::Client::builder()
        .timeout(config.timeout)
        .build()
        .map_err(|e| FeagiAgentError::connection_failed(format!("failed to build http client: {e}")))?;
    let response = client
        .get(config.url())
        .send()
        .await
        .map_err(|e| FeagiAgentError::connection_failed(format!("health_check request: {e}")))?;
    let parsed: HealthCheckResponse = response
        .json()
        .await
        .map_err(|e| FeagiAgentError::connection_failed(format!("health_check parse: {e}")))?;
    Ok(HealthSnapshot::from_response(&parsed))
}

/// Blocking fetch of `/v1/system/health_check` -> [`HealthSnapshot`].
#[cfg(feature = "agent-client-asynchelper-tokio")]
pub fn fetch_health_snapshot_blocking(config: &HealthFetchConfig) -> Result<HealthSnapshot, FeagiAgentError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(config.timeout)
        .build()
        .map_err(|e| FeagiAgentError::connection_failed(format!("failed to build http client: {e}")))?;
    let response = client
        .get(config.url())
        .send()
        .map_err(|e| FeagiAgentError::connection_failed(format!("health_check request: {e}")))?;
    let parsed: HealthCheckResponse = response
        .json()
        .map_err(|e| FeagiAgentError::connection_failed(format!("health_check parse: {e}")))?;
    Ok(HealthSnapshot::from_response(&parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_formats_host_and_port() {
        let config = HealthFetchConfig::new("feagi.local", 8000, Duration::from_millis(500));
        assert_eq!(config.url(), "http://feagi.local:8000/v1/system/health_check");
    }
}
