//! HTTP registration helpers for FEAGI agents.

use crate::core::SdkError;
use crate::sdk::base::TopologyCache;

/// Configuration for FEAGI HTTP API access.
#[derive(Debug, Clone)]
pub struct FeagiApiConfig {
    host: String,
    port: u16,
    timeout: std::time::Duration,
}

impl FeagiApiConfig {
    /// Create a new FEAGI API configuration.
    pub fn new(host: impl Into<String>, port: u16, timeout: std::time::Duration) -> Self {
        Self {
            host: host.into(),
            port,
            timeout,
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

/// Device registration counts returned from a sync operation.
#[derive(Debug, Clone, Copy)]
pub struct DeviceRegistrationCounts {
    pub input_units: usize,
    pub output_units: usize,
    pub feedbacks: usize,
}

/// Registrar helper for syncing device registrations and accessing topology.
#[derive(Debug, Clone)]
pub struct AgentRegistrar {
    config: FeagiApiConfig,
    #[cfg(feature = "sdk-io")]
    client: reqwest::Client,
}

impl AgentRegistrar {
    /// Create a new registrar with FEAGI API configuration.
    pub fn new(config: FeagiApiConfig) -> Result<Self, SdkError> {
        #[cfg(feature = "sdk-io")]
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| SdkError::Other(format!("Registrar HTTP client init failed: {e}")))?;
        Ok(Self {
            config,
            #[cfg(feature = "sdk-io")]
            client,
        })
    }

    /// Build a topology cache tied to this registrar's API config.
    pub fn topology_cache(&self) -> Result<TopologyCache, SdkError> {
        TopologyCache::new(
            self.config.host.clone(),
            self.config.port,
            self.config.timeout.as_secs_f64(),
        )
    }

    /// Export registrations from the connector and push them to FEAGI.
    #[cfg(feature = "sdk-io")]
    pub async fn sync_device_registrations(
        &self,
        device_registrations: serde_json::Value,
        agent_id: &str,
    ) -> Result<DeviceRegistrationCounts, SdkError> {
        let counts = count_device_registrations(&device_registrations)?;

        let url = format!(
            "{}/v1/agent/{}/device_registrations",
            self.config.base_url(),
            agent_id
        );

        let response = self
            .client
            .post(url)
            .json(&serde_json::json!({
                "device_registrations": device_registrations
            }))
            .send()
            .await
            .map_err(|e| SdkError::Other(format!("Device registration sync failed: {e}")))?;

        response
            .error_for_status()
            .map_err(|e| SdkError::Other(format!("Device registration sync error: {e}")))?;

        Ok(counts)
    }
}

fn count_device_registrations(
    device_registrations: &serde_json::Value,
) -> Result<DeviceRegistrationCounts, SdkError> {
    let input_units = device_registrations
        .get("input_units_and_encoder_properties")
        .and_then(|v| v.as_object())
        .map(|m| m.len())
        .ok_or_else(|| SdkError::Other("Device registrations missing input units".to_string()))?;
    let output_units = device_registrations
        .get("output_units_and_decoder_properties")
        .and_then(|v| v.as_object())
        .map(|m| m.len())
        .ok_or_else(|| SdkError::Other("Device registrations missing output units".to_string()))?;
    let feedbacks_value = device_registrations
        .get("feedbacks")
        .ok_or_else(|| SdkError::Other("Device registrations missing feedbacks".to_string()))?;
    let feedbacks = if let Some(list) = feedbacks_value.as_array() {
        list.len()
    } else if let Some(obj) = feedbacks_value.as_object() {
        obj.get("registered_feedbacks")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .ok_or_else(|| {
                SdkError::Other(
                    "Device registrations feedbacks missing registered_feedbacks".to_string(),
                )
            })?
    } else {
        return Err(SdkError::Other(
            "Device registrations feedbacks must be an array or object".to_string(),
        ));
    };

    Ok(DeviceRegistrationCounts {
        input_units,
        output_units,
        feedbacks,
    })
}
