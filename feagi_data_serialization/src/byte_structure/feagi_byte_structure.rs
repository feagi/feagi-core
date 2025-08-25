//! Core implementation of the FEAGI bytes structure container.
//!
//! This module provides the main `FeagiByteStructure` type, which serves as a validated
//! container for serialized FEAGI data. It handles both single structures and multi-structure
//! wrapped_io_data, providing comprehensive validation, parsing, and manipulation capabilities.

use byteorder::{ByteOrder, LittleEndian};
use feagi_data_structures::data::FeagiJSON;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::neurons::xyzp::CorticalMappedXYZPNeuronData;
use super::FeagiByteStructureType;
use super::FeagiByteStructureCompatible;

/// Core container for validated FEAGI bytes structure data.
///
/// `FeagiByteStructure` is the central type for handling serialized FEAGI data structures.
/// It provides a validated wrapper around raw bytes data, ensuring format compliance and
/// offering high-level operations for manipulation and extraction.
///
/// # Key Features
///
/// - **Format Validation**: Ensures bytes data conforms to FEAGI standards
/// - **Type Safety**: Validates structure types and versions before operations
/// - **Multi-Structure Support**: Can contain and manage multiple structures in one container
/// - **Zero-Copy Operations**: Provides efficient access to internal data without unnecessary copying
/// - **Extensible Design**: Supports new formats through the trait system
///
/// # Binary Format Overview
///
/// ## Single Structure Format
/// ```text
/// [Type=? (1)][Version (1)][Type-specific payload...]
/// ```
///
/// ## Multi-Structure Container Format
/// ```text
/// [Type=9 (1)][Version (1)][Count (1)][Headers...][Data...]
/// 
/// Where Headers = [Start₁ (4)][Length₁ (4)][Start₂ (4)][Length₂ (4)]...
/// And Data = [Structure₁][Structure₂]...
/// ```
/// 
/// # Thread Safety
///
/// `FeagiByteStructure` is thread-safe for concurrent read operations. Write operations
/// (like capacity management) require exclusive access.
#[derive(Clone)]
pub struct FeagiByteStructure {
    bytes: Vec<u8>,
}

impl FeagiByteStructure {
    /// Size of the global header (type + version) in bytes.
    pub const GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES: usize = 2;
    
    /// Minimum bytes length required for a valid FEAGI bytes structure.
    pub const MINIMUM_LENGTH_TO_BE_CONSIDERED_VALID: usize = 4;
    
    /// Size of the structure count field in multi-structure wrapped_io_data.
    pub const MULTISTRUCT_STRUCT_COUNT_BYTE_SIZE: usize = 1;
    
    /// Size of each structure's header entry in multi-structure wrapped_io_data (start position + length).
    pub const MULTISTRUCT_PER_STRUCT_HEADER_SIZE_IN_BYTES: usize = 8;

    /// Currently supported version for JSON format structures.
    pub const SUPPORTED_VERSION_JSON: u8 = 1;
    
    /// Currently supported version for multi-structure wrapped_io_data.
    pub const SUPPORTED_VERSION_MULTI_STRUCT: u8 = 1;
    
    /// Currently supported version for neuron XYZP format structures.
    pub const SUPPORTED_VERSION_NEURON_XYZP: u8 = 1;
    
    //region Constructors
    
    /// Creates a new FeagiByteStructure from raw bytes data with full validation.
    ///
    /// This is the primary constructor that validates the provided bytes data against
    /// FEAGI format requirements. It performs comprehensive checks to ensure the data
    /// represents a valid bytes structure before creating the container.
    ///
    /// # Arguments
    /// * `bytes` - Raw bytes data containing a serialized FEAGI structure
    ///
    /// # Returns
    /// * `Ok(FeagiByteStructure)` - Validated bytes structure ready for use
    /// * `Err(FeagiDataError)` - If validation fails due to:
    ///   - Insufficient data length (< 4 bytes minimum)
    ///   - Invalid structure type identifier
    ///   - Version number of 0 (reserved/invalid)
    ///   - Format-specific validation failures
    ///
    /// # Validation Performed
    /// - Minimum length check (at least 4 bytes)
    /// - Valid structure type identifier (bytes 0)
    /// - Non-zero version number (bytes 1)
    /// - Additional format-specific validations may be added
    pub fn create_from_bytes(bytes: Vec<u8>) -> Result<FeagiByteStructure, FeagiDataError> {
        if bytes.len() < Self::MINIMUM_LENGTH_TO_BE_CONSIDERED_VALID {
            return Err(FeagiDataError::DeserializationError(format!("Byte structure needs to be at least {} long to be considered valid. Given structure is only {} long!", Self::MINIMUM_LENGTH_TO_BE_CONSIDERED_VALID, bytes.len())).into());
        }
        _ = FeagiByteStructureType::try_from(bytes[0])?; // check if structure type is valid
        if bytes[1] == 0 {return Err(FeagiDataError::DeserializationError("Byte structure cannot have version number of 0!".into()).into());}
        // NOTE: Other checks go here
        
        Ok(Self { bytes })
    }
    
    /// Creates a multi-structure container from exactly two existing structures.
    ///
    /// This convenience method combines two FEAGI bytes structures into a single
    /// multi-structure container. It's optimized for the common case of combining
    /// two structures and delegates to the general multi-structure creation logic.
    ///
    /// # Arguments
    /// * `a` - First structure to include in the container
    /// * `b` - Second structure to include in the container
    ///
    /// # Returns
    /// * `Ok(FeagiByteStructure)` - Multi-structure container with both inputs
    /// * `Err(FeagiDataError)` - If container creation fails
    ///
    /// # Behavior
    /// - If either input is already a multi-structure, it will be flattened
    /// - The result will contain exactly the individual structures from both inputs
    /// - No nested multi-structures are created
    pub fn create_from_2_existing(a: &FeagiByteStructure, b: &FeagiByteStructure) -> Result<FeagiByteStructure, FeagiDataError> {
        // TODO Using vectors here is easier now, but we can squeeze a bit more performance by making a specific 2 slice system
        let structs = vec!(a, b);
        FeagiByteStructure::create_from_multiple_existing(structs)
    }
    
    /// Creates a multi-structure container from multiple existing structures.
    ///
    /// This method combines any number of FEAGI bytes structures into a single
    /// multi-structure container. It automatically flattens any input multi-structures
    /// to avoid nesting and enforces reasonable limits on container size.
    ///
    /// # Arguments
    /// * `existing` - Vector of references to structures to combine
    ///
    /// # Returns
    /// * `Ok(FeagiByteStructure)` - Multi-structure container with all inputs
    /// * `Err(FeagiDataError)` - If creation fails due to:
    ///   - Empty input vector (at least one structure required)
    ///   - Too many structures (maximum 255 supported)
    ///   - Memory allocation failures
    ///
    /// # Behavior
    /// - **Single Input**: Returns a clone of the input (no container needed)
    /// - **Multiple Inputs**: Creates a new multi-structure container
    /// - **Flattening**: Any input multi-structures are flattened into individual structures
    /// - **Ordering**: Output order matches input order, with multi-structure contents expanded in place
    ///
    /// # Container Limits
    /// - Maximum of 255 individual structures in the final container
    /// - This limit is enforced after flattening all input multi-structures
    pub fn create_from_multiple_existing(existing: Vec<&FeagiByteStructure>) -> Result<FeagiByteStructure, FeagiDataError> {
        
        if existing.is_empty() {
            return Err(FeagiDataError::BadParameters("You must specify at least one bytes structure to put into a multistruct!".into()).into());
        }
        
        if existing.len() == 1 {
            // No need to make a whole new structure, just copy this
            return Ok(existing[0].clone());
        }
        
        // Break apart any input multistructs, we don't want nesting. Assuming FeagiByteStructures are Valid
        let mut slices: Vec<&[u8]> = Vec::new();
        for input in existing {
            if input.is_multistruct()? {
                slices.extend(input.get_all_multistruct_internal_slices())
            }
            else {
                slices.push(input.borrow_data_as_slice())
            }
        }

        if slices.len() > 255 {
            // wtf are you doing
            return Err(FeagiDataError::BadParameters("The maximum number of structures that can exist in a multistruct is 255!".into()).into());
        }
        
        Ok(FeagiByteStructure::build_multistruct_from_slices(slices))
    }
    
    /// Creates a FeagiByteStructure from any compatible object.
    ///
    /// This convenience constructor takes any object implementing the
    /// `FeagiByteStructureCompatible` trait and converts it to a validated
    /// bytes structure. Useful for serializing custom types.
    ///
    /// # Arguments
    /// * `object` - Boxed object implementing FeagiByteStructureCompatible
    ///
    /// # Returns
    /// * `Ok(FeagiByteStructure)` - Validated bytes structure containing the serialized object
    /// * `Err(FeagiDataError)` - If serialization or validation fails
    pub fn create_from_compatible(object: Box<dyn FeagiByteStructureCompatible>) -> Result<FeagiByteStructure, FeagiDataError> {
        // Essentially just an alias
        object.as_new_feagi_byte_structure()
    }
    
    /*
    pub fn create_from_multiple_compatible(objects: Vec<Box<dyn FeagiByteStructureCompatible>>) -> Result<FeagiByteStructure, FeagiDataError> {
        todo!()
    }
     */
    //endregion
    
    //region static safety checks
    pub(crate) fn verify_matching_structure_type_and_version(feagi_byte_structure: &FeagiByteStructure, expected_type: FeagiByteStructureType, expected_version: u8) -> Result<(), FeagiDataError> {
        if feagi_byte_structure.try_get_structure_type()? != expected_type {
            return Err(FeagiDataError::DeserializationError(format!(
                "Given structure of type {} cannot be instantiated for entity corresponding to type {}!", feagi_byte_structure.try_get_structure_type()? as u8, expected_type as u8)).into());
        }
        if feagi_byte_structure.try_get_version()? != expected_version {
            return Err(FeagiDataError::DeserializationError(format!(
                "Given structure of version {} cannot be instantiated for entity corresponding to version {}!", feagi_byte_structure.try_get_version()?, expected_version)).into());
        }
        Ok(())
    }
    //endregion
    
    //region Get Properties
    
    // NOTE: These functions have safety checks as they can be called externally
    
    /// Returns the format type of this bytes structure.
    ///
    /// Extracts and validates the format type identifier from the first bytes
    /// of the structure. This is used to determine how to interpret the
    /// remaining data in the structure.
    ///
    /// # Returns
    /// * `Ok(FeagiByteStructureType)` - The format type of this structure
    /// * `Err(FeagiDataError)` - If the structure is empty or contains an invalid type
    pub fn try_get_structure_type(&self) -> Result<FeagiByteStructureType, FeagiDataError> {
        if self.bytes.len() == 0 {
            return Err(FeagiDataError::InternalError("Empty bytes structure!".to_string())); // This shouldn't be possible as this struct should be checked before being created
        }
        FeagiByteStructureType::try_from(self.bytes[0])
    }
    
    /// Returns the format version of this bytes structure.
    ///
    /// Extracts the version number from the second bytes of the structure.
    /// Version numbers allow for format evolution while maintaining compatibility.
    ///
    /// # Returns
    /// * `Ok(u8)` - The version number of this structure's format
    /// * `Err(FeagiDataError)` - If the structure is too short to contain version information
    pub fn try_get_version(&self) -> Result<u8, FeagiDataError> {
        if self.bytes.len() < 2 {
            return Err(FeagiDataError::InternalError("Unable to get version information! Byte struct is too short!".into()))
        }
        Ok(self.bytes[1])
    }
    
    /// Checks if this bytes structure is a multi-structure container.
    ///
    /// Multi-structure wrapped_io_data can hold multiple individual FEAGI structures
    /// in a single bytes stream, each with its own format type and data.
    ///
    /// # Returns
    /// * `Ok(true)` - This is a multi-structure container
    /// * `Ok(false)` - This is a single structure
    /// * `Err(FeagiDataError)` - If the structure type cannot be determined
    pub fn is_multistruct(&self) -> Result<bool, FeagiDataError> {
        Ok(FeagiByteStructureType::MultiStructHolder == self.try_get_structure_type()?)
    }
    
    /// Returns the number of individual structures contained in this bytes structure.
    ///
    /// For single structures, this always returns 1. For multi-structure wrapped_io_data,
    /// this returns the number of individual structures contained within.
    ///
    /// # Returns
    /// * `Ok(usize)` - The number of contained structures (always ≥ 1)
    /// * `Err(FeagiDataError)` - If structure validation fails
    pub fn contained_structure_count(&self) -> Result<usize, FeagiDataError> {
        if self.is_multistruct()? {
            self.verify_valid_multistruct_internal_count()?;
            return Ok(self.get_multistruct_contained_count());
        }
        Ok(1) // if not a multistruct, there's only one struct
    }
    
    pub fn get_ordered_object_types(&self) -> Result<Vec<FeagiByteStructureType>, FeagiDataError> {
        if self.is_multistruct()? {
            self.verify_valid_multistruct_internal_count()?;
            self.verify_valid_multistruct_internal_positionings_header()?;
            
            let struct_slices = self.get_all_multistruct_internal_slices();
            self.verify_valid_multistruct_internal_slices_header_and_size(&struct_slices)?;
            
            let mut output: Vec<FeagiByteStructureType> = Vec::with_capacity(struct_slices.len());
            for slice in struct_slices {
                output.push(FeagiByteStructureType::try_from(slice[0])?);
            };
            return Ok(output);
        }
        Ok(vec![self.try_get_structure_type()?])
    }

    pub fn copy_out_single_byte_structure_from_multistruct(&self, index: usize) -> Result<FeagiByteStructure, FeagiDataError> {
        if !self.is_multistruct()? {
            return Ok(self.clone());
        }
        if index > self.contained_structure_count()? {
            return Err(FeagiDataError::BadParameters(format!("Given struct index {} is out of bounds given this multistruct only contains {} elements!", index, self.contained_structure_count()?)).into());
        }
        Ok(FeagiByteStructure::create_from_bytes(
            self.get_multistruct_specific_slice(index).to_vec()
        )?)
    }
    
    /// Extracts the original object from a single (non-multi-structure) bytes structure.
    ///
    /// This method deserializes the contained data back to its original object form
    /// using the appropriate deserialization logic based on the structure's format type.
    /// Only works with single structures - use `copy_out_single_object_from_multistruct()`
    /// for multi-structure wrapped_io_data.
    ///
    /// # Returns
    /// * `Ok(Box<dyn FeagiByteStructureCompatible>)` - The deserialized original object
    /// * `Err(FeagiDataError)` - If:
    ///   - This is a multi-structure container (use the multistruct variant instead)
    ///   - Deserialization fails due to corrupted data
    ///   - Unsupported format type
    ///
    /// # Supported Format Types
    /// - `JSON` → `JsonStructure`
    /// - `NeuronCategoricalXYZP` → `CorticalMappedXYZPNeuronData`
    pub fn copy_out_single_object_from_single_struct(&self) -> Result<Box<dyn FeagiByteStructureCompatible>, FeagiDataError> {
        let this_struct_type = self.try_get_structure_type()?;
        if this_struct_type == FeagiByteStructureType::MultiStructHolder {
            return Err(FeagiDataError::BadParameters("Cannot return a multistruct holding multiple structs as a single object!".into()).into())
        }
        
        // Factory pattern to create the appropriate concrete type based on structure type
        match this_struct_type {
            FeagiByteStructureType::JSON => {
                Ok(Box::new(FeagiJSON::new_from_feagi_byte_structure(self)?))
            },
            FeagiByteStructureType::NeuronCategoricalXYZP => {
                Ok(Box::new(CorticalMappedXYZPNeuronData::new_from_feagi_byte_structure(self)?))
            },
            FeagiByteStructureType::MultiStructHolder => {
                // This case is already handled above, but included for completeness
                Err(FeagiDataError::BadParameters("Cannot return a multistruct holding multiple structs as a single object!".into()).into())
            }
            //_ => {
            //    Err(FeagiDataError::InternalError(format!("Missing export definition for FBS object type {}!", this_struct_type)))
            //}
        }
    }
    
    pub fn copy_out_single_object_from_multistruct(&self, index: usize) -> Result<Box<dyn FeagiByteStructureCompatible>, FeagiDataError> {
        // TODO this method is slow, we should have a dedicated create from bytes slice for FeagiByteStructureCompatible
        if !self.is_multistruct()? {
            return Err(FeagiDataError::DeserializationError("Cannot deserialize this object as a multistruct when it is not!".into()).into())
        }
        let internal = self.copy_out_single_byte_structure_from_multistruct(index)?;
        internal.copy_out_single_object_from_single_struct()
    }

    pub fn copy_out_as_byte_vector(&self) -> Vec<u8> {
        self.bytes.clone()
    }
    
    //endregion
    
    //region Verifications
    // NOTE: These functions are used to ensure internal data is reasonable. Not all have all
    // safety checks/
    
    fn verify_valid_multistruct_internal_count(&self) -> Result<(), FeagiDataError> {
        let len = self.bytes.len();
        if len < Self::MINIMUM_LENGTH_TO_BE_CONSIDERED_VALID {
            return Err(FeagiDataError::InternalError("bytes structure too short!".into()))
        }
        if self.bytes[0] != FeagiByteStructureType::MultiStructHolder as u8 { // faster header check
            return Err(FeagiDataError::DeserializationError("Byte structure is not identified as a multistruct!".into()).into())
        }
        if self.bytes[2] == 0 {
            return Err(FeagiDataError::InternalError("Multistruct reports 0 contained structures!".into()))
        }
        Ok(())
    }
    
    fn verify_valid_multistruct_internal_positionings_header(&self) -> Result<(), FeagiDataError> {
        // We are assuming the internal structure count was already verified as existing and valid ( not 0)
        let len = self.bytes.len();
        let contained_struct_count = self.bytes[2] as usize;
        if contained_struct_count == 0 {
            return Err(FeagiDataError::InternalError("Multistruct reports 0 contained structures!".into())); // Explicitly check for this again because if we dont, we are going to underflow below
        }
        let header_size_bytes = contained_struct_count * Self::MULTISTRUCT_PER_STRUCT_HEADER_SIZE_IN_BYTES;
        if len <  Self::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + Self::MULTISTRUCT_STRUCT_COUNT_BYTE_SIZE + header_size_bytes {
            return Err(FeagiDataError::InternalError("Multi Struct too short to hold contained positionings header!".into()))
        } 
        
        let len = len as u32;
        let mut struct_header_start_index = Self::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + Self::MULTISTRUCT_STRUCT_COUNT_BYTE_SIZE;
        for _ in 0..contained_struct_count {
            let struct_start_index = LittleEndian::read_u32(&self.bytes[struct_header_start_index..struct_header_start_index + 4]);
            let struct_length = LittleEndian::read_u32(&self.bytes[struct_header_start_index + 4..struct_header_start_index + 8]);
            if struct_start_index + struct_length > len {
                return Err(FeagiDataError::InternalError("Multi Struct too short to hold all reported contained structures!".into()))
            }
            struct_header_start_index += Self::MULTISTRUCT_PER_STRUCT_HEADER_SIZE_IN_BYTES;
        }
        Ok(())
    }
    
    fn verify_valid_multistruct_internal_slices_header_and_size(&self, internal_slices: &Vec<&[u8]>) -> Result<(), FeagiDataError> {
        for slice in internal_slices {
            if slice.len() < Self::MINIMUM_LENGTH_TO_BE_CONSIDERED_VALID {
                return Err(FeagiDataError::InternalError("Multi Struct contains a internal structure too small to be valid!".into()))
            }
            FeagiByteStructureType::try_from(slice[0])?;
        }
        Ok(())
    }
    
    //endregion
    
    //region Borrow Data

    /// Returns a read-only reference to the internal bytes data.
    ///
    /// Provides direct access to the underlying bytes array for reading.
    /// Useful for efficient transmission or when you need to work with
    /// the raw bytes format directly.
    ///
    /// # Returns
    /// Read-only slice containing the complete bytes structure data
    pub fn borrow_data_as_slice(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns a mutable reference to the internal bytes data.
    ///
    /// Provides direct write access to the underlying bytes array.
    /// **Caution**: Modifying the data directly can invalidate the structure
    /// and cause deserialization failures. Use with care.
    ///
    /// # Returns
    /// Mutable slice for direct bytes manipulation
    ///
    /// # Safety
    /// Direct modification bypasses validation. Ensure any changes maintain
    /// proper FEAGI bytes structure format compliance.
    pub fn borrow_data_as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    /// Returns a mutable reference to the internal bytes vector.
    ///
    /// Provides full access to the underlying Vec<u8> for advanced operations
    /// like resizing or bulk modifications. **Caution**: Direct modifications
    /// can break format compliance.
    ///
    /// # Returns
    /// Mutable reference to the internal bytes vector
    ///
    /// # Safety
    /// Direct vector manipulation bypasses all validation. Only use when you
    /// understand the FEAGI bytes structure format requirements.
    pub fn borrow_data_as_mut_vec(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }
    
    //endregion
    
    //region Interactions with Internal Vector

    pub fn get_wasted_capacity_count(&self) -> usize {
        self.bytes.capacity() - self.bytes.len()
    }

    pub fn get_utilized_capacity_percentage(&self) -> f32 {
        (self.bytes.len() as f32 / self.bytes.capacity() as f32) * 100.0
    }

    pub fn ensure_capacity_of_at_least(&mut self, size: usize) -> Result<(), FeagiDataError> {
        if size < Self::MINIMUM_LENGTH_TO_BE_CONSIDERED_VALID {
            return Err(FeagiDataError::BadParameters(format!("Cannot set capacity to less than minimum required capacity of {}!", Self::MINIMUM_LENGTH_TO_BE_CONSIDERED_VALID)).into());
        }

        if self.bytes.capacity() < size {
            //self.bytes.reserve(size - self.bytes.capacity());
        }
        Ok(())
    }

    pub fn shed_wasted_capacity(&mut self) {
        self.bytes.shrink_to_fit();
    }

    pub fn reset_write_index(&mut self) {
        self.bytes.truncate(0);
    }
    
    //endregion
    
    //region Internals
    // WARNING: Most of these functions do not check for bytes structure validity, be cautious

    fn build_multistruct_from_slices(all_slices: Vec<&[u8]>) -> FeagiByteStructure {
        // NOTE: does not check if internal slices are sensible
        let slice_count = all_slices.len();
        let mut total_slices_byte_count: usize = 0;
        for slice in &all_slices {
            total_slices_byte_count += slice.len();
        };
        let header_output_length = Self::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + Self::MULTISTRUCT_STRUCT_COUNT_BYTE_SIZE +
            (Self::MULTISTRUCT_PER_STRUCT_HEADER_SIZE_IN_BYTES * slice_count);
        
        let total_output_length = header_output_length + total_slices_byte_count;
        
        // Write output data
        let mut output_bytes: Vec<u8> = Vec::with_capacity(total_output_length);
        output_bytes.resize(total_output_length, 0);
        
        // global header
        output_bytes[0] = FeagiByteStructureType::MultiStructHolder as u8;
        output_bytes[1] = Self::SUPPORTED_VERSION_MULTI_STRUCT;
        
        // struct count subheader
        output_bytes[2] = slice_count as u8;
        
        // subheader + data
        let mut subheader_write_index: usize =  Self::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + Self::MULTISTRUCT_STRUCT_COUNT_BYTE_SIZE;
        let mut data_write_index: usize = header_output_length; // start right after header
        
        for slice in &all_slices {
            let slice_length = slice.len();
            
            // sub header
            output_bytes[subheader_write_index.. subheader_write_index + 4].copy_from_slice(
                &(data_write_index as u32).to_le_bytes() // location
            );
            output_bytes[subheader_write_index + 4.. subheader_write_index + 8].copy_from_slice(
                &(slice_length as u32).to_le_bytes() // length
            );
            
            // data
            output_bytes[data_write_index..data_write_index + slice_length].copy_from_slice(
                slice
            );

            subheader_write_index += Self::MULTISTRUCT_PER_STRUCT_HEADER_SIZE_IN_BYTES;
            data_write_index += slice_length;
        };
        
        // Skip any checks and instantiate directly
        FeagiByteStructure {bytes: output_bytes}
    }
    
    fn get_multistruct_contained_count(&self) -> usize {
        // NOTE no safety checks, make sure your vector is a valid multistruct
        self.bytes[2] as usize
    }

    fn get_multistruct_specific_slice(&self, index: usize) -> &[u8] {
        // WARNING: No boundary checks, be careful!
        let reading_offset = Self::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + Self::MULTISTRUCT_STRUCT_COUNT_BYTE_SIZE +
            (index * Self::MULTISTRUCT_PER_STRUCT_HEADER_SIZE_IN_BYTES);
        let struct_start = LittleEndian::read_u32(&self.bytes[reading_offset..reading_offset + 4]) as usize;
        let struct_length = LittleEndian::read_u32(&self.bytes[reading_offset + 4..reading_offset + 8]) as usize;
        &self.bytes[struct_start..struct_start + struct_length]
    }
    
    fn get_all_multistruct_internal_slices(&self) -> Vec<&[u8]> {
        let mut output = Vec::with_capacity(self.get_multistruct_contained_count());
        for i in 0..self.get_multistruct_contained_count() {
            output.push(self.get_multistruct_specific_slice(i));
        }
        output
    }
    
    
    //endregion
    
}

/// Extracts the version number from raw bytes data without full validation.
///
/// This utility function reads the version number from the second bytes of raw
/// bytes data that represents a FEAGI bytes structure. It performs minimal validation
/// (length check only) and is useful when you need just the version without
/// creating a full FeagiByteStructure instance.
///
/// # Arguments
/// * `bytes` - Raw bytes data containing a FEAGI bytes structure
///
/// # Returns
/// * `Ok(u8)` - The version number from the structure header
/// * `Err(FeagiDataError)` - If the bytes array is too short to contain a valid header
///
/// # Performance
/// This function is more efficient than creating a full FeagiByteStructure when you
/// only need the version information, as it skips comprehensive validation.
pub fn try_get_version_from_bytes(bytes: &[u8]) -> Result<u8, FeagiDataError> {
    if bytes.len() <  FeagiByteStructure::MINIMUM_LENGTH_TO_BE_CONSIDERED_VALID {
        return Err(FeagiDataError::DeserializationError("Structure too short to be a Feagi Byte Structure".into()).into());
    }
    Ok(bytes[1])
}