// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Identity types for neurons and synapses

use core::fmt;

/// Neuron ID (globally unique across the entire brain)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NeuronId(pub u32);

impl fmt::Display for NeuronId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Neuron({})", self.0)
    }
}

/// Synapse ID (unique identifier for a synaptic connection)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SynapseId(pub u32);

impl fmt::Display for SynapseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Synapse({})", self.0)
    }
}

// Note: CorticalAreaId REMOVED - use feagi_data_structures::genomic::cortical_area::CorticalID instead
