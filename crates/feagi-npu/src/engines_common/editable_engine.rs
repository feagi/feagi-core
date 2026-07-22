use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::neuron::common_structs::cortical_area_layout::CorticalAreaLayoutNested;
use feagi_models::neuron::genome_interface::cortical_area_spawner::NeuronModelWriter;
use feagi_models::neuron::model_and_quantization::NestedNeuronModelTypeAndQuantization;
use feagi_models::neuron::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};
use feagi_models::wrapped_indexes::{CorticalModelIndexedVector, NeuronModelIndexedVector};
// 1: append consequences of commands (deleting / remaking connections)
//  2: sort into the following order -> delete synapses -> delete areas -> resize area? -> make areas -> (re)make connections

/// Engine with an editable connectome
pub trait EditableEngine<FIQ: FeagiIndexQuantization> {
    //region Cortical Area

    fn get_editable_cortical_area_data<CPQ: MembranePotentialQuantization,  NMCD: NeuronModelCorticalData<CPQ>>(&mut self) -> &CorticalModelIndexedVector<FIQ::CorticalAreaIndexCountQuant, NMCD>;
    
    fn get_editable_neuron_data<CPQ: MembranePotentialQuantization,  NMND: NeuronModelNeuronData<CPQ>>(&mut self) -> &NeuronModelIndexedVector<FIQ::NeuronIndexCountQuant, NMND>;
    
    fn add_cortical_area<CPQ: MembranePotentialQuantization, NMCD: NeuronModelCorticalData<CPQ>, NMND: NeuronModelNeuronData<CPQ>>(
        &mut self,
        cortical_area_layout: CorticalAreaLayoutNested<FIQ>,
        neuron_model_type_and_quantization: NestedNeuronModelTypeAndQuantization,
        neuron_writer: impl NeuronModelWriter<CPQ, NMCD, NMND>,
    ); // TODO

    fn remove_cortical_area(&mut self); // TODO

    fn resize_cortical_area(&mut self); // TODO

    //endregion

    fn add_connections(&mut self); // TODO

    fn remove_connections(&mut self); // TODO

    //region Dynamic Metrics

    // TODO

    //endregion
}
