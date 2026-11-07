use crate::neuron_model::neurogenesis::from_storage::NeurogenesisFromStorage;
use crate::neuron_model::neurogenesis::NeurogenesisMethod;
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;

/// Specifies that the Neurogenesis process occurs by algorithm, not mearly reading preproduced
/// data
pub trait NeurogenesisByGenerator<FIQ, NM>: NeurogenesisMethod<FIQ, NM> + Sized
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    /// Any other fields needed to create this generator from a storage generator
    type OtherDataNeededForFromStorage;

    fn new_generator_from_storage(storage: &NeurogenesisFromStorage<FIQ, NM>, other_data: Self::OtherDataNeededForFromStorage) -> Self;

    // This trait is to be implemented using the super trait stuff
}