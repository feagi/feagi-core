// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Bulk, read-only per-neuron runtime state for one cortical area.
//!
//! A field is a plain name for one per-neuron quantity held by the NPU neuron storage (or by an
//! opt-in registry such as the rate-modulated leak homeostat). Every value is exposed as `f32`;
//! integer and boolean quantities are exact in that range.

/// One per-neuron quantity that can be sampled with [`crate::npu::RustNPU::read_neuron_state_columns`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeuronStateField {
    MembranePotential,
    Threshold,
    ThresholdLimit,
    /// The leak coefficient the LIF step uses (the homeostat writes its value into storage).
    LeakCoefficient,
    RestingPotential,
    NeuronType,
    RefractoryPeriod,
    RefractoryCountdown,
    Excitability,
    ConsecutiveFireCount,
    ConsecutiveFireLimit,
    SnoozePeriod,
    /// 1.0 when membrane-potential charge accumulation is enabled, else 0.0.
    MpChargeAccumulation,
    /// Firing-rate EMA from the rate-modulated-leak registry; absent for unregistered areas.
    RateEma,
}

impl NeuronStateField {
    /// Every field, in the order reported to clients.
    pub const ALL: [NeuronStateField; 14] = [
        NeuronStateField::MembranePotential,
        NeuronStateField::Threshold,
        NeuronStateField::ThresholdLimit,
        NeuronStateField::LeakCoefficient,
        NeuronStateField::RestingPotential,
        NeuronStateField::NeuronType,
        NeuronStateField::RefractoryPeriod,
        NeuronStateField::RefractoryCountdown,
        NeuronStateField::Excitability,
        NeuronStateField::ConsecutiveFireCount,
        NeuronStateField::ConsecutiveFireLimit,
        NeuronStateField::SnoozePeriod,
        NeuronStateField::MpChargeAccumulation,
        NeuronStateField::RateEma,
    ];

    /// Plain wire name.
    pub fn name(self) -> &'static str {
        match self {
            Self::MembranePotential => "membrane_potential",
            Self::Threshold => "threshold",
            Self::ThresholdLimit => "threshold_limit",
            Self::LeakCoefficient => "leak_coefficient",
            Self::RestingPotential => "resting_potential",
            Self::NeuronType => "neuron_type",
            Self::RefractoryPeriod => "refractory_period",
            Self::RefractoryCountdown => "refractory_countdown",
            Self::Excitability => "excitability",
            Self::ConsecutiveFireCount => "consecutive_fire_count",
            Self::ConsecutiveFireLimit => "consecutive_fire_limit",
            Self::SnoozePeriod => "snooze_period",
            Self::MpChargeAccumulation => "mp_charge_accumulation",
            Self::RateEma => "rate_ema",
        }
    }

    /// All wire names, in [`Self::ALL`] order.
    pub fn all_names() -> Vec<&'static str> {
        Self::ALL.iter().map(|f| f.name()).collect()
    }

    /// Parse a comma-separated field list. Empty or blank input selects every field.
    /// Duplicates are dropped (first occurrence wins). `Err` lists the unknown names.
    pub fn parse_list(list: Option<&str>) -> Result<Vec<NeuronStateField>, Vec<String>> {
        let Some(list) = list.map(str::trim).filter(|s| !s.is_empty()) else {
            return Ok(Self::ALL.to_vec());
        };
        let mut fields: Vec<NeuronStateField> = Vec::new();
        let mut unknown: Vec<String> = Vec::new();
        for raw in list.split(',') {
            let name = raw.trim();
            if name.is_empty() {
                continue;
            }
            match Self::ALL.iter().find(|f| f.name() == name) {
                Some(f) => {
                    if !fields.contains(f) {
                        fields.push(*f);
                    }
                }
                None => unknown.push(name.to_string()),
            }
        }
        if unknown.is_empty() {
            Ok(fields)
        } else {
            Err(unknown)
        }
    }
}

/// Columnar per-neuron state for one cortical area (dense neurons only).
#[derive(Debug, Clone, Default)]
pub struct NeuronStateColumns {
    pub x: Vec<u32>,
    pub y: Vec<u32>,
    pub z: Vec<u32>,
    /// One column per requested field, aligned with `x`/`y`/`z`. `None` = not available.
    pub fields: Vec<(NeuronStateField, Vec<Option<f32>>)>,
}

impl NeuronStateColumns {
    pub fn count(&self) -> usize {
        self.x.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_none_or_blank_selects_all() {
        assert_eq!(
            NeuronStateField::parse_list(None).unwrap().len(),
            NeuronStateField::ALL.len()
        );
        assert_eq!(
            NeuronStateField::parse_list(Some("  ")).unwrap().len(),
            NeuronStateField::ALL.len()
        );
    }

    #[test]
    fn parse_list_trims_and_dedups() {
        let f = NeuronStateField::parse_list(Some("threshold, rate_ema,threshold")).unwrap();
        assert_eq!(
            f,
            vec![NeuronStateField::Threshold, NeuronStateField::RateEma]
        );
    }

    #[test]
    fn parse_rejects_unknown_names() {
        let err = NeuronStateField::parse_list(Some("threshold,bogus,other")).unwrap_err();
        assert_eq!(err, vec!["bogus".to_string(), "other".to_string()]);
    }

    #[test]
    fn names_round_trip() {
        for f in NeuronStateField::ALL {
            assert_eq!(
                NeuronStateField::parse_list(Some(f.name())).unwrap(),
                vec![f]
            );
        }
    }
}
