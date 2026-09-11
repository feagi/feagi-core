//! Episodic-control metric pack — balance-duration / success-rate / return aggregates.
//!
//! Aggregates closed-loop [`EpisodeTrajectory`]s into the embodied/control metrics named in
//! design Section 5.8. The **headline** metric is `mean_episode_length` (mean balance
//! duration); `success_rate`, `mean_return`, and `duration_stddev` are reported alongside.
//!
//! `success_threshold_steps` is a pinned `EvaluationSpec` parameter supplied at construction —
//! it is never hardcoded here, so the success definition is part of the run's comparability
//! provenance (`evaluation_protocol_version` = `ctrl-pendulum-v1`).

use std::collections::BTreeMap;

use crate::contracts::common::{PluginId, PluginRef};
use crate::error::TrainerError;
use crate::plugins::episodic_metric::{EpisodeTrajectory, EpisodicMetricPack};
use crate::plugins::MetricResult;

/// Aggregates closed-loop episode trajectories into control metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpisodicControlMetricPack {
    /// Episodes whose duration reaches this many steps count as a success.
    success_threshold_steps: u32,
}

impl EpisodicControlMetricPack {
    /// Stable plugin id for this metric pack.
    pub const PLUGIN_ID: &'static str = "episodic_control";

    /// Creates the pack with the pinned success threshold (steps survived to count as a
    /// success). The threshold must be non-zero; it is sourced from the `EvaluationSpec`.
    pub fn new(success_threshold_steps: u32) -> Result<Self, TrainerError> {
        if success_threshold_steps == 0 {
            return Err(TrainerError::Config(
                "episodic_control success_threshold_steps must be > 0".to_string(),
            ));
        }
        Ok(Self {
            success_threshold_steps,
        })
    }
}

impl EpisodicMetricPack for EpisodicControlMetricPack {
    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId(Self::PLUGIN_ID.to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn evaluate(&self, episodes: &[EpisodeTrajectory]) -> Result<MetricResult, TrainerError> {
        if episodes.is_empty() {
            return Err(TrainerError::Evaluation(
                "cannot evaluate an empty episode set".to_string(),
            ));
        }

        let n = episodes.len() as f64;
        let durations: Vec<f64> = episodes.iter().map(|e| e.duration() as f64).collect();

        let mean_episode_length = durations.iter().sum::<f64>() / n;
        let mean_return = episodes.iter().map(|e| e.cumulative_reward()).sum::<f64>() / n;
        let successes = episodes
            .iter()
            .filter(|e| e.duration() as u32 >= self.success_threshold_steps)
            .count() as f64;
        let success_rate = successes / n;

        // Population standard deviation of episode duration (n-normalized; the rollout is the
        // whole population for this run, not a sample of a larger one).
        let variance = durations
            .iter()
            .map(|d| (d - mean_episode_length).powi(2))
            .sum::<f64>()
            / n;
        let duration_stddev = variance.sqrt();

        let mut metrics = BTreeMap::new();
        metrics.insert("mean_episode_length".to_string(), mean_episode_length);
        metrics.insert("success_rate".to_string(), success_rate);
        metrics.insert("mean_return".to_string(), mean_return);
        metrics.insert("duration_stddev".to_string(), duration_stddev);

        Ok(MetricResult {
            metrics,
            confusion: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::episodic_metric::EpisodeOutcome;

    fn episode(step_rewards: Vec<f64>, outcome: EpisodeOutcome) -> EpisodeTrajectory {
        EpisodeTrajectory {
            step_rewards,
            outcome,
        }
    }

    #[test]
    fn zero_threshold_is_rejected() {
        assert!(EpisodicControlMetricPack::new(0).is_err());
    }

    #[test]
    fn empty_episode_set_is_error() {
        let pack = EpisodicControlMetricPack::new(5).unwrap();
        assert!(pack.evaluate(&[]).is_err());
    }

    #[test]
    fn aggregates_duration_return_and_success() {
        let pack = EpisodicControlMetricPack::new(3).unwrap();
        // durations 2 and 4 -> mean 3; returns 2.0 and 4.0 -> mean 3.0.
        // success threshold 3 -> only the 4-step episode succeeds -> 0.5.
        let episodes = vec![
            episode(vec![1.0, 1.0], EpisodeOutcome::Terminated),
            episode(vec![1.0, 1.0, 1.0, 1.0], EpisodeOutcome::Truncated),
        ];
        let result = pack.evaluate(&episodes).expect("evaluate");
        assert!((result.metrics["mean_episode_length"] - 3.0).abs() < 1e-12);
        assert!((result.metrics["mean_return"] - 3.0).abs() < 1e-12);
        assert!((result.metrics["success_rate"] - 0.5).abs() < 1e-12);
        // stddev of {2,4} around 3 is 1.0.
        assert!((result.metrics["duration_stddev"] - 1.0).abs() < 1e-12);
        assert!(result.confusion.is_none());
    }

    #[test]
    fn shaped_rewards_sum_into_return() {
        let pack = EpisodicControlMetricPack::new(1).unwrap();
        let episodes = vec![episode(vec![0.5, 0.25, -0.75], EpisodeOutcome::Terminated)];
        let result = pack.evaluate(&episodes).expect("evaluate");
        assert!((result.metrics["mean_return"] - 0.0).abs() < 1e-12);
        assert!((result.metrics["mean_episode_length"] - 3.0).abs() < 1e-12);
        assert!((result.metrics["success_rate"] - 1.0).abs() < 1e-12);
    }
}
