use feagi_structures::base_feagi_types::quantizable_types::{QuantizableNonzeroUIntType, QuantizableUIntType};
use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{FeagiNeuronError, LinearNeuronIndexCount, NeuronMembranePotential};
use crate::dynamics::neuron::linear::collections::{NeuronCollectionType, NeuronModelCollectionBaseLinearTrait, NeuronModelCollectionPackedLinearTrait};
use crate::dynamics::neuron::shared::iteration::{EnumeratedLinearNeuron, EnumeratedLinearNeuronMut, PackedLinearIteration, PackedLinearIterationMut};
use crate::dynamics::neuron::shared::neurons::{NeuronData, NeuronDataRef, NeuronDataRefMut, NeuronModelParametersTrait};

pub struct NeuronCollectionLinearPackedVector<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{
    membrane_potentials: Vec<NeuronMembranePotential<CANQ::NeuronValueQuant>>,
    model_parameters: Vec<NMP>
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronModelCollectionBaseLinearTrait<CANQ, NMP> for NeuronCollectionLinearPackedVector<CANQ, NMP>
{
    const NEURON_COLLECTION_TYPE: NeuronCollectionType = NeuronCollectionType::PackedResizableVector;

    fn is_sorted_in_increasing_index_order(&self) -> bool {
        true
    }

    fn get_neuron_max_linear_index(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        LinearNeuronIndexCount::from_usize(self.membrane_potentials.len())
    }

    fn get_number_contained_neurons(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        LinearNeuronIndexCount::from_usize(self.membrane_potentials.len())
    }

    fn try_get_neuron_data(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronData<CANQ, NMP>, FeagiNeuronError> {
        //TODO Debug mode bounds checking only
        Ok(NeuronData::new(
            self.membrane_potentials[index.to_usize()],
            self.model_parameters[index.to_usize()],
        ))
    }

    fn try_get_neuron_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronDataRef<CANQ, NMP>, FeagiNeuronError> {
        //TODO Debug mode bounds checking only
        Ok(NeuronDataRef::new(
            &self.membrane_potentials[index.to_usize()],
            &self.model_parameters[index.to_usize()],
        ))
    }

    fn try_get_neuron_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronDataRefMut<CANQ, NMP>, FeagiNeuronError> {
        //TODO Debug mode bounds checking only
        Ok(NeuronDataRefMut::new(
            &mut self.membrane_potentials[index.to_usize()],
            &mut self.model_parameters[index.to_usize()],
        ))
    }

    fn try_get_membrane_potential_data(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError> {
        self.membrane_potentials[index.to_usize()]
    }

    fn try_get_membrane_potential_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError> {
        &self.membrane_potentials[index.to_usize()]
    }

    fn try_get_membrane_potential_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&mut NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError> {
        &mut self.membrane_potentials[index.to_usize()]
    }

    fn try_get_neuron_model_data(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NMP, FeagiNeuronError> {
        self.model_parameters[index.to_usize()]
    }

    fn try_get_neuron_model_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&NMP, FeagiNeuronError> {
        &self.model_parameters[index.to_usize()]
    }

    fn try_get_neuron_model_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&mut NMP, FeagiNeuronError> {
        &mut self.model_parameters[index.to_usize()]
    }

    fn enumerated_linear_neuron_iter(&self) -> impl Iterator<Item=EnumeratedLinearNeuron<CANQ, NeuronDataRef<CANQ, NMP>>> {
        todo!()
    }

    fn enumerated_linear_neuron_iter_mut(&self) -> impl Iterator<Item=EnumeratedLinearNeuronMut<CANQ, NeuronDataRefMut<CANQ, NMP>>> {
        todo!()
    }
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> PackedLinearIterationMut<'static, CANQ, NMP> for NeuronCollectionLinearPackedVector<CANQ, NMP> {
    fn linear_neuron_iter_mut(&mut self) -> impl Iterator<Item=NeuronDataRefMut<'static, CANQ, NMP>> {
        todo!()
    }
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> PackedLinearIteration<'static, CANQ, NMP> for NeuronCollectionLinearPackedVector<CANQ, NMP> {
    fn linear_neuron_iter(&self) -> impl Iterator<Item=NeuronDataRef<'static, CANQ, NMP>> {
        todo!()
    }
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronModelCollectionPackedLinearTrait<CANQ, NMP> for NeuronCollectionLinearPackedVector<CANQ, NMP>
{
    fn get_membrane_potentials_as_slice(&self) -> &[NeuronMembranePotential<CANQ::NeuronValueQuant>] {
        self.membrane_potentials.as_slice()
    }

    fn get_membrane_potentials_as_slice_mut(&mut self) -> &mut [NeuronMembranePotential<CANQ::NeuronValueQuant>] {
        self.membrane_potentials.as_mut_slice()
    }

    fn get_neuron_model_data_as_slice(&self) -> &[NMP] {
        self.model_parameters.as_slice()
    }

    fn get_neuron_model_data_as_slice_mut(&mut self) -> &mut [NMP] {
        self.model_parameters.as_mut_slice()
    }
}

impl<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>> NeuronCollectionLinearPackedVector<CANQ, NMP>
{
    pub(crate) fn new() -> Self {
        todo!()
    }
}

