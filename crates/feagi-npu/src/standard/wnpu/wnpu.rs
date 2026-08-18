use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantizationLevel;
use crate::standard::wnpu::wrapped_neuron_processor_unit_error::WNPUError;

pub struct WrappedNeuronProcessingUnit {
    // TODO
}

impl WrappedNeuronProcessingUnit {
    
    /// Initialized given burst engine configurations (including init hardware) and returns as a
    /// ready but empty and paused `WrappedNeuronProcessingUnit`
    pub async fn new(
        global_quantization: FeagiIndexQuantizationLevel,
        burst_engine_configurations: Vec<()>, // TODO
    ) -> Result<Self, WNPUError> {
        todo!()
    }
    
    /// Attempts to set the frequency of the burst rate to the given value, resuming if the engine
    /// was paused
    pub async fn run_at_frequency(&mut self, new_frequency: ()) -> Result<(), WNPUError> {
        todo!()
    }

    /// Pauses burst calculations but does not clear any data, NPU data continues to exist within
    /// the NPU
    pub async fn pause(&mut self) -> Result<(), WNPUError> {
        todo!()
    }
    
    /// Requests editing the connectome (adding area / mapping, etc) in some manner
    pub async fn request_connectome_change(&mut self, connectome_request: ()) -> Result<(), WNPUError> {
        todo!()
    }
    
    /// Subscribes an agent to some NPU data and vice versa
    pub async fn subscribe_agent_to_npu(&mut self, subscription_details: ()) -> Result<(), WNPUError> {
        todo!()
    }

    /// Returns a subscription of an agent, which allows the NPU to free the memory
    pub async fn unsubscribe_agent_from_npu(&mut self, subscription_details: ()) -> Result<(), WNPUError> {
        todo!()
    }
    
    
}