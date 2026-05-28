use feagi_data::quantizable_collections::dim_3d::spatial_shared_traits::{
    QuantizableSpatialCollection3DBase, QuantizableSpatialCollection3DIterWithCoordinate,
};
use feagi_data::quantizable_collections::dim_3d::QuantizableSpatialCollection3DHashmapSparse;
use feagi_data::quantizable_collections::shared_traits::{
    QuantizableLinearCollectionCPUData, QuantizableLinearCollectionIterWithIndex,
    QuantizableLinearCollectionSyncSparse,
};
use feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_data::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase;
use feagi_data::shared_quantization_sets::{CorticalAreaModelQuantizationBase, CorticalAreasIndexQuantization};
use crate::neuron_voxels::nondense_collections::shared_traits::{CPUNeuronVoxelCollection, CPUNeuronVoxelCollectionSparse, NeuronVoxelCollection, NeuronVoxelCollectionSparse};
use crate::neuron_voxels::voxel_collection_generic_descriptors::*;

pub struct NeuronVoxelCollectionSparseHashmapGeneric<CAIQ: CorticalAreasIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase>
(
    QuantizableSpatialCollection3DHashmapSparse<
        CAIQ::NeuronIndexCountQuant,
        NeuronVoxelPotentialGeneric<
            CANQ::NeuronPotentialQuant
        >
    >
);

impl<CAIQ: CorticalAreasIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase> NeuronVoxelCollectionSparseHashmapGeneric<CAIQ, CANQ>
{
    pub fn new(dimensions: NeuronVoxelDimensionsGeneric<CAIQ::NeuronIndexCountQuant>) -> Self {
        Self(
            QuantizableSpatialCollection3DHashmapSparse::new(
                dimensions.unwrap()
            )
        )
    }
}

impl<CAIQ: CorticalAreasIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase> NeuronVoxelCollection<CAIQ, CANQ> for NeuronVoxelCollectionSparseHashmapGeneric<CAIQ, CANQ> {
    fn get_voxel_dimensions(&self) -> &NeuronVoxelDimensionsGeneric<CAIQ::NeuronIndexCountQuant> {
        NeuronVoxelDimensionsGeneric::wrap_ref(self.0.get_dimensions())
    }
}

impl<CAIQ: CorticalAreasIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase> NeuronVoxelCollectionSparse<CAIQ, CANQ> for NeuronVoxelCollectionSparseHashmapGeneric<CAIQ, CANQ> {

}

impl<CAIQ: CorticalAreasIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase> CPUNeuronVoxelCollection<CAIQ, CANQ> for NeuronVoxelCollectionSparseHashmapGeneric<CAIQ, CANQ> {
    fn try_get_potential_by_voxel_index(&self, voxel_index: NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>) -> Option<&NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>> {
        self.0.try_get_value(voxel_index.unwrap())
    }

    fn try_get_potential_by_voxel_index_mut(&mut self, voxel_index: NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>) -> Option<&mut NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>> {
        self.0.try_get_value_mut(voxel_index.unwrap())
    }

    fn iter_with_voxel_index<'a>(&'a self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>, &'a NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)>
    where
        NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>: 'a
    {
        self.0
            .iter_with_index().map(|(i, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), v)
        )
    }

    fn iter_mut_with_voxel_index<'a>(&'a mut self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>, &'a mut NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)>
    where
        NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>: 'a
    {
        self.0
            .iter_mut_with_index().map(|(i, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), v)
        )
    }

    fn iter_with_index_and_coordinate<'a>(&'a self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<CAIQ::NeuronIndexCountQuant>, &'a NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)>
    where
        NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>: 'a
    {
        self.0
            .iter_with_index_and_coordinate().map(|(i, c, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), NeuronVoxelCoordinateGeneric::wrap(c),  v)
        )
    }

    fn iter_mut_with_index_and_coordinate<'a>(&'a mut self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<CAIQ::NeuronIndexCountQuant>, &'a mut NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)>
    where
        NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>: 'a
    {
        self.0
            .iter_mut_with_index_and_coordinate().map(|(i, c, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), NeuronVoxelCoordinateGeneric::wrap(c),  v)
        )
    }
}

impl<CAIQ: CorticalAreasIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase> CPUNeuronVoxelCollectionSparse<CAIQ, CANQ> for NeuronVoxelCollectionSparseHashmapGeneric<CAIQ, CANQ> {
    fn insert_potential_at_voxel_index(
        &mut self,
        voxel_index: NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>,
        potential: NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>>
    {
        self.0.insert_value_at_index(voxel_index.unwrap(), potential)
    }

    fn remove_potential_at_voxel_index(
        &mut self,
        index: NeuronVoxelLinearIndexGeneric<CAIQ::NeuronIndexCountQuant>)
        -> Option<NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>>
    {
        self.0.remove_value_at_index(index.unwrap())
    }
}
