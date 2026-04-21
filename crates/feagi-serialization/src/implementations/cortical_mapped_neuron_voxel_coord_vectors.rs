//! Serialization implementation for [`CorticalMappedNeuronVoxelCoordVectors`].
//!
//! This is the quantization-generic replacement for the pre-refactor
//! `CorticalMappedXYZPNeuronVoxels` serialization. The on-wire per-voxel width
//! depends on the concrete quantization parameters (`C::NUMBER_OF_BYTES` per
//! coordinate axis and `V::NUMBER_OF_BYTES` per potential); producer and
//! consumer must agree on the same parameterization.
//!
//! # Wire format (version 2)
//!
//! ```text
//! [0]   = structure type byte (FeagiByteStructureType::NeuronCategoricalXYZP = 11)
//! [1]   = structure version byte (BYTE_STRUCT_VERSION = 2)
//! [2..4]              = u16 LE  number of cortical areas
//! Per cortical area header (repeated for each CA):
//!   [..CorticalID::NUMBER_OF_BYTES]       cortical id bytes
//!   [..u32 LE]                            data-section start offset (absolute, from slice start)
//!   [..u32 LE]                            data-section byte count
//! Per-voxel data (AoS, repeated):
//!   [..C::NUMBER_OF_BYTES]  x LE
//!   [..C::NUMBER_OF_BYTES]  y LE
//!   [..C::NUMBER_OF_BYTES]  z LE
//!   [..V::NUMBER_OF_BYTES]  p LE
//! ```
//!
//! Note: the pre-refactor format (version 1) used a structure-of-arrays (SoA)
//! layout with hardcoded u32 coordinates and f32 potentials. Version 2 switches
//! to array-of-structures (AoS) to make quant-generic serialization tractable
//! with per-element trait methods (`write_le_bytes`/`read_le_bytes`) instead of
//! type-erased bulk memcpy.

use crate::{FeagiByteContainer, FeagiByteStructureType, FeagiSerializable};
use byteorder::{ByteOrder, LittleEndian};
use feagi_structures::base_quantizable::{QuantizableUIntType, QuantizableValueType};
use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::neuron_voxels::coord_potential::CorticalMappedNeuronVoxelCoordVectors;
use feagi_structures::neuron_voxels::descriptors::{
    NeuronVoxelCoordinate, NeuronVoxelPotential,
};
use feagi_structures::neuron_voxels::traits::{
    SingleCorticalNeuronVoxelCollectionAlloc, SingleCorticalNeuronVoxelCollectionSparse,
};
use feagi_structures::FeagiStructuresError;
use std::any::Any;

/// Current version of the neuron XYZP serialization format.
///
/// Bumped to 2 when the wire format was generalized over quantization; v1 used
/// hardcoded u32/f32 SoA layout.
const BYTE_STRUCT_VERSION: u8 = 2;

/// Bytes per cortical ID header on the wire: 8 (ID) + 4 (start offset) + 4 (byte count).
const NUMBER_BYTES_PER_CORTICAL_ID_HEADER: usize =
    CorticalID::NUMBER_OF_BYTES + size_of::<u32>() + size_of::<u32>();

/// Bytes for the cortical-area-count field in the header.
const NUMBER_BYTES_CORTICAL_COUNT_HEADER: usize = size_of::<u16>();

impl<V, C, N, A> FeagiSerializable for CorticalMappedNeuronVoxelCoordVectors<V, C, N, A>
where
    V: QuantizableValueType,
    C: QuantizableUIntType,
    N: QuantizableUIntType,
    A: QuantizableUIntType,
{
    fn get_type(&self) -> FeagiByteStructureType {
        FeagiByteStructureType::NeuronCategoricalXYZP
    }

    fn get_version(&self) -> u8 {
        BYTE_STRUCT_VERSION
    }

    fn get_number_of_bytes_needed(&self) -> usize {
        let per_voxel_bytes = 3 * C::NUMBER_OF_BYTES + V::NUMBER_OF_BYTES;

        let mut total = FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT
            + NUMBER_BYTES_CORTICAL_COUNT_HEADER;
        for (_cortical_id, collection) in self.iter() {
            let num_voxels = collection.get_number_neuron_voxel_contained_count().to_usize();
            total += NUMBER_BYTES_PER_CORTICAL_ID_HEADER + num_voxels * per_voxel_bytes;
        }
        total
    }

    fn try_serialize_struct_to_byte_slice(
        &self,
        byte_destination: &mut [u8],
    ) -> Result<(), FeagiStructuresError> {
        let coord_bytes = C::NUMBER_OF_BYTES;
        let value_bytes = V::NUMBER_OF_BYTES;
        let per_voxel_bytes = 3 * coord_bytes + value_bytes;

        let expected_total = self.get_number_of_bytes_needed();
        if byte_destination.len() != expected_total {
            return Err(FeagiStructuresError::SerializationError(format!(
                "CorticalMappedNeuronVoxelCoordVectors serialization needs exactly {} bytes, \
                 got slice of length {}",
                expected_total,
                byte_destination.len()
            )));
        }

        byte_destination[0] = self.get_type() as u8;
        byte_destination[1] = self.get_version();

        let number_cortical_areas: usize = self.len();
        if number_cortical_areas > u16::MAX as usize {
            return Err(FeagiStructuresError::SerializationError(format!(
                "Too many cortical areas to serialize: {} exceeds u16::MAX ({})",
                number_cortical_areas,
                u16::MAX
            )));
        }
        LittleEndian::write_u16(
            &mut byte_destination[FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT
                ..FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT
                    + NUMBER_BYTES_CORTICAL_COUNT_HEADER],
            number_cortical_areas as u16,
        );

        let mut subheader_write_index: usize = FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT
            + NUMBER_BYTES_CORTICAL_COUNT_HEADER;
        let mut neuron_data_write_index: usize =
            subheader_write_index + number_cortical_areas * NUMBER_BYTES_PER_CORTICAL_ID_HEADER;

        for (cortical_id, collection) in self.iter() {
            let num_voxels = collection.get_number_neuron_voxel_contained_count().to_usize();
            let data_byte_count = num_voxels * per_voxel_bytes;

            let id_slice: &mut [u8; CorticalID::NUMBER_OF_BYTES] = (&mut byte_destination
                [subheader_write_index..subheader_write_index + CorticalID::NUMBER_OF_BYTES])
                .try_into()
                .unwrap();
            cortical_id.write_id_to_bytes(id_slice);

            LittleEndian::write_u32(
                &mut byte_destination[subheader_write_index + CorticalID::NUMBER_OF_BYTES
                    ..subheader_write_index + CorticalID::NUMBER_OF_BYTES + size_of::<u32>()],
                neuron_data_write_index as u32,
            );
            LittleEndian::write_u32(
                &mut byte_destination[subheader_write_index
                    + CorticalID::NUMBER_OF_BYTES
                    + size_of::<u32>()
                    ..subheader_write_index + NUMBER_BYTES_PER_CORTICAL_ID_HEADER],
                data_byte_count as u32,
            );

            let mut voxel_offset = neuron_data_write_index;
            for (coord, potential) in collection.iter_coordinate() {
                coord
                    .x
                    .write_le_bytes(&mut byte_destination[voxel_offset..voxel_offset + coord_bytes]);
                coord.y.write_le_bytes(
                    &mut byte_destination[voxel_offset + coord_bytes..voxel_offset + 2 * coord_bytes],
                );
                coord.z.write_le_bytes(
                    &mut byte_destination
                        [voxel_offset + 2 * coord_bytes..voxel_offset + 3 * coord_bytes],
                );
                potential.write_le_bytes(
                    &mut byte_destination[voxel_offset + 3 * coord_bytes..voxel_offset + per_voxel_bytes],
                );
                voxel_offset += per_voxel_bytes;
            }

            subheader_write_index += NUMBER_BYTES_PER_CORTICAL_ID_HEADER;
            neuron_data_write_index += data_byte_count;
        }

        Ok(())
    }

    fn try_deserialize_and_update_self_from_byte_slice(
        &mut self,
        byte_reading: &[u8],
    ) -> Result<(), FeagiStructuresError> {
        self.verify_byte_slice_is_of_correct_type(byte_reading)?;
        self.verify_byte_slice_is_of_correct_version(byte_reading)?;

        let coord_bytes = C::NUMBER_OF_BYTES;
        let value_bytes = V::NUMBER_OF_BYTES;
        let per_voxel_bytes = 3 * coord_bytes + value_bytes;

        let header_end =
            FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT + NUMBER_BYTES_CORTICAL_COUNT_HEADER;
        if byte_reading.len() < header_end {
            return Err(FeagiStructuresError::DeserializationError(format!(
                "CorticalMappedNeuronVoxelCoordVectors deserialization: slice too short ({} bytes) \
                 to contain cortical-count header ({} bytes)",
                byte_reading.len(),
                header_end
            )));
        }

        let number_cortical_areas: usize = LittleEndian::read_u16(
            &byte_reading[FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT..header_end],
        ) as usize;

        // NOTE: deserialization contract — the caller MUST pre-populate `self` with
        // one entry per expected cortical ID whose `NeuronVoxelCoordVector` is
        // initialized to the correct `NeuronVoxelDimensions`. The wire format does
        // not carry per-cortical dimensions, so we cannot synthesize them here;
        // instead we locate each existing entry by id, clear its voxels, and push
        // the incoming voxels into it (preserving dimensions). Any incoming id
        // missing from `self` is an error.

        let mut reading_header_byte_index: usize = header_end;

        for _cortical_index in 0..number_cortical_areas {
            let header_end_index = reading_header_byte_index + NUMBER_BYTES_PER_CORTICAL_ID_HEADER;
            if byte_reading.len() < header_end_index {
                return Err(FeagiStructuresError::DeserializationError(
                    "CorticalMappedNeuronVoxelCoordVectors deserialization: slice too short to \
                     read cortical-area header"
                        .into(),
                ));
            }

            let cortical_id_slice: &[u8; CorticalID::NUMBER_OF_BYTES] = (&byte_reading
                [reading_header_byte_index
                    ..reading_header_byte_index + CorticalID::NUMBER_OF_BYTES])
                .try_into()
                .unwrap();
            let cortical_id = CorticalID::try_from_bytes(cortical_id_slice).map_err(|e| {
                FeagiStructuresError::DeserializationError(format!(
                    "Failed to parse CorticalID from bytes: {:?}",
                    e
                ))
            })?;

            const ID_PLUS_U32: usize = CorticalID::NUMBER_OF_BYTES + size_of::<u32>();
            let data_start_reading: usize = LittleEndian::read_u32(
                &byte_reading[reading_header_byte_index + CorticalID::NUMBER_OF_BYTES
                    ..reading_header_byte_index + ID_PLUS_U32],
            ) as usize;
            let number_bytes_to_read: usize = LittleEndian::read_u32(
                &byte_reading[reading_header_byte_index + ID_PLUS_U32..header_end_index],
            ) as usize;

            if byte_reading.len() < data_start_reading.saturating_add(number_bytes_to_read) {
                return Err(FeagiStructuresError::DeserializationError(format!(
                    "CorticalMappedNeuronVoxelCoordVectors deserialization: payload for cortical \
                     area starting at offset {} with byte count {} overruns slice of length {}",
                    data_start_reading,
                    number_bytes_to_read,
                    byte_reading.len()
                )));
            }

            if number_bytes_to_read % per_voxel_bytes != 0 {
                return Err(FeagiStructuresError::DeserializationError(format!(
                    "CorticalMappedNeuronVoxelCoordVectors deserialization: per-cortical-area \
                     byte count {} is not divisible by per-voxel byte count {}",
                    number_bytes_to_read, per_voxel_bytes
                )));
            }

            let num_voxels = number_bytes_to_read / per_voxel_bytes;

            let collection = self.get_mut(&cortical_id).ok_or_else(|| {
                FeagiStructuresError::DeserializationError(format!(
                    "CorticalMappedNeuronVoxelCoordVectors deserialization: incoming cortical id \
                     {:?} is not pre-populated in target; callers must initialize the target \
                     with the correct NeuronVoxelDimensions before deserialization",
                    cortical_id
                ))
            })?;
            collection.clear_all_neurons();
            collection.reserve(N::from_usize(num_voxels));

            let mut voxel_offset = data_start_reading;
            for _voxel_index in 0..num_voxels {
                let x = C::read_le_bytes(&byte_reading[voxel_offset..voxel_offset + coord_bytes]);
                let y = C::read_le_bytes(
                    &byte_reading[voxel_offset + coord_bytes..voxel_offset + 2 * coord_bytes],
                );
                let z = C::read_le_bytes(
                    &byte_reading[voxel_offset + 2 * coord_bytes..voxel_offset + 3 * coord_bytes],
                );
                let p = V::read_le_bytes(
                    &byte_reading[voxel_offset + 3 * coord_bytes..voxel_offset + per_voxel_bytes],
                );
                collection.push_neuron_voxel_unchecked(
                    NeuronVoxelCoordinate::new(x, y, z),
                    NeuronVoxelPotential::from(p),
                );
                voxel_offset += per_voxel_bytes;
            }

            reading_header_byte_index += NUMBER_BYTES_PER_CORTICAL_ID_HEADER;
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
