//! Serialization implementation for JSON data structures.
//!
//! Provides UTF-8 text serialization for `FeagiJSON` values, allowing
//! arbitrary JSON data to be stored in FEAGI byte containers.

use crate::{FeagiByteContainer, FeagiByteStructureType, FeagiJSON, FeagiSerializable};
use std::any::Any;

/// Current version of the JSON serialization format.
const BYTE_STRUCT_VERSION: u8 = 1;

impl FeagiSerializable for FeagiJSON {
    fn get_type(&self) -> FeagiByteStructureType {
        FeagiByteStructureType::JSON
    }

    fn get_version(&self) -> u8 {
        BYTE_STRUCT_VERSION
    }

    fn get_number_of_bytes_needed(&self) -> usize {
        self.borrow_json_value().to_string().len() + FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT
    }

    fn try_serialize_struct_to_byte_slice(&self, byte_destination: &mut [u8]) -> Result<(), ()> {
        byte_destination[0] = self.get_type() as u8;
        byte_destination[1] = self.get_version();

        let json_string = self.borrow_json_value().to_string();
        let json_bytes = json_string.as_bytes();
        let header = FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT;
        let end = header + json_bytes.len();
        if end > byte_destination.len() {
            // JSON serialization overflow: the destination is smaller than the encoded value.
            return Err(());
        }
        // Write the JSON data as UTF-8 bytes (length may differ from get_number_of_bytes_needed due to serde_json formatting)
        byte_destination[header..end].copy_from_slice(json_bytes);
        Ok(())
    }

    fn try_deserialize_and_update_self_from_byte_slice(&mut self, byte_structure_slice: &[u8]) -> Result<(), ()> {
        // Assuming type is correct
        self.verify_byte_slice_is_of_correct_version(byte_structure_slice)?;

        let json_bytes = &byte_structure_slice[FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT..];

        // Invalid JSON data.
        let json_value = serde_json::from_slice(json_bytes).map_err(|_| ())?;
        self.update_json_value(json_value);

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
