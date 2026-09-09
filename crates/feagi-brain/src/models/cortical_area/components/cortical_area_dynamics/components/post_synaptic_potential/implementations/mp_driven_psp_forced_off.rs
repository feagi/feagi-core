use feagi_data::feagi_data_neuron::neurons::wrapped_types::CorticalNeuronPotential;
use feagi_data::feagi_data_neuron::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use crate::models::cortical_area::components::cortical_area_dynamics::components::post_synaptic_potential::MPDrivenPSPConfigurability;

pub struct MPDrivenPSPForcedOff<MP: MembranePotentialQuantization> {
    pub cortical_psp: CorticalNeuronPotential<MP::MembranePotentialQuant>
}

impl<MP: MembranePotentialQuantization> PostSynapticPotential for MPDrivenPSPForcedOff<MP> {
    fn get_if_psp_is_mp_driven(&self) -> bool {
        false
    }
}
