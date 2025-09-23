use feagi_data_structures::FeagiDataError;
use crate::{FeagiByteStructureType};

pub trait FeagiSerializable {

    // NOTE: None of these methods should be exposed outside this crate! THey should remain private!

    fn get_type(&self) -> FeagiByteStructureType;

    fn get_version(&self) -> u8;

    fn get_maximum_number_of_bytes_needed(&self) -> usize;

    fn try_write_to_byte_slice(&self, byte_destination: &mut [u8]) -> Result<(), FeagiDataError>;

    fn try_update_from_byte_slice(&mut self, byte_reading: &[u8]) -> Result<(), FeagiDataError>;

    fn try_make_from_byte_slice(byte_source: &[u8]) -> Result<Box<dyn FeagiSerializable>, FeagiDataError>
    where
        Self: Sized;

    fn verify_byte_slice_is_of_type(&self, byte_source: &[u8]) -> Result<(), FeagiDataError> {
        const MIN_SLICE_SIZE: usize = 2;
        if byte_source.len() < MIN_SLICE_SIZE {
            return Err(FeagiDataError::DeserializationError(
                format!("Byte slice needs to be at least {} bytes long to be considered valid! Given slice is {} elements long!", MIN_SLICE_SIZE, byte_source.len())
            ))
        }
        if byte_source[0] != self.get_type() as u8 {
            return Err(FeagiDataError::DeserializationError(format!("Attempted to process byte slice as structure type {} when given slice seems to be type {}!", self.get_type(), byte_source[0])))
        }
        if byte_source[1] != self.get_version() {
            return Err(FeagiDataError::DeserializationError(format!("Current implementation of Feagi Data Serialization supports structure ID {} of version {}, however version {} was given!!", self.get_type(), self.get_version(), byte_source[1])))
        }
        Ok(())
    }
}