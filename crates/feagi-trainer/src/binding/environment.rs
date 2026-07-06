//! Closed-loop environment seam.
//!
//! PARKED (ADR-014/ADR-015): this seam was the engine for the superseded "Topology C — Trainer
//! drives the sim" model. The live embodied path is now the **parallel co-agent** model: the
//! embodiment controller owns the robot's sensory/motor + physics, and the Trainer is a separate
//! FEAGI agent injecting training signals on disjoint cortical_area I/O — it does **not** drive the
//! environment. This module is retained only for a possible *trainer-owned, no-controller* sim
//! path and is not on the live embodied path. Do not wire it into `RunConfig`/CLI without a
//! decision reopening that use case. The episodic-control metric pack + `EpisodeTrajectory`
//! (which consume `StepOutcome`-shaped results) remain in active use.
//!
//! Embodied/control tasks are not driven by a static dataset: the observation at step *t+1* is
//! the environment state produced by applying the brain's decoded action at step *t*. The
//! [`Environment`] trait wraps a simulator (e.g. the MuJoCo inverted pendulum). The Trainer
//! implements no physics — it only sequences `reset` → (`step`)* and reads the
//! environment-provided reward + termination.

use crate::error::TrainerError;

/// A continuous observation vector read from the environment (e.g. cart/pole state).
pub type Observation = Vec<f64>;

/// A continuous action vector applied to the environment (e.g. cart force).
pub type Action = Vec<f64>;

/// The result of applying one action to the environment for one control step.
///
/// Failure and time-limit endings are distinguished (standard RL `terminated` vs `truncated`):
/// a failure (`terminated`) is an MDP-terminal event such as the pole falling, whereas a
/// `truncated` ending is an external cap (max steps) reached while still healthy. The reward
/// policy treats them differently (Pain on failure only), and they map to distinct
/// [`EpisodeOutcome`](crate::plugins::EpisodeOutcome)s.
#[derive(Debug, Clone, PartialEq)]
pub struct StepOutcome {
    /// The observation after the action was applied.
    pub observation: Observation,
    /// The scalar reward emitted by the environment for this step.
    pub reward: f64,
    /// The episode reached an MDP-terminal failure state on this step (e.g. the pole fell).
    pub terminated: bool,
    /// The episode hit an external step cap on this step while still healthy.
    pub truncated: bool,
}

impl StepOutcome {
    /// Whether the episode ended on this step for any reason (failure or time cap).
    pub fn done(&self) -> bool {
        self.terminated || self.truncated
    }
}

/// A closed-loop environment the Trainer drives for embodied/control rollouts.
///
/// Implementations own all simulator/transport semantics; the Trainer only sequences
/// `reset` → (`step`)* and reads the returned reward + `done`. Determinism is the
/// implementation's responsibility: [`reset`](Self::reset) takes an explicit seed so a run is
/// reproducible (plan Section 9).
pub trait Environment {
    /// Resets to an initial state for a new episode and returns the first observation. `seed`
    /// makes the episode reproducible.
    fn reset(&mut self, seed: u64) -> Result<Observation, TrainerError>;

    /// Applies one continuous action and advances the environment by one control step.
    fn step(&mut self, action: &Action) -> Result<StepOutcome, TrainerError>;

    /// Inclusive per-dimension action bounds `(low, high)` used to clamp decoded actions to the
    /// actuator's `ctrlrange`. Both vectors have length equal to the action dimensionality.
    fn action_bounds(&self) -> (Action, Action);
}

/// A deterministic, physics-free environment for testing the closed-loop seam.
///
/// This is **not** a simulator. It is a minimal 1-D balance toy: a scalar state integrates the
/// (clamped) applied action each step, the episode survives while `|state| <= fail_threshold`,
/// truncates at `max_steps`, and emits `+1` reward per surviving step. It lets the closed-loop
/// executor and episodic metric pack be tested without a MuJoCo dependency (it is a test
/// collaborator, never the subject under test).
#[derive(Debug, Clone)]
pub struct StubEnvironment {
    state: f64,
    steps: u32,
    fail_threshold: f64,
    max_steps: u32,
    force_low: f64,
    force_high: f64,
}

impl StubEnvironment {
    /// Creates a balance toy that fails when `|state| > fail_threshold`, truncates at
    /// `max_steps`, and clamps actions to `[force_low, force_high]`.
    pub fn new(
        fail_threshold: f64,
        max_steps: u32,
        force_low: f64,
        force_high: f64,
    ) -> Result<Self, TrainerError> {
        if fail_threshold <= 0.0 {
            return Err(TrainerError::Config(
                "StubEnvironment fail_threshold must be positive".to_string(),
            ));
        }
        if max_steps == 0 {
            return Err(TrainerError::Config(
                "StubEnvironment max_steps must be > 0".to_string(),
            ));
        }
        if force_low > force_high {
            return Err(TrainerError::Config(
                "StubEnvironment force_low must not exceed force_high".to_string(),
            ));
        }
        Ok(Self {
            state: 0.0,
            steps: 0,
            fail_threshold,
            max_steps,
            force_low,
            force_high,
        })
    }
}

impl Environment for StubEnvironment {
    fn reset(&mut self, seed: u64) -> Result<Observation, TrainerError> {
        // Deterministic, seed-dependent initial offset within the safe band so different
        // seeds give different (but reproducible) episodes.
        let span = self.fail_threshold;
        let unit = (seed % 1000) as f64 / 1000.0; // [0, 1)
        self.state = (unit - 0.5) * span; // (-span/2, span/2)
        self.steps = 0;
        Ok(vec![self.state])
    }

    fn step(&mut self, action: &Action) -> Result<StepOutcome, TrainerError> {
        let force = *action
            .first()
            .ok_or_else(|| TrainerError::Runtime("empty action vector".to_string()))?;
        let clamped = force.clamp(self.force_low, self.force_high);
        // Constant drift away from center plus the corrective action integrates the state.
        self.state += self.state.signum() * 0.01 + clamped;
        self.steps += 1;

        let failed = self.state.abs() > self.fail_threshold;
        // A failure takes precedence over the step cap on the same step.
        let truncated = !failed && self.steps >= self.max_steps;
        Ok(StepOutcome {
            observation: vec![self.state],
            reward: 1.0,
            terminated: failed,
            truncated,
        })
    }

    fn action_bounds(&self) -> (Action, Action) {
        (vec![self.force_low], vec![self.force_high])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_config() {
        assert!(StubEnvironment::new(0.0, 10, -1.0, 1.0).is_err());
        assert!(StubEnvironment::new(1.0, 0, -1.0, 1.0).is_err());
        assert!(StubEnvironment::new(1.0, 10, 1.0, -1.0).is_err());
    }

    #[test]
    fn reset_is_deterministic_for_a_seed() {
        let mut env = StubEnvironment::new(1.0, 100, -1.0, 1.0).unwrap();
        let a = env.reset(42).unwrap();
        let b = env.reset(42).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_give_different_starts() {
        let mut env = StubEnvironment::new(1.0, 100, -1.0, 1.0).unwrap();
        let a = env.reset(1).unwrap();
        let b = env.reset(999).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn action_is_clamped_to_bounds() {
        let mut env = StubEnvironment::new(100.0, 100, -0.5, 0.5).unwrap();
        env.reset(0).unwrap();
        // A huge force is clamped to 0.5, so the state moves by ~0.5 (+ tiny drift), not 1000.
        let outcome = env.step(&vec![1000.0]).unwrap();
        assert!(outcome.observation[0] < 1.0);
    }

    #[test]
    fn truncates_at_max_steps_when_balanced() {
        // No drift away can be overcome: zero action, small drift, high threshold -> reaches cap.
        let mut env = StubEnvironment::new(1000.0, 5, -1.0, 1.0).unwrap();
        env.reset(0).unwrap();
        let mut steps = 0;
        let outcome = loop {
            let outcome = env.step(&vec![0.0]).unwrap();
            steps += 1;
            if outcome.done() {
                break outcome;
            }
        };
        assert_eq!(steps, 5);
        assert!(outcome.truncated && !outcome.terminated);
    }

    #[test]
    fn empty_action_is_error() {
        let mut env = StubEnvironment::new(1.0, 10, -1.0, 1.0).unwrap();
        env.reset(0).unwrap();
        assert!(env.step(&vec![]).is_err());
    }
}
