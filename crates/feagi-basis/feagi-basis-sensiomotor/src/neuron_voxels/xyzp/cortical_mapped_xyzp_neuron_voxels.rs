use super::{NeuronVoxelXYZP, NeuronVoxelXYZPArrays};
use feagi_genomic_context::cortical_area::CorticalID;
use feagi_serialization::{FeagiByteContainer, FeagiByteStructureType, FeagiSerializable};
use std::any::Any;
use std::collections::HashMap;

const BYTE_STRUCT_VERSION: u8 = 1;
const NUMBER_BYTES_PER_CORTICAL_ID_HEADER: usize =
    CorticalID::NUMBER_OF_BYTES + size_of::<u32>() + size_of::<u32>();
const NUMBER_BYTES_CORTICAL_COUNT_HEADER: usize = size_of::<u16>();

/// Neuron voxel data organized by cortical area.
#[derive(Debug, Clone, PartialEq)]
pub struct CorticalMappedXYZPNeuronVoxels {
    pub mappings: HashMap<CorticalID, NeuronVoxelXYZPArrays>,
}

impl CorticalMappedXYZPNeuronVoxels {
    pub fn new() -> Self {
        CorticalMappedXYZPNeuronVoxels {
            mappings: HashMap::new(),
        }
    }

    pub fn new_with_capacity(capacity: usize) -> Self {
        CorticalMappedXYZPNeuronVoxels {
            mappings: HashMap::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    pub fn get_neurons_of(&self, cortical_id: &CorticalID) -> Option<&NeuronVoxelXYZPArrays> {
        self.mappings.get(cortical_id)
    }

    pub fn get_neurons_of_mut(
        &mut self,
        cortical_id: &CorticalID,
    ) -> Option<&mut NeuronVoxelXYZPArrays> {
        self.mappings.get_mut(cortical_id)
    }

    pub fn insert(
        &mut self,
        cortical_id: CorticalID,
        neuron_data: NeuronVoxelXYZPArrays,
    ) -> Option<NeuronVoxelXYZPArrays> {
        self.mappings.insert(cortical_id, neuron_data)
    }

    pub fn clear(&mut self) {
        self.mappings.clear();
    }

    pub fn clear_neurons_only(&mut self) {
        for neuron_arrays in self.mappings.values_mut() {
            neuron_arrays.clear();
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &NeuronVoxelXYZPArrays> + '_ {
        self.mappings.values()
    }

    pub fn ensure_clear_and_borrow_mut(
        &mut self,
        cortical_id: &CorticalID,
    ) -> &mut NeuronVoxelXYZPArrays {
        if self.mappings.contains_key(cortical_id) {
            let neurons = self.mappings.get_mut(cortical_id).unwrap();
            neurons.clear();
            return neurons;
        }
        self.mappings
            .insert(*cortical_id, NeuronVoxelXYZPArrays::new());
        self.mappings.get_mut(cortical_id).unwrap()
    }
}

impl Default for CorticalMappedXYZPNeuronVoxels {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for CorticalMappedXYZPNeuronVoxels {
    type Item = (CorticalID, NeuronVoxelXYZPArrays);
    type IntoIter = std::collections::hash_map::IntoIter<CorticalID, NeuronVoxelXYZPArrays>;

    fn into_iter(self) -> Self::IntoIter {
        self.mappings.into_iter()
    }
}

impl<'a> IntoIterator for &'a CorticalMappedXYZPNeuronVoxels {
    type Item = (&'a CorticalID, &'a NeuronVoxelXYZPArrays);
    type IntoIter = std::collections::hash_map::Iter<'a, CorticalID, NeuronVoxelXYZPArrays>;

    fn into_iter(self) -> Self::IntoIter {
        self.mappings.iter()
    }
}

impl<'a> IntoIterator for &'a mut CorticalMappedXYZPNeuronVoxels {
    type Item = (&'a CorticalID, &'a mut NeuronVoxelXYZPArrays);
    type IntoIter = std::collections::hash_map::IterMut<'a, CorticalID, NeuronVoxelXYZPArrays>;

    fn into_iter(self) -> Self::IntoIter {
        self.mappings.iter_mut()
    }
}

impl std::fmt::Display for CorticalMappedXYZPNeuronVoxels {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "CorticalMappedXYZPNeuronVoxels({} cortical areas)",
            self.len()
        )
    }
}

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

    fn try_serialize_struct_to_byte_slice(&self, byte_destination: &mut [u8]) -> Result<(), ()> {
        byte_destination[0] = self.get_type() as u8;
        byte_destination[1] = self.get_version();

        let number_cortical_areas: usize = self.mappings.len();
        byte_destination[FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT
            ..FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT + 2]
            .copy_from_slice(&(number_cortical_areas as u16).to_le_bytes());

        let mut subheader_write_index: usize =
            FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT + NUMBER_BYTES_CORTICAL_COUNT_HEADER;
        let mut neuron_data_write_index: usize =
            subheader_write_index + (number_cortical_areas * NUMBER_BYTES_PER_CORTICAL_ID_HEADER);

        for (cortical_id, neuron_data) in &self.mappings {
            let cortical_area_lookup_header_slice = &mut byte_destination
                [subheader_write_index..subheader_write_index + CorticalID::NUMBER_OF_BYTES];
            let cortical_area_lookup_header_slice: &mut [u8; CorticalID::NUMBER_OF_BYTES] =
                cortical_area_lookup_header_slice.try_into().unwrap();
            cortical_id.write_id_to_bytes(cortical_area_lookup_header_slice);

            let reading_length: u32 = neuron_data.get_size_in_number_of_bytes() as u32;
            byte_destination[subheader_write_index + CorticalID::NUMBER_OF_BYTES
                ..subheader_write_index + CorticalID::NUMBER_OF_BYTES + 4]
                .copy_from_slice(&(neuron_data_write_index as u32).to_le_bytes());
            byte_destination[subheader_write_index + CorticalID::NUMBER_OF_BYTES + 4
                ..subheader_write_index + CorticalID::NUMBER_OF_BYTES + 8]
                .copy_from_slice(&reading_length.to_le_bytes());

            write_neuron_array_to_bytes(
                neuron_data,
                &mut byte_destination
                    [neuron_data_write_index..(neuron_data_write_index + reading_length as usize)],
            )?;

            subheader_write_index += NUMBER_BYTES_PER_CORTICAL_ID_HEADER;
            neuron_data_write_index += reading_length as usize;
        }

        Ok(())
    }

    fn try_deserialize_and_update_self_from_byte_slice(
        &mut self,
        byte_reading: &[u8],
    ) -> Result<(), ()> {
        self.verify_byte_slice_is_of_correct_version(byte_reading)?;
        self.clear_neurons_only();

        let number_cortical_areas: usize =
            u16::from_le_bytes(byte_reading[2..4].try_into().map_err(|_| ())?) as usize;
        let mut reading_header_byte_index: usize =
            FeagiByteContainer::STRUCT_HEADER_BYTE_COUNT + NUMBER_BYTES_CORTICAL_COUNT_HEADER;

        for _cortical_index in 0..number_cortical_areas {
            let cortical_id = CorticalID::try_from_bytes(
                <&[u8; CorticalID::NUMBER_OF_BYTES]>::try_from(
                    &byte_reading[reading_header_byte_index
                        ..reading_header_byte_index + CorticalID::NUMBER_OF_BYTES],
                )
                .map_err(|_| ())?,
            )
            .map_err(|_| ())?;

            let data_start_reading: usize = u32::from_le_bytes(
                byte_reading[reading_header_byte_index + CorticalID::NUMBER_OF_BYTES
                    ..reading_header_byte_index + CorticalID::NUMBER_OF_BYTES + 4]
                    .try_into()
                    .map_err(|_| ())?,
            ) as usize;
            let number_bytes_to_read: usize = u32::from_le_bytes(
                byte_reading[reading_header_byte_index + CorticalID::NUMBER_OF_BYTES + 4
                    ..reading_header_byte_index + CorticalID::NUMBER_OF_BYTES + 8]
                    .try_into()
                    .map_err(|_| ())?,
            ) as usize;

            if byte_reading.len() < data_start_reading + number_bytes_to_read {
                return Err(());
            }

            let neuron_bytes =
                &byte_reading[data_start_reading..data_start_reading + number_bytes_to_read];
            let bytes_length = neuron_bytes.len();

            if bytes_length % NeuronVoxelXYZP::NUMBER_BYTES_PER_NEURON != 0 {
                return Err(());
            }

            let x_end = bytes_length / 4;
            let y_end = bytes_length / 2;
            let z_end = x_end * 3;

            let num_neurons = bytes_length / NeuronVoxelXYZP::NUMBER_BYTES_PER_NEURON;
            let neuron_array = self.ensure_clear_and_borrow_mut(&cortical_id);
            neuron_array.ensure_capacity(num_neurons);

            for i in 0..num_neurons {
                let x_start = i * 4;
                let y_start = x_end + x_start;
                let z_start = y_end + x_start;
                let p_start = z_end + x_start;

                neuron_array.push_raw(
                    u32::from_le_bytes(neuron_bytes[x_start..x_start + 4].try_into().unwrap()),
                    u32::from_le_bytes(neuron_bytes[y_start..y_start + 4].try_into().unwrap()),
                    u32::from_le_bytes(neuron_bytes[z_start..z_start + 4].try_into().unwrap()),
                    f32::from_le_bytes(neuron_bytes[p_start..p_start + 4].try_into().unwrap()),
                );
            }
            reading_header_byte_index += NUMBER_BYTES_PER_CORTICAL_ID_HEADER;
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn write_neuron_array_to_bytes(
    neuron_array: &NeuronVoxelXYZPArrays,
    bytes_to_write_to: &mut [u8],
) -> Result<(), ()> {
    const U32_F32_LENGTH: usize = 4;
    let number_of_neurons_to_write: usize = neuron_array.len();
    let number_bytes_needed = neuron_array.get_size_in_number_of_bytes();
    if bytes_to_write_to.len() != number_bytes_needed {
        return Err(());
    }

    let x_offset: usize = 0;
    let y_offset = number_of_neurons_to_write * U32_F32_LENGTH;
    let z_offset = number_of_neurons_to_write * U32_F32_LENGTH * 2;
    let p_offset = number_of_neurons_to_write * U32_F32_LENGTH * 3;

    let (x, y, z, p) = neuron_array.borrow_xyzp_vectors();

    #[cfg(target_endian = "little")]
    {
        let x_len = x.len() * U32_F32_LENGTH;
        let y_len = y.len() * U32_F32_LENGTH;
        let z_len = z.len() * U32_F32_LENGTH;
        let p_len = p.len() * U32_F32_LENGTH;

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
        let mut x_off = x_offset;
        let mut y_off = y_offset;
        let mut z_off = z_offset;
        let mut p_off = p_offset;

        for i in 0..number_of_neurons_to_write {
            bytes_to_write_to[x_off..x_off + U32_F32_LENGTH].copy_from_slice(&x[i].to_le_bytes());
            bytes_to_write_to[y_off..y_off + U32_F32_LENGTH].copy_from_slice(&y[i].to_le_bytes());
            bytes_to_write_to[z_off..z_off + U32_F32_LENGTH].copy_from_slice(&z[i].to_le_bytes());
            bytes_to_write_to[p_off..p_off + U32_F32_LENGTH].copy_from_slice(&p[i].to_le_bytes());

            x_off += U32_F32_LENGTH;
            y_off += U32_F32_LENGTH;
            z_off += U32_F32_LENGTH;
            p_off += U32_F32_LENGTH;
        }
    }

    Ok(())
}
