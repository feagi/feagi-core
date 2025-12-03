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

pub mod ids;
pub mod numeric;
pub mod synapse_types;
pub mod spatial;
pub mod error;
pub mod fire;
pub mod brain;
pub mod id_manager;

// Re-export commonly used types
pub use ids::{NeuronId, SynapseId};
pub use numeric::{NeuralValue, INT8Value, INT8LeakCoefficient, Precision, QuantizationSpec};
pub use synapse_types::{Synapse, SynapticWeight, SynapticConductance, SynapticContribution};
pub use spatial::{Dimensions, Position};
pub use error::{FeagiError, Result, Error};
pub use fire::{FireCandidateList, FireQueue, FireLedger};
pub use brain::{CorticalArea, BrainRegion, BrainRegionHierarchy, RegionType};
pub use id_manager::NeuronArrayType;

// Note: SynapseType is in crate::synapse module (shared with algorithms)
// Import it here for convenience
pub use crate::synapse::SynapseType;

