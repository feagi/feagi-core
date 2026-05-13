use crate::base_feagi_types::quantizable_types::{FeagiBaseSingleElementQuantizationType, QuantizableUIntType};
use crate::neuron_collections::common_neuron_structs::{NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelIndexCount, NeuronVoxelPotential};
use crate::neuron_collections::neuron_voxel_collections::traits::{NeuronVoxelCollectionResizable, NeuronVoxelCollectionBase, NeuronVoxelCollectionSparse};
use crate::quantization_level::CorticalAreaNeuronQuantization;

pub struct NeuronVoxelIndexVector<CANQ: CorticalAreaNeuronQuantization>
{
    cortical_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>,
    indexes: Vec<NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>>,
    potentials: Vec<NeuronVoxelPotential<CANQ::NeuronValueQuant>>,
}

impl<CANQ: CorticalAreaNeuronQuantization> NeuronVoxelIndexVector<CANQ>
{
    pub fn new(
        cortical_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>,
        number_neurons_preallocated: NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    ) -> Self {
        Self {
            cortical_dimensions,
            indexes: Vec::with_capacity(number_neurons_preallocated.to_usize()),
            potentials: Vec::with_capacity(number_neurons_preallocated.to_usize()),
        }
    }

    pub fn iter_coordinate(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> + '_ {
        let dims = &self.cortical_dimensions;
        self.iter_index()
            .map(move |(idx, p)| (dims.linear_index_to_standard_voxel_coordinate(idx), p))
    }

    #[cfg(feature = "rayon")]
    pub fn iter_coordinate_par(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> + '_ {
        self.iter_coordinate()
    }
}

impl<CANQ: CorticalAreaNeuronQuantization> NeuronVoxelCollectionBase<CANQ>
for NeuronVoxelIndexVector<CANQ>
{
    const COLLECTION_TYPE: SingleCorticalNeuronVoxelCollectionType = SingleCorticalNeuronVoxelCollectionType::IndexVector;

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant> {
        &self.cortical_dimensions
    }

    fn get_neuron_voxel_max_index(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        NeuronVoxelIndexCount::from_usize(self.cortical_dimensions.get_max_allowed_index_exclusive())
    }

    fn iter_index(&self) -> impl Iterator<Item=(NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>, NeuronVoxelPotential<CANQ::NeuronValueQuant>)> {
        self.indexes
            .iter()
            .copied()
            .zip(self.potentials.iter().copied())
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
        NeuronVoxelIndexVector::iter_coordinate(self)
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
for NeuronVoxelIndexVector<CANQ>
{
    fn get_number_neuron_voxel_contained_count(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        NeuronVoxelIndexCount::from_usize(self.potentials.len())
    }

    fn get_neuron_voxel_count_allocated_capacity(&self) -> NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        NeuronVoxelIndexCount::from_usize(self.potentials.capacity())
    }

    fn reserve(&mut self, number_of_neuron_voxels_to_reserve_for: NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>) {
        self.potentials.reserve(number_of_neuron_voxels_to_reserve_for.to_usize());
        self.indexes.reserve(number_of_neuron_voxels_to_reserve_for.to_usize());
    }

    fn empty_and_change_cortical_area_dimensions(&mut self, new_dimensions: NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>) {
        self.clear_all_neurons();
        self.cortical_dimensions = new_dimensions;
    }

    fn shrink_to_fit(&mut self) {
        self.potentials.shrink_to_fit();
        self.indexes.shrink_to_fit();
    }
}

impl<CANQ: CorticalAreaNeuronQuantization> NeuronVoxelCollectionSparse<CANQ>
for NeuronVoxelIndexVector<CANQ>
{
    fn is_sorted(&self) -> bool {
        self.indexes.windows(2).all(|pair| pair[0] <= pair[1])
    }

    fn clear_all_neurons(&mut self) {
        self.potentials.clear();
        self.indexes.clear();
    }

    fn sort(&mut self) {
        let n = self.indexes.len();
        if n <= 1 {
            return;
        }
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by_key(|&i| self.indexes[i].to_usize());
        let indexes = core::mem::take(&mut self.indexes);
        let potentials = core::mem::take(&mut self.potentials);
        self.indexes = order.iter().map(|&i| indexes[i]).collect();
        self.potentials = order.iter().map(|&i| potentials[i]).collect();
    }
}