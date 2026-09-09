use feagi_data::feagi_data_neuron::neurons::wrapped_types::{CorticalNeuronLocalIndex, CorticalNeuronPotential};
use feagi_data::feagi_data_neuron::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::feagi_data_neuron::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::models::cortical_area::components::cortical_area_dynamics::components::cortical_area_quantization::CorticalAreaQuantization;
use crate::models::cortical_area::components::cortical_area_dynamics::components::cortical_model_data::CorticalModelQuantizedData;
use crate::models::cortical_area::components::cortical_area_dynamics::components::data::{CorticalDataInternal, CorticalDataProperties, CorticalDataShared, NeuronDataInternal, NeuronDataProperties};
use crate::models::cortical_area::components::cortical_area_dynamics::components::post_synaptic_potential::PostSynapticPotential;
use crate::models::cortical_area::components::neuron_layout::neuron_layout_model::NeuronLayout;
// TODO maybe we should allow the BEIQ type in here so people can better use the layout information (tie it to a unique data struct that is not saved, that must instead be generated at instantiation / layout edit)

pub trait CorticalAreaDynamics<FIQ, NL>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
{
    /// Defines the quantization level of the data of the cortical area dynamics and the membrane potential
    type CorticalAreaModelQuantization: CorticalAreaQuantization;
    
    /// Defines how the Post Synaptic Potential as it is seen by downstream mappings
    type MPDrivenPSPConfigurability: PostSynapticPotential;

    // NOTE: The data properties for the cortical and neurons are each in one uniform struct.
    // Beware overall byte alignment for them all!

    //region Cortical level Data
    /// The cortical level data that should be exposed to genome developers
    type CorticalDataProperties: CorticalModelQuantizedData<Self::CorticalAreaModelQuantization>;

    /// The cortical level data that is for internal processing, will not be
    /// exposed to genome developers but is saved in the connectome
    type CorticalDataInternal: CorticalModelQuantizedData<Self::CorticalAreaModelQuantization>;

    /// The cortical level data that is used for runtime processing, is not exposed to
    /// genome developers nor is it saved (starts clean with every init)
    type CorticalDataScratch: CorticalModelQuantizedData<Self::CorticalAreaModelQuantization>;
    //endregion

    //region Per Neuron Data
    /// The per neuron level data that should be saved to the connectome
    type NeuronDataInternal: CorticalModelQuantizedData<Self::CorticalAreaModelQuantization>;

    /// The per neuron level data that should not be saved to the connectome
    /// (starts clean with every init)
    type NeuronDataScratch: CorticalModelQuantizedData<Self::CorticalAreaModelQuantization>;

    //endregion

    /// Set to true to ensure the burst engine calls the `process_cortical_dynamics` function
    const HAS_CORTICAL_DYNAMICS_PROCESSING: bool;
    
    /// Set to true to ensure the burst engine calls the `process_neuron_dynamics` function
    const HAS_NEURON_DYNAMICS_PROCESSING: bool;
    
    /* // TODO move to CPU 
    /// called per area. Called before neuron call
    fn process_cortical_dynamics(
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>, 
        cortical_properties: &mut Self::CorticalDataProperties,
        cortical_internal: &mut Self::CorticalDataInternal,
        cortical_shared: &mut Self::CorticalDataShared,
        layout_context: &NL,
    ) -> (); // TODO return type?

    /// called per neuron, outputs the firing potential to be further post processed (or ignored)
    fn process_neuron_dynamics(
        incoming_potential: &CorticalNeuronPotential<CAMQ::MembranePotentialQuant>,
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        cortical_properties: &Self::CorticalDataProperties,
        cortical_internal: &Self::CorticalDataInternal,
        cortical_shared: &Self::CorticalDataShared,
        neuron_properties: &mut Self::NeuronDataProperties,
        neuron_internal: &mut Self::NeuronDataInternal,
        neuron_linear_index: &CorticalNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        layout_context: &NL,
    ) -> NeuronDynamicsOutput<CAMQ>; // This is used immediately 


     */
}

/// Output of Neuron Dynamics, may be processed further with things like PSP uniformity
pub struct NeuronDynamicsOutput<CAMQ: MembranePotentialQuantization> {
    Firing(CorticalNeuronPotential<CAMQ::MembranePotentialQuant>),
    NotFiring
}
