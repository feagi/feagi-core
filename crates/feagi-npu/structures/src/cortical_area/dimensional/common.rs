
use feagi_npu_neuron_models::shared_traits_and_structs::cortical_configuration::CorticalConfiguration;
use feagi_npu_neuron_models::shared_traits_and_structs::base_traits_all_devices::CorticalModelData;
use feagi_structures::feagi_data::create_quantized_index_count_wrapper;
use feagi_structures::feagi_data::shared_quantization_sets::{NeuronModelQuantization, FeagiGlobalQuantization};

// TODO a synapse redirect index of MAX suggests nothing is connected, so dont continue

create_quantized_index_count_wrapper!(SynapseRedirectIndex);
create_quantized_index_count_wrapper!(SynapseRedirectSliceLength);

/// Represents a dimensional cortical area, and all their neurons and other properties
pub trait DimensionalCorticalAreaCommon<FGIQ, CAMQB, CC, CMC,  ND>:
where
    FGIQ: FeagiGlobalQuantization,
    CAMQB: NeuronModelQuantization,
    CC: CorticalConfiguration<FGIQ>,
    CMC: CorticalModelData<FGIQ, CAMQB>,
    ND: NeuronDataCommon<FGIQ, CAMQB, CC, CMC>
{
    // Cant store data for unknown device

    // Stores the following information
    //
    // neuron model data, as well as potential data, and cortical levvel flags as well
    // (as a header)


}






/// Represents a firing neuron that has nonplastic connections. This is a collected structure
/// element to de-sparsify the sparse firing dimensional cortical area. This probably isnt needed
/// for CPU implementations ( we can just do something separate to count the number of firing
/// neurons but otherwise just read forward
pub trait FiringNeuronRedirectCommon<FGIQ>:
FeagiECSElement
where
    FGIQ: FeagiGlobalQuantization,
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
    FGIQ: FeagiGlobalQuantization,
    FNRE: FiringNeuronRedirectCommon<FGIQ>
{

}



