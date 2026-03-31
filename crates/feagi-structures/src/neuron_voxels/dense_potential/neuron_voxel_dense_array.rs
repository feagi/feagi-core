use crate::base_quantizable::QuantizableUIntType;
use crate::base_quantizable::QuantizableValueType;
use crate::neuron_voxels::descriptors::{
    NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelPotential, SingleCorticalNeuronVoxelCollectionType,
};
use crate::neuron_voxels::traits::{
    SingleCorticalNeuronVoxelCollectionBase, SingleCorticalNeuronVoxelCollectionDense,
    SingleCorticalNeuronVoxelCollectionSparse,
};

pub struct NeuronVoxelDenseArray<
    VoxelPotentialQuant,
    CoordQuant,
    NeuronVoxelIndexQuant,
    const NUMBER_NEURON_VOXELS: usize,
> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
{
    cortical_dimensions: NeuronVoxelDimensions<CoordQuant>,
    potentials: [NeuronVoxelPotential<VoxelPotentialQuant>; NUMBER_NEURON_VOXELS],
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, const NUMBER_NEURON_VOXELS: usize>
    NeuronVoxelDenseArray<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, NUMBER_NEURON_VOXELS>
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
{
    pub fn new(cortical_dimensions: NeuronVoxelDimensions<CoordQuant>) -> Self {
        Self {
            cortical_dimensions,
            potentials: [NeuronVoxelPotential(VoxelPotentialQuant::ZERO); NUMBER_NEURON_VOXELS],
        }
    }
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, const NUMBER_NEURON_VOXELS: usize>
    SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
    for NeuronVoxelDenseArray<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, NUMBER_NEURON_VOXELS>
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
{
    const COLLECTION_TYPE: SingleCorticalNeuronVoxelCollectionType =
        SingleCorticalNeuronVoxelCollectionType::DenseArray;

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CoordQuant> {
        &self.cortical_dimensions
    }

    fn neuron_index_max_limit(&self) -> NeuronVoxelIndexQuant {
        NeuronVoxelIndexQuant::from_usize(self.potentials.len())
    }
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, const NUMBER_NEURON_VOXELS: usize>
    SingleCorticalNeuronVoxelCollectionDense<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
    for NeuronVoxelDenseArray<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, NUMBER_NEURON_VOXELS>
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
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
