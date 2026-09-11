//! Runtime abstraction the Trainer drives to execute a sample.
//!
//! `FeagiRuntime` is intentionally generic over its frame types so the Trainer is decoupled
//! from any concrete neuron-data representation. This lets the same orchestration drive:
//!
//! - a **remote** runtime (an existing FEAGI process over `feagi-agent` ZMQ), and
//! - an **embedded** runtime (in-process `feagi-npu`, preferred for benchmark determinism),
//!
//! and survive the in-flight NPU/quantization refactor that changes the concrete voxel types.

use crate::binding::reward::RewardSignal;
use crate::error::TrainerError;

/// A FEAGI runtime the Trainer can submit sensory frames to and collect motor frames from.
///
/// `SensoryFrame` is produced by an [`crate::binding::EncoderPlugin`]; `MotorFrame` is
/// consumed by a [`crate::binding::DecoderPlugin`]. Implementations own all FEAGI protocol
/// and tick semantics; the Trainer only sequences submit -> step -> collect.
///
/// ## Supervisory signals
///
/// Two supervisory channels feed learning, injected at different points in the loop:
///
/// - **Reward** ([`submit_reward`](Self::submit_reward)) — affect-channel stimulation derived
///   from the *observed* output, so it is injected *after* [`collect_motor`](Self::collect_motor).
///   This is the path implemented for the pendulum slice (plan Phase 1).
/// - **Target-motor / teaching** ([`submit_target_motor`](Self::submit_target_motor)) — a
///   *demonstrated* action injected *with sensory input, before* [`step`](Self::step) for
///   supervised forcing (behavior cloning). It is **reserved** here (default `Unsupported`) so
///   the executor loop seam and this contract do not change when imitation lands (plan §5.6).
pub trait FeagiRuntime {
    /// The encoded sensory payload type this runtime accepts.
    type SensoryFrame;
    /// The motor/OPU output type this runtime produces.
    type MotorFrame;

    /// Submits an encoded sensory frame for the next burst(s).
    fn submit_sensory(&mut self, frame: Self::SensoryFrame) -> Result<(), TrainerError>;

    /// Injects affect-channel reward stimulation (Pain/Pleasure/Fear/Hope) for the next
    /// burst(s). Called by the executor after the output is observed and scored.
    fn submit_reward(&mut self, signals: &[RewardSignal]) -> Result<(), TrainerError>;

    /// Reserved teaching / target-motor channel for imitation (behavior cloning).
    ///
    /// The `target` is a *demonstrated* action expressed in this runtime's own
    /// [`MotorFrame`](Self::MotorFrame) space (the expected motor output). The default returns
    /// [`TrainerError::Unsupported`]; a runtime opts in by overriding this when imitation lands
    /// (plan Phase 5; VLA gap 2). Reserving it keeps the loop seam fixed.
    fn submit_target_motor(&mut self, _target: Self::MotorFrame) -> Result<(), TrainerError> {
        Err(TrainerError::Unsupported(
            "target-motor teaching channel not implemented by this runtime".to_string(),
        ))
    }

    /// Advances the runtime by `ticks` bursts.
    fn step(&mut self, ticks: u32) -> Result<(), TrainerError>;

    /// Collects the motor/OPU output produced since the last collection.
    fn collect_motor(&mut self) -> Result<Self::MotorFrame, TrainerError>;
}
