use crate::genomic::cortical_area::descriptors::NeuronVoxelCoordinate;
use crate::neuron_voxels::xyzp::NeuronVoxelXYZP;
use crate::FeagiDataError;
use ndarray::Array1;
use rayon::prelude::*;
use std::ops::RangeInclusive;

/// Structure-of-arrays storage for neuron voxel data.
///
/// Stores neuron voxel coordinates and potentials in separate parallel arrays.
/// WARNING: Does not check for duplicate neuron coordinates automatically!
#[derive(Clone, Debug, PartialEq)]
pub struct NeuronVoxelXYZPArrays {
    /// X coordinates of neuron voxels (using Cartesian coordinate system)
    x: Vec<u32>, // Remember, FEAGI is cartesian!
    /// Y coordinates of neuron voxels
    y: Vec<u32>,
    /// Channel indices of neuron voxels
    z: Vec<u32>,
    /// Potential/activation values of neuron voxels
    p: Vec<f32>,
}

impl NeuronVoxelXYZPArrays {
    //region Unique Constructors

    /// Creates a new empty NeuronVoxelXYZPArrays instance.
    ///
    /// # Returns
    /// * `Self` - A new empty instance with no allocated capacity
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays;
    ///
    /// let arrays = NeuronVoxelXYZPArrays::new();
    /// assert_eq!(arrays.len(), 0);
    /// assert!(arrays.is_empty());
    /// ```
    pub fn new() -> Self {
        NeuronVoxelXYZPArrays {
            x: Vec::new(),
            y: Vec::new(),
            z: Vec::new(),
            p: Vec::new(),
        }
    }
}

impl Default for NeuronVoxelXYZPArrays {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuronVoxelXYZPArrays {
    /// Creates a new NeuronVoxelXYZPArrays instance from four separate vectors of equal length.
    ///
    /// # Arguments
    /// * `x` - Vector of X coordinates
    /// * `y` - Vector of Y coordinates
    /// * `z` - Vector of Z coordinates (channel indices)
    /// * `p` - Vector of potential/activation values
    ///
    /// # Returns
    /// * `Result<Self, NeuronError>` - A new instance or an error if the vectors have different lengths
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays;
    ///
    /// let x = vec![1, 2, 3];
    /// let y = vec![4, 5, 6];
    /// let z = vec![7, 8, 9];
    /// let p = vec![0.1, 0.2, 0.3];
    ///
    /// let arrays = NeuronVoxelXYZPArrays::new_from_vectors(x, y, z, p).unwrap();
    /// assert_eq!(arrays.len(), 3);
    /// ```
    pub fn new_from_vectors(
        x: Vec<u32>,
        y: Vec<u32>,
        z: Vec<u32>,
        p: Vec<f32>,
    ) -> Result<Self, FeagiDataError> {
        let len = x.len();
        if len != y.len() || len != z.len() || len != p.len() {
            return Err(FeagiDataError::BadParameters(
                "Input vectors must be the same length to generate XYZP neuron data!!".into(),
            ));
        }
        Ok(NeuronVoxelXYZPArrays { x, y, z, p })
    }

    /// Creates a new NeuronVoxelXYZPArrays instance from four ndarray Array1 instances of equal length.
    ///
    /// # Arguments
    /// * `x_nd` - Array1 of X coordinates
    /// * `y_nd` - Array1 of Y coordinates
    /// * `z_nd` - Array1 of Z coordinates (channel indices)
    /// * `p_nd` - Array1 of potential/activation values
    ///
    /// # Returns
    /// * `Result<Self, NeuronError>` - A new instance or an error if the arrays have different lengths
    ///
    /// # Examples
    /// ```
    /// use ndarray::Array1;
    /// use feagi_data_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays;
    ///
    /// let x_nd = Array1::from_vec(vec![1, 2, 3]);
    /// let y_nd = Array1::from_vec(vec![4, 5, 6]);
    /// let z_nd = Array1::from_vec(vec![7, 8, 9]);
    /// let p_nd = Array1::from_vec(vec![0.1, 0.2, 0.3]);
    ///
    /// let arrays = NeuronVoxelXYZPArrays::new_from_ndarrays(x_nd, y_nd, z_nd, p_nd).unwrap();
    /// assert_eq!(arrays.len(), 3);
    /// ```
    pub fn new_from_ndarrays(
        x_nd: Array1<u32>,
        y_nd: Array1<u32>,
        z_nd: Array1<u32>,
        p_nd: Array1<f32>,
    ) -> Result<Self, FeagiDataError> {
        let len = x_nd.len();
        if len != y_nd.len() || len != z_nd.len() || len != p_nd.len() {
            return Err(FeagiDataError::BadParameters(
                "ND Arrays must be the same length to generate XYZP neuron data!".into(),
            ));
        }
        Ok(NeuronVoxelXYZPArrays {
            x: x_nd.to_vec(),
            y: y_nd.to_vec(),
            z: z_nd.to_vec(),
            p: p_nd.to_vec(),
        })
    }

    //endregion

    //region Array-Like Implementations

    /// Creates a new NeuronVoxelXYZPArrays instance with capacity for the specified maximum number of neuron voxels.
    ///
    /// # Arguments
    /// * `number_of_neurons_initial` - The number of neuron voxels to allocate space for
    ///
    /// # Returns
    /// * `Self` - A new instance
    pub fn with_capacity(number_of_neurons_initial: usize) -> Self {
        NeuronVoxelXYZPArrays {
            x: Vec::with_capacity(number_of_neurons_initial),
            y: Vec::with_capacity(number_of_neurons_initial),
            z: Vec::with_capacity(number_of_neurons_initial),
            p: Vec::with_capacity(number_of_neurons_initial),
        }
    }

    /// Returns the current capacity, IE the number of neurons that can be stored in allocated memory.
    ///
    /// # Returns
    /// * `usize` - The maximum number of neuron voxels that can be stored without reallocation
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays;
    ///
    /// let arrays = NeuronVoxelXYZPArrays::with_capacity(100);
    /// assert_eq!(arrays.capacity(), 100);
    /// ```
    pub fn capacity(&self) -> usize {
        self.x.capacity() // all are the same
    }

    /// Returns the number of additional neuron voxels that can be stored without reallocation.
    ///
    /// # Returns
    /// * `usize` - The difference between capacity and current length
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(10);
    /// arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
    /// assert_eq!(arrays.spare_capacity(), 9);
    /// ```
    pub fn spare_capacity(&self) -> usize {
        self.x.capacity() - self.x.len()
    }

    /// Returns the current number of neuron voxels stored in this structure.
    ///
    /// # Returns
    /// * `usize` - The number of neuron voxels currently stored
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(1);
    /// assert_eq!(arrays.len(), 0);
    /// arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
    /// assert_eq!(arrays.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.p.len() // all of these are of equal length
    }

    /// Shrinks the capacity of all internal vectors to match their current length.
    ///
    /// This reduces memory usage by deallocating unused capacity.
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(100);
    /// arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
    /// arrays.shrink_to_fit();
    /// assert_eq!(arrays.capacity(), 1);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.x.shrink_to_fit();
        self.y.shrink_to_fit();
        self.z.shrink_to_fit();
        self.p.shrink_to_fit();
    }

    /// Ensures the vectors have at least the specified total capacity.
    ///
    /// If the current capacity is already sufficient, this function does nothing.
    /// Otherwise, it reserves additional space to reach the target capacity.
    ///
    /// # Arguments
    /// * `number_of_neurons_total` - The minimum total capacity required
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays;
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(10);
    /// arrays.ensure_capacity(50);
    /// assert!(arrays.capacity() >= 50);
    /// ```
    pub fn ensure_capacity(&mut self, number_of_neurons_total: usize) {
        if self.capacity() >= number_of_neurons_total {
            return;
        }
        self.reserve(number_of_neurons_total - self.len());
    }

    /// Reserves capacity for at least the specified number of additional neuron voxels.
    ///
    /// The actual capacity reserved may be greater than requested to optimize
    /// for future insertions. This operation affects all four internal vectors.
    ///
    /// # Arguments
    /// * `additional_neuron_count` - The number of additional neuron voxels to reserve space for
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays;
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::new();
    /// arrays.reserve(100);
    /// assert!(arrays.capacity() >= 100);
    /// ```
    pub fn reserve(&mut self, additional_neuron_count: usize) {
        self.x.reserve(additional_neuron_count);
        self.y.reserve(additional_neuron_count);
        self.z.reserve(additional_neuron_count);
        self.p.reserve(additional_neuron_count);
    }

    /// Adds a single neuron to the end of the arrays.
    ///
    /// # Arguments
    /// * `neuron` - The NeuronVoxelXYZP instance to add
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(1);
    /// let neuron = NeuronVoxelXYZP::new(1, 2, 3, 0.5);
    /// arrays.push(&neuron);
    /// assert_eq!(arrays.len(), 1);
    /// ```
    pub fn push(&mut self, neuron: &NeuronVoxelXYZP) {
        self.x.push(neuron.neuron_voxel_coordinate.x);
        self.y.push(neuron.neuron_voxel_coordinate.y);
        self.z.push(neuron.neuron_voxel_coordinate.z);
        self.p.push(neuron.potential);
    }

    /// Adds a neuron voxel from raw coordinate and potential values.
    ///
    /// # Arguments
    /// * `x` - X-coordinate within the cortical area
    /// * `y` - Y-coordinate within the cortical area
    /// * `z` - Z-coordinate (channel index)
    /// * `p` - Potential/activation value
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::NeuronVoxelXYZPArrays;
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::new();
    /// arrays.push_raw(1, 2, 3, 0.5);
    /// assert_eq!(arrays.len(), 1);
    /// ```
    pub fn push_raw(&mut self, x: u32, y: u32, z: u32, p: f32) {
        self.x.push(x);
        self.y.push(y);
        self.z.push(z);
        self.p.push(p);
    }

    /// Gets a neuron at the specified index.
    ///
    /// # Arguments
    /// * `index` - The index of the neuron to retrieve
    ///
    /// # Returns
    /// * `Result<NeuronVoxelXYZP, FeagiDataError>` - The neuron at the index or an error if out of bounds
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(1);
    /// arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
    /// let neuron = arrays.get(0).unwrap();
    /// assert_eq!(neuron.neuron_voxel_coordinate.x, 1);
    /// ```
    pub fn get(&self, index: usize) -> Result<NeuronVoxelXYZP, FeagiDataError> {
        if index >= self.len() {
            return Err(FeagiDataError::BadParameters(format!(
                "Given index {} is exceeds NeuronVoxelXYZPArray length of {}!",
                index,
                self.len()
            )));
        }
        let x = self.x[index];
        let y = self.y[index];
        let z = self.z[index];
        let potential = self.p[index];
        Ok(NeuronVoxelXYZP {
            neuron_voxel_coordinate: NeuronVoxelCoordinate::new(x, y, z),
            potential,
        })
    }

    /// Gets the X component of the neuron at the specified index
    pub fn get_x(&self, index: usize) -> Result<u32, FeagiDataError> {
        if index >= self.len() {
            return Err(FeagiDataError::BadParameters(format!(
                "Given index {} is exceeds NeuronVoxelXYZPArray length of {}!",
                index,
                self.len()
            )));
        }
        Ok(self.x[index])
    }

    /// Gets the Y component of the neuron at the specified index
    pub fn get_y(&self, index: usize) -> Result<u32, FeagiDataError> {
        if index >= self.len() {
            return Err(FeagiDataError::BadParameters(format!(
                "Given index {} is exceeds NeuronVoxelXYZPArray length of {}!",
                index,
                self.len()
            )));
        }
        Ok(self.y[index])
    }

    /// Gets the Z component of the neuron at the specified index
    pub fn get_z(&self, index: usize) -> Result<u32, FeagiDataError> {
        if index >= self.len() {
            return Err(FeagiDataError::BadParameters(format!(
                "Given index {} is exceeds NeuronVoxelXYZPArray length of {}!",
                index,
                self.len()
            )));
        }
        Ok(self.z[index])
    }

    /// Gets the P component of the neuron at the specified index
    pub fn get_p(&self, index: usize) -> Result<f32, FeagiDataError> {
        if index >= self.len() {
            return Err(FeagiDataError::BadParameters(format!(
                "Given index {} is exceeds NeuronVoxelXYZPArray length of {}!",
                index,
                self.len()
            )));
        }
        Ok(self.p[index])
    }

    /// Removes and returns the last neuron from the arrays.
    ///
    /// # Returns
    /// * `Option<NeuronVoxelXYZP>` - The last neuron if the arrays are not empty, `None` otherwise
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(1);
    /// arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
    /// let neuron = arrays.pop().unwrap();
    /// assert_eq!(neuron.neuron_voxel_coordinate.x, 1);
    /// assert!(arrays.is_empty());
    /// ```
    pub fn pop(&mut self) -> Option<NeuronVoxelXYZP> {
        let x = self.x.pop();
        let y = self.y.pop();
        let z = self.z.pop();
        let p = self.p.pop();
        x.map(|x| NeuronVoxelXYZP {
            neuron_voxel_coordinate: NeuronVoxelCoordinate::new(x, y.unwrap(), z.unwrap()),
            potential: p.unwrap(),
        })
    }

    /// Clears all vectors by truncating them to zero length without deallocating its memory.
    /// This effectively resets the structure while maintaining capacity.
    pub fn clear(&mut self) {
        self.x.clear();
        self.y.clear();
        self.z.clear();
        self.p.clear();
    }

    /// Checks if no neuron voxels are in this structure.
    ///
    /// # Returns
    /// * `bool` - True if there are no neuron voxels stored, false otherwise
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(1);
    /// assert!(arrays.is_empty());
    ///
    /// arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
    /// assert!(!arrays.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }

    /// Returns an iterator over all neuron voxels in the arrays.
    ///
    /// # Returns
    /// * `impl Iterator<Item=NeuronVoxelXYZP> + '_` - An iterator yielding NeuronVoxelXYZP instances
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(2);
    /// arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
    /// arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.7));
    ///
    /// let mut iter = arrays.iter();
    /// let first = iter.next().unwrap();
    /// assert_eq!(first.neuron_voxel_coordinate.x, 1);
    /// assert_eq!(first.potential, 0.5);
    ///
    /// let second = iter.next().unwrap();
    /// assert_eq!(second.neuron_voxel_coordinate.y, 5);
    /// assert_eq!(second.neuron_voxel_coordinate.z, 6);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = NeuronVoxelXYZP> + '_ {
        self.x
            .iter()
            .zip(&self.y)
            .zip(&self.z)
            .zip(&self.p)
            .map(|(((x, y), z), p)| NeuronVoxelXYZP {
                neuron_voxel_coordinate: NeuronVoxelCoordinate::new(*x, *y, *z),
                potential: *p,
            })
    }

    /// Returns an iterator over all neuron voxels with their indices.
    ///
    /// # Returns
    /// * `impl Iterator<Item=(usize, NeuronVoxelXYZP)> + '_` - An iterator yielding (index, neuron) pairs
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(2);
    /// arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
    /// arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.7));
    ///
    /// for (index, neuron) in arrays.enumerate() {
    ///     println!("Neuron {} at position {}", neuron.neuron_voxel_coordinate.x, index);
    /// }
    /// ```
    pub fn enumerate(&self) -> impl Iterator<Item = (usize, NeuronVoxelXYZP)> + '_ {
        self.x
            .iter()
            .enumerate()
            .zip(&self.y)
            .zip(&self.z)
            .zip(&self.p)
            .map(|(((x, y), z), p)| {
                (
                    x.0,
                    NeuronVoxelXYZP {
                        neuron_voxel_coordinate: NeuronVoxelCoordinate::new(*x.1, *y, *z),
                        potential: *p,
                    },
                )
            })
    }

    //endregion

    /// Updates the internal vectors using an external function before checking for validity.
    /// This allows for custom in-place modifications of the neuron data vectors with automated checking.
    ///
    /// # Arguments
    /// * `vectors_changer` - A function that takes mutable references to the four vectors and updates them
    ///
    /// # Returns
    /// * `Result<(), NeuronError>` - Success or an error if the update fails or results in the
    ///   x y z p vectors being of different lengths by its conclusion
    pub fn update_vectors_from_external<F>(
        &mut self,
        vectors_changer: F,
    ) -> Result<(), FeagiDataError>
    where
        F: FnOnce(
            &mut Vec<u32>,
            &mut Vec<u32>,
            &mut Vec<u32>,
            &mut Vec<f32>,
        ) -> Result<(), FeagiDataError>,
    {
        vectors_changer(&mut self.x, &mut self.y, &mut self.z, &mut self.p)?;
        self.verify_equal_vector_lengths()
    }

    /// Creates a vector of NeuronVoxelXYZP instances from the current arrays.
    ///
    /// # Returns
    /// * `Vec<NeuronVoxelXYZP>` - A vector containing all neuron voxels as individual NeuronVoxelXYZP instances
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(2);
    /// arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
    /// arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.7));
    ///
    /// let neuron_voxels = arrays.copy_as_neuron_xyzp_vec();
    /// assert_eq!(neuron_voxels.len(), 2);
    /// assert_eq!(neuron_voxels[0].neuron_voxel_coordinate.x, 1);
    /// assert_eq!(neuron_voxels[1].potential, 0.7);
    /// ```
    pub fn copy_as_neuron_xyzp_vec(&self) -> Vec<NeuronVoxelXYZP> {
        let mut output: Vec<NeuronVoxelXYZP> = Vec::with_capacity(self.len());
        for i in 0..self.x.len() {
            output.push(NeuronVoxelXYZP::new(
                self.x[i], self.y[i], self.z[i], self.p[i],
            ));
        }
        output
    }

    /// Converts the current arrays into a tuple of ndarray Array1 instances.
    ///
    /// # Returns
    /// * `(Array1<u32>, Array1<u32>, Array1<u32>, Array1<f32>)` - A tuple containing the four arrays
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(2);
    /// arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
    /// arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.7));
    ///
    /// let (x, y, z, p) = arrays.copy_as_tuple_of_nd_arrays();
    /// assert_eq!(x[0], 1);
    /// assert_eq!(y[1], 5);
    /// assert_eq!(z[0], 3);
    /// assert_eq!(p[1], 0.7);
    /// ```
    pub fn copy_as_tuple_of_nd_arrays(
        &self,
    ) -> (Array1<u32>, Array1<u32>, Array1<u32>, Array1<f32>) {
        (
            Array1::from_vec(self.x.clone()),
            Array1::from_vec(self.y.clone()),
            Array1::from_vec(self.z.clone()),
            Array1::from_vec(self.p.clone()),
        )
    }

    /// Returns the total size in bytes required to store all neuron voxels.
    ///
    /// This calculates the number of bytes needed for binary serialization
    /// of the current neuron data.
    ///
    /// # Returns
    /// * `usize` - Total bytes required (number of neuron voxels × 16 bytes per neuron)
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(2);
    /// arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
    /// arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.7));
    /// assert_eq!(arrays.get_size_in_number_of_bytes(), 32); // 2 neuron voxels × 16 bytes
    /// ```
    pub fn get_size_in_number_of_bytes(&self) -> usize {
        self.len() * NeuronVoxelXYZP::NUMBER_BYTES_PER_NEURON
    }

    /// Returns references to the internal vectors.
    ///
    /// # Returns
    /// * `(&Vec<u32>, &Vec<u32>, &Vec<u32>, &Vec<f32>)` - References to the x, y, z, and p vectors
    ///
    /// # Examples
    /// ```
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(1);
    /// arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
    ///
    /// let (x, y, z, p) = arrays.borrow_xyzp_vectors();
    /// assert_eq!(x[0], 1);
    /// assert_eq!(y[0], 2);
    /// assert_eq!(z[0], 3);
    /// assert_eq!(p[0], 0.5);
    /// ```
    pub fn borrow_xyzp_vectors(&self) -> (&Vec<u32>, &Vec<u32>, &Vec<u32>, &Vec<f32>) {
        (&self.x, &self.y, &self.z, &self.p)
    }

    /// Creates a new NeuronVoxelXYZPArrays from filtering neuron voxels based on their locations.
    ///
    /// # Arguments
    /// * `x_range` - Range of valid X coordinates
    /// * `y_range` - Range of valid Y coordinates
    /// * `z_range` - Range of valid Z coordinates
    ///
    /// # Returns
    /// * `Result<NeuronVoxelXYZPArrays, NeuronError>` - A new instance containing only neuron voxels within the specified ranges
    ///
    /// # Examples
    /// ```
    /// use std::ops::RangeInclusive;
    /// use feagi_data_structures::neuron_voxels::xyzp::{NeuronVoxelXYZPArrays, NeuronVoxelXYZP};
    ///
    /// let mut arrays = NeuronVoxelXYZPArrays::with_capacity(3);
    /// arrays.push(&NeuronVoxelXYZP::new(1, 2, 3, 0.5));
    /// arrays.push(&NeuronVoxelXYZP::new(4, 5, 6, 0.7));
    /// arrays.push(&NeuronVoxelXYZP::new(7, 8, 9, 0.9));
    ///
    /// let filtered = arrays.filter_neurons_by_location_bounds(
    ///     RangeInclusive::new(1, 4),
    ///     RangeInclusive::new(2, 5),
    ///     RangeInclusive::new(3, 6)
    /// ).unwrap();
    ///
    /// assert_eq!(filtered.len(), 2);
    /// ```
    pub fn filter_neurons_by_location_bounds(
        &self,
        x_range: RangeInclusive<u32>,
        y_range: RangeInclusive<u32>,
        z_range: RangeInclusive<u32>,
    ) -> Result<NeuronVoxelXYZPArrays, FeagiDataError> {
        let mut xv: Vec<u32> = Vec::new();
        let mut yv: Vec<u32> = Vec::new();
        let mut zv: Vec<u32> = Vec::new();
        let mut pv: Vec<f32> = Vec::new();

        // TODO Could this be optimized at all?
        for (&x, (&y, (&z, &p))) in self
            .x
            .iter()
            .zip(self.y.iter().zip(self.z.iter().zip(self.p.iter())))
        {
            if x_range.contains(&x) && y_range.contains(&y) && z_range.contains(&z) {
                xv.push(x);
                yv.push(y);
                zv.push(z);
                pv.push(p);
            }
        }

        NeuronVoxelXYZPArrays::new_from_vectors(xv, yv, zv, pv)
    }

    /// Validates that all four internal vectors have the same length. This must never fail.
    ///
    /// # Returns
    /// * `Result<(), NeuronError>` - Success or an error if the vectors have different lengths
    fn verify_equal_vector_lengths(&self) -> Result<(), FeagiDataError> {
        let len = self.x.len();
        if !((self.y.len() == len) && (self.x.len() == len) && (self.z.len() == len)) {
            return Err(FeagiDataError::InternalError(
                "Internal XYCP Arrays do not have equal lengths!".into(),
            ));
        }
        Ok(())
    }
}

impl std::fmt::Display for NeuronVoxelXYZPArrays {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = format!(
            "'NeuronVoxelXYZPArrays(X: {:?}, Y: {:?}, Z: {:?}, P: {:?})'",
            self.x, self.y, self.z, self.p
        );
        write!(f, "{}", s)
    }
}

// Implement IntoIterator for owned NeuronVoxelXYZPArrays
impl IntoIterator for NeuronVoxelXYZPArrays {
    type Item = NeuronVoxelXYZP;
    type IntoIter = NeuronVoxelXYZPArraysIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        NeuronVoxelXYZPArraysIntoIter {
            x: self.x.into_iter(),
            y: self.y.into_iter(),
            z: self.z.into_iter(),
            p: self.p.into_iter(),
        }
    }
}

/// Iterator for consuming NeuronVoxelXYZPArrays and producing owned NeuronVoxelXYZP instances.
pub struct NeuronVoxelXYZPArraysIntoIter {
    x: std::vec::IntoIter<u32>,
    y: std::vec::IntoIter<u32>,
    z: std::vec::IntoIter<u32>,
    p: std::vec::IntoIter<f32>,
}

impl Iterator for NeuronVoxelXYZPArraysIntoIter {
    type Item = NeuronVoxelXYZP;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.x.next(), self.y.next(), self.z.next(), self.p.next()) {
            (Some(x), Some(y), Some(z), Some(p)) => Some(NeuronVoxelXYZP {
                neuron_voxel_coordinate: NeuronVoxelCoordinate::new(x, y, z),
                potential: p,
            }),
            _ => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.x.size_hint()
    }
}

impl ExactSizeIterator for NeuronVoxelXYZPArraysIntoIter {
    fn len(&self) -> usize {
        self.x.len()
    }
}

// Implement IntoParallelIterator for owned NeuronVoxelXYZPArrays
impl IntoParallelIterator for NeuronVoxelXYZPArrays {
    type Iter = NeuronVoxelXYZPArraysParIter;
    type Item = NeuronVoxelXYZP;

    fn into_par_iter(self) -> Self::Iter {
        NeuronVoxelXYZPArraysParIter {
            x: self.x,
            y: self.y,
            z: self.z,
            p: self.p,
        }
    }
}

/// Parallel iterator for processing NeuronVoxelXYZPArrays using Rayon.
pub struct NeuronVoxelXYZPArraysParIter {
    x: Vec<u32>,
    y: Vec<u32>,
    z: Vec<u32>,
    p: Vec<f32>,
}

impl ParallelIterator for NeuronVoxelXYZPArraysParIter {
    type Item = NeuronVoxelXYZP;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        // Use rayon's zip to parallelize across the four vectors
        self.x
            .into_par_iter()
            .zip(self.y.into_par_iter())
            .zip(self.z.into_par_iter())
            .zip(self.p.into_par_iter())
            .map(|(((x, y), z), p)| NeuronVoxelXYZP {
                neuron_voxel_coordinate: NeuronVoxelCoordinate::new(x, y, z),
                potential: p,
            })
            .drive_unindexed(consumer)
    }
}

impl IndexedParallelIterator for NeuronVoxelXYZPArraysParIter {
    fn len(&self) -> usize {
        self.x.len()
    }

    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::Consumer<Self::Item>,
    {
        self.x
            .into_par_iter()
            .zip(self.y.into_par_iter())
            .zip(self.z.into_par_iter())
            .zip(self.p.into_par_iter())
            .map(|(((x, y), z), p)| NeuronVoxelXYZP {
                neuron_voxel_coordinate: NeuronVoxelCoordinate::new(x, y, z),
                potential: p,
            })
            .drive(consumer)
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: rayon::iter::plumbing::ProducerCallback<Self::Item>,
    {
        self.x
            .into_par_iter()
            .zip(self.y.into_par_iter())
            .zip(self.z.into_par_iter())
            .zip(self.p.into_par_iter())
            .map(|(((x, y), z), p)| NeuronVoxelXYZP {
                neuron_voxel_coordinate: NeuronVoxelCoordinate::new(x, y, z),
                potential: p,
            })
            .with_producer(callback)
    }
}
