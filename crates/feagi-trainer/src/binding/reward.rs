//! Reward axis — maps a sample outcome onto FEAGI's native affect channels (Appendix D.2).
//!
//! Reward is a first-class, versioned plugin axis. A policy converts the comparison of a
//! prediction against its target into stimulation of FEAGI Core affect areas
//! (Pain/Pleasure/Fear/Hope). The reward-policy version is part of the run comparability key.

use serde::{Deserialize, Serialize};

use crate::contracts::common::{PluginId, PluginRef};
use crate::contracts::ir_sample::TypedTarget;
use crate::contracts::prediction_record::TypedPrediction;
use crate::error::TrainerError;

/// A FEAGI Core affect channel a reward signal stimulates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AffectChannel {
    /// Negative reinforcement.
    Pain,
    /// Positive reinforcement.
    Pleasure,
    /// Anticipated negative outcome.
    Fear,
    /// Anticipated positive outcome.
    Hope,
}

/// A single reward stimulation produced by a policy for one sample.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewardSignal {
    /// The affect channel to stimulate.
    pub channel: AffectChannel,
    /// Stimulation magnitude in `[0.0, 1.0]`.
    pub magnitude: f64,
}

/// Converts a per-sample outcome into reward stimulation of FEAGI affect areas.
pub trait RewardPolicy {
    /// Versioned identity of this reward policy (recorded in provenance; comparability key).
    fn plugin_ref(&self) -> PluginRef;

    /// Produces the reward signals for one prediction/target pair.
    fn reward(
        &self,
        predicted: &TypedPrediction,
        target: &TypedTarget,
    ) -> Result<Vec<RewardSignal>, TrainerError>;
}

/// Single-label classification reward: Pleasure on a correct class, Pain otherwise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PainPleasureReward {
    magnitude: f64,
}

impl PainPleasureReward {
    /// Creates a policy with a fixed stimulation magnitude in `[0.0, 1.0]`.
    pub fn new(magnitude: f64) -> Result<Self, TrainerError> {
        if !(0.0..=1.0).contains(&magnitude) {
            return Err(TrainerError::Config(format!(
                "reward magnitude must be in [0.0, 1.0], got {magnitude}"
            )));
        }
        Ok(Self { magnitude })
    }
}

impl RewardPolicy for PainPleasureReward {
    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId("reward.pain_pleasure".to_string()),
            version: "1.0.0".to_string(),
        }
    }

    fn reward(
        &self,
        predicted: &TypedPrediction,
        target: &TypedTarget,
    ) -> Result<Vec<RewardSignal>, TrainerError> {
        let predicted_class = match predicted {
            TypedPrediction::Class { class_id, .. } => *class_id,
            other => {
                return Err(TrainerError::Config(format!(
                "pain_pleasure reward supports single-label class predictions only, got {other:?}"
            )))
            }
        };
        let target_class = match target {
            TypedTarget::Class { class_id, .. } => *class_id,
            other => {
                return Err(TrainerError::Config(format!(
                    "pain_pleasure reward supports single-label class targets only, got {other:?}"
                )))
            }
        };

        let channel = if predicted_class == target_class {
            AffectChannel::Pleasure
        } else {
            AffectChannel::Pain
        };
        Ok(vec![RewardSignal {
            channel,
            magnitude: self.magnitude,
        }])
    }
}

/// Converts an environment step outcome into FEAGI affect stimulation for **control** runs.
///
/// Unlike [`RewardPolicy`] (prediction-vs-target), control reward is environment-derived: there
/// is no per-step target. The policy maps the environment's scalar step reward plus the failure
/// flag into Pain/Pleasure stimulation, injected after each control step (paradigms doc §2.4 /
/// §2.1). Like `RewardPolicy` its version is part of the run comparability key.
pub trait EnvironmentRewardPolicy {
    /// Versioned identity of this reward policy (recorded in provenance; comparability key).
    fn plugin_ref(&self) -> PluginRef;

    /// Produces affect signals for one control step.
    ///
    /// `step_reward` is the environment's scalar reward for the step; `terminated` is true when
    /// the step ended the episode in an MDP-terminal failure (e.g. the pole fell) — distinct
    /// from a healthy time-limit truncation.
    fn reward(&self, step_reward: f64, terminated: bool)
        -> Result<Vec<RewardSignal>, TrainerError>;
}

/// Survival reward for balance-style control: Pleasure while progressing, Pain on failure.
///
/// - A failure step (`terminated`) emits Pain at `pain_magnitude` (the fall penalty).
/// - Otherwise a positive `step_reward` emits Pleasure at `pleasure_magnitude`, a negative one
///   emits Pain at `pain_magnitude`, and a zero reward emits no signal.
///
/// A healthy truncation (time cap reached) is *not* a failure and so carries no Pain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurvivalReward {
    pleasure_magnitude: f64,
    pain_magnitude: f64,
}

impl SurvivalReward {
    /// Stable plugin id for this control reward policy.
    pub const PLUGIN_ID: &'static str = "reward.survival";

    /// Creates a policy with fixed Pleasure/Pain magnitudes, each in `[0.0, 1.0]`.
    pub fn new(pleasure_magnitude: f64, pain_magnitude: f64) -> Result<Self, TrainerError> {
        for (name, m) in [
            ("pleasure_magnitude", pleasure_magnitude),
            ("pain_magnitude", pain_magnitude),
        ] {
            if !(0.0..=1.0).contains(&m) {
                return Err(TrainerError::Config(format!(
                    "survival reward {name} must be in [0.0, 1.0], got {m}"
                )));
            }
        }
        Ok(Self {
            pleasure_magnitude,
            pain_magnitude,
        })
    }
}

impl EnvironmentRewardPolicy for SurvivalReward {
    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId(Self::PLUGIN_ID.to_string()),
            version: "1.0.0".to_string(),
        }
    }

    fn reward(
        &self,
        step_reward: f64,
        terminated: bool,
    ) -> Result<Vec<RewardSignal>, TrainerError> {
        if terminated {
            return Ok(vec![RewardSignal {
                channel: AffectChannel::Pain,
                magnitude: self.pain_magnitude,
            }]);
        }
        let signal = if step_reward > 0.0 {
            Some(RewardSignal {
                channel: AffectChannel::Pleasure,
                magnitude: self.pleasure_magnitude,
            })
        } else if step_reward < 0.0 {
            Some(RewardSignal {
                channel: AffectChannel::Pain,
                magnitude: self.pain_magnitude,
            })
        } else {
            None
        };
        Ok(signal.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class(id: u32) -> TypedTarget {
        TypedTarget::Class {
            class_id: id,
            label: None,
        }
    }

    fn predict(id: u32) -> TypedPrediction {
        TypedPrediction::Class {
            class_id: id,
            scores: vec![],
        }
    }

    #[test]
    fn correct_prediction_rewards_pleasure() {
        let policy = PainPleasureReward::new(0.8).unwrap();
        let signals = policy.reward(&predict(2), &class(2)).unwrap();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].channel, AffectChannel::Pleasure);
        assert_eq!(signals[0].magnitude, 0.8);
    }

    #[test]
    fn incorrect_prediction_rewards_pain() {
        let policy = PainPleasureReward::new(0.5).unwrap();
        let signals = policy.reward(&predict(0), &class(1)).unwrap();
        assert_eq!(signals[0].channel, AffectChannel::Pain);
    }

    #[test]
    fn out_of_range_magnitude_rejected() {
        assert!(PainPleasureReward::new(1.5).is_err());
        assert!(PainPleasureReward::new(-0.1).is_err());
    }

    #[test]
    fn non_class_prediction_rejected() {
        let policy = PainPleasureReward::new(0.5).unwrap();
        assert!(policy
            .reward(&TypedPrediction::Scalar(0.5), &class(0))
            .is_err());
    }

    #[test]
    fn survival_rejects_out_of_range_magnitudes() {
        assert!(SurvivalReward::new(1.5, 0.5).is_err());
        assert!(SurvivalReward::new(0.5, -0.1).is_err());
    }

    #[test]
    fn survival_failure_step_is_pain() {
        let policy = SurvivalReward::new(0.7, 0.9).unwrap();
        let signals = policy.reward(1.0, true).unwrap();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].channel, AffectChannel::Pain);
        assert_eq!(signals[0].magnitude, 0.9);
    }

    #[test]
    fn survival_positive_step_is_pleasure() {
        let policy = SurvivalReward::new(0.7, 0.9).unwrap();
        let signals = policy.reward(1.0, false).unwrap();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].channel, AffectChannel::Pleasure);
        assert_eq!(signals[0].magnitude, 0.7);
    }

    #[test]
    fn survival_zero_step_emits_no_signal() {
        let policy = SurvivalReward::new(0.7, 0.9).unwrap();
        assert!(policy.reward(0.0, false).unwrap().is_empty());
    }

    #[test]
    fn survival_negative_step_is_pain() {
        let policy = SurvivalReward::new(0.7, 0.9).unwrap();
        let signals = policy.reward(-0.5, false).unwrap();
        assert_eq!(signals[0].channel, AffectChannel::Pain);
    }
}
