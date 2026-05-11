use crate::base_feagi_types::quantizable_types::{FeagiBaseSingleElementQuantizationType, QuantizableUIntType};
use crate::neuron_collections::neuron_voxel_collections::voxel_structs::{
    NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelIndexCount, NeuronVoxelPotential,
    SingleCorticalNeuronVoxelCollectionType,
};
use crate::neuron_collections::neuron_voxel_collections::traits::{
    NeuronVoxelCollectionBase, SingleCorticalNeuronVoxelCollectionDense,
    NeuronVoxelCollectionSparse,
};
use crate::quantization_level::CorticalAreaNeuronQuantization;

pub struct NeuronVoxelDenseArray<
    CANQ: CorticalAreaNeuronQuantization,
    const NUMBER_NEURON_VOXELS: usize,
>
{
    cortical_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>,
    potentials: [NeuronVoxelPotential<CANQ::NeuronValueQuant>; NUMBER_NEURON_VOXELS],
}

impl<CANQ: CorticalAreaNeuronQuantization, const NUMBER_NEURON_VOXELS: usize>
    NeuronVoxelDenseArray<CANQ, NUMBER_NEURON_VOXELS>
{
    pub fn new(cortical_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>) -> Self {
        Self {
            cortical_dimensions,
            potentials: [NeuronVoxelPotential::ZERO; NUMBER_NEURON_VOXELS],
        }
    }
}

impl<CANQ: CorticalAreaNeuronQuantization, const NUMBER_NEURON_VOXELS: usize>
    NeuronVoxelCollectionBase<CANQ>
    for NeuronVoxelDenseArray<CANQ, NUMBER_NEURON_VOXELS>
{
    const COLLECTION_TYPE: SingleCorticalNeuronVoxelCollectionType =
        SingleCorticalNeuronVoxelCollectionType::DenseArray;

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant> {
        &self.cortical_dimensions
    }

    fn get_neuron_voxel_max_index(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        NeuronVoxelIndexCount::from_usize(self.potentials.len())
    }

    fn iter_index(&self) -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        self.potentials
            .iter()
            .enumerate()
            .map(|(index, potential)| (NeuronVoxelIndexCount::from_usize(index), *potential))
    }

    #[cfg(feature = "rayon")]
    fn iter_index_par(&self) -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        self.iter_index()
    }

    fn iter_nonzero_potential_index(&self) -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        self.iter_index()
            .filter(|(_, potential)| *potential != NeuronVoxelPotential::ZERO)
    }

    #[cfg(feature = "rayon")]
    fn iter_nonzero_potential_index_par(&self) -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        self.iter_nonzero_potential_index()
    }

    fn iter_coordinate(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        let dims = &self.cortical_dimensions;
        self.iter_index()
            .map(move |(idx, p)| (dims.linear_index_to_standard_voxel_coordinate(idx), p))
    }

    #[cfg(feature = "rayon")]
    fn iter_coordinate_par(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        self.iter_coordinate()
    }

    fn iter_nonzero_potential_coordinate(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        let dims = &self.cortical_dimensions;
        self.iter_nonzero_potential_index()
            .map(move |(idx, p)| (dims.linear_index_to_standard_voxel_coordinate(idx), p))
    }

    #[cfg(feature = "rayon")]
    fn iter_nonzero_potential_coordinate_par(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        self.iter_nonzero_potential_coordinate()
    }
}

impl<CANQ: CorticalAreaNeuronQuantization, const NUMBER_NEURON_VOXELS: usize>
    SingleCorticalNeuronVoxelCollectionDense<CANQ>
    for NeuronVoxelDenseArray<CANQ, NUMBER_NEURON_VOXELS>
{
    fn get_all_neuron_voxel_potentials(&self) -> &[NeuronVoxelPotential<CANQ::NeuronValueQuant>] {
        self.potentials.as_slice()
    }

    fn get_all_neuron_voxel_potentials_mut(&mut self) -> &mut [NeuronVoxelPotential<CANQ::NeuronValueQuant>] {
        self.potentials.as_mut_slice()
    }

    fn zero_all_neuron_voxel_potentials(&mut self) {
        self.potentials.fill(NeuronVoxelPotential::ZERO);
    }

    #[cfg(feature = "alloc")]
    fn inplace_overwrite_data_from_sparse(&mut self, sparse_neurons: &impl NeuronVoxelCollectionSparse<CANQ>, zero_out_first: bool) {
        if zero_out_first {
            self.zero_all_neuron_voxel_potentials();
        }

        for (index, potential) in sparse_neurons.iter_nonzero_potential_index() {
            self.potentials[index.to_usize()] = potential;
        }
    }
}
