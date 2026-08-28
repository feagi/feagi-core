use feagi_data::neurons::NeuronCorticalLocalIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_index_collections::CorticalEngineIndex;

/// For a given cortical index, denotes the local neuron indexes that should be forced to fire
pub struct CorticalAreaForceFireInjection<FIQ: FeagiIndexQuantization> {
    cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    force_fire_indexes: Vec<NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant>>
}