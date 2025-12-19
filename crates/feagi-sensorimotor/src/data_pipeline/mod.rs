//! Data processing pipeline infrastructure.
//!
//! Provides a flexible pipeline system for preprocessing sensor data before encoding
//! and postprocessing motor data after decoding. Stages can perform transformations
//! like image segmentation, filtering, normalization, and more.
//!
//! Pipelines are configured via properties and can be dynamically modified at runtime.

mod descriptors;
pub(crate) mod per_channel_stream_caches;
mod pipeline_stage;
mod pipeline_stage_conversions;
mod pipeline_stage_properties;
pub mod stages;

pub use descriptors::PipelineStagePropertyIndex;
pub(crate) use pipeline_stage::PipelineStage;
pub(crate) use pipeline_stage_conversions::stage_properties_to_stages;
pub use pipeline_stage_properties::PipelineStageProperties;
