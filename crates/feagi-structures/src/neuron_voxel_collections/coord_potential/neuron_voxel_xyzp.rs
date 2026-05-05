use crate::base_feagi_types::quantizable_types::{QuantizableUIntType, QuantizableValueType};
use crate::neuron_voxel_collections::data_values::{NeuronVoxelCoordinate, NeuronVoxelPotential};
use crate::neuron_voxel_collections::traits::NeuronVoxel;

/// A single neuron voxel storing spatial coordinates and activation potential in XYZP format.
///
/// Represents a voxel containing neural activity at a specific 3D location within
/// a cortical area, along with its current activation/voltage level.
#[derive(Clone, Debug, PartialEq)]
pub struct NeuronVoxelXYZP<VoxelPotentialQuant, CoordQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType
{
    /// coordinate within the cortical area.
    pub coordinate: NeuronVoxelCoordinate<CoordQuant>,
    /// potential (voltage) of the voxel
    pub potential: NeuronVoxelPotential<VoxelPotentialQuant>,
}

impl<VoxelPotentialQuant: QuantizableValueType, CoordQuant: QuantizableUIntType> NeuronVoxelXYZP<VoxelPotentialQuant, CoordQuant> {

    pub fn new(x: CoordQuant, y: CoordQuant, z: CoordQuant, potential: NeuronVoxelPotential<VoxelPotentialQuant>) -> Self {
        NeuronVoxelXYZP {
            coordinate: NeuronVoxelCoordinate::new(x, y, z),
            potential,
        }
    }
}

impl<VoxelPotentialQuant, CoordQuant> NeuronVoxel<VoxelPotentialQuant> for NeuronVoxelXYZP<VoxelPotentialQuant, CoordQuant> where
    VoxelPotentialQuant: QuantizableValueType,
    CoordQuant: QuantizableUIntType
{
    const NUMBER_OF_BYTES: usize = NeuronVoxelCoordinate::<CoordQuant>::NUMBER_OF_BYTES + VoxelPotentialQuant::NUMBER_OF_BYTES;

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
impl<PotentialQuant: QuantizableValueType, CoordQuant: QuantizableUIntType> std::fmt::Display for NeuronVoxelXYZP<PotentialQuant, CoordQuant> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = format!(
            "NeuronVoxelXYZP({}, {}, {}, {})",
            self.coordinate.x,
            self.coordinate.y,
            self.coordinate.z,
            self.potential
        );
        write!(f, "{}", s)
    }
}
