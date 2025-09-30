use byteorder::{ByteOrder, LittleEndian};
use feagi_data_structures::FeagiDataError;
use crate::feagi_serializable::FeagiSerializable;
use crate::FeagiByteStructureType;

type StructureIndex = usize;
type ByteIndexReadingStart = usize;
type NumberBytesToRead = usize;

//region Feagi Byte Container

/// A container for serialized FEAGI data structures with efficient binary format.
/// 
/// `FeagiByteContainer` manages multiple serializable structures in a single byte array,
/// providing methods to read, write, and validate the contained data. The container uses
/// a header-based format with version control and structure indexing.
/// 
/// # Format
/// - Global header: version (1 byte) + increment counter (2 bytes) + struct count (1 byte)
/// - Per-structure headers: data length (4 bytes each)
/// - Structure data: serialized structure bytes
/// 
/// # Example
/// ```
/// use feagi_data_serialization::FeagiByteContainer;
/// 
/// let mut container = FeagiByteContainer::new_empty();
/// assert!(container.is_valid());
/// assert_eq!(container.get_number_of_bytes_used(), 4); // Just the header
/// ```
pub struct FeagiByteContainer {
    /// The actual contained byte data
    bytes: Vec<u8>,
    /// If the data inside the array is considered valid. If not, most functionality is disabled
    is_data_valid: bool,
    /// A vector of references to where in the bytes to get the slices of specific structs, and what type of Feagi data they are
    contained_struct_references: Vec<ContainedStructReference>,
}

impl FeagiByteContainer{
    pub const CURRENT_SUPPORTED_VERSION: u8 = 2;

    pub const GLOBAL_BYTE_HEADER_BYTE_COUNT: usize = 4; // 1 u8, 1 u16, 1 u8

    pub const PER_STRUCT_HEADER_BYTE_COUNT: usize = 4; // 1 u32

    pub const STRUCT_HEADER_BYTE_COUNT: usize = 2; // 1 u8, 1 u8

    //region Constructors

    /// Creates a new empty container with default header.
    /// 
    /// The container starts with a 4-byte header containing version, zero increment counter,
    /// and zero structure count. The container is initially valid with just a 4 byte header
    /// stating 0 contained structures
    /// 
    /// # Example
    /// ```
    /// use feagi_data_serialization::FeagiByteContainer;
    /// 
    /// let container = FeagiByteContainer::new_empty();
    /// assert!(container.is_valid());
    /// assert_eq!(container.get_number_of_bytes_used(), 4);
    /// ```
    pub fn new_empty() -> Self {
        Self { bytes: vec![Self::CURRENT_SUPPORTED_VERSION, 0, 0, 0], is_data_valid: true, contained_struct_references: Vec::new() }
    }

    //endregion

    // region Direct Data Access

    /// Returns a reference to the internal byte array.
    /// 
    /// Provides direct read access to the raw bytes of the container,
    /// including headers and all serialized structure data.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_serialization::FeagiByteContainer;
    /// 
    /// let container = FeagiByteContainer::new_empty();
    /// let bytes = container.get_byte_ref();
    /// assert_eq!(bytes.len(), 4);
    /// assert_eq!(bytes[0], 2); // Current version
    /// ```
    pub fn get_byte_ref(&self) -> &[u8] {
        &self.bytes
    }

    /// Writes data using a callback function and validates the container.
    /// 
    /// Allows external code to write directly to the byte array, then validates
    /// that the resulting data forms a valid container structure.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_serialization::{FeagiByteContainer};
    ///
    /// // NOTE: This function is just here as an example, but this specific implementation is invalid
    /// let mut container = FeagiByteContainer::new_empty();
    /// let result = container.try_write_data_to_container_and_verify(&mut |bytes| {
    ///     *bytes = vec![20u8, 2u8, 3u8]; // This is an invalid byte sequence
    ///     Ok(())
    /// });
    /// // This will fail validation since we're setting invalid data
    /// assert!(result.is_err());
    /// ```
    pub fn try_write_data_to_container_and_verify<F>(&mut self, byte_writer: &mut F) -> Result<(), FeagiDataError>
    where F: FnMut(&mut Vec<u8>) -> Result<(), FeagiDataError> {
        byte_writer(&mut self.bytes)?;
        self.verify_container_valid_and_populate()
    }

    //endregion

    //region Get Properties

    /// Checks if the container has valid data structure.
    /// 
    /// Returns true if the container has been validated and contains properly
    /// formatted header and structure data.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_serialization::FeagiByteContainer;
    /// 
    /// let container = FeagiByteContainer::new_empty();
    /// assert!(container.is_valid());
    /// ```
    pub fn is_valid(&self) -> bool {
        self.is_data_valid
    }

    /// Returns the number of structures contained in this container.
    /// 
    /// Only works if the container is valid. Returns an error if the container
    /// has not been validated or contains invalid data.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_serialization::FeagiByteContainer;
    ///
    /// let container = FeagiByteContainer::new_empty();
    /// assert_eq!(container.try_get_number_contained_structures().unwrap(), 0);
    /// ```
    pub fn try_get_number_contained_structures(&self) -> Result<usize, FeagiDataError> {
        if self.is_data_valid {
            return Ok(self.contained_struct_references.len())
        }
        Err(FeagiDataError::DeserializationError("Given Byte Container is invalid and thus cannot be read!".into()))
    }

    /// Returns the total number of bytes currently used by the container.
    /// 
    /// This includes headers and all structure data.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_serialization::FeagiByteContainer;
    /// 
    /// let container = FeagiByteContainer::new_empty();
    /// assert_eq!(container.get_number_of_bytes_used(), 4); // Header only
    /// ```
    pub fn get_number_of_bytes_used(&self) -> usize {
        self.bytes.len()
    }

    /// Returns the total memory allocated for the byte array.
    /// 
    /// This may be larger than the number of bytes used due to Vec capacity.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_serialization::FeagiByteContainer;
    /// 
    /// let container = FeagiByteContainer::new_empty();
    /// assert!(container.get_number_of_bytes_allocated() >= 4);
    /// ```
    pub fn get_number_of_bytes_allocated(&self) -> usize {
        self.bytes.capacity()
    }

    /// Returns the increment counter value from the header.
    /// 
    /// The increment counter is a 16-bit value stored in bytes 1-2 of the header.
    /// Only works if the container is valid.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_serialization::FeagiByteContainer;
    ///
    /// let container = FeagiByteContainer::new_empty();
    /// assert_eq!(container.get_increment_counter().unwrap(), 0u16);
    /// ```
    pub fn get_increment_counter(&self) -> Result<u16, FeagiDataError> {
        if self.is_data_valid {
            return Ok(LittleEndian::read_u16(&self.bytes[1..3]))
        }
        Err(FeagiDataError::DeserializationError("Given Byte Container is invalid and thus cannot be read!".into()))
    }

    //endregion

    //region Extracting Struct Data

    /// Creates a new structure instance from the data at the specified index.
    /// 
    /// Deserializes the structure data at the given index and returns a boxed
    /// trait object. The structure type is determined from the stored metadata.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_serialization::FeagiByteContainer;
    /// 
    /// let container = FeagiByteContainer::new_empty();
    /// // This will fail since there are no structures
    /// assert!(container.try_create_new_struct_from_index(0).is_err());
    /// ```
    pub fn try_create_new_struct_from_index(&self, index: StructureIndex) -> Result<Box<dyn FeagiSerializable>, FeagiDataError> {
        self.verify_structure_index_valid(index)?;
        let relevant_slice = self.contained_struct_references[index].get_as_byte_slice(&self.bytes);
        let mut boxed_struct: Box<dyn FeagiSerializable> = self.contained_struct_references[index].structure_type.create_new_struct_of_type();
        boxed_struct.try_update_from_byte_slice(relevant_slice)?;
        Ok(boxed_struct)
    }

    pub fn try_create_struct_from_first_found_struct_of_type(&self, structure_type: FeagiByteStructureType) -> Result<Option<Box<dyn FeagiSerializable>>, FeagiDataError> {
        let getting_slice = self.try_get_first_structure_slice_of_type(structure_type);
        if getting_slice.is_none() {
            return Ok(None);
        }
        let mut boxed_struct: Box<dyn FeagiSerializable> = structure_type.create_new_struct_of_type();
        boxed_struct.try_update_from_byte_slice(getting_slice.unwrap())?;
        Ok(Some(boxed_struct))
    }

    pub fn try_update_struct_from_index(&self, index: StructureIndex, updating_boxed_struct: &mut Box<dyn FeagiSerializable>) -> Result<(), FeagiDataError> {
        self.verify_structure_index_valid(index)?;
        let relevant_slice = self.contained_struct_references[index].get_as_byte_slice(&self.bytes);
        updating_boxed_struct.verify_byte_slice_is_of_correct_type(relevant_slice)?;
        updating_boxed_struct.try_update_from_byte_slice(relevant_slice)?;
        Ok(())
    }

    pub fn try_update_struct_from_first_found_struct_of_type(&self, updating_boxed_struct: &mut Box<dyn FeagiSerializable>) -> Result<bool, FeagiDataError> {
        let structure_type: FeagiByteStructureType = updating_boxed_struct.get_type();
        let getting_slice = self.try_get_first_structure_slice_of_type(structure_type);
        if getting_slice.is_none() {
            return Ok(false);
        }
        updating_boxed_struct.try_update_from_byte_slice(getting_slice.unwrap())?;
        Ok(true)
    }

    /// Returns a list of all structure types contained in this container.
    /// 
    /// Provides a quick way to see what types of structures are available
    /// without deserializing them.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_serialization::FeagiByteContainer;
    /// 
    /// let container = FeagiByteContainer::new_empty();
    /// let types = container.get_contained_struct_types();
    /// assert_eq!(types.len(), 0); // Empty container
    /// ```
    pub fn get_contained_struct_types(&self) -> Vec<FeagiByteStructureType> {
        let mut output: Vec<FeagiByteStructureType> = Vec::with_capacity(self.contained_struct_references.len());
        for contained_struct_reference in &self.contained_struct_references {
            output.push(contained_struct_reference.structure_type);
        };
        output
    }

    //endregion

    //region Overwriting Data

    pub fn overwrite_byte_data_with_multiple_struct_data(&mut self, incoming_structs: Vec<&Box<dyn FeagiSerializable>>, new_increment_value: u16) -> Result<(), FeagiDataError> {

        self.bytes.clear();
        self.contained_struct_references.clear(); // Technically this causes a memory leak. Too Bad!
        self.is_data_valid = false;

        let header_total_number_of_bytes: usize = Self::GLOBAL_BYTE_HEADER_BYTE_COUNT +
            Self::PER_STRUCT_HEADER_BYTE_COUNT * incoming_structs.len();

        // Fill out contained_struct_references, calculate total number of bytes used for the data section
        let data_total_number_of_bytes = {
            let mut data_start_index = header_total_number_of_bytes;
            for incoming_struct in &incoming_structs {
                let per_struct_number_bytes = incoming_struct.get_number_of_bytes_needed();
                self.contained_struct_references.push(
                    ContainedStructReference{
                        structure_type: incoming_struct.get_type(),
                        byte_start_index: data_start_index,
                        number_bytes_to_read: per_struct_number_bytes,
                    }
                );
                data_start_index += per_struct_number_bytes;
            }
            data_start_index
        };

        if data_total_number_of_bytes > self.bytes.capacity() {
            self.bytes.reserve(data_total_number_of_bytes - self.bytes.capacity());
        }

        // Every single byte will be overridden, don't worry
        unsafe {
            self.bytes.set_len(data_total_number_of_bytes); // Fun!
        }

        // Setup global header
        self.bytes[0] = Self::CURRENT_SUPPORTED_VERSION;
        LittleEndian::write_u16(&mut self.bytes[1..3], new_increment_value); // Next 2 bytes is increment counter
        self.bytes[3] = incoming_structs.len() as u8; // Struct count

        // Write Header and Data bytes at the same time
        let mut header_byte_index = Self::GLOBAL_BYTE_HEADER_BYTE_COUNT;
        for struct_index in 0..incoming_structs.len() {
            let incoming_struct = &incoming_structs[struct_index];
            let contained_struct_reference = &self.contained_struct_references[struct_index];

            LittleEndian::write_u32(&mut self.bytes[header_byte_index..header_byte_index + 4], contained_struct_reference.number_bytes_to_read as u32);
            incoming_struct.try_write_to_byte_slice(contained_struct_reference.get_as_byte_slice_mut(&mut self.bytes))?;

            header_byte_index += Self::PER_STRUCT_HEADER_BYTE_COUNT;
        };

        self.is_data_valid = true;
        Ok(())

    }

    /// Updates the increment counter in the header.
    /// 
    /// Modifies the 16-bit increment counter stored in bytes 1-2 of the header.
    /// The container must be valid for this operation to succeed.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_serialization::FeagiByteContainer;
    /// 
    /// let mut container = FeagiByteContainer::new_empty();
    /// container.set_increment_counter_state(42);
    /// ```
    pub fn set_increment_counter_state(&mut self, new_increment_value: u16) -> Result<(), FeagiDataError> {
        if !self.is_data_valid {
            return Err(FeagiDataError::DeserializationError("Given Byte Container is invalid and thus cannot have its increment counter changed!".into()))
        };
        LittleEndian::write_u16(&mut self.bytes[1..3], new_increment_value);
        Ok(())
    }

    /// Frees any unused memory allocation in the byte vector.
    /// 
    /// Shrinks the capacity of the internal byte vector to match its length,
    /// potentially reducing memory usage.
    /// 
    /// # Example
    /// ```
    /// use feagi_data_serialization::FeagiByteContainer;
    /// 
    /// let mut container = FeagiByteContainer::new_empty();
    /// container.free_unused_allocation();
    /// assert_eq!(container.get_number_of_bytes_allocated(), container.get_number_of_bytes_used());
    /// ```
    pub fn free_unused_allocation(&mut self) {
        self.bytes.shrink_to_fit()
    }

    //endregion

    //region Internal

    /// Verifies the bytes loaded in create a valid FBC container, with indexing that doesn't leave bounds,
    /// and also configures contained_struct_references.
    /// WARNING: Does not verify the contained structures themselves!
    fn verify_container_valid_and_populate(&mut self) -> Result<(), FeagiDataError> {
        self.is_data_valid = false;
        self.contained_struct_references.clear();
        let byte_length = self.bytes.len();

        // Verify Global Header
        if byte_length < Self::GLOBAL_BYTE_HEADER_BYTE_COUNT { // If we cant even fit the global header, something is wrong
            return Err(FeagiDataError::DeserializationError("Given Feagi Byte Structure byte length is too short! (Less than 4!)".into()));
        }
        if self.bytes[0] != Self::CURRENT_SUPPORTED_VERSION {
            return Err(FeagiDataError::DeserializationError(format!("Given FEAGI Byte Structure is using version {} when this application only supports version {}!", self.bytes[0], Self::CURRENT_SUPPORTED_VERSION)));
        }
        let number_contained_structs = LittleEndian::read_u16(&self.bytes[1..3]) as usize;
        if number_contained_structs == 0 {
            self.is_data_valid = true; // This is technically valid, even though no meaningful data was sent
            return Ok(())
            // NOTE: It is possible due to an error, that there is data sent after this point. However, we are going to treat this FBC as empty and report it as such.
        }

        let minimum_count_header_size = Self::PER_STRUCT_HEADER_BYTE_COUNT * number_contained_structs;
        let total_header_size = Self::GLOBAL_BYTE_HEADER_BYTE_COUNT + minimum_count_header_size;
        if byte_length < total_header_size {
            return Err(FeagiDataError::DeserializationError(format!("Feagi Byte Data specifies the existence of {} structures, but the given byte array is under the required {} byte length!", minimum_count_header_size, Self::GLOBAL_BYTE_HEADER_BYTE_COUNT + minimum_count_header_size)));
        }

        let mut structure_header_byte_index: usize = Self::GLOBAL_BYTE_HEADER_BYTE_COUNT;
        let mut structure_data_byte_index: usize = total_header_size;
        for contained_structure_index in 0..number_contained_structs {
            let structure_length = LittleEndian::read_u32(&self.bytes[structure_header_byte_index..structure_header_byte_index + 4]);

            if structure_data_byte_index + structure_length as usize > byte_length {
                return Err(FeagiDataError::DeserializationError(
                    format!("Structure of index {} goes out of bound reaching position {} when given byte length is only {} long!", contained_structure_index, structure_data_byte_index + structure_length as usize, byte_length)));
            }

            let structure_type = FeagiByteStructureType::try_from(self.bytes[structure_data_byte_index])?;
            self.contained_struct_references.push( ContainedStructReference {
                structure_type,
                byte_start_index: structure_data_byte_index,
                number_bytes_to_read: structure_length as usize
            });

            structure_header_byte_index += 4; // Next u32
            structure_data_byte_index += structure_length as usize;
        }
        Ok(())
    }

    /// Makes sure the given index is valid (not out of range given number of contained structs)
    fn verify_structure_index_valid(&self, structure_index: StructureIndex) -> Result<(), FeagiDataError> {
        if structure_index >= self.contained_struct_references.len() {
            return Err(FeagiDataError::BadParameters(format!("Structure index {} out of bounds! Feagi Byte Container only contains {} structures!", structure_index, self.contained_struct_references.len())));
        }
        Ok(())
    }

    /// Tries to the get the first structure in the contained structure list that is of the requested type. If none are found, returns None.
    fn try_get_first_structure_slice_of_type(&self, structure_type: FeagiByteStructureType) -> Option<&[u8]> {
        for index in 0..self.contained_struct_references.len() {
            if self.contained_struct_references[index].structure_type == structure_type {
                return Some(self.contained_struct_references[index].get_as_byte_slice(&self.bytes));
            }
        };
        None
    }

    //endregion

}

//endregion

//region Contained Struct Reference
struct ContainedStructReference {
    structure_type: FeagiByteStructureType,
    byte_start_index: ByteIndexReadingStart,
    number_bytes_to_read: NumberBytesToRead
}

impl ContainedStructReference {
    pub fn get_as_byte_slice<'a>(&self, byte_source: &'a Vec<u8>) -> &'a [u8] {
        &byte_source[self.byte_start_index ..self.byte_start_index + self.number_bytes_to_read]
    }

    pub fn get_as_byte_slice_mut<'a>(&self, byte_source: &'a mut Vec<u8>) -> &'a mut [u8] {
        &mut byte_source[self.byte_start_index ..self.byte_start_index + self.number_bytes_to_read]
    }
}

//endregion