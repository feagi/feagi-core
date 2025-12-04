// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Standard runtime implementation for desktop/server platforms

use feagi_runtime::{Runtime, Result};
use crate::{NeuronArray, SynapseArray};
use feagi_neural::types::NeuralValue;

/// Standard runtime for desktop/server (Vec-based, dynamic allocation)
#[derive(Debug, Clone, Copy)]
pub struct StdRuntime;

impl StdRuntime {
    /// Create a new standard runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime for StdRuntime {
    type NeuronStorage<T: NeuralValue> = NeuronArray<T>;
    type SynapseStorage = SynapseArray;
    
    fn create_neuron_storage<T: NeuralValue>(&self, capacity: usize) -> Result<Self::NeuronStorage<T>> {
        Ok(NeuronArray::new(capacity))
    }
    
    fn create_synapse_storage(&self, capacity: usize) -> Result<Self::SynapseStorage> {
        Ok(SynapseArray::new(capacity))
    }
    
    fn supports_parallel(&self) -> bool {
        true  // Rayon for parallel processing
    }
    
    fn supports_simd(&self) -> bool {
        true  // x86_64 SIMD
    }
    
    fn memory_limit(&self) -> Option<usize> {
        None  // Unlimited (system RAM)
    }
    
    fn platform_name(&self) -> &'static str {
        "Standard (Desktop/Server)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_std_runtime_creation() {
        let runtime = StdRuntime::new();
        assert_eq!(runtime.platform_name(), "Standard (Desktop/Server)");
        assert!(runtime.supports_parallel());
        assert!(runtime.memory_limit().is_none());
    }
    
    #[test]
    fn test_create_neuron_storage_f32() {
        let runtime = StdRuntime::new();
        let storage = runtime.create_neuron_storage::<f32>(1000).unwrap();
        assert_eq!(storage.count, 0);
    }
}

