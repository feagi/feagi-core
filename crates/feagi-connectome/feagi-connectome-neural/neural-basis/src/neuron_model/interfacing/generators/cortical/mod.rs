pub mod cortical_area_generator_cortical_from_stored;

use feagi_basis::neurons::CorticalAreaNeuronPotential;
use crate::neuron_model::interfacing::shared::NeuronModelUniversalCorticalProperties;
use crate::neuron_model::neuron_model::{NeuronLayoutOf, NeuronModel};
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub trait CorticalAreaGeneratorCortical<FIQ, NM>: Send
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    /// Get the neuron layout that will be used
    fn get_neuron_layout(&self) -> &NeuronLayoutOf<NM, FIQ>;

    /// Get properties expected of all cortical areas
    fn get_cortical_universal_properties(&self) -> &NeuronModelUniversalCorticalProperties<NM>;

    /// Get the cortical level data needed for the cortical area
    fn get_cortical_level_data(&self) -> (
        &NM::CorticalDataProperties,
        &NM::CorticalDataInternal,
        &NM::CorticalDataScratch,
    ); // Do not return variation context as its not actually intended to be written, just used for neuron generation
    
    
}
