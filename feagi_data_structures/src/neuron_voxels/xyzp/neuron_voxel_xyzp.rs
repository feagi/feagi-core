use crate::genomic::cortical_area::descriptors::NeuronVoxelCoordinate;

/// A single neuron voxel storing spatial coordinates and activation potential in XYZP format.
/// 
/// Represents a voxel containing neural activity at a specific 3D location within
/// a cortical area, along with its current activation/voltage level.
#[derive(Clone, Debug, PartialEq)]
pub struct NeuronVoxelXYZP {
    /// coordinate within the cortical area.
    pub neuron_voxel_coordinate: NeuronVoxelCoordinate,
    /// potential (voltage) of the voxel
    pub potential: f32
}

impl NeuronVoxelXYZP {

    /// Number of bytes used to represent a single neuron voxel in memory (x, y, z, p elements).
    pub const NUMBER_BYTES_PER_NEURON: usize = (size_of::<u32>() *  3) + size_of::<f32>(); // 16 bytes per voxel
    
    /// Creates a new neuron voxel with the specified coordinates and potential.
    ///
    /// # Arguments
    ///
    /// * `x` - X-coordinate within the cortical area
    /// * `y` - Y-coordinate within the cortical area  
    /// * `z` - Z-coordinate within the cortical area
    /// * `p` - Neuron potential/activation value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::NeuronVoxelXYZP;
    ///
    /// // Create a neuron at the origin with no activation
    /// let inactive_neuron = NeuronVoxelXYZP::new(0, 0, 0, 0.0);
    ///
    /// // Create an active neuron at a specific location
    /// let active_neuron = NeuronVoxelXYZP::new(100, 200, 50, 0.85);
    /// ```
    pub fn new(x: u32, y: u32, z: u32, potential: f32) -> Self {
        NeuronVoxelXYZP {
            neuron_voxel_coordinate: NeuronVoxelCoordinate::new(x, y, z),
            potential
        }
    }
    
    /// Returns the neuron voxel's coordinates and potential as a tuple.
    ///
    /// This method provides a convenient way to destructure the neuron's
    /// data for pattern matching or function arguments that expect tuples.
    ///
    /// # Returns
    ///
    /// A tuple `(x, y, z, p)` containing the neuron's coordinates and potential.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::NeuronVoxelXYZP;
    ///
    /// let neuron = NeuronVoxelXYZP::new(10, 20, 30, 0.5);
    /// let (x, y, z, potential) = neuron.as_tuple();
    ///
    /// assert_eq!(x, 10);
    /// assert_eq!(y, 20);
    /// assert_eq!(z, 30);
    /// assert_eq!(potential, 0.5);
    ///
    /// // Useful for pattern matching
    /// match neuron.as_tuple() {
    ///     (_, _, _, p) if p > 0.8 => println!("Highly active neuron"),
    ///     (_, _, _, p) if p > 0.3 => println!("Moderately active neuron"),
    ///     _ => println!("Low activity neuron"),
    /// }
    /// ```
    pub fn as_tuple(&self) -> (u32, u32, u32, f32) {
        (self.neuron_voxel_coordinate.x, self.neuron_voxel_coordinate.y, self.neuron_voxel_coordinate.z, self.potential)
    }
}

impl std::fmt::Display for NeuronVoxelXYZP {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = format!("NeuronVoxelXYZP({}, {}, {}, {})", self.neuron_voxel_coordinate.x, self.neuron_voxel_coordinate.y, self.neuron_voxel_coordinate.z, self.potential);
        write!(f, "{}", s)
    }
}