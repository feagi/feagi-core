use crate::neuron_model::neurogenesis::from_storage::NeurogenesisByStorage;
use crate::neuron_model::neurogenesis::NeurogenesisMethod;
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;


/// The saveable data a cortical area is generated from, rather than read back from storage. Its
/// defaults are set by the cortical type, not the neuron model
pub trait NeurogenesisByGenerator<FIQ, NM>: NeurogenesisMethod<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    // new() gets defined by the impl itself

    /// Any other fields needed to create this from a storage generator
    type OtherDataNeededForFromStorage;

    fn new_from_storage(
        storage: &NeurogenesisByStorage<FIQ, NM>,
        other_data: Self::OtherDataNeededForFromStorage
    ) -> Self;
}



