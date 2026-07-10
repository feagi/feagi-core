use crate::data_wrappers::quantizable::wrapper_traits::QuantizedElementWrapperBase;
use crate::data_wrappers::spatial::wrapper_traits::QuantizedSpatialWrapperBase;
use crate::neuron_voxels::collections::shared_traits::{CPUNeuronVoxelCollection, CPUNeuronVoxelCollectionSparse, NeuronVoxelCollection, NeuronVoxelCollectionSparse};
use crate::neuron_voxels::voxel_collection_generic_descriptors::*;
use crate::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use crate::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;

pub struct NeuronVoxelCollectionSparseHashmapGeneric<FIQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>
(
    QuantizableSpatialCollection3DHashmapSparse<
        FIQ::NeuronIndexCountQuant,
        NeuronVoxelPotentialGeneric<
            CPQ::NeuronPotentialQuant
        >
    >
);

impl<FIQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> NeuronVoxelCollectionSparseHashmapGeneric<FIQ, CPQ>
{
    pub fn new(dimensions: NeuronVoxelDimensionsGeneric<FIQ::NeuronIndexCountQuant>) -> Self {
        Self(
            QuantizableSpatialCollection3DHashmapSparse::new(
                dimensions.unwrap()
            )
        )
    }
}

impl<FIQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> NeuronVoxelCollection<FIQ, CPQ> for NeuronVoxelCollectionSparseHashmapGeneric<FIQ, CPQ> {
    fn get_voxel_dimensions(&self) -> &NeuronVoxelDimensionsGeneric<FIQ::NeuronIndexCountQuant> {
        NeuronVoxelDimensionsGeneric::wrap_ref(self.0.get_dimensions())
    }
}

impl<FIQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> NeuronVoxelCollectionSparse<FIQ, CPQ> for NeuronVoxelCollectionSparseHashmapGeneric<FIQ, CPQ> {

}

impl<FIQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> CPUNeuronVoxelCollection<FIQ, CPQ> for NeuronVoxelCollectionSparseHashmapGeneric<FIQ, CPQ> {
    fn try_get_potential_by_voxel_index(&self, voxel_index: NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>) -> Option<&NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>> {
        self.0.try_get_value(voxel_index.unwrap())
    }

    fn try_get_potential_by_voxel_index_mut(&mut self, voxel_index: NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>) -> Option<&mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>> {
        self.0.try_get_value_mut(voxel_index.unwrap())
    }

    fn iter_with_voxel_index(&self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>, &NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>
    {
        self.0
            .iter_with_index().map(|(i, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), v)
        )
    }

    fn iter_mut_with_voxel_index(&mut self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>, &mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>
    {
        self.0
            .iter_mut_with_index().map(|(i, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), v)
        )
    }

    fn iter_with_index_and_coordinate(&self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<FIQ::NeuronIndexCountQuant>, &NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>
    {
        self.0
            .iter_with_index_and_coordinate().map(|(i, c, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), NeuronVoxelCoordinateGeneric::wrap(c),  v)
        )
    }

    fn iter_mut_with_index_and_coordinate(&mut self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<FIQ::NeuronIndexCountQuant>, &mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)>
    {
        self.0
            .iter_mut_with_index_and_coordinate().map(|(i, c, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), NeuronVoxelCoordinateGeneric::wrap(c),  v)
        )
    }
}

impl<FIQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> CPUNeuronVoxelCollectionSparse<FIQ, CPQ> for NeuronVoxelCollectionSparseHashmapGeneric<FIQ, CPQ> {
    fn insert_potential_at_voxel_index(
        &mut self,
        voxel_index: NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>,
        potential: NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>>
    {
        self.0.insert_value_at_index(voxel_index.unwrap(), potential)
    }

    fn remove_potential_at_voxel_index(
        &mut self,
        index: NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>>
    {
        self.0.remove_value_at_index(index.unwrap())
    }
}
