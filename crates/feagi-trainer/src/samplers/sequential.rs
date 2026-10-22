//! Sequential sampler — visits samples in their natural order.
//!
//! The simplest deterministic sampler: the plan is `0..sample_count` regardless of seed.
//! This is the benchmark-friendly baseline ordering.

use crate::contracts::common::{PluginId, PluginRef};
use crate::plugins::SamplerPlugin;

/// Visits samples in index order.
#[derive(Debug, Clone, Default)]
pub struct SequentialSampler;

impl SequentialSampler {
    /// Stable plugin id for this sampler.
    pub const PLUGIN_ID: &'static str = "sequential";

    /// Creates a new sequential sampler.
    pub fn new() -> Self {
        Self
    }
}

impl SamplerPlugin for SequentialSampler {
    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId(Self::PLUGIN_ID.to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn plan(&self, sample_count: usize, _seed: u64) -> Vec<usize> {
        (0..sample_count).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_is_natural_order() {
        let sampler = SequentialSampler::new();
        assert_eq!(sampler.plan(4, 7), vec![0, 1, 2, 3]);
    }

    #[test]
    fn plan_ignores_seed_deterministically() {
        let sampler = SequentialSampler::new();
        assert_eq!(sampler.plan(3, 1), sampler.plan(3, 999));
    }

    #[test]
    fn empty_split_yields_empty_plan() {
        assert!(SequentialSampler::new().plan(0, 0).is_empty());
    }
}
