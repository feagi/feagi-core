//! A temporary wrapper

use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationGenomic;

use crate::npu::neuron_processor_unit_composable::{NeuronProcessingUnitComposable};
use crate::npu::npu_target_frequency::NPUTargetFrequency;

/// Compatibility wrapper for the new NPU as it is developed and as we use the old FEAGI architecture.
/// Can be owned directly, handles the threading and quantization shenanigans internally
pub struct WrappedNeuronProcessingUnit {
    npu: NeuronProcessingUnitComposable<FeagiIndexQuantizationGenomic>
}

impl WrappedNeuronProcessingUnit {

    pub fn new() -> WrappedNeuronProcessingUnit {
        Self {
            npu: NeuronProcessingUnitComposable::new()
        }
    }

    /// Start engines at a frequency or update the frequency if its already running
    pub fn run_at(&mut self, burst_frequency: NPUTargetFrequency) {
        self.npu.start_engines(burst_frequency)
    }

    /// Stop the NPU from running. Best called during shutdown for now
    pub fn stop_npu(&mut self) {
        self.npu.stop_engines()
    }

    /// Submit  change request (adding cortical area, mapping, etc) to the NPU.Change will be implemented next best oppertunity
    pub fn request_change(&mut self,) {
        todo!()
    }

    pub fn register_agent(&mut self) {
        todo!()
    }
}