use feagi_connectome::neuron_models::models::container::{NMAccessError, NMContainer, NMContainerMarker, NMMember, NMVisitor, NMVisitorMut, NMVisitorRef};
use feagi_connectome::neuron_models::models::exclusive_death::quantizations::NMExclusiveDeathQuantBFloat16;
use feagi_connectome::neuron_models::models::exclusive_power::quantizations::{NMExclusivePowerQuantFloat32, NMExclusivePowerQuantKind, NMExclusivePowerQuanted};
use feagi_connectome::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleQuantKind, NMFeagiSimpleQuantStandard32Bit};
use feagi_connectome::neuron_models::models::kind::{NMKind, NMQuantedKind};
use feagi_connectome::neuron_models::models::NMModeled;

/// A container holding a plain number for every member, to test the tree without neuron data
#[derive(Clone, Copy, Debug)]
struct Number;

impl NMContainer for Number {
    type Of<NM: feagi_connectome_neural::neural_basis::neuron_model::NeuronModel> = u32;
}

#[test]
fn member_wraps_and_unwraps_directly() {
    let modeled = NMModeled::<Number>::from_member::<NMExclusivePowerQuantFloat32>(7);
    assert_eq!(modeled.kind(), NMKind::ExclusivePower(NMExclusivePowerQuantKind::Float32));
    assert!(matches!(modeled, NMModeled::ExclusivePower(NMExclusivePowerQuanted::Float32(7))));
    assert_eq!(modeled.try_as_member::<NMExclusivePowerQuantFloat32>(), Ok(&7));
    assert_eq!(modeled.try_into_member::<NMExclusivePowerQuantFloat32>(), Ok(7));
}

#[test]
fn accessing_another_member_errors_with_both_kinds() {
    let mut modeled = NMExclusivePowerQuantFloat32::into_modeled::<Number>(7);
    let expected = NMAccessError::IncorrectMember {
        expected: NMKind::FeagiSimple(NMFeagiSimpleQuantKind::Standard32Bit),
        actual: NMKind::ExclusivePower(NMExclusivePowerQuantKind::Float32),
    };
    assert_eq!(modeled.try_as_member::<NMFeagiSimpleQuantStandard32Bit>(), Err(expected));
    assert_eq!(modeled.try_as_member_mut::<NMFeagiSimpleQuantStandard32Bit>(), Err(expected));
    assert_eq!(modeled.clone().try_into_member::<NMFeagiSimpleQuantStandard32Bit>(), Err(expected));
    // Same model, other quantization
    assert!(matches!(
        modeled.try_into_member::<NMExclusiveDeathQuantBFloat16>(),
        Err(NMAccessError::IncorrectMember { .. })
    ));
}

#[test]
fn mutating_through_member_access() {
    let mut modeled = NMModeled::<Number>::from_member::<NMFeagiSimpleQuantStandard32Bit>(1);
    *modeled.try_as_member_mut::<NMFeagiSimpleQuantStandard32Bit>().unwrap() += 1;
    assert_eq!(modeled.try_into_member::<NMFeagiSimpleQuantStandard32Bit>(), Ok(2));
}

#[test]
fn model_level_conversions_go_both_ways() {
    let modeled = NMModeled::<Number>::from(NMExclusivePowerQuanted::Float32(3));
    assert_eq!(NMExclusivePowerQuanted::<Number>::try_from(modeled.clone()).map(|q| q.quant_kind()), Ok(NMExclusivePowerQuantKind::Float32));
    let simple = NMModeled::<Number>::from_member::<NMFeagiSimpleQuantStandard32Bit>(3);
    assert!(matches!(
        NMExclusivePowerQuanted::<Number>::try_from(simple),
        Err(NMAccessError::IncorrectModel { expected: "ExclusivePower", .. })
    ));
}

/// Reports the kind of the member visited, and the data it held
struct KindOf;

impl NMVisitor<Number> for KindOf {
    type Output = (NMKind, u32);
    fn visit<NMM: NMMember>(self, data: u32) -> Self::Output {
        (NM::KIND, data)
    }
}

impl<'a> NMVisitorRef<'a, Number> for KindOf {
    type Output = (NMKind, &'a u32);
    fn visit<NMM: NMMember>(self, data: &'a u32) -> Self::Output {
        (NM::KIND, data)
    }
}

struct Double;

impl<'a> NMVisitorMut<'a, Number> for Double {
    type Output = ();
    fn visit<NMM: NMMember>(self, data: &'a mut u32) {
        *data *= 2;
    }
}

#[test]
fn dispatch_runs_generic_code_on_the_member_held() {
    let mut modeled = NMModeled::<Number>::from_member::<NMExclusiveDeathQuantBFloat16>(5);
    let kind = NMExclusiveDeathQuantBFloat16::KIND;
    modeled.dispatch_mut(Double);
    assert_eq!(modeled.dispatch_ref(KindOf), (kind, &10));
    assert_eq!(modeled.dispatch(KindOf), (kind, 10));
}

#[test]
fn quanted_kind_is_the_marker_container() {
    let kind: NMQuantedKind = NMModeled::<NMContainerMarker>::from_member::<NMExclusivePowerQuantFloat32>(NMExclusivePowerQuantFloat32);
    assert_eq!(kind, NMQuantedKind::from_nm_quant::<NMExclusivePowerQuantFloat32>());
    assert_ne!(kind, NMQuantedKind::from_nm_quant::<NMFeagiSimpleQuantStandard32Bit>());
    let json = serde_json::to_string(&kind).unwrap();
    assert_eq!(serde_json::from_str::<NMQuantedKind>(&json).unwrap(), kind);
}
