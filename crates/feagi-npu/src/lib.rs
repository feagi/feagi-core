pub mod dynamic_npu;
mod engines_common;
pub mod genome;
// Memory-neuron ID ranges, relocated from the dissolved `feagi-npu-plasticity` crate.
#[cfg(feature = "std")]
pub mod memory_neuron_ids;
// Burst-loop introspection taps, relocated from the dissolved `feagi-npu-burst-engine` crate.
// Needs `OnceLock` and the system clock, so it is std-only.
#[cfg(feature = "std")]
pub mod runtime_taps;
pub mod visualization;
pub mod engine_runners;
pub mod burst_engine;
// TODO: `npu_requests::npu_request_builder` references a `NPURequests` type that was never
// defined (pre-refactor scaffolding). Re-enable once that request model is designed;
// `dynamic_npu::DynamicNPU` currently uses `feagi_models::connectome_requests` instead.
// pub mod npu_requests;
