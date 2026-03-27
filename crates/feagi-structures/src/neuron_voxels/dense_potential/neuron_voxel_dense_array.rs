use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::base_quantizable::value::QuantizableValue;
use crate::neuron_descriptors::NeuronPotentialUnit;
use crate::neuron_voxels::descriptors::{NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelPotential};
use crate::neuron_voxels::traits::{SingleCorticalNeuronVoxelCollectionBase, SingleCorticalNeuronVoxelCollectionDense, SingleCorticalNeuronVoxelCollectionSparse};

pub struct NeuronVoxelDenseArray<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, const NUMBER_NEURON_VOXELS> where
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt
{
    cortical_dimensions: NeuronVoxelDimensions<CoordQuant>,
    potentials: [NeuronVoxelPotential<VoxelPotentialQuant>; NUMBER_NEURON_VOXELS],
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, const NUMBER_NEURON_VOXELS> NeuronVoxelDenseArray<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, NUMBER_NEURON_VOXELS> {

    pub const fn new(cortical_dimensions: NeuronVoxelDimensions<CoordQuant>) -> Self {
        Self {
            cortical_dimensions,
            potentials: [0; NUMBER_NEURON_VOXELS],
        }
    }


}


impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelDenseArray<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, const NUMBER_NEURON_VOXELS>
{
    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CoordQuant> {
        &self.cortical_dimensions
    }

    fn neuron_index_max_limit(&self) -> NeuronVoxelIndexQuant {
        self.potentials.len() as NeuronVoxelIndexQuant
    }
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, const NUMBER_NEURON_VOXELS>
SingleCorticalNeuronVoxelCollectionDense<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelDenseArray<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, NUMBER_NEURON_VOXELS>
{
    fn get_all_neuron_voxel_potentials(&self) -> &[NeuronVoxelPotential<VoxelPotentialQuant>] {
        self.potentials.as_slice()
    }

    fn get_all_neuron_voxel_potentials_mut(&mut self) -> &mut [NeuronVoxelPotential<VoxelPotentialQuant>] {
        self.potentials.as_mut_slice()
    }

    fn iter_nonzero_index(&self) -> impl Iterator<Item=(&NeuronVoxelIndexQuant, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        todo!()
    }

    fn iter_nonzero_coordinate(&self) -> impl Iterator<Item=(&NeuronVoxelCoordinate<CoordQuant>, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        todo!()
    }

    fn zero_all_neuron_voxel_potentials(&mut self) {
        todo!()
    }

    #[cfg(feature = "alloc")]
    fn inplace_overwrite_data_from_sparse(&mut self, sparse_neurons: &impl SingleCorticalNeuronVoxelCollectionSparse<NeuronVoxelPotential<VoxelPotentialQuant>, CoordQuant, NeuronVoxelIndexQuant>) {
        todo!()
    }
}