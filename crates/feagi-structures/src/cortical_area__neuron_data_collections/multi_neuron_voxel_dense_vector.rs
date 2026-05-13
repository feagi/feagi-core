use ahash::AHashMap;
use crate::genomic::cortical_area::CorticalID;
use crate::neuron_collections::neuron_voxel_collections::FeagiStructuresNeuronVoxelError;
use crate::neuron_collections::neuron_voxel_collections::traits::{
    MultiCorticalNeuronVoxelCollectionAlloc, MultiCorticalNeuronVoxelCollectionBase,
    MultiCorticalNeuronVoxelCollectionDense, NeuronVoxelCollectionBase,
    SingleCorticalNeuronVoxelCollectionDense,
};
use crate::neuron_collections::neuron_voxel_collections::voxel_structs::SingleCorticalNeuronVoxelCollectionType;
use crate::quantization_level::CorticalAreaNeuronQuantization;
use crate::neuron_collections::neuron_voxel_collections::dense_potential::NeuronVoxelDenseVector;

pub struct MultiNeuronVoxelDenseVector<CANQ: CorticalAreaNeuronQuantization>
{
    dense_vectors: AHashMap<CorticalID, NeuronVoxelDenseVector<CANQ>>,
    cache_included_types: AHashMap<CorticalID, SingleCorticalNeuronVoxelCollectionType>,
    cortical_ids: Vec<CorticalID>,
}

impl<CANQ: CorticalAreaNeuronQuantization> MultiNeuronVoxelDenseVector<CANQ>
{
    pub fn new() -> Self {
        Self {
            dense_vectors: AHashMap::new(),
            cache_included_types: AHashMap::new(),
            cortical_ids: Vec::new(),
        }
    }

    pub fn insert(&mut self, id: CorticalID, dense_neuron_vector: NeuronVoxelDenseVector<CANQ>) -> Result<(), FeagiStructuresNeuronVoxelError> {
        if !self.dense_vectors.contains_key(&id) {
            self.cortical_ids.push(id);
        }
        self.dense_vectors.insert(id, dense_neuron_vector);
        self.cache_included_types.insert(id, SingleCorticalNeuronVoxelCollectionType::DenseVector);
        Ok(())
    }

    pub fn get_neuron_voxel_dense_vector(&self, cortical_id: &CorticalID) -> Result<&NeuronVoxelDenseVector<CANQ>, FeagiStructuresNeuronVoxelError> {
        self.dense_vectors.get(cortical_id).ok_or_else(|| {
            FeagiStructuresNeuronVoxelError::NoCorticalIDInNeuronCollection {
                context: "Given Cortical ID was not found in the dense vector neuron voxel collection!",
                cortical_id: *cortical_id,
            }
        })
    }

    pub fn get_neuron_voxel_dense_vector_mut(&mut self, cortical_id: &CorticalID) -> Result<&mut NeuronVoxelDenseVector<CANQ>, FeagiStructuresNeuronVoxelError> {
        self.dense_vectors.get_mut(cortical_id).ok_or_else(|| {
            FeagiStructuresNeuronVoxelError::NoCorticalIDInNeuronCollection {
                context: "Given Cortical ID was not found in the dense vector neuron voxel collection!",
                cortical_id: *cortical_id,
            }
        })
    }
}

impl<CANQ: CorticalAreaNeuronQuantization> MultiCorticalNeuronVoxelCollectionBase<CANQ>
for MultiNeuronVoxelDenseVector<CANQ>
{
    fn get_contained_cortical_collection_type(&self, cortical_id: &CorticalID) -> Result<&SingleCorticalNeuronVoxelCollectionType, FeagiStructuresNeuronVoxelError> {
        self.cache_included_types.get(cortical_id).ok_or_else(|| {
            FeagiStructuresNeuronVoxelError::NoCorticalIDInNeuronCollection {
                context: "Given Cortical ID was not found in the dense vector neuron voxel collection!",
                cortical_id: *cortical_id,
            }
        })
    }

    fn get_contained_cortical_area_ids(&self) -> &[CorticalID] {
        self.cortical_ids.as_slice()
    }

    fn get_base_collection_implementation(&self, cortical_id: &CorticalID) -> Result<&impl NeuronVoxelCollectionBase<CANQ>, FeagiStructuresNeuronVoxelError> {
        let implementation = self.get_neuron_voxel_dense_vector(cortical_id)?;
        Ok(implementation)
    }

    fn get_base_collection_implementation_mut(&mut self, cortical_id: &CorticalID) -> Result<&mut impl NeuronVoxelCollectionBase<CANQ>, FeagiStructuresNeuronVoxelError> {
        let implementation = self.get_neuron_voxel_dense_vector_mut(cortical_id)?;
        Ok(implementation)
    }
}

impl<CANQ: CorticalAreaNeuronQuantization> MultiCorticalNeuronVoxelCollectionDense<CANQ>
for MultiNeuronVoxelDenseVector<CANQ>
{
    fn get_dense_collection_implementation(&self, cortical_id: &CorticalID) -> Result<&impl SingleCorticalNeuronVoxelCollectionDense<CANQ>, FeagiStructuresNeuronVoxelError> {
        let implementation = self.get_neuron_voxel_dense_vector(cortical_id)?;
        Ok(implementation)
    }

    fn get_dense_collection_implementation_mut(&mut self, cortical_id: &CorticalID) -> Result<&mut impl SingleCorticalNeuronVoxelCollectionDense<CANQ>, FeagiStructuresNeuronVoxelError> {
        let implementation = self.get_neuron_voxel_dense_vector_mut(cortical_id)?;
        Ok(implementation)
    }
}

impl<CANQ: CorticalAreaNeuronQuantization> MultiCorticalNeuronVoxelCollectionAlloc<CANQ>
for MultiNeuronVoxelDenseVector<CANQ>
{
    fn get_contained_cortical_collection_types(&self) -> &AHashMap<CorticalID, SingleCorticalNeuronVoxelCollectionType> {
        &self.cache_included_types
    }

    fn remove_by_cortical_id(&mut self, cortical_id: &CorticalID) -> Result<(), FeagiStructuresNeuronVoxelError> {
        _ = self.get_neuron_voxel_dense_vector(cortical_id)?;
        self.dense_vectors.remove(cortical_id);
        self.cache_included_types.remove(cortical_id);
        self.cortical_ids.retain(|id| id != cortical_id);
        Ok(())
    }
}