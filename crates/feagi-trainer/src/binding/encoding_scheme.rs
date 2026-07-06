//! Encoding-scheme registry (design Appendix E).
//!
//! Encoding *scheme* (how a value becomes spikes) is a dimension distinct from data *type*.
//! FEAGI exposes population/positional coding today; rate/temporal/value are registered here
//! so a researcher can select them, but resolving an unavailable scheme is an explicit error
//! (no silent fallback to positional coding). When core adds those coders, only `resolve`
//! changes — the contract and selection UX stay stable.

use serde::{Deserialize, Serialize};

use crate::error::TrainerError;

/// Bin spacing along the cortical_area column for population coding.
///
/// Maps to FEAGI's `PercentageNeuronPositioning` at selection time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinSpacing {
    /// Uniformly spaced bins.
    Linear,
    /// Exponentially spaced bins (denser near zero).
    Fractional,
}

/// A neural encoding scheme selectable per run.
///
/// Tagged by `scheme` on the wire so it is self-describing in provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "scheme", rename_all = "snake_case")]
pub enum EncodingScheme {
    /// Single spike at the bin matching the value (population/positional coding). Available.
    PopulationSingleSpike {
        /// Number of bins along the cortical_area column (the `NeuronDepth`).
        bins: u32,
        /// Bin spacing.
        spacing: BinSpacing,
    },
    /// Value -> firing rate over a tick window. Registered; not yet available (Appendix E).
    Rate {
        /// Number of ticks the rate is integrated over.
        window_ticks: u32,
        /// Maximum spike count mapped to the value's upper bound.
        max_rate: u32,
    },
    /// Value -> spike latency within a tick window. Registered; not yet available.
    Temporal {
        /// Number of ticks the latency is measured within.
        window_ticks: u32,
    },
    /// Value encoded as graded potential. Registered; not yet available.
    Value,
}

/// A resolved, available population-coding configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedEncodingScheme {
    /// Number of bins (the cortical_area column `NeuronDepth`).
    pub bins: u32,
    /// Bin spacing.
    pub spacing: BinSpacing,
}

impl EncodingScheme {
    /// Stable wire name of the scheme variant.
    pub fn name(&self) -> &'static str {
        match self {
            EncodingScheme::PopulationSingleSpike { .. } => "population_single_spike",
            EncodingScheme::Rate { .. } => "rate",
            EncodingScheme::Temporal { .. } => "temporal",
            EncodingScheme::Value => "value",
        }
    }

    /// Whether this scheme is backed by a FEAGI coder today.
    pub fn is_available(&self) -> bool {
        matches!(self, EncodingScheme::PopulationSingleSpike { .. })
    }

    /// Resolves the scheme to a concrete configuration, or fails explicitly.
    ///
    /// Returns [`TrainerError::Config`] for a scheme that has no backing FEAGI coder yet, or
    /// for invalid parameters (e.g. zero bins). Never falls back to a different scheme.
    pub fn resolve(&self) -> Result<ResolvedEncodingScheme, TrainerError> {
        match self {
            EncodingScheme::PopulationSingleSpike { bins, spacing } => {
                if *bins == 0 {
                    return Err(TrainerError::Config(
                        "population_single_spike requires bins > 0".to_string(),
                    ));
                }
                Ok(ResolvedEncodingScheme {
                    bins: *bins,
                    spacing: *spacing,
                })
            }
            other => Err(TrainerError::Config(format!(
                "encoding scheme '{}' is registered but not yet available in feagi-sensorimotor \
                 (see design Appendix E); select 'population_single_spike' or add the coder in core",
                other.name()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn population_resolves() {
        let scheme = EncodingScheme::PopulationSingleSpike {
            bins: 8,
            spacing: BinSpacing::Linear,
        };
        assert!(scheme.is_available());
        let resolved = scheme.resolve().expect("resolve");
        assert_eq!(resolved.bins, 8);
        assert_eq!(resolved.spacing, BinSpacing::Linear);
    }

    #[test]
    fn zero_bins_is_error() {
        let scheme = EncodingScheme::PopulationSingleSpike {
            bins: 0,
            spacing: BinSpacing::Linear,
        };
        assert!(scheme.resolve().is_err());
    }

    #[test]
    fn unavailable_schemes_error_explicitly() {
        for scheme in [
            EncodingScheme::Rate {
                window_ticks: 10,
                max_rate: 5,
            },
            EncodingScheme::Temporal { window_ticks: 10 },
            EncodingScheme::Value,
        ] {
            assert!(!scheme.is_available());
            assert!(matches!(scheme.resolve(), Err(TrainerError::Config(_))));
        }
    }

    #[test]
    fn scheme_is_self_describing_on_wire() {
        let scheme = EncodingScheme::PopulationSingleSpike {
            bins: 8,
            spacing: BinSpacing::Fractional,
        };
        let json = serde_json::to_value(&scheme).expect("serialize");
        assert_eq!(json["scheme"], serde_json::json!("population_single_spike"));
        let restored: EncodingScheme = serde_json::from_value(json).expect("deserialize");
        assert_eq!(restored, scheme);
    }
}
