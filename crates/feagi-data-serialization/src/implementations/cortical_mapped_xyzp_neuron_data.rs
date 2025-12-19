//! Serialization implementation for cortical-mapped neuron voxel xyzp data.

use crate::{FeagiByteContainer, FeagiByteStructureType, FeagiSerializable};
use byteorder::{ByteOrder, LittleEndian};
use feagi_data_structures::genomic::cortical_area::CorticalID;
use feagi_data_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZP, NeuronVoxelXYZPArrays,
};
use feagi_data_structures::FeagiDataError;
use std::any::Any;

/// Current version of the neuron XYZP serialization format.
const BYTE_STRUCT_VERSION: u8 = 1;

/// Bytes per cortical ID header: 8 (ID) + 4 (start index) + 4 (byte count).
const NUMBER_BYTES_PER_CORTICAL_ID_HEADER: usize =
    CorticalID::NUMBER_OF_BYTES + size_of::<u32>() + size_of::<u32>();

/// Bytes for cortical area count header.
const NUMBER_BYTES_CORTICAL_COUNT_HEADER: usize = size_of::<u16>();

impl FeagiSerializable for CorticalMappedXYZPNeuronVoxels {
    fn get_type(&self) -> FeagiByteStructureType {
        FeagiByteStructureType::NeuronCategoricalXYZP
    }

    fn get_version(&self) -> u8 {
        BYTE_STRUCT_VERSION
    }

    fn get_number_of_bytes_needed(&self) -> usize {
        let mut number_bytes_needed: usize =
            FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT + NUMBER_BYTES_CORTICAL_COUNT_HEADER;
        for neuron_data in self.iter() {
            number_bytes_needed +=
                neuron_data.get_size_in_number_of_bytes() + NUMBER_BYTES_PER_CORTICAL_ID_HEADER;
        }
        number_bytes_needed
    }

    fn try_serialize_struct_to_byte_slice(
        &self,
        byte_destination: &mut [u8],
    ) -> Result<(), FeagiDataError> {
        // write per struct header
        byte_destination[0] = self.get_type() as u8;
        byte_destination[1] = self.get_version();

        // Initial Section Header
        let number_cortical_areas: usize = self.mappings.len();
        const NUMBER_BYTES_INITIAL_SECTION_HEADER: usize = size_of::<u16>();
        LittleEndian::write_u16(
            &mut byte_destination[FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT
                ..FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT
                    + NUMBER_BYTES_INITIAL_SECTION_HEADER],
            number_cortical_areas as u16,
        );

        // Write Cortical Secondary Header and Neuron Data Together in a single loop
        let mut subheader_write_index: usize =
            FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT + NUMBER_BYTES_INITIAL_SECTION_HEADER;
        let mut neuron_data_write_index: usize =
            subheader_write_index + (number_cortical_areas * NUMBER_BYTES_PER_CORTICAL_ID_HEADER);

        for (cortical_id, neuron_data) in &self.mappings {
            // Write cortical subheader
            let cortical_area_lookup_header_slice = &mut byte_destination
                [subheader_write_index..subheader_write_index + CorticalID::NUMBER_OF_BYTES];
            let cortical_area_lookup_header_slice: &mut [u8; CorticalID::NUMBER_OF_BYTES] =
                cortical_area_lookup_header_slice.try_into().unwrap();
            cortical_id.write_id_to_bytes(cortical_area_lookup_header_slice);

            let reading_length: u32 = neuron_data.get_size_in_number_of_bytes() as u32;
            LittleEndian::write_u32(
                &mut byte_destination[subheader_write_index + CorticalID::NUMBER_OF_BYTES
                    ..subheader_write_index + CorticalID::NUMBER_OF_BYTES + size_of::<u32>()],
                neuron_data_write_index as u32,
            );
            LittleEndian::write_u32(
                &mut byte_destination[subheader_write_index
                    + CorticalID::NUMBER_OF_BYTES
                    + size_of::<u32>()
                    ..subheader_write_index
                        + CorticalID::NUMBER_OF_BYTES
                        + size_of::<u32>()
                        + size_of::<u32>()],
                reading_length,
            );

            // write neuron data
            write_neuron_array_to_bytes(
                neuron_data,
                &mut byte_destination
                    [neuron_data_write_index..(neuron_data_write_index + reading_length as usize)],
            )?;

            // update indexes
            subheader_write_index += NUMBER_BYTES_PER_CORTICAL_ID_HEADER;
            neuron_data_write_index += reading_length as usize;
        }

        Ok(())
    }

    fn try_deserialize_and_update_self_from_byte_slice(
        &mut self,
        byte_reading: &[u8],
    ) -> Result<(), FeagiDataError> {
        // Assuming type is correct
        self.verify_byte_slice_is_of_correct_version(byte_reading)?;
        self.clear_neurons_only(); // This causes a memory leak. Too Bad!

        let number_cortical_areas: usize = LittleEndian::read_u16(&byte_reading[2..4]) as usize;
        let mut reading_header_byte_index: usize =
            FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT + NUMBER_BYTES_CORTICAL_COUNT_HEADER;

        for _cortical_index in 0..number_cortical_areas {
            let cortical_id = CorticalID::try_from_bytes(
                <&[u8; CorticalID::NUMBER_OF_BYTES]>::try_from(
                    &byte_reading[reading_header_byte_index
                        ..reading_header_byte_index + CorticalID::NUMBER_OF_BYTES],
                )
                .unwrap(),
            )?;

            const CORTICAL_ID_AND_U32_OFFSET: usize =
                size_of::<u32>() + CorticalID::NUMBER_OF_BYTES;
            const CORTICAL_ID_AND_U32_AND_U32_OFFSET: usize =
                size_of::<u32>() + size_of::<u32>() + CorticalID::NUMBER_OF_BYTES;

            let data_start_reading: usize = LittleEndian::read_u32(
                &byte_reading[reading_header_byte_index + CorticalID::NUMBER_OF_BYTES
                    ..reading_header_byte_index + CORTICAL_ID_AND_U32_OFFSET],
            ) as usize;
            let number_bytes_to_read: usize = LittleEndian::read_u32(
                &byte_reading[reading_header_byte_index + CORTICAL_ID_AND_U32_OFFSET
                    ..reading_header_byte_index + CORTICAL_ID_AND_U32_AND_U32_OFFSET],
            ) as usize;

            if byte_reading.len() < data_start_reading + number_bytes_to_read {
                return Err(FeagiDataError::SerializationError("Byte structure for NeuronCategoricalXYZP is too short to fit the data the header says it contains!".into()));
            }

            let neuron_bytes =
                &byte_reading[data_start_reading..data_start_reading + number_bytes_to_read];
            let bytes_length = neuron_bytes.len();

            if bytes_length % NeuronVoxelXYZP::NUMBER_BYTES_PER_NEURON != 0 {
                return Err(FeagiDataError::SerializationError("As NeuronXYCPArrays contains 4 internal arrays of equal length, each of elements of 4 bytes each (uint32 and float), the input bytes array must be divisible by 16!".into()).into());
            }

            let x_end = bytes_length / 4; // q1
            let y_end = bytes_length / 2; // q2
            let z_end = x_end * 3; // q3

            let num_neurons = bytes_length / NeuronVoxelXYZP::NUMBER_BYTES_PER_NEURON;
            let neuron_array = self.ensure_clear_and_borrow_mut(&cortical_id);
            neuron_array.ensure_capacity(num_neurons);

            // TODO this could potentially be parallelized
            for i in 0..num_neurons {
                let x_start = i * 4;
                let y_start = x_end + x_start;
                let z_start = y_end + x_start;
                let p_start = z_end + x_start;

                neuron_array.push_raw(
                    LittleEndian::read_u32(&neuron_bytes[x_start..x_start + 4]),
                    LittleEndian::read_u32(&neuron_bytes[y_start..y_start + 4]),
                    LittleEndian::read_u32(&neuron_bytes[z_start..z_start + 4]),
                    LittleEndian::read_f32(&neuron_bytes[p_start..p_start + 4]),
                )
            }
            reading_header_byte_index += NUMBER_BYTES_PER_CORTICAL_ID_HEADER;
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Serializes a neuron voxel array to bytes in structure-of-arrays format.
///
/// Writes X, Y, Z coordinates and potential values as separate contiguous blocks
/// for memory efficiency and cache locality during deserialization.
#[inline]
fn write_neuron_array_to_bytes(
    neuron_array: &NeuronVoxelXYZPArrays,
    bytes_to_write_to: &mut [u8],
) -> Result<(), FeagiDataError> {
    const U32_F32_LENGTH: usize = 4;
    let number_of_neurons_to_write: usize = neuron_array.len();
    let number_bytes_needed = neuron_array.get_size_in_number_of_bytes();
    if bytes_to_write_to.len() != number_bytes_needed {
        return Err(FeagiDataError::SerializationError(
            format!(
                "Need exactly {} bytes to write xyzp neuron data, but given a space of {} bytes!",
                bytes_to_write_to.len(),
                number_bytes_needed
            ),
        ));
    }

    let x_offset: usize = 0;
    let y_offset = number_of_neurons_to_write * U32_F32_LENGTH; // quarter way through the total bytes
    let z_offset = number_of_neurons_to_write * U32_F32_LENGTH * 2; // halfway through the total bytes
    let p_offset = number_of_neurons_to_write * U32_F32_LENGTH * 3; // three quarters way through the total bytes

    let (x, y, z, p) = neuron_array.borrow_xyzp_vectors();

    // OPTIMIZATION: Use bulk memory operations for little-endian systems (x86_64, ARM64)
    #[cfg(target_endian = "little")]
    {
        let x_len = x.len() * U32_F32_LENGTH;
        let y_len = y.len() * U32_F32_LENGTH;
        let z_len = z.len() * U32_F32_LENGTH;
        let p_len = p.len() * U32_F32_LENGTH;

        // Use direct pointer copies for maximum performance
        // These will use optimized memcpy (often SIMD-accelerated) internally
        unsafe {
            std::ptr::copy_nonoverlapping(
                x.as_ptr() as *const u8,
                bytes_to_write_to.as_mut_ptr().add(x_offset),
                x_len,
            );
            std::ptr::copy_nonoverlapping(
                y.as_ptr() as *const u8,
                bytes_to_write_to.as_mut_ptr().add(y_offset),
                y_len,
            );
            std::ptr::copy_nonoverlapping(
                z.as_ptr() as *const u8,
                bytes_to_write_to.as_mut_ptr().add(z_offset),
                z_len,
            );
            std::ptr::copy_nonoverlapping(
                p.as_ptr() as *const u8,
                bytes_to_write_to.as_mut_ptr().add(p_offset),
                p_len,
            );
        }
    }

    #[cfg(not(target_endian = "little"))]
    {
        // Fallback for big-endian systems: use individual writes with endianness conversion
        let mut x_off = x_offset;
        let mut y_off = y_offset;
        let mut z_off = z_offset;
        let mut p_off = p_offset;

        for i in 0..number_of_neurons_to_write {
            LittleEndian::write_u32(&mut bytes_to_write_to[x_off..x_off + U32_F32_LENGTH], x[i]);
            LittleEndian::write_u32(&mut bytes_to_write_to[y_off..y_off + U32_F32_LENGTH], y[i]);
            LittleEndian::write_u32(&mut bytes_to_write_to[z_off..z_off + U32_F32_LENGTH], z[i]);
            LittleEndian::write_f32(&mut bytes_to_write_to[p_off..p_off + U32_F32_LENGTH], p[i]);

            x_off += U32_F32_LENGTH;
            y_off += U32_F32_LENGTH;
            z_off += U32_F32_LENGTH;
            p_off += U32_F32_LENGTH;
        }
    }

    Ok(())
}
