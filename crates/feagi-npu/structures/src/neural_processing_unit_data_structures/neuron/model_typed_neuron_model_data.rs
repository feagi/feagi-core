use crate::neuron_models::shared_traits_and_structs::base_traits_all_devices::{CorticalModelData, NeuronModelData, NeuronModelProcessor};
use crate::neuron_models::shared_traits_and_structs::cortical_configuration::{CorticalConfiguration, CorticalConfigurationDimensionalCPU};
use feagi_structures::feagi_data::feagi_pdi::PDICollection;
use feagi_structures::feagi_data::feagi_pdi::tag_device::PDITagGenericDevice;
use feagi_structures::feagi_data::shared_quantization_sets::{FeagiGlobalQuantization, NeuronModelQuantization};
use crate::neural_processing_unit_data_structures::wrapped_indexing::NPUNeuronIndexModelQuantizationLocal;



pub trait ModelQuantTypedNeuronModelDataCollection<FGQ, NMQ, CCC, CMD,  NMD, NMP>:
PDICollection
+ PDITagGenericDevice
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCC: CorticalConfiguration<FGQ>,
    CMD: CorticalModelData<FGQ, NMQ>,
    NMD: NeuronModelData<FGQ, NMQ>,
    NMP: NeuronModelProcessor<FGQ, NMQ, CCC, CMD, NMD>
{
    fn get_number_neurons_of_this_neuron_model_and_quantization(&self) -> NPUNeuronIndexModelQuantizationLocal<FGQ::NeuronIndexCountQuant>;
}


//region CPU implementation


pub struct ModelQuantTypedNeuronModelDataCollectionCPU<FGQ, NMQ, CCC, CMD,  NMD, NMP>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    CCC: CorticalConfiguration<FGQ>,
    CMD: CorticalModelData<FGQ, NMQ>,
    NMD: NeuronModelData<FGQ, NMQ>,
    NMP: NeuronModelProcessor<FGQ, NMQ, CCC, CMD, NMD>
{
    pub neuron_data: Vec<NMD>,
    pub neuron_processor: NMP
}

impl<FGQ, NMQ, NMD> ModelQuantTypedNeuronModelDataCollectionCPU<FGQ, NMQ, NMD>
where
    FGQ: FeagiGlobalQuantization,
    NMQ: NeuronModelQuantization,
    NMD: NeuronModelData<FGQ, NMQ>
{

}



//endregion