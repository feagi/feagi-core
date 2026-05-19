use core::ops::Range;
use feagi_structures::base_feagi_types::quantizable_types::{QuantizableNonzeroUIntType, QuantizableUIntType};
use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{LinearNeuronIndexCount, NeuronDensityTrait, NeuronMembranePotential};
use crate::dynamics::neuron::linear::collections::{NeuronModelCollectionBaseLinearTrait, NeuronModelCollectionPackedLinearTrait, NeuronModelMutSlice, NeuronModelSlice};
use crate::dynamics::neuron::shared::neurons::{NeuronDataRef, NeuronModelParametersTrait};

/// Neurons are further grouped in regularly sized subunits, henceforth called "sets"
pub trait NeuronModelCollectionMultiNeuronLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType>:
NeuronModelCollectionBaseLinearTrait<CANQ, NMP>
{
    fn get_number_neurons_per_set(&self) -> ND;

    fn get_max_number_neuron_sets(&self) -> NeuronSetIndexTrait;

    fn try_get_neuron_set_ref(&self, set_index: NeuronSetIndexTrait) -> NeuronModelSlice<CANQ, NMP>;

    fn try_get_neuron_set_mut_ref(&mut self, set_index: NeuronSetIndexTrait) -> NeuronModelMutSlice<CANQ, NMP>;

    fn enumerated_linear_neuron_set_iter(&self) -> impl Iterator<Item = EnumeratedLinearSetNeuron<CANQ, NMP, NeuronSetIndexTrait>>;

    fn enumerated_linear_neuron_set_iter_mut(&self) -> impl Iterator<Item = EnumeratedLinearSetNeuronMut<CANQ, NMP, NeuronSetIndexTrait>>;

    // TODO RAYON iterators
}

//region Iterators

pub struct EnumeratedLinearSetNeuron<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, NeuronSetIndexTrait: QuantizableUIntType> {
    neuron_set_index: NeuronSetIndexTrait,
    potentials: &'a [NeuronMembranePotential<CANQ::NeuronValueQuant>],
    model_parameters: &'a [NMP],
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, NeuronSetIndexTrait: QuantizableUIntType> EnumeratedLinearSetNeuron<'a, CANQ, NMP, NeuronSetIndexTrait> {
    pub fn get_set_index(&self) -> &NeuronSetIndexTrait {
        &self.neuron_set_index
    }

    pub fn neuron_ref(&self) -> NeuronModelSlice<'a, CANQ, NMP> {
        NeuronModelSlice {
            neuron_potentials: self.potentials,
            get_model_parameters: self.model_parameters,
        }
    }
}

pub struct EnumeratedLinearSetNeuronMut<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, NeuronSetIndexTrait: QuantizableUIntType> {
    neuron_set_index: NeuronSetIndexTrait,
    potentials: &'a mut [NeuronMembranePotential<CANQ::NeuronValueQuant>],
    model_parameters: &'a mut [NMP],
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, NeuronSetIndexTrait: QuantizableUIntType> EnumeratedLinearSetNeuronMut<'a, CANQ, NMP, NeuronSetIndexTrait> {
    pub fn get_set_index(&self) -> &NeuronSetIndexTrait {
        &self.neuron_set_index
    }

    pub fn neuron_ref(&self) -> NeuronModelSlice<'a, CANQ, NMP> {
        NeuronModelSlice {
            neuron_potentials: self.potentials,
            get_model_parameters: self.model_parameters,
        }
    }

    pub fn neuron_ref_mut(&mut self) -> NeuronModelMutSlice<'a, CANQ, NMP> {
        NeuronModelMutSlice {
            neuron_potentials: self.potentials,
            get_model_parameters: self.model_parameters,
        }
    }
}

//endregion

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