use crate::base_feagi_types::quantizable_types::FeagiBaseSingleElementQuantizationType;
use crate::neuron::neuron_model::dimensional::dimensional_structs::{NeuronVoxelCoordinate, NeuronVoxelDimensions, NeuronVoxelIndexCount, NeuronVoxelPotential};
use crate::quantization_level::CorticalAreaNeuronQuantization;

/// Neuron Voxel iteration item. Since voxels will only show neuron potential,
/// we will only include that as a value
pub struct VoxelIterItem<'a, CANQ: CorticalAreaNeuronQuantization>
{
    pub voxel_index: NeuronVoxelIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
    pub voxel_potential: NeuronVoxelPotential<CANQ::NeuronValueQuant>,
    dimensions: &'a NeuronVoxelDimensions<CANQ::NeuronIndexVoxelCountQuant>,
}

impl<'a, CANQ: CorticalAreaNeuronQuantization> VoxelIterItem<'a, CANQ> {

    pub fn get_voxel_coordinate(&self) -> NeuronVoxelCoordinate<CANQ::NeuronIndexVoxelCountQuant> {
        self.dimensions.linear_index_to_standard_voxel_coordinate(self.voxel_index)
    }

}

pub enum VoxelSmartIterator<'a, CANQ, SDI, MDI>
where
    CANQ: CorticalAreaNeuronQuantization,
    //SDI: SingleDensityIterator,
    //MDI: MultipleDensityIterator,
{
    Single(SDI),
    Multiple(MDI),
}

// TODO is this ok, checking every iteration?
impl<'a, CANQ, SDI, MDI> Iterator for VoxelSmartIterator<'a, CANQ, SDI, MDI>
where
    CANQ: CorticalAreaNeuronQuantization,
    //SDI: SingleDensityIterator<Item = EDMN>,
    //MDI: MultipleDensityIterator<Item = EDMN>,
{
    type Item = VoxelIterItem<'a, CANQ>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            VoxelSmartIterator::Single(iter) => iter.next(),
            VoxelSmartIterator::Multiple(iter) => iter.next(),
        }
    }
}

impl<'a, CANQ, SDI, MDI> VoxelSmartIterator<'a, CANQ, SDI, MDI>
where
    CANQ: CorticalAreaNeuronQuantization,
    //SDI: SingleDensityIterator<Item = DMN>,
    //MDI: MultipleDensityIterator<Item = DMN>,
{
    pub fn iter_skip_zero_potential(self) -> impl Iterator<Item = VoxelIterItem<'a, CANQ>>
    {
        self.filter(|e| e.voxel_potential != NeuronVoxelPotential::ZERO)
    }
}

// TODO RAYON