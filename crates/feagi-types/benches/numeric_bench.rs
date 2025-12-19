// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Benchmarks for numeric quantization performance
//!
//! Verifies that f32 implementation has zero overhead.

use feagi_types::numeric::{NeuralValue, INT8Value};

/// Benchmark f32 neural dynamics (baseline)
#[no_mangle]
pub fn bench_f32_neural_dynamics(iterations: usize) -> f32 {
    let mut potential = 0.0f32;
    let threshold = 50.0f32;
    let leak = 0.97f32;
    let candidate = 1.5f32;
    
    let mut fired_count = 0;
    
    for _ in 0..iterations {
        potential = potential.saturating_add(candidate);
        potential = potential.mul_leak(leak);
        
        if potential.ge(threshold) {
            potential = 0.0;
            fired_count += 1;
        }
    }
    
    fired_count as f32
}

/// Benchmark INT8 neural dynamics (quantized)
#[no_mangle]
pub fn bench_int8_neural_dynamics(iterations: usize) -> f32 {
    let mut potential = INT8Value::from_f32(0.0);
    let threshold = INT8Value::from_f32(50.0);
    let leak = INT8Value::from_f32(0.97);
    let candidate = INT8Value::from_f32(1.5);
    
    let mut fired_count = 0;
    
    for _ in 0..iterations {
        potential = potential.saturating_add(candidate);
        potential = potential.mul_leak(leak);
        
        if potential.ge(threshold) {
            potential = INT8Value::zero();
            fired_count += 1;
        }
    }
    
    fired_count as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_f32_benchmark() {
        let result = bench_f32_neural_dynamics(1000);
        assert!(result > 0.0);
    }
    
    #[test]
    fn test_int8_benchmark() {
        let result = bench_int8_neural_dynamics(1000);
        assert!(result > 0.0);
    }
    
    #[test]
    fn test_f32_vs_int8_behavior() {
        let f32_result = bench_f32_neural_dynamics(1000);
        let int8_result = bench_int8_neural_dynamics(1000);
        
        // INT8 should produce similar results (within 15%)
        let diff_ratio = (f32_result - int8_result).abs() / f32_result;
        assert!(
            diff_ratio < 0.15,
            "INT8 fired {} vs FP32 fired {}, difference: {:.1}%",
            int8_result, f32_result, diff_ratio * 100.0
        );
    }
}


