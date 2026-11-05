use feagi_basis::collections::spatial_context::integer_signed::SignedIntegerSpatial;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_connectome::cortical_area::{CorticalAreaEngineIndex, NeuronModelUniversalCorticalProperties};
use feagi_connectome::neuron_models::neuron_model::feagi_advanced::{self, FeagiAdvancedCorticalDataProperties, FeagiAdvancedCorticalDataVariance, NeuronModelFeagiAdvancedDefaultQuant};
use feagi_connectome::neuron_models::neuron_model::feagi_simple::{self, FeagiSimpleCorticalDataProperties, FeagiSimpleCorticalDataVariance, NeuronModelFeagiSimpleDefaultQuant};
use feagi_connectome::neuron_models::neuron_model::memory_fire_hashing::{self, MemoryFireHashingCorticalDataProperties, MemoryFireHashingCorticalDataVariance, NeuronModelMemoryFireHashingDefaultQuant};
use feagi_connectome::neuron_models::neuron_model::{FeagiAdvancedQuantEnum, FeagiSimpleQuantEnum, MemoryFireHashingQuantEnum};
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationStandard;
use feagi_genome::basis::specifications::brain_regions::BrainRegionID;
use feagi_genome::cortical_area::foundation::Power;
use feagi_genome::cortical_area::motor::Led;
use feagi_genome::cortical_area::processing::{Interconnect, Memory};
use feagi_genome::cortical_area::sensor::Camera;
use feagi_genome::cortical_area::{
    CorticalAreaTypeMap, CorticalAreaTypeMapValue, CorticalAreaTypeMember, CorticalAreaTypeRef, CorticalAreaTypeRefMut, CorticalIdentifier, ProcessingInterconnectModelEnum,
    ProcessingMemoryModelEnum, ProcessingSuperTypeRef, ProcessingSuperTypeRefMut, SensorSuperTypeRef,
};
use feagi_genome::genome::containers::cortical_area::{CorticalAreaContainerDesigner, CorticalAreaContainersDesigner, NeuronModelDataDesigner};
use feagi_genome::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use feagi_genome::genome::genomes::designer_genome::DesignerGenome;
use std::collections::HashSet;

type Fiq = FeagiIndexQuantizationStandard;

const INNER: [u8; 6] = *b"abcdef";
const OTHER: [u8; 6] = *b"zzzzzz";

/// A map holding the name of the cortical type of each area
struct Named;

impl CorticalAreaTypeMapValue for Named {
    type Of<T: CorticalAreaTypeMember> = String;
}

fn named_map() -> CorticalAreaTypeMap<Named> {
    let mut map = CorticalAreaTypeMap::<Named>::default();
    map.of_mut::<Interconnect>().insert(INNER, "interconnect".to_string());
    map.of_mut::<Memory>().insert(INNER, "memory".to_string());
    map.of_mut::<Camera>().insert(OTHER, "camera".to_string());
    map
}

#[test]
fn starts_empty() {
    let map = CorticalAreaTypeMap::<Named>::default();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);
    assert_eq!(map.identifiers().count(), 0);
}

#[test]
fn get_finds_the_area_of_the_identifier_by_its_cortical_type() {
    let map = named_map();
    let id = CorticalIdentifier::new::<Interconnect>(INNER);
    let Some(CorticalAreaTypeRef::Processing(ProcessingSuperTypeRef::Interconnect(name))) = map.get(id) else {
        panic!("not found as an interconnect")
    };
    assert_eq!(name, "interconnect");
    assert_eq!(map.get(id).unwrap().kind(), Interconnect::KIND);

    let id = CorticalIdentifier::new::<Camera>(OTHER);
    assert!(matches!(map.get(id), Some(CorticalAreaTypeRef::Sensor(SensorSuperTypeRef::Camera(name))) if name == "camera"));
}

#[test]
fn the_same_inner_id_is_a_separate_area_per_cortical_type() {
    let map = named_map();
    assert!(map.contains(CorticalIdentifier::new::<Interconnect>(INNER)));
    assert!(map.contains(CorticalIdentifier::new::<Memory>(INNER)));
    assert!(!map.contains(CorticalIdentifier::new::<Camera>(INNER)));
    assert!(!map.contains(CorticalIdentifier::new::<Power>(INNER)));
    assert!(map.get(CorticalIdentifier::new::<Led>(INNER)).is_none());
    assert_eq!(map.of::<Memory>()[&INNER], "memory");
    assert_eq!(map.of::<Interconnect>()[&INNER], "interconnect");
}

#[test]
fn get_mut_edits_what_is_held() {
    let mut map = named_map();
    let id = CorticalIdentifier::new::<Memory>(INNER);
    let Some(CorticalAreaTypeRefMut::Processing(ProcessingSuperTypeRefMut::Memory(name))) = map.get_mut(id) else {
        panic!("not found as memory")
    };
    name.push('!');
    assert_eq!(map.of::<Memory>()[&INNER], "memory!");
    assert_eq!(map.of::<Interconnect>()[&INNER], "interconnect");
    assert_eq!(map.get_mut(id).unwrap().kind(), Memory::KIND);
}

#[test]
fn identifiers_and_lengths_cover_every_area() {
    let mut map = named_map();
    assert_eq!(map.len(), 3);
    assert_eq!(map.processing.len(), 2);
    assert_eq!(map.sensor.len(), 1);
    assert!(map.foundation.is_empty());

    let identifiers: HashSet<CorticalIdentifier> = map.identifiers().collect();
    let expected = HashSet::from([
        CorticalIdentifier::new::<Interconnect>(INNER),
        CorticalIdentifier::new::<Memory>(INNER),
        CorticalIdentifier::new::<Camera>(OTHER),
    ]);
    assert_eq!(identifiers, expected);

    map.of_mut::<Memory>().remove(&INNER);
    assert_eq!(map.len(), 2);
    assert!(!map.contains(CorticalIdentifier::new::<Memory>(INNER)));
}

fn universal<NM: feagi_connectome::cortical_area::NeuronModel>() -> NeuronModelUniversalCorticalProperties<NM> {
    NeuronModelUniversalCorticalProperties { cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE, is_psp_uniform: false, is_psp_using_neuron_potential: false }
}

fn container<CT: CorticalAreaTypeMember>(
    layout_context: <<CT::NeuronLayout as feagi_genome::cortical_area::NeuronLayoutTag>::Layout<Fiq> as feagi_connectome::cortical_area::neuron_layout::NeuronLayout<Fiq>>::CorticalLevelContext,
    neuron_model_data: CT::Models<feagi_genome::genome::containers::cortical_area::NeuronModelDataDesignerContainer<Fiq>>,
) -> CorticalAreaContainerDesigner<Fiq, CT> {
    CorticalAreaContainerDesigner {
        metadata: GenomeObjectMetadata { name: "area".to_string(), description: String::new() },
        positioning: GenomeObjectPositioning {
            position_2d: SignedIntegerSpatial::new_from_array([0, 0]),
            position_3d: SignedIntegerSpatial::new_from_array([0, 0, 0]),
        },
        parent_id: BrainRegionID::new_now(),
        incoming_mappings: HashSet::new(),
        outgoing_mappings: HashSet::new(),
        layout_context,
        neuron_model_data,
    }
}

fn simple_data() -> NeuronModelDataDesigner<Fiq, NeuronModelFeagiSimpleDefaultQuant> {
    NeuronModelDataDesigner {
        cortical_universal: universal(),
        cortical_properties: FeagiSimpleCorticalDataProperties::new(feagi_simple::Period::QUANT_ONE, 7.0, 1.0, 1.0),
        cortical_data_variance: FeagiSimpleCorticalDataVariance::new(),
        cortical_npu_index: CorticalAreaEngineIndex::QUANT_ZERO,
        burst_engine_index: 0,
    }
}

fn advanced_data() -> NeuronModelDataDesigner<Fiq, NeuronModelFeagiAdvancedDefaultQuant> {
    NeuronModelDataDesigner {
        cortical_universal: universal(),
        cortical_properties: FeagiAdvancedCorticalDataProperties::new(feagi_advanced::Period::QUANT_ONE, 9.0, 1.0, 1.0, 1.0, 1.0, 1.0),
        cortical_data_variance: FeagiAdvancedCorticalDataVariance::new(0.0, 0.0, 0.0),
        cortical_npu_index: CorticalAreaEngineIndex::QUANT_ZERO,
        burst_engine_index: 0,
    }
}

fn dimensions() -> DimensionalCorticalAreaDimensions<<Fiq as feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization>::NeuronIndexQuant> {
    DimensionalCorticalAreaDimensions::new_dimensions_from_usize([1, 1, 1, 1]).unwrap()
}

/// The whole path of the design, by ID: cortical type, then the models it allows, then the
/// quantizations of that model, then the data
#[test]
fn designer_containers_hold_only_what_their_cortical_type_allows() {
    let mut map = CorticalAreaTypeMap::<CorticalAreaContainersDesigner<Fiq>>::default();
    map.of_mut::<Interconnect>().insert(INNER, container::<Interconnect>(dimensions(), ProcessingInterconnectModelEnum::FeagiSimple(FeagiSimpleQuantEnum::DefaultQuant(simple_data()))));
    map.of_mut::<Memory>().insert(
        INNER,
        container::<Memory>(
            CorticalAreaNeuronCount::QUANT_ZERO,
            ProcessingMemoryModelEnum::MemoryFireHashing(MemoryFireHashingQuantEnum::DefaultQuant(NeuronModelDataDesigner::<Fiq, NeuronModelMemoryFireHashingDefaultQuant> {
                cortical_universal: universal(),
                cortical_properties: MemoryFireHashingCorticalDataProperties::new(memory_fire_hashing::Period::QUANT_ONE, 3.0, 1.0, 1.0),
                cortical_data_variance: MemoryFireHashingCorticalDataVariance::new(),
                cortical_npu_index: CorticalAreaEngineIndex::QUANT_ZERO,
                burst_engine_index: 0,
            })),
        ),
    );

    // Exhaustive with only the models interconnect allows
    let threshold = match map.get(CorticalIdentifier::new::<Interconnect>(INNER)) {
        Some(CorticalAreaTypeRef::Processing(ProcessingSuperTypeRef::Interconnect(area))) => match &area.neuron_model_data {
            ProcessingInterconnectModelEnum::FeagiSimple(FeagiSimpleQuantEnum::DefaultQuant(data)) => data.cortical_properties.firing_threshold,
            ProcessingInterconnectModelEnum::FeagiAdvanced(FeagiAdvancedQuantEnum::DefaultQuant(data)) => data.cortical_properties.firing_threshold,
        },
        _ => panic!("not found as an interconnect"),
    };
    assert_eq!(threshold, 7.0);

    let threshold = match &map.of::<Memory>()[&INNER].neuron_model_data {
        ProcessingMemoryModelEnum::MemoryFireHashing(MemoryFireHashingQuantEnum::DefaultQuant(data)) => data.cortical_properties.firing_threshold,
    };
    assert_eq!(threshold, 3.0);
}

/// The identifier stays the same and only the data changes
#[test]
fn a_model_swap_replaces_the_data_in_place() {
    let mut map = CorticalAreaTypeMap::<CorticalAreaContainersDesigner<Fiq>>::default();
    map.of_mut::<Interconnect>().insert(INNER, container::<Interconnect>(dimensions(), ProcessingInterconnectModelEnum::FeagiSimple(FeagiSimpleQuantEnum::DefaultQuant(simple_data()))));

    map.of_mut::<Interconnect>().get_mut(&INNER).unwrap().neuron_model_data = ProcessingInterconnectModelEnum::FeagiAdvanced(FeagiAdvancedQuantEnum::DefaultQuant(advanced_data()));

    assert_eq!(map.len(), 1);
    assert_eq!(map.of::<Interconnect>()[&INNER].neuron_model_data.neuron_model_kind(), <NeuronModelFeagiAdvancedDefaultQuant as feagi_connectome::neuron_models::neuron_model::NeuronModelMember>::KIND);
}

#[test]
fn designer_genome_builds_with_the_nested_map() {
    let _genome = DesignerGenome::<Fiq>::new_genome_now("test".to_string());
}
