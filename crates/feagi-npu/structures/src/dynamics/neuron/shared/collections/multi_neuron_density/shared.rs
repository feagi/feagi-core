use core::ops::Range;
use feagi_structures::base_feagi_types::quantizable_types::{QuantizableUIntType};
use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{FeagiNeuronError, NeuronDensityTrait, NeuronMembranePotential};
use crate::dynamics::neuron::linear::collections::{NeuronModelCollectionBaseLinearTrait, NeuronModelCollectionPackedLinearTrait};
use crate::dynamics::neuron::shared::iteration::{EnumeratedLinearSetNeuron, EnumeratedLinearSetNeuronMut};
use crate::dynamics::neuron::shared::neuron_slices::{NeuronModelMutSlice, NeuronModelSlice};
use crate::dynamics::neuron::shared::neurons::NeuronModelParametersTrait;

/// Neurons are further grouped in regularly sized subunits, henceforth called "groups"
pub trait NeuronModelCollectionMultiNeuronLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait>:
NeuronModelCollectionBaseLinearTrait<CANQ, NMP>
{
    type GroupIndexStruct: QuantizableUIntType;

    fn get_number_neurons_per_set(&self) -> ND;

    fn try_get_neuron_group_ref(&self, set_index: Self::GroupIndexStruct) -> Result<NeuronModelSlice<CANQ, NMP>, FeagiNeuronError>;

    fn try_get_neuron_group_mut_ref(&mut self, set_index: Self::GroupIndexStruct) -> Result<NeuronModelMutSlice<CANQ, NMP>, FeagiNeuronError>;

    fn try_get_membrane_potential_data_group_ref(&self, set_index: Self::GroupIndexStruct) -> Result<&NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError>;

    fn try_get_membrane_potential_data_group_ref_mut(&mut self, set_index: Self::GroupIndexStruct) -> Result<&mut NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError>;

    fn try_get_neuron_model_data_group_ref(&self, set_index: Self::GroupIndexStruct) -> Result<&NMP, FeagiNeuronError>;

    fn try_get_neuron_model_data_group_ref_mut(&mut self, set_index: Self::GroupIndexStruct) -> Result<&mut NMP, FeagiNeuronError>;
    
    fn enumerated_linear_neuron_group_iter(&self) -> impl Iterator<Item = EnumeratedLinearSetNeuron<CANQ, NMP, Self::GroupIndexStruct>>;

    fn enumerated_linear_neuron_group_iter_mut(&self) -> impl Iterator<Item = EnumeratedLinearSetNeuronMut<CANQ, NMP, Self::GroupIndexStruct>>;

    // TODO RAYON iterators

    fn get_linear_range_from_group_index(&self, group_index: Self::GroupIndexStruct)

    fn get_max_number_neuron_groups(&self) -> Self::GroupIndexStruct {
        Self::GroupIndexStruct::from_usize(self.get_neuron_max_linear_index().to_usize() / self.get_number_neurons_per_set().to_usize())
    }
}


//region Packed

pub trait NeuronModelCollectionMultiNeuronPackedLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait>:
NeuronModelCollectionMultiNeuronLinearTrait<CANQ, NMP, ND> +
NeuronModelCollectionPackedLinearTrait<CANQ, NMP>
{
    fn linear_neuron_group_iter(&self) -> impl Iterator<Item = NeuronModelSlice<CANQ, NMP>> {
        todo!()
    }

    fn linear_neuron_group_iter_mut(&mut self) -> impl Iterator<Item = NeuronModelMutSlice<CANQ, NMP>> {
        todo!()
    }
}

//endregion

//region Sparse

pub trait NeuronModelCollectionMultiNeuronSparseLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait>:
NeuronModelCollectionMultiNeuronLinearTrait<CANQ, NMP, ND>
{
    // idk?
}

//endregion


pub(crate) fn neuron_group_to_usize_linear_range<NeuronSetIndexTrait: QuantizableUIntType>(set_index: NeuronSetIndexTrait, neurons_per_set: u8) -> Range<usize> {
    (neurons_per_set.to_usize() * set_index.to_usize()).. ((neurons_per_set.to_usize() * set_index.to_usize()) + neurons_per_set.to_usize())
}