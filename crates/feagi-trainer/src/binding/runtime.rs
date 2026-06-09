//! Runtime abstraction the Trainer drives to execute a sample.
//!
//! `FeagiRuntime` is intentionally generic over its frame types so the Trainer is decoupled
//! from any concrete neuron-data representation. This lets the same orchestration drive:
//!
//! - a **remote** runtime (an existing FEAGI process over `feagi-agent` ZMQ), and
//! - an **embedded** runtime (in-process `feagi-npu`, preferred for benchmark determinism),
//!
//! and survive the in-flight NPU/quantization refactor that changes the concrete voxel types.

use crate::error::TrainerError;

/// A FEAGI runtime the Trainer can submit sensory frames to and collect motor frames from.
///
/// `SensoryFrame` is produced by an [`crate::binding::EncoderPlugin`]; `MotorFrame` is
/// consumed by a [`crate::binding::DecoderPlugin`]. Implementations own all FEAGI protocol
/// and tick semantics; the Trainer only sequences submit -> step -> collect.
pub trait FeagiRuntime {
    /// The encoded sensory payload type this runtime accepts.
    type SensoryFrame;
    /// The motor/OPU output type this runtime produces.
    type MotorFrame;

    /// Submits an encoded sensory frame for the next burst(s).
    fn submit_sensory(&mut self, frame: Self::SensoryFrame) -> Result<(), TrainerError>;

    /// Advances the runtime by `ticks` bursts.
    fn step(&mut self, ticks: u32) -> Result<(), TrainerError>;

    /// Collects the motor/OPU output produced since the last collection.
    fn collect_motor(&mut self) -> Result<Self::MotorFrame, TrainerError>;
}
