use crate::base_feagi_types::quantizable_types::{FeagiBaseSingleElementQuantizationType, QuantizableUIntType};
use crate::neuron::neuron_collections::neuron_voxel_collections::voxel_structs::{NeuronVoxelCoordinate, NeuronVoxelIndexCount, NeuronVoxelDimensions, NeuronVoxelPotential, SingleCorticalNeuronVoxelCollectionType};
use crate::neuron::neuron_collections::neuron_voxel_collections::traits::{NeuronVoxelCollectionResizable, NeuronVoxelCollectionBase, SingleCorticalNeuronVoxelCollectionDense, NeuronVoxelCollectionSparse};
use crate::quantization_level::CorticalAreaNeuronQuantization;

pub struct NeuronVoxelDenseVector<CANQ: CorticalAreaNeuronQuantization>
{
    cortical_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>,
    potentials: Vec<NeuronVoxelPotential<CANQ::NeuronValueQuant>>,
}

impl<CANQ: CorticalAreaNeuronQuantization> NeuronVoxelDenseVector<CANQ>
{
    pub fn new(cortical_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>) -> Self {
        let number_neurons = cortical_dimensions.get_max_allowed_index_exclusive();
        Self {
            cortical_dimensions,
            potentials: vec!(NeuronVoxelPotential::ZERO; number_neurons),
        }
    }
}

impl<CANQ: CorticalAreaNeuronQuantization> NeuronVoxelCollectionBase<CANQ>
for NeuronVoxelDenseVector<CANQ>
{
    const COLLECTION_TYPE: SingleCorticalNeuronVoxelCollectionType = SingleCorticalNeuronVoxelCollectionType::DenseVector;

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

impl<CANQ: CorticalAreaNeuronQuantization> NeuronVoxelCollectionResizable<CANQ>
for NeuronVoxelDenseVector<CANQ>
{
    fn get_number_neuron_voxel_contained_count(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        NeuronVoxelIndexCount::from_usize(self.potentials.len())
    }

    fn get_neuron_voxel_count_allocated_capacity(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        NeuronVoxelIndexCount::from_usize(self.potentials.capacity())
    }

    fn reserve(&mut self, number_of_neuron_voxels_to_reserve_for: NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>) {
        self.potentials.reserve(number_of_neuron_voxels_to_reserve_for.to_usize());
    }

    fn empty_and_change_cortical_area_dimensions(&mut self, new_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>) {
        let number_neurons = new_dimensions.get_max_allowed_index_exclusive();
        self.potentials.clear();
        self.potentials.resize(number_neurons, NeuronVoxelPotential::ZERO);
        self.cortical_dimensions = new_dimensions;
    }

    fn shrink_to_fit(&mut self) {
        // Does nothing, we can never shrink to fit as this is always dense
    }
}

impl<CANQ: CorticalAreaNeuronQuantization> SingleCorticalNeuronVoxelCollectionDense<CANQ>
for NeuronVoxelDenseVector<CANQ>
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

    fn inplace_overwrite_data_from_sparse(&mut self, sparse_neurons: &impl NeuronVoxelCollectionSparse<CANQ>, zero_out_first: bool) {
        if zero_out_first {
            self.zero_all_neuron_voxel_potentials();
        }

        for (index, potential) in sparse_neurons.iter_nonzero_potential_index() {
            self.potentials[index.to_usize()] = potential;
        }
    }
}