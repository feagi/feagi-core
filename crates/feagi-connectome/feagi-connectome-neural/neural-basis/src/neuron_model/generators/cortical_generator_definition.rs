use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::prelude::QuantizedUnsignedIntegerTrait;
use crate::neuron_model::generators::cortical_storage_loader::CorticalScratchGenerator;
use crate::neuron_model::neuron_model::{NeuronLayoutOf, NeuronModel};
use crate::neuron_model::data::neuron_layout::NeuronLayout;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Defines generating a new cortical area, to be taken in by any burst engine type. This struct
/// will own the context it needs to do the generation, so it can be passed around easily
pub trait CorticalGeneratorDefinition<FIQ, NM>: Send
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    /// Get the neuron layout that will be used
    fn get_neuron_layout(&self) -> &NeuronLayoutOf<NM, FIQ>;

    /// Get the cortical level psp to set
    fn get_cortical_level_psp(&self) -> CorticalAreaNeuronPotential<NM::MembranePotentialQuant>;

    /// Get if the psp is uniform
    fn get_is_psp_uniform(&self) -> bool;

    /// Get if the psp is using the membrane potential (and not the cortical level psp)
    fn get_is_psp_using_membrane_potential(&self) -> bool;

    /// Get the cortical level data needed for the cortical area
    fn get_cortical_level_data<LSG: CorticalScratchGenerator<FIQ, NM>>(&self) -> (
        NM::CorticalDataProperties,
        NM::CorticalDataInternal,
        NM::CorticalDataScratch,
    ); // Do not return function context as its not actually intended to be written, just used for neuron generation
    
    /// A function that returns what the neuron data should be at a given index. Also runs the function for
    /// the cortical level neuron modifier contexts
    fn generate_neuron_data_at<LSG: CorticalScratchGenerator<FIQ, NM>>(&self, index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>)
        -> (NM::NeuronDataInternal, NM::NeuronDataScratch);

    //TODO into enum


}
