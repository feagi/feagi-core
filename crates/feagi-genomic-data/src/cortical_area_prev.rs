// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
CorticalArea data structure (genome representation).

Pure data definition - no business logic.
Transformation methods live in feagi-bdu.
Moved from feagi-core/crates/feagi-bdu/src/models/cortical_area.rs
*/

use feagi_data::neurons::neuron_voxels::wrapped_values::{NeuronVoxelDimensionsGenomic};
use feagi_genomic_context::cortical_area::{CorticalAreaType, CorticalID};
use feagi_genomic_context::genome_positioning::GenomeCoordinate3D;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use feagi_data::values::quantizable::QuantizedUnsignedIntegerTrait;

/// Cortical area metadata (genome representation)
///
/// Pure data structure containing static genome metadata.
/// Runtime operations and transformations are implemented in feagi-bdu.

// TODO This seems completely unusued. Disabling this for now, we will rewrite it later
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorticalArea {
    /// Unique typed cortical_area identifier
    pub cortical_id: CorticalID,

    /// Integer index assigned by ConnectomeManager
    pub cortical_idx: u32,

    /// Human-readable name
    pub name: String,

    /// 3D dimensions (width, height, depth in voxels)
    pub dimensions: NeuronVoxelDimensionsGenomic,

    /// 3D position in genome space
    pub position: GenomeCoordinate3D,

    /// Cortical area type (encoding method and functional classification)
    pub cortical_type: CorticalAreaType,

    /// Additional user-defined properties
    /// Note: See PROPERTIES_STRUCT_MIGRATION_PROPOSAL.md for future struct-based design
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl CorticalArea {
    /// Create a new cortical_area area with validation
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Unique typed cortical_area identifier
    /// * `cortical_idx` - Integer index for fast lookups
    /// * `name` - Human-readable name
    /// * `dimensions` - 3D dimensions (width, height, depth)
    /// * `position` - 3D position in genome space
    /// * `cortical_type` - Cortical area type (encoding method)
    ///
    /// # Errors
    ///
    /// Returns error if name is empty
    ///
    pub fn new(
        cortical_id: CorticalID,
        cortical_idx: u32,
        name: String,
        dimensions: NeuronVoxelDimensionsGenomic,
        position: GenomeCoordinate3D,
        cortical_type: CorticalAreaType,
    ) -> Result<Self, ()> {
        // Validate name
        if name.trim().is_empty() {
            panic!("cortical_area area name cannot be empty")
        }

        // Note: CorticalID validation happens in CorticalID constructors
        // Note: dimensions validation happens in CorticalAreaDimensions::new()

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

    /// Get a property value by key
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    /// Get the total number of voxels in this area
    pub fn total_voxels(&self) -> u32 {
        self.dimensions.number_contained_elements().quant_to_u32_unchecked()
    }
}
