//! Genome edit sequences sent through the genome manager's command channel, editing the connectome
//! of a neural processor running the CPU burst engine
#![cfg(feature = "std")]

use feagi_basis::collections::spatial_context::integer_signed::SignedIntegerSpatial;
use feagi_basis::neurons::DimensionalCorticalAreaDimensions;
use feagi_basis::parallelism::block_on_thread;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;
use feagi_basis::target_frequency::TargetFrequency;
use feagi_connectome::connectome_basis::engine::BurstEngineCPURayon;
use feagi_connectome::connectome_basis::neuron_model::feagi_advanced::NeuronModelFeagiAdvancedDefaultQuant;
use feagi_connectome::connectome_basis::neuron_model::foundation_power::NeuronModelFoundationPowerFloat32;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantizationStandard;
use feagi_connectome::neural_processors::singular::{NeuronProcessorSingleEngineVisualizationTransmitter, NeuronProcessorSingleEngineWorkerBuilder};
use feagi_genome::basis::specifications::brain_regions::BrainRegionID;
use feagi_genome::cortical_area::foundation::Power;
use feagi_genome::cortical_area::processing::Interconnect;
use feagi_genome::cortical_area::{CorticalAreaTypeEnum, CorticalIdentifierPath};
use feagi_genome::genome::containers::cortical_area::{CorticalAreaGenerator, CorticalAreaGeneratorEnum};
use feagi_genome::genome::genome_editing::{GenomeEditError, GenomeEditSequence, GenomeEditStepResult};
use feagi_genome::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use feagi_genome::workers::genome_manager::worker_builder::GenomeManagerWorkerBuilder;
use feagi_genome::workers::genome_manager::worker_loop::{GenomeManagerRequest, GenomeManagerResponse};

type Fiq = FeagiIndexQuantizationStandard;
type Engine = BurstEngineCPURayon<Fiq, NeuronProcessorSingleEngineVisualizationTransmitter<Fiq>>;

fn metadata(name: &str) -> GenomeObjectMetadata {
    GenomeObjectMetadata { name: name.to_string(), description: String::new() }
}

fn origin() -> GenomeObjectPositioning {
    GenomeObjectPositioning {
        position_2d: SignedIntegerSpatial::new_from_array([0, 0]),
        position_3d: SignedIntegerSpatial::new_from_array([0, 0, 0]),
    }
}

fn dimensions(x: usize, y: usize, z: usize) -> DimensionalCorticalAreaDimensions<u32> {
    DimensionalCorticalAreaDimensions::new_dimensions_from_usize([x, y, z, 1]).unwrap()
}

#[test]
fn edit_sequences_add_brain_regions_and_cortical_areas() {
    let spawner = FeagiStdThreadSpawner;

    let neural_processor_builder = NeuronProcessorSingleEngineWorkerBuilder::<Fiq, Engine> {
        initial_target_frequency: TargetFrequency::new_from_frequency(100.0),
        initial_visualization_frequency: TargetFrequency::new_from_frequency(100.0),
        engine_init_parameters: (),
    };
    let (visualization_transmitter, _visualization_receiver) = neural_processor_builder.spawn_related_channel_pairs();
    let (mut neural_processor, neural_processor_commands) = neural_processor_builder.build_worker(&spawner, visualization_transmitter).unwrap();

    let genome_manager_builder = GenomeManagerWorkerBuilder::<Fiq>::default();
    let (_requesters, responders) = genome_manager_builder.spawn_related_channel_pairs();
    let (mut genome_manager, ()) = genome_manager_builder.build_worker(&spawner, (responders, neural_processor_commands)).unwrap();

    block_on_thread(async {
        let GenomeManagerResponse::RootBrainRegion(root) = genome_manager.make_request(GenomeManagerRequest::RootBrainRegion).await.unwrap() else {
            panic!("wrong response");
        };

        // Brain regions only live in the genome
        let mut sequence = GenomeEditSequence::new();
        sequence.brain_region().add_now(root, metadata("Region"), origin()).unwrap();
        let region = match apply(&mut genome_manager, sequence).await.unwrap().as_slice() {
            [GenomeEditStepResult::BrainRegionAdded(region)] => *region,
            other => panic!("unexpected results {other:?}"),
        };

        let power: CorticalAreaGeneratorEnum<Fiq> = CorticalAreaTypeEnum::from_data::<Power, NeuronModelFoundationPowerFloat32>(
            CorticalAreaGenerator::from_defaults::<Power>(dimensions(1, 1, 1)).unwrap(),
        );
        let mut sequence = GenomeEditSequence::new();
        sequence.cortical_area().add(0, power, root, metadata("Power"), origin()).unwrap();
        let power_area = match apply(&mut genome_manager, sequence).await.unwrap().as_slice() {
            [GenomeEditStepResult::CorticalAreaAdded(area)] => *area,
            other => panic!("unexpected results {other:?}"),
        };
        assert!(matches!(power_area.path(), CorticalIdentifierPath::Foundation(_)));

        let interconnect: CorticalAreaGeneratorEnum<Fiq> = CorticalAreaTypeEnum::from_data::<Interconnect, NeuronModelFeagiAdvancedDefaultQuant>(
            CorticalAreaGenerator::from_defaults::<Interconnect>(dimensions(10, 10, 10)).unwrap(),
        );
        let mut sequence = GenomeEditSequence::new();
        sequence.cortical_area().add(1, interconnect, region, metadata("Interconnect"), origin()).unwrap();
        let interconnect_area = match apply(&mut genome_manager, sequence).await.unwrap().as_slice() {
            [GenomeEditStepResult::CorticalAreaAdded(area)] => *area,
            other => panic!("unexpected results {other:?}"),
        };
        assert!(matches!(interconnect_area.path(), CorticalIdentifierPath::Processing(_)));
        assert_ne!(power_area, interconnect_area);

        // A step that does not hold up against the genome rejects the whole sequence
        let unknown_region = BrainRegionID::new_now();
        let power: CorticalAreaGeneratorEnum<Fiq> = CorticalAreaTypeEnum::from_data::<Power, NeuronModelFoundationPowerFloat32>(
            CorticalAreaGenerator::from_defaults::<Power>(dimensions(1, 1, 1)).unwrap(),
        );
        let mut sequence = GenomeEditSequence::new();
        sequence.cortical_area().add(2, power, unknown_region, metadata("Lost"), origin()).unwrap();
        assert!(matches!(apply(&mut genome_manager, sequence).await, Err(GenomeEditError::BrainRegionNotFound(missing)) if missing == unknown_region));

        genome_manager.stop().await.unwrap();
        neural_processor.stop().await.unwrap();
    });
    genome_manager.join().unwrap();
    neural_processor.join().unwrap();
}

async fn apply(
    genome_manager: &mut feagi_basis::parallelism::feagi_worker::feagi_worker::FeagiWorker<
        FeagiStdThreadSpawner,
        feagi_basis::parallelism::feagi_channel::request_response_channel::dynamic::FeagiRequestResponseChannelsAllocBackend,
        GenomeManagerRequest<Fiq>,
        GenomeManagerResponse,
    >,
    sequence: GenomeEditSequence<Fiq>,
) -> Result<Vec<GenomeEditStepResult>, GenomeEditError> {
    match genome_manager.make_request(GenomeManagerRequest::ApplyEditSequence(sequence)).await.unwrap() {
        GenomeManagerResponse::EditSequenceApplied(results) => results,
        _ => panic!("wrong response"),
    }
}
