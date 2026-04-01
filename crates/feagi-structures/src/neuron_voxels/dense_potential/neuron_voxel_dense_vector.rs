use core::marker::PhantomData;
use crate::base_quantizable::QuantizableUIntType;
use crate::base_quantizable::QuantizableValueType;
use crate::neuron_voxels::descriptors::{NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelPotential, SingleCorticalNeuronVoxelCollectionType};
use crate::neuron_voxels::traits::{SingleCorticalNeuronVoxelCollectionAlloc, SingleCorticalNeuronVoxelCollectionBase, SingleCorticalNeuronVoxelCollectionDense, SingleCorticalNeuronVoxelCollectionSparse};

pub struct NeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
{
    cortical_dimensions: NeuronVoxelDimensions<CoordQuant>,
    potentials: Vec<NeuronVoxelPotential<VoxelPotentialQuant>>,
    _index_quant: PhantomData<NeuronVoxelIndexQuant>,
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant> NeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType,
{

    pub fn new(cortical_dimensions: NeuronVoxelDimensions<CoordQuant>) -> Self {
        let number_neurons = cortical_dimensions.get_max_allowed_index_exclusive();
        Self {
            cortical_dimensions,
            potentials: vec!(NeuronVoxelPotential::ZERO; number_neurons),
            _index_quant: PhantomData,
        }
    }
}


impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    const COLLECTION_TYPE: SingleCorticalNeuronVoxelCollectionType = SingleCorticalNeuronVoxelCollectionType::DenseVector;

    fn get_representing_cortical_area_dimensions(&self) -> &NeuronVoxelDimensions<CoordQuant> {
        &self.cortical_dimensions
    }

    fn neuron_index_max_limit(&self) -> NeuronVoxelIndexQuant {
        NeuronVoxelIndexQuant::from_usize(self.potentials.len())
    }
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionAlloc<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
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
    }

    fn empty_and_change_cortical_area_dimensions(&mut self, new_dimensions: NeuronVoxelDimensions<CoordQuant>) {
        let number_neurons = new_dimensions.get_max_allowed_index_exclusive();
        self.potentials.clear();
        self.cortical_dimensions = new_dimensions;
    }

    fn shrink_to_fit(&mut self) {
        // Does nothing, we can never shrink to fit as this is always dense
        return;
    }
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
SingleCorticalNeuronVoxelCollectionDense<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
for NeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>
where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    fn get_all_neuron_voxel_potentials(&self) -> &[NeuronVoxelPotential<VoxelPotentialQuant>] {
        self.potentials.as_slice()
    }

    fn get_all_neuron_voxel_potentials_mut(&mut self) -> &mut [NeuronVoxelPotential<VoxelPotentialQuant>] {
        self.potentials.as_mut_slice()
    }

    fn iter_nonzero_index(&self) -> impl Iterator<Item=(NeuronVoxelIndexQuant, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        self.potentials
            .iter()
            .enumerate()
            .filter(|(_, p)| **p != NeuronVoxelPotential::ZERO)
            .map(|(i, p)| (NeuronVoxelIndexQuant::from_usize(i), *p))
    }

    fn iter_nonzero_coordinate(&self) -> impl Iterator<Item=(NeuronVoxelCoordinate<CoordQuant>, NeuronVoxelPotential<VoxelPotentialQuant>)> {
        let dims = &self.cortical_dimensions;
        self.iter_nonzero_index()
            .map(move |(idx, p)| (dims.linear_index_to_coordinate(idx), p))
    }

    fn zero_all_neuron_voxel_potentials(&mut self) {
        self.potentials.fill(NeuronVoxelPotential::ZERO);
    }

    fn inplace_overwrite_data_from_sparse(&mut self, sparse_neurons: &impl SingleCorticalNeuronVoxelCollectionSparse<NeuronVoxelPotential<VoxelPotentialQuant>, CoordQuant, NeuronVoxelIndexQuant>) {
        todo!()
    }
}