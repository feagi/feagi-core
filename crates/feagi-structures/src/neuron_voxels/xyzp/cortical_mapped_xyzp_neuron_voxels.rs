use std::collections::HashMap;
use crate::base_quantizable::unsigned_integer::QuantizableUInt;
use crate::base_quantizable::value::QuantizableValue;
use crate::genomic::cortical_area::CorticalID;
use crate::neuron_voxels::xyzp::NeuronVoxelXYZPVectors;

/// Neuron voxel data organized by cortical area.
///
/// Maps cortical area identifiers to their respective neuron voxel collections,
/// allowing efficient storage and retrieval of neural activity across different
/// brain regions.
#[derive(Debug, Clone, PartialEq)]
pub struct CorticalMappedXYZPNeuronVoxels<Potential, CoordQuant>
where
    Potential: QuantizableValue,
    CoordQuant: QuantizableUInt
{
    /// Hash map storing neuron collections for each cortical area.
    ///
    /// The key is a unique cortical area identifier, and the value contains
    /// all neuron_voxels belonging to that cortical area.
    pub mappings: HashMap<CorticalID, NeuronVoxelXYZPVectors<Potential, CoordQuant>>,
}

impl<Potential: QuantizableValue, CoordQuant: QuantizableUInt> CorticalMappedXYZPNeuronVoxels<Potential, CoordQuant> {
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
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let neuron_data = CorticalMappedXYZPNeuronVoxels::new();
    /// assert_eq!(neuron_data.len(), 0);
    /// ```
    pub fn new() -> CorticalMappedXYZPNeuronVoxels<Potential, CoordQuant> {
        CorticalMappedXYZPNeuronVoxels {
            mappings: HashMap::new(),
        }
    }

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
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// // Pre-allocate for a brain with 100 cortical areas
    /// let neuron_data = CorticalMappedXYZPNeuronVoxels::new_with_capacity(100);
    /// assert_eq!(neuron_data.len(), 0);
    /// ```
    pub fn new_with_capacity(capacity: usize) -> CorticalMappedXYZPNeuronVoxels<Potential, CoordQuant> {
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
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
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
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
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
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
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
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
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
    pub fn get_neurons_of(
        &self,
        cortical_id: &CorticalID,
    ) -> Option<&NeuronVoxelXYZPVectors<Potential, CoordQuant>> {
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
    ) -> Option<&mut NeuronVoxelXYZPVectors<Potential, CoordQuant>> {
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
        neuron_data: NeuronVoxelXYZPVectors<Potential, CoordQuant>,
    ) -> Option<NeuronVoxelXYZPVectors<Potential, CoordQuant>> {
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
    pub fn remove(&mut self, cortical_id: CorticalID) -> Option<NeuronVoxelXYZPVectors<Potential, CoordQuant>> {
        self.mappings.remove(&cortical_id)
    }

    /// Removes all cortical areas and their neuron data.
    ///
    /// This operation clears the entire collection while maintaining the allocated capacity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
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
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let neuron_data = CorticalMappedXYZPNeuronVoxels::new();
    /// for neurons in neuron_data.iter() {
    ///     println!("Cortical area has {} neurons", neurons.len());
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &NeuronVoxelXYZPVectors<Potential, CoordQuant>> + '_ {
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
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
    ///
    /// let mut neuron_data = CorticalMappedXYZPNeuronVoxels::new();
    /// for neurons in neuron_data.iter_mut() {
    ///     neurons.clear(); // Clear all neuron arrays
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut NeuronVoxelXYZPVectors<Potential, CoordQuant>> + '_ {
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
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
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
    ) -> &mut NeuronVoxelXYZPVectors<Potential, CoordQuant> {
        if self.mappings.contains_key(cortical_id) {
            // If already contains neuron array, clear it and return it
            let neurons = self.mappings.get_mut(cortical_id).unwrap();
            neurons.clear();
            return neurons;
        }
        _ = self
            .mappings
            .insert(*cortical_id, NeuronVoxelXYZPVectors::new());
        self.mappings.get_mut(cortical_id).unwrap()
    }
}

impl<Potential: QuantizableValue, CoordQuant: QuantizableUInt> Default for CorticalMappedXYZPNeuronVoxels<Potential, CoordQuant> {
    fn default() -> Self {
        Self::new()
    }
}


//region Iterators

impl<Potential: QuantizableValue, CoordQuant: QuantizableUInt> IntoIterator for CorticalMappedXYZPNeuronVoxels<Potential, CoordQuant> {
    type Item = (CorticalID, NeuronVoxelXYZPVectors<Potential, CoordQuant>);
    type IntoIter = std::collections::hash_map::IntoIter<CorticalID, NeuronVoxelXYZPVectors<Potential, CoordQuant>>;

    /// Consumes the collection and returns an iterator over owned (CorticalID, NeuronVoxelXYZPArrays) pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
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

impl<'a, Potential: QuantizableValue, CoordQuant: QuantizableUInt> IntoIterator for &'a CorticalMappedXYZPNeuronVoxels<Potential, CoordQuant> {
    type Item = (&'a CorticalID, &'a NeuronVoxelXYZPVectors<Potential, CoordQuant>);
    type IntoIter = std::collections::hash_map::Iter<'a, CorticalID, NeuronVoxelXYZPVectors<Potential, CoordQuant>>;

    /// Returns an iterator over references to (CorticalID, NeuronVoxelXYZPArrays) pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
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

impl<'a, Potential: QuantizableValue, CoordQuant: QuantizableUInt> IntoIterator for &'a mut CorticalMappedXYZPNeuronVoxels<Potential, CoordQuant> {
    type Item = (&'a CorticalID, &'a mut NeuronVoxelXYZPVectors<Potential, CoordQuant>);
    type IntoIter = std::collections::hash_map::IterMut<'a, CorticalID, NeuronVoxelXYZPVectors<Potential, CoordQuant>>;

    /// Returns a mutable iterator over (CorticalID, NeuronVoxelXYZPArrays) pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
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

impl<Potential: QuantizableValue, CoordQuant: QuantizableUInt> std::fmt::Display for CorticalMappedXYZPNeuronVoxels<Potential, CoordQuant> {
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
