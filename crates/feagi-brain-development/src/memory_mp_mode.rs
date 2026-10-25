// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Maps genome memory-area properties to the plasticity runtime MP mode.
//!
//! The genome keeps `mp_learning_enabled` and `mp_change_mode` as separate,
//! mutually exclusive properties; the runtime takes a single `MemoryMpMode`.

use std::collections::HashMap;

use feagi_evolutionary::{
    validate_memory_mp_properties, LtmEvictionMode, MemoryAreaProperties, MpChangeMode,
};
use feagi_npu_plasticity::{
    LtmEviction, MemoryMpMode, MemoryNeuronLifecycleConfig, MpChangeEncoding,
};
use serde_json::Value;

/// Resolve the runtime MP mode for a memory area.
///
/// `properties` are the area's raw properties and are re-validated so that an
/// invalid genome value is reported instead of being silently reinterpreted.
/// Temporal-depth eligibility is enforced later, at plasticity registration.
pub fn memory_mp_mode(
    properties: &HashMap<String, Value>,
    mem_props: &MemoryAreaProperties,
) -> Result<MemoryMpMode, String> {
    validate_memory_mp_properties(properties)?;
    Ok(match mem_props.mp_change_mode {
        MpChangeMode::None if mem_props.mp_learning_enabled => MemoryMpMode::MpLearning,
        MpChangeMode::None => MemoryMpMode::PatternOnly,
        MpChangeMode::Differential => MemoryMpMode::Change(MpChangeEncoding::Differential {
            quantization: mem_props.mp_delta_quantization,
        }),
        MpChangeMode::Ratio => MemoryMpMode::Change(MpChangeEncoding::Ratio {
            quantization_percent: mem_props.mp_ratio_quantization,
        }),
    })
}

/// Plasticity-runtime lifecycle settings for a memory area, from its genome properties.
///
/// Carries the per-area `memory_neuron_cap` and `ltm_eviction` along with the lifespan
/// settings, so every registration path (genome load, area update, rebind) applies them.
pub fn memory_lifecycle_config(mem_props: &MemoryAreaProperties) -> MemoryNeuronLifecycleConfig {
    MemoryNeuronLifecycleConfig {
        initial_lifespan: mem_props.init_lifespan,
        lifespan_growth_rate: mem_props.lifespan_growth_rate,
        longterm_threshold: mem_props.longterm_threshold,
        max_reactivations: 1000,
        memory_neuron_cap: mem_props.memory_neuron_cap,
        ltm_eviction: match mem_props.ltm_eviction {
            LtmEvictionMode::Lru => LtmEviction::Lru,
            LtmEvictionMode::None => LtmEviction::None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_evolutionary::extract_memory_properties;
    use serde_json::json;

    fn resolve(entries: &[(&str, Value)]) -> Result<MemoryMpMode, String> {
        let mut properties = HashMap::new();
        properties.insert("is_mem_type".to_string(), json!(true));
        for (k, v) in entries {
            properties.insert(k.to_string(), v.clone());
        }
        let mem_props = extract_memory_properties(&properties).unwrap();
        memory_mp_mode(&properties, &mem_props)
    }

    #[test]
    fn lifecycle_config_carries_cap_and_eviction() {
        let mut properties = HashMap::new();
        properties.insert("is_mem_type".to_string(), json!(true));
        let defaults = memory_lifecycle_config(&extract_memory_properties(&properties).unwrap());
        assert_eq!(defaults.memory_neuron_cap, 0);
        assert_eq!(defaults.ltm_eviction, LtmEviction::Lru);

        properties.insert("memory_neuron_cap".to_string(), json!(250));
        properties.insert("ltm_eviction".to_string(), json!("none"));
        let set = memory_lifecycle_config(&extract_memory_properties(&properties).unwrap());
        assert_eq!(set.memory_neuron_cap, 250);
        assert_eq!(set.ltm_eviction, LtmEviction::None);
    }

    #[test]
    fn defaults_to_pattern_only() {
        assert_eq!(resolve(&[]), Ok(MemoryMpMode::PatternOnly));
    }

    #[test]
    fn mp_learning_maps_to_mp_learning() {
        assert_eq!(
            resolve(&[("mp_learning_enabled", json!(true))]),
            Ok(MemoryMpMode::MpLearning)
        );
    }

    #[test]
    fn differential_uses_delta_quantization_with_default() {
        assert_eq!(
            resolve(&[("mp_change_mode", json!("mp_differential"))]),
            Ok(MemoryMpMode::Change(MpChangeEncoding::Differential {
                quantization: 1.0
            }))
        );
        assert_eq!(
            resolve(&[
                ("mp_change_mode", json!("mp_differential")),
                ("mp_delta_quantization", json!(0.25)),
            ]),
            Ok(MemoryMpMode::Change(MpChangeEncoding::Differential {
                quantization: 0.25
            }))
        );
    }

    #[test]
    fn ratio_uses_ratio_quantization_with_default() {
        assert_eq!(
            resolve(&[("mp_change_mode", json!("mp_ratio"))]),
            Ok(MemoryMpMode::Change(MpChangeEncoding::Ratio {
                quantization_percent: 20.0
            }))
        );
    }

    #[test]
    fn invalid_settings_are_errors() {
        assert!(resolve(&[("mp_change_mode", json!("bogus"))]).is_err());
        assert!(resolve(&[
            ("mp_change_mode", json!("mp_ratio")),
            ("mp_ratio_quantization", json!(0)),
        ])
        .is_err());
        assert!(resolve(&[
            ("mp_change_mode", json!("mp_ratio")),
            ("mp_learning_enabled", json!(true)),
        ])
        .is_err());
    }
}
