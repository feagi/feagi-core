use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::neuron::common_structs::cortical_area_layout::CorticalAreaLayoutNested;
use feagi_models::neuron::model_and_quantization::NestedNeuronModelTypeAndQuantization;
use feagi_models::neuron::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};
use feagi_models::wrapped_index_collections::{CorticalModelIndexedVector, NeuronModelIndexedVector};
use crate::engines::rayon::data::RayonEngineData;
use crate::editable::editable_engine::EditableEngine;

pub struct RayonBurstEngine<FIQ: FeagiIndexQuantization> {
    data: RayonEngineData<FIQ>
}

impl<FIQ: FeagiIndexQuantization> RayonBurstEngine<FIQ> {
    pub fn new() -> Self {
        todo!()
    }
}
