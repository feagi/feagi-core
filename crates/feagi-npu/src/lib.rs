
pub mod wnpu;
pub use npu::npu_target_frequency::NPUTargetFrequency; // this is an exception! Do not expose anything else!

// internal, do not expose!
mod npu;


