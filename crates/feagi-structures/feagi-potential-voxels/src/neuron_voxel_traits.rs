use feagi_data::shared_quantization_sets::{CorticalAreaModelQuantization};
use crate::descriptors::{NeuronVoxelCoordinateGeneric, NeuronVoxelLinearIndexGeneric, NeuronVoxelPotentialGeneric};

pub trait NeuronVoxelImmutableRefs<CANQ: CorticalAreaModelQuantization>
{
    fn get_potential_ref(&self) -> &NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>;
    fn get_linear_index(&self) -> &NeuronVoxelLinearIndexGeneric<CANQ::NeuronIndexCountQuant>;
    fn get_coordinate(&self) -> &NeuronVoxelCoordinateGeneric<CANQ::NeuronIndexCountQuant>;
}

pub trait NeuronVoxelMutableRefs<CANQ: CorticalAreaModelQuantization>:
NeuronVoxelImmutableRefs<CANQ>
{
    fn get_potential_ref_mut(&self) -> &mut NeuronVoxelPotentialGeneric<CANQ::NeuronPotentialQuant>;
}
