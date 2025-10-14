use std::any::Any;
use std::fmt::Debug;
use feagi_data_structures::{FeagiDataError, FeagiJSON};
use feagi_data_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use crate::{FeagiByteContainer, FeagiByteStructureType};

/// Trait for structures that can be serialized to and from FEAGI byte format.
/// 
/// Implementations must provide methods for determining their type, version,
/// size requirements, and serialization/deserialization logic. The trait includes
/// default validation methods for type and version checking.
pub trait FeagiSerializable: Debug + Any {

    /// Returns the structure type identifier.
    fn get_type(&self) -> FeagiByteStructureType;

    /// Returns the version number of this structure format.
    fn get_version(&self) -> u8;

    /// Returns the total number of bytes needed for serialization.
    fn get_number_of_bytes_needed(&self) -> usize;

    /// Serializes this structure into the provided byte slice.
    /// 
    /// The slice must be exactly the size returned by `get_number_of_bytes_needed`.
    fn try_serialize_struct_to_byte_slice(&self, byte_destination: &mut [u8]) -> Result<(), FeagiDataError>;

    /// Deserializes data from a byte slice and updates this structure.
    fn try_deserialize_and_update_self_from_byte_slice(&mut self, byte_reading: &[u8]) -> Result<(), FeagiDataError>;

    /// Provide access to `Any` trait for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Verifies that a byte slice contains data of the correct type.
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

    /// Verifies that a byte slice contains data of the correct version.
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

    // TODO universal method to export as a new FBS
}

// TODO there is cloning here, can it be avoided?

impl TryFrom<Box<dyn FeagiSerializable>> for FeagiJSON {
    type Error = FeagiDataError;
    fn try_from(value: Box<dyn FeagiSerializable>) -> Result<Self, Self::Error> {
        let option = value.as_any().downcast_ref::<FeagiJSON>();
        match option {
            Some(value) => Ok(value.clone()),
            None => Err(FeagiDataError::DeserializationError("This struct is not a FeagiJSON struct and cannot be deserialized as such!".into()))
        }
    }
}

impl TryFrom<Box<dyn FeagiSerializable>> for CorticalMappedXYZPNeuronVoxels {
    type Error = FeagiDataError;
    fn try_from(value: Box<dyn FeagiSerializable>) -> Result<Self, Self::Error> {
        let option = value.as_any().downcast_ref::<CorticalMappedXYZPNeuronVoxels>();
        match option {
            Some(value) => Ok(value.clone()),
            None => Err(FeagiDataError::DeserializationError("This struct is not a CorticalMappedXYZPNeuronVoxels struct and cannot be deserialized as such!".into()))
        }
    }
}