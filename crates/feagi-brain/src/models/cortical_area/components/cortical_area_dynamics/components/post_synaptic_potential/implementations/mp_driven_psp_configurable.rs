use feagi_data::feagi_data_neuron::neurons::wrapped_types::CorticalNeuronPotential;
use feagi_data::feagi_data_neuron::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use crate::models::cortical_area::components::cortical_area_dynamics::components::post_synaptic_potential::MPDrivenPSPConfigurability;

pub struct MPDrivenPSPConfigurable<MP: MembranePotentialQuantization> {
    pub cortical_psp: CorticalNeuronPotential<MP::MembranePotentialQuant>,
    pub psp_is_mp_driven: bool // Not actually a data point in cortical area, its per all neurons uniformly
}

impl<MP: MembranePotentialQuantization> PostSynapticPotential for MPDrivenPSPConfigurable<MP> {
    fn get_if_psp_is_mp_driven(&self) -> bool {
        self.psp_is_mp_driven
    }
}
