use serde_json::Number;
use crate::base_feagi_types::quantizable_types::{QuantizableUIntType, QuantizableValueType};
use crate::neuron_voxel_collections::data_values::{NeuronVoxelDimensions, NeuronVoxelIndexCount};
use crate::neuron_collections::data_values::{NeuronDensityPerVoxel, NeuronIndexCount, NeuronMembranePotential};



pub trait SingleCorticalNeuronCollectionBase<NCQL: NeuronCollectionQuantizationLevelType>
{
    fn get_neuron_voxel_density(&self) -> NeuronDensityPerVoxel;

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<NCQL::VoxelCoordQuant>;

    fn number_neurons(&self) -> NeuronIndexCount<NCQL::NeuronIndexCountQuant>;

    fn number_voxels(&self) -> NeuronVoxelIndexCount<NCQL::VoxelCoordQuant>;
}

pub trait SingleCorticalNeuronCollectionDense<NCQL: NeuronCollectionQuantizationLevelType>:
SingleCorticalNeuronCollectionBase<NCQL>
{
    fn get_all_neuron_potentials(&self) -> &[NeuronMembranePotential<NCQL::NeuronPotentialQuant>];

    fn get_all_neuron_potentials_mut(&mut self) -> &mut [NeuronMembranePotential<NCQL::NeuronPotentialQuant>];

    // TODO iterators?

    // TODO par iterators?
}