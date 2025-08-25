use std::fmt;
use crate::FeagiDataError;
use crate::genomic::{CorticalType, CoreCorticalType, SensorCorticalType, MotorCorticalType};
use crate::genomic::descriptors::CorticalGroupIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CorticalID {
    pub(crate) bytes: [u8; CorticalID::CORTICAL_ID_LENGTH],
}

impl CorticalID {
    /// The fixed length of all cortical identifiers in bytes.
    pub const CORTICAL_ID_LENGTH: usize = 6;

    /// Alias for `CORTICAL_ID_LENGTH` for consistency with other bytes-oriented APIs.
    pub const NUMBER_OF_BYTES: usize = Self::CORTICAL_ID_LENGTH;
    
    //region Constructors

    pub fn new_custom_cortical_area_id(desired_id_string: String) -> Result<Self, FeagiDataError> {
        verify_input_length(&desired_id_string)?;
        verify_input_ascii(&desired_id_string)?;
        verify_allowed_characters(&desired_id_string)?;

        let bytes = desired_id_string.as_bytes();
        let bytes: &[u8; CorticalID::CORTICAL_ID_LENGTH] = bytes.try_into().unwrap();
        if bytes[0] != b'c' {
            return Err(FeagiDataError::BadParameters(format!("A custom cortical area ID must start with 'c'! Cortical area given: {}", desired_id_string)).into());
        }
        Ok(CorticalID { bytes: *bytes })
    }
    
    pub fn new_memory_cortical_area_id(desired_id_string: String) -> Result<Self, FeagiDataError> {

        verify_input_length(&desired_id_string)?;
        verify_input_ascii(&desired_id_string)?;
        verify_allowed_characters(&desired_id_string)?;

        let bytes = desired_id_string.as_bytes();
        let bytes: &[u8; CorticalID::CORTICAL_ID_LENGTH] = bytes.try_into().unwrap();
        if bytes[0] != b'm' {
            return Err(FeagiDataError::BadParameters(format!("A memory cortical area ID must start with 'm'! Cortical area given: {}", desired_id_string)).into());
        }
        Ok(CorticalID { bytes: *bytes })
    }
    
    pub fn new_core_cortical_area_id(core_type: CoreCorticalType) -> Result<Self, FeagiDataError> {
        Ok(core_type.to_cortical_id())
    }
    
    pub fn new_sensor_cortical_area_id(input_type: SensorCorticalType, input_index: CorticalGroupIndex) -> Result<Self, FeagiDataError> {
        Ok(input_type.to_cortical_id(input_index))
    }
    
    pub fn new_motor_cortical_area_id(output_type: MotorCorticalType, output_index: CorticalGroupIndex) -> Result<Self, FeagiDataError> {
        Ok(output_type.to_cortical_id(output_index))
    }
    
    pub fn from_bytes(bytes: &[u8; CorticalID::CORTICAL_ID_LENGTH]) -> Result<Self, FeagiDataError> {
        
        // Note: we didn't remove this function to avoid having to generate the string twice
        let as_string = String::from_utf8(bytes.to_vec());
        if as_string.is_err() {
            return Err(FeagiDataError::DeserializationError("Unable to parse cortical ID as ASCII!".into()));
        }

        let as_string = as_string.unwrap();
        verify_input_ascii(&as_string)?;
        verify_allowed_characters(&as_string)?;

        let _ = CorticalType::try_get_type_from_bytes(bytes)?; // if type is invalid, error
        Ok(CorticalID {bytes: *bytes})
    }
    
    pub fn from_string(string: String) -> Result<Self, FeagiDataError> {

        verify_input_length(&string)?;
        verify_input_ascii(&string)?;
        verify_allowed_characters(&string)?;

        let bytes: [u8; CorticalID::CORTICAL_ID_LENGTH] = string.as_bytes().try_into().unwrap();
        let _ = CorticalType::try_get_type_from_bytes(&bytes)?; // if type is invalid, error
        Ok(CorticalID {bytes })
    }
    
    pub fn try_from_cortical_type(cortical_type: &CorticalType, io_cortical_index: CorticalGroupIndex) -> Result<Self, FeagiDataError> {
        CorticalType::to_cortical_id(cortical_type, io_cortical_index)
    }
    
    //endregion
    
    //region Properties

    /// Returns the raw bytes of this cortical identifier.
    ///
    /// Provides direct access to the underlying 6-bytes array representation
    /// of the cortical ID. Useful for serialization, hashing, or low-level operations.
    ///
    /// # Returns
    /// Reference to the 6-bytes array containing the ASCII characters
    pub fn as_bytes(&self) -> &[u8; CorticalID::CORTICAL_ID_LENGTH] {
        &self.bytes
    }

    pub fn as_ascii_string(&self) -> String {
        risky_bytes_to_string(&self.bytes)
    }
    
    pub fn get_cortical_type(&self) -> CorticalType {
        CorticalType::try_get_type_from_bytes(&self.bytes).unwrap() // will never error
    }
    //endregion

}

impl fmt::Display for CorticalID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = risky_bytes_to_string(&self.bytes);
        write!(f, "'{}'", ch)
    }
}


fn risky_bytes_to_string(bytes: &[u8; CorticalID::CORTICAL_ID_LENGTH]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn verify_input_length(string: &String) -> Result<(), FeagiDataError> {
    if string.len() != CorticalID::CORTICAL_ID_LENGTH {
        return Err(FeagiDataError::BadParameters(format!("A cortical ID must have a length of {}! Given cortical ID '{}' is not!", CorticalID::CORTICAL_ID_LENGTH, string)));
    }
    Ok(())
}

fn verify_input_ascii(string: &String) -> Result<(), FeagiDataError> {
    if !string.is_ascii() {
        return Err(FeagiDataError::BadParameters(format!("A cortical ID must be entirely ASCII! Given cortical ID '{}' is not!", string)));
    }
    Ok(())
}

fn verify_allowed_characters(string: &String) -> Result<(), FeagiDataError> {
    if !string.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(FeagiDataError::BadParameters(format!("A cortical ID must be made only of alphanumeric characters and underscores! Given cortical ID '{}' is not!", string)));
    }
    Ok(())
}