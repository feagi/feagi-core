//! Data of any container wraps into and unwraps out of the `NM*Quanted` and `NMModeled` enums

use feagi_connectome::neuron_models::models::exclusive_power::quantizations::{NMExclusivePowerQuantFloat16, NMExclusivePowerQuantFloat32, NMExclusivePowerQuantKind, NMExclusivePowerQuanted};
use feagi_connectome::neuron_models::models::feagi_advanced::quantizations::NMFeagiAdvancedQuanted;
use feagi_connectome::neuron_models::models::kind::NMKind;
use feagi_connectome::neuron_models::models::{NMContainer, NMMember, NMModeled, NeuronModelEnumConversionError};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;

/// Holds the marker struct itself, as the simplest data to test with
struct Marker;

impl NMContainer for Marker {
    type Of<NM: NeuronModel> = NM;
}

#[test]
fn wraps_up_to_modeled_and_back() {
    let quanted = NMExclusivePowerQuantFloat16::wrap::<Marker>(NMExclusivePowerQuantFloat16);
    assert_eq!(quanted.quant_kind(), NMExclusivePowerQuantKind::Float16);
    let modeled: NMModeled<Marker> = quanted.into();
    assert_eq!(modeled.kind(), NMKind::ExclusivePower(NMExclusivePowerQuantKind::Float16));
    let quanted = NMExclusivePowerQuanted::try_from(modeled).unwrap();
    assert_eq!(NMExclusivePowerQuantFloat16::unwrap::<Marker>(quanted).unwrap(), NMExclusivePowerQuantFloat16);
}

#[test]
fn unwrapping_another_quant_errors() {
    let quanted = NMExclusivePowerQuantFloat16::wrap::<Marker>(NMExclusivePowerQuantFloat16);
    assert!(matches!(
        NMExclusivePowerQuantFloat32::unwrap::<Marker>(quanted),
        Err(NeuronModelEnumConversionError::IncompatibleUnwrap { expected: NMExclusivePowerQuantKind::Float32, actual: NMExclusivePowerQuantKind::Float16 })
    ));
}

#[test]
fn unwrapping_another_model_errors() {
    let modeled: NMModeled<Marker> = NMExclusivePowerQuantFloat16::wrap::<Marker>(NMExclusivePowerQuantFloat16).into();
    assert!(matches!(NMFeagiAdvancedQuanted::try_from(modeled), Err(NeuronModelEnumConversionError::IncompatibleModel)));
}

#[test]
fn quanted_serde_round_trip() {
    let quanted = NMExclusivePowerQuantFloat32::wrap::<Marker>(NMExclusivePowerQuantFloat32);
    let json = serde_json::to_string(&quanted).unwrap();
    let back: NMExclusivePowerQuanted<Marker> = serde_json::from_str(&json).unwrap();
    assert_eq!(back.quant_kind(), NMExclusivePowerQuantKind::Float32);
}

/// The neurogenesis containers resolve, and their enums get the traits their data has
#[test]
fn neurogenesis_containers_resolve() {
    use feagi_connectome::neuron_models::models::{NMNeurogenesisGenerator, NMNeurogenesisValidatedGenerator};
    use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantizationStandard as Fiq;
    fn saveable<T: Clone + core::fmt::Debug + serde::Serialize + serde::de::DeserializeOwned>() {}
    fn cloneable<T: Clone + core::fmt::Debug>() {}
    saveable::<NMExclusivePowerQuanted<NMNeurogenesisGenerator<Fiq>>>();
    cloneable::<NMModeled<NMNeurogenesisGenerator<Fiq>>>();
    cloneable::<NMModeled<NMNeurogenesisValidatedGenerator<Fiq>>>();
    #[cfg(feature = "alloc")]
    {
        use feagi_connectome::neuron_models::models::{NMNeurogenesisStorageOrGenerator, NMNeurogenesisValidatedStorageOrGenerator};
        saveable::<NMExclusivePowerQuanted<NMNeurogenesisStorageOrGenerator<Fiq>>>();
        cloneable::<NMModeled<NMNeurogenesisValidatedStorageOrGenerator<Fiq>>>();
    }
}

#[test]
fn modeled_serde_round_trip() {
    let modeled: NMModeled<Marker> = NMExclusivePowerQuantFloat32::wrap::<Marker>(NMExclusivePowerQuantFloat32).into();
    let json = serde_json::to_string(&modeled).unwrap();
    assert_eq!(json, r#"{"ExclusivePower":{"Float32":null}}"#);
    let back: NMModeled<Marker> = serde_json::from_str(&json).unwrap();
    assert_eq!(back.kind(), NMKind::ExclusivePower(NMExclusivePowerQuantKind::Float32));
}
