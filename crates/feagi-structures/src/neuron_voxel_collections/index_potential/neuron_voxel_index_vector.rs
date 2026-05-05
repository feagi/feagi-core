use crate::base_feagi_types::quantizable_types::QuantizableUIntType;
use crate::base_feagi_types::quantizable_types::QuantizableValueType;
use crate::neuron_voxel_collections::data_values::{NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelPotential, SingleCorticalNeuronVoxelCollectionType};
use crate::neuron_voxel_collections::traits::{SingleCorticalNeuronVoxelCollectionAlloc, SingleCorticalNeuronVoxelCollectionBase, SingleCorticalNeuronVoxelCollectionSparse};

pub struct NeuronVoxelIndexVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    cortical_dimensions: NeuronVoxelDimensions<CoordQuant>,
    indexes: Vec<NeuronVoxelIndexQuant>,
    potentials: Vec<VoxelPotentialQuant>,
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> NeuronVoxelIndexVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{

    pub fn new(cortical_dimensions: NeuronVoxelDimensions<CoordQuant>, number_neurons_preallocated: NeuronVoxelIndexQuant) -> Self {
        Self {
            cortical_dimensions,
            indexes: Vec::with_capacity(number_neurons_preallocated.to_usize()),
            potentials: Vec::with_capacity(number_neurons_preallocated.to_usize()),
        }
    }
}


impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelIndexVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    const COLLECTION_TYPE: SingleCorticalNeuronVoxelCollectionType = SingleCorticalNeuronVoxelCollectionType::IndexVector;

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CoordQuant> {
        &self.cortical_dimensions
    }

    fn neuron_index_max_limit(&self) -> NeuronVoxelIndexQuant {
        NeuronVoxelIndexQuant::from_usize(self.cortical_dimensions.get_max_allowed_index_exclusive())
    }
}


impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionAlloc<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelIndexVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    fn get_number_neuron_voxel_contained_count(&self) -> NeuronVoxelIndexQuant {
        NeuronVoxelIndexQuant::from_usize(self.potentials.len())
    }

    fn get_neuron_voxel_count_allocated_capacity(&self) -> NeuronVoxelIndexQuant {
        NeuronVoxelIndexQuant::from_usize(self.potentials.capacity())

    }

    fn reserve(&mut self, number_of_neuron_voxels_to_reserve_for: NeuronVoxelIndexQuant) {
        self.potentials.reserve(number_of_neuron_voxels_to_reserve_for.to_usize());
        self.indexes.reserve(number_of_neuron_voxels_to_reserve_for.to_usize());
    }

    fn empty_and_change_cortical_area_dimensions(&mut self, new_dimensions: NeuronVoxelDimensions<CoordQuant>) {
        self.clear_all_neurons();
        self.cortical_dimensions = new_dimensions;
    }

    fn shrink_to_fit(&mut self) {
        self.potentials.shrink_to_fit();
        self.indexes.shrink_to_fit();
    }
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionSparse<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelIndexVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    fn clear_all_neurons(&mut self) {
        self.potentials.clear();
        self.indexes.clear();
    }

    fn iter_index(&self) -> impl Iterator<Item=(NeuronVoxelIndexQuant, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        self.indexes
            .iter()
            .copied()
            .zip(self.potentials.iter().copied().map(NeuronVoxelPotential))
    }

    fn iter_coordinate(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CoordQuant>, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        let dims = &self.cortical_dimensions;
        self.iter_index()
            .map(move |(idx, p)| (dims.linear_index_to_coordinate(idx), p))
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

    #[cfg(feature = "rayon")]
    fn iter_index_par(&self) -> impl Iterator<Item=(NeuronVoxelIndexQuant, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        self.iter_index()
    }

    #[cfg(feature = "rayon")]
    fn iter_coordinate_par(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CoordQuant>, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        self.iter_coordinate()
    }
}