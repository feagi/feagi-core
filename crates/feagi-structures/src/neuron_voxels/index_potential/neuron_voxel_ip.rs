use crate::quantization::{QuantizableUIntType, QuantizableValueType};
use crate::neuron_voxels::descriptors::NeuronVoxelPotential;
use crate::neuron_voxels::traits::NeuronVoxel;

#[derive(Clone, Debug, PartialEq)]
pub struct NeuronVoxelIP<VoxelPotentialQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    pub index: NeuronVoxelIndexQuant,
    /// potential (voltage) of the voxel
    pub potential: NeuronVoxelPotential<VoxelPotentialQuant>,
}

impl<VoxelPotentialQuant, NeuronVoxelIndexQuant>  NeuronVoxelIP<VoxelPotentialQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    pub fn new(i: NeuronVoxelIndexQuant, potential: NeuronVoxelPotential<VoxelPotentialQuant>) -> Self {
        Self {
            index: i,
            potential,
        }
    }
}

impl<VoxelPotentialQuant, NeuronVoxelIndexQuant> NeuronVoxel<VoxelPotentialQuant> for NeuronVoxelIP<VoxelPotentialQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    const NUMBER_OF_BYTES: usize = NeuronVoxelIndexQuant::NUMBER_OF_BYTES + VoxelPotentialQuant::NUMBER_OF_BYTES;

    fn get_voxel_potential(&self) -> NeuronVoxelPotential<VoxelPotentialQuant> {
        self.potential
    }

    fn get_voxel_potential_ref(&self) -> &NeuronVoxelPotential<VoxelPotentialQuant> {
        &self.potential
    }

    fn set_voxel_potential_ref_mut(&mut self) -> &mut NeuronVoxelPotential<VoxelPotentialQuant> {
        &mut self.potential
    }

    fn set_voxel_potential(&mut self, potential: NeuronVoxelPotential<VoxelPotentialQuant>) {
        self.potential = potential;
    }
}

#[cfg(feature = "alloc")]
impl<VoxelPotentialQuant, NeuronVoxelIndexQuant> std::fmt::Display for NeuronVoxelIP<VoxelPotentialQuant, NeuronVoxelIndexQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    NeuronVoxelIndexQuant: QuantizableUIntType
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = format!(
            "NeuronVoxelIP({}, {})",
            self.index,
            self.potential
        );
        write!(f, "{}", s)
    }
}