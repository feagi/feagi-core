use byteorder::{ByteOrder, LittleEndian};
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZP, NeuronXYZPArrays};
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::CorticalID;
use crate::byte_structure::{FeagiByteStructureCompatible, FeagiByteStructure, FeagiByteStructureType};

const BYTE_STRUCT_VERSION: u8 = 1;

const NUMBER_BYTES_PER_CORTICAL_ID_HEADER: usize = CorticalID::NUMBER_OF_BYTES + size_of::<u32>() + size_of::<u32>();
const NUMBER_BYTES_CORTICAL_COUNT_HEADER: usize = size_of::<u16>();

impl FeagiByteStructureCompatible for CorticalMappedXYZPNeuronData {
    fn get_type(&self) -> FeagiByteStructureType { FeagiByteStructureType::NeuronCategoricalXYZP }
    fn get_version(&self) -> u8 { BYTE_STRUCT_VERSION }
    fn overwrite_feagi_byte_structure_slice(&self, slice: &mut [u8]) -> Result<usize, FeagiDataError> {

        if self.mappings.len() == 0 {
            return Err(FeagiDataError::DeserializationError("Cannot generate a bytes structure export with an empty cortical mappings object!".into()).into())
        }

        let num_bytes_needed: usize = self.max_number_bytes_needed();
        if slice.len() < num_bytes_needed {
            return Err(FeagiDataError::SerializationError(format!("Not enough space given to store neuron XYZP data! Need {} bytes but given {}!", num_bytes_needed, slice.len())).into());
        }

        slice[0] = self.get_type() as u8;
        slice[1] = self.get_version();

        let number_cortical_areas: usize = self.mappings.len();
        LittleEndian::write_u16(&mut slice[2..4], number_cortical_areas as u16);

        let mut subheader_write_index: usize = FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + NUMBER_BYTES_CORTICAL_COUNT_HEADER;
        let mut neuron_data_write_index: u32 = subheader_write_index as u32 + (number_cortical_areas as u32 * NUMBER_BYTES_PER_CORTICAL_ID_HEADER as u32);

        for (cortical_id, neuron_data) in &self.mappings {
            // Write cortical subheader
            let write_target = &mut slice[subheader_write_index .. subheader_write_index + CorticalID::CORTICAL_ID_LENGTH];
            let write_target: &mut[u8; 6] = write_target.try_into().unwrap();
            write_target.copy_from_slice(cortical_id.as_bytes());
            let reading_start: u32 = neuron_data_write_index;
            let reading_length: u32 = neuron_data.get_size_in_number_of_bytes() as u32;
            LittleEndian::write_u32(&mut slice[subheader_write_index + 6 .. subheader_write_index + 10], reading_start);
            LittleEndian::write_u32(&mut slice[subheader_write_index + 10 .. subheader_write_index + 14], reading_length);

            // write neuron data
            write_neuron_array_to_bytes(neuron_data, &mut slice[reading_start as usize .. (reading_start + reading_length) as usize])?;

            // update indexes
            neuron_data_write_index += reading_length;
            subheader_write_index += NUMBER_BYTES_PER_CORTICAL_ID_HEADER;
        };
        let wasted_space = slice.len() - num_bytes_needed;
        Ok(wasted_space)
    }

    fn max_number_bytes_needed(&self) -> usize {
        let mut number_bytes_needed: usize = FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + NUMBER_BYTES_CORTICAL_COUNT_HEADER;
        for neuron_data in self.iter() {
            number_bytes_needed += NUMBER_BYTES_PER_CORTICAL_ID_HEADER + neuron_data.get_size_in_number_of_bytes();
        }
        number_bytes_needed
    }

    fn new_from_feagi_byte_structure(feagi_byte_structure: &FeagiByteStructure) -> Result<Self, FeagiDataError> {
        FeagiByteStructure::verify_matching_structure_type_and_version(&feagi_byte_structure,
                                                                       FeagiByteStructureType::NeuronCategoricalXYZP,
                                                                       BYTE_STRUCT_VERSION)?;

        let bytes = feagi_byte_structure.borrow_data_as_slice();
        let number_cortical_areas: u16 = LittleEndian::read_u16(&bytes[2..4]);

        let min_array_length_with_cortical_headers: usize = FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES +  NUMBER_BYTES_CORTICAL_COUNT_HEADER +
            (NUMBER_BYTES_PER_CORTICAL_ID_HEADER * number_cortical_areas as usize);

        if bytes.len() < min_array_length_with_cortical_headers {
            return Err(FeagiDataError::SerializationError(format!("Byte structure for NeuronCategoricalXYZPV1 needs a length of {} to fit just the cortical details header, but is a length of {}",
                                                                  min_array_length_with_cortical_headers, bytes.len())).into());
        }

        let number_cortical_areas: usize = number_cortical_areas as usize;
        let mut output: CorticalMappedXYZPNeuronData = CorticalMappedXYZPNeuronData::new_with_capacity(number_cortical_areas);

        let mut reading_header_index: usize = FeagiByteStructure::GLOBAL_BYTE_HEADER_BYTE_SIZE_IN_BYTES + NUMBER_BYTES_CORTICAL_COUNT_HEADER;

        for _cortical_index in 0..number_cortical_areas {
            let cortical_id = CorticalID::from_bytes(
                <&[u8; 6]>::try_from(&bytes[reading_header_index..reading_header_index + 6]).unwrap()
            )?;
            let data_start_reading: usize = LittleEndian::read_u32(&bytes[reading_header_index + 6..reading_header_index + 10]) as usize;
            let number_bytes_to_read: usize = LittleEndian::read_u32(&bytes[reading_header_index + 10..reading_header_index + 14]) as usize;

            if bytes.len() < data_start_reading + number_bytes_to_read {
                return Err(FeagiDataError::SerializationError("Byte structure for NeuronCategoricalXYZPV1 is too short to fit the data the header says it contains!".into()).into());
            }

            let neuron_bytes = &bytes[data_start_reading..data_start_reading + number_bytes_to_read];
            let bytes_length = neuron_bytes.len();
            if bytes_length % NeuronXYZP::NUMBER_BYTES_PER_NEURON != 0 {
                return Err(FeagiDataError::SerializationError("Byte structure for NeuronCategoricalXYZPV1 seems invalid! Size is nonsensical given neuron data size!".into()).into());
            }

            if bytes_length % NeuronXYZP::NUMBER_BYTES_PER_NEURON != 0 {
                return Err(FeagiDataError::SerializationError("As NeuronXYCPArrays contains 4 internal arrays of equal length, each of elements of 4 bytes each (uint32 and float), the input bytes array must be divisible by 16!".into()).into());
            }
            let x_end = bytes_length / 4; // q1
            let y_end = bytes_length / 2; // q2
            let z_end = x_end * 3; // q3

            // Create vectors using bytes order to avoid alignment issues
            let num_neurons = bytes_length / NeuronXYZP::NUMBER_BYTES_PER_NEURON;
            let mut x_coords: Vec<u32> = Vec::with_capacity(num_neurons);
            let mut y_coords: Vec<u32> = Vec::with_capacity(num_neurons);
            let mut z_coords: Vec<u32> = Vec::with_capacity(num_neurons);
            let mut potentials: Vec<f32> = Vec::with_capacity(num_neurons);

            for i in 0..num_neurons {
                let x_start = i * 4;
                let y_start = x_end + x_start;
                let z_start = y_end + x_start;
                let p_start = z_end + x_start;

                x_coords.push(LittleEndian::read_u32(&neuron_bytes[x_start..x_start + 4]));
                y_coords.push(LittleEndian::read_u32(&neuron_bytes[y_start..y_start + 4]));
                z_coords.push(LittleEndian::read_u32(&neuron_bytes[z_start..z_start + 4]));
                potentials.push(LittleEndian::read_f32(&neuron_bytes[p_start..p_start + 4]));
            }

            let neurons = NeuronXYZPArrays::new_from_vectors(
                x_coords,
                y_coords,
                z_coords,
                potentials,
            )?;

            output.insert(cortical_id, neurons);
            reading_header_index += NUMBER_BYTES_PER_CORTICAL_ID_HEADER;
        };

        Ok(output)
    }
}

fn write_neuron_array_to_bytes(neuron_array: &NeuronXYZPArrays, bytes_to_write_to: &mut [u8]) -> Result<(), FeagiDataError> {
    const U32_F32_LENGTH: usize = 4;
    let number_of_neurons_to_write: usize = neuron_array.len();
    let number_bytes_needed = neuron_array.get_size_in_number_of_bytes();
    if bytes_to_write_to.len() != number_bytes_needed {
        return Err(FeagiDataError::SerializationError(format!("Need exactly {} bytes to write xyzp neuron data, but given a space of {} bytes!", bytes_to_write_to.len(), number_bytes_needed).into()))
    }
    let mut x_offset: usize = 0;
    let mut y_offset = number_of_neurons_to_write * U32_F32_LENGTH; // quarter way through the total bytes
    let mut z_offset = number_of_neurons_to_write * U32_F32_LENGTH * 2; // halfway through the total bytes
    let mut p_offset = number_of_neurons_to_write * U32_F32_LENGTH * 3; // three quarters way through the total bytes

    let (x, y, z, p) = neuron_array.borrow_xyzp_vectors();

    for i in 0 .. number_of_neurons_to_write {
        LittleEndian::write_u32(&mut bytes_to_write_to[x_offset .. x_offset + U32_F32_LENGTH], x[i]);
        LittleEndian::write_u32(&mut bytes_to_write_to[y_offset .. y_offset + U32_F32_LENGTH], y[i]);
        LittleEndian::write_u32(&mut bytes_to_write_to[z_offset .. z_offset + U32_F32_LENGTH], z[i]);
        LittleEndian::write_f32(&mut bytes_to_write_to[p_offset .. p_offset + U32_F32_LENGTH], p[i]);

        x_offset += U32_F32_LENGTH;
        y_offset += U32_F32_LENGTH;
        z_offset += U32_F32_LENGTH;
        p_offset += U32_F32_LENGTH;
    };

    Ok(())
}