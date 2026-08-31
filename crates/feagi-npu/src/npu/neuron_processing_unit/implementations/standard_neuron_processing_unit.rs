use std::thread::{JoinHandle};
use crate::npu::neuron_processing_unit::neuron_processing_unit::{ComposableNeuralProcessingUnit, NeuralProcessingUnit};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_npu_burst_engines::BurstEngineEnum;
use crate::npu::neuron_processing_unit::neuron_processing_unit_compose_messaging::{NeuronProcessingUnitComposeRequest, NeuronProcessingUnitComposeResponse};

pub struct StandardNeuronProcessingUnit<FIQ: FeagiIndexQuantization> {
    worker_pool: PoolState<FIQ>
}

impl<FIQ: FeagiIndexQuantization> StandardNeuronProcessingUnit<FIQ> {
    pub fn new() -> Self {
        
        // TODO allow specifying desired burst engines
        
        let burst_engine = BurstEngineEnum::new_cpu_rayon();
        let burst_engines = vec![burst_engine];
        
        Self {
            worker_pool: PoolState::Paused(burst_engines)
        }
    }
}

impl<FIQ: FeagiIndexQuantization> NeuralProcessingUnit<FIQ> for StandardNeuronProcessingUnit<FIQ> {
    async fn run_at_global_frequency(&mut self) -> () {}

    async fn run_as_fast_as_possible(&mut self) -> () {
        todo!()
    }

    async fn pause_engines(&mut self) -> () { // TODO
    }
}

impl<FIQ: FeagiIndexQuantization> ComposableNeuralProcessingUnit<FIQ> for StandardNeuronProcessingUnit<FIQ> {
    // composable
    async fn request_connectome_change(&mut self, request: NeuronProcessingUnitComposeRequest) -> NeuronProcessingUnitComposeResponse {

    }
}


enum PoolState<FIQ: FeagiIndexQuantization> {
    Paused(Vec<BurstEngineEnum<FIQ>>),
    Running(JoinHandle<()>)
}