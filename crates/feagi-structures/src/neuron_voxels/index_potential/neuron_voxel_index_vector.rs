use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::base_quantizable::value::QuantizableValue;
use crate::neuron_voxels::descriptors::{NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelPotential};
use crate::neuron_voxels::traits::{SingleCorticalNeuronVoxelCollectionAlloc, SingleCorticalNeuronVoxelCollectionBase, SingleCorticalNeuronVoxelCollectionSparse};

pub struct NeuronVoxelIndexVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt
{
    cortical_dimensions: NeuronVoxelDimensions<CoordQuant>,
    indexes: Vec<NeuronVoxelIndexQuant>,
    potentials: Vec<VoxelPotentialQuant>,
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> NeuronVoxelIndexVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt
{

    pub fn new(cortical_dimensions: NeuronVoxelDimensions<CoordQuant>, number_neurons_preallocated: NeuronVoxelIndexQuant) -> Self {
        Self {
            cortical_dimensions,
            indexes: Vec::with_capacity(number_neurons_preallocated),
            potentials: Vec::with_capacity(number_neurons_preallocated),
        }
    }
}


impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelIndexVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt
{
    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CoordQuant> {
        &self.cortical_dimensions
    }

    fn neuron_index_max_limit(&self) -> NeuronVoxelIndexQuant {
        self.cortical_dimensions.get_max_allowed_index_exclusive()
    }
}


impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionAlloc<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelIndexVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt
{
    fn get_number_neuron_voxel_contained_count(&self) -> NeuronVoxelIndexQuant {
        self.potentials.len() as NeuronVoxelIndexQuant
    }

    fn get_neuron_voxel_count_allocated_capacity(&self) -> usize {
        self.potentials.capacity()
    }

    fn reserve(&mut self, number_of_neuron_voxels_to_reserve_for: NeuronVoxelIndexQuant) {
        self.potentials.reserve(number_of_neuron_voxels_to_reserve_for as usize);
        self.indexes.reserve(number_of_neuron_voxels_to_reserve_for as usize);
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
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt
{
    fn clear_all_neurons(&mut self) {
        self.potentials.clear();
        self.indexes.clear();
    }

    fn iter_index(&self) -> impl Iterator<Item=(&NeuronVoxelIndexQuant, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        todo!()
    }

    fn iter_coordinate(&self) -> impl Iterator<Item=(&NeuronVoxelCoordinate<CoordQuant>, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        todo!()
    }

    fn sort(&mut self) {
        todo!()
    }

    #[cfg(feature = "rayon")]
    fn iter_index_par(&self) -> impl Iterator<Item=(&NeuronVoxelIndexQuant, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        todo!()
    }

    #[cfg(feature = "rayon")]
    fn iter_coordinate_par(&self) -> impl Iterator<Item=(&NeuronVoxelCoordinate<CoordQuant>, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        todo!()
    }
}