use feagi_data_structures::data::FeagiJSON;
use feagi_data_structures::FeagiDataError;
use crate::byte_structure::{FeagiByteContainer, FeagiByteStructureType, FeagiSerializable};

const BYTE_STRUCT_VERSION: u8 = 1;

impl FeagiSerializable for FeagiJSON {
    fn get_type(&self) -> FeagiByteStructureType {
        FeagiByteStructureType::JSON
    }

    fn get_version(&self) -> u8 {
        BYTE_STRUCT_VERSION
    }

    fn get_number_of_bytes_needed(&self) -> usize {
        self.borrow_json_value().to_string().as_bytes().len() + FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT
    }

    fn try_write_to_byte_slice(&self, byte_destination: &mut [u8]) -> Result<(), FeagiDataError> {
        byte_destination[0] = self.get_type() as u8;
        byte_destination[1] = self.get_version();

        let json_string = self.borrow_json_value().to_string();
        let json_bytes = json_string.as_bytes();

        // Write the JSON data as UTF-8 bytes
        byte_destination[FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT..].copy_from_slice(json_bytes);
        Ok(())
    }

    fn try_update_from_byte_slice(&mut self, byte_reading: &[u8]) -> Result<(), FeagiDataError> {
        let json_bytes = &byte_reading[FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT..];

        // Parse JSON string
        let json_value = match serde_json::from_slice(json_bytes) {
            Ok(value) => value,
            Err(e) => return Err(FeagiDataError::DeserializationError(format!("Invalid JSON data: {}", e))),
        };
        self.update_json_value(json_value);

        Ok(())

    }
}