use feagi_data_structures::data::FeagiJSON;
use feagi_data_structures::FeagiDataError;
use crate::byte_structure::{FeagiByteStructureCompatible, FeagiByteStructure, FeagiByteStructureType};

const BYTE_STRUCT_VERSION: u8 = 1;

impl FeagiByteStructureCompatible for FeagiJSON {
    fn get_type(&self) -> FeagiByteStructureType { FeagiByteStructureType::JSON }

    fn get_version(&self) -> u8 {BYTE_STRUCT_VERSION}

    fn overwrite_feagi_byte_structure_slice(&self, slice: &mut [u8]) -> Result<usize, FeagiDataError> {

        // doing this here instead of using the max_number_bytes_needed func so we can double-dip on data usage
        let json_string = self.borrow_json_value().to_string();
        let json_bytes = json_string.as_bytes();

        let num_bytes_needed: usize = FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + json_bytes.len();
        if slice.len() < num_bytes_needed {
            return Err(FeagiDataError::SerializationError(format!("Not enough space given to store JSON data! Need {} bytes but given {}!", num_bytes_needed, slice.len())));
        }

        // Write the global header
        slice[0] = self.get_type() as u8;
        slice[1] = self.get_version();

        // Write the JSON data as UTF-8 bytes
        slice[FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES..FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + json_bytes.len()].copy_from_slice(json_bytes);

        let wasted_space = slice.len() - num_bytes_needed;
        Ok(wasted_space)
    }

    fn max_number_bytes_needed(&self) -> usize {
        // Global header (2 bytes) + JSON data as UTF-8 bytes
        // TODO this is pretty slow, any faster way to do this?
        FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + self.borrow_json_value().to_string().as_bytes().len()
    }

    fn new_from_feagi_byte_structure(feagi_byte_structure: &FeagiByteStructure) -> Result<Self, FeagiDataError>
    where
        Self: Sized
    {
        FeagiByteStructure::verify_matching_structure_type_and_version(
            feagi_byte_structure,
            FeagiByteStructureType::JSON,
            BYTE_STRUCT_VERSION
        )?;

        let bytes = feagi_byte_structure.borrow_data_as_slice();

        if bytes.len() < FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES {
            return Err(FeagiDataError::DeserializationError("JSON bytes structure too short to contain global header".into()));
        }

        // Extract JSON data (everything after the global header)
        let json_bytes = &bytes[FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES..];

        // Parse JSON string
        let json_value = match serde_json::from_slice(json_bytes) {
            Ok(value) => value,
            Err(e) => return Err(FeagiDataError::DeserializationError(format!("Invalid JSON data: {}", e))),
        };

        Ok(FeagiJSON::from_json_value(json_value))
    }
}