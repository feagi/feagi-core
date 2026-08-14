//! A temporary wrapper

use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationGenomic;
use crate::npu::neuron_processing_unit_commands::BurstFrequency;
use crate::npu::neuron_processor_unit_composable::{NeuronProcessingUnitComposable};

/// Compatibility wrapper for the new NPU as it is developed and as we use the old FEAGI architecture.
/// Can be owned directly, handles the threading shenanigans internally
pub struct WrappedNeuronProcessingUnit {
    npu: NeuronProcessingUnitComposable<FeagiIndexQuantizationGenomic>
}

impl WrappedNeuronProcessingUnit {

    pub fn new(initial_frequency: BurstFrequency) -> WrappedNeuronProcessingUnit {
        Self {
            npu: NeuronProcessingUnitComposable::new(initial_frequency)
        }
    }
    
    pub fn run_at(&mut self, burst_frequency: BurstFrequency) {
        todo!()
    }

    pub fn pause(&mut self, ) {
        self.npu.stop_engines()
    }

    pub fn request_change(&mut self,) {
        todo!()
    }

    pub fn register_agent(&mut self) {
        todo!()
    }
}