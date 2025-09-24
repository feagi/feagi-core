use byteorder::{ByteOrder, LittleEndian};
use feagi_data_structures::FeagiDataError;
use crate::feagi_serializable::FeagiSerializable;
use crate::FeagiByteStructureType;

type StructureIndex = usize;
type ByteIndexReadingStart = usize;
type NumberBytesToRead = usize;

//region Feagi Byte Container

pub struct FeagiByteContainer {
    bytes: Vec<u8>,
    is_data_valid: bool,
    contained_struct_references: Vec<ContainedStructReference>,
}

impl FeagiByteContainer{
    pub const CURRENT_SUPPORTED_VERSION: u8 = 2;

    pub const GLOBAL_BYTE_HEADER_BYTE_COUNT: usize = 4; // 1 u8, 1 u16, 1 u8

    pub const PER_STRUCT_HEADER_BYTE_COUNT: usize = 4; // 1 u32

    pub const STRUCT_HEADER_BYTE_COUNT: usize = 2; // 1 u8, 1 u8

    //region Constructors

    pub fn new_empty() -> Self {
        Self { bytes: vec![Self::CURRENT_SUPPORTED_VERSION, 0, 0, 0], is_data_valid: false, contained_struct_references: Vec::new() }
    }

    //endregion

    // region Direct Data Access

    pub fn get_byte_ref(&self) -> &[u8] {
        &self.bytes
    }

    pub fn try_write_data_to_container_and_verify<F>(&mut self, byte_writer: &mut F) -> Result<(), FeagiDataError>
    where F: FnMut(&mut Vec<u8>) -> Result<(), FeagiDataError> {
        byte_writer(&mut self.bytes)?;
        self.verify_container_valid_and_populate()
    }

    //endregion

    //region Get Properties

    pub fn is_valid(&self) -> bool {
        self.is_data_valid
    }

    pub fn try_get_number_contained_structures(&self) -> Result<usize, FeagiDataError> {
        if self.is_data_valid {
            return Ok(self.contained_struct_references.len())
        }
        Err(FeagiDataError::DeserializationError("Given Byte Container is invalid and thus cannot be read!".into()))
    }

    pub fn get_number_of_bytes_used(&self) -> usize {
        self.bytes.len()
    }

    pub fn get_number_of_bytes_allocated(&self) -> usize {
        self.bytes.capacity()
    }

    pub fn get_increment_counter(&self) -> Result<u16, FeagiDataError> {
        if self.is_data_valid {
            return Ok(LittleEndian::read_u16(&self.bytes[1..3]))
        }
        Err(FeagiDataError::DeserializationError("Given Byte Container is invalid and thus cannot be read!".into()))
    }

    //endregion

    //region Extracting Struct Data

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

    pub fn get_contained_struct_types(&self) -> Vec<FeagiByteStructureType> {
        let mut output: Vec<FeagiByteStructureType> = Vec::with_capacity(self.contained_struct_references.len());
        for contained_struct_reference in &self.contained_struct_references {
            output.push(contained_struct_reference.structure_type);
        };
        output
    }

    //endregion

    //region Overwriting Data

    pub fn overwrite_byte_data_with_struct_data(&mut self, incoming_structs: Vec<Box<dyn FeagiSerializable>>, new_increment_value: u16) -> Result<(), FeagiDataError> {

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

        let number_needed_bytes_total = header_total_number_of_bytes + data_total_number_of_bytes;

        if number_needed_bytes_total > self.bytes.capacity() {
            self.bytes.reserve(number_needed_bytes_total - self.bytes.capacity());
        }

        // Every single byte will be overridden, don't worry
        unsafe {
            self.bytes.set_len(number_needed_bytes_total); // Fun!
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

    pub fn set_increment_counter_state(&mut self, new_increment_value: u16) -> Result<(), FeagiDataError> {
        if !self.is_data_valid {
            return Err(FeagiDataError::DeserializationError("Given Byte Container is invalid and thus cannot have its increment counter changed!".into()))
        };
        LittleEndian::write_u16(&mut self.bytes[1..3], new_increment_value);
        Ok(())
    }

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