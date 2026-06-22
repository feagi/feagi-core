use crate::data_wrappers::quantizable::wrapper_traits::QuantizedElementWrapperBase;
use crate::data_wrappers::spatial::wrapper_traits::QuantizedSpatialWrapperBase;
use crate::neuron_voxels::collections::shared_traits::{CPUNeuronVoxelCollection, CPUNeuronVoxelCollectionSparse, NeuronVoxelCollection, NeuronVoxelCollectionSparse};
use crate::neuron_voxels::voxel_collection_generic_descriptors::*;
use crate::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use crate::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;

pub struct NeuronVoxelCollectionSparseHashmapGeneric<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>
(
    QuantizableSpatialCollection3DHashmapSparse<
        FGQ::NeuronIndexCountQuant,
        NeuronVoxelPotentialGeneric<
            CPQ::NeuronPotentialQuant
        >
    >
);

impl<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> NeuronVoxelCollectionSparseHashmapGeneric<FGQ, CPQ>
{
    pub fn new(dimensions: NeuronVoxelDimensionsGeneric<FGQ::NeuronIndexCountQuant>) -> Self {
        Self(
            QuantizableSpatialCollection3DHashmapSparse::new(
                dimensions.unwrap()
            )
        )
    }
}

impl<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> NeuronVoxelCollection<FGQ, CPQ> for NeuronVoxelCollectionSparseHashmapGeneric<FGQ, CPQ> {
    fn get_voxel_dimensions(&self) -> &NeuronVoxelDimensionsGeneric<FGQ::NeuronIndexCountQuant> {
        NeuronVoxelDimensionsGeneric::wrap_ref(self.0.get_dimensions())
    }
}

impl<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> NeuronVoxelCollectionSparse<FGQ, CPQ> for NeuronVoxelCollectionSparseHashmapGeneric<FGQ, CPQ> {

}

impl<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> CPUNeuronVoxelCollection<FGQ, CPQ> for NeuronVoxelCollectionSparseHashmapGeneric<FGQ, CPQ> {
    fn try_get_potential_by_voxel_index(&self, voxel_index: NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>) -> Option<&NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>> {
        self.0.try_get_value(voxel_index.unwrap())
    }

    fn try_get_potential_by_voxel_index_mut(&mut self, voxel_index: NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>) -> Option<&mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>> {
        self.0.try_get_value_mut(voxel_index.unwrap())
    }

    fn iter_with_voxel_index(&self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>, &NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>
    {
        self.0
            .iter_with_index().map(|(i, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), v)
        )
    }

    fn iter_mut_with_voxel_index(&mut self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>, &mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>
    {
        self.0
            .iter_mut_with_index().map(|(i, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), v)
        )
    }

    fn iter_with_index_and_coordinate(&self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<FGQ::NeuronIndexCountQuant>, &NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>
    {
        self.0
            .iter_with_index_and_coordinate().map(|(i, c, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), NeuronVoxelCoordinateGeneric::wrap(c),  v)
        )
    }

    fn iter_mut_with_index_and_coordinate(&mut self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<FGQ::NeuronIndexCountQuant>, &mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>
    {
        self.0
            .iter_mut_with_index_and_coordinate().map(|(i, c, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), NeuronVoxelCoordinateGeneric::wrap(c),  v)
        )
    }
}

impl<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> CPUNeuronVoxelCollectionSparse<FGQ, CPQ> for NeuronVoxelCollectionSparseHashmapGeneric<FGQ, CPQ> {
    fn insert_potential_at_voxel_index(
        &mut self,
        voxel_index: NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>,
        potential: NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>>
    {
        self.0.insert_value_at_index(voxel_index.unwrap(), potential)
    }

    fn remove_potential_at_voxel_index(
        &mut self,
        index: NeuronVoxelLinearIndexGeneric<FGQ::NeuronIndexCountQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>>
    {
        self.0.remove_value_at_index(index.unwrap())
    }
}
