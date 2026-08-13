pub mod dynamic_npu;
mod engines_common;
// Memory-neuron ID ranges, relocated from the dissolved `feagi-npu-plasticity` crate.
#[cfg(feature = "std")]
// Burst-loop introspection taps, relocated from the dissolved `feagi-npu-burst-engine` crate.
// Needs `OnceLock` and the system clock, so it is std-only.
#[cfg(feature = "std")]
pub mod burst_engine;
pub mod burst_engine_enum;
pub mod wnpu;
pub mod npu;
// TODO: `npu_requests::npu_request_builder` references a `NPURequests` type that was never
// defined (pre-refactor scaffolding). Re-enable once that request model is designed;
// `dynamic_npu::DynamicNPU` currently uses `feagi_models::connectome_requests` instead.
// pub mod npu_requests;
