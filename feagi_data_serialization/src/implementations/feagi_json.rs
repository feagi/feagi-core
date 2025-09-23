use feagi_data_structures::data::FeagiJSON;
use feagi_data_structures::FeagiDataError;
use crate::byte_structure::{FeagiByteStructureType, FeagiSerializable};

impl FeagiSerializable for FeagiJSON {
    fn get_type(&self) -> FeagiByteStructureType {
        todo!()
    }

    fn get_version(&self) -> u8 {
        todo!()
    }

    fn get_number_of_bytes_needed(&self) -> usize {
        todo!()
    }

    fn try_write_to_byte_slice(&self, byte_destination: &mut [u8]) -> Result<(), FeagiDataError> {
        todo!()
    }

    fn try_update_from_byte_slice(&mut self, byte_reading: &[u8]) -> Result<(), FeagiDataError> {
        todo!()
    }
}