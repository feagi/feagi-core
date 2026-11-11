use crate::neuron_model::neurogenesis::from_storage::NeurogenesisFromStorage;
use crate::neuron_model::neurogenesis::NeurogenesisMethod;
use crate::neuron_model::neuron_layout::{NeuronLayoutOf, NeuronLayoutTag};
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;

/// Specifies that the Neurogenesis process occurs by algorithm, not mearly reading preproduced
/// data
pub trait NeurogenesisByGenerator<FIQ, NM>: NeurogenesisMethod<FIQ, NM> + Sized
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    /// Any fields needed to init this generator. Note that there is nothing stopping the generator
    /// from having additional functions or parameters to tune later, but this is what is needed
    /// to init it to a functional state at all. Note this does NOT include the layout!
    type GeneratorInitParameters;
    
    /// Any other fields needed to create this generator from a storage generator
    type OtherDataNeededForFromStorage;

    // NOTE: Layout Restrictor checks should be left to the Cortical Type
    /// Creates a new generator wtih init values that is ready for use (but can still be modified)
    fn new(input: Self::GeneratorInitParameters, layout: NeuronLayoutOf<NM, FIQ>) -> Self;
    
    fn new_generator_from_storage(storage: &NeurogenesisFromStorage<FIQ, NM>, other_data: Self::OtherDataNeededForFromStorage) -> Self;

    // This trait is to be implemented using the super trait stuff
}