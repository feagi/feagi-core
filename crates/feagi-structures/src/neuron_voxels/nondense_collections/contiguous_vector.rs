use feagi_data::quantizable_collections::dim_3d::QuantizableSpatialCollection3DVectorDense;
use feagi_data::quantizable_collections::dim_3d::spatial_shared_traits::{QuantizableSpatialCollection3DBase, QuantizableSpatialCollection3DIterWithCoordinate};
use feagi_data::quantizable_collections::shared_traits::{QuantizableLinearCollectionAsSlice, QuantizableLinearCollectionCPUData, QuantizableLinearCollectionCPUIterWithIndex};
use feagi_data::quantizable_linear::base_types::QuantizedElementBase;
use feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_data::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase;
use feagi_data::shared_quantization_sets::{CorticalAreaModelQuantizationBase, FeagiGlobalIndexQuantization};
use crate::neuron_voxels::nondense_collections::shared_traits::{CPUNeuronVoxelCollection, CPUNeuronVoxelCollectionDense, NeuronVoxelCollection, NeuronVoxelCollectionDense};
use crate::neuron_voxels::voxel_collection_generic_descriptors::*;

pub struct NeuronVoxelCollectionContiguousVectorGeneric<CAIQ: FeagiGlobalIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase>
(
    QuantizableSpatialCollection3DVectorDense<
        CAIQ::NeuronIndexCountQuant,
        NeuronVoxelPotentialGeneric<
            CANQ::NeuronPotentialQuant
        >
    >
);

impl<CAIQ: FeagiGlobalIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase> NeuronVoxelCollectionContiguousVectorGeneric<CAIQ, CANQ>
{
    pub fn new(dimensions: NeuronVoxelDimensionsGeneric<CAIQ::NeuronIndexCountQuant>) -> Self {
        Self(
            QuantizableSpatialCollection3DVectorDense::new_uniform(
                dimensions.unwrap(),
                NeuronVoxelPotentialGeneric::QUANT_ZERO
            )
        )
    }
}

impl<CAIQ: FeagiGlobalIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase> NeuronVoxelCollection<CAIQ, CANQ> for NeuronVoxelCollectionContiguousVectorGeneric<CAIQ, CANQ> {
    fn get_voxel_dimensions(&self) -> &NeuronVoxelDimensionsGeneric<CAIQ::NeuronIndexCountQuant> {
        NeuronVoxelDimensionsGeneric::wrap_ref(self.0.get_dimensions())
    }
}

impl<CAIQ: FeagiGlobalIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase> NeuronVoxelCollectionDense<CAIQ, CANQ> for NeuronVoxelCollectionContiguousVectorGeneric<CAIQ, CANQ> {

}

impl<CAIQ: FeagiGlobalIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase> CPUNeuronVoxelCollection<CAIQ, CANQ> for NeuronVoxelCollectionContiguousVectorGeneric<CAIQ, CANQ> {
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

impl<CAIQ: FeagiGlobalIndexQuantization,  CANQ: CorticalAreaModelQuantizationBase> CPUNeuronVoxelCollectionDense<CAIQ, CANQ> for NeuronVoxelCollectionContiguousVectorGeneric<CAIQ, CANQ> {
    fn get_neuron_voxel_potentials_slice(&self) -> &[NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>] {
        self.0.get_values_slice()
    }

    fn get_neuron_voxel_potentials_slice_mut(&mut self) -> &mut [NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>] {
        self.0.get_values_slice_mut()
    }
}