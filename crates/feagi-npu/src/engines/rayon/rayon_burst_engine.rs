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
        Self {
            data: Default::default()
        }
    }
}




impl<FIQ: FeagiIndexQuantization> EditableEngine<FIQ> for RayonBurstEngine<FIQ> {
    
    fn get_editable_cortical_area_data<CPQ: MembranePotentialQuantization,  NMCD: NeuronModelCorticalData<CPQ>>(&mut self) -> &CorticalModelIndexedVector<FIQ::CorticalAreaIndexCountQuant, NMCD> {
        self.data.neuron_model_data.get_quant_cortical_data()
    }

    fn get_editable_neuron_data<CPQ: MembranePotentialQuantization,  NMND: NeuronModelNeuronData<CPQ>>(&mut self) -> &NeuronModelIndexedVector<FIQ::NeuronIndexCountQuant, NMND> {
        self.data.neuron_model_data.get_quant_neuron_data()
    }
    
    

    fn add_cortical_area<CPQ: MembranePotentialQuantization, NMCD: NeuronModelCorticalData<CPQ>, NMND: NeuronModelNeuronData<CPQ>>(&mut self, cortical_area_layout: CorticalAreaLayoutNested<FIQ>, neuron_model_type_and_quantization: NestedNeuronModelTypeAndQuantization, neuron_writer: impl NeuronModelWriter<CPQ, NMCD, NMND>) {
        
        let cortical_areas = self.get_editable_cortical_area_data();
        cortical_areas.extend(1.into())
        let number_neurons = cortical_area_layout.get_total_number_neurons();
        let neurons = self.get_editable_neuron_data();
    }

    fn remove_cortical_area(&mut self) {
        todo!()
    }

    fn resize_cortical_area(&mut self) {
        todo!()
    }

    fn add_connections(&mut self) {
        let connections = self.get_editable_neuron_data();
        let adding = connections.into();
        
    }

    fn remove_connections(&mut self) {
        todo!()
    }
}


