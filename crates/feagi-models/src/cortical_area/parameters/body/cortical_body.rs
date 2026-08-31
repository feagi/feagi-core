use feagi_data::neurons::wrapped_types::CorticalNeuronPotential;
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use crate::cortical_area::components::neuron_layout::neuron_layout_model::NeuronLayout;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// A body of a cortical area
pub trait CorticalBody<FIQ, NL, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CAMQ: MembranePotentialQuantization,
{
    /// Defines if we can configure the usage of MP as the PSP or not
    type MPDrivenPSPConfigurability;

    /// Defines if we can configure the cortical level of PSP
    type CorticalLevelPSPConfigurability;

    // TODO should this really be here?

    fn get_cortical_level_psp(&self) -> CorticalNeuronPotential<CAMQ::MembranePotentialQuant>;

    fn set_cortical_level_psp(&mut self, potential: CorticalNeuronPotential<CAMQ::MembranePotentialQuant>) -> ();
}