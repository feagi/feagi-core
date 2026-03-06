use crate::neuron::descriptors::NeuronVoxelCoordinate;
use crate::neuron::NeuralPotentialValue;

/// A single neuron voxel storing spatial coordinates and activation potential in XYZP format.
///
/// Represents a voxel containing neural activity at a specific 3D location within
/// a cortical area, along with its current activation/voltage level.
#[derive(Clone, Debug, PartialEq)]
pub struct NeuronVoxelXYZP<Potential> where Potential: NeuralPotentialValue {
    /// coordinate within the cortical area.
    pub neuron_voxel_coordinate: NeuronVoxelCoordinate,
    /// potential (voltage) of the voxel
    pub potential: Potential, // TODO: We may decided to have multiple ways to hold potential
}

impl<Potential: NeuralPotentialValue> NeuronVoxelXYZP<Potential> {

    /// Number of bytes used to represent a single neuron voxel in memory (x, y, z, p elements).
    pub const NUMBER_OF_BYTES: usize = NeuronVoxelCoordinate::NUMBER_OF_BYTES + Potential::NUMBER_OF_BYTES;

    pub fn new(x: u32, y: u32, z: u32, potential: Potential) -> Self {
        NeuronVoxelXYZP {
            neuron_voxel_coordinate: NeuronVoxelCoordinate::new(x, y, z),
            potential,
        }
    }
}

#[cfg(feature = "alloc")]
impl<Potential: NeuralPotentialValue> std::fmt::Display for NeuronVoxelXYZP<Potential> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = format!(
            "NeuronVoxelXYZP({}, {}, {}, {})",
            self.neuron_voxel_coordinate.x,
            self.neuron_voxel_coordinate.y,
            self.neuron_voxel_coordinate.z,
            self.potential
        );
        write!(f, "{}", s)
    }
}
