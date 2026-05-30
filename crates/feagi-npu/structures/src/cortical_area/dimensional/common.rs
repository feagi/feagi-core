
use feagi_npu_neuron_models::shared_traits_and_structs::neuron_model_common::cortical_configuration::CorticalConfiguration;
use feagi_npu_neuron_models::shared_traits_and_structs::neuron_model_common::cortical_data_traits::CorticalModelData;
use feagi_npu_neuron_models::shared_traits_and_structs::neuron_model_common::neuron_data_traits::NeuronDataCommon;
use feagi_structures::feagi_data::create_quantized_index_count_wrapper;
use feagi_structures::feagi_data::feagi_ecs::collection::FeagiECSCollection;
use feagi_structures::feagi_data::feagi_ecs::element::FeagiECSElement;
use feagi_structures::feagi_data::feagi_ecs::element_set::FeagiECSElementSet;
use feagi_structures::feagi_data::shared_quantization_sets::{CorticalAreaModelQuantizationBase, FeagiGlobalIndexQuantization};

// TODO a synapse redirect index of MAX suggests nothing is connected, so dont continue

create_quantized_index_count_wrapper!(SynapseRedirectIndex);
create_quantized_index_count_wrapper!(SynapseRedirectSliceLength);

/// Represents a dimensional cortical area, and all their neurons and other properties
pub trait DimensionalCorticalAreaCommon<FGIQ, CAMQB, CC, CMC,  ND>:
FeagiECSElementSet<ND> // Stores more than neuron data but this is fine
where
    FGIQ: FeagiGlobalIndexQuantization,
    CAMQB: CorticalAreaModelQuantizationBase,
    CC: CorticalConfiguration<FGIQ>,
    CMC: CorticalModelData<FGIQ, CAMQB>,
    ND: NeuronDataCommon<FGIQ, CAMQB, CC, CMC>
{
    // Cant store data for unknown device

    // Stores neuron model data, as well as potential data, and cortical levvel flags as well
    // (as a header)


}






/// Represents a firing neuron that has nonplastic connections. This is a collected structure
/// element to de-sparsify the sparse firing dimensional cortical area. This probably isnt needed
/// for CPU implementations ( we can just do something separate to count the number of firing
/// neurons but otherwise just read forward
pub trait FiringNeuronRedirectCommon<FGIQ>:
FeagiECSElement
where
    FGIQ: FeagiGlobalIndexQuantization,
{
    // Cant store data for unknown device

    // The implementation will have the:
    // - source neuron index
    // - source cortical area index
    // - destination synapse redirect slice start index
    // - destination synapse redirect slice length
}

pub trait FiringNeuronRedirectsCommon<FGIQ, FNRE>:
FeagiECSCollection<FNRE>
where
    FGIQ: FeagiGlobalIndexQuantization,
    FNRE: FiringNeuronRedirectCommon<FGIQ>
{

}



