use feagi_basis::feagi_neuron::wrapped_types::{CorticalAreaNeuronLocalIndex};
use feagi_basis::prelude::*;
use crate::cortical_area::components::neuron_layout::NeuronLayout;
use crate::cortical_area::extendable_components::cortical_area_quantization::CorticalAreaQuantization;
use crate::cortical_area::extendable_components::cortical_model_data_field::CorticalModelDataField;
use crate::cortical_area::data_structs::{CorticalAreaModelConnectomeCorticalData, CorticalAreaModelNeuronData};


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
    type CorticalDataProperties: CorticalModelDataField<CAQ>;

    /// The cortical level data that is for mutable internal processing, will not be
    /// exposed to genome developers but is saved in the connectome
    type CorticalDataInternal: CorticalModelDataField<CAQ>;

    /// The cortical level data that is used for runtime processing, is not exposed to
    /// genome developers nor is it saved (starts clean with every init)
    type CorticalDataScratch: CorticalModelDataField<CAQ>;
    //endregion

    //region Per Neuron Data
    /// The per neuron level data that should be saved to the connectome
    type NeuronDataInternal: CorticalModelDataField<CAQ>;

    /// The per neuron level data that should not be saved to the connectome
    /// (starts clean with every init)
    type NeuronDataScratch: CorticalModelDataField<CAQ>;

    //endregion

    /// Not actual cortical data, but rather represents various "values" that are actually the result
    /// of running functions on the neuron data
    type CorticalFunctionContext: CorticalModelDataField<CAQ>;
    
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


    /// called per neuron, calls `process_model_neuron_dynamics` and does the psp postprocessing 
    /// upon it. DO NOT override this implementation!
    fn process_neuron_dynamics_for_psp (
        // Inputs as the incoming potential, but is output as the firing potential
        processing_buffer: &mut CorticalAreaNeuronPotential<CAQ::MembranePotentialQuant>,
        burst_index: &BurstIndex<FIQ::BurstIndexQuant>,
        cortical_data: &CorticalAreaModelConnectomeCorticalData<
            CAQ,
            Self::CorticalDataProperties,
            Self::CorticalDataInternal,
            Self::CorticalDataScratch
        >,
        neuron_data: &mut CorticalAreaModelNeuronData<
            CAQ,
            Self::NeuronDataInternal,
            Self::NeuronDataScratch
        >,
        neuron_linear_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        layout_context: &NL,
    ) {
        
        // Only some members of cortical data should be read, and as an immutable ref (we cant have
        // neurons in parallel change the shared cortical data state!)
        let cortical = cortical_data.get_parameters_for_neuron_dynamics();
        
        let model_is_firing = Self::process_model_neuron_dynamics(
            processing_buffer,
            burst_index,
            cortical.0,
            cortical.1,
            cortical.2,
            &mut neuron_data.neuron_data_internal,
            &mut neuron_data.neuron_data_scratch,
            neuron_linear_index,
            layout_context
        );
        
        // At this point, the output neuron potential (regardless of firing state) was written
        // to processing_buffer

        // We should apply any modifiers to the outgoing neuron potential, 
        // regardless of firing state
        if !cortical_data.cortical_area_flags.is_psp_uniform() {
            *processing_buffer = cortical_data.cortical_level_psp;
        }

        if cortical_data.get_is_psp_uniform() {
            *processing_buffer *= neuron_data.get_inverse_number_mappings_out().deref().into();
        }
        
        // This struct will apply any force fires / probe overrides and store firing state 
        _ = neuron_data.neuron_flags.process_neuron_firing(model_is_firing);
    }


}
