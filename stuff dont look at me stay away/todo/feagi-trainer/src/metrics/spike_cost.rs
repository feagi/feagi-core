//! Spike-cost metric pack — network-wide hardware-level cost aggregates.
//!
//! Computes hardware-level cost metrics from a collected [`SpikeCostObservation`]: total
//! spikes, mean spikes per burst, and mean spike density (the fraction of the network firing
//! per burst). Pure and deterministic; the observation is sourced from runtime counters by the
//! collector, not read here.
//!
//! Metric keys are namespaced under `cost.` so they merge into a scorecard alongside a primary
//! (classification / episodic-control) pack without colliding.

use std::collections::BTreeMap;

use crate::contracts::common::{PluginId, PluginRef};
use crate::error::TrainerError;
use crate::plugins::spike_cost_metric::{SpikeCostMetricPack, SpikeCostObservation};
use crate::plugins::MetricResult;

/// Aggregates collected network spike telemetry into hardware-level cost metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NetworkSpikeCostPack;

impl NetworkSpikeCostPack {
    /// Stable plugin id for this metric pack.
    pub const PLUGIN_ID: &'static str = "network_spike_cost";

    /// Metric key: total network spikes over the measurement window.
    pub const KEY_TOTAL_SPIKES: &'static str = "cost.total_spikes";
    /// Metric key: mean spikes per burst.
    pub const KEY_MEAN_SPIKES_PER_BURST: &'static str = "cost.mean_spikes_per_burst";
    /// Metric key: mean fraction of the network firing per burst, in `[0, 1]`.
    pub const KEY_MEAN_SPIKE_DENSITY: &'static str = "cost.mean_spike_density";

    /// Creates the pack. Stateless; construction cannot fail.
    pub fn new() -> Self {
        Self
    }
}

impl SpikeCostMetricPack for NetworkSpikeCostPack {
    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId(Self::PLUGIN_ID.to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn evaluate(&self, observation: &SpikeCostObservation) -> Result<MetricResult, TrainerError> {
        if observation.bursts_delta == 0 {
            return Err(TrainerError::Evaluation(
                "spike-cost observation has zero bursts; cannot define per-burst metrics"
                    .to_string(),
            ));
        }
        if observation.network_neuron_count == 0 {
            return Err(TrainerError::Evaluation(
                "spike-cost observation has zero network neurons; spike density is undefined"
                    .to_string(),
            ));
        }

        let total_spikes = observation.spikes_delta as f64;
        let mean_spikes_per_burst = total_spikes / observation.bursts_delta as f64;
        let mean_spike_density = mean_spikes_per_burst / observation.network_neuron_count as f64;

        let mut metrics = BTreeMap::new();
        metrics.insert(Self::KEY_TOTAL_SPIKES.to_string(), total_spikes);
        metrics.insert(
            Self::KEY_MEAN_SPIKES_PER_BURST.to_string(),
            mean_spikes_per_burst,
        );
        metrics.insert(Self::KEY_MEAN_SPIKE_DENSITY.to_string(), mean_spike_density);

        Ok(MetricResult {
            metrics,
            confusion: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(spikes: u64, bursts: u64, neurons: u64) -> SpikeCostObservation {
        SpikeCostObservation {
            spikes_delta: spikes,
            bursts_delta: bursts,
            network_neuron_count: neurons,
        }
    }

    #[test]
    fn aggregates_total_mean_and_density() {
        // 1000 spikes over 10 bursts in a 200-neuron network:
        //   total = 1000, mean/burst = 100, density = 100/200 = 0.5.
        let result = NetworkSpikeCostPack::new()
            .evaluate(&observation(1000, 10, 200))
            .expect("evaluate");
        assert_eq!(
            result.metrics[NetworkSpikeCostPack::KEY_TOTAL_SPIKES],
            1000.0
        );
        assert_eq!(
            result.metrics[NetworkSpikeCostPack::KEY_MEAN_SPIKES_PER_BURST],
            100.0
        );
        assert!((result.metrics[NetworkSpikeCostPack::KEY_MEAN_SPIKE_DENSITY] - 0.5).abs() < 1e-12);
        assert!(result.confusion.is_none());
    }

    #[test]
    fn zero_spikes_is_valid_and_yields_zero_cost() {
        // A silent window is a legitimate, informative measurement (not an error).
        let result = NetworkSpikeCostPack::new()
            .evaluate(&observation(0, 5, 100))
            .expect("evaluate");
        assert_eq!(result.metrics[NetworkSpikeCostPack::KEY_TOTAL_SPIKES], 0.0);
        assert_eq!(
            result.metrics[NetworkSpikeCostPack::KEY_MEAN_SPIKE_DENSITY],
            0.0
        );
    }

    #[test]
    fn zero_bursts_is_explicit_error() {
        let err = NetworkSpikeCostPack::new()
            .evaluate(&observation(10, 0, 100))
            .unwrap_err();
        assert!(matches!(err, TrainerError::Evaluation(_)));
    }

    #[test]
    fn zero_network_neurons_is_explicit_error() {
        let err = NetworkSpikeCostPack::new()
            .evaluate(&observation(10, 5, 0))
            .unwrap_err();
        assert!(matches!(err, TrainerError::Evaluation(_)));
    }

    #[test]
    fn keys_are_cost_namespaced_for_collision_free_scorecard_merge() {
        let result = NetworkSpikeCostPack::new()
            .evaluate(&observation(10, 5, 100))
            .expect("evaluate");
        assert!(result.metrics.keys().all(|k| k.starts_with("cost.")));
    }

    #[test]
    fn merges_into_a_primary_scorecard_metrics_map() {
        // Wiring: cost metrics combine into the same map a primary pack produced, which is what
        // `Scorecard::metrics` carries. `cost.` namespacing keeps them collision-free.
        let mut scorecard_metrics = BTreeMap::new();
        scorecard_metrics.insert("accuracy".to_string(), 0.9);
        scorecard_metrics.insert("macro_f1".to_string(), 0.88);

        let cost = NetworkSpikeCostPack::new()
            .evaluate(&observation(1000, 10, 200))
            .expect("evaluate");
        cost.merge_into(&mut scorecard_metrics).expect("merge");

        assert_eq!(scorecard_metrics["accuracy"], 0.9);
        assert_eq!(
            scorecard_metrics[NetworkSpikeCostPack::KEY_MEAN_SPIKES_PER_BURST],
            100.0
        );
        assert_eq!(scorecard_metrics.len(), 5);
    }

    #[test]
    fn merge_refuses_to_overwrite_existing_key() {
        let mut metrics = BTreeMap::new();
        metrics.insert(NetworkSpikeCostPack::KEY_TOTAL_SPIKES.to_string(), 1.0);
        let cost = NetworkSpikeCostPack::new()
            .evaluate(&observation(1000, 10, 200))
            .expect("evaluate");
        assert!(cost.merge_into(&mut metrics).is_err());
    }
}
