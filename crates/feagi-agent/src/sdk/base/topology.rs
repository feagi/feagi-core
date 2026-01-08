// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Topology cache for cortical area information

use crate::sdk::error::{Result, SdkError};
use feagi_structures::genomic::cortical_area::CorticalID;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Cortical area topology information
#[derive(Debug, Clone)]
pub struct CorticalTopology {
    pub cortical_id: CorticalID,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub channels: u32,
}

/// Shared topology cache to avoid redundant HTTP requests
///
/// This cache is designed for explicit dependency injection:
/// ```ignore
/// let topology_cache = TopologyCache::new("localhost", 8080, 5.0)?;
///
/// // Pass to multiple controllers
/// let video = VideoController::new(config, topology_cache.clone())?;
/// let text = TextController::new(config, topology_cache.clone())?;
/// ```
#[derive(Clone)]
pub struct TopologyCache {
    cache: Arc<RwLock<HashMap<String, CorticalTopology>>>,
    http_client: reqwest::Client,
    feagi_host: String,
    feagi_api_port: u16,
}

impl TopologyCache {
    /// Create a new topology cache
    ///
    /// # Arguments
    /// * `feagi_host` - FEAGI API host (e.g., "localhost" or "192.168.1.100")
    /// * `feagi_api_port` - FEAGI API port (typically 8080)
    /// * `http_timeout_s` - HTTP request timeout in seconds
    ///
    /// # Example
    /// ```ignore
    /// let cache = TopologyCache::new("localhost", 8080, 5.0)?;
    /// ```
    pub fn new(
        feagi_host: impl Into<String>,
        feagi_api_port: u16,
        http_timeout_s: f64,
    ) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs_f64(http_timeout_s))
            .build()?;

        Ok(Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            http_client,
            feagi_host: feagi_host.into(),
            feagi_api_port,
        })
    }

    /// Create a new topology cache using a pre-configured HTTP client.
    ///
    /// This is useful when you want to share a single `reqwest::Client` (with a consistent
    /// timeout, headers, proxy settings, etc.) across multiple SDK components.
    pub fn with_http_client(
        feagi_host: impl Into<String>,
        feagi_api_port: u16,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            http_client,
            feagi_host: feagi_host.into(),
            feagi_api_port,
        }
    }

    /// Fetch topology for a single cortical area (with caching)
    ///
    /// This method checks the cache first. If not found, it fetches from FEAGI
    /// and caches the result for future calls.
    ///
    /// # Example
    /// ```ignore
    /// let topology = cache.get_topology(&cortical_id).await?;
    /// println!("Resolution: {}x{}x{}", topology.width, topology.height, topology.depth);
    /// ```
    pub async fn get_topology(&self, cortical_id: &CorticalID) -> Result<CorticalTopology> {
        let key = cortical_id.as_base_64();

        // Check cache first (read lock)
        {
            let cache = self.cache.read().unwrap();
            if let Some(topo) = cache.get(&key) {
                return Ok(topo.clone());
            }
        }

        // Fetch from FEAGI
        let topo = self.fetch_from_feagi(cortical_id).await?;

        // Cache result (write lock)
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(key, topo.clone());
        }

        Ok(topo)
    }

    /// Fetch topologies for multiple cortical areas in parallel
    ///
    /// This is more efficient than calling `get_topology` in a loop.
    ///
    /// # Example
    /// ```ignore
    /// let topologies = cache.get_topologies(&[id1, id2, id3]).await?;
    /// ```
    pub async fn get_topologies(
        &self,
        cortical_ids: &[CorticalID],
    ) -> Result<Vec<CorticalTopology>> {
        let futures = cortical_ids.iter().map(|id| self.get_topology(id));
        futures::future::try_join_all(futures).await
    }

    /// Pre-fetch topologies without returning them (warm the cache)
    ///
    /// Useful for warming the cache during initialization before real-time operation.
    ///
    /// # Example
    /// ```ignore
    /// // During initialization (non-time-critical)
    /// cache.prefetch(&[id1, id2, id3]).await?;
    ///
    /// // Later, during real-time operation (fast, cached)
    /// let topology = cache.get_topology(&id1).await?; // No network call
    /// ```
    pub async fn prefetch(&self, cortical_ids: &[CorticalID]) -> Result<()> {
        self.get_topologies(cortical_ids).await?;
        Ok(())
    }

    /// Clear the cache
    ///
    /// Useful for testing or when topologies change in FEAGI.
    pub fn clear_cache(&self) {
        self.cache.write().unwrap().clear();
    }

    /// Get cache statistics
    pub fn cache_size(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    /// Fetch topology from FEAGI (internal, unified implementation)
    async fn fetch_from_feagi(&self, cortical_id: &CorticalID) -> Result<CorticalTopology> {
        let url = format!(
            "http://{}:{}/v1/cortical_area/multi/cortical_area_properties",
            self.feagi_host, self.feagi_api_port
        );

        let cortical_b64 = cortical_id.as_base_64();
        let resp = self
            .http_client
            .post(url)
            .json(&vec![cortical_b64.clone()])
            .send()
            .await?
            .error_for_status()?;

        let payload: serde_json::Value = resp.json().await?;
        Self::parse_topology_payload(cortical_id, &payload)
    }

    /// Parse the topology response payload from FEAGI.
    ///
    /// Exposed primarily for contract tests to ensure SDK parsing stays compatible with
    /// FEAGI backend API changes.
    pub fn parse_topology_payload(cortical_id: &CorticalID, payload: &Value) -> Result<CorticalTopology> {
        let cortical_b64 = cortical_id.as_base_64();
        let info = payload
            .get(&cortical_b64)
            .ok_or_else(|| SdkError::TopologyNotFound(cortical_b64.clone()))?;

        // Try modern fields first (cortical_dimensions_per_device + dev_count)
        if let (Some(per_device_dims), Some(dev_count)) = (
            info.get("cortical_dimensions_per_device"),
            info.get("dev_count"),
        ) {
            let dims = per_device_dims
                .as_array()
                .ok_or_else(|| {
                    SdkError::InvalidConfiguration(
                        "cortical_dimensions_per_device is not an array".to_string(),
                    )
                })?;

            if dims.len() != 3 {
                return Err(SdkError::InvalidConfiguration(format!(
                    "cortical_dimensions_per_device expected len=3, got {}",
                    dims.len()
                )));
            }

            let width = dims[0].as_u64().ok_or_else(|| {
                SdkError::InvalidConfiguration("width is not a number".to_string())
            })? as u32;
            let height = dims[1].as_u64().ok_or_else(|| {
                SdkError::InvalidConfiguration("height is not a number".to_string())
            })? as u32;
            let depth = dims[2].as_u64().ok_or_else(|| {
                SdkError::InvalidConfiguration("depth is not a number".to_string())
            })? as u32;
            let channels = dev_count.as_u64().ok_or_else(|| {
                SdkError::InvalidConfiguration("dev_count is not a number".to_string())
            })? as u32;

            return Ok(CorticalTopology {
                cortical_id: cortical_id.clone(),
                width,
                height,
                depth,
                channels,
            });
        }

        // Fallback to legacy fields (dimensions or cortical_dimensions)
        let dims = info
            .get("dimensions")
            .or_else(|| info.get("cortical_dimensions"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                SdkError::InvalidConfiguration(
                    "Topology response missing dimensions array".to_string(),
                )
            })?;

        if dims.len() != 3 {
            return Err(SdkError::InvalidConfiguration(format!(
                "dimensions expected len=3, got {}",
                dims.len()
            )));
        }

        let width = dims[0]
            .as_u64()
            .ok_or_else(|| SdkError::InvalidConfiguration("width is not a number".to_string()))?
            as u32;
        let height = dims[1]
            .as_u64()
            .ok_or_else(|| SdkError::InvalidConfiguration("height is not a number".to_string()))?
            as u32;
        let depth = dims[2]
            .as_u64()
            .ok_or_else(|| SdkError::InvalidConfiguration("depth is not a number".to_string()))?
            as u32;

        // Default to 1 channel if dev_count is missing (backward compatibility)
        let channels = info
            .get("dev_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;

        Ok(CorticalTopology {
            cortical_id: cortical_id.clone(),
            width,
            height,
            depth,
            channels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_cache_creation() {
        let cache = TopologyCache::new("localhost", 8080, 5.0);
        assert!(cache.is_ok());
    }

    #[test]
    fn test_cache_clear() {
        let cache = TopologyCache::new("localhost", 8080, 5.0).unwrap();
        assert_eq!(cache.cache_size(), 0);
        cache.clear_cache();
        assert_eq!(cache.cache_size(), 0);
    }
}

