use feagi_connectome::neuron_models::neuron_model;
use feagi_connectome::neuron_models::neuron_model::feagi_advanced::NeuronModelFeagiAdvancedDefaultQuant;
use feagi_connectome::neuron_models::neuron_model::foundation_power::{NeuronModelFoundationPowerFloat16, NeuronModelFoundationPowerFloat32, NeuronModelFoundationPowerFloat64};
use feagi_connectome::neuron_models::neuron_model::{
    FoundationPowerQuantEnum, FoundationPowerQuantKind, NeuronModelContainer, NeuronModelEnum, NeuronModelKind,
    NeuronModelMember, NeuronModelSelectionError,
};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;

/// A container holding the name of the member it is held for
struct Named;

impl NeuronModelContainer for Named {
    type Of<M: NeuronModel> = String;
}

fn named<M: NeuronModelMember>() -> NeuronModelEnum<Named> {
    NeuronModelEnum::from_data::<M>(M::KIND.to_string())
}

#[test]
fn from_data_and_try_as() {
    let mut value = named::<NeuronModelFoundationPowerFloat32>();
    assert_eq!(value.kind(), NeuronModelFoundationPowerFloat32::KIND);
    assert_eq!(value.try_as::<NeuronModelFoundationPowerFloat32>().unwrap(), "FoundationPower::Float32");

    value.try_as_mut::<NeuronModelFoundationPowerFloat32>().unwrap().push('!');
    assert_eq!(value.try_as::<NeuronModelFoundationPowerFloat32>().unwrap(), "FoundationPower::Float32!");

    assert_eq!(
        value.try_as::<NeuronModelFoundationPowerFloat16>(),
        Err(NeuronModelSelectionError::IncorrectMember {
            expected: NeuronModelFoundationPowerFloat16::KIND,
            actual: NeuronModelFoundationPowerFloat32::KIND,
        })
    );
    assert!(value.try_as::<NeuronModelFeagiAdvancedDefaultQuant>().is_err());
    assert_eq!(value.try_into_data::<NeuronModelFoundationPowerFloat32>().unwrap(), "FoundationPower::Float32!");
}

#[test]
fn try_as_marker() {
    let value = named::<NeuronModelFeagiAdvancedDefaultQuant>();
    assert_eq!(value.try_as_marker::<NeuronModelFeagiAdvancedDefaultQuant>(), Ok(NeuronModelFeagiAdvancedDefaultQuant));
    assert!(value.try_as_marker::<NeuronModelFoundationPowerFloat32>().is_err());
    assert_eq!(value.kind().try_as_marker::<NeuronModelFeagiAdvancedDefaultQuant>(), Ok(NeuronModelFeagiAdvancedDefaultQuant));
}

#[test]
fn nested_conversions() {
    let value = named::<NeuronModelFoundationPowerFloat16>();
    let inner: FoundationPowerQuantEnum<Named> = value.clone().try_into().unwrap();
    assert_eq!(inner.quant_kind(), FoundationPowerQuantKind::Float16);
    assert_eq!(inner.try_as::<NeuronModelFoundationPowerFloat16>().unwrap(), "FoundationPower::Float16");
    assert_eq!(NeuronModelEnum::from(inner), value);

    let other = named::<NeuronModelFeagiAdvancedDefaultQuant>();
    assert_eq!(
        FoundationPowerQuantEnum::try_from(other),
        Err(NeuronModelSelectionError::IncorrectModel {
            expected: "FoundationPower",
            actual: NeuronModelFeagiAdvancedDefaultQuant::KIND,
        })
    );
}

#[test]
fn dispatch_macros() {
    for kind in NeuronModelKind::ALL {
        let value = feagi_connectome::with_neuron_model_kind!(kind, <NM> named::<NM>());
        assert_eq!(value.kind(), kind);

        let length = feagi_connectome::with_neuron_model!(&value, |data| data.len());
        assert_eq!(length, kind.to_string().len());

        let marker_kind = feagi_connectome::with_neuron_model!(&value, <NM> |_data| NM::KIND);
        assert_eq!(marker_kind, kind);
    }

    let inner = FoundationPowerQuantEnum::<Named>::from_data::<NeuronModelFoundationPowerFloat16>(String::from("x"));
    let level = feagi_connectome::with_neuron_model_foundation_power!(&inner, <NM> |_data| <NM as neuron_model::NeuronModelFoundationPowerMember>::QUANT_KIND);
    assert_eq!(level, FoundationPowerQuantKind::Float16);
}

#[test]
fn dispatch_macros_alias_model_module() {
    use feagi_connectome::quantization_levels::FeagiIndexQuantizationStandard;

    let foundation_power = named::<NeuronModelFoundationPowerFloat32>();
    let dynamics = feagi_connectome::with_neuron_model!(&foundation_power, <NM, Model> |_data| {
        core::any::type_name::<Model::BurstEngineDynamics<FeagiIndexQuantizationStandard, NM>>()
    });
    assert!(dynamics.contains("FoundationPowerBurstEngineDynamics"));
    assert!(dynamics.contains("NeuronModelFoundationPowerFloat32"));

    let feagi_advanced = named::<NeuronModelFeagiAdvancedDefaultQuant>();
    let dynamics = feagi_connectome::with_neuron_model!(&feagi_advanced, <NM, Model> |_data| {
        core::any::type_name::<Model::BurstEngineDynamics<FeagiIndexQuantizationStandard, NM>>()
    });
    assert!(dynamics.contains("FeagiAdvancedBurstEngineDynamics"));

    let inner = FoundationPowerQuantEnum::<Named>::from_data::<NeuronModelFoundationPowerFloat16>(String::from("x"));
    let dynamics = feagi_connectome::with_neuron_model_foundation_power!(&inner, <NM, Model> |_data| {
        core::any::type_name::<Model::BurstEngineDynamics<FeagiIndexQuantizationStandard, NM>>()
    });
    assert!(dynamics.contains("NeuronModelFoundationPowerFloat16"));
}

#[test]
fn serde_round_trip() {
    let value = named::<NeuronModelFoundationPowerFloat64>();
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, r#"{"FoundationPower":{"Float64":"FoundationPower::Float64"}}"#);
    assert_eq!(serde_json::from_str::<NeuronModelEnum<Named>>(&json).unwrap(), value);

    let kind_json = serde_json::to_string(&value.kind()).unwrap();
    assert_eq!(serde_json::from_str::<NeuronModelKind>(&kind_json).unwrap(), value.kind());
}