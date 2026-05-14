use crate::neuron::model_specifications::base_specifications::LinearNeuronIndexCount;
use crate::neuron::model_specifications::base_dimensional_specifications::dimensional_neuron_common_structs::{VoxelCoordinate, VoxelIndexCount};
use crate::neuron::model_specifications::base_dimensional_specifications::base_dimensional_neuron_specifications::DimensionalNeuronModelCollectionSharedTrait;
use crate::quantization_level::CorticalAreaNeuronQuantization;





pub struct EnumeratedDimensionalNeuronIteratedItem<'a, CANQ: CorticalAreaNeuronQuantization, BNMC: DimensionalNeuronModelCollectionSharedTrait<CANQ>> {
    voxel_index: VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_collection_ref: &'a BNMC
}


impl<'a, CANQ: CorticalAreaNeuronQuantization, BNMC: DimensionalNeuronModelCollectionSharedTrait<CANQ>> EnumeratedDimensionalNeuronIteratedItem<'a, CANQ, BNMC> {
    pub fn get_voxel_index(&self) -> &VoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.voxel_index
    }

    pub fn get_coordinate(&self) -> VoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant> {
        self.neuron_collection_ref.get_cortical_area_voxel_dimensions()
            .linear_index_to_standard_voxel_coordinate(self.voxel_index) // TODO move voxel transfer to quantize unit, add density aware function
    }

    pub fn neuron_data_ref(&self) -> &'a BNMC::SingleNeuronReference {
        self.neuron_collection_ref.try_get_neuron_data_ref(self.voxel_index)
    }
}







pub struct EnumeratedDimensionalNeuronReference<'a, CANQ: CorticalAreaNeuronQuantization, BNMC: DimensionalNeuronModelCollectionSharedTrait<CANQ>> {
    linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_collection_ref: &'a BNMC
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, BNMC: DimensionalNeuronModelCollectionSharedTrait<CANQ>> EnumeratedDimensionalNeuronReference<'a, CANQ, BNMC> {
    pub fn get_linear_index(&self) -> &LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.linear_neuron_index
    }

    pub fn get_voxel_coordinate(&self)

    pub fn voxel_ref(&self) -> &'a BNMC::SingleNeuronReference {
        self.neuron_collection_ref.try_get_neuron_data_ref(self.linear_neuron_index).unwrap() // Assuming this is correct
    }
}



pub struct EnumeratedDimensionalNeuronReferenceMut<'a, CANQ: CorticalAreaNeuronQuantization, BNMC: DimensionalNeuronModelCollectionSharedTrait<CANQ>> {
    linear_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    neuron_collection_ref: &'a mut BNMC
}

impl<'a, CANQ: CorticalAreaNeuronQuantization, BNMC: DimensionalNeuronModelCollectionSharedTrait<CANQ>> EnumeratedDimensionalNeuronReference<'a, CANQ, BNMC> {
    pub fn get_linear_index(&self) -> &LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant> {
        &self.linear_neuron_index
    }

    pub fn voxel_ref_mut(&mut self) -> &'a BNMC::SingleNeuronReference {
        self.neuron_collection_ref.try_get_neuron_data_ref_mut(self.linear_neuron_index).unwrap() // Assuming this is correct
    }
}