use feagi_basis::feagi_neuron::wrapped_types::CorticalAreaNeuronPotential;
use crate::cortical_area::components::cortical_area_dynamics::components::cortical_area_quantization::CorticalAreaQuantization;

/// Data for a cortical area that is saved to the connectome, is  viewable and editable by genome 
/// developers, but cannot be mutated by cortical processing. Required to be able to get
/// psp uniformity setting, mp driven psp, and the value for cortical driven psp. No Null variant.
pub trait CorticalDataProperties<CAQ: CorticalAreaQuantization> {    
    fn get_is_psp_uniform(&self) -> bool;
    
    fn get_is_psp_membrane_driven(&self) -> bool;
    
    fn get_cortical_driven_psp(&self) -> CorticalAreaNeuronPotential<CAQ::MembranePotentialQuant>;
}