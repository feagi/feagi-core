// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Brain Region DTOs for V1 API
//
// These DTOs must match Python FastAPI response structures exactly.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Brain region information (summary)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "region_id": "visual_cortex",
    "name": "Visual Cortex",
    "region_type": "Sensory",
    "parent_id": "neocortex",
    "cortical_areas": ["v1", "v2", "v4"],
    "child_count": 3
}))]
pub struct BrainRegionSummary {
    /// Brain region ID
    pub region_id: String,

    /// Human-readable name
    pub name: String,

    /// Functional type (Sensory, Motor, Association, Custom)
    pub region_type: String,

    /// Parent region ID (if any)
    pub parent_id: Option<String>,

    /// List of cortical area IDs in this region
    pub cortical_areas: Vec<String>,

    /// Number of child regions
    pub child_count: usize,
}

/// Detailed brain region information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "region_id": "visual_cortex",
    "name": "Visual Cortex",
    "region_type": "Sensory",
    "parent_id": "neocortex",
    "cortical_areas": ["v1", "v2", "v4"],
    "child_regions": ["primary_visual", "secondary_visual"],
    "properties": {}
}))]
pub struct BrainRegionDetail {
    /// Brain region ID
    pub region_id: String,

    /// Human-readable name
    pub name: String,

    /// Functional type
    pub region_type: String,

    /// Parent region ID
    pub parent_id: Option<String>,

    /// Cortical areas in this region
    pub cortical_areas: Vec<String>,

    /// Child region IDs
    pub child_regions: Vec<String>,

    /// Additional properties
    pub properties: serde_json::Value,
}

/// One designated port's encoding, as returned under `port_encoding` by region reads.
///
/// Keyed by the designated area id (base64). `null` means unknown.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "kind": "SignedPercentage",
    "signed": true,
    "frame": "Absolute",
    "positioning": "Fractional",
    "source": "wiring"
}))]
pub struct RegionPortEncodingEntry {
    /// One of Boolean, Percentage, Percentage2D, Percentage3D, Percentage4D, SignedPercentage,
    /// SignedPercentage2D, SignedPercentage3D, SignedPercentage4D, CartesianPlane, Misc,
    /// PoseEstimation.
    pub kind: Option<String>,

    /// Whether the port carries signed values.
    pub signed: Option<bool>,

    /// Absolute or Incremental.
    pub frame: Option<String>,

    /// Linear or Fractional.
    pub positioning: Option<String>,

    /// `wiring`: derived by FEAGI from the IPU/OPU areas wired to the port. `author`: set through
    /// `PUT /v1/region/region`.
    pub source: String,
}

/// An author entry in the `port_encoding` delta of `PUT /v1/region/region`.
///
/// An absent field is `null`. `source` is not accepted: the server stores `author`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "kind": "SignedPercentage",
    "signed": true,
    "frame": "Absolute",
    "positioning": "Fractional"
}))]
pub struct RegionPortEncodingAuthorEntry {
    /// Same vocabulary as [`RegionPortEncodingEntry::kind`].
    pub kind: Option<String>,

    /// Whether the port carries signed values.
    pub signed: Option<bool>,

    /// Absolute or Incremental.
    pub frame: Option<String>,

    /// Linear or Fractional.
    pub positioning: Option<String>,
}

/// Create brain region request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateBrainRegionRequest {
    /// Brain region ID
    pub region_id: String,

    /// Human-readable name
    pub name: String,

    /// Functional type
    pub region_type: String,

    /// Optional: Parent region ID
    #[serde(default)]
    pub parent_id: Option<String>,
}

/// List brain regions response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BrainRegionListResponse {
    /// List of brain regions
    pub brain_regions: Vec<BrainRegionSummary>,

    /// Total count
    pub total_count: usize,
}
