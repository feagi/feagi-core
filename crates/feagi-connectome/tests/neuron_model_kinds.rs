use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::quant_standard_32_bit::NMFeagiAdvancedQuantStandard32Bit;
use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuantKind;
use feagi_connectome::neuron_models::models::exclusive_power::quantizations::quant_float_16::NMExclusivePowerQuantFloat16;
use feagi_connectome::neuron_models::models::exclusive_power::quantizations::NMExclusivePowerQuantKind;
use feagi_connectome::neuron_models::models::kind::NMKind;
use feagi_connectome::neuron_models::models::NeuronModelEnumConversionError;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;

#[test]
fn marker_structs_know_their_quant_kind() {
    assert_eq!(NMExclusivePowerQuantFloat16::QUANT_KIND, NMExclusivePowerQuantKind::Float16);
    assert_eq!(NMFeagiAdvancedQuantStandard32Bit::QUANT_KIND, NMFeagiAdvancedQuantKind::Standard32Bit);
}

#[test]
fn quant_kind_nests_into_nm_kind_and_back() {
    let kind: NMKind = NMExclusivePowerQuantKind::Float16.into();
    assert_eq!(kind, NMKind::ExclusivePower(NMExclusivePowerQuantKind::Float16));
    assert_eq!(NMExclusivePowerQuantKind::try_from(kind).unwrap(), NMExclusivePowerQuantKind::Float16);
}

#[test]
fn unwrapping_nm_kind_as_another_model_errors() {
    let kind: NMKind = NMFeagiAdvancedQuantKind::Standard32Bit.into();
    assert!(matches!(
        NMExclusivePowerQuantKind::try_from(kind),
        Err(NeuronModelEnumConversionError::IncompatibleModel)
    ));
}

#[test]
fn serde_round_trip() {
    let kind = NMKind::ExclusivePower(NMExclusivePowerQuantKind::Float64);
    let json = serde_json::to_string(&kind).unwrap();
    assert_eq!(json, r#"{"ExclusivePower":"Float64"}"#);
    assert_eq!(serde_json::from_str::<NMKind>(&json).unwrap(), kind);
}
