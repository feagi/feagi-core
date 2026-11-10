use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedQuantKind, NMFeagiAdvancedQuantStandard32Bit};
use feagi_connectome::neuron_models::models::foundation_power::quantizations::{NMFoundationPowerQuantFloat16, NMFoundationPowerQuantKind};
use feagi_connectome::neuron_models::models::{NMKind, NMModelKind, NeuronModelEnumConversionError};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;

#[test]
fn marker_structs_know_their_quant_kind() {
    assert_eq!(NMFoundationPowerQuantFloat16::QUANT_KIND, NMFoundationPowerQuantKind::Float16);
    assert_eq!(NMFeagiAdvancedQuantStandard32Bit::QUANT_KIND, NMFeagiAdvancedQuantKind::Standard32Bit);
}

#[test]
fn quant_kind_nests_into_nm_kind_and_back() {
    let kind: NMKind = NMFoundationPowerQuantKind::Float16.into();
    assert_eq!(kind, NMKind::FoundationPower(NMFoundationPowerQuantKind::Float16));
    assert_eq!(kind.model_kind(), NMModelKind::FoundationPower);
    assert_eq!(NMFoundationPowerQuantKind::try_from(kind).unwrap(), NMFoundationPowerQuantKind::Float16);
}

#[test]
fn unwrapping_nm_kind_as_another_model_errors() {
    let kind: NMKind = NMFeagiAdvancedQuantKind::Standard32Bit.into();
    assert!(matches!(
        NMFoundationPowerQuantKind::try_from(kind),
        Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
            expected: NMModelKind::FoundationPower,
            actual: NMModelKind::FeagiAdvanced,
        })
    ));
}

#[test]
fn serde_round_trip() {
    let kind = NMKind::FoundationPower(NMFoundationPowerQuantKind::Float64);
    let json = serde_json::to_string(&kind).unwrap();
    assert_eq!(json, r#"{"FoundationPower":"Float64"}"#);
    assert_eq!(serde_json::from_str::<NMKind>(&json).unwrap(), kind);

    let model_json = serde_json::to_string(&kind.model_kind()).unwrap();
    assert_eq!(serde_json::from_str::<NMModelKind>(&model_json).unwrap(), NMModelKind::FoundationPower);
}
