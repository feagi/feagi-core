use feagi_basis::neurons::CorticalAreaNeuronPotential;
use crate::neuron_model::neuron_model::NeuronModel;


pub struct NeuronModelUniversalCorticalProperties<NM>
where NM: NeuronModel
{
    /// The psp to use on neuron fire if psp_using_neuron_potential is disabled
    pub cortical_level_psp: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
    pub is_psp_uniform: bool,
    pub is_psp_using_neuron_potential: bool
}

