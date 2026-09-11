/// 3D voxel coordinate within a cortical area (x, y, z).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NeuronVoxelCoordinate {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl NeuronVoxelCoordinate {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }
}

/// A single neuron voxel storing spatial coordinates and activation potential in XYZP format.
#[derive(Clone, Debug, PartialEq)]
pub struct NeuronVoxelXYZP {
    pub neuron_voxel_coordinate: NeuronVoxelCoordinate,
    pub potential: f32,
}

impl NeuronVoxelXYZP {
    /// Number of bytes used to represent a single neuron voxel (x, y, z, p).
    pub const NUMBER_BYTES_PER_NEURON: usize = (size_of::<u32>() * 3) + size_of::<f32>();

    pub fn new(x: u32, y: u32, z: u32, potential: f32) -> Self {
        NeuronVoxelXYZP {
            neuron_voxel_coordinate: NeuronVoxelCoordinate::new(x, y, z),
            potential,
        }
    }

    pub fn as_tuple(&self) -> (u32, u32, u32, f32) {
        (
            self.neuron_voxel_coordinate.x,
            self.neuron_voxel_coordinate.y,
            self.neuron_voxel_coordinate.z,
            self.potential,
        )
    }
}

impl std::fmt::Display for NeuronVoxelXYZP {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "NeuronVoxelXYZP({}, {}, {}, {})",
            self.neuron_voxel_coordinate.x,
            self.neuron_voxel_coordinate.y,
            self.neuron_voxel_coordinate.z,
            self.potential
        )
    }
}
