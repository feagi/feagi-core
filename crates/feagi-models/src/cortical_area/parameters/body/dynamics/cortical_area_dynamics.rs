use feagi_data::neurons::wrapped_types::{CorticalNeuronLocalIndex, CorticalNeuronPotential};
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use crate::cortical_area::parameters::body::dynamics::components::data::{CorticalDataProperties, CorticalDataInternal, CorticalDataShared, NeuronDataProperties, NeuronDataInternal};
use crate::cortical_area::components::neuron_layout::neuron_layout_model::{NeuronLayout};
use crate::cortical_area::parameters::body::dynamics::components::mp_driven_psp_configurability::MPDrivenPSPConfigurability;
use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::CorticalAreaQuantization;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use crate::wrapped_indexes::BurstIndex;

// TODO maybe we should allow the BEIQ type in here so people can better use the layout information (tie it to a unique data struct that is not saved, that must instead be generated at instantiation / layout edit)

pub trait CorticalAreaDynamics<NPUIQ, BEIQ, NL, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    NL: NeuronLayout<BEIQ>,
    CAMQ: CorticalAreaQuantization,
{
    /// Defines if we can configure the usage of MP as the PSP or not
    type MPDrivenPSPConfigurability: MPDrivenPSPConfigurability;

    // /// Defines if we can configure the cortical level of PSP
    // type CorticalLevelPSPConfigurability; // TODO how do we implement this?

    // NOTE: The data properties for the cortical and neurons are each tupled. Beware overall
    // byte alignment for them all
    
    /// The cortical level data that should be exposed to genome developers
    type CorticalDataProperties: CorticalDataProperties<CAMQ>;

    /// The cortical level data that is for internal processing, will not be
    /// exposed to genome developers
    type CorticalDataInternal: CorticalDataInternal<CAMQ>;

    /// The cortical level data that is shared and accessible by mappings (not other areas)
    type CorticalDataShared: CorticalDataShared<CAMQ>;

    /// The per neuron level data that should be exposed to genome developers
    type NeuronDataProperties: NeuronDataProperties<CAMQ>;

    /// The per neuron level data that is for internal processing, will not be
    /// exposed to genome developers
    type NeuronDataInternal: NeuronDataInternal<CAMQ>;

    /// Set to true to ensure the burst engine calls the `process_cortical_dynamics` function
    const HAS_CORTICAL_DYNAMICS_PROCESSING: bool;
    
    /// Set to true to ensure the burst engine calls the `process_neuron_dynamics` function
    const HAS_NEURON_DYNAMICS_PROCESSING: bool;
    
    /// called per area. Called before neuron call
    fn process_cortical_dynamics(
        burst_index: &BurstIndex<NPUIQ::BurstIndexQuant>, 
        cortical_properties: &mut Self::CorticalDataProperties,
        cortical_internal: &mut Self::CorticalDataInternal,
        cortical_shared: &mut Self::CorticalDataShared,
        layout_context: &NL,
    ) -> (); // TODO return type?

    /// called per neuron, outputs the firing potential to be further post processed (or ignored)
    fn process_neuron_dynamics(
        incoming_potential: &CorticalNeuronPotential<CAMQ::MembranePotentialQuant>,
        burst_index: &BurstIndex<NPUIQ::BurstIndexQuant>,
        cortical_properties: &Self::CorticalDataProperties,
        cortical_internal: &Self::CorticalDataInternal,
        cortical_shared: &Self::CorticalDataShared,
        neuron_properties: &mut Self::NeuronDataProperties,
        neuron_internal: &mut Self::NeuronDataInternal,
        neuron_linear_index: &CorticalNeuronLocalIndex<BEIQ::NeuronIndexQuant>,
        layout_context: &NL,
    ) -> NeuronDynamicsOutput<CAMQ>; // This is used immediately 

}

/// Output of Neuron Dynamics, may be processed further with things like PSP uniformity
#[derive(Debug, Clone, )]
pub enum NeuronDynamicsOutput<CAMQ: MembranePotentialQuantization> {
    Firing(CorticalNeuronPotential<CAMQ::MembranePotentialQuant>),
    NotFiring
}
