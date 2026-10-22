//! Spike-cost metric axis — evaluation from collected runtime spike telemetry.
//!
//! Unlike [`MetricPackPlugin`](crate::plugins::MetricPackPlugin) (offline prediction/target
//! scoring) and [`EpisodicMetricPack`](crate::plugins::EpisodicMetricPack) (episode
//! trajectories), the hardware-level *cost* metrics (spike count, spike density) are runtime
//! telemetry. This axis keeps the same purity contract: the pack consumes a
//! [`SpikeCostObservation`] collected during the run and aggregates it deterministically; it
//! never reads the runtime itself.
//!
//! Source-agnostic by design: the observation is expressed as cumulative-counter *deltas* over
//! the measurement window, so it is exact regardless of how often the source is read (no
//! sampling approximation). The network-wide spike/burst counter that populates it is a
//! burst-engine telemetry output; that source is intentionally decoupled here (it is being
//! built into the NPU rearchitecture), and this contract is stable across that change.

use crate::contracts::common::PluginRef;
use crate::error::TrainerError;
use crate::plugins::MetricResult;

/// Collected spike telemetry for one measurement window, expressed as cumulative-counter deltas.
///
/// All fields are exact deltas between a start and end read of monotonic runtime counters, so
/// the observation does not depend on polling cadence. The collector is responsible for reading
/// the counters at the run-window boundaries; this struct carries no timing assumptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpikeCostObservation {
    /// Total neurons that fired across the whole network during the window
    /// (`end_cumulative_spikes - start_cumulative_spikes`).
    pub spikes_delta: u64,
    /// Number of bursts processed during the window (`end_burst - start_burst`). Must be > 0.
    pub bursts_delta: u64,
    /// Total neuron count of the network under test — the spike-density denominator. Must be > 0.
    pub network_neuron_count: u64,
}

/// Scores collected spike telemetry into hardware-level cost metrics.
///
/// Mirrors the other metric-pack traits: pure, deterministic, and never reads the runtime.
pub trait SpikeCostMetricPack {
    /// Identifies this metric pack (axis provenance).
    fn plugin_ref(&self) -> PluginRef;

    /// Computes aggregate cost metrics from a collected observation.
    ///
    /// Returns an explicit error when the observation cannot define the metrics (zero bursts or
    /// zero network neurons) — never a fallback value.
    fn evaluate(&self, observation: &SpikeCostObservation) -> Result<MetricResult, TrainerError>;
}
