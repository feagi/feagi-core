use feagi_data::quantizable_collections::dim_3d::QuantizableSpatialCollection3DVectorDense;
use feagi_data::quantizable_collections::dim_3d::spatial_shared_traits::{QuantizableSpatialCollection3DBase, QuantizableSpatialCollection3DIterWithCoordinate};
use feagi_data::quantizable_collections::shared_traits::{QuantizableLinearCollectionAsSlice, QuantizableLinearCollectionCPUData, QuantizableLinearCollectionCPUIterWithIndex};
use feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_data::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase;
use feagi_data::shared_quantization_sets::{CorticalPotentialQuantization, FeagiGlobalQuantization};
use crate::neuron_voxels::collections::shared_traits::{CPUNeuronVoxelCollection, CPUNeuronVoxelCollectionDense, NeuronVoxelCollection, NeuronVoxelCollectionDense};
use crate::neuron_voxels::voxel_collection_generic_descriptors::*;

pub struct NeuronVoxelCollectionContiguousVectorGeneric<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization>
(
    QuantizableSpatialCollection3DVectorDense<
        FGQ::NeuronIndexCountQuant,
        NeuronVoxelPotentialGeneric<
            CPQ::NeuronPotentialQuant
        >
    >
);

impl<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> NeuronVoxelCollectionContiguousVectorGeneric<FGQ, CPQ>
{
    pub fn new(dimensions: NeuronVoxelDimensionsGeneric<FGQ::NeuronIndexCountQuant>) -> Self {
        Self(
            QuantizableSpatialCollection3DVectorDense::new_uniform(
                dimensions.unwrap(),
                NeuronVoxelPotentialGeneric::QUANT_ZERO
            )
        )
    }
}

impl<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> NeuronVoxelCollection<FGQ, CPQ> for NeuronVoxelCollectionContiguousVectorGeneric<FGQ, CPQ> {
    fn get_voxel_dimensions(&self) -> &NeuronVoxelDimensionsGeneric<FGQ::NeuronIndexCountQuant> {
        NeuronVoxelDimensionsGeneric::wrap_ref(self.0.get_dimensions())
    }
}

impl<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> NeuronVoxelCollectionDense<FGQ, CPQ> for NeuronVoxelCollectionContiguousVectorGeneric<FGQ, CPQ> {

}

impl<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> CPUNeuronVoxelCollection<FGQ, CPQ> for NeuronVoxelCollectionContiguousVectorGeneric<FGQ, CPQ> {
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

impl<FGQ: FeagiGlobalQuantization,  CPQ: CorticalPotentialQuantization> CPUNeuronVoxelCollectionDense<FGQ, CPQ> for NeuronVoxelCollectionContiguousVectorGeneric<FGQ, CPQ> {
    fn get_neuron_voxel_potentials_slice(&self) -> &[NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>] {
        self.0.get_values_slice()
    }

    fn get_neuron_voxel_potentials_slice_mut(&mut self) -> &mut [NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>] {
        self.0.get_values_slice_mut()
    }
}