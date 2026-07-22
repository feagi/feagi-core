use feagi_data::index_range_managers::index_manager::IndexManager;
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::neuron::common_structs::cortical_area_layout::CorticalAreaLayoutNested;
use feagi_models::neuron::model_and_quantization::NestedNeuronModelTypeAndQuantization;
use feagi_models::neuron::neuron_model_data::NeuronModelNeuronData;
use feagi_npu_common::wrapped_indexes::CorticalEngineIndex;


/// A given index for a burst engine directly managed by this NPU. Is an u8 since if you have
/// more than 256 GPUs attached to one motherboard, you are doing something wrong
pub type BurstEngineIndex = u8;

pub struct BurstEngineContext<FIQ: FeagiIndexQuantization> {
    cortical_index_manager: IndexManager<FIQ::CorticalAreaIndexCountQuant>
}


impl<FIQ: FeagiIndexQuantization> BurstEngineContext<FIQ> {

    fn try_add_cortical_area(
        &mut self,
        neuron_model_and_type: NestedNeuronModelTypeAndQuantization,
        cortical_area_layout: CorticalAreaLayoutNested<FIQ>

    )



    fn add_cortical_area<CPQ: MembranePotentialQuantization, Model: NeuronModelNeuronData<CPQ>>(
        &mut self,
        neuron_writer: &impl CorticalWriter<FIQ, CPQ, Model, Layout>,
    ) -> CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant> {



        todo!()
    }
}