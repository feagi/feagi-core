use feagi_data::quantizable_collections::dim_3d::QuantizableSpatialCollection3DVectorDense;
use feagi_data::quantizable_collections::dim_3d::spatial_shared_traits::{QuantizableSpatialCollection3DBase, QuantizableSpatialCollection3DIterWithCoordinate};
use feagi_data::quantizable_collections::shared_traits::{QuantizableLinearCollectionAsSlice, QuantizableLinearCollectionCPUData, QuantizableLinearCollectionIterWithIndex};
use feagi_data::quantizable_linear::base_types::QuantizedElementBase;
use feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_data::quantizable_spatial::wrappers::QuantizedSpatialWrapperBase;
use feagi_data::shared_quantization_sets::CorticalAreaModelQuantization;
use crate::neuron_voxels::nondense_collections::shared_traits::{CPUNeuronVoxelCollection, CPUNeuronVoxelCollectionDense, NeuronVoxelCollection, NeuronVoxelCollectionDense};
use crate::neuron_voxels::voxel_collection_generic_descriptors::*;

pub struct NeuronVoxelCollectionContiguousVectorGeneric<CANQ: CorticalAreaModelQuantization>
(
    QuantizableSpatialCollection3DVectorDense<
        CANQ::NeuronIndexCountQuant,
        NeuronVoxelPotentialGeneric<
            CANQ::NeuronPotentialQuant
        >
    >
);

impl<CANQ: CorticalAreaModelQuantization> NeuronVoxelCollectionContiguousVectorGeneric<CANQ>
{
    pub fn new(dimensions: NeuronVoxelDimensionsGeneric<CANQ::NeuronIndexCountQuant>) -> Self {
        Self(
            QuantizableSpatialCollection3DVectorDense::new_uniform(
                dimensions.unwrap(),
                NeuronVoxelPotentialGeneric::QUANT_ZERO
            )
        )
    }
}

impl<CANQ: CorticalAreaModelQuantization> NeuronVoxelCollection<CANQ> for NeuronVoxelCollectionContiguousVectorGeneric<CANQ> {
    fn get_voxel_dimensions(&self) -> &NeuronVoxelDimensionsGeneric<CANQ::NeuronIndexCountQuant> {
        NeuronVoxelDimensionsGeneric::wrap_ref(self.0.get_dimensions())
    }
}

impl<CANQ: CorticalAreaModelQuantization> NeuronVoxelCollectionDense<CANQ> for NeuronVoxelCollectionContiguousVectorGeneric<CANQ> {

}

impl<CANQ: CorticalAreaModelQuantization> CPUNeuronVoxelCollection<CANQ> for NeuronVoxelCollectionContiguousVectorGeneric<CANQ> {
    fn try_get_potential_by_voxel_index(&self, voxel_index: NeuronVoxelLinearIndexGeneric<CANQ::NeuronIndexCountQuant>) -> Option<&NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>> {
        self.0.try_get_value(voxel_index.unwrap())
    }

    fn try_get_potential_by_voxel_index_mut(&mut self, voxel_index: NeuronVoxelLinearIndexGeneric<CANQ::NeuronIndexCountQuant>) -> Option<&mut NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>> {
        self.0.try_get_value_mut(voxel_index.unwrap())
    }

    fn iter_with_voxel_index<'a>(&'a self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<CANQ::NeuronIndexCountQuant>, &'a NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)>
    where
        NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>: 'a
    {
        self.0
            .iter_with_index().map(|(i, v)|
                (NeuronVoxelLinearIndexGeneric::wrap(i), v)
            )
    }

    fn iter_mut_with_voxel_index<'a>(&'a mut self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<CANQ::NeuronIndexCountQuant>, &'a mut NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)>
    where
        NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>: 'a
    {
        self.0
            .iter_mut_with_index().map(|(i, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), v)
        )
    }

    fn iter_with_index_and_coordinate<'a>(&'a self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<CANQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<CANQ::NeuronIndexCountQuant>, &'a NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)>
    where
        NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>: 'a
    {
        self.0
            .iter_with_index_and_coordinate().map(|(i, c, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), NeuronVoxelCoordinateGeneric::wrap(c),  v)
        )
    }

    fn iter_mut_with_index_and_coordinate<'a>(&'a mut self) -> impl Iterator<Item=(NeuronVoxelLinearIndexGeneric<CANQ::NeuronIndexCountQuant>, NeuronVoxelCoordinateGeneric<CANQ::NeuronIndexCountQuant>, &'a mut NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>)>
    where
        NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>: 'a
    {
        self.0
            .iter_mut_with_index_and_coordinate().map(|(i, c, v)|
            (NeuronVoxelLinearIndexGeneric::wrap(i), NeuronVoxelCoordinateGeneric::wrap(c),  v)
        )
    }
}

impl<CANQ: CorticalAreaModelQuantization> CPUNeuronVoxelCollectionDense<CANQ> for NeuronVoxelCollectionContiguousVectorGeneric<CANQ> {
    fn get_neuron_voxel_potentials_slice(&self) -> &[NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>] {
        self.0.get_values_slice()
    }

    fn get_neuron_voxel_potentials_slice_mut(&mut self) -> &mut [NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>] {
        self.0.get_values_slice_mut()
    }
}