//! All Cortical Areas have a header, which describe cortical level properties but also cortical
//! level index conversions

use feagi_npu_neuron_models::NeuronChunkSize;
use feagi_structures::feagi_data::feagi_pdi::{PDICollection, PDIElement};
use feagi_structures::feagi_data::feagi_pdi::tag_device::PDITagGenericDevice;
use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_structures::feagi_data::shared_quantization_sets::{FeagiGlobalQuantization, NeuronModelQuantization};
use crate::neural_processing_unit_data_structures::wrapped_indexing::{NPUCorticalAreaIndexGlobal, NPUNeuronChunkIndexCorticalAreaLocal, NPUNeuronChunkIndexGlobal, NPUNeuronChunkIndexModelQuantizationLocal, NPUNeuronChunkIndexQuantizationLocal, NPUNeuronIndexCorticalLocal, NPUNeuronIndexModelQuantizationLocal, NPUNeuronIndexQuantizationLocal, NPUNeuronMembranePotential, NPUneuronFCLInputPotential};


pub trait NPUGlobalCorticalAreaHeaderTable<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization>:
PDICollection
+ PDITagGenericDevice
{
    fn get_number_cortical_areas(&self) -> NPUCorticalAreaIndexGlobal<FGQ::CorticalAreaIndexCountQuant>;
}


pub trait NPUGlobalCorticalHeader<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization>:
PDIElement
+ PDITagGenericDevice
{
    fn get_number_neurons_in_cortical_area(&self) -> NPUNeuronIndexCorticalLocal<FGQ::NeuronIndexCountQuant>;
}


//region CPU Implementations
#[repr(C)]
pub(crate) struct NPUGlobalCorticalAreaHeaderCPU<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization>
{
    number_neurons_in_area: NPUNeuronIndexCorticalLocal<FGQ::NeuronIndexCountQuant>, // u32
    /// The global neuron chunk index of the first neuron chunk of this cortical area
    neuron_chunk_global_index_of_first_cortical_neuron_chunk: NPUNeuronChunkIndexGlobal<FGQ::NeuronIndexCountQuant>, //u32
    // TODO other offsets


    neuron_chunk_neuron_count: NeuronChunkSize, //u8
    number_firing_neurons_this_burst: NPUNeuronIndexCorticalLocal<FGQ::NeuronIndexCountQuant>,
}

impl<FGQ: FeagiGlobalQuantization, NMQ: NeuronModelQuantization> NPUGlobalCorticalAreaHeaderCPU<FGQ, NMQ>
{

    pub fn get_cortical_local_neuron_chunk_index(&self, global_neuron_chunk_index: NPUNeuronChunkIndexGlobal<FGQ>) -> NPUNeuronChunkIndexCorticalAreaLocal<FGQ> {
        NPUNeuronChunkIndexCorticalAreaLocal::wrap((global_neuron_chunk_index - self.neuron_chunk_global_index_of_first_cortical_neuron_chunk).unwrap())
    }

    pub fn get_cortical_local_neuron_first_index_from_chunk_global_index(&self, global_neuron_chunk_index: NPUNeuronChunkIndexGlobal<FGQ>) -> NPUNeuronIndexCorticalLocal<FGQ> {
        todo!()
    }

    pub fn get_model_and_quant_local_neuron_first_index_from_chunk_global_index(&self, global_neuron_chunk_index: NPUNeuronChunkIndexGlobal<FGQ>) -> NPUNeuronIndexModelQuantizationLocal<FGQ> {
        todo!()
    }

    pub fn get_quant_local_neuron_first_index_from_chunk_global_index(&self, global_neuron_chunk_index: NPUNeuronChunkIndexGlobal<FGQ>) -> NPUNeuronIndexQuantizationLocal<FGQ> {
        todo!()
    }



    pub fn get_fcl_cortical_area_input(&self) -> &[NPUneuronFCLInputPotential<NMQ::NeuronPotentialQuant>] {
        todo!()
    }

    pub fn get_fcl_cortical_area_input_mut(&mut self) -> &mut [NPUneuronFCLInputPotential<NMQ::NeuronPotentialQuant>] {
        todo!()
    }

    pub fn get_cortical_area_membrane_potential(&self) -> &[NPUNeuronMembranePotential<NMQ::NeuronPotentialQuant>] {
        todo!()
    }

    pub fn get_cortical_area_membrane_potential_mut(&mut self) -> &mut [NPUNeuronMembranePotential<NMQ::NeuronPotentialQuant>] {
        todo!()
    }




}



//endregion