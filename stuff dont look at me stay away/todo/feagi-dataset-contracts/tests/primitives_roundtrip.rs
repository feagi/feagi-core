//! Serde round-trip tests for the shared contract primitives.
//!
//! These pin the wire format of the primitives this crate owns, independent of any
//! consuming engine: transparent id newtypes serialize as bare strings, taxonomy enums use
//! `snake_case`, and `MetadataValue` is untagged. The trainer/capture aggregate contracts
//! rely on these guarantees, so a regression here is a breaking interchange change.

use std::collections::BTreeMap;

use feagi_dataset_contracts::{
    BackendKind, ConnectomeHash, DatasetAssetId, MetadataMap, MetadataValue, Modality, OutputType,
    PluginId, PluginRef, QuantizationFingerprint, Split,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Serializes `value`, deserializes it back, and asserts equality.
fn assert_json_round_trip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(value).expect("serialize");
    let restored: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(value, &restored);
}

#[test]
fn string_id_serializes_transparently() {
    let id = DatasetAssetId("local:iris".to_string());
    let json = serde_json::to_string(&id).expect("serialize");
    assert_eq!(json, "\"local:iris\"");
    assert_json_round_trip(&id);
    assert_eq!(id.as_str(), "local:iris");
}

#[test]
fn plugin_ref_round_trips() {
    let plugin = PluginRef {
        id: PluginId("classification".to_string()),
        version: "1.0.0".to_string(),
    };
    assert_json_round_trip(&plugin);
}

#[test]
fn enums_use_snake_case() {
    assert_eq!(
        serde_json::to_string(&BackendKind::Cpu).expect("serialize"),
        "\"cpu\""
    );
    assert_eq!(
        serde_json::to_string(&Modality::Multimodal).expect("serialize"),
        "\"multimodal\""
    );
    assert_eq!(
        serde_json::to_string(&OutputType::Pose6Dof).expect("serialize"),
        "\"pose6_dof\""
    );
}

#[test]
fn split_custom_variant_round_trips() {
    let split = Split::Custom("holdout".to_string());
    assert_json_round_trip(&split);
}

#[test]
fn metadata_value_is_untagged() {
    assert_eq!(
        serde_json::to_string(&MetadataValue::Int(5)).expect("serialize"),
        "5"
    );
    assert_eq!(
        serde_json::to_string(&MetadataValue::Text("x".to_string())).expect("serialize"),
        "\"x\""
    );
    let list = MetadataValue::List(vec![MetadataValue::Bool(true), MetadataValue::Float(1.5)]);
    assert_json_round_trip(&list);
}

#[test]
fn metadata_map_orders_keys_deterministically() {
    let mut map: MetadataMap = BTreeMap::new();
    map.insert("zeta".to_string(), MetadataValue::Int(1));
    map.insert("alpha".to_string(), MetadataValue::Int(2));
    let json = serde_json::to_string(&map).expect("serialize");
    assert_eq!(json, "{\"alpha\":2,\"zeta\":1}");
    assert_json_round_trip(&map);
}

#[test]
fn quantization_fingerprint_round_trips() {
    let fingerprint = QuantizationFingerprint {
        level: "bit16".to_string(),
        details: serde_json::json!({ "neuron": 16, "synapse": 8 }),
    };
    assert_json_round_trip(&fingerprint);
}

#[test]
fn connectome_hash_round_trips() {
    assert_json_round_trip(&ConnectomeHash("sha256:connectome".to_string()));
}
