use feagi_structures::base_feagi_types::quantizable_types::QuantizableUIntType;
use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{FeagiNeuronError, LinearNeuronIndexCount, NeuronDensityTrait, NeuronMembranePotential};
use crate::dynamics::neuron::linear::collections::{NeuronCollectionType, NeuronModelCollectionBaseLinearTrait, NeuronModelCollectionPackedLinearTrait};
use crate::dynamics::neuron::shared::collections::multi_neuron_density::shared::{NeuronModelCollectionMultiNeuronLinearTrait, NeuronModelCollectionMultiNeuronPackedLinearTrait};
use crate::dynamics::neuron::shared::collections::structs::NeuronCollectionLinearPackedVector;
use crate::dynamics::neuron::shared::iteration::{EnumeratedLinearNeuron, EnumeratedLinearNeuronMut, EnumeratedLinearSetNeuron, EnumeratedLinearSetNeuronMut, PackedLinearIteration, PackedLinearIterationMut};
use crate::dynamics::neuron::shared::neuron_slices::{NeuronModelMutSlice, NeuronModelSlice};
use crate::dynamics::neuron::shared::neurons::{NeuronData, NeuronDataRef, NeuronDataRefMut, NeuronModelParametersTrait};

pub struct NeuronCollectionMultiLinearPackedVector<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType>
{
    pub linear_packed: NeuronCollectionLinearPackedVector<CANQ, NMP>,
    density: ND
}

//region Proxied

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType> NeuronModelCollectionBaseLinearTrait<CANQ, NMP> for NeuronCollectionMultiLinearPackedVector<CANQ, NMP, ND, NeuronSetIndexTrait> {
    const NEURON_COLLECTION_TYPE: NeuronCollectionType = NeuronCollectionType::PackedFixedArray;

    fn is_sorted_in_increasing_index_order(&self) -> bool {
        todo!()
    }

    fn get_neuron_max_linear_index(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        todo!()
    }

    fn get_number_contained_neurons(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        todo!()
    }

    fn try_get_neuron_data(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronData<CANQ, NMP>, FeagiNeuronError> {
        todo!()
    }

    fn try_get_neuron_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronDataRef<CANQ, NMP>, FeagiNeuronError> {
        todo!()
    }

    fn try_get_neuron_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronDataRefMut<CANQ, NMP>, FeagiNeuronError> {
        todo!()
    }

    fn try_get_membrane_potential_data(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError> {
        todo!()
    }

    fn try_get_membrane_potential_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError> {
        todo!()
    }

    fn try_get_membrane_potential_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&mut NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError> {
        todo!()
    }

    fn try_get_neuron_model_data(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NMP, FeagiNeuronError> {
        todo!()
    }

    fn try_get_neuron_model_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&NMP, FeagiNeuronError> {
        todo!()
    }

    fn try_get_neuron_model_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&mut NMP, FeagiNeuronError> {
        todo!()
    }

    fn enumerated_linear_neuron_iter(&self) -> impl Iterator<Item=EnumeratedLinearNeuron<CANQ, NeuronDataRef<CANQ, NMP>>> {
        todo!()
    }

    fn enumerated_linear_neuron_iter_mut(&self) -> impl Iterator<Item=EnumeratedLinearNeuronMut<CANQ, NeuronDataRefMut<CANQ, NMP>>> {
        todo!()
    }
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType> PackedLinearIterationMut<CANQ, NMP> for NeuronCollectionMultiLinearPackedVector<CANQ, NMP, ND, NeuronSetIndexTrait> {
    fn linear_neuron_iter_mut(&mut self) -> impl Iterator<Item=NeuronDataRefMut<'a, CANQ, NMP>> {
        todo!()
    }
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType> PackedLinearIteration<CANQ, NMP> for NeuronCollectionMultiLinearPackedVector<CANQ, NMP, ND, NeuronSetIndexTrait> {

    fn linear_neuron_iter(&self) -> impl Iterator<Item=NeuronDataRef<'a, CANQ, NMP>> {
        todo!()
    }
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType> NeuronModelCollectionPackedLinearTrait<CANQ, NMP> for NeuronCollectionMultiLinearPackedVector<CANQ, NMP, ND, NeuronSetIndexTrait> {
    fn get_membrane_potentials_as_slice(&self) -> &[NeuronMembranePotential<CANQ::NeuronValueQuant>] {
        todo!()
    }

    fn get_membrane_potentials_as_slice_mut(&mut self) -> &mut [NeuronMembranePotential<CANQ::NeuronValueQuant>] {
        todo!()
    }

    fn get_neuron_model_data_as_slice(&self) -> &[NMP] {
        todo!()
    }

    fn get_neuron_model_data_as_slice_mut(&mut self) -> &mut [NMP] {
        todo!()
    }
}

//endregion

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType> NeuronModelCollectionMultiNeuronLinearTrait<CANQ, NMP,ND, NeuronSetIndexTrait> for NeuronCollectionMultiLinearPackedVector<CANQ, NMP,ND, NeuronSetIndexTrait>
{
    fn get_number_neurons_per_set(&self) -> ND {
        self.density
    }

    fn try_get_neuron_set_ref(&self, set_index: NeuronSetIndexTrait) -> NeuronModelSlice<CANQ, NMP> {
        todo!()
    }

    fn try_get_neuron_set_mut_ref(&mut self, set_index: NeuronSetIndexTrait) -> NeuronModelMutSlice<CANQ, NMP> {
        todo!()
    }

    fn enumerated_linear_neuron_set_iter(&self) -> impl Iterator<Item=EnumeratedLinearSetNeuron<CANQ, NMP, NeuronSetIndexTrait>> {
        todo!()
    }

    fn enumerated_linear_neuron_set_iter_mut(&self) -> impl Iterator<Item=EnumeratedLinearSetNeuronMut<CANQ, NMP, NeuronSetIndexTrait>> {
        todo!()
    }
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType> NeuronModelCollectionMultiNeuronPackedLinearTrait<CANQ, NMP,ND, NeuronSetIndexTrait> for NeuronCollectionMultiLinearPackedVector<CANQ, NMP,ND, NeuronSetIndexTrait>
{

}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>, ND: NeuronDensityTrait, NeuronSetIndexTrait: QuantizableUIntType> NeuronCollectionMultiLinearPackedVector<CANQ, NMP, ND, NeuronSetIndexTrait>
{
    pub(crate) fn new() -> Self {
        todo!()
    }
}







