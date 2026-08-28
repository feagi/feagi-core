use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;


pub enum ComposableBurstEngineWorkerResponse<NPUIQ: NeuronProcessingUnitIndexQuantization> {
    KernelRan {
        burst_index: BurstIndex<NPUIQ::BurstIndexQuant>,
        // TODO carry Result<(), FeagiBurstEngineError> or metrics
    },
    //EngineConnectomeEdited(Vec<EngineConnectomeEditResponse<FIQ>>),
    Stopped,
}