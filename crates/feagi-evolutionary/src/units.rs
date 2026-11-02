//! Units of stored genome values.

/// A neuron excitability as the fraction 0..=1 the NPU and the update API use.
///
/// Older genomes and tools stored it as a percent (100 meant "always fires"). Anything above 1 can
/// only be such a legacy percent, because the update path rejects out-of-range values, so it is
/// divided by 100 (and capped at 1). Negative values become 0; a value that is not a number
/// becomes the default, 1.
pub fn excitability_fraction(value: f64) -> f64 {
    if !value.is_finite() {
        return 1.0;
    }
    if value > 1.0 {
        return (value / 100.0).min(1.0);
    }
    value.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::excitability_fraction;

    #[test]
    fn fractions_are_kept_and_legacy_percent_is_converted() {
        assert_eq!(excitability_fraction(0.0), 0.0);
        assert_eq!(excitability_fraction(0.25), 0.25);
        assert_eq!(excitability_fraction(1.0), 1.0);
        assert_eq!(excitability_fraction(100.0), 1.0);
        assert!((excitability_fraction(25.0) - 0.25).abs() < 1e-12);
        assert_eq!(excitability_fraction(250.0), 1.0, "capped");
        assert_eq!(excitability_fraction(-3.0), 0.0);
        assert_eq!(excitability_fraction(f64::NAN), 1.0);
    }
}
