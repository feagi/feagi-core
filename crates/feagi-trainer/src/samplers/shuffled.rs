//! Shuffled sampler — visits every sample once, in a seeded random order.
//!
//! The plan is a permutation of `0..sample_count` fixed by the seed: the same seed gives the
//! same order, different seeds give different orders. It shares the SplitMix64 generator with
//! the capped-split draw in `adapters::sample_draw`.

use crate::adapters::sample_draw::shuffled_order;
use crate::contracts::common::{PluginId, PluginRef};
use crate::plugins::SamplerPlugin;

/// Visits samples in a seeded permutation.
#[derive(Debug, Clone, Default)]
pub struct ShuffledSampler;

impl ShuffledSampler {
    /// Stable plugin id for this sampler.
    pub const PLUGIN_ID: &'static str = "shuffled";

    /// Creates a new shuffled sampler.
    pub fn new() -> Self {
        Self
    }
}

impl SamplerPlugin for ShuffledSampler {
    fn plugin_ref(&self) -> PluginRef {
        PluginRef {
            id: PluginId(Self::PLUGIN_ID.to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn plan(&self, sample_count: usize, seed: u64) -> Vec<usize> {
        shuffled_order(sample_count, seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vector_for_fixed_seed() {
        // Pinned: a change here changes every shuffled run's order.
        assert_eq!(ShuffledSampler::new().plan(8, 42), PINNED_8_SEED_42);
    }

    #[test]
    fn plan_is_a_valid_permutation() {
        for n in [0usize, 1, 2, 3, 7, 50, 257] {
            for seed in [0u64, 1, 42, u64::MAX] {
                let mut plan = ShuffledSampler::new().plan(n, seed);
                plan.sort_unstable();
                assert_eq!(plan, (0..n).collect::<Vec<_>>(), "n={n} seed={seed}");
            }
        }
    }

    #[test]
    fn same_seed_same_order() {
        let sampler = ShuffledSampler::new();
        assert_eq!(sampler.plan(100, 9), sampler.plan(100, 9));
    }

    #[test]
    fn different_seeds_differ() {
        let sampler = ShuffledSampler::new();
        assert_ne!(sampler.plan(100, 1), sampler.plan(100, 2));
    }

    const PINNED_8_SEED_42: [usize; 8] = [5, 6, 2, 7, 1, 0, 3, 4];
}
