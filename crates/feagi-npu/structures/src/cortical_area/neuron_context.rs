//! The NPU Neuron Context


use feagi_structures::feagi_data::shared_quantization_sets::{FeagiGlobalQuantization, NeuronModelQuantization};

#[repr(C)]
pub struct NPUNeuronContextCPU<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> {
    fire_candidate_cache_index: FGQ::SynapseIndexCountQuant,
    fire_candidate_length: FGQ::SynapseIndexCountQuant,
    synapse_one_to_one_index: FGQ::SynapseIndexCountQuant,
    synapse_one_to_one_length: FGQ::SynapseIndexCountQuant,
    fire_candidate_potential_in: NMQ::NeuronPotentialQuant,
    cortical_area_index: FGQ::CorticalAreaIndexCountQuant,

}


