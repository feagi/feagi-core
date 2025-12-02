// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # Neuron Array Type Definitions
//!
//! Defines types for neuron model classification and routing.
//!
//! ## Current Status
//! - Phase 0: Only LIF model supported
//! - Future: Izhikevich, AdEx, Hodgkin-Huxley, etc.

/// Neuron Array Type Classification
///
/// Used for routing global neuron IDs to model-specific arrays.
///
/// **Current**: Only `LIF` is active
/// **Future**: Full multi-model support
///
/// **Note**: Serialization support will be added when multi-model is implemented
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum NeuronArrayType {
    /// Leaky Integrate-and-Fire model (current default)
    LIF = 0,
    
    /// Izhikevich spiking neuron model (future)
    #[allow(dead_code)]
    Izhikevich = 1,
    
    /// Adaptive Exponential Integrate-and-Fire model (future)
    #[allow(dead_code)]
    AdEx = 2,
    
    /// Hodgkin-Huxley model (future)
    #[allow(dead_code)]
    HodgkinHuxley = 3,
    
    /// Morris-Lecar model (future)
    #[allow(dead_code)]
    MorrisLecar = 4,
    
    /// Memory neurons (existing, separate system)
    Memory = 254,
    
    /// Invalid/unallocated ID
    Invalid = 255,
}

impl NeuronArrayType {
    /// Get human-readable model name
    pub fn name(&self) -> &'static str {
        match self {
            NeuronArrayType::LIF => "LIF",
            NeuronArrayType::Izhikevich => "Izhikevich",
            NeuronArrayType::AdEx => "AdEx",
            NeuronArrayType::HodgkinHuxley => "Hodgkin-Huxley",
            NeuronArrayType::MorrisLecar => "Morris-Lecar",
            NeuronArrayType::Memory => "Memory",
            NeuronArrayType::Invalid => "Invalid",
        }
    }

    /// Check if model type is currently implemented
    pub fn is_implemented(&self) -> bool {
        matches!(self, NeuronArrayType::LIF | NeuronArrayType::Memory)
    }
}

impl Default for NeuronArrayType {
    fn default() -> Self {
        NeuronArrayType::LIF
    }
}

impl std::fmt::Display for NeuronArrayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_array_type_names() {
        assert_eq!(NeuronArrayType::LIF.name(), "LIF");
        assert_eq!(NeuronArrayType::Izhikevich.name(), "Izhikevich");
        assert_eq!(NeuronArrayType::Memory.name(), "Memory");
    }

    #[test]
    fn test_is_implemented() {
        assert!(NeuronArrayType::LIF.is_implemented());
        assert!(NeuronArrayType::Memory.is_implemented());
        assert!(!NeuronArrayType::Izhikevich.is_implemented());
        assert!(!NeuronArrayType::AdEx.is_implemented());
    }

    #[test]
    fn test_default() {
        assert_eq!(NeuronArrayType::default(), NeuronArrayType::LIF);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", NeuronArrayType::LIF), "LIF");
        assert_eq!(format!("{}", NeuronArrayType::Izhikevich), "Izhikevich");
    }
}

