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

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{format, string::String};
#[cfg(feature = "std")]
use std::{format, string::String};

/// Quantization precision mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "lowercase"))]
pub enum Precision {
    #[default]
    FP32,
    FP16,
    INT8,
}

impl std::str::FromStr for Precision {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fp32" | "float32" | "f32" => Ok(Self::FP32),
            "fp16" | "float16" | "f16" => Ok(Self::FP16),
            "int8" | "i8" => Ok(Self::INT8),
            _ => Err(format!("Unknown precision: {}", s)),
        }
    }
}

impl Precision {
    /// Convert to canonical string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FP32 => "fp32",
            Self::FP16 => "fp16",
            Self::INT8 => "int8",
        }
    }
}

/// Trait for neural computation values
pub trait NeuralValue: Copy + Clone + Send + Sync + fmt::Debug + 'static {
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
    fn saturating_add(self, other: Self) -> Self;
    /// Add an unquantized synaptic current and return only the stored level.
    ///
    /// The fractional part is discarded. Call [`NeuralValue::integrate_charge`]
    /// when the leftover must survive until the next burst.
    fn add_f32(self, delta: f32) -> Self;
    /// Split a linear charge into a stored level and a leftover fraction.
    ///
    /// `f32` keeps the whole value in the level and returns a zero fraction.
    /// `INT8Value` stores the integer byte and a fraction in `[0, 1)`.
    fn split_charge(value: f32) -> (Self, f32);
    /// Add `delta` to `self + fraction`, then split the sum again.
    fn integrate_charge(self, fraction: f32, delta: f32) -> (Self, f32) {
        let base = self.to_f32() + fraction;
        let sum = if delta.is_finite() {
            base + delta
        } else {
            base
        };
        Self::split_charge(sum)
    }
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
    fn from_f32(value: f32) -> Self {
        value
    }

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self
    }

    #[inline(always)]
    fn saturating_add(self, other: Self) -> Self {
        self + other
    }

    #[inline(always)]
    fn add_f32(self, delta: f32) -> Self {
        if !delta.is_finite() || delta == 0.0 {
            return self;
        }
        self + delta
    }

    fn split_charge(value: f32) -> (Self, f32) {
        (value, 0.0)
    }

    #[inline(always)]
    fn mul_leak(self, leak_coefficient: f32) -> Self {
        self * (1.0 - leak_coefficient)
    }

    #[inline(always)]
    fn ge(self, other: Self) -> bool {
        self >= other
    }

    #[inline(always)]
    fn lt(self, other: Self) -> bool {
        self < other
    }

    #[inline(always)]
    fn zero() -> Self {
        0.0
    }

    #[inline(always)]
    fn one() -> Self {
        1.0
    }

    #[inline(always)]
    fn max_value() -> Self {
        f32::MAX
    }

    #[inline(always)]
    fn min_value() -> Self {
        f32::MIN
    }
}

/// INT8 quantized value for neural computations.
///
/// The stored byte is an absolute level on **0..=255**, the same scale as a
/// sensor byte (vision channel intensity, PSP, weight). It is not a biological
/// millivolt. Raw `i8` -128 is level 0 and raw `i8` 127 is level 255, so every
/// integer in range round-trips. Values outside 0..=255 saturate to the ends.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct INT8Value(pub i8);

impl INT8Value {
    pub const MEMBRANE_MIN: f32 = 0.0;
    pub const MEMBRANE_MAX: f32 = 255.0;
    pub const MEMBRANE_RANGE: f32 = Self::MEMBRANE_MAX - Self::MEMBRANE_MIN;
    /// `decoded = raw + RAW_OFFSET`, with raw i8 -128..=127 covering 0..=255.
    pub const RAW_OFFSET: i16 = 128;
    pub const RESOLUTION: f32 = 1.0;

    #[inline]
    fn from_level(level: i16) -> Self {
        let level = level.clamp(0, Self::MEMBRANE_MAX as i16);
        Self((level - Self::RAW_OFFSET) as i8)
    }

    /// Map a linear value onto a byte with round-half-up.
    ///
    /// Exact integers in `0..=255` round-trip. A fractional value such as
    /// `0.38` rounds to `0` here; [`NeuralValue::split_charge`] keeps that
    /// fraction beside the byte so it still participates in firing.
    #[inline]
    fn level_from_f32(value: f32) -> i16 {
        if !value.is_finite() || value <= 0.0 {
            return 0;
        }
        if value >= Self::MEMBRANE_MAX {
            return Self::MEMBRANE_MAX as i16;
        }
        ((value + 0.5) as i16).clamp(0, Self::MEMBRANE_MAX as i16)
    }

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
        Self::from_level(Self::level_from_f32(value))
    }

    #[inline]
    fn to_f32(self) -> f32 {
        f32::from(i16::from(self.0) + Self::RAW_OFFSET)
    }

    #[inline]
    fn saturating_add(self, other: Self) -> Self {
        // Add in decoded 0..=255 space. Raw i8 addition is not addition of levels,
        // because level 0 is stored as -128.
        let sum = i16::from(self.0) + i16::from(other.0) + (Self::RAW_OFFSET * 2);
        let clamped = sum.clamp(0, Self::MEMBRANE_MAX as i16);
        Self((clamped - Self::RAW_OFFSET) as i8)
    }

    #[inline]
    fn add_f32(self, delta: f32) -> Self {
        if !delta.is_finite() || delta == 0.0 {
            return self;
        }
        // No fraction slot: the leftover below one byte is dropped.
        self.integrate_charge(0.0, delta).0
    }

    fn split_charge(value: f32) -> (Self, f32) {
        if !value.is_finite() || value <= 0.0 {
            return (Self::zero(), 0.0);
        }
        if value >= Self::MEMBRANE_MAX {
            return (Self::from_level(Self::MEMBRANE_MAX as i16), 0.0);
        }
        let level = value.floor() as i16;
        let fraction = value - (level as f32);
        (Self::from_level(level), fraction)
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
        // Level 0, not raw i8 0 (raw 0 is level 128).
        Self::from_f32(0.0)
    }

    #[inline]
    fn one() -> Self {
        Self::from_f32(1.0)
    }

    #[inline]
    fn max_value() -> Self {
        Self(127) // level 255
    }

    #[inline]
    fn min_value() -> Self {
        Self(i8::MIN) // level 0
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
            membrane_potential_min: 0.0,
            membrane_potential_max: 255.0,
            threshold_min: 0.0,
            threshold_max: 255.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{INT8Value, NeuralValue};

    #[test]
    fn byte_levels_round_trip() {
        for level in [0.0, 1.0, 50.0, 128.0, 150.0, 254.0, 255.0] {
            let encoded = INT8Value::from_f32(level);
            assert_eq!(encoded.to_f32(), level, "level {level} did not round-trip");
        }
    }

    #[test]
    fn zero_is_level_zero_not_raw_zero() {
        assert_eq!(INT8Value::zero().to_f32(), 0.0);
        assert_eq!(INT8Value::zero().to_raw(), i8::MIN);
        assert_eq!(INT8Value::from_raw(0).to_f32(), 128.0);
        assert_eq!(INT8Value::max_value().to_f32(), 255.0);
        assert_eq!(INT8Value::min_value().to_f32(), 0.0);
    }

    #[test]
    fn add_onto_zero_preserves_the_byte() {
        let pixel = INT8Value::from_f32(141.0);
        let charged = INT8Value::zero().saturating_add(pixel);
        assert_eq!(charged.to_f32(), 141.0);
    }

    #[test]
    fn add_saturates_at_255() {
        let sum = INT8Value::from_f32(200.0).saturating_add(INT8Value::from_f32(200.0));
        assert_eq!(sum.to_f32(), 255.0);
    }

    #[test]
    fn sub_byte_charge_survives_beside_the_byte() {
        let (level, fraction) = INT8Value::split_charge(0.38);
        assert_eq!(level.to_f32(), 0.0);
        assert!((fraction - 0.38).abs() < 1.0e-6);

        let (level, fraction) = INT8Value::zero().integrate_charge(fraction, 0.38);
        assert_eq!(level.to_f32(), 0.0);
        assert!((fraction - 0.76).abs() < 1.0e-5);

        let (level, fraction) = level.integrate_charge(fraction, 0.38);
        assert_eq!(level.to_f32(), 1.0);
        assert!((fraction - 0.14).abs() < 1.0e-5);
        assert_eq!(INT8Value::zero().add_f32(1.0).to_f32(), 1.0);
    }

    #[test]
    fn inhibition_lowers_the_byte() {
        let membrane = INT8Value::from_f32(100.0);
        assert_eq!(membrane.add_f32(-30.0).to_f32(), 70.0);
        let (level, fraction) = membrane.integrate_charge(0.0, -0.2);
        assert_eq!(level.to_f32(), 99.0);
        assert!((fraction - 0.8).abs() < 1.0e-5);
        assert_eq!(INT8Value::from_f32(1.0).add_f32(-1.0).to_f32(), 0.0);
    }

    #[test]
    fn threshold_inside_the_byte_range_compares_against_the_pixel() {
        let dim = INT8Value::from_f32(100.0);
        let bright = INT8Value::from_f32(200.0);
        let threshold = INT8Value::from_f32(150.0);
        assert!(dim.lt(threshold));
        assert!(bright.ge(threshold));
    }

    #[test]
    fn values_above_255_saturate_to_white() {
        assert_eq!(INT8Value::from_f32(260.0), INT8Value::from_f32(255.0));
        assert!(INT8Value::from_f32(254.0).lt(INT8Value::from_f32(260.0)));
        assert!(INT8Value::from_f32(255.0).ge(INT8Value::from_f32(260.0)));
        assert_eq!(INT8Value::from_f32(-40.0).to_f32(), 0.0);
    }
}
