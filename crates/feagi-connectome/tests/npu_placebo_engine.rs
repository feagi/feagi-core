#![cfg(feature = "burst-cpu")]
use feagi_basis::neurons::{CorticalAreaNeuronPotential, DimensionalCorticalAreaDimensions};
use feagi_basis::parallelism::block_on_thread;
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_basis::parallelism::feagi_worker::worker_internals::worker_spawner_implementation::impl_std::FeagiStdThreadSpawner;
use feagi_basis::prelude::*;
use feagi_basis::target_frequency::TargetFrequency;
use feagi_connectome::connectome_basis::connectome_editing::*;
use feagi_connectome::connectome_basis::neuron_model::model_data::NeuronModelUniversalCorticalProperties;
use feagi_connectome::connectome_basis::neuron_model::neurogenesis::NeurogenesisStorageOrGenerator;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantizationStandard;
use feagi_connectome::connectome_burst::burst_cpu::placebo_engine::BurstEngineCPUPlacebo;
use feagi_connectome::connectome_models::neural_processors::singular::commands::{NeuronProcessorSingleEngineRequest, NeuronProcessorSingleEngineResponse};
use feagi_connectome::connectome_models::neural_processors::singular::NeuronProcessorSingleEngineWorkerBuilder;
use feagi_connectome::connectome_models::neuron_models::models::exclusive_power::quantizations::quant_float_32::NMExclusivePowerQuantFloat32;
use feagi_connectome::connectome_models::neuron_models::models::exclusive_power::{
    NMExclusivePowerDataCorticalInternal, NMExclusivePowerDataCorticalProperties, NMExclusivePowerDataCorticalVariance, NMExclusivePowerDataNeuronInternal,
};
use feagi_connectome::connectome_models::neuron_models::models::{NMMember, NMModeled, NMNeurogenesisStorageOrGenerator};
use feagi_connectome::connectome_models::neuron_models::neuron_model_types::FeagiNeuronModelTypes;
use feagi_connectome::connectome_models::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;

type Fiq = FeagiIndexQuantizationStandard;
type Engine = BurstEngineCPUPlacebo<Fiq, FeagiNeuronModelTypes, ()>;
type Nm = NMExclusivePowerQuantFloat32;

fn hz(f: f64) -> TargetFrequency { TargetFrequency::new_from_frequency(f).unwrap() }

fn power_area(dims: [usize; 4]) -> ConnectomeEditStepEnum<Fiq, FeagiNeuronModelTypes> {
    let generator = NeurogenesisGeneratorUniformDefault::<Fiq, Nm> {
        universal_properties: NeuronModelUniversalCorticalProperties {
            cortical_level_psp: CorticalAreaNeuronPotential::QUANT_ONE,
            is_psp_uniform: false,
            is_psp_using_neuron_potential: false,
        },
        cortical_context: DimensionalCorticalAreaDimensions::new_dimensions_from_usize(dims).unwrap(),
        cortical_properties: NMExclusivePowerDataCorticalProperties::new(),
        cortical_internal: NMExclusivePowerDataCorticalInternal::new(),
        cortical_variance: NMExclusivePowerDataCorticalVariance::new(),
        neuron_internal: NMExclusivePowerDataNeuronInternal::new(),
    };
    let neurogenesis = NeurogenesisStorageOrGenerator::Generator(generator);
    let modeled: NMModeled<NMNeurogenesisStorageOrGenerator<Fiq>> = Nm::wrap::<NMNeurogenesisStorageOrGenerator<Fiq>>(neurogenesis).into();
    ConnectomeEditStepCorticalAreaAdd::<Fiq, FeagiNeuronModelTypes>::new(modeled).into()
}

#[test]
fn placebo_engine_through_npu() {
    let builder = NeuronProcessorSingleEngineWorkerBuilder::<Fiq, Engine> {
        initial_target_frequency: hz(200.0),
        initial_visualization_frequency: hz(100.0),
        engine_init_parameters: (),
    };
    let (tx, _rx) = builder.spawn_related_channel_pairs();
    let (mut worker, ()) = builder.build_worker(&FeagiStdThreadSpawner, tx).unwrap();
    block_on_thread(async {
        let steps = vec![ConnectomeEditStepEnum::Nothing, power_area([2, 3, 1, 1]), power_area([1, 1, 1, 1])];
        let response = worker.make_request(NeuronProcessorSingleEngineRequest::ConnectomeEdits(steps)).await.unwrap();
        let results = match response {
            NeuronProcessorSingleEngineResponse::ConnectomeEdits(results) => results,
            _ => panic!("wrong response"),
        };
        assert_eq!(results.len(), 3);
        for r in &results {
            match r {
                Ok(ConnectomeEditResultantEnum::Nothing) => println!("nothing"),
                Ok(ConnectomeEditResultantEnum::CorticalAreaAdd(Ok(a))) => println!("added idx {} vis {}", a.new_cortical_index.quant_to_usize(), a.visualization_start_index.quant_to_usize()),
                _ => panic!("step failed"),
            }
        }
        let response = worker.make_request(NeuronProcessorSingleEngineRequest::Resume).await.unwrap();
        assert!(matches!(response, NeuronProcessorSingleEngineResponse::Resumed));
        std::thread::sleep(std::time::Duration::from_millis(100));
        worker.stop().await.unwrap();
    });
    worker.join().unwrap();
}
