// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Synapse type definitions

use core::fmt;

/// Synaptic weight (0-255, stored as u8 for memory efficiency)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SynapticWeight(pub u8);

impl SynapticWeight {
    #[inline(always)]
    pub fn to_float(self) -> f32 {
        self.0 as f32
    }

    #[inline(always)]
    pub fn from_float(value: f32) -> Self {
        Self(value as u8)
    }
}

/// Synaptic conductance (0-255, stored as u8)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SynapticConductance(pub u8);

impl SynapticConductance {
    #[inline(always)]
    pub fn to_float(self) -> f32 {
        self.0 as f32
    }

    #[inline(always)]
    pub fn from_float(value: f32) -> Self {
        Self(value as u8)
    }
}

/// Synaptic contribution (weight × conductance × sign)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SynapticContribution(pub f32);

/// A single synapse (compact representation)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Synapse {
    pub source_neuron: super::NeuronId,
    pub target_neuron: super::NeuronId,
    pub weight: SynapticWeight,
    pub conductance: SynapticConductance,
    pub synapse_type: crate::synapse::SynapseType,
    pub valid: bool,
}

impl Synapse {
    #[inline(always)]
    pub fn calculate_contribution(&self) -> SynapticContribution {
        if !self.valid {
            return SynapticContribution(0.0);
        }
        let weight = self.weight.to_float();
        let conductance = self.conductance.to_float();
        let sign = match self.synapse_type {
            crate::synapse::SynapseType::Excitatory => 1.0,
            crate::synapse::SynapseType::Inhibitory => -1.0,
        };
        SynapticContribution(weight * conductance * sign)
    }
}

/// Membrane potential (in arbitrary units)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MembranePotential(pub f32);

/// Neuron firing threshold
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FiringThreshold(pub f32);

