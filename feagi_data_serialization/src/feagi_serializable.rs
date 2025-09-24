use feagi_data_structures::FeagiDataError;
use crate::{FeagiByteContainer, FeagiByteStructureType};

pub trait FeagiSerializable {

    // NOTE: None of these methods should be exposed outside this crate! THey should remain private!
    /// Returns type of structure this is, as defined in the FEAGI Data Serialization Docs
    fn get_type(&self) -> FeagiByteStructureType;

    /// Returns the specific version of the structure supported by the current code base
    fn get_version(&self) -> u8;

    /// Returns the number of bytes needed by be allocated by the FeagiByteContainer when storing the data
    fn get_number_of_bytes_needed(&self) -> usize;

    /// When given a mutable slice of bytes size specified by "get_number_of_bytes_needed", serialized the struct into it
    fn try_write_to_byte_slice(&self, byte_destination: &mut [u8]) -> Result<(), FeagiDataError>;

    /// Given a slice of data of this structure of correct size, Deserialize the slice and update (replace) the data of the structure
    fn try_update_from_byte_slice(&mut self, byte_reading: &[u8]) -> Result<(), FeagiDataError>;

    /// Verifies that the data slice is of the type expected of the struct
    fn verify_byte_slice_is_of_correct_type(&self, byte_source: &[u8]) -> Result<(), FeagiDataError> {

        if byte_source.len() <= FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT {
            return Err(FeagiDataError::DeserializationError(
                format!("Byte slice needs to be at least {} bytes long to be considered valid! Given slice is {} elements long!", FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT, byte_source.len())
            ))
        }
        if byte_source[0] != self.get_type() as u8 {
            return Err(FeagiDataError::DeserializationError(format!("Attempted to process byte slice as structure type {} when given slice seems to be type {}!", self.get_type(), byte_source[0])))
        }
        Ok(())
    }

    /// Verifies that the data slice is of the version expected of the struct
    fn verify_byte_slice_is_of_correct_version(&self, byte_source: &[u8]) -> Result<(), FeagiDataError> {

        if byte_source.len() < FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT {
            return Err(FeagiDataError::DeserializationError(
                format!("Byte slice needs to be at least {} bytes long to be considered valid! Given slice is {} elements long!", FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT, byte_source.len())
            ))
        }
        if byte_source[1] != self.get_version() {
            return Err(FeagiDataError::DeserializationError(format!("Current implementation of Feagi Data Serialization supports structure ID {} of version {}, however version {} was given!!", self.get_type(), self.get_version(), byte_source[1])))
        }
        Ok(())
    }
}