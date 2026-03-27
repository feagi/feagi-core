use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::base_quantizable::value::QuantizableValue;
use crate::neuron_voxels::descriptors::{NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelPotential};
use crate::neuron_voxels::traits::{SingleCorticalNeuronVoxelCollectionAlloc, SingleCorticalNeuronVoxelCollectionBase, SingleCorticalNeuronVoxelCollectionSparse};

pub struct NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt
{
    cortical_dimensions: NeuronVoxelDimensions<CoordQuant>,
    coord_x: Vec<CoordQuant>,
    coord_y: Vec<CoordQuant>,
    coord_z: Vec<CoordQuant>,
    potentials: Vec<NeuronVoxelPotential<VoxelPotentialQuant>>,
}



impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt
{
    pub fn new(cortical_dimensions: NeuronVoxelDimensions<CoordQuant>, number_neurons_preallocated: NeuronVoxelIndexQuant) -> Self {
        Self {
            cortical_dimensions,
            coord_x: Vec::with_capacity(number_neurons_preallocated),
            coord_y: Vec::with_capacity(number_neurons_preallocated),
            coord_z: Vec::with_capacity(number_neurons_preallocated),
            potentials: Vec::with_capacity(number_neurons_preallocated),
        }
    }
}



impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
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
for NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
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
        self.coord_x.reserve(number_of_neuron_voxels_to_reserve_for as usize);
        self.coord_y.reserve(number_of_neuron_voxels_to_reserve_for as usize);
        self.coord_z.reserve(number_of_neuron_voxels_to_reserve_for as usize);
    }

    fn empty_and_change_cortical_area_dimensions(&mut self, new_dimensions: NeuronVoxelDimensions<CoordQuant>) {
        self.clear_all_neurons();
        self.cortical_dimensions = new_dimensions;
    }

    fn shrink_to_fit(&mut self) {
        self.potentials.shrink_to_fit();
        self.coord_x.shrink_to_fit();
        self.coord_y.shrink_to_fit();
        self.coord_z.shrink_to_fit();
    }
}



impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionSparse<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelCoordVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt
{
    fn clear_all_neurons(&mut self) {
        self.potentials.clear();
        self.coord_x.clear();
        self.coord_y.clear();
        self.coord_z.clear();
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