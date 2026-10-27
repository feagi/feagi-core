//! Where a reward signal goes in the brain.
//!
//! FEAGI learns from reward through `synaptic.reward` modulators: a modulator acts when its
//! driver area fires, and the sign of its `magnitude_percent` says reward (+) or punishment (-).
//! A run may name a list of those modulators. [`RewardTargets::resolve`] reads the genome's
//! modulator listing (`GET /v1/modulator/modulators`, `{instance_id: {type, magnitude_percent,
//! driver_cortical_id, ...}}`) once at run start and splits the listed drivers by sign:
//!
//! - [`AffectChannel::Pleasure`] and [`AffectChannel::Hope`] stimulate the drivers of listed
//!   modulators with a positive magnitude;
//! - [`AffectChannel::Pain`] and [`AffectChannel::Fear`] stimulate the drivers of listed
//!   modulators with a negative magnitude.
//!
//! An empty list keeps the older behavior: each channel stimulates its core affect area
//! (Pain, Pleasure, Fear, Hope), so experiments saved before the list existed run unchanged.

use serde_json::Value;

use feagi_structures::genomic::cortical_area::{CoreCorticalType, CorticalID};

use crate::binding::reward::AffectChannel;
use crate::error::TrainerError;

/// Modulator type that carries reward and punishment.
pub const REWARD_MODULATOR_TYPE: &str = "synaptic.reward";

/// One listed reward modulator, resolved against the genome.
#[derive(Debug, Clone, PartialEq)]
pub struct RewardModulatorTarget {
    /// Modulator instance id as the genome names it.
    pub modulator_id: String,
    /// Signed magnitude from the genome; its sign picks the channel group.
    pub magnitude_percent: f64,
    /// Area whose firing triggers the modulator.
    pub driver_cortical_id: CorticalID,
}

impl RewardModulatorTarget {
    /// True for a reward (positive magnitude), false for a punishment.
    pub fn is_reward(&self) -> bool {
        self.magnitude_percent > 0.0
    }
}

/// The areas each affect channel stimulates for one run.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RewardTargets {
    modulators: Vec<RewardModulatorTarget>,
}

impl RewardTargets {
    /// Core affect areas (the behavior when no reward modulators are listed).
    pub fn core_affect() -> Self {
        Self::default()
    }

    /// Resolve `modulator_ids` against a modulator listing. An empty list yields
    /// [`RewardTargets::core_affect`].
    ///
    /// Rejects, with [`TrainerError::Config`]: a blank or repeated id, an id the listing does
    /// not have, a modulator whose type is not `synaptic.reward`, a zero or non-finite
    /// magnitude, an unreadable driver id, and a listed driver that also drives a reward
    /// modulator of the opposite sign (stimulating it would reward and punish at once).
    pub fn resolve(modulator_ids: &[String], listing: &Value) -> Result<Self, TrainerError> {
        if modulator_ids.is_empty() {
            return Ok(Self::core_affect());
        }
        let entries = listing.as_object().ok_or_else(|| {
            TrainerError::Config("modulator listing is not a JSON object".to_string())
        })?;

        let mut modulators: Vec<RewardModulatorTarget> = Vec::with_capacity(modulator_ids.len());
        for raw_id in modulator_ids {
            let id = raw_id.trim();
            if id.is_empty() {
                return Err(TrainerError::Config(
                    "reward_modulator_ids contains a blank id".to_string(),
                ));
            }
            if modulators.iter().any(|m| m.modulator_id == id) {
                return Err(TrainerError::Config(format!(
                    "reward_modulator_ids lists '{id}' more than once"
                )));
            }
            let entry = entries.get(id).ok_or_else(|| {
                TrainerError::Config(format!(
                    "reward modulator '{id}' is not in the genome's modulators"
                ))
            })?;
            modulators.push(parse_reward_modulator(id, entry)?);
        }

        // Firing a driver triggers every modulator it drives, listed or not.
        for (other_id, other) in entries {
            if other.get("type").and_then(Value::as_str) != Some(REWARD_MODULATOR_TYPE) {
                continue;
            }
            let Ok(other) = parse_reward_modulator(other_id, other) else {
                continue;
            };
            if let Some(listed) = modulators.iter().find(|m| {
                m.driver_cortical_id == other.driver_cortical_id
                    && m.is_reward() != other.is_reward()
            }) {
                return Err(TrainerError::Config(format!(
                    "reward modulator '{}' shares its driver area with '{}', which has the opposite sign; stimulating it would reward and punish at once",
                    listed.modulator_id, other.modulator_id
                )));
            }
        }

        Ok(Self { modulators })
    }

    /// True when no reward modulators are listed and the core affect areas are used.
    pub fn is_core_affect(&self) -> bool {
        self.modulators.is_empty()
    }

    /// The listed modulators, in the order they were given.
    pub fn modulators(&self) -> &[RewardModulatorTarget] {
        &self.modulators
    }

    /// Areas one signal on `channel` stimulates, without repeats. With listed modulators, a
    /// channel whose sign has no listed modulator stimulates nothing.
    pub fn cortical_ids_for(&self, channel: AffectChannel) -> Vec<CorticalID> {
        if self.is_core_affect() {
            return vec![core_affect_cortical_id(channel)];
        }
        let wants_reward = matches!(channel, AffectChannel::Pleasure | AffectChannel::Hope);
        let mut ids: Vec<CorticalID> = Vec::new();
        for modulator in &self.modulators {
            if modulator.is_reward() == wants_reward && !ids.contains(&modulator.driver_cortical_id)
            {
                ids.push(modulator.driver_cortical_id);
            }
        }
        ids
    }
}

/// Core affect area a channel stimulates when no reward modulators are listed.
pub fn core_affect_cortical_id(channel: AffectChannel) -> CorticalID {
    let core = match channel {
        AffectChannel::Pain => CoreCorticalType::Pain,
        AffectChannel::Pleasure => CoreCorticalType::Pleasure,
        AffectChannel::Fear => CoreCorticalType::Fear,
        AffectChannel::Hope => CoreCorticalType::Hope,
    };
    core.to_cortical_id()
}

fn parse_reward_modulator(id: &str, entry: &Value) -> Result<RewardModulatorTarget, TrainerError> {
    let kind = entry.get("type").and_then(Value::as_str).unwrap_or("");
    if kind != REWARD_MODULATOR_TYPE {
        return Err(TrainerError::Config(format!(
            "modulator '{id}' has type '{kind}'; reward needs a '{REWARD_MODULATOR_TYPE}' modulator"
        )));
    }
    let magnitude_percent = entry
        .get("magnitude_percent")
        .and_then(Value::as_f64)
        .filter(|m| m.is_finite() && *m != 0.0)
        .ok_or_else(|| {
            TrainerError::Config(format!(
                "reward modulator '{id}' needs a non-zero magnitude_percent; its sign says reward or punishment"
            ))
        })?;
    let driver = entry
        .get("driver_cortical_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            TrainerError::Config(format!("reward modulator '{id}' has no driver_cortical_id"))
        })?;
    let driver_cortical_id = CorticalID::try_from_base_64(driver).map_err(|e| {
        TrainerError::Config(format!(
            "reward modulator '{id}' has an unreadable driver_cortical_id '{driver}': {e}"
        ))
    })?;
    Ok(RewardModulatorTarget {
        modulator_id: id.to_string(),
        magnitude_percent,
        driver_cortical_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Distinct valid driver ids, borrowed from core areas so the bytes are well formed.
    fn driver(n: usize) -> CorticalID {
        [
            CoreCorticalType::Death,
            CoreCorticalType::Power,
            CoreCorticalType::Fatigue,
        ][n]
            .to_cortical_id()
    }

    fn entry(kind: &str, magnitude: f64, driver_id: CorticalID) -> Value {
        json!({
            "type": kind,
            "magnitude_percent": magnitude,
            "effect_duration_bursts": 1,
            "rest_bursts": 0,
            "graded": false,
            "driver_cortical_id": driver_id.as_base_64(),
        })
    }

    fn listing() -> Value {
        json!({
            "reward_a": entry(REWARD_MODULATOR_TYPE, 40.0, driver(0)),
            "punish_b": entry(REWARD_MODULATOR_TYPE, -25.0, driver(1)),
            "memory_c": entry("synaptic.memory_formation", 10.0, driver(2)),
        })
    }

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn empty_list_keeps_core_affect_areas() {
        let targets = RewardTargets::resolve(&[], &listing()).unwrap();
        assert!(targets.is_core_affect());
        for channel in [
            AffectChannel::Pain,
            AffectChannel::Pleasure,
            AffectChannel::Fear,
            AffectChannel::Hope,
        ] {
            assert_eq!(
                targets.cortical_ids_for(channel),
                vec![core_affect_cortical_id(channel)]
            );
        }
    }

    #[test]
    fn sign_picks_the_channel() {
        let targets = RewardTargets::resolve(&ids(&["reward_a", "punish_b"]), &listing()).unwrap();
        assert_eq!(
            targets.cortical_ids_for(AffectChannel::Pleasure),
            vec![driver(0)]
        );
        assert_eq!(
            targets.cortical_ids_for(AffectChannel::Hope),
            vec![driver(0)]
        );
        assert_eq!(
            targets.cortical_ids_for(AffectChannel::Pain),
            vec![driver(1)]
        );
        assert_eq!(
            targets.cortical_ids_for(AffectChannel::Fear),
            vec![driver(1)]
        );
    }

    #[test]
    fn a_sign_without_listed_modulators_stimulates_nothing() {
        let targets = RewardTargets::resolve(&ids(&["reward_a"]), &listing()).unwrap();
        assert!(targets.cortical_ids_for(AffectChannel::Pain).is_empty());
        assert_eq!(
            targets.cortical_ids_for(AffectChannel::Pleasure),
            vec![driver(0)]
        );
    }

    #[test]
    fn shared_driver_is_stimulated_once() {
        let listing = json!({
            "reward_a": entry(REWARD_MODULATOR_TYPE, 40.0, driver(0)),
            "reward_b": entry(REWARD_MODULATOR_TYPE, 10.0, driver(0)),
        });
        let targets = RewardTargets::resolve(&ids(&["reward_a", "reward_b"]), &listing).unwrap();
        assert_eq!(
            targets.cortical_ids_for(AffectChannel::Pleasure),
            vec![driver(0)]
        );
    }

    #[test]
    fn rejects_a_modulator_that_is_not_reward() {
        let err = RewardTargets::resolve(&ids(&["memory_c"]), &listing()).unwrap_err();
        assert!(err.to_string().contains("synaptic.reward"), "{err}");
    }

    #[test]
    fn rejects_an_unknown_id() {
        let err = RewardTargets::resolve(&ids(&["missing"]), &listing()).unwrap_err();
        assert!(err.to_string().contains("'missing'"), "{err}");
    }

    #[test]
    fn rejects_blank_and_repeated_ids() {
        assert!(RewardTargets::resolve(&ids(&[" "]), &listing()).is_err());
        assert!(RewardTargets::resolve(&ids(&["reward_a", "reward_a"]), &listing()).is_err());
    }

    #[test]
    fn rejects_zero_magnitude() {
        let listing = json!({ "flat": entry(REWARD_MODULATOR_TYPE, 0.0, driver(0)) });
        let err = RewardTargets::resolve(&ids(&["flat"]), &listing).unwrap_err();
        assert!(err.to_string().contains("non-zero"), "{err}");
    }

    #[test]
    fn rejects_a_driver_shared_with_the_opposite_sign() {
        let listing = json!({
            "reward_a": entry(REWARD_MODULATOR_TYPE, 40.0, driver(0)),
            "punish_b": entry(REWARD_MODULATOR_TYPE, -40.0, driver(0)),
        });
        let err = RewardTargets::resolve(&ids(&["reward_a"]), &listing).unwrap_err();
        assert!(err.to_string().contains("opposite sign"), "{err}");
    }
}
