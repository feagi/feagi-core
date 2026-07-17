use crate::collections::linear::contiguous_data::QuantizedContiguousVector;
use crate::collections::spatial::contiguous_data::SpatialContiguousVector3D;
use crate::neuron_voxels::wrapped_values::{NeuronVoxelDimensions, NeuronVoxelPotential};
use crate::feagi_quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use crate::feagi_quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::values::spatial::quantizable_index::QuantizedIndexDimension3D;

// TODO wrapper generator?
// TODO proper wrapper, dont just pub the internals

pub struct ContiguousVoxelVector<FIQ: FeagiIndexQuantization, CPQ: CorticalPotentialQuantization>(
    pub SpatialContiguousVector3D<FIQ::NeuronIndexCountQuant, CPQ::MembranePotentialQuant>
);

impl<FIQ: FeagiIndexQuantization, CPQ: CorticalPotentialQuantization> ContiguousVoxelVector<FIQ, CPQ> {

    pub fn new_uniform(dimensions: NeuronVoxelDimensions<FIQ::NeuronIndexCountQuant>, initial_value: NeuronVoxelPotential<CPQ::MembranePotentialQuant>) -> Self {
        let vector_3d = SpatialContiguousVector3D::new_uniform(dimensions.temp_to_ref(), *initial_value.as_ref());
        Self(vector_3d)
    }
}


/*
pub struct NeuronVoxelCollectionContiguousVectorGeneric<FIQ: FeagiIndexQuantization, CPQ: CorticalPotentialQuantization>(
    QuantizableSpatialCollection3DVectorDense<FIQ::NeuronIndexCountQuant, NeuronVoxelPotential<CPQ::MembranePotentialQuant>>,
);

impl<FIQ: FeagiIndexQuantization, CPQ: CorticalPotentialQuantization> NeuronVoxelCollectionContiguousVectorGeneric<FIQ, CPQ> {
    pub fn new(dimensions: NeuronVoxelDimensionsGeneric<FIQ::NeuronIndexCountQuant>) -> Self {
        Self(QuantizableSpatialCollection3DVectorDense::new_uniform(
            dimensions.unwrap(),
            NeuronVoxelPotentialGeneric::QUANT_ZERO,
        ))
    }
}

impl<FIQ: FeagiIndexQuantization, CPQ: CorticalPotentialQuantization> NeuronVoxelCollection<FIQ, CPQ>
    for NeuronVoxelCollectionContiguousVectorGeneric<FIQ, CPQ>
{
    fn get_voxel_dimensions(&self) -> &NeuronVoxelDimensionsGeneric<FIQ::NeuronIndexCountQuant> {
        NeuronVoxelDimensionsGeneric::wrap_ref(self.0.get_dimensions())
    }
}

impl<FIQ: FeagiIndexQuantization, CPQ: CorticalPotentialQuantization> NeuronVoxelCollectionDense<FIQ, CPQ>
    for NeuronVoxelCollectionContiguousVectorGeneric<FIQ, CPQ>
{
}

impl<FIQ: FeagiIndexQuantization, CPQ: CorticalPotentialQuantization> CPUNeuronVoxelCollection<FIQ, CPQ>
    for NeuronVoxelCollectionContiguousVectorGeneric<FIQ, CPQ>
{
    fn try_get_potential_by_voxel_index(
        &self,
        voxel_index: NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>,
    ) -> Option<&NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>> {
        self.0.try_get_value(voxel_index.unwrap())
    }

    fn try_get_potential_by_voxel_index_mut(
        &mut self,
        voxel_index: NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>,
    ) -> Option<&mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>> {
        self.0.try_get_value_mut(voxel_index.unwrap())
    }

    fn iter_with_voxel_index(
        &self,
    ) -> impl Iterator<
        Item = (
            NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>,
            &NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>,
        ),
    > {
        self.0.iter_with_index().map(|(i, v)| (NeuronVoxelLinearIndexGeneric::wrap(i), v))
    }

    fn iter_mut_with_voxel_index(
        &mut self,
    ) -> impl Iterator<
        Item = (
            NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>,
            &mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>,
        ),
    > {
        self.0.iter_mut_with_index().map(|(i, v)| (NeuronVoxelLinearIndexGeneric::wrap(i), v))
    }

    fn iter_with_index_and_coordinate(
        &self,
    ) -> impl Iterator<
        Item = (
            NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>,
            NeuronVoxelCoordinateGeneric<FIQ::NeuronIndexCountQuant>,
            &NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>,
        ),
    > {
        self.0
            .iter_with_index_and_coordinate()
            .map(|(i, c, v)| (NeuronVoxelLinearIndexGeneric::wrap(i), NeuronVoxelCoordinateGeneric::wrap(c), v))
    }

    fn iter_mut_with_index_and_coordinate(
        &mut self,
    ) -> impl Iterator<
        Item = (
            NeuronVoxelLinearIndexGeneric<FIQ::NeuronIndexCountQuant>,
            NeuronVoxelCoordinateGeneric<FIQ::NeuronIndexCountQuant>,
            &mut NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>,
        ),
    > {
        self.0
            .iter_mut_with_index_and_coordinate()
            .map(|(i, c, v)| (NeuronVoxelLinearIndexGeneric::wrap(i), NeuronVoxelCoordinateGeneric::wrap(c), v))
    }
}

impl<FIQ: FeagiIndexQuantization, CPQ: CorticalPotentialQuantization> CPUNeuronVoxelCollectionDense<FIQ, CPQ>
    for NeuronVoxelCollectionContiguousVectorGeneric<FIQ, CPQ>
{
    fn get_neuron_voxel_potentials_slice(&self) -> &[NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>] {
        self.0.get_values_slice()
    }

    fn get_neuron_voxel_potentials_slice_mut(&mut self) -> &mut [NeuronVoxelPotentialGeneric<CPQ::NeuronPotentialQuant>] {
        self.0.get_values_slice_mut()
    }
}



 */
