use byteorder::{ByteOrder, LittleEndian};
use feagi_data_structures::FeagiDataError;
use crate::FeagiByteStructureType;


type StructureIndex = usize;
type ByteIndexReadingStart = usize;
type NumberBytesToRead = usize;

pub struct FeagiByteContainer {
    bytes: Vec<u8>,
    is_data_valid: bool,
    contained_struct_references: Vec<ContainedStructReference>,
}

impl FeagiByteContainer{
    pub const CURRENT_SUPPORTED_VERSION: u8 = 2;

    pub const GLOBAL_BYTE_HEADER_BYTE_COUNT: usize = 4; // 1 u8, 1 u16, 1 u8

    pub const PER_STRUCT_HEADER_BYTE_COUNT: usize = 4; // 1 u32

    //region Constructors

    pub fn new_empty() -> Self {
        Self { bytes: Vec::new(), is_data_valid: false, contained_struct_references: Vec::new() }
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

    //endregion

    //region Internal

    /// Verifies the bytes loaded in create a valid FBS container, with indexing that doesn't leave bounds,
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
            // NOTE: It is possible due to an error, that there is data sent after this point. However, we are going to treat this FBS as empty and report it as such.
        }

        let minimum_count_header_size = Self::PER_STRUCT_HEADER_BYTE_COUNT * number_contained_structs;
        let total_header_size = Self::GLOBAL_BYTE_HEADER_BYTE_COUNT + minimum_count_header_size;
        if byte_length < total_header_size {
            return Err(FeagiDataError::DeserializationError(format!("Feagi Byte Data specifies the existence of {} structures, but the given byte array is under the required {} byte length!", minimum_count_header_size, Self::GLOBAL_BYTE_HEADER_BYTE_COUNT + minimum_count_header_size)));
        }

        let mut structure_header_byte_index: usize = Self::GLOBAL_BYTE_HEADER_BYTE_COUNT;
        let mut structure_data_byte_index: usize = total_header_size;
        let mut structure_length: u32 = 0;
        let mut structure_type: FeagiByteStructureType;
        for contained_structure_index in 0..number_contained_structs {
            structure_length = LittleEndian::read_u32(&self.bytes[structure_header_byte_index..structure_header_byte_index + 4]);

            if structure_data_byte_index + structure_length as usize > byte_length {
                return Err(FeagiDataError::DeserializationError(
                    format!("Structure of index {} goes out of bound reaching position {} when given byte length is only {} long!", contained_structure_index, structure_data_byte_index + structure_length as usize, byte_length)));
            }

            structure_type = FeagiByteStructureType::try_from(self.bytes[structure_data_byte_index])?;
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

    fn try_get_first_structure_slice_of_type<'a>(&self, structure_type: FeagiByteStructureType) -> Option<&'a[u8]> {
        for index in 0..self.contained_struct_references.len() {
            if self.contained_struct_references[index].structure_type == structure_type {
                return Some(self.contained_struct_references[index].get_as_byte_slice(&self.bytes));
            }
        };
        None
    }

    //endregion

}

struct ContainedStructReference {
    structure_type: FeagiByteStructureType,
    byte_start_index: ByteIndexReadingStart,
    number_bytes_to_read: NumberBytesToRead
}

impl ContainedStructReference {
    pub fn get_as_byte_slice<'a>(&self, byte_source: &'a Vec<u8>) -> &'a [u8] {
        &byte_source[self.byte_start_index ..self.byte_start_index + self.number_bytes_to_read]
    }
}