use ahash::AHashMap;
use crate::genomic::cortical_area::CorticalID;
use crate::neuron_collections::neuron_voxel_collections::FeagiStructuresNeuronVoxelError;
use crate::neuron_collections::neuron_voxel_collections::traits::{NeuronVoxelCollectionBase, SingleCorticalNeuronVoxelCollectionDense};
use crate::quantization_level::CorticalAreaNeuronQuantization;

//region MultiCACollection
pub trait MultiCorticalNeuronVoxelCollectionBase<CANQ: CorticalAreaNeuronQuantization>
{
    fn get_contained_cortical_collection_type(&self, cortical_id: &CorticalID) -> Result<&SingleCorticalNeuronVoxelCollectionType, FeagiStructuresNeuronVoxelError>;

    fn get_contained_cortical_area_ids(&self) -> &[CorticalID];

    /// Only gets the base implementation, you probably should NOT use this as it doesn't allow
    /// access to more specialized performant functions
    fn get_base_collection_implementation(&self, cortical_id: &CorticalID) ->
    Result<&impl NeuronVoxelCollectionBase<CANQ>, FeagiStructuresNeuronVoxelError>;

    fn get_base_collection_implementation_mut(&mut self, cortical_id: &CorticalID) ->
    Result<&mut impl NeuronVoxelCollectionBase<CANQ>, FeagiStructuresNeuronVoxelError>;

}

pub trait MultiCorticalNeuronVoxelCollectionDense<CANQ: CorticalAreaNeuronQuantization>:
MultiCorticalNeuronVoxelCollectionBase<CANQ>
{
    fn get_dense_collection_implementation(&self, cortical_id: &CorticalID) -> Result<&impl SingleCorticalNeuronVoxelCollectionDense<CANQ>, FeagiStructuresNeuronVoxelError>;

    fn get_dense_collection_implementation_mut(&mut self, cortical_id: &CorticalID) -> Result<&mut impl SingleCorticalNeuronVoxelCollectionDense<CANQ>, FeagiStructuresNeuronVoxelError>;
}

#[cfg(feature = "alloc")]
pub trait MultiCorticalNeuronVoxelCollectionAlloc<CANQ: CorticalAreaNeuronQuantization>:
MultiCorticalNeuronVoxelCollectionBase<CANQ>
{
    // NOTE: Not practical to do any sort of data retrieval functions here, but we can do housekeeping

    fn get_contained_cortical_collection_types(&self) -> &AHashMap<CorticalID, SingleCorticalNeuronVoxelCollectionType>;

    // NOTE: Adding must be handled by specific implementations

    fn remove_by_cortical_id(&mut self, cortical_id: &CorticalID) -> Result<(), FeagiStructuresNeuronVoxelError>;

}

//endregion


// NOTE: The mixed type is also alone so it doesn't need a trait either