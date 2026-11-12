use serde::Serialize;
use crate::neuron_model::neurogenesis::from_storage::NeurogenesisStorage;
use crate::neuron_model::neurogenesis::NeurogenesisMethod;
use crate::neuron_model::neuron_layout::{NeuronLayoutError, NeuronLayoutOf, NeuronLayoutRestrictor, NeuronLayoutTag};
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;

/// Specifies that the Neurogenesis process occurs by algorithm, not mearly reading preproduced
/// data. Effectively a marker trait for all `NeurogenesisMethod` impls that arent Storage
pub trait NeurogenesisGenerator<FIQ, NM>: NeurogenesisMethod<FIQ, NM> + Sized
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    // This trait is to be implemented using the super trait stuff
}

/// The data needed to build the Neurogenesis Generator for a given neuron model
pub trait NeurogenesisGeneratorPrecursor<FIQ, NM>: Sized + Clone + Serialize + serde::de::DeserializeOwned
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    // new() gets defined by the impl itself

    // TODO requant function?

    /// Any other fields needed to create this from a storage generator
    type OtherDataNeededForFromStorage;

    fn new_from_storage(
        storage: &NeurogenesisStorage<FIQ, NM>,
        other_data: Self::OtherDataNeededForFromStorage
    ) -> Self;

    fn into_generator<Res: NeuronLayoutRestrictor<FIQ, NeuronLayout=NeuronLayoutOf<FIQ, NM>>>(self) -> Result<NM::NeurogenesisGenerator<FIQ>, NeuronLayoutError>;
}

