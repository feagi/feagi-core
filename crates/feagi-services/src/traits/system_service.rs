/*!
System service trait.

Provides system health, status, and configuration operations.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::types::*;
use async_trait::async_trait;

/// System service (transport-agnostic)
#[async_trait]
pub trait SystemService: Send + Sync {
    // ========================================================================
    // HEALTH & STATUS
    // ========================================================================

    /// Get system health status
    ///
    /// # Returns
    /// * `HealthStatus` - Overall system health with component statuses
    ///
    async fn get_health(&self) -> ServiceResult<HealthStatus>;

    /// Get system status (comprehensive information)
    ///
    /// # Returns
    /// * `SystemStatus` - Detailed system status including counters, memory, uptime
    ///
    async fn get_status(&self) -> ServiceResult<SystemStatus>;

    /// Get system version information
    ///
    /// # Returns
    /// * `VersionInfo` - Version numbers for all components
    ///
    async fn get_version(&self) -> ServiceResult<VersionInfo>;

    // ========================================================================
    // RUNTIME CONTROL
    // ========================================================================

    /// Check if the system is initialized (has a loaded genome)
    ///
    /// # Returns
    /// * `bool` - True if system is initialized
    ///
    async fn is_initialized(&self) -> ServiceResult<bool>;

    /// Get the current burst count (total bursts executed)
    ///
    /// # Returns
    /// * `u64` - Total burst count
    ///
    async fn get_burst_count(&self) -> ServiceResult<u64>;

    /// Get runtime statistics
    ///
    /// # Returns
    /// * `RuntimeStats` - Detailed runtime statistics
    ///
    async fn get_runtime_stats(&self) -> ServiceResult<RuntimeStats>;

    // ========================================================================
    // MEMORY & RESOURCES
    // ========================================================================

    /// Get memory usage information
    ///
    /// # Returns
    /// * `MemoryUsage` - Current memory usage across all components
    ///
    async fn get_memory_usage(&self) -> ServiceResult<MemoryUsage>;

    /// Get capacity information (max neurons, synapses, etc.)
    ///
    /// # Returns
    /// * `CapacityInfo` - Current and maximum capacities
    ///
    async fn get_capacity(&self) -> ServiceResult<CapacityInfo>;
}

