//! `MetricStat` v1 — the distribution of a single metric across N-seed repeats.
//!
//! Publication-credible scoring reports a metric not as a single number but as a distribution
//! over independent repeats (ADR-012 verification-by-re-run, extended to N seeds). Each repeat
//! re-plans the sampler order with a derived seed and runs the full train+eval against the same
//! pinned connectome; because reward-modulated plasticity makes the learning trajectory
//! order-dependent (ADR-003: the runtime itself is deterministic), the spread across repeats is
//! the genuine run-to-run variance a journal reviewer expects to see quantified.
//!
//! A `MetricStat` is descriptive provenance: it carries the sample size `n`, the mean, the
//! sample standard deviation, and a two-sided confidence interval at `confidence_level`. The
//! interval uses the Student's t-distribution (correct for the small `n` typical of seed
//! repeats); for `n == 1` the interval collapses to the point estimate.

use serde::{Deserialize, Serialize};

/// The distribution of one metric across N-seed repeats (deterministically ordered by the
/// containing map's key).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricStat {
    /// Number of repeats the statistic was computed over (the `N` in "N-seed repeats").
    pub n: u32,
    /// Arithmetic mean of the metric across repeats — the headline point estimate.
    pub mean: f64,
    /// Sample standard deviation (Bessel-corrected, `n - 1` denominator); `0.0` when `n == 1`.
    pub stddev: f64,
    /// Lower bound of the two-sided confidence interval at [`confidence_level`](Self::confidence_level).
    pub ci_low: f64,
    /// Upper bound of the two-sided confidence interval at [`confidence_level`](Self::confidence_level).
    pub ci_high: f64,
    /// Confidence level the interval was computed at, in the open interval `(0, 1)` (e.g. `0.95`).
    pub confidence_level: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trip_preserves_metric_stat() {
        let stat = MetricStat {
            n: 5,
            mean: 0.9,
            stddev: 0.05,
            ci_low: 0.84,
            ci_high: 0.96,
            confidence_level: 0.95,
        };
        let json = serde_json::to_string(&stat).expect("serialize");
        let restored: MetricStat = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(stat, restored);
    }
}
