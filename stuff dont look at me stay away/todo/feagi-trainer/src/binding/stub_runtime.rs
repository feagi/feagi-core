//! Deterministic in-process [`FeagiRuntime`] for testing the run executor and binding seam.
//!
//! This stub implements the runtime contract with a fixed, side-effect-free transform so the
//! executor (and binding) can be unit-tested without a live FEAGI (per the plan's Phase 1
//! test strategy: "an integration test driving a stub `FeagiRuntime` (deterministic)").
//!
//! It is **not** a FEAGI simulation: `collect_motor` returns a pure function of the most
//! recently submitted sensory frame, and `step` only advances an internal burst counter. It
//! records submitted reward and target-motor signals so tests can assert the loop sequenced
//! them correctly. Because it is mocking a collaborator *outside* the unit under test (the
//! runtime), not the subject itself, it complies with the project's mocking policy.

use crate::binding::reward::RewardSignal;
use crate::binding::runtime::FeagiRuntime;
use crate::error::TrainerError;

/// A deterministic, in-process runtime used to test the executor/binding loop.
///
/// `SensoryFrame` and `MotorFrame` are simple `Vec<f64>` channel vectors so tests can assert
/// exact values without a neuron-voxel representation. The motor output is produced by
/// applying [`StubFeagiRuntime::transform`] to the last submitted sensory frame.
#[derive(Debug, Clone)]
pub struct StubFeagiRuntime {
    transform: fn(&[f64]) -> Vec<f64>,
    last_sensory: Option<Vec<f64>>,
    burst_count: u64,
    submitted_rewards: Vec<RewardSignal>,
    submitted_targets: Vec<Vec<f64>>,
    teaching_supported: bool,
}

impl StubFeagiRuntime {
    /// Creates a stub whose motor output is `transform(last_sensory)`.
    ///
    /// `teaching_supported` controls whether [`FeagiRuntime::submit_target_motor`] is honored
    /// (records the target) or left to the trait default (returns
    /// [`TrainerError::Unsupported`]) — letting tests exercise both the reserved-seam default
    /// and an opt-in imitation-capable runtime.
    pub fn new(transform: fn(&[f64]) -> Vec<f64>, teaching_supported: bool) -> Self {
        Self {
            transform,
            last_sensory: None,
            burst_count: 0,
            submitted_rewards: Vec::new(),
            submitted_targets: Vec::new(),
            teaching_supported,
        }
    }

    /// Creates an identity stub (motor output echoes the last sensory frame), no teaching.
    pub fn identity() -> Self {
        Self::new(|sensory| sensory.to_vec(), false)
    }

    /// Total bursts advanced via [`FeagiRuntime::step`] since construction.
    pub fn burst_count(&self) -> u64 {
        self.burst_count
    }

    /// Reward signals submitted so far, in submission order.
    pub fn submitted_rewards(&self) -> &[RewardSignal] {
        &self.submitted_rewards
    }

    /// Target-motor (teaching) frames submitted so far, in submission order.
    pub fn submitted_targets(&self) -> &[Vec<f64>] {
        &self.submitted_targets
    }
}

impl FeagiRuntime for StubFeagiRuntime {
    type SensoryFrame = Vec<f64>;
    type MotorFrame = Vec<f64>;

    fn submit_sensory(&mut self, frame: Self::SensoryFrame) -> Result<(), TrainerError> {
        self.last_sensory = Some(frame);
        Ok(())
    }

    fn submit_reward(&mut self, signals: &[RewardSignal]) -> Result<(), TrainerError> {
        self.submitted_rewards.extend_from_slice(signals);
        Ok(())
    }

    fn submit_target_motor(&mut self, target: Self::MotorFrame) -> Result<(), TrainerError> {
        if !self.teaching_supported {
            return Err(TrainerError::Unsupported(
                "stub runtime constructed without teaching support".to_string(),
            ));
        }
        self.submitted_targets.push(target);
        Ok(())
    }

    fn step(&mut self, ticks: u32) -> Result<(), TrainerError> {
        self.burst_count += u64::from(ticks);
        Ok(())
    }

    fn collect_motor(&mut self) -> Result<Self::MotorFrame, TrainerError> {
        match &self.last_sensory {
            Some(sensory) => Ok((self.transform)(sensory)),
            None => Err(TrainerError::Runtime(
                "collect_motor called before any sensory frame was submitted".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::reward::AffectChannel;

    /// A runtime that implements only the required methods so the reserved
    /// `submit_target_motor` default (Unsupported) is exercised directly.
    struct MinimalRuntime;

    impl FeagiRuntime for MinimalRuntime {
        type SensoryFrame = ();
        type MotorFrame = ();

        fn submit_sensory(&mut self, _frame: ()) -> Result<(), TrainerError> {
            Ok(())
        }
        fn submit_reward(&mut self, _signals: &[RewardSignal]) -> Result<(), TrainerError> {
            Ok(())
        }
        fn step(&mut self, _ticks: u32) -> Result<(), TrainerError> {
            Ok(())
        }
        fn collect_motor(&mut self) -> Result<(), TrainerError> {
            Ok(())
        }
    }

    fn pleasure(magnitude: f64) -> RewardSignal {
        RewardSignal {
            channel: AffectChannel::Pleasure,
            magnitude,
        }
    }

    #[test]
    fn loop_sequences_submit_step_collect() {
        let mut runtime = StubFeagiRuntime::identity();
        runtime.submit_sensory(vec![0.1, 0.2, 0.7]).unwrap();
        runtime.step(4).unwrap();
        let motor = runtime.collect_motor().unwrap();
        assert_eq!(motor, vec![0.1, 0.2, 0.7]);
        assert_eq!(runtime.burst_count(), 4);
    }

    #[test]
    fn transform_is_applied_to_latest_sensory() {
        let mut runtime = StubFeagiRuntime::new(|s| s.iter().map(|v| v * 2.0).collect(), false);
        runtime.submit_sensory(vec![1.0, 2.0]).unwrap();
        runtime.submit_sensory(vec![3.0, 4.0]).unwrap();
        assert_eq!(runtime.collect_motor().unwrap(), vec![6.0, 8.0]);
    }

    #[test]
    fn collect_before_submit_errors() {
        let mut runtime = StubFeagiRuntime::identity();
        assert!(matches!(
            runtime.collect_motor(),
            Err(TrainerError::Runtime(_))
        ));
    }

    #[test]
    fn reward_signals_are_recorded_in_order() {
        let mut runtime = StubFeagiRuntime::identity();
        runtime.submit_reward(&[pleasure(0.8)]).unwrap();
        runtime.submit_reward(&[pleasure(0.5)]).unwrap();
        let recorded = runtime.submitted_rewards();
        assert_eq!(recorded.len(), 2);
        assert_eq!(recorded[0].magnitude, 0.8);
        assert_eq!(recorded[1].magnitude, 0.5);
    }

    #[test]
    fn target_motor_default_is_unsupported() {
        let mut runtime = MinimalRuntime;
        assert!(matches!(
            runtime.submit_target_motor(()),
            Err(TrainerError::Unsupported(_))
        ));
    }

    #[test]
    fn target_motor_unsupported_when_not_enabled() {
        let mut runtime = StubFeagiRuntime::identity();
        assert!(matches!(
            runtime.submit_target_motor(vec![0.0]),
            Err(TrainerError::Unsupported(_))
        ));
        assert!(runtime.submitted_targets().is_empty());
    }

    #[test]
    fn target_motor_recorded_when_teaching_enabled() {
        let mut runtime = StubFeagiRuntime::new(|s| s.to_vec(), true);
        runtime.submit_target_motor(vec![0.3, 0.7]).unwrap();
        assert_eq!(runtime.submitted_targets(), &[vec![0.3, 0.7]]);
    }
}
