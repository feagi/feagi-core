//! Sampler axis — deterministic ordering/scheduling over a dataset split (design Section 5.5).
//!
//! Benchmark mode requires deterministic sampling, so the plan is a pure function of the
//! sample count and the seed.

use crate::contracts::common::PluginRef;

/// Produces a deterministic visitation order over a split's samples.
pub trait SamplerPlugin {
    /// Identifies this sampler (axis provenance).
    fn plugin_ref(&self) -> PluginRef;

    /// Plans the visitation order as indices into the split's sample list.
    ///
    /// Must be a deterministic function of `(sample_count, seed)`. The returned vector is a
    /// permutation of `0..sample_count`.
    fn plan(&self, sample_count: usize, seed: u64) -> Vec<usize>;
}
