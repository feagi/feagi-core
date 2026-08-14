//! A temporary wrapper

use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationGenomic;
use crate::npu::neuron_processor_unit_composable::{BurstFrequency, NeuronProcessingUnitComposable};

/// Compatibility wrapper for the new NPU as it is developed and as we use the old FEAGI architecture.
/// Can be owned directly, handles the threading shenanigans internally
pub struct WrappedNeuronProcessingUnit {
    npu: Option<NeuronProcessingUnitComposable<FeagiIndexQuantizationGenomic>>
}

impl WrappedNeuronProcessingUnit {

    pub fn new() -> WrappedNeuronProcessingUnit {
        todo!()
    }

    pub fn init_burst_engines(& mut self) {
        // TODO take some parameters to choose what burst engines we want

        let generate_npu = NeuronProcessingUnitComposable::new();




        self.npu = Some(generate_npu.npu)

    }

    pub fn run_at(&mut self, burst_frequency: BurstFrequency) {
        todo!()
    }

    pub fn pause(&mut self, ) {
        todo!()
    }

    pub fn request_change(&mut self,) {
        todo!()
    }

    pub fn register_agent(&mut self) {
        todo!()
    }

}