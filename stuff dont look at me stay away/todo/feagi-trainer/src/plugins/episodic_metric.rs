//! Episodic-control metric axis — evaluation from rolled-out episode trajectories.
//!
//! Unlike [`MetricPackPlugin`](crate::plugins::MetricPackPlugin) (which scores aligned
//! prediction/target pairs from a static dataset), embodied/control tasks are scored from
//! *trajectories* produced by a closed-loop rollout: there is no per-step ground-truth target,
//! and success is episodic (design Section 5.8; `FEAGI_TRAINER_TRAINING_PARADIGMS.md` §2.4).
//!
//! This axis is purely additive — it does not change the offline prediction/target metric
//! path. A control run produces [`EpisodeTrajectory`]s instead of `PredictionRecord`s and
//! scores them with an [`EpisodicMetricPack`].

use crate::contracts::common::PluginRef;
use crate::error::TrainerError;
use crate::plugins::MetricResult;

/// How an episode ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpisodeOutcome {
    /// The episode failed (e.g. the pole fell past the angle threshold).
    Terminated,
    /// The episode reached the max-steps cap without failing.
    Truncated,
}

/// One completed episode's trajectory: per-step rewards in visit order plus how it ended.
///
/// `step_rewards.len()` is the number of control steps the agent survived (= balance
/// duration). The terminal step is included, so a survival-reward task has one reward entry
/// per step taken.
#[derive(Debug, Clone, PartialEq)]
pub struct EpisodeTrajectory {
    /// Per-step scalar rewards, in visit order.
    pub step_rewards: Vec<f64>,
    /// How the episode terminated.
    pub outcome: EpisodeOutcome,
}

impl EpisodeTrajectory {
    /// Number of control steps the agent survived (the balance duration).
    pub fn duration(&self) -> usize {
        self.step_rewards.len()
    }

    /// Cumulative (summed) reward over the episode.
    pub fn cumulative_reward(&self) -> f64 {
        self.step_rewards.iter().sum()
    }
}

/// Scores a set of episode trajectories for an embodied/control task family.
///
/// Mirrors [`MetricPackPlugin`](crate::plugins::MetricPackPlugin) for the closed-loop case:
/// pure, deterministic, and never reads the runtime or environment directly.
pub trait EpisodicMetricPack {
    /// Identifies this metric pack (axis provenance).
    fn plugin_ref(&self) -> PluginRef;

    /// Computes aggregate metrics from completed episode trajectories.
    ///
    /// Returns an explicit error on empty input (a control run must roll at least one episode).
    fn evaluate(&self, episodes: &[EpisodeTrajectory]) -> Result<MetricResult, TrainerError>;
}
