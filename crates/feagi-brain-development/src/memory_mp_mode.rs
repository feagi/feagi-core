// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Maps genome memory-area properties to the plasticity runtime MP mode.
//!
//! The genome keeps `mp_learning_enabled` and `mp_change_mode` as separate,
//! mutually exclusive properties; the runtime takes a single `MemoryMpMode`.

use std::collections::HashMap;

use feagi_evolutionary::{validate_memory_mp_properties, MemoryAreaProperties, MpChangeMode};
use feagi_npu_plasticity::{MemoryMpMode, MpChangeEncoding};
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
