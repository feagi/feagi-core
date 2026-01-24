//! Cortical topology access and caching.

use std::collections::HashMap;
use std::sync::Arc;

use std::sync::RwLock;

use crate::core::SdkError;
use crate::sdk::types::CorticalID;

/// Parsed cortical topology for a single cortical area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorticalTopology {
    /// X dimension (width) per device.
    pub width: u32,
    /// Y dimension (height) per device.
    pub height: u32,
    /// Z dimension (depth) per device.
    pub depth: u32,
    /// Number of channels/devices.
    pub channels: u32,
}

/// HTTP-backed topology cache for cortical areas.
#[derive(Debug, Clone)]
pub struct TopologyCache {
    host: String,
    port: u16,
    timeout: std::time::Duration,
    cache: Arc<RwLock<HashMap<CorticalID, CorticalTopology>>>,
    #[cfg(feature = "sdk-io")]
    client: reqwest::Client,
}

impl TopologyCache {
    /// Create a new topology cache for a FEAGI HTTP endpoint.
    pub fn new(
        host: impl Into<String>,
        port: u16,
        timeout_s: f64,
    ) -> Result<Self, SdkError> {
        let host = host.into();
        let timeout = std::time::Duration::from_secs_f64(timeout_s);
        #[cfg(feature = "sdk-io")]
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| SdkError::Other(format!("TopologyCache HTTP client init failed: {e}")))?;
        Ok(Self {
            host,
            port,
            timeout,
            cache: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "sdk-io")]
            client,
        })
    }

    /// Fetch and cache topology for a single cortical ID.
    #[cfg(feature = "sdk-io")]
    pub async fn get_topology(&self, id: &CorticalID) -> Result<CorticalTopology, SdkError> {
        if let Ok(cache) = self.cache.read() {
            if let Some(existing) = cache.get(id).copied() {
                return Ok(existing);
            }
        }
        let payload = self.fetch_topologies(&[id.as_base_64()]).await?;
        let topo = Self::parse_topology_payload(id, &payload)?;
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(*id, topo);
        }
        Ok(topo)
    }

    /// Fetch and cache topologies for multiple cortical IDs.
    #[cfg(feature = "sdk-io")]
    pub async fn get_topologies(
        &self,
        ids: &[CorticalID],
    ) -> Result<Vec<CorticalTopology>, SdkError> {
        let missing: Vec<String> = {
            let cache = self.cache.read().map_err(|_| {
                SdkError::Other("Topology cache lock poisoned".to_string())
            })?;
            ids.iter()
                .filter(|id| !cache.contains_key(*id))
                .map(|id| id.as_base_64())
                .collect()
        };

        if !missing.is_empty() {
            let payload = self.fetch_topologies(&missing).await?;
            let mut cache = self
                .cache
                .write()
                .map_err(|_| SdkError::Other("Topology cache lock poisoned".to_string()))?;
            for id in ids {
                if cache.contains_key(id) {
                    continue;
                }
                let topo = Self::parse_topology_payload(id, &payload)?;
                cache.insert(*id, topo);
            }
        }

        let cache = self
            .cache
            .read()
            .map_err(|_| SdkError::Other("Topology cache lock poisoned".to_string()))?;
        ids.iter()
            .map(|id| {
                cache
                    .get(id)
                    .copied()
                    .ok_or_else(|| SdkError::Other(format!("Topology missing in cache: {}", id)))
            })
            .collect()
    }

    /// Prefetch and cache topologies for the provided cortical IDs.
    #[cfg(feature = "sdk-io")]
    pub async fn prefetch(&self, ids: &[CorticalID]) -> Result<(), SdkError> {
        self.get_topologies(ids).await.map(|_| ())
    }

    /// Clear all cached topology entries.
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Return the number of cached cortical topologies.
    pub fn cache_size(&self) -> usize {
        self.cache.read().map(|c| c.len()).unwrap_or(0)
    }

    /// Parse a topology payload returned by FEAGI HTTP APIs.
    pub fn parse_topology_payload(
        id: &CorticalID,
        payload: &serde_json::Value,
    ) -> Result<CorticalTopology, SdkError> {
        let key = id.as_base_64();
        let entry = payload
            .get(&key)
            .ok_or_else(|| SdkError::Other(format!("Topology payload missing key: {key}")))?;
        let entry_obj = entry.as_object().ok_or_else(|| {
            SdkError::Other(format!("Topology entry is not an object for key: {key}"))
        })?;

        let (dims, channels) = Self::parse_dimensions(entry_obj)
            .ok_or_else(|| SdkError::Other(format!("Topology dimensions missing for key: {key}")))?;

        Ok(CorticalTopology {
            width: dims.0,
            height: dims.1,
            depth: dims.2,
            channels,
        })
    }

    #[cfg(feature = "sdk-io")]
    async fn fetch_topologies(
        &self,
        cortical_ids: &[String],
    ) -> Result<serde_json::Value, SdkError> {
        let url = format!(
            "http://{}:{}/v1/cortical_area/multi/cortical_area_properties",
            self.host, self.port
        );
        let response = self
            .client
            .post(url)
            .json(&serde_json::Value::Array(
                cortical_ids.iter().map(|id| id.clone().into()).collect(),
            ))
            .send()
            .await
            .map_err(|e| SdkError::Other(format!("Topology request failed: {e}")))?;

        let response = response
            .error_for_status()
            .map_err(|e| SdkError::Other(format!("Topology response error: {e}")))?;

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| SdkError::Other(format!("Topology response parse failed: {e}")))
    }

    fn parse_dimensions(
        entry_obj: &serde_json::Map<String, serde_json::Value>,
    ) -> Option<((u32, u32, u32), u32)> {
        if let (Some(dim_val), Some(dev_count)) = (
            entry_obj.get("cortical_dimensions_per_device"),
            entry_obj.get("dev_count"),
        ) {
            let dims = Self::parse_dim_array(dim_val)?;
            let channels = dev_count.as_u64()? as u32;
            return Some((dims, channels));
        }

        if let Some(dim_val) = entry_obj.get("dimensions") {
            let dims = if dim_val.is_array() {
                Self::parse_dim_array(dim_val)?
            } else if dim_val.is_object() {
                Self::parse_dim_object(dim_val)?
            } else {
                return None;
            };
            let channels = entry_obj
                .get("dev_count")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)?;
            return Some((dims, channels));
        }

        // TODO: Support nested properties.cortical_dimensions_* shapes if API returns them.
        None
    }

    fn parse_dim_array(value: &serde_json::Value) -> Option<(u32, u32, u32)> {
        let arr = value.as_array()?;
        if arr.len() != 3 {
            return None;
        }
        let w = arr[0].as_u64()? as u32;
        let h = arr[1].as_u64()? as u32;
        let d = arr[2].as_u64()? as u32;
        Some((w, h, d))
    }

    fn parse_dim_object(value: &serde_json::Value) -> Option<(u32, u32, u32)> {
        let obj = value.as_object()?;
        let w = obj.get("x")?.as_u64()? as u32;
        let h = obj.get("y")?.as_u64()? as u32;
        let d = obj.get("z")?.as_u64()? as u32;
        Some((w, h, d))
    }
}
