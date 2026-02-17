//! SDK base types (e.g. topology parsing).

use feagi_structures::genomic::cortical_area::CorticalID;

/// Parsed topology dimensions for a cortical area (from connectome/topology payload).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologyCache {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub channels: u32,
}

impl TopologyCache {
    /// Parse topology from JSON payload. Supports modern schema
    /// (`cortical_dimensions_per_device`, `dev_count`) and legacy (`dimensions`, `dev_count`).
    pub fn parse_topology_payload(
        id: &CorticalID,
        payload: &serde_json::Value,
    ) -> Result<Self, crate::FeagiAgentClientError> {
        let key = id.as_base_64();
        let obj = payload
            .as_object()
            .ok_or_else(|| {
                crate::FeagiAgentClientError::UnableToDecodeReceivedData(
                    "topology payload must be a JSON object".to_string(),
                )
            })?
            .get(&key)
            .ok_or_else(|| {
                crate::FeagiAgentClientError::UnableToDecodeReceivedData(format!(
                    "topology payload missing key {}",
                    key
                ))
            })?;

        let (width, height, depth, channels) = if let Some(dims) =
            obj.get("cortical_dimensions_per_device")
        {
            let arr = dims.as_array().ok_or_else(|| {
                crate::FeagiAgentClientError::UnableToDecodeReceivedData(
                    "cortical_dimensions_per_device must be array".to_string(),
                )
            })?;
            let w = arr.first().and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let h = arr.get(1).and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let d = arr.get(2).and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let dev_count = obj.get("dev_count").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            (w, h, d, dev_count)
        } else if let Some(dims) = obj.get("dimensions") {
            let arr = dims.as_array().ok_or_else(|| {
                crate::FeagiAgentClientError::UnableToDecodeReceivedData(
                    "dimensions must be array".to_string(),
                )
            })?;
            let w = arr.first().and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let h = arr.get(1).and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let d = arr.get(2).and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let dev_count = obj.get("dev_count").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            (w, h, d, dev_count)
        } else {
            return Err(crate::FeagiAgentClientError::UnableToDecodeReceivedData(
                "topology entry must have cortical_dimensions_per_device or dimensions".to_string(),
            ));
        };

        Ok(TopologyCache {
            width,
            height,
            depth,
            channels,
        })
    }
}

/// Client that fetches topology from the FEAGI API. Returned by `AgentRegistrar::topology_cache()`.
#[cfg(feature = "sdk-io")]
#[derive(Clone)]
pub struct TopologyClient {
    base_url: String,
}

#[cfg(feature = "sdk-io")]
impl TopologyClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    /// Clear cached topology (no-op for HTTP client).
    pub fn clear_cache(&self) {}

    /// Fetch topology for a cortical ID from GET /v1/connectome/topology.
    pub async fn get_topology(
        &self,
        id: &CorticalID,
    ) -> Result<TopologyCache, crate::FeagiAgentClientError> {
        let url = format!(
            "{}/v1/connectome/topology",
            self.base_url.trim_end_matches('/')
        );
        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::FeagiAgentClientError::ConnectionFailed(e.to_string()))?;
        let payload = resp
            .json::<serde_json::Value>()
            .await
            .map_err(|e| crate::FeagiAgentClientError::UnableToDecodeReceivedData(e.to_string()))?;
        TopologyCache::parse_topology_payload(id, &payload)
    }
}
