pub mod pattern_all;
pub mod pattern_certain_indexes;

use std::process::Output;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use crate::neuron_model::neuron_layout::NeuronLayoutTag;
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;

/// Defines which neurons get accessed. Contains the neuron layout and it must be equal to the one
/// in the burst engine of the given cortical index
pub trait NeuronPattern<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    fn get_contained_neuron_layout(&self) -> &<NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>;

    /// Get what neurons to edit. NOTE: We assume that this iterator will not exceed the max neuron count!
    fn iterate_neurons_indexes_to_edit(&self) -> impl Iterator<Item=CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>>;

    /// For engines that support it, allow them to validate that the carried neuron layout of this pattern is valid
    fn validate_neuron_layout(&self, engine_cortical_layout: &<NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>) -> Result<(), NeuronPatternError> {
        if engine_cortical_layout != self.get_contained_neuron_layout() {
            return Err(NeuronPatternError::NonMatchingNeuronLayout)
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NeuronPatternError {
    #[error("Neuron Layout of given Neuron Pattern does not match the one define for the engine!")]
    NonMatchingNeuronLayout,
}