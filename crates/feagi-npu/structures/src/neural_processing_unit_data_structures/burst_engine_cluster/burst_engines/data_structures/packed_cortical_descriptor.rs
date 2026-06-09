
/// Is used as a tag to identify what classes a cortical area belongs in, and more pertinently, 
/// where it stored within the burst engine. Uses a u8 to be cross device compatible, where the
/// first 2 bits describe the cortical structure type, the next 3 bits describe the neuron
/// model used, and the next 3 describe the quantization of that neuron model
#[repr(C)]
pub struct PackedCorticalDescriptor {
    inner: u8
}

impl PackedCorticalDescriptor
{
    
    pub fn new(cortical_structure_type: X, neuron_model_and_quantization: Y) -> Self {
        
    }
    
    pub fn get_cortical_structure_type(&self) -> X {
        
    }

    pub fn get_neuron_model_and_quantization(&self) -> Y {

    }

    pub fn get_neuron_membrane_potential_quantization(&self) -> Z {

    }
    
}