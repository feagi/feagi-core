use crate::neuron_voxel_collections::voxel_structs::{NeuronVoxelDimensions, NeuronVoxelIndexCount};
use crate::neuron_collections::data_values::{NeuronDensityPerVoxel, NeuronIndexCount, NeuronMembranePotential};
use crate::quantization_level::CorticalAreaNeuronQuantization;

pub trait SingleCorticalNeuronCollectionBase<CANQ: CorticalAreaNeuronQuantization>
{
    fn get_neuron_voxel_density(&self) -> NeuronDensityPerVoxel;

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CANQ::VoxelCoordQuant>;

    fn number_neurons(&self) -> NeuronIndexCount<CANQ::NeuronIndexCountQuant>;

    fn number_voxels(&self) -> NeuronVoxelIndexCount<CANQ::VoxelCoordQuant>;
}

pub trait SingleCorticalNeuronCollectionDense<CANQ: NeuronCollectionQuantizationLevelType>:
SingleCorticalNeuronCollectionBase<CANQ>
{
    fn get_all_neuron_potentials(&self) -> &[NeuronMembranePotential<CANQ::NeuronPotentialQuant>];

    fn get_all_neuron_potentials_mut(&mut self) -> &mut [NeuronMembranePotential<CANQ::NeuronPotentialQuant>];

    // TODO iterators?

    // TODO par iterators?
}