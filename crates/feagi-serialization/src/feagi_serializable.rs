use crate::{FeagiByteContainer, FeagiByteStructureType};
use feagi_structures::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use feagi_structures::neuron_voxels::coord_potential::CorticalMappedNeuronVoxelCoordVectors;
use feagi_structures::{FeagiJSON, FeagiStructuresError};
use std::any::Any;
use std::fmt::Debug;

/// Trait for structures that can be serialized to and from FEAGI byte format.
///
/// The trait is intentionally NOT generic over quantization parameters. All
/// methods operate on raw bytes, so the trait stays object-safe and one
/// `Box<dyn FeagiSerializable>` can carry any implementor regardless of its
/// internal quant parameterization. Concrete implementations that ARE generic
/// (e.g. `CorticalMappedNeuronVoxelCoordVectors<V, C, N, A>`) supply their own
/// `impl` block with the appropriate quant bounds.
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
    fn try_serialize_struct_to_byte_slice(
        &self,
        byte_destination: &mut [u8],
    ) -> Result<(), FeagiStructuresError>;

    /// Deserializes data from a byte slice and updates this structure.
    fn try_deserialize_and_update_self_from_byte_slice(
        &mut self,
        byte_reading: &[u8],
    ) -> Result<(), FeagiStructuresError>;

    /// Provides access to `Any` trait for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Verifies that a byte slice contains data of the correct type.
    fn verify_byte_slice_is_of_correct_type(
        &self,
        byte_source: &[u8],
    ) -> Result<(), FeagiStructuresError> {
        if byte_source.len() <= FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT {
            return Err(FeagiStructuresError::DeserializationError(format!(
                "Byte slice needs to be at least {} bytes long to be considered valid! Given slice is {} elements long!",
                FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT,
                byte_source.len()
            )));
        }
        if byte_source[0] != self.get_type() as u8 {
            return Err(FeagiStructuresError::DeserializationError(format!(
                "Attempted to process byte slice as structure type {} when given slice seems to be type {}!",
                self.get_type(),
                byte_source[0]
            )));
        }
        Ok(())
    }

    /// Verifies that a byte slice contains data of the correct version.
    fn verify_byte_slice_is_of_correct_version(
        &self,
        byte_source: &[u8],
    ) -> Result<(), FeagiStructuresError> {
        if byte_source.len() < FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT {
            return Err(FeagiStructuresError::DeserializationError(format!(
                "Byte slice needs to be at least {} bytes long to be considered valid! Given slice is {} elements long!",
                FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT,
                byte_source.len()
            )));
        }
        if byte_source[1] != self.get_version() {
            return Err(FeagiStructuresError::DeserializationError(format!(
                "Current implementation of Feagi Data Serialization supports structure ID {} of version {}, however version {} was given!!",
                self.get_type(),
                self.get_version(),
                byte_source[1]
            )));
        }
        Ok(())
    }
}

impl TryFrom<Box<dyn FeagiSerializable>> for FeagiJSON {
    type Error = FeagiStructuresError;
    fn try_from(value: Box<dyn FeagiSerializable>) -> Result<Self, Self::Error> {
        match value.as_any().downcast_ref::<FeagiJSON>() {
            Some(value) => Ok(value.clone()),
            None => Err(FeagiStructuresError::DeserializationError(
                "This struct is not a FeagiJSON struct and cannot be deserialized as such!".into(),
            )),
        }
    }
}

impl<V, C, N, A> TryFrom<Box<dyn FeagiSerializable>>
    for CorticalMappedNeuronVoxelCoordVectors<V, C, N, A>
where
    V: QuantizableValueType,
    C: QuantizableUIntType,
    N: QuantizableUIntType,
    A: QuantizableUIntType,
    Self: Clone,
{
    type Error = FeagiStructuresError;
    fn try_from(value: Box<dyn FeagiSerializable>) -> Result<Self, Self::Error> {
        match value.as_any().downcast_ref::<Self>() {
            Some(value) => Ok(value.clone()),
            None => Err(FeagiStructuresError::DeserializationError(
                "This struct is not a CorticalMappedNeuronVoxelCoordVectors struct (with matching \
                 quantization parameters) and cannot be deserialized as such!"
                    .into(),
            )),
        }
    }
}
