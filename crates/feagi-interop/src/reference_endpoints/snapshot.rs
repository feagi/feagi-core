//! Snapshot: creating, listing, comparing and restoring brain snapshots.
//!
//! Converted from the old `snapshot` endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;

/// Information about one stored snapshot
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SnapshotInfo {
    pub snapshot_id: String,
    pub name: String,
    pub created_at: String,
    pub description: Option<String>,
}

define_request_message_category! {
    category_name: "Snapshot",
    base_path: "v1",
    category_description: "Creating, listing, comparing and restoring brain snapshots",
    read: {
        "List": {
            path: "snapshot",
            description: "List all stored snapshots",
            alloc_only: true,
            response: {
                snapshots: Vec<SnapshotInfo>,
            },
        },
        "Artifact": {
            path: "snapshot/:snapshot_id<String>/artifact/:fmt<String>",
            path_item_descriptions: {
                snapshot_id: "ID of the snapshot",
                fmt: "Artifact format",
            },
            description: "Get a snapshot artifact in the given format",
            alloc_only: true,
            response: {
                artifact: HashMap<String, serde_json::Value>,
            },
        },
    },
    create: {
        "Create": {
            path: "snapshot/create",
            description: "Create a snapshot of the current brain state",
            alloc_only: true,
            payload: {
                name: Option<String>,
                description: Option<String>,
                metadata: Option<HashMap<String, serde_json::Value>>,
            },
            response: {
                snapshot_id: String,
                success: bool,
                message: String,
            },
        },
        "Restore": {
            path: "snapshot/restore",
            description: "Restore the brain state from a snapshot",
            alloc_only: true,
            payload: {
                snapshot_id: String,
            },
            response: {
                message: String,
                success: bool,
            },
        },
        "Compare": {
            path: "snapshot/compare",
            description: "Compare two snapshots",
            alloc_only: true,
            payload: {
                snapshot_id_1: String,
                snapshot_id_2: String,
            },
            response: {
                diff: HashMap<String, serde_json::Value>,
            },
        },
        "Upload": {
            path: "snapshot/upload",
            description: "Upload a snapshot",
            alloc_only: true,
            payload: {
                data: HashMap<String, serde_json::Value>, "Snapshot contents",
            },
            response: {
                snapshot_id: String,
                success: bool,
            },
        },
    },
    delete: {
        "Snapshot": {
            path: "snapshot/:snapshot_id<String>",
            path_item_descriptions: {
                snapshot_id: "ID of the snapshot",
            },
            description: "Delete a snapshot",
            alloc_only: true,
            response: {
                message: String,
                success: bool,
            },
        },
    },
}

// NOTES:
// - The old snapshot endpoint file had no handlers; paths come from the commented-out router and types from snapshot_dtos.rs.
// - List was at the category root (/v1/snapshot/), so base_path is "v1" and every path starts with "snapshot/".
// - Restore / Delete responses assumed to be SnapshotSuccessResponse (message, success).
// - Artifact `artifact`, Compare `diff`, Upload `data` and Create `metadata` are untyped maps in the old DTOs.
// - Skipped POST /v1/snapshots/connectome and /v1/snapshots/connectome/:snapshot_id/restore (commented out, no types).
