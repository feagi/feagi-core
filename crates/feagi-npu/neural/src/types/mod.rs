// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # Neural Types Module
//!
//! Core type definitions for FEAGI neural processing (merged from feagi-types).

pub mod brain;
pub mod error;
pub mod fire;
pub mod id_manager;
pub mod ids;
pub mod numeric;
pub mod spatial;
pub mod synapse_types;

// Re-export commonly used types
pub use ids::{NeuronId, SynapseId};
pub use numeric::{INT8LeakCoefficient, INT8Value, NeuralValue, Precision, QuantizationSpec};
pub use spatial::Position;
pub use synapse_types::{Synapse, SynapticConductance, SynapticContribution, SynapticWeight};
// Dimensions moved to feagi_data_structures::genomic::cortical_area::CorticalAreaDimensions
pub use error::{Error, FeagiError, Result};
pub use fire::{FireCandidateList, FireLedger, FireQueue};
// CorticalArea, BrainRegion, RegionType, BrainRegionHierarchy moved to feagi_data_structures
pub use id_manager::NeuronArrayType;

// Note: SynapseType is in crate::synapse module (shared with algorithms)
// Import it here for convenience
pub use crate::synapse::SynapseType;
