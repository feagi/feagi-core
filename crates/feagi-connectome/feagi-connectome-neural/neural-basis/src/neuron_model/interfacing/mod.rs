use crate::neuron_model::NeuronModel;
use feagi_basis::neurons::CorticalAreaNeuronPotential;

pub mod cortical_area_spec;
pub mod editors;




#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeuronModelUniversalCorticalProperties<NM>
where
    NM: NeuronModel
{
    /// The psp to use on neuron fire if psp_using_neuron_potential is disabled
    pub cortical_level_psp: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
    pub is_psp_uniform: bool,
    pub is_psp_using_neuron_potential: bool
}
