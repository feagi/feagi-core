use feagi_connectome::connectome_basis::neuron_model;
use feagi_connectome::connectome_basis::neuron_model::feagi_advanced::NMFeagiAdvancedQuantDefaultQuant;
use feagi_connectome::connectome_basis::neuron_model::feagi_simple::NMFeagiSimpleQuantDefaultQuant;
use feagi_connectome::connectome_basis::neuron_model::foundation_death::NMFoundationDeathQuantFloat64;
use feagi_connectome::connectome_basis::neuron_model::foundation_power::{NMFoundationPowerQuantFloat16, NMFoundationPowerQuantFloat32, NMFoundationPowerQuantFloat64};
use feagi_connectome::connectome_basis::neuron_model::memory_fire_hashing::NMMemoryFireHashingQuantDefaultQuant;
use feagi_connectome::connectome_basis::neuron_model::{NMFoundationPowerQuantKind, NMContainer, NMKind, NMMember, NMAccessError};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantizationStandard;
use feagi_connectome::{
    with_nm_feagi_advanced, with_nm_feagi_simple, with_nm_foundation_death, with_nm_foundation_power,
    with_nm_memory_fire_hashing,
};
use feagi_genome::basis::cortical_area::cortical_area_type;
use feagi_genome::basis::cortical_area::cortical_area_type::{
    CorticalAreaTypeEnum, CorticalAreaTypeKind, CorticalAreaTypeMember, CorticalAreaTypeMemberAt, CorticalAreaTypeSelectionError, FoundationPowerModelEnum, FoundationSuperTypeEnum,
    FoundationSuperTypeKind, MotorSuperTypeKind, ProcessingSuperTypeKind, SensorSuperTypeKind, TunableSuperTypeKind,
};
use feagi_genome::basis::cortical_area::foundation::{Death, Power};
use feagi_genome::basis::cortical_area::motor::Led;
use feagi_genome::basis::cortical_area::processing::{Interconnect, Memory};
use feagi_genome::basis::cortical_area::sensor::Camera;
use feagi_genome::basis::cortical_area::tunable::BrightnessAdjust;
use feagi_genome::basis::cortical_area::NeurogenesisFromStorageOrByGeneratorCorticalTypeEnum;
use feagi_connectome::connectome_basis::neuron_model::NeuronModel;
use feagi_connectome::connectome_basis::neuron_model::{
    NeurogenesisFromStorageOrByGeneratorOf, NeurogenesisDefaultsGenerator, NeurogenesisMethod, NeuronLayoutData, NeuronLayoutOf, NeuronLayoutSingleSteppedDynamic, NeuronLayoutVoxel,
};
use feagi_connectome::connectome_basis::neuron_model::feagi_advanced::{NMFeagiAdvancedDataNeuronInternal, Period};
use feagi_genome::basis::cortical_area::processing::{InterconnectFeagiAdvancedGenerator, InterconnectFeagiSimpleGenerator};
use feagi_basis::neurons::{CorticalAreaNeuronCount, DimensionalCorticalAreaDimensions};

type Fiq = FeagiIndexQuantizationStandard;
type Advanced = NMFeagiAdvancedQuantDefaultQuant;
type Simple = NMFeagiSimpleQuantDefaultQuant;
type Hashing = NMMemoryFireHashingQuantDefaultQuant;

/// A container holding the name of the neuron model member it is held for
struct Named;

impl NMContainer for Named {
    type Of<M: NeuronModel> = String;
}

fn named<T: CorticalAreaTypeMemberAt<M>, M: NMMember>() -> CorticalAreaTypeEnum<Named> {
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
    type Power32 = NMFoundationPowerQuantFloat32;
    type Power16 = NMFoundationPowerQuantFloat16;

    let mut value = named::<Power, Power32>();
    assert_eq!(value.kind(), Power::KIND);
    assert_eq!(value.neuron_model_kind(), NMKind::FoundationPower(NMFoundationPowerQuantKind::Float32));
    assert_eq!(value.try_as::<Power, Power32>().unwrap(), "FoundationPower::Float32");

    value.try_as_mut::<Power, Power32>().unwrap().push('!');
    assert_eq!(value.try_as::<Power, Power32>().unwrap(), "FoundationPower::Float32!");

    assert_eq!(
        value.try_as::<Power, Power16>(),
        Err(CorticalAreaTypeSelectionError::NeuronModel(NMAccessError::IncorrectMember {
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
        Err(CorticalAreaTypeSelectionError::NeuronModel(NMAccessError::IncorrectMember {
            expected: Advanced::KIND,
            actual: Simple::KIND,
        }))
    );
}

#[test]
fn conversions_between_the_layers() {
    let value = named::<Power, NMFoundationPowerQuantFloat16>();
    let foundation: FoundationSuperTypeEnum<Named> = value.clone().try_into().unwrap();
    assert_eq!(foundation.kind(), FoundationSuperTypeKind::Power);

    let models: FoundationPowerModelEnum<Named> = foundation.clone().try_into().unwrap();
    assert_eq!(models.neuron_model_kind(), NMFoundationPowerQuantFloat16::KIND);

    assert_eq!(FoundationSuperTypeEnum::from(models), foundation);
    assert_eq!(CorticalAreaTypeEnum::from(foundation), value);

    assert_eq!(
        FoundationSuperTypeEnum::try_from(named::<Interconnect, Simple>()),
        Err(CorticalAreaTypeSelectionError::IncorrectSuperType { expected: "Foundation", actual: "Processing" })
    );
    assert_eq!(
        FoundationPowerModelEnum::try_from(FoundationSuperTypeEnum::try_from(named::<Death, NMFoundationDeathQuantFloat64>()).unwrap()),
        Err(CorticalAreaTypeSelectionError::IncorrectCorticalType { expected: Power::KIND, actual: Death::KIND })
    );
}

#[test]
fn dispatch_macro_reaches_every_cortical_type_and_model() {
    let values = [
        named::<Power, NMFoundationPowerQuantFloat16>(),
        named::<Death, NMFoundationDeathQuantFloat64>(),
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
            <NM as NMMember>::KIND,
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
    let value = named::<Power, NMFoundationPowerQuantFloat32>();
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, r#"{"Foundation":{"Power":{"FoundationPower":{"Float32":"FoundationPower::Float32"}}}}"#);
    assert_eq!(serde_json::from_str::<CorticalAreaTypeEnum<Named>>(&json).unwrap(), value);

    let value = named::<Memory, Hashing>();
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, r#"{"Processing":{"Memory":{"MemoryFireHashing":{"DefaultQuant":"MemoryFireHashing::DefaultQuant"}}}}"#);
    assert_eq!(serde_json::from_str::<CorticalAreaTypeEnum<Named>>(&json).unwrap(), value);
}

/// A new area of this type and member, from the defaults of the cortical type
fn generator_from_defaults<T: CorticalAreaTypeMemberAt<M>, M: NMMember>(layout: NeuronLayoutOf<M, Fiq>) -> NeurogenesisDefaultsGenerator<Fiq, M> {
    <T as CorticalAreaTypeMemberAt<M>>::new_defaults_generator::<Fiq>(layout).unwrap()
}

/// How many neurons a new area of this type and member starts with, generated from its defaults
fn generated_neuron_count<T: CorticalAreaTypeMemberAt<M>, M: NMMember>(layout: NeuronLayoutOf<M, Fiq>) -> usize {
    generator_from_defaults::<T, M>(layout).iter().len()
}

/// A voxel layout of 2 neurons
fn two_voxels() -> NeuronLayoutVoxel<Fiq> {
    NeuronLayoutVoxel::new(DimensionalCorticalAreaDimensions::new_dimensions_from_usize([2, 1, 1, 1]).unwrap())
}

#[test]
fn every_cortical_type_and_model_generates_from_its_defaults() {
    assert_eq!(generated_neuron_count::<Power, NMFoundationPowerQuantFloat16>(two_voxels()), 2);
    assert_eq!(generated_neuron_count::<Power, NMFoundationPowerQuantFloat64>(two_voxels()), 2);
    assert_eq!(generated_neuron_count::<Death, NMFoundationDeathQuantFloat64>(two_voxels()), 2);
    assert_eq!(generated_neuron_count::<Interconnect, Advanced>(two_voxels()), 2);
    assert_eq!(generated_neuron_count::<Interconnect, Simple>(two_voxels()), 2);
    assert_eq!(generated_neuron_count::<Camera, Advanced>(two_voxels()), 2);
    assert_eq!(generated_neuron_count::<Camera, Simple>(two_voxels()), 2);
    assert_eq!(generated_neuron_count::<Led, Advanced>(two_voxels()), 2);
    assert_eq!(generated_neuron_count::<Led, Simple>(two_voxels()), 2);
    assert_eq!(generated_neuron_count::<BrightnessAdjust, Advanced>(two_voxels()), 2);
    assert_eq!(generated_neuron_count::<BrightnessAdjust, Simple>(two_voxels()), 2);
    // Memory grows dynamically, so it starts empty
    assert_eq!(generated_neuron_count::<Memory, Hashing>(NeuronLayoutSingleSteppedDynamic::new(CorticalAreaNeuronCount::QUANT_ZERO)), 0);
}

/// The generic access through the family gives the generator the wrapper of the cortical type
/// builds
#[test]
fn family_access_gives_the_generator_of_the_wrapper() {
    let wrapper = InterconnectFeagiAdvancedGenerator::<Fiq, Advanced>::new(two_voxels()).unwrap();
    assert_eq!(generator_from_defaults::<Interconnect, Advanced>(two_voxels()), wrapper.0);
    let wrapper = InterconnectFeagiSimpleGenerator::<Fiq, Simple>::new(two_voxels()).unwrap();
    assert_eq!(generator_from_defaults::<Interconnect, Simple>(two_voxels()), wrapper.0);
}

/// A client edits the defaults of the wrapper, which the generator of the model then holds
#[test]
fn edited_defaults_become_the_model_generator() {
    let mut wrapper = InterconnectFeagiAdvancedGenerator::<Fiq, Advanced>::new(two_voxels()).unwrap();
    wrapper.0.cortical_properties.firing_threshold = 7.0;
    let expected_properties = wrapper.0.cortical_properties.clone();
    let expected_internal = wrapper.0.cortical_internal.clone();

    let generator: <Advanced as NeuronModel>::NeurogenesisGenerator<Fiq> = wrapper.into_model_generator();
    assert_eq!(generator.neuron_internal, NMFeagiAdvancedDataNeuronInternal::new(Period::QUANT_ZERO));
    assert_eq!(generator.get_layout(), &two_voxels());
    assert_eq!(generator.into_model_cortical_data(), (expected_properties, expected_internal));
}

#[test]
fn genomic_data_holds_the_neurogenesis_of_a_cortical_type() {
    let generator = InterconnectFeagiSimpleGenerator::<Fiq, Simple>::new(two_voxels()).unwrap().into_model_generator();
    let data: NeurogenesisFromStorageOrByGeneratorCorticalTypeEnum<Fiq> = CorticalAreaTypeEnum::from_data::<Interconnect, Simple>(NeurogenesisFromStorageOrByGeneratorOf::ByGenerator(generator));
    assert_eq!(data.kind(), Interconnect::KIND);
    assert_eq!(data.neuron_model_kind(), Simple::KIND);
    assert!(matches!(data.try_as::<Interconnect, Simple>(), Ok(NeurogenesisFromStorageOrByGeneratorOf::ByGenerator(_))));
}
