use feagi_connectome::neuron_models::neuron_model;
use feagi_connectome::neuron_models::neuron_model::feagi_advanced::NeuronModelFeagiAdvancedDefaultQuant;
use feagi_connectome::neuron_models::neuron_model::feagi_simple::NeuronModelFeagiSimpleDefaultQuant;
use feagi_connectome::neuron_models::neuron_model::foundation_death::NeuronModelFoundationDeathFloat64;
use feagi_connectome::neuron_models::neuron_model::foundation_power::{NeuronModelFoundationPowerFloat16, NeuronModelFoundationPowerFloat32, NeuronModelFoundationPowerFloat64};
use feagi_connectome::neuron_models::neuron_model::memory_fire_hashing::NeuronModelMemoryFireHashingDefaultQuant;
use feagi_connectome::neuron_models::neuron_model::{FoundationPowerQuantKind, NeuronModelContainer, NeuronModelKind, NeuronModelMember, NeuronModelSelectionError};
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationStandard;
use feagi_connectome::{
    with_neuron_model_feagi_advanced, with_neuron_model_feagi_simple, with_neuron_model_foundation_death, with_neuron_model_foundation_power,
    with_neuron_model_memory_fire_hashing,
};
use feagi_genome::cortical_area::cortical_area_type;
use feagi_genome::cortical_area::cortical_area_type::{
    CorticalAreaTypeEnum, CorticalAreaTypeKind, CorticalAreaTypeMember, CorticalAreaTypeMemberAt, CorticalAreaTypeSelectionError, FoundationPowerModelEnum, FoundationSuperTypeEnum,
    FoundationSuperTypeKind, MotorSuperTypeKind, ProcessingSuperTypeKind, SensorSuperTypeKind, TunableSuperTypeKind,
};
use feagi_genome::cortical_area::foundation::{Death, Power};
use feagi_genome::cortical_area::motor::Led;
use feagi_genome::cortical_area::processing::{Interconnect, Memory};
use feagi_genome::cortical_area::sensor::Camera;
use feagi_genome::cortical_area::tunable::BrightnessAdjust;
use feagi_genome::cortical_area::CorticalGenomicDataEnum;
use neural_basis::neuron_model::NeuronModel;

type Fiq = FeagiIndexQuantizationStandard;
type Advanced = NeuronModelFeagiAdvancedDefaultQuant;
type Simple = NeuronModelFeagiSimpleDefaultQuant;
type Hashing = NeuronModelMemoryFireHashingDefaultQuant;

/// A container holding the name of the neuron model member it is held for
struct Named;

impl NeuronModelContainer for Named {
    type Of<M: NeuronModel> = String;
}

fn named<T: CorticalAreaTypeMemberAt<M>, M: NeuronModelMember>() -> CorticalAreaTypeEnum<Named> {
    CorticalAreaTypeEnum::from_data::<T, M>(M::KIND.to_string())
}

#[test]
fn kinds_name_every_cortical_type() {
    assert_eq!(
        CorticalAreaTypeKind::ALL,
        [
            CorticalAreaTypeKind::Foundation(FoundationSuperTypeKind::Power),
            CorticalAreaTypeKind::Foundation(FoundationSuperTypeKind::Death),
            CorticalAreaTypeKind::Processing(ProcessingSuperTypeKind::Interconnect),
            CorticalAreaTypeKind::Processing(ProcessingSuperTypeKind::Memory),
            CorticalAreaTypeKind::Sensor(SensorSuperTypeKind::Camera),
            CorticalAreaTypeKind::Motor(MotorSuperTypeKind::Led),
            CorticalAreaTypeKind::Tunable(TunableSuperTypeKind::BrightnessAdjust),
        ]
    );
    assert_eq!(FoundationSuperTypeKind::ALL, [FoundationSuperTypeKind::Power, FoundationSuperTypeKind::Death]);
    assert_eq!(Power::KIND.to_string(), "Foundation::Power");
    assert_eq!(BrightnessAdjust::KIND.to_string(), "Tunable::BrightnessAdjust");
    assert_eq!(Interconnect::KIND.name(), "Interconnect");
    assert_eq!(Interconnect::KIND.super_type_name(), "Processing");
}

/// These are the values saved data will depend on, so a change here should be deliberate
#[test]
fn keys_are_as_the_template_numbers_them() {
    let keys = CorticalAreaTypeKind::ALL.map(|kind| (kind.super_type_key(), kind.key()));
    assert_eq!(keys, [(1, 1), (1, 2), (2, 1), (2, 2), (3, 1), (4, 1), (5, 1)]);
}

#[test]
fn from_data_and_try_as() {
    type Power32 = NeuronModelFoundationPowerFloat32;
    type Power16 = NeuronModelFoundationPowerFloat16;

    let mut value = named::<Power, Power32>();
    assert_eq!(value.kind(), Power::KIND);
    assert_eq!(value.neuron_model_kind(), NeuronModelKind::FoundationPower(FoundationPowerQuantKind::Float32));
    assert_eq!(value.try_as::<Power, Power32>().unwrap(), "FoundationPower::Float32");

    value.try_as_mut::<Power, Power32>().unwrap().push('!');
    assert_eq!(value.try_as::<Power, Power32>().unwrap(), "FoundationPower::Float32!");

    assert_eq!(
        value.try_as::<Power, Power16>(),
        Err(CorticalAreaTypeSelectionError::NeuronModel(NeuronModelSelectionError::IncorrectMember {
            expected: Power16::KIND,
            actual: Power32::KIND,
        }))
    );
    assert_eq!(
        value.try_as::<Interconnect, Advanced>(),
        Err(CorticalAreaTypeSelectionError::IncorrectCorticalType { expected: Interconnect::KIND, actual: Power::KIND })
    );
    assert_eq!(value.try_into_data::<Power, Power32>().unwrap(), "FoundationPower::Float32!");
}

#[test]
fn a_cortical_type_may_allow_several_models() {
    let advanced = named::<Interconnect, Advanced>();
    let simple = named::<Interconnect, Simple>();

    assert_eq!(advanced.kind(), simple.kind());
    assert_eq!(advanced.neuron_model_kind(), Advanced::KIND);
    assert_eq!(simple.neuron_model_kind(), Simple::KIND);
    assert_eq!(simple.try_as::<Interconnect, Simple>().unwrap(), "FeagiSimple::DefaultQuant");
    assert_eq!(
        simple.try_as::<Interconnect, Advanced>(),
        Err(CorticalAreaTypeSelectionError::NeuronModel(NeuronModelSelectionError::IncorrectMember {
            expected: Advanced::KIND,
            actual: Simple::KIND,
        }))
    );
}

#[test]
fn conversions_between_the_layers() {
    let value = named::<Power, NeuronModelFoundationPowerFloat16>();
    let foundation: FoundationSuperTypeEnum<Named> = value.clone().try_into().unwrap();
    assert_eq!(foundation.kind(), FoundationSuperTypeKind::Power);

    let models: FoundationPowerModelEnum<Named> = foundation.clone().try_into().unwrap();
    assert_eq!(models.neuron_model_kind(), NeuronModelFoundationPowerFloat16::KIND);

    assert_eq!(FoundationSuperTypeEnum::from(models), foundation);
    assert_eq!(CorticalAreaTypeEnum::from(foundation), value);

    assert_eq!(
        FoundationSuperTypeEnum::try_from(named::<Interconnect, Simple>()),
        Err(CorticalAreaTypeSelectionError::IncorrectSuperType { expected: "Foundation", actual: "Processing" })
    );
    assert_eq!(
        FoundationPowerModelEnum::try_from(FoundationSuperTypeEnum::try_from(named::<Death, NeuronModelFoundationDeathFloat64>()).unwrap()),
        Err(CorticalAreaTypeSelectionError::IncorrectCorticalType { expected: Power::KIND, actual: Death::KIND })
    );
}

#[test]
fn dispatch_macro_reaches_every_cortical_type_and_model() {
    let values = [
        named::<Power, NeuronModelFoundationPowerFloat16>(),
        named::<Death, NeuronModelFoundationDeathFloat64>(),
        named::<Interconnect, Advanced>(),
        named::<Interconnect, Simple>(),
        named::<Memory, Hashing>(),
        named::<Camera, Advanced>(),
        named::<Camera, Simple>(),
        named::<Led, Advanced>(),
        named::<Led, Simple>(),
        named::<BrightnessAdjust, Advanced>(),
        named::<BrightnessAdjust, Simple>(),
    ];
    for value in values {
        let kind = feagi_genome::with_cortical_area_type!(&value, |inner| inner.neuron_model_kind());
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

/// Every level is present even with a single model, so that allowing a second model later does not
/// change how existing data is saved
#[test]
fn serde_round_trip() {
    let value = named::<Power, NeuronModelFoundationPowerFloat32>();
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, r#"{"Foundation":{"Power":{"FoundationPower":{"Float32":"FoundationPower::Float32"}}}}"#);
    assert_eq!(serde_json::from_str::<CorticalAreaTypeEnum<Named>>(&json).unwrap(), value);

    let value = named::<Memory, Hashing>();
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, r#"{"Processing":{"Memory":{"MemoryFireHashing":{"DefaultQuant":"MemoryFireHashing::DefaultQuant"}}}}"#);
    assert_eq!(serde_json::from_str::<CorticalAreaTypeEnum<Named>>(&json).unwrap(), value);
}

/// How many neurons a new area of this type and member starts with
fn default_neuron_count<T: CorticalAreaTypeMemberAt<M>, M: NeuronModelMember>() -> usize {
    <T as CorticalAreaTypeMemberAt<M>>::default_spec::<Fiq>().unwrap().iter().len()
}

#[test]
fn every_cortical_type_and_model_has_a_default_spec() {
    assert_eq!(default_neuron_count::<Power, NeuronModelFoundationPowerFloat16>(), 1);
    assert_eq!(default_neuron_count::<Power, NeuronModelFoundationPowerFloat64>(), 1);
    assert_eq!(default_neuron_count::<Death, NeuronModelFoundationDeathFloat64>(), 1);
    assert_eq!(default_neuron_count::<Interconnect, Advanced>(), 1);
    assert_eq!(default_neuron_count::<Interconnect, Simple>(), 1);
    assert_eq!(default_neuron_count::<Camera, Advanced>(), 1);
    assert_eq!(default_neuron_count::<Camera, Simple>(), 1);
    assert_eq!(default_neuron_count::<Led, Advanced>(), 1);
    assert_eq!(default_neuron_count::<Led, Simple>(), 1);
    assert_eq!(default_neuron_count::<BrightnessAdjust, Advanced>(), 1);
    assert_eq!(default_neuron_count::<BrightnessAdjust, Simple>(), 1);
    // Memory grows dynamically, so it starts empty
    assert_eq!(default_neuron_count::<Memory, Hashing>(), 0);
}

#[test]
fn genomic_data_holds_the_spec_of_a_cortical_type() {
    let spec = <Interconnect as CorticalAreaTypeMemberAt<Simple>>::default_spec::<Fiq>().unwrap();
    let data: CorticalGenomicDataEnum<Fiq> = CorticalAreaTypeEnum::from_data::<Interconnect, Simple>(spec);
    assert_eq!(data.kind(), Interconnect::KIND);
    assert_eq!(data.neuron_model_kind(), Simple::KIND);
    assert!(data.try_as::<Interconnect, Simple>().is_ok());
}
