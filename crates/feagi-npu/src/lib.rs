pub mod engines;
pub mod dynamic_npu;
mod engines_common;
pub mod flags;
pub mod genome;
pub mod visualization;
// TODO: `npu_requests::npu_request_builder` references a `NPURequests` type that was never
// defined (pre-refactor scaffolding). Re-enable once that request model is designed;
// `dynamic_npu::DynamicNPU` currently uses `feagi_models::connectome_requests` instead.
// pub mod npu_requests;