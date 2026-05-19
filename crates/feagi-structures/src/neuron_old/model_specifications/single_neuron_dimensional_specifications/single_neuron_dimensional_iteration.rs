use crate::neuron_old::model_specifications::base_dimensional_specifications::dimensional_neuron_common_structs::{VoxelCoordinate, VoxelIndexCount};
use crate::neuron_old::model_specifications::single_neuron_dimensional_specifications::single_neuron_dimensional_neuron_specifications::SingleNeuronDimensionalNeuronCollectionSharedTrait;
use crate::quantization_level::CorticalAreaNeuronQuantization;


pub struct EnumeratedSingleNeuronDimensionalVoxelIteratedItem<'a, CANQ: CorticalAreaNeuronQuantization, NCT: SingleNeuronDimensionalNeuronCollectionSharedTrait<CANQ>> {
    voxel_index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_collection_ref: &'a NCT
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, NCT: SingleNeuronDimensionalNeuronCollectionSharedTrait<CANQ>> EnumeratedSingleNeuronDimensionalVoxelIteratedItem<'a, CANQ, NCT> {
    pub fn get_voxel_index(&self) -> &VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.voxel_index
    }

    pub fn get_coordinate(&self) -> VoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant> {
        self.neuron_collection_ref.get_cortical_area_voxel_dimensions()
            .linear_index_to_standard_voxel_coordinate(self.voxel_index) // TODO move voxel transfer to quantize unit, add density aware function
    }

    pub fn neuron_data_ref(&self) -> &'a NCT::SingleNeuronReference {
        self.neuron_collection_ref.try_get_neuron_data_ref(self.voxel_index)
    }
}