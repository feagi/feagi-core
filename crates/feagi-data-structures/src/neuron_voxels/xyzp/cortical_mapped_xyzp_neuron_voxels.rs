use crate::genomic::cortical_area::CorticalID;
use crate::neuron_voxels::xyzp::NeuronVoxelXYZPArrays;
use std::collections::HashMap;
use std::mem::size_of;

/// Neuron voxel data organized by cortical area.
///
/// Maps cortical area identifiers to their respective neuron voxel collections,
/// allowing efficient storage and retrieval of neural activity across different
/// brain regions.
#[derive(Debug, Clone, PartialEq)]
pub struct CorticalMappedXYZPNeuronVoxels {
    /// Hash map storing neuron collections for each cortical area.
    ///
    /// The key is a unique cortical area identifier, and the value contains
    /// all neuron_voxels belonging to that cortical area.
    pub mappings: HashMap<CorticalID, NeuronVoxelXYZPArrays>,
}

impl CorticalMappedXYZPNeuronVoxels {
    /// Size in bytes of each cortical area header in binary format.
    pub const NUMBER_BYTES_PER_CORTICAL_ID_HEADER: usize =
        CorticalID::NUMBER_OF_BYTES + size_of::<u32>() + size_of::<u32>();
    /// Size in bytes of the cortical count field in binary format.
    pub const NUMBER_BYTES_CORTICAL_COUNT_HEADER: usize = size_of::<u16>();

    /// Creates a new empty neuron data collection.
    ///
    /// This creates a new instance with an empty hash map, suitable for
    /// dynamic addition of cortical areas as needed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let neuron_data = CorticalMappedXYZPNeuronVoxels::new();
    /// assert_eq!(neuron_data.len(), 0);
    /// ```
    pub fn new() -> CorticalMappedXYZPNeuronVoxels {
        CorticalMappedXYZPNeuronVoxels {
            mappings: HashMap::new(),
        }
    }
}

impl Default for CorticalMappedXYZPNeuronVoxels {
    fn default() -> Self {
        Self::new()
    }
}

impl CorticalMappedXYZPNeuronVoxels {
    //region HashMap like implementation
    /// Creates a new neuron data collection with pre-allocated capacity.
    ///
    /// This is more efficient when the approximate number of cortical areas
    /// is known in advance, as it reduces hash map reallocations.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Expected number of cortical areas
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// // Pre-allocate for a brain with 100 cortical areas
    /// let neuron_data = CorticalMappedXYZPNeuronVoxels::new_with_capacity(100);
    /// assert_eq!(neuron_data.len(), 0);
    /// ```
    pub fn new_with_capacity(capacity: usize) -> CorticalMappedXYZPNeuronVoxels {
        CorticalMappedXYZPNeuronVoxels {
            mappings: HashMap::with_capacity(capacity),
        }
    }

    /// Returns the number of cortical areas currently stored.
    ///
    /// # Returns
    ///
    /// The count of cortical areas that have neuron data.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Checks if the neuron data collection is empty.
    ///
    /// # Returns
    ///
    /// `true` if no cortical areas have neuron data, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let neuron_data = CorticalMappedXYZPNeuronVoxels::new();
    /// assert!(neuron_data.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Returns the current capacity of the internal hash map.
    ///
    /// # Returns
    ///
    /// The number of cortical areas that can be stored without reallocation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let neuron_data = CorticalMappedXYZPNeuronVoxels::new_with_capacity(100);
    /// assert!(neuron_data.capacity() >= 100);
    /// ```
    pub fn capacity(&self) -> usize {
        self.mappings.capacity()
    }

    /// Reserves capacity for at least the specified number of additional cortical areas.
    ///
    /// The actual capacity reserved may be greater than requested to optimize
    /// for future insertions.
    ///
    /// # Arguments
    ///
    /// * `additional_capacity` - The number of additional cortical areas to reserve space for
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let mut neuron_data = CorticalMappedXYZPNeuronVoxels::new();
    /// neuron_data.reserve(50);
    /// assert!(neuron_data.capacity() >= 50);
    /// ```
    pub fn reserve(&mut self, additional_capacity: usize) {
        self.mappings.reserve(additional_capacity);
    }

    /// Shrinks the capacity of the hash map to match its current size.
    ///
    /// This reduces memory usage by deallocating unused capacity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let mut neuron_data = CorticalMappedXYZPNeuronVoxels::new_with_capacity(100);
    /// // ... add some data
    /// neuron_data.shrink_to_fit();
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.mappings.shrink_to_fit();
    }

    /// Gets an immutable reference to neuron data for a cortical area.
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Cortical area identifier
    ///
    /// # Returns
    ///
    /// `Some(&NeuronVoxelXYZPArrays)` if the cortical area exists, `None` otherwise.
    pub fn get_neurons_of(&self, cortical_id: &CorticalID) -> Option<&NeuronVoxelXYZPArrays> {
        self.mappings.get(cortical_id)
    }

    /// Gets a mutable reference to neuron data for a cortical area.
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Cortical area identifier
    ///
    /// # Returns
    ///
    /// `Some(&mut NeuronVoxelXYZPArrays)` if the cortical area exists, `None` otherwise.
    pub fn get_neurons_of_mut(
        &mut self,
        cortical_id: &CorticalID,
    ) -> Option<&mut NeuronVoxelXYZPArrays> {
        self.mappings.get_mut(cortical_id)
    }

    /// Checks if a cortical area has neuron data.
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Cortical area identifier to check
    ///
    /// # Returns
    ///
    /// `true` if the cortical area exists, `false` otherwise.
    pub fn contains_cortical_id(&self, cortical_id: &CorticalID) -> bool {
        self.mappings.contains_key(cortical_id)
    }

    /// Inserts neuron data for a cortical area.
    ///
    /// If the cortical area already exists, its data will be replaced, and the old data returned.
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Unique identifier for the cortical area
    /// * `neuron_data` - Collection of neuron_voxels for this cortical area
    ///
    /// # Returns
    ///
    /// `Some(NeuronVoxelXYZPArrays)` of the old data if being overwritten
    /// `None` if nothing is being overwritten
    pub fn insert(
        &mut self,
        cortical_id: CorticalID,
        neuron_data: NeuronVoxelXYZPArrays,
    ) -> Option<NeuronVoxelXYZPArrays> {
        self.mappings.insert(cortical_id, neuron_data)
    }

    /// Removes neuron data for a cortical area.
    ///
    /// # Arguments
    ///
    /// * `cortical_id` - Cortical area identifier to remove
    ///
    /// # Returns
    ///
    /// `Some(NeuronVoxelXYZPArrays)` of the removed data if the cortical area existed,
    /// `None` if the cortical area was not found.
    pub fn remove(&mut self, cortical_id: CorticalID) -> Option<NeuronVoxelXYZPArrays> {
        self.mappings.remove(&cortical_id)
    }

    /// Removes all cortical areas and their neuron data.
    ///
    /// This operation clears the entire collection while maintaining the allocated capacity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let mut neuron_data = CorticalMappedXYZPNeuronVoxels::new();
    /// // ... add some data
    /// neuron_data.clear();
    /// assert!(neuron_data.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.mappings.clear();
    }

    /// Removes all neuron_voxels from the neuron arrays of this CorticalMappedXYZPNeuronData without
    /// clearing allocated capacity on them. Better to use over clear() if you are writing to
    /// similar cortical areas repeatedly.
    pub fn clear_neurons_only(&mut self) {
        for neuron_arrays in self.mappings.values_mut() {
            neuron_arrays.clear();
        }
    }

    /// Returns an iterator over the neuron data collections.
    ///
    /// This iterator yields references to the neuron arrays for each cortical area,
    /// without the cortical IDs.
    ///
    /// # Returns
    ///
    /// An iterator that yields `&NeuronVoxelXYZPArrays` for each cortical area.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let neuron_data = CorticalMappedXYZPNeuronVoxels::new();
    /// for neurons in neuron_data.iter() {
    ///     println!("Cortical area has {} neurons", neurons.len());
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &NeuronVoxelXYZPArrays> + '_ {
        self.mappings.values()
    }

    /// Returns a mutable iterator over the neuron data collections.
    ///
    /// This iterator yields mutable references to the neuron arrays for each cortical area,
    /// allowing modification of the neuron data.
    ///
    /// # Returns
    ///
    /// An iterator that yields `&mut NeuronVoxelXYZPArrays` for each cortical area.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let mut neuron_data = CorticalMappedXYZPNeuronVoxels::new();
    /// for neurons in neuron_data.iter_mut() {
    ///     neurons.clear(); // Clear all neuron arrays
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut NeuronVoxelXYZPArrays> + '_ {
        self.mappings.values_mut()
    }

    /// Returns an iterator over the cortical area identifiers.
    ///
    /// This iterator yields references to the cortical IDs for each area that has neuron data,
    /// without the neuron data itself.
    ///
    /// # Returns
    ///
    /// An iterator that yields `&CorticalID` for each cortical area.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let neuron_data = CorticalMappedXYZPNeuronVoxels::new();
    /// for cortical_id in neuron_data.keys() {
    ///     println!("Found cortical area: {:?}", cortical_id);
    /// }
    /// ```
    pub fn keys(&self) -> impl Iterator<Item = &CorticalID> + '_ {
        self.mappings.keys()
    }

    //endregion

    /// Ensures a cortical area exists and returns a cleared, mutable reference to its neuron voxels.
    ///
    /// If the cortical area exists, clears its neuron voxels and returns a mutable reference.
    /// If it doesn't exist, creates a new empty array for that cortical area.
    ///
    /// # Arguments
    /// * `cortical_id` - The cortical area identifier
    ///
    /// # Returns
    /// A mutable reference to the (cleared) neuron voxel array for this cortical area.
    pub fn ensure_clear_and_borrow_mut(
        &mut self,
        cortical_id: &CorticalID,
    ) -> &mut NeuronVoxelXYZPArrays {
        if self.mappings.contains_key(cortical_id) {
            // If already contains neuron array, clear it and return it
            let neurons = self.mappings.get_mut(cortical_id).unwrap();
            neurons.clear();
            return neurons;
        }
        _ = self
            .mappings
            .insert(*cortical_id, NeuronVoxelXYZPArrays::new());
        self.mappings.get_mut(cortical_id).unwrap()
    }
}
/*
impl FeagiByteStructureCompatible for CorticalMappedXYZPNeuronData {
    fn get_type(&self) -> FeagiByteStructureType { Self::BYTE_STRUCT_TYPE }
    fn get_version(&self) -> u8 { Self::BYTE_STRUCT_VERSION }
    fn overwrite_feagi_byte_structure_slice(&self, slice: &mut [u8]) -> Result<usize, FeagiDataError> {

        if self.mappings.len() == 0 {
            return Err(FeagiDataError::DeserializationError("Cannot generate a bytes structure export with an empty cortical mappings object!".into()).into())
        }

        let num_bytes_needed: usize = self.max_number_bytes_needed();
        if slice.len() < num_bytes_needed {
            return Err(FeagiDataError::SerializationError(format!("Not enough space given to store neuron XYZP data! Need {} bytes but given {}!", num_bytes_needed, slice.len())).into());
        }

        slice[0] = self.get_type() as u8;
        slice[1] = self.get_version();

        let number_cortical_areas: usize = self.mappings.len();
        LittleEndian::write_u16(&mut slice[2..4], number_cortical_areas as u16);

        let mut subheader_write_index: usize = FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + Self::NUMBER_BYTES_CORTICAL_COUNT_HEADER;
        let mut neuron_data_write_index: u32 = subheader_write_index as u32 + (number_cortical_areas as u32 * Self::NUMBER_BYTES_PER_CORTICAL_ID_HEADER as u32);

        for (cortical_id, neuron_data) in &self.mappings {
            // Write cortical subheader
            let write_target = &mut slice[subheader_write_index .. subheader_write_index + CorticalID::CORTICAL_ID_LENGTH];
            let write_target = write_target.try_into().unwrap();
            cortical_id.write_bytes_at(write_target)?;
            let reading_start: u32 = neuron_data_write_index;
            let reading_length: u32 = neuron_data.get_size_in_number_of_bytes() as u32;
            LittleEndian::write_u32(&mut slice[subheader_write_index + 6 .. subheader_write_index + 10], reading_start);
            LittleEndian::write_u32(&mut slice[subheader_write_index + 10 .. subheader_write_index + 14], reading_length);

            // write neuron data
            neuron_data.write_neural_data_to_bytes(&mut slice[reading_start as usize .. (reading_start + reading_length) as usize])?;

            // update indexes
            neuron_data_write_index += reading_length;
            subheader_write_index += Self::NUMBER_BYTES_PER_CORTICAL_ID_HEADER;
        };
        let wasted_space = slice.len() - num_bytes_needed;
        Ok(wasted_space)
    }

    fn max_number_bytes_needed(&self) -> usize {
        let mut number_bytes_needed: usize = FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + Self::NUMBER_BYTES_CORTICAL_COUNT_HEADER;
        for neuron_data in self.iter() {
            number_bytes_needed += Self::NUMBER_BYTES_PER_CORTICAL_ID_HEADER + neuron_data.get_size_in_number_of_bytes();
        }
        number_bytes_needed
    }

    fn new_from_feagi_byte_structure(feagi_byte_structure: &FeagiByteStructure) -> Result<Self, FeagiDataError> {
        FeagiByteStructure::verify_matching_structure_type_and_version(&feagi_byte_structure,
                                                                       Self::BYTE_STRUCT_TYPE,
                                                                       Self::BYTE_STRUCT_VERSION)?;

        let bytes = feagi_byte_structure.borrow_data_as_slice();
        let number_cortical_areas: u16 = LittleEndian::read_u16(&bytes[2..4]);

        let min_array_length_with_cortical_headers: usize = FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES +  Self::NUMBER_BYTES_CORTICAL_COUNT_HEADER +
            (Self::NUMBER_BYTES_PER_CORTICAL_ID_HEADER * number_cortical_areas as usize);

        if bytes.len() < min_array_length_with_cortical_headers {
            return Err(FeagiDataError::SerializationError(format!("Byte structure for NeuronCategoricalXYZPV1 needs a length of {} to fit just the cortical details header, but is a length of {}",
                                                                  min_array_length_with_cortical_headers, bytes.len())).into());
        }

        let number_cortical_areas: usize = number_cortical_areas as usize;
        let mut output: CorticalMappedXYZPNeuronData = CorticalMappedXYZPNeuronData::new_with_capacity(number_cortical_areas);

        let mut reading_header_index: usize = FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + Self::NUMBER_BYTES_CORTICAL_COUNT_HEADER;

        for _cortical_index in 0..number_cortical_areas {
            let cortical_id = CorticalID::from_bytes(
                <&[u8; 6]>::try_from(&bytes[reading_header_index..reading_header_index + 6]).unwrap()
            )?;
            let data_start_reading: usize = LittleEndian::read_u32(&bytes[reading_header_index + 6..reading_header_index + 10]) as usize;
            let number_bytes_to_read: usize = LittleEndian::read_u32(&bytes[reading_header_index + 10..reading_header_index + 14]) as usize;

            if bytes.len() < data_start_reading + number_bytes_to_read {
                return Err(FeagiDataError::SerializationError("Byte structure for NeuronCategoricalXYZPV1 is too short to fit the data the header says it contains!".into()).into());
            }

            let neuron_bytes = &bytes[data_start_reading..data_start_reading + number_bytes_to_read];
            let bytes_length = neuron_bytes.len();
            if bytes_length % NeuronVoxelXYZP::NUMBER_BYTES_PER_NEURON != 0 {
                return Err(FeagiDataError::SerializationError("Byte structure for NeuronCategoricalXYZPV1 seems invalid! Size is nonsensical given neuron data size!".into()).into());
            }

            if bytes_length % NeuronVoxelXYZP::NUMBER_BYTES_PER_NEURON != 0 {
                return Err(FeagiDataError::SerializationError("As NeuronXYCPArrays contains 4 internal arrays of equal length, each of elements of 4 bytes each (uint32 and float), the input bytes array must be divisible by 16!".into()).into());
            }
            let x_end = bytes_length / 4; // q1
            let y_end = bytes_length / 2; // q2
            let z_end = x_end * 3; // q3

            // Create vectors using bytes order to avoid alignment issues
            let num_neurons = bytes_length / NeuronVoxelXYZP::NUMBER_BYTES_PER_NEURON;
            let mut x_coords: Vec<u32> = Vec::with_capacity(num_neurons);
            let mut y_coords: Vec<u32> = Vec::with_capacity(num_neurons);
            let mut z_coords: Vec<u32> = Vec::with_capacity(num_neurons);
            let mut potentials: Vec<f32> = Vec::with_capacity(num_neurons);

            for i in 0..num_neurons {
                let x_start = i * 4;
                let y_start = x_end + x_start;
                let z_start = y_end + x_start;
                let p_start = z_end + x_start;

                x_coords.push(LittleEndian::read_u32(&neuron_bytes[x_start..x_start + 4]));
                y_coords.push(LittleEndian::read_u32(&neuron_bytes[y_start..y_start + 4]));
                z_coords.push(LittleEndian::read_u32(&neuron_bytes[z_start..z_start + 4]));
                potentials.push(LittleEndian::read_f32(&neuron_bytes[p_start..p_start + 4]));
            }

            let neuron_voxels = NeuronVoxelXYZPArrays::new_from_vectors(
                x_coords,
                y_coords,
                z_coords,
                potentials,
            )?;

            output.insert(cortical_id, neuron_voxels);
            reading_header_index += Self::NUMBER_BYTES_PER_CORTICAL_ID_HEADER;
        };

        Ok(output)
    }
}

 */

//region Iterators

impl IntoIterator for CorticalMappedXYZPNeuronVoxels {
    type Item = (CorticalID, NeuronVoxelXYZPArrays);
    type IntoIter = std::collections::hash_map::IntoIter<CorticalID, NeuronVoxelXYZPArrays>;

    /// Consumes the collection and returns an iterator over owned (CorticalID, NeuronVoxelXYZPArrays) pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let neuron_data = CorticalMappedXYZPNeuronVoxels::new();
    /// for (cortical_id, neurons) in neuron_data {
    ///     println!("Area {:?} has {} neurons", cortical_id, neurons.len());
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.mappings.into_iter()
    }
}

impl<'a> IntoIterator for &'a CorticalMappedXYZPNeuronVoxels {
    type Item = (&'a CorticalID, &'a NeuronVoxelXYZPArrays);
    type IntoIter = std::collections::hash_map::Iter<'a, CorticalID, NeuronVoxelXYZPArrays>;

    /// Returns an iterator over references to (CorticalID, NeuronVoxelXYZPArrays) pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let neuron_data = CorticalMappedXYZPNeuronVoxels::new();
    /// for (cortical_id, neurons) in &neuron_data {
    ///     println!("Area {:?} has {} neurons", cortical_id, neurons.len());
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.mappings.iter()
    }
}

impl<'a> IntoIterator for &'a mut CorticalMappedXYZPNeuronVoxels {
    type Item = (&'a CorticalID, &'a mut NeuronVoxelXYZPArrays);
    type IntoIter = std::collections::hash_map::IterMut<'a, CorticalID, NeuronVoxelXYZPArrays>;

    /// Returns a mutable iterator over (CorticalID, NeuronVoxelXYZPArrays) pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let mut neuron_data = CorticalMappedXYZPNeuronVoxels::new();
    /// for (cortical_id, neurons) in &mut neuron_data {
    ///     neurons.clear(); // Clear all neuron arrays
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.mappings.iter_mut()
    }
}

impl std::fmt::Display for CorticalMappedXYZPNeuronVoxels {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut inner: String = String::new();
        for cortical_id_and_data in self {
            inner.push_str(
                format!("[{}, {}],", cortical_id_and_data.0, cortical_id_and_data.1).as_str(),
            );
        }
        _ = inner.pop(); // Remove the last comma
        write!(f, "CorticalMappedXYZPNeuronData({})", inner)
    }
}

//endregion
