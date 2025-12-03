// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Numeric type abstractions for quantization support
//!
//! Moved from feagi-types/src/numeric.rs (Phase 2c)

// Re-export the full numeric.rs from feagi-types
// This file will be copied in full from feagi-types/src/numeric.rs

// TODO: Copy full content from feagi-types/src/numeric.rs
// For now, just define the core trait signature

use core::fmt;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Quantization precision mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "lowercase"))]
pub enum Precision {
    FP32,
    FP16,
    INT8,
}

impl Default for Precision {
    fn default() -> Self {
        Self::FP32
    }
}

/// Trait for neural computation values
pub trait NeuralValue: Copy + Clone + Send + Sync + fmt::Debug + 'static {
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
    fn saturating_add(self, other: Self) -> Self;
    fn mul_leak(self, leak_coefficient: f32) -> Self;
    fn ge(self, other: Self) -> bool;
    fn lt(self, other: Self) -> bool;
    fn zero() -> Self;
    fn one() -> Self;
    fn max_value() -> Self;
    fn min_value() -> Self;
}

impl NeuralValue for f32 {
    #[inline(always)]
    fn from_f32(value: f32) -> Self { value }
    
    #[inline(always)]
    fn to_f32(self) -> f32 { self }
    
    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self { self + other }
    
    #[inline(always)]
    fn mul_leak(self, leak_coefficient: f32) -> Self {
        self * (1.0 - leak_coefficient)
    }
    
    #[inline(always)]
    fn ge(self, other: Self) -> bool { self >= other }
    
    #[inline(always)]
    fn lt(self, other: Self) -> bool { self < other }
    
    #[inline(always)]
    fn zero() -> Self { 0.0 }
    
    #[inline(always)]
    fn one() -> Self { 1.0 }
    
    #[inline(always)]
    fn max_value() -> Self { f32::MAX }
    
    #[inline(always)]
    fn min_value() -> Self { f32::MIN }
}

/// INT8 quantized value for neural computations
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct INT8Value(pub i8);

impl INT8Value {
    pub const MEMBRANE_MIN: f32 = -100.0;
    pub const MEMBRANE_MAX: f32 = 50.0;
    pub const MEMBRANE_RANGE: f32 = Self::MEMBRANE_MAX - Self::MEMBRANE_MIN;
    pub const SCALE: f32 = 254.0;
    pub const RESOLUTION: f32 = Self::MEMBRANE_RANGE / Self::SCALE;
    
    #[inline]
    pub const fn from_raw(value: i8) -> Self {
        Self(value)
    }
    
    #[inline]
    pub const fn to_raw(self) -> i8 {
        self.0
    }
}

impl NeuralValue for INT8Value {
    #[inline]
    fn from_f32(value: f32) -> Self {
        let normalized = (value - Self::MEMBRANE_MIN) / Self::MEMBRANE_RANGE;
        let scaled = (normalized * Self::SCALE) - 127.0;
        // Use manual rounding (no_std compatible)
        let quantized = if scaled >= 0.0 {
            (scaled + 0.5).min(127.0) as i8
        } else {
            (scaled - 0.5).max(-127.0) as i8
        };
        Self(quantized)
    }
    
    #[inline]
    fn to_f32(self) -> f32 {
        let normalized = (self.0 as f32 + 127.0) / Self::SCALE;
        normalized * Self::MEMBRANE_RANGE + Self::MEMBRANE_MIN
    }
    
    #[inline]
    fn saturating_add(self, other: Self) -> Self {
        let result = self.0.saturating_add(other.0);
        Self(result.max(-127))
    }
    
    #[inline]
    fn mul_leak(self, leak_coefficient: f32) -> Self {
        let potential_f32 = self.to_f32();
        let retention = 1.0 - leak_coefficient;
        Self::from_f32(potential_f32 * retention)
    }
    
    #[inline]
    fn ge(self, other: Self) -> bool {
        self.0 >= other.0
    }
    
    #[inline]
    fn lt(self, other: Self) -> bool {
        self.0 < other.0
    }
    
    #[inline]
    fn zero() -> Self {
        Self(0)
    }
    
    #[inline]
    fn one() -> Self {
        Self::from_f32(1.0)
    }
    
    #[inline]
    fn max_value() -> Self {
        Self(127)
    }
    
    #[inline]
    fn min_value() -> Self {
        Self(-127)
    }
}

/// Leak coefficient for INT8 computations
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct INT8LeakCoefficient(pub i16);

impl INT8LeakCoefficient {
    pub const SCALE: i32 = 10000;
    
    #[inline]
    pub fn from_f32(value: f32) -> Self {
        // Use manual rounding (no_std compatible)
        let scaled_f = value * Self::SCALE as f32;
        let scaled = if scaled_f >= 0.0 {
            (scaled_f + 0.5) as i16
        } else {
            (scaled_f - 0.5) as i16
        };
        Self(scaled.clamp(0, Self::SCALE as i16))
    }
    
    #[inline]
    pub fn to_f32(self) -> f32 {
        self.0 as f32 / Self::SCALE as f32
    }
}

/// Quantization specification from genome
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct QuantizationSpec {
    pub precision: Precision,
    pub membrane_potential_min: f32,
    pub membrane_potential_max: f32,
    pub threshold_min: f32,
    pub threshold_max: f32,
}

impl Default for QuantizationSpec {
    fn default() -> Self {
        Self {
            precision: Precision::INT8,
            membrane_potential_min: -100.0,
            membrane_potential_max: 50.0,
            threshold_min: 0.0,
            threshold_max: 100.0,
        }
    }
}

