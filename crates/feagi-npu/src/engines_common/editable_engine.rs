use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::neuron::cortical_area_layout::CorticalAreaLayoutNested;
use feagi_models::neuron::interfacing::data_io::NeuronModelWriter;
use feagi_models::neuron::interfacing::model_and_quantization::NeuronModelTypeAndQuantization;
use feagi_models::neuron::models_shared::data::{NeuronModelCorticalData, NeuronModelNeuronData};
use feagi_models::wrapped_indexes::{CorticalModelIndexedVector, NeuronModelIndexedVector};
// 1: append consequences of commands (deleting / remaking connections)
//  2: sort into the following order -> delete synapses -> delete areas -> resize area? -> make areas -> (re)make connections

/// Engine with an editable connectome
pub trait EditableEngine<FIQ: FeagiIndexQuantization> {
    //region Cortical Area

    fn get_editable_cortical_area_data<CPQ: CorticalPotentialQuantization,  NMCD: NeuronModelCorticalData<CPQ>>(&mut self) -> &CorticalModelIndexedVector<FIQ::CorticalAreaIndexCountQuant, NMCD>;
    
    fn get_editable_neuron_data<CPQ: CorticalPotentialQuantization,  NMND: NeuronModelNeuronData<CPQ>>(&mut self) -> &NeuronModelIndexedVector<FIQ::NeuronIndexCountQuant, NMND>;
    
    fn add_cortical_area<CPQ: CorticalPotentialQuantization, NMCD: NeuronModelCorticalData<CPQ>, NMND: NeuronModelNeuronData<CPQ>>(
        &mut self,
        cortical_area_layout: CorticalAreaLayoutNested<FIQ>,
        neuron_model_type_and_quantization: NeuronModelTypeAndQuantization,
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
