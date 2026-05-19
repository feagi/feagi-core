use feagi_structures::CorticalAreaNeuronQuantization;
use feagi_structures::neuron::{FeagiNeuronError, LinearNeuronIndexCount, NeuronMembranePotential};
use crate::dynamics::neuron::shared::iteration::{EnumeratedLinearNeuron, EnumeratedLinearNeuronMut, PackedLinearIterationMut};
use crate::dynamics::neuron::shared::neuron_slices::{NeuronModelMutSlice, NeuronModelSlice};
use crate::dynamics::neuron::shared::neurons::{NeuronData, NeuronDataRef, NeuronDataRefMut, NeuronModelParametersTrait};

//region Shared Enums



/// Enum for defining what kind of collection a neuron collection is
#[derive(Clone, PartialEq)]
pub enum NeuronCollectionType {
    PackedFixedArray,
    PackedResizableVector,
    IndexedResizableVector,
    IndexedResizableHashmap,

    // TODO Array, fixed Hashmaps?
    // TODO others? Device support specifics?
}

//endregion


/// Base trait shared by all neuron model collections, which establishes a fallback universal way
/// to reference, modify, and iterate through any neuron collection structure
pub trait NeuronModelCollectionBaseLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>
{
    const NEURON_COLLECTION_TYPE: NeuronCollectionType;

    fn is_sorted_in_increasing_index_order(&self) -> bool;

    /// What is the upper bound (exclusive) neuron  index allowed?
    fn get_neuron_max_linear_index(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    /// In sparse implementations, then number of neurons contained may be less than the max possible index
    fn get_number_contained_neurons(&self) -> LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>;

    fn try_get_neuron_data(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronData<CANQ, NMP>, FeagiNeuronError>;

    fn try_get_neuron_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronDataRef<CANQ, NMP>, FeagiNeuronError>;

    fn try_get_neuron_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronDataRefMut<CANQ, NMP>, FeagiNeuronError>;

    fn try_get_membrane_potential_data(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError>;

    fn try_get_membrane_potential_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError>;

    fn try_get_membrane_potential_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&mut NeuronMembranePotential<CANQ::NeuronValueQuant>, FeagiNeuronError>;

    fn try_get_neuron_model_data(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<NMP, FeagiNeuronError>;

    fn try_get_neuron_model_data_ref(&self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&NMP, FeagiNeuronError>;

    fn try_get_neuron_model_data_ref_mut(&mut self, index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>) -> Result<&mut NMP, FeagiNeuronError>;

    fn enumerated_linear_neuron_iter(&self) -> impl Iterator<Item = EnumeratedLinearNeuron<CANQ, NeuronDataRef<CANQ, NMP>>>;

    fn enumerated_linear_neuron_iter_mut(&self) -> impl Iterator<Item = EnumeratedLinearNeuronMut<CANQ, NeuronDataRefMut<CANQ, NMP>>>;

    // TODO RAYON iterators
}

//region Shared Packed

/// Trait for structs that are densely packed (not sparse)
pub trait NeuronModelCollectionPackedLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
NeuronModelCollectionBaseLinearTrait<CANQ, NMP> +
PackedLinearIterationMut<'static, CANQ, NMP>
{
    fn get_membrane_potentials_as_slice(&self) -> &[NeuronMembranePotential<CANQ::NeuronValueQuant>];
    fn get_membrane_potentials_as_slice_mut(&mut self) -> &mut [NeuronMembranePotential<CANQ::NeuronValueQuant>];
    fn get_neuron_model_data_as_slice(&self) -> &[NMP];
    fn get_neuron_model_data_as_slice_mut(&mut self) -> &mut [NMP];

    fn get_neuron_data_as_slice(&self) -> NeuronModelSlice<'static, CANQ, NMP> {
        NeuronModelSlice {
            neuron_potentials: self.get_membrane_potentials_as_slice(),
            get_model_parameters: self.get_neuron_model_data_as_slice(),
        }
    }
    fn get_neuron_data_as_slice_mut(&mut self) -> NeuronModelMutSlice<'static, CANQ, NMP> {
        NeuronModelMutSlice {
            neuron_potentials: self.get_membrane_potentials_as_slice_mut(),
            get_model_parameters: self.get_neuron_model_data_as_slice_mut(),
        }
    }

    fn is_sorted_in_increasing_index_order(&self) -> bool {
        true // All packed instances are always sorted
    }
}

//endregion

//region Shared Sparse

/// Trait for structs that are sparse
pub trait NeuronModelCollectionSparseLinearTrait<CANQ: CorticalAreaNeuronQuantization, NMP: NeuronModelParametersTrait<CANQ>>:
NeuronModelCollectionBaseLinearTrait<CANQ, NMP>
{
    // idk?
}



//endregion