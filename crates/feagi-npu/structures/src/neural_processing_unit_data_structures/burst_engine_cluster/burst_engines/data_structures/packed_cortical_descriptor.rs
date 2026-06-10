use crate::neural_processing_unit_data_structures::burst_engine_cluster::burst_engines::data_structures::cortical_structure_configuration::CorticalConfigurationType;
use crate::neuron_models::neuron_models::NeuronModelTypeAndQuantization;

/// Is used as a tag to identify what classes a cortical area belongs in, and more pertinently, 
/// where it stored within the burst engine. Uses a u8 to be cross device compatible, where the
/// first 2 bits describe the cortical structure type, the next 3 bits describe the neuron
/// model used, and the next 3 describe the quantization of that neuron model
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PackedCorticalDescriptor(u8);

impl PackedCorticalDescriptor
{
    pub fn new(cortical_structure_type: CorticalConfigurationType, neuron_model_and_quantization: NeuronModelTypeAndQuantization) -> Self {
        PackedCorticalDescriptor(
            (cortical_structure_type as u8) 
                & neuron_model_and_quantization.as_neuron_model_and_quantization_u8()
        )
    }
    
    /*
    pub fn get_neuron_membrane_potential_quantization(&self) -> Z {
        
    }
    
     */
}

impl Into<u8> for PackedCorticalDescriptor {
    fn into(self) -> u8
    {
        self.0
    }
}