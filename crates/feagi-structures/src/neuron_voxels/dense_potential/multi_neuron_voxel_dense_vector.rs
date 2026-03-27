use ahash::AHashMap;
use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::base_quantizable::value::QuantizableValue;
use crate::genomic::cortical_area::CorticalID;
use crate::neuron_descriptors::CorticalAreaIndex;
use crate::neuron_voxels::dense_potential::neuron_voxel_dense_vector::NeuronVoxelDenseVector;
use crate::neuron_voxels::descriptors::SingleCorticalNeuronVoxelCollectionType;
use crate::neuron_voxels::FeagiNeuronVoxelError;
use crate::neuron_voxels::traits::{MultiCorticalNeuronVoxelCollectionAlloc, MultiCorticalNeuronVoxelCollectionBase, MultiCorticalNeuronVoxelCollectionDense, SingleCorticalNeuronVoxelCollectionBase, SingleCorticalNeuronVoxelCollectionDense};

pub struct MultiNeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant> where
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt,
    CorticalAreaIndexQuant: QuantizableUInt
{
    dense_vectors: AHashMap<CorticalID, NeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>>,
    cache_included_types: AHashMap<CorticalID, SingleCorticalNeuronVoxelCollectionType>,
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant> MultiNeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant> where
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt,
    CorticalAreaIndexQuant: QuantizableUInt
{
    pub fn new() -> Self {
        Self {
            dense_vectors: AHashMap::new(),
            cache_included_types: AHashMap::new(),
        }
    }

    pub fn insert(&mut self, id: CorticalID, dense_neuron_vector: NeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>) -> Result<(), FeagiNeuronVoxelError> {
        self.dense_vectors.insert(id, dense_neuron_vector);
        self.cache_included_types.insert(id, SingleCorticalNeuronVoxelCollectionType::DenseVector);
        Ok(())
    }

    pub fn get_neuron_voxel_dense_vector(&self, cortical_id: &CorticalID) -> Result<&NeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>, FeagiNeuronVoxelError> {
        self.dense_vectors.get(cortical_id).ok_or_else(
            Err(FeagiNeuronVoxelError::NoCorticalIDInNeuronCollection{ context: "Given Cortical ID was not found in the dense vector neuron voxel collection!", cortical_id: cortical_id.clone() })
        )
    }

    pub fn get_neuron_voxel_dense_vector_mut(&mut self, cortical_id: &CorticalID) -> Result<&mut NeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>, FeagiNeuronVoxelError> {
        self.dense_vectors.get_mut(cortical_id).ok_or_else(
            Err(FeagiNeuronVoxelError::NoCorticalIDInNeuronCollection{ context: "Given Cortical ID was not found in the dense vector neuron voxel collection!", cortical_id: cortical_id.clone() })
        )
    }
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant>
MultiCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant>
for MultiNeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant> where
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt,
    CorticalAreaIndexQuant: QuantizableUInt
{
    fn get_contained_cortical_collection_type(&self, cortical_id: &CorticalID) -> Result<&SingleCorticalNeuronVoxelCollectionType, FeagiNeuronVoxelError> {
        self.cache_included_types.get(cortical_id).ok_or_else(
            || FeagiNeuronVoxelError::NoCorticalIDInNeuronCollection{ context: "Given Cortical ID was not found in the dense vector neuron voxel collection!", cortical_id: cortical_id.clone() }
        )
    }

    fn get_contained_cortical_area_ids(&self) -> &[CorticalID] {
        let mut cortical_ids: Vec<CorticalID> = Vec::new();
        for pair in self.cache_included_types {
            cortical_ids.push(pair.0)
        };
        cortical_ids.as_slice()
    }

    fn get_base_collection_implementation(&self, cortical_id: &CorticalID) -> Result<&impl SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>, FeagiNeuronVoxelError> {
        let implementation = self.get_neuron_voxel_dense_vector(cortical_id)?;
        Ok(&implementation)
    }

    fn get_base_collection_implementation_mut(&mut self, cortical_id: &CorticalID) -> Result<&mut impl SingleCorticalNeuronVoxelCollectionBase<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>, FeagiNeuronVoxelError> {
        let implementation = self.get_neuron_voxel_dense_vector_mut(cortical_id)?;
        Ok(implementation)
    }
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant>
MultiCorticalNeuronVoxelCollectionDense<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant>
for MultiNeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant> where
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt,
    CorticalAreaIndexQuant: QuantizableUInt
{
    fn get_dense_collection_implementation(&self, cortical_id: &CorticalID) -> Result<&impl SingleCorticalNeuronVoxelCollectionDense<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>, FeagiNeuronVoxelError> {
        let implementation = self.get_neuron_voxel_dense_vector(cortical_id)?;
        Ok(&implementation)
    }

    fn get_dense_collection_implementation_mut(&mut self, cortical_id: &CorticalID) -> Result<&mut impl SingleCorticalNeuronVoxelCollectionDense<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant>, FeagiNeuronVoxelError> {
        let implementation = self.get_neuron_voxel_dense_vector_mut(cortical_id)?;
        Ok(implementation)
    }
}

impl<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant>
MultiCorticalNeuronVoxelCollectionAlloc<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant>
for MultiNeuronVoxelDenseVector<VoxelPotentialQuant, CoordQuant, NeuronVoxelIndexQuant, CorticalAreaIndexQuant> where
    VoxelPotentialQuant: QuantizableValue,
    CoordQuant: QuantizableUInt,
    NeuronVoxelIndexQuant: QuantizableUInt,
    CorticalAreaIndexQuant: QuantizableUInt
{
    fn get_contained_cortical_collection_types(&self) -> &AHashMap<CorticalID, SingleCorticalNeuronVoxelCollectionType> {
        &self.cache_included_types
    }

    fn remove_by_cortical_id(&mut self, cortical_id: &CorticalID) -> Result<(), FeagiNeuronVoxelError> {
        _ = self.get_neuron_voxel_dense_vector(cortical_id)?;
        self.dense_vectors.remove(cortical_id);
        self.cache_included_types.remove(cortical_id);
        Ok(())
    }
}