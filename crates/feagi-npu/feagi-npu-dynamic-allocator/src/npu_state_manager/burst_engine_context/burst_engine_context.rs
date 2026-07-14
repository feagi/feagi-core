use feagi_data::collections::index_range_managers::index_manager::IndexManager;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialCPUQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

use feagi_models::cortical_area_layout::{CorticalAreaLayoutData, CorticalAreaLayoutDataDimensional};
use feagi_models::neuron_models::neuron_model_traits::neuron_model_data::NeuronModelNeuronData;
use feagi_npu_common::wrapped_indexes::CorticalEngineIndex;
use crate::iterators::neuron_data_writer::CorticalWriter;

/// A given index for a burst engine directly managed by this NPU. Is an u8 since if you have
/// more than 256 GPUs attached to one motherboard, you are doing something wrong
pub type BurstEngineIndex = u8;

pub struct BurstEngineContext<FIQ: FeagiIndexQuantization> {
    cortical_index_manager: IndexManager<FIQ::CorticalAreaIndexCountQuant>
}


impl<FIQ: FeagiIndexQuantization> BurstEngineContext<FIQ> {

    fn add_cortical_area<CPQ: CorticalPotentialCPUQuantization, Model: NeuronModelNeuronData<CPQ>, Layout: CorticalAreaLayoutData<FIQ>>(
        &mut self,
        neuron_writer: &impl CorticalWriter<FIQ, CPQ, Model, Layout>,
    ) -> CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant> {



        todo!()
    }
}