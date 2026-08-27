use feagi_data::neurons::wrapped_types::CorticalNeuronPotential;
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use crate::cortical_area::components::neuron_layout::neuron_layout_model::NeuronLayout;
use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::CorticalAreaQuantization;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

/// A body of a cortical area
pub trait CorticalBody<NPUIQ, BEIQ, NL, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    NL: NeuronLayout<BEIQ>,
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