//! Multi-seed repeat aggregation — publication-credible scoring over N independent rollouts.
//!
//! A single rollout yields point-estimate metrics. Because the connectome is pinned and the
//! FEAGI runtime is deterministic by construction (ADR-003), the only honest way to obtain a
//! distribution is to vary the *sample ordering*: reward-modulated plasticity makes the learning
//! trajectory order-dependent, so re-planning the sampler order with a derived seed and re-running
//! the full train+eval produces genuinely different final brains and therefore a spread of
//! scores. This module turns those per-repeat metric maps into per-metric
//! [`MetricStat`](crate::contracts::MetricStat)s (mean / sample stddev / Student's-t confidence
//! interval).
//!
//! Everything here is pure and deterministic: given the same per-repeat inputs and confidence
//! level it returns the same statistics, with no I/O and no wall-clock reads.

use std::collections::BTreeMap;

use distrs::StudentsT;
use serde::{Deserialize, Serialize};

use crate::contracts::MetricStat;
use crate::error::TrainerError;

/// Configuration for an N-seed repeated run.
///
/// `repeats` is the number of independent rollouts (the `N`); `confidence_level` is the
/// two-sided interval coverage (e.g. `0.95`). Both are supplied by the caller (resolved from run
/// configuration) rather than defaulted in logic, so no credibility-bearing value is hardcoded.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RepeatConfig {
    /// Number of independent repeats; must be `>= 1`.
    pub repeats: u32,
    /// Two-sided confidence level in the open interval `(0, 1)`.
    pub confidence_level: f64,
}

impl RepeatConfig {
    /// Validates the configuration, returning an explicit error rather than coercing.
    ///
    /// # Errors
    /// [`TrainerError::Config`] when `repeats == 0` or `confidence_level` is not a finite value
    /// strictly inside `(0, 1)`.
    pub fn validate(&self) -> Result<(), TrainerError> {
        if self.repeats == 0 {
            return Err(TrainerError::Config(
                "RepeatConfig.repeats must be >= 1".to_string(),
            ));
        }
        validate_confidence_level(self.confidence_level)
    }
}

/// The artifacts of an N-seed repeated run: the raw per-repeat metrics, the per-metric mean
/// (the headline point estimate), and the per-metric distribution.
#[derive(Debug, Clone, PartialEq)]
pub struct RepeatedRolloutOutcome {
    /// One metric map per repeat, in repeat order (repeat `r` used the `r`-th derived seed).
    pub per_repeat_metrics: Vec<BTreeMap<String, f64>>,
    /// Per-metric mean across repeats — the value stamped into `Scorecard.metrics`.
    pub mean_metrics: BTreeMap<String, f64>,
    /// Per-metric distribution (mean / stddev / confidence interval) — stamped into
    /// `Scorecard.metric_stats`.
    pub metric_stats: BTreeMap<String, MetricStat>,
}

/// Derives the seed for repeat index `r` from the run's base seed.
///
/// Repeats walk consecutive seeds (`base`, `base + 1`, …) so the whole sweep is reproducible
/// from the single base seed recorded in the run spec (plan Section 9). Uses wrapping addition so
/// a base near `u64::MAX` cannot panic.
pub fn repeat_seed(base_seed: u64, repeat_index: u32) -> u64 {
    base_seed.wrapping_add(repeat_index as u64)
}

/// Runs `config.repeats` independent rollouts via `run_once` and aggregates their metrics.
///
/// `run_once(seed)` must execute one full train+eval rollout against a freshly-reset brain
/// (the caller is responsible for restoring the pinned connectome between repeats) using `seed`
/// to plan the sampler order, and return that rollout's metric map. This indirection keeps the
/// orchestrator transport-agnostic: it works identically over the deterministic stub and a live
/// remote runtime.
///
/// # Errors
/// Propagates the first error from `config.validate()` or any `run_once` call, and any
/// aggregation error (see [`aggregate_metric_stats`]).
pub fn run_repeated<F>(
    config: &RepeatConfig,
    base_seed: u64,
    mut run_once: F,
) -> Result<RepeatedRolloutOutcome, TrainerError>
where
    F: FnMut(u64) -> Result<BTreeMap<String, f64>, TrainerError>,
{
    config.validate()?;

    let mut per_repeat_metrics = Vec::with_capacity(config.repeats as usize);
    for r in 0..config.repeats {
        let metrics = run_once(repeat_seed(base_seed, r))?;
        per_repeat_metrics.push(metrics);
    }

    let metric_stats = aggregate_metric_stats(&per_repeat_metrics, config.confidence_level)?;
    let mean_metrics = metric_stats
        .iter()
        .map(|(name, stat)| (name.clone(), stat.mean))
        .collect();

    Ok(RepeatedRolloutOutcome {
        per_repeat_metrics,
        mean_metrics,
        metric_stats,
    })
}

/// Aggregates per-repeat metric maps into per-metric [`MetricStat`]s.
///
/// Every repeat must report the **same** metric keys (a differing key set is a programming error,
/// not a recoverable condition) and every value must be finite. For `n == 1` the standard
/// deviation is `0.0` and the confidence interval collapses to the point estimate (a single
/// sample carries no spread). For `n > 1` the interval is `mean ± t * stddev / sqrt(n)` with the
/// Student's-t critical value at `df = n - 1`.
///
/// # Errors
/// [`TrainerError::Evaluation`] on empty input, a metric-key mismatch across repeats, or a
/// non-finite metric value; [`TrainerError::Config`] on an invalid `confidence_level`.
pub fn aggregate_metric_stats(
    per_repeat: &[BTreeMap<String, f64>],
    confidence_level: f64,
) -> Result<BTreeMap<String, MetricStat>, TrainerError> {
    validate_confidence_level(confidence_level)?;

    let n = per_repeat.len();
    if n == 0 {
        return Err(TrainerError::Evaluation(
            "cannot aggregate metric stats over zero repeats".to_string(),
        ));
    }

    // The first repeat fixes the expected metric-key set; every other repeat must match exactly.
    let keys: &BTreeMap<String, f64> = &per_repeat[0];
    for (i, repeat) in per_repeat.iter().enumerate().skip(1) {
        if repeat.len() != keys.len() || !repeat.keys().all(|k| keys.contains_key(k)) {
            return Err(TrainerError::Evaluation(format!(
                "metric keys differ across repeats: repeat 0 has {:?}, repeat {i} has {:?}",
                keys.keys().collect::<Vec<_>>(),
                repeat.keys().collect::<Vec<_>>()
            )));
        }
    }

    let mut stats = BTreeMap::new();
    for name in keys.keys() {
        let mut values = Vec::with_capacity(n);
        for repeat in per_repeat {
            // Key presence is guaranteed by the equality check above.
            let value = repeat[name];
            if !value.is_finite() {
                return Err(TrainerError::Evaluation(format!(
                    "metric '{name}' has a non-finite value ({value})"
                )));
            }
            values.push(value);
        }
        stats.insert(name.clone(), metric_stat(&values, confidence_level));
    }

    Ok(stats)
}

/// Computes a single [`MetricStat`] from a non-empty slice of finite values.
fn metric_stat(values: &[f64], confidence_level: f64) -> MetricStat {
    let n = values.len();
    let mean = values.iter().sum::<f64>() / n as f64;

    if n == 1 {
        return MetricStat {
            n: 1,
            mean,
            stddev: 0.0,
            ci_low: mean,
            ci_high: mean,
            confidence_level,
        };
    }

    let degrees_of_freedom = (n - 1) as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / degrees_of_freedom;
    let stddev = variance.sqrt();
    let standard_error = stddev / (n as f64).sqrt();

    // Two-sided interval: the upper-tail probability is (1 + cl) / 2.
    let t_critical = StudentsT::ppf((1.0 + confidence_level) / 2.0, degrees_of_freedom);
    let margin = t_critical * standard_error;

    MetricStat {
        n: n as u32,
        mean,
        stddev,
        ci_low: mean - margin,
        ci_high: mean + margin,
        confidence_level,
    }
}

/// Validates a confidence level is a finite value strictly inside `(0, 1)`.
fn validate_confidence_level(confidence_level: f64) -> Result<(), TrainerError> {
    if !confidence_level.is_finite() || confidence_level <= 0.0 || confidence_level >= 1.0 {
        return Err(TrainerError::Config(format!(
            "confidence_level must be a finite value in (0, 1), got {confidence_level}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn single_repeat_has_zero_spread_and_point_interval() {
        let stats = aggregate_metric_stats(&[map(&[("accuracy", 0.9)])], 0.95).unwrap();
        let stat = &stats["accuracy"];
        assert_eq!(stat.n, 1);
        assert_eq!(stat.mean, 0.9);
        assert_eq!(stat.stddev, 0.0);
        assert_eq!(stat.ci_low, 0.9);
        assert_eq!(stat.ci_high, 0.9);
    }

    #[test]
    fn mean_and_sample_stddev_are_correct() {
        // values 0.8, 0.9, 1.0 -> mean 0.9, sample variance 0.01, stddev 0.1.
        let per_repeat = vec![
            map(&[("accuracy", 0.8)]),
            map(&[("accuracy", 0.9)]),
            map(&[("accuracy", 1.0)]),
        ];
        let stats = aggregate_metric_stats(&per_repeat, 0.95).unwrap();
        let stat = &stats["accuracy"];
        assert_eq!(stat.n, 3);
        assert!((stat.mean - 0.9).abs() < 1e-12);
        assert!((stat.stddev - 0.1).abs() < 1e-12);
        // Interval is symmetric about the mean and strictly contains it for n > 1.
        assert!(stat.ci_low < stat.mean && stat.mean < stat.ci_high);
        assert!(((stat.ci_low + stat.ci_high) / 2.0 - stat.mean).abs() < 1e-12);
    }

    #[test]
    fn confidence_interval_matches_known_t_value() {
        // n = 5, mean 0.9, known sample stddev; check the 95% half-width against the t table.
        let per_repeat = vec![
            map(&[("m", 0.85)]),
            map(&[("m", 0.88)]),
            map(&[("m", 0.90)]),
            map(&[("m", 0.92)]),
            map(&[("m", 0.95)]),
        ];
        let stats = aggregate_metric_stats(&per_repeat, 0.95).unwrap();
        let stat = &stats["m"];
        // t(0.975, df=4) = 2.776; half-width = 2.776 * stddev / sqrt(5).
        let expected_half = 2.776_445 * stat.stddev / 5f64.sqrt();
        assert!(((stat.ci_high - stat.ci_low) / 2.0 - expected_half).abs() < 1e-3);
    }

    #[test]
    fn empty_input_is_an_error() {
        assert!(matches!(
            aggregate_metric_stats(&[], 0.95),
            Err(TrainerError::Evaluation(_))
        ));
    }

    #[test]
    fn mismatched_metric_keys_are_an_error() {
        let per_repeat = vec![map(&[("a", 1.0)]), map(&[("b", 1.0)])];
        assert!(matches!(
            aggregate_metric_stats(&per_repeat, 0.95),
            Err(TrainerError::Evaluation(_))
        ));
    }

    #[test]
    fn non_finite_value_is_an_error() {
        let per_repeat = vec![map(&[("a", 1.0)]), map(&[("a", f64::NAN)])];
        assert!(matches!(
            aggregate_metric_stats(&per_repeat, 0.95),
            Err(TrainerError::Evaluation(_))
        ));
    }

    #[test]
    fn invalid_confidence_level_is_an_error() {
        let per_repeat = vec![map(&[("a", 1.0)])];
        for bad in [0.0, 1.0, -0.5, 1.5, f64::NAN] {
            assert!(matches!(
                aggregate_metric_stats(&per_repeat, bad),
                Err(TrainerError::Config(_))
            ));
        }
    }

    #[test]
    fn repeat_seed_walks_consecutive_seeds() {
        assert_eq!(repeat_seed(100, 0), 100);
        assert_eq!(repeat_seed(100, 3), 103);
        // Wraps rather than panicking near the top of the range.
        assert_eq!(repeat_seed(u64::MAX, 1), 0);
    }

    #[test]
    fn run_repeated_invokes_each_seed_and_aggregates() {
        let mut seen_seeds = Vec::new();
        let config = RepeatConfig {
            repeats: 3,
            confidence_level: 0.95,
        };
        let outcome = run_repeated(&config, 10, |seed| {
            seen_seeds.push(seed);
            // Metric value tracks the seed so we can assert the mean deterministically.
            Ok(map(&[("score", seed as f64)]))
        })
        .unwrap();

        assert_eq!(seen_seeds, vec![10, 11, 12]);
        assert_eq!(outcome.per_repeat_metrics.len(), 3);
        // mean of 10, 11, 12 = 11.
        assert!((outcome.mean_metrics["score"] - 11.0).abs() < 1e-12);
        assert_eq!(outcome.metric_stats["score"].n, 3);
    }

    #[test]
    fn run_repeated_is_deterministic() {
        let config = RepeatConfig {
            repeats: 4,
            confidence_level: 0.9,
        };
        let run =
            || run_repeated(&config, 7, |seed| Ok(map(&[("score", (seed % 5) as f64)]))).unwrap();
        assert_eq!(run(), run());
    }

    #[test]
    fn run_repeated_rejects_zero_repeats() {
        let config = RepeatConfig {
            repeats: 0,
            confidence_level: 0.95,
        };
        let result = run_repeated(&config, 0, |_| Ok(map(&[("score", 1.0)])));
        assert!(matches!(result, Err(TrainerError::Config(_))));
    }

    #[test]
    fn run_repeated_propagates_run_once_error() {
        let config = RepeatConfig {
            repeats: 2,
            confidence_level: 0.95,
        };
        let result = run_repeated(&config, 0, |_| {
            Err(TrainerError::Runtime("boom".to_string()))
        });
        assert!(matches!(result, Err(TrainerError::Runtime(_))));
    }
}
