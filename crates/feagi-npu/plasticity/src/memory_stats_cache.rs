// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Global memory area statistics cache
//!
//! Architecture: Event-driven stats updates (NOT query-based)
//! - Updated by PlasticityService when neurons created/deleted
//! - Read by health check API (O(1) cache read, no queries)
//! - Thread-safe via Arc<RwLock>

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Statistics for a single memory cortical area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAreaStats {
    /// Current number of active neurons in this memory area
    pub neuron_count: usize,

    /// Total neurons created since area was initialized
    pub created_total: usize,

    /// Total neurons deleted/expired since area was initialized
    pub deleted_total: usize,

    /// Last update timestamp (milliseconds since epoch)
    pub last_updated: u64,
}

impl Default for MemoryAreaStats {
    fn default() -> Self {
        Self {
            neuron_count: 0,
            created_total: 0,
            deleted_total: 0,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

/// Global memory statistics cache
/// Key: cortical_area_name (e.g., "mem_00")
/// Value: MemoryAreaStats
pub type MemoryStatsCache = Arc<RwLock<HashMap<String, MemoryAreaStats>>>;

/// Create a new empty memory stats cache
pub fn create_memory_stats_cache() -> MemoryStatsCache {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Update stats when a neuron is created
pub fn on_neuron_created(cache: &MemoryStatsCache, area_name: &str) {
    let mut stats = cache.write();
    let area_stats = stats.entry(area_name.to_string()).or_default();
    area_stats.neuron_count += 1;
    area_stats.created_total += 1;
    area_stats.last_updated = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
}

/// Update stats when a neuron is deleted/expired
pub fn on_neuron_deleted(cache: &MemoryStatsCache, area_name: &str) {
    let mut stats = cache.write();
    if let Some(area_stats) = stats.get_mut(area_name) {
        area_stats.neuron_count = area_stats.neuron_count.saturating_sub(1);
        area_stats.deleted_total += 1;
        area_stats.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }
}

/// Initialize stats for a new memory area
pub fn init_memory_area(cache: &MemoryStatsCache, area_name: &str) {
    let mut stats = cache.write();
    stats.entry(area_name.to_string()).or_default();
}

/// Remove stats for a deleted memory area
pub fn remove_memory_area(cache: &MemoryStatsCache, area_name: &str) {
    let mut stats = cache.write();
    stats.remove(area_name);
}

/// Get a snapshot of all memory area stats (for health check)
pub fn get_stats_snapshot(cache: &MemoryStatsCache) -> HashMap<String, MemoryAreaStats> {
    cache.read().clone()
}

/// Get stats for a specific memory area
pub fn get_area_stats(cache: &MemoryStatsCache, area_name: &str) -> Option<MemoryAreaStats> {
    cache.read().get(area_name).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats_creation() {
        let cache = create_memory_stats_cache();

        on_neuron_created(&cache, "mem_00");
        on_neuron_created(&cache, "mem_00");
        on_neuron_created(&cache, "mem_01");

        let snapshot = get_stats_snapshot(&cache);
        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot.get("mem_00").unwrap().neuron_count, 2);
        assert_eq!(snapshot.get("mem_01").unwrap().neuron_count, 1);
    }

    #[test]
    fn test_memory_stats_deletion() {
        let cache = create_memory_stats_cache();

        on_neuron_created(&cache, "mem_00");
        on_neuron_created(&cache, "mem_00");
        on_neuron_deleted(&cache, "mem_00");

        let stats = get_area_stats(&cache, "mem_00").unwrap();
        assert_eq!(stats.neuron_count, 1);
        assert_eq!(stats.created_total, 2);
        assert_eq!(stats.deleted_total, 1);
    }

    #[test]
    fn test_memory_area_removal() {
        let cache = create_memory_stats_cache();

        on_neuron_created(&cache, "mem_00");
        remove_memory_area(&cache, "mem_00");

        assert!(get_area_stats(&cache, "mem_00").is_none());
    }
}
