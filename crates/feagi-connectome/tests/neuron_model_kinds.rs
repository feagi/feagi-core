use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedQuantKind, NMFeagiAdvancedQuantStandard32Bit, NMFeagiAdvancedQuantedKind};
use feagi_connectome::neuron_models::models::exclusive_power::quantizations::{NMExclusivePowerQuantFloat16, NMExclusivePowerQuantKind, NMExclusivePowerQuantedKind};
use feagi_connectome::neuron_models::models::kind::{NMKind, NMQuantedKind};
use feagi_connectome::neuron_models::models::container::NMAccessError;
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
        Err(NMAccessError::IncorrectModel { expected: "ExclusivePower", .. })
    ));
}

#[test]
fn serde_round_trip() {
    let kind = NMKind::ExclusivePower(NMExclusivePowerQuantKind::Float64);
    let json = serde_json::to_string(&kind).unwrap();
    assert_eq!(json, r#"{"ExclusivePower":"Float64"}"#);
    assert_eq!(serde_json::from_str::<NMKind>(&json).unwrap(), kind);
}

#[test]
fn quanted_kind_nests_into_nm_quanted_kind_and_back() {
    let kind: NMQuantedKind = NMExclusivePowerQuantedKind::from(NMExclusivePowerQuantFloat16).into();
    assert_eq!(kind, NMQuantedKind::ExclusivePower(NMExclusivePowerQuantedKind::Float16(NMExclusivePowerQuantFloat16)));

    let model_level = NMExclusivePowerQuantedKind::try_from(kind).unwrap();
    assert_eq!(model_level.quant_kind(), NMExclusivePowerQuantKind::Float16);
    assert_eq!(NMExclusivePowerQuantFloat16::try_from(model_level).unwrap(), NMExclusivePowerQuantFloat16);
}

#[test]
fn nm_quanted_kind_from_any_neuron_model() {
    assert_eq!(
        NMQuantedKind::from_nm_quant::<NMExclusivePowerQuantFloat16>(),
        NMQuantedKind::ExclusivePower(NMExclusivePowerQuantedKind::Float16(NMExclusivePowerQuantFloat16)),
    );
    assert_eq!(
        NMQuantedKind::from_nm_quant::<NMFeagiAdvancedQuantStandard32Bit>(),
        NMQuantedKind::FeagiAdvanced(NMFeagiAdvancedQuantedKind::Standard32Bit(NMFeagiAdvancedQuantStandard32Bit)),
    );
}

#[test]
fn unwrapping_nm_quanted_kind_as_another_model_errors() {
    let kind: NMQuantedKind = NMFeagiAdvancedQuantStandard32Bit::QUANTED_KIND.into();
    assert_eq!(kind, NMQuantedKind::FeagiAdvanced(NMFeagiAdvancedQuantedKind::Standard32Bit(NMFeagiAdvancedQuantStandard32Bit)));
    assert!(matches!(
        NMExclusivePowerQuantedKind::try_from(kind),
        Err(NMAccessError::IncorrectModel { expected: "ExclusivePower", .. })
    ));
}
