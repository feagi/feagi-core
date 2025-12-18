// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Embedded runtime implementation for ESP32, RTOS, no_std platforms

use crate::{NeuronArray, SynapseArray};
use feagi_npu_neural::types::NeuralValue;
use feagi_npu_runtime::{NeuronStorage, Result, Runtime, RuntimeError, SynapseStorage};

/// Embedded runtime for ESP32, Arduino, STM32 (fixed-size, no_std)
#[derive(Debug, Clone, Copy)]
pub struct EmbeddedRuntime;

impl EmbeddedRuntime {
    /// Create a new embedded runtime
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmbeddedRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime for EmbeddedRuntime {
    type NeuronStorage<T: NeuralValue> = NeuronArray<T, 10000>; // Default const
    type SynapseStorage = SynapseArray<50000>; // Default const

    fn create_neuron_storage<T: NeuralValue>(
        &self,
        capacity: usize,
    ) -> Result<Self::NeuronStorage<T>> {
        // Embedded uses const generics, capacity is compile-time
        // For now, use default const (will be improved with const generics in Phase 3)
        if capacity > 10000 {
            return Err(RuntimeError::CapacityExceeded {
                requested: capacity,
                available: 10000,
            });
        }
        Ok(NeuronArray::new())
    }

    fn create_synapse_storage(&self, capacity: usize) -> Result<Self::SynapseStorage> {
        if capacity > 50000 {
            return Err(RuntimeError::CapacityExceeded {
                requested: capacity,
                available: 50000,
            });
        }
        Ok(SynapseArray::new())
    }

    fn supports_parallel(&self) -> bool {
        false // Single-threaded (no OS or basic RTOS)
    }

    fn supports_simd(&self) -> bool {
        false // Most embedded targets don't have SIMD
    }

    fn memory_limit(&self) -> Option<usize> {
        Some(512 * 1024) // 512 KB typical (ESP32-S3 has more)
    }

    fn platform_name(&self) -> &'static str {
        "Embedded (ESP32/RTOS/no_std)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_runtime_creation() {
        let runtime = EmbeddedRuntime::new();
        assert_eq!(runtime.platform_name(), "Embedded (ESP32/RTOS/no_std)");
        assert!(!runtime.supports_parallel());
        assert_eq!(runtime.memory_limit(), Some(512 * 1024));
    }

    #[test]
    fn test_create_neuron_storage_within_limit() {
        let runtime = EmbeddedRuntime::new();
        let storage = runtime.create_neuron_storage::<f32>(1000).unwrap();
        assert_eq!(storage.count, 0);
    }

    #[test]
    fn test_create_neuron_storage_exceeds_limit() {
        let runtime = EmbeddedRuntime::new();
        let result = runtime.create_neuron_storage::<f32>(20000);
        assert!(result.is_err());
    }
}
