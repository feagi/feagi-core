use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::npu::neuron_processing_unit::neuron_processing_unit_compose_messaging::NeuronProcessingUnitComposeRequest;

/// Contains logic for continuously processing neuron dynamics in a loop on some backend(s)
pub trait NeuralProcessingUnit<FIQ: FeagiIndexQuantization> {
    async fn run_at_global_frequency(&mut self) -> ();
    
    async fn run_as_fast_as_possible(&mut self) -> ();
    async fn pause_engines(&mut self) -> ();
}

/// Allows the call for editing the connectome that the burst engines are executing upon
pub trait ComposableNeuralProcessingUnit<FIQ: FeagiIndexQuantization>: NeuralProcessingUnit<FIQ> {
    // composable
    async fn request_connectome_change(&mut self, request: NeuronProcessingUnitComposeRequest) -> ();
}