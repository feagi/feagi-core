// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Built-in modulator kinds and the burst signal model.
//!
//! Modulator *types* are fixed by FEAGI. Users create instances of these types.
//! The signal formulas here are the single definition used by the genome
//! validator and the burst engine.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Built-in modulator type. The set is not user-editable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModulatorKind {
    /// Neuron firing threshold. `threshold * (1 + signal)`.
    FiringThreshold,
    /// Neuron leak coefficient, clamped to `[0, 1]`.
    Leak,
    /// Probabilistic firing excitability, clamped to `[0, 1]`.
    FiringProbability,
    /// PSP of the subscribed mapping's synapses. `psp * (1 + signal)`.
    TransmissionGain,
    /// Adds `signal` into the mapping's R-STDP reward `R(t)`.
    Reward,
    /// STDP / R-STDP eta of the mapping. `eta * (1 + signal)`.
    LearningRate,
}

impl ModulatorKind {
    /// Stable genome and API identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FiringThreshold => "neuro.firing_threshold",
            Self::Leak => "neuro.leak",
            Self::FiringProbability => "neuro.firing_probability",
            Self::TransmissionGain => "synaptic.transmission_gain",
            Self::Reward => "synaptic.reward",
            Self::LearningRate => "synaptic.learning_rate",
        }
    }

    /// Parse a genome or API type string.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "neuro.firing_threshold" => Ok(Self::FiringThreshold),
            "neuro.leak" => Ok(Self::Leak),
            "neuro.firing_probability" => Ok(Self::FiringProbability),
            "synaptic.transmission_gain" => Ok(Self::TransmissionGain),
            "synaptic.reward" => Ok(Self::Reward),
            "synaptic.learning_rate" => Ok(Self::LearningRate),
            other => Err(format!("unknown modulator type '{other}'")),
        }
    }

    /// Every built-in type, in stable order.
    pub fn all() -> &'static [Self] {
        &[
            Self::FiringThreshold,
            Self::Leak,
            Self::FiringProbability,
            Self::TransmissionGain,
            Self::Reward,
            Self::LearningRate,
        ]
    }

    /// Neuromodulators subscribe on cortical areas.
    pub fn is_neuromodulator(self) -> bool {
        matches!(
            self,
            Self::FiringThreshold | Self::Leak | Self::FiringProbability
        )
    }

    /// Synaptic modulators subscribe on mapping rules.
    pub fn is_synaptic(self) -> bool {
        !self.is_neuromodulator()
    }

    /// Reward combines by addition. Every other kind combines by multiplication.
    pub fn combines_by_addition(self) -> bool {
        matches!(self, Self::Reward)
    }
}

impl fmt::Display for ModulatorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why an instance definition was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModulatorValidationError(pub String);

impl fmt::Display for ModulatorValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Check instance fields that do not depend on the rest of the genome.
///
/// `magnitude_percent` must be finite and `>= -100` so every multiplicative
/// factor stays non-negative. `effect_duration_bursts` must be at least 1.
/// Graded instances require `full_scale_potential > 0`.
pub fn validate_instance_fields(
    magnitude_percent: f32,
    effect_duration_bursts: u16,
    graded: bool,
    full_scale_potential: Option<f32>,
) -> Result<(), ModulatorValidationError> {
    if !magnitude_percent.is_finite() || magnitude_percent < -100.0 {
        return Err(ModulatorValidationError(
            "magnitude_percent must be finite and >= -100".to_string(),
        ));
    }
    if effect_duration_bursts < 1 {
        return Err(ModulatorValidationError(
            "effect_duration_bursts must be >= 1".to_string(),
        ));
    }
    if graded {
        match full_scale_potential {
            Some(scale) if scale.is_finite() && scale > 0.0 => {}
            _ => {
                return Err(ModulatorValidationError(
                    "graded modulators require full_scale_potential > 0".to_string(),
                ));
            }
        }
    }
    Ok(())
}

/// `spike_train` with `consecutive_fire_limit == 0` would never end.
pub fn validate_spike_train(
    enabled: bool,
    consecutive_fire_limit: u16,
) -> Result<(), ModulatorValidationError> {
    if enabled && consecutive_fire_limit < 1 {
        return Err(ModulatorValidationError(
            "spike_train requires consecutive_fire_limit >= 1".to_string(),
        ));
    }
    Ok(())
}

/// Signal of one instance at one burst.
///
/// `fired` includes spike-train firings. When `graded` is false, a firing has
/// strength 1. When graded, strength is `min(membrane_potential / full_scale, 1)`.
/// A silent driver contributes 0.
pub fn modulator_signal(
    magnitude_percent: f32,
    fired: bool,
    graded: bool,
    membrane_potential: f32,
    full_scale_potential: f32,
) -> f32 {
    if !fired {
        return 0.0;
    }
    let strength = if graded {
        (membrane_potential / full_scale_potential).min(1.0)
    } else {
        1.0
    };
    (magnitude_percent / 100.0) * strength
}

/// Product of `(1 + signal)` across instances that share a parameter.
pub fn multiplicative_factor(signals: &[f32]) -> f32 {
    signals
        .iter()
        .fold(1.0_f32, |acc, signal| acc * (1.0 + signal))
}

/// Sum of reward signals. Reward does not use the multiplicative factor.
pub fn summed_reward(signals: &[f32]) -> f32 {
    signals.iter().sum()
}

/// Scale a stored baseline and clamp to the parameter's valid range.
///
/// Leak and excitability are clamped to `[0, 1]`. Threshold, transmission
/// gain, and learning rate stay non-negative. Reward is not a scaled parameter.
pub fn scale_baseline(kind: ModulatorKind, baseline: f32, factor: f32) -> f32 {
    let value = baseline * factor;
    match kind {
        ModulatorKind::Leak | ModulatorKind::FiringProbability => value.clamp(0.0, 1.0),
        ModulatorKind::FiringThreshold
        | ModulatorKind::TransmissionGain
        | ModulatorKind::LearningRate => value.max(0.0),
        ModulatorKind::Reward => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_driver_contributes_nothing() {
        assert_eq!(modulator_signal(30.0, false, false, 10.0, 10.0), 0.0);
    }

    #[test]
    fn ungraded_firing_uses_full_magnitude() {
        assert!((modulator_signal(30.0, true, false, 1.0, 10.0) - 0.3).abs() < 1e-6);
        assert!((modulator_signal(-80.0, true, false, 1.0, 10.0) + 0.8).abs() < 1e-6);
    }

    #[test]
    fn graded_firing_scales_with_membrane_potential_and_caps_at_one() {
        let half = modulator_signal(30.0, true, true, 5.0, 10.0);
        assert!((half - 0.15).abs() < 1e-6);
        let capped = modulator_signal(30.0, true, true, 40.0, 10.0);
        assert!((capped - 0.3).abs() < 1e-6);
    }

    #[test]
    fn factors_multiply_and_reward_adds() {
        let factor = multiplicative_factor(&[0.3, -0.5]);
        assert!((factor - (1.3 * 0.5)).abs() < 1e-6);
        assert!((summed_reward(&[0.3, -0.5]) + 0.2).abs() < 1e-6);
    }

    #[test]
    fn leak_and_excitability_clamp_to_unit_interval() {
        assert!((scale_baseline(ModulatorKind::Leak, 0.8, 2.0) - 1.0).abs() < 1e-6);
        assert!((scale_baseline(ModulatorKind::FiringProbability, 0.2, 0.0) - 0.0).abs() < 1e-6);
        assert!(scale_baseline(ModulatorKind::FiringThreshold, 2.0, 3.0) > 0.0);
    }

    #[test]
    fn magnitude_below_minus_100_is_rejected() {
        assert!(validate_instance_fields(-100.1, 1, false, None).is_err());
        assert!(validate_instance_fields(-100.0, 1, false, None).is_ok());
    }

    #[test]
    fn graded_requires_positive_full_scale() {
        assert!(validate_instance_fields(10.0, 1, true, None).is_err());
        assert!(validate_instance_fields(10.0, 1, true, Some(0.0)).is_err());
        assert!(validate_instance_fields(10.0, 1, true, Some(4.0)).is_ok());
    }

    #[test]
    fn spike_train_rejects_unlimited_consecutive_fire() {
        assert!(validate_spike_train(true, 0).is_err());
        assert!(validate_spike_train(true, 1).is_ok());
        assert!(validate_spike_train(false, 0).is_ok());
    }

    #[test]
    fn type_strings_round_trip() {
        for kind in ModulatorKind::all() {
            assert_eq!(ModulatorKind::parse(kind.as_str()).unwrap(), *kind);
        }
    }
}
