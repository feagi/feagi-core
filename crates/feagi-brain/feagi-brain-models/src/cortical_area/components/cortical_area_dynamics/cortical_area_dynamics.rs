use feagi_basis::feagi_neuron::wrapped_types::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::prelude::*;
use crate::cortical_area::common_structs::per_neuron_flags::PerNeuronFlags;
use crate::cortical_area::components::cortical_area_dynamics::components::cortical_area_quantization::CorticalAreaQuantization;
use crate::cortical_area::components::cortical_area_dynamics::components::model_data::{CorticalDataInternal, CorticalDataProperties, CorticalDataScratch, NeuronDataInternal, NeuronDataScratch};
use crate::cortical_area::components::neuron_layout::NeuronLayout;

type IsFiring = bool;

pub trait CorticalAreaDynamics<FIQ, NL, CAQ>
where
    FIQ: FeagiIndexQuantization,
    NL: NeuronLayout<FIQ>,
    CAQ: CorticalAreaQuantization
{
    // NOTE: The data properties for the cortical and neurons are each in one uniform struct.
    // Beware overall byte alignment for them all!

    //region Cortical level Data
    /// The cortical level data that should be exposed to genome developers. Not mutable during
    /// cortical processing
    type CorticalDataProperties: CorticalDataProperties<CAQ>;

    /// The cortical level data that is for mutable internal processing, will not be
    /// exposed to genome developers but is saved in the connectome
    type CorticalDataInternal: CorticalDataInternal<CAQ>;

    /// The cortical level data that is used for runtime processing, is not exposed to
    /// genome developers nor is it saved (starts clean with every init)
    type CorticalDataScratch: CorticalDataScratch<CAQ>;
    //endregion

    //region Per Neuron Data
    /// The per neuron level data that should be saved to the connectome
    type NeuronDataInternal: NeuronDataInternal<CAQ>;

    /// The per neuron level data that should not be saved to the connectome
    /// (starts clean with every init)
    type NeuronDataScratch: NeuronDataScratch<CAQ>;

    //endregion

    /// Set to true to ensure the burst engine calls the `process_cortical_dynamics` function
    const HAS_CORTICAL_DYNAMICS_PROCESSING: bool;
    
    /// Set to true to ensure the burst engine calls the `process_neuron_dynamics` function
    const HAS_NEURON_DYNAMICS_PROCESSING: bool;

    /// called per area. Called before neuron call
    fn process_model_cortical_dynamics(
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>, 
        cortical_properties: &Self::CorticalDataProperties,
        cortical_internal: &mut Self::CorticalDataInternal,
        cortical_shared: &mut Self::CorticalDataScratch,
        layout_context: &NL,
    ) -> (); // TODO return type?

    /// called per neuron, outputs the firing potential (to processing buffer) to be further post processed (or ignored) and if we are firing
    fn process_model_neuron_dynamics(
        // Inputs as the incoming potential, but is output as the firing potential
        processing_buffer: &mut CorticalAreaNeuronPotential<CAQ::MembranePotentialQuant>,
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        cortical_properties: &Self::CorticalDataProperties,
        cortical_internal: &Self::CorticalDataInternal,
        cortical_scratch: &Self::CorticalDataScratch,
        neuron_internal: &mut Self::NeuronDataInternal,
        neuron_scratch: &mut Self::NeuronDataScratch,
        neuron_linear_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        layout_context: &NL,
    ) -> IsFiring;

    /// called per neuron, calls `process_model_neuron_dynamics` and does the psp postprocessing upon it
    fn process_neuron_dynamics_for_psp (
        // Inputs as the incoming potential, but is output as the firing potential
        processing_buffer: &mut CorticalAreaNeuronPotential<CAQ::MembranePotentialQuant>,
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        cortical_properties: &Self::CorticalDataProperties,
        cortical_internal: &Self::CorticalDataInternal,
        cortical_scratch: &Self::CorticalDataScratch,
        neuron_internal: &mut Self::NeuronDataInternal,
        neuron_scratch: &mut Self::NeuronDataScratch,
        neuron_linear_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        layout_context: &NL,
        per_neuron_flags: &mut PerNeuronFlags,
    ) {
        
        let model_is_firing = Self::process_model_neuron_dynamics(
            processing_buffer,
            burst_index,
            cortical_properties,
            cortical_internal,
            cortical_scratch,
            neuron_internal,
            neuron_scratch,
            neuron_linear_index,
            layout_context
        );

        // This struct will apply any force fires / pressure / inhibition and store firing state
        let is_firing = per_neuron_flags.process_neuron_firing(model_is_firing);
        
        // If we are firing, we should update the outputting potential
        if is_firing {
            if !cortical_properties.get_is_psp_membrane_driven() {
                *processing_buffer = cortical_properties.get_cortical_driven_psp();
            }
            
            if cortical_properties.get_is_psp_uniform() {
                *processing_buffer *= neuron_scratch.get_inverse_number_mappings_out().deref().into();
            }
        }
    }


}
