// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Genome-layer (editor) types for feagi-evolutionary.
//!
//! These types own the alloc/serde-heavy metadata (name, properties, JSON values)
//! that is deliberately kept out of `feagi-structures` so that the structures crate
//! remains quantization-generic and suitable for no_std / embedded / WASM targets.
//!
//! The `CorticalArea` struct defined here is the evolutionary/genome representation
//! of a cortical area and is only meaningful in genome-editing contexts. Runtime
//! code (burst engine, NPU, bridges) should consume the lean primitives from
//! `feagi-structures` directly (`CorticalID`, `CorticalAreaType`,
//! `NeuronVoxelDimensions<_>`, `CorticalAreaIndex<_>`) rather than this struct.

use feagi_structures::genomic::cortical_area::{CorticalAreaType, CorticalID};
use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
use feagi_structures::FeagiStructuresError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Re-export the quant-generic publicly-exposed genome coordinate (signed i32 axes).
pub use feagi_structures::genomic::GenomeCoordinate3DI32 as GenomeCoordinate3D;

/// Re-export brain region / region type from feagi-structures for genome code.
pub use feagi_structures::genomic::brain_regions::{BrainRegion, RegionID, RegionType};

/// Cortical-area 3D dimensions (in voxels) used by the genome editor.
///
/// This is a *genome-layer* type that intentionally permits zero values so that
/// genome loading / migration / validation can observe and repair malformed
/// genomes. Runtime code must convert to
/// `feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions<u32>`
/// (non-zero enforced) via [`CorticalAreaDimensions::to_runtime`] before
/// crossing into the lean no_std-friendly structures API.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct CorticalAreaDimensions {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl CorticalAreaDimensions {
    /// Construct genome-layer dimensions. Does not reject zero.
    #[inline]
    pub fn new(width: u32, height: u32, depth: u32) -> Result<Self, FeagiStructuresError> {
        Ok(Self {
            width,
            height,
            depth,
        })
    }

    /// Total voxel count. Uses saturating multiplication to avoid u32 overflow
    /// while still surfacing suspicious sizes to genome validators.
    #[inline]
    pub fn total_voxels(&self) -> u32 {
        self.width
            .saturating_mul(self.height)
            .saturating_mul(self.depth)
    }

    /// Convert to the runtime (non-zero enforced) `NeuronVoxelDimensions<u32>`.
    #[inline]
    pub fn to_runtime(&self) -> Result<NeuronVoxelDimensions<u32>, FeagiStructuresError> {
        NeuronVoxelDimensions::<u32>::new(self.width, self.height, self.depth)
    }
}

impl From<NeuronVoxelDimensions<u32>> for CorticalAreaDimensions {
    #[inline]
    fn from(value: NeuronVoxelDimensions<u32>) -> Self {
        Self {
            width: value.x.get(),
            height: value.y.get(),
            depth: value.z.get(),
        }
    }
}

/// Genome-layer (editor) view of a cortical area.
///
/// Owns editor metadata (`name`, `properties`) that does not belong in a
/// no_std-friendly runtime type. Pure data container; all transformation logic
/// lives in the genome parser / validator / converter modules of this crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorticalArea {
    /// Unique typed cortical identifier.
    pub cortical_id: CorticalID,

    /// Integer index assigned by ConnectomeManager.
    pub cortical_idx: u32,

    /// Human-readable name.
    pub name: String,

    /// 3D dimensions (width, height, depth) in voxels.
    pub dimensions: CorticalAreaDimensions,

    /// 3D position in genome space.
    pub position: GenomeCoordinate3D,

    /// Cortical area type (encoding method / functional classification).
    pub cortical_type: CorticalAreaType,

    /// Additional user-defined editor properties (JSON-typed).
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl CorticalArea {
    /// Create a new cortical area with minimal validation (non-empty name).
    pub fn new(
        cortical_id: CorticalID,
        cortical_idx: u32,
        name: String,
        dimensions: CorticalAreaDimensions,
        position: GenomeCoordinate3D,
        cortical_type: CorticalAreaType,
    ) -> Result<Self, FeagiStructuresError> {
        if name.trim().is_empty() {
            return Err(FeagiStructuresError::BadParameters(
                "cortical area name cannot be empty".into(),
            ));
        }
        Ok(Self {
            cortical_id,
            cortical_idx,
            name,
            dimensions,
            position,
            cortical_type,
            properties: HashMap::new(),
        })
    }

    /// Get a property value by key.
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    /// Total number of voxels (product of axes). Saturates at `u32::MAX` on
    /// overflow; callers expecting wider ranges should use
    /// `dimensions.total_voxels()` directly.
    pub fn total_voxels(&self) -> u32 {
        self.dimensions.total_voxels()
    }
}
