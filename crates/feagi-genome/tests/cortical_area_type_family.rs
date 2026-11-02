use feagi_connectome::neuron_models::neuron_model;
use feagi_connectome::neuron_models::neuron_model::feagi_advanced::NeuronModelFeagiAdvancedDefaultQuant;
use feagi_connectome::neuron_models::neuron_model::foundation_power::{NeuronModelFoundationPowerFloat16, NeuronModelFoundationPowerFloat32};
use feagi_connectome::neuron_models::neuron_model::{FoundationPowerQuantKind, NeuronModel, NeuronModelContainer, NeuronModelKind, NeuronModelMember, NeuronModelSelectionError};
use feagi_connectome::{with_neuron_model_feagi_advanced, with_neuron_model_foundation_power};
use feagi_genome::basis::cortical_area_types::cortical_area_type;
use feagi_genome::basis::cortical_area_types::cortical_area_type::foundation::power::CorticalAreaTypeFoundationPower;
use feagi_genome::basis::cortical_area_types::cortical_area_type::interconnect::CorticalAreaTypeInterconnect;
use feagi_genome::basis::cortical_area_types::cortical_area_type::{
    CorticalAreaTypeEnum, CorticalAreaTypeKind, CorticalAreaTypeMember, CorticalAreaTypeSelectionError, FoundationSuperTypeEnum, FoundationSuperTypeKind,
};
use feagi_genome::basis::cortical_area_types::enums::CorticalGenomicDataEnum;

/// A container holding the name of the neuron model member it is held for
struct Named;

impl NeuronModelContainer for Named {
    type Of<M: NeuronModel> = String;
}

fn named<T: CorticalAreaTypeMember>() -> CorticalAreaTypeEnum<Named> {
    CorticalAreaTypeEnum::from_data::<T>(<T::NeuronModel as NeuronModelMember>::KIND.to_string())
}

type Power32 = CorticalAreaTypeFoundationPower<NeuronModelFoundationPowerFloat32>;
type Power16 = CorticalAreaTypeFoundationPower<NeuronModelFoundationPowerFloat16>;
type Interconnect = CorticalAreaTypeInterconnect<NeuronModelFeagiAdvancedDefaultQuant>;

#[test]
fn kinds_name_every_cortical_type() {
    assert_eq!(CorticalAreaTypeKind::ALL, [CorticalAreaTypeKind::Foundation(FoundationSuperTypeKind::Power), CorticalAreaTypeKind::Interconnect]);
    assert_eq!(FoundationSuperTypeKind::ALL, [FoundationSuperTypeKind::Power]);
    assert_eq!(CorticalAreaTypeKind::Foundation(FoundationSuperTypeKind::Power).to_string(), "Foundation::Power");
    assert_eq!(CorticalAreaTypeKind::Interconnect.name(), "Interconnect");
    assert_eq!(Power32::KIND, CorticalAreaTypeKind::Foundation(FoundationSuperTypeKind::Power));
}

#[test]
fn from_data_and_try_as() {
    let mut value = named::<Power32>();
    assert_eq!(value.kind(), Power32::KIND);
    assert_eq!(value.neuron_model_kind(), NeuronModelKind::FoundationPower(FoundationPowerQuantKind::Float32));
    assert_eq!(value.try_as::<Power32>().unwrap(), "FoundationPower::Float32");

    value.try_as_mut::<Power32>().unwrap().push('!');
    assert_eq!(value.try_as::<Power32>().unwrap(), "FoundationPower::Float32!");

    assert_eq!(
        value.try_as::<Power16>(),
        Err(CorticalAreaTypeSelectionError::NeuronModel(NeuronModelSelectionError::IncorrectMember {
            expected: NeuronModelFoundationPowerFloat16::KIND,
            actual: NeuronModelFoundationPowerFloat32::KIND,
        }))
    );
    assert_eq!(
        value.try_as::<Interconnect>(),
        Err(CorticalAreaTypeSelectionError::IncorrectCorticalType { expected: Interconnect::KIND, actual: Power32::KIND })
    );
    assert_eq!(value.try_into_data::<Power32>().unwrap(), "FoundationPower::Float32!");
}

#[test]
fn super_type_conversions() {
    let value = named::<Power16>();
    let foundation: FoundationSuperTypeEnum<Named> = value.clone().try_into().unwrap();
    assert_eq!(foundation.kind(), FoundationSuperTypeKind::Power);
    assert_eq!(CorticalAreaTypeEnum::from(foundation), value);

    assert_eq!(
        FoundationSuperTypeEnum::try_from(named::<Interconnect>()),
        Err(CorticalAreaTypeSelectionError::IncorrectSuperType { expected: "Foundation", actual: "Interconnect" })
    );
}

#[test]
fn dispatch_macro() {
    for value in [named::<Power16>(), named::<Interconnect>()] {
        let kind = feagi_genome::with_cortical_area_type!(&value, |inner| inner.kind());
        assert_eq!(kind, value.neuron_model_kind());

        let (cortical_kind, member_kind, length) = feagi_genome::with_cortical_area_type!(&value, <T, NM> |data| (
            <T as CorticalAreaTypeMember>::KIND,
            <NM as NeuronModelMember>::KIND,
            data.len()
        ));
        assert_eq!(cortical_kind, value.kind());
        assert_eq!(member_kind, value.neuron_model_kind());
        assert_eq!(length, member_kind.to_string().len());
    }
}

#[test]
fn serde_round_trip() {
    let value = named::<Power32>();
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, r#"{"Foundation":{"Power":{"Float32":"FoundationPower::Float32"}}}"#);
    assert_eq!(serde_json::from_str::<CorticalAreaTypeEnum<Named>>(&json).unwrap(), value);
}

/// The genomic data names a valid enum of the family
#[allow(dead_code)]
fn genomic_data_kind(data: &CorticalGenomicDataEnum<feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationStandard>) -> CorticalAreaTypeKind {
    data.kind()
}
