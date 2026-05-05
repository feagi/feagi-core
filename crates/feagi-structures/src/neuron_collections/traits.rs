use serde_json::Number;
use crate::base_feagi_types::quantizable_types::{QuantizableUIntType, QuantizableValueType};
use crate::neuron_voxel_collections::data_values::{NeuronVoxelCount, NeuronVoxelDimensions, NeuronVoxelIndex};
use crate::neuron_collections::data_values::{NeuronDensityPerVoxel, NeuronIndex, NeuronMembranePotential};

/// Defines quantization level for a Neuron Collection (NOT a voxel neuron collection!)
pub trait NeuronCollectionQuantizationLevelType {
    type NeuronIndexCountQuant: QuantizableUIntType;
    type VoxelCoordQuant: QuantizableUIntType;
    type NeuronPotentialQuant: QuantizableValueType;
}

pub trait SingleCorticalNeuronCollectionBase<NCQL: NeuronCollectionQuantizationLevelType>
{
    fn get_neuron_voxel_density(&self) -> NeuronDensityPerVoxel;

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<NCQL::VoxelCoordQuant>;

    fn neuron_index_max_limit(&self) -> NeuronIndex<NCQL::NeuronIndexCountQuant>;

    fn neuron_voxel_index_max_limit(&self) -> NeuronVoxelIndex<NCQL::NeuronIndexCountQuant>;

    fn number_of_voxels(&self) -> NeuronVoxelCount<NCQL::NeuronIndexCountQuant>;
}

pub trait SingleCorticalNeuronCollectionDense<NCQL: NeuronCollectionQuantizationLevelType>:
SingleCorticalNeuronCollectionBase<NCQL>
{
    fn get_all_neuron_potentials(&self) -> &[NeuronMembranePotential<NCQL::NeuronPotentialQuant>];

    fn get_all_neuron_potentials_mut(&mut self) -> &mut [NeuronMembranePotential<NCQL::NeuronPotentialQuant>];

    // TODO iterators?

    // TODO par iterators?
}