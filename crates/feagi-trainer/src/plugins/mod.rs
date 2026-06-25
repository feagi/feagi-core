//! Plugin-axis interfaces (ADR-002 four-axis (+reward) model).
//!
//! Each axis is a trait so new dataset-and-architecture combinations are supported by
//! composing or adding plugins, never by editing orchestration. This module defines the
//! pure-pipeline axes that do not touch the FEAGI runtime:
//!
//! - [`adapter::AdapterPlugin`] — ingest source data into the canonical IR.
//! - [`sampler::SamplerPlugin`] — deterministic ordering/scheduling.
//! - [`metric_pack::MetricPackPlugin`] — evaluation from predictions + targets.
//!
//! The FEAGI-binding axes (`EncoderPlugin` / `DecoderPlugin`) and `RewardPolicy` are defined
//! alongside the runtime abstraction in a later slice, since their signatures reference the
//! runtime payload types.

pub mod adapter;
pub mod episodic_metric;
pub mod metric_pack;
pub mod sampler;
pub mod spike_cost_metric;

pub use adapter::{AdapterPlugin, DatasetSource, ValidationReport};
pub use episodic_metric::{EpisodeOutcome, EpisodeTrajectory, EpisodicMetricPack};
pub use metric_pack::{ClassMetrics, ConfusionMatrix, MetricPackPlugin, MetricResult};
pub use sampler::SamplerPlugin;
pub use spike_cost_metric::{SpikeCostMetricPack, SpikeCostObservation};
