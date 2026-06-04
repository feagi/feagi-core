
use feagi_npu_neuron_models::shared_traits_and_structs::cortical_configuration::CorticalConfiguration;
use feagi_npu_neuron_models::shared_traits_and_structs::base_traits_all_devices::CorticalModelData;
use feagi_structures::feagi_data::create_quantized_index_count_wrapper;
use feagi_structures::feagi_data::shared_quantization_sets::{NeuronModelQuantization, FeagiGlobalQuantization};

// TODO a synapse redirect index of MAX suggests nothing is connected, so dont continue

create_quantized_index_count_wrapper!(SynapseRedirectIndex);
create_quantized_index_count_wrapper!(SynapseRedirectSliceLength);

/// Represents a dimensional cortical area, and all their neurons and other properties
pub trait DimensionalCorticalAreaCommon<FGQ, NMQ, CC, CMC>:
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CC: CorticalConfiguration<FGQ>,
    CMC: CorticalModelData<FGQ, NMQ>,
{
    // Cant store data for unknown device

    // Stores the following information
    //
    // neuron model data, as well as potential data, and cortical levvel flags as well
    // (as a header)


}



