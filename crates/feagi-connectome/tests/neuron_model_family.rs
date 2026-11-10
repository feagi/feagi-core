use feagi_connectome::neuron_models::models::::NMFeagiAdvancedQuantDefaultQuant;
use feagi_connectome::neuron_models::models::::{NMFoundationPowerQuantFloat16, NMFoundationPowerQuantFloat32, NMFoundationPowerQuantFloat64};
use feagi_connectome::connectome_basis::neuron_model::{
    NMAccessError, NMContainer, NMEnum,
    NMFoundationPowerQuantEnum, NMFoundationPowerQuantKind, NMKind,
    NMMember,
};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;

/// A container holding the name of the member it is held for
struct Named;

impl NMContainer for Named {
    type Of<M: NeuronModel> = String;
}

fn named<M: NMMember>() -> NMEnum<Named> {
    NMEnum::from_data::<M>(M::KIND.to_string())
}

#[test]
fn from_data_and_try_as() {
    let mut value = named::<NMFoundationPowerQuantFloat32>();
    assert_eq!(value.kind(), NMFoundationPowerQuantFloat32::KIND);
    assert_eq!(value.try_as::<NMFoundationPowerQuantFloat32>().unwrap(), "FoundationPower::Float32");

    value.try_as_mut::<NMFoundationPowerQuantFloat32>().unwrap().push('!');
    assert_eq!(value.try_as::<NMFoundationPowerQuantFloat32>().unwrap(), "FoundationPower::Float32!");

    assert_eq!(
        value.try_as::<NMFoundationPowerQuantFloat16>(),
        Err(NMAccessError::IncorrectMember {
            expected: NMFoundationPowerQuantFloat16::KIND,
            actual: NMFoundationPowerQuantFloat32::KIND,
        })
    );
    assert!(value.try_as::<NMFeagiAdvancedQuantDefaultQuant>().is_err());
    assert_eq!(value.try_into_data::<NMFoundationPowerQuantFloat32>().unwrap(), "FoundationPower::Float32!");
}

#[test]
fn nested_conversions() {
    let value = named::<NMFoundationPowerQuantFloat16>();
    let inner: NMFoundationPowerQuantEnum<Named> = value.clone().try_into().unwrap();
    assert_eq!(inner.quant_kind(), NMFoundationPowerQuantKind::Float16);
    assert_eq!(inner.try_as::<NMFoundationPowerQuantFloat16>().unwrap(), "FoundationPower::Float16");
    assert_eq!(NMEnum::from(inner), value);

    let other = named::<NMFeagiAdvancedQuantDefaultQuant>();
    assert_eq!(
        NMFoundationPowerQuantEnum::try_from(other),
        Err(NMAccessError::IncorrectModel {
            expected: "FoundationPower",
            actual: NMFeagiAdvancedQuantDefaultQuant::KIND,
        })
    );
}

#[test]
fn dispatch_macros() {
    let values = [
        named::<NMFoundationPowerQuantFloat16>(),
        named::<NMFoundationPowerQuantFloat64>(),
        named::<NMFeagiAdvancedQuantDefaultQuant>(),
    ];
    for value in values {
        let kind = value.kind();

        let length = feagi_connectome::with_nm!(&value, |data| data.len());
        assert_eq!(length, kind.to_string().len());

        let marker_kind = feagi_connectome::with_nm!(&value, <NM> |_data| NM::KIND);
        assert_eq!(marker_kind, kind);
    }

    let inner = NMFoundationPowerQuantEnum::<Named>::from_data::<NMFoundationPowerQuantFloat16>(String::from("x"));
    let level = feagi_connectome::with_nm_foundation_power!(&inner, <NM> |_data| <NM as NMMember>::KIND);
    assert_eq!(level, NMKind::FoundationPower(NMFoundationPowerQuantKind::Float16));
}

#[test]
fn dispatch_macros_alias_model_module() {
    use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantizationStandard;

    let foundation_power = named::<NMFoundationPowerQuantFloat32>();
    let dynamics = feagi_connectome::with_nm!(&foundation_power, <NM, Model> |_data| {
        core::any::type_name::<Model::BurstEngineDynamics<FeagiIndexQuantizationStandard, NM>>()
    });
    assert!(dynamics.contains("FoundationPowerBurstEngineDynamics"));
    assert!(dynamics.contains("NMFoundationPowerQuantFloat32"));

    let feagi_advanced = named::<NMFeagiAdvancedQuantDefaultQuant>();
    let dynamics = feagi_connectome::with_nm!(&feagi_advanced, <NM, Model> |_data| {
        core::any::type_name::<Model::BurstEngineDynamics<FeagiIndexQuantizationStandard, NM>>()
    });
    assert!(dynamics.contains("FeagiAdvancedBurstEngineDynamics"));

    let inner = NMFoundationPowerQuantEnum::<Named>::from_data::<NMFoundationPowerQuantFloat16>(String::from("x"));
    let dynamics = feagi_connectome::with_nm_foundation_power!(&inner, <NM, Model> |_data| {
        core::any::type_name::<Model::BurstEngineDynamics<FeagiIndexQuantizationStandard, NM>>()
    });
    assert!(dynamics.contains("NMFoundationPowerQuantFloat16"));
}

#[test]
fn serde_round_trip() {
    let value = named::<NMFoundationPowerQuantFloat64>();
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, r#"{"FoundationPower":{"Float64":"FoundationPower::Float64"}}"#);
    assert_eq!(serde_json::from_str::<NMEnum<Named>>(&json).unwrap(), value);

    let kind_json = serde_json::to_string(&value.kind()).unwrap();
    assert_eq!(serde_json::from_str::<NMKind>(&kind_json).unwrap(), value.kind());
}