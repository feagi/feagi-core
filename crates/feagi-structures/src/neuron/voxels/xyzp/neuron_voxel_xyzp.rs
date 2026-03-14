use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::neuron::descriptors::{NeuralPotentialValue, NeuronVoxelCoordinate};

/// A single neuron voxel storing spatial coordinates and activation potential in XYZP format.
///
/// Represents a voxel containing neural activity at a specific 3D location within
/// a cortical area, along with its current activation/voltage level.
#[derive(Clone, Debug, PartialEq)]
pub struct NeuronVoxelXYZP<Potential, CoordQuant> where
    Potential: NeuralPotentialValue,
    CoordQuant: QuantizableUInt
{

    /// coordinate within the cortical area.
    pub neuron_voxel_coordinate: NeuronVoxelCoordinate<CoordQuant>,
    /// potential (voltage) of the voxel
    pub potential: Potential, // TODO: We may decided to have multiple ways to hold potential
}

impl<Potential: NeuralPotentialValue, CoordQuant: QuantizableUInt> NeuronVoxelXYZP<Potential, CoordQuant> {

    /// Number of bytes used to represent a single neuron voxel in memory (x, y, z, p elements).
    pub const NUMBER_OF_BYTES: usize = NeuronVoxelCoordinate::NUMBER_OF_BYTES + Potential::NUMBER_OF_BYTES;

    pub fn new(x: CoordQuant, y: CoordQuant, z: CoordQuant, potential: Potential) -> Self {
        NeuronVoxelXYZP {
            neuron_voxel_coordinate: NeuronVoxelCoordinate::new(x, y, z),
            potential,
        }
    }
}

#[cfg(feature = "alloc")]
impl<Potential: NeuralPotentialValue, CoordQuant: QuantizableUInt> std::fmt::Display for NeuronVoxelXYZP<Potential, CoordQuant> {
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
