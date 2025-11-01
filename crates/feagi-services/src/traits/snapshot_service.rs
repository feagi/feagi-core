// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Snapshot management service trait
//!
//! This service manages brain snapshots (genome + optional NPU state).

use async_trait::async_trait;
use crate::types::*;

/// Snapshot metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotMetadata {
    pub snapshot_id: String,
    pub created_at: String,
    pub name: String,
    pub description: Option<String>,
    pub stateful: bool,
    pub size_bytes: u64,
}

/// Snapshot creation options
#[derive(Debug, Clone)]
pub struct SnapshotCreateOptions {
    pub name: Option<String>,
    pub description: Option<String>,
    pub stateful: bool, // Include NPU state
    pub compression: bool, // Compress snapshot
}

/// Service for managing brain snapshots
#[async_trait]
pub trait SnapshotService: Send + Sync {
    /// Create a new snapshot
    ///
    /// # Arguments
    /// * `options` - Snapshot creation options
    ///
    /// # Returns
    /// * `SnapshotMetadata` - Created snapshot metadata
    ///
    async fn create_snapshot(&self, options: SnapshotCreateOptions) -> ServiceResult<SnapshotMetadata>;
    
    /// Restore a snapshot
    ///
    /// # Arguments
    /// * `snapshot_id` - Snapshot to restore
    ///
    async fn restore_snapshot(&self, snapshot_id: &str) -> ServiceResult<()>;
    
    /// List all available snapshots
    ///
    /// # Returns
    /// * `Vec<SnapshotMetadata>` - List of snapshot metadata
    ///
    async fn list_snapshots(&self) -> ServiceResult<Vec<SnapshotMetadata>>;
    
    /// Delete a snapshot
    ///
    /// # Arguments
    /// * `snapshot_id` - Snapshot to delete
    ///
    async fn delete_snapshot(&self, snapshot_id: &str) -> ServiceResult<()>;
    
    /// Get snapshot artifact data
    ///
    /// # Arguments
    /// * `snapshot_id` - Snapshot ID
    /// * `format` - Format (json, binary, etc.)
    ///
    /// # Returns
    /// * Raw snapshot data
    ///
    async fn get_snapshot_artifact(&self, snapshot_id: &str, format: &str) -> ServiceResult<Vec<u8>>;
}

