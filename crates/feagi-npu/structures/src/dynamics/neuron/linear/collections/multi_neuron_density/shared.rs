use core::ops::Range;
use feagi_structures::base_feagi_types::quantizable_types::{QuantizableNonzeroUIntType, QuantizableUIntType};
use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{LinearNeuronIndexCount, NeuronDensityTrait, NeuronMembranePotential};
use crate::dynamics::neuron::linear::collections::{NeuronModelCollectionBaseLinearTrait, NeuronModelMutSlice, NeuronModelSlice};
use crate::dynamics::neuron::linear::neurons::NeuronModelParametersTrait;

/// Neurons are further grouped in regularly sized subunits, henceforth called "sets"
pub trait NeuronModelCollectionMultiNeuronLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType>:
NeuronModelCollectionBaseLinearTrait<CANQ, NMP>
{
    fn get_number_neurons_per_set(&self) -> ND;

    fn get_max_number_neuron_sets(&self) -> NeuronSetIndexTrait;

    fn try_get_neuron_set_ref(&self, set_index: NeuronSetIndexTrait) -> NeuronModelSlice<CANQ, NMP>;

    fn try_get_neuron_set_mut_ref(&mut self, set_index: NeuronSetIndexTrait) -> NeuronModelMutSlice<CANQ, NMP>;
}



pub(crate) fn neuron_set_to_usize_linear_range<NeuronSetIndexTrait: QuantizableUIntType>(set_index: NeuronSetIndexTrait, neurons_per_set: u8) -> Range<usize> {
    (neurons_per_set.to_usize() * set_index.to_usize()).. ((neurons_per_set.to_usize() * set_index.to_usize()) + neurons_per_set.to_usize())
}