use core::ops::Range;
use feagi_structures::base_feagi_types::quantizable_types::{QuantizableNonzeroUIntType, QuantizableUIntType};
use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::NeuronDensityTrait;
use crate::dynamics::neuron::linear::collections::{NeuronModelCollectionBaseLinearTrait, NeuronModelCollectionPackedLinearTrait};
use crate::dynamics::neuron::shared::iteration::{EnumeratedLinearSetNeuron, EnumeratedLinearSetNeuronMut};
use crate::dynamics::neuron::shared::neuron_slices::{NeuronModelMutSlice, NeuronModelSlice};
use crate::dynamics::neuron::shared::neurons::NeuronModelParametersTrait;

/// Neurons are further grouped in regularly sized subunits, henceforth called "sets"
pub trait NeuronModelCollectionMultiNeuronLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType>:
NeuronModelCollectionBaseLinearTrait<CANQ, NMP>
{
    fn get_number_neurons_per_set(&self) -> ND;

    fn try_get_neuron_set_ref(&self, set_index: NeuronSetIndexTrait) -> NeuronModelSlice<CANQ, NMP>;

    fn try_get_neuron_set_mut_ref(&mut self, set_index: NeuronSetIndexTrait) -> NeuronModelMutSlice<CANQ, NMP>;

    fn enumerated_linear_neuron_set_iter(&self) -> impl Iterator<Item = EnumeratedLinearSetNeuron<CANQ, NMP, NeuronSetIndexTrait>>;

    fn enumerated_linear_neuron_set_iter_mut(&self) -> impl Iterator<Item = EnumeratedLinearSetNeuronMut<CANQ, NMP, NeuronSetIndexTrait>>;

    // TODO RAYON iterators

    fn get_max_number_neuron_sets(&self) -> NeuronSetIndexTrait {
        NeuronSetIndexTrait::from_usize(self.get_neuron_max_linear_index().to_usize() / self.get_number_neurons_per_set().to_usize())
    }
}


//region Packed

pub trait NeuronModelCollectionMultiNeuronPackedLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType>:
NeuronModelCollectionMultiNeuronLinearTrait<CANQ, NMP, ND, NeuronSetIndexTrait> +
NeuronModelCollectionPackedLinearTrait<CANQ, NMP>
{
    fn linear_neuron_set_iter(&self) -> impl Iterator<Item = NeuronModelSlice<CANQ, NMP>> {
        todo!()
    }

    fn linear_neuron_set_iter_mut(&mut self) -> impl Iterator<Item = NeuronModelMutSlice<CANQ, NMP>> {
        todo!()
    }
}

//endregion

//region Sparse

pub trait NeuronModelCollectionMultiNeuronSparseLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType>:
NeuronModelCollectionMultiNeuronLinearTrait<CANQ, NMP, ND, NeuronSetIndexTrait>
{
    // idk?
}

//endregion


pub(crate) fn neuron_set_to_usize_linear_range<NeuronSetIndexTrait: QuantizableUIntType>(set_index: NeuronSetIndexTrait, neurons_per_set: u8) -> Range<usize> {
    (neurons_per_set.to_usize() * set_index.to_usize()).. ((neurons_per_set.to_usize() * set_index.to_usize()) + neurons_per_set.to_usize())
}