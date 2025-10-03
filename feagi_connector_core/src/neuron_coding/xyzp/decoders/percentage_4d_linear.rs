use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::CorticalID;
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelDimensions, CorticalChannelIndex};
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPArrays};
use crate::data_types::Percentage4D;
use crate::neuron_coding::xyzp::NeuronXYZPDecoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

#[derive(Debug)]
pub struct Percentage4DLinearNeuronXYZPDecoder {
    channel_dimensions: CorticalChannelDimensions,
    cortical_read_target: CorticalID,
    scratch_space: Vec<[Vec<i32>; 4]>, // channel, 4d, z_index
}

// NOTE: we need ot be cautious of multiple neurons coming in affecting the result (we should average them)


impl NeuronXYZPDecoder for Percentage4DLinearNeuronXYZPDecoder {
    fn get_decoded_data_type(&self) -> WrappedIOType {
        WrappedIOType::Percentage
    }

    fn read_neuron_data_single_channel(&self, read_target: &CorticalMappedXYZPNeuronData, cortical_channel: CorticalChannelIndex, write_target: &mut WrappedIOData) -> Result<bool, FeagiDataError> {

        const CHANNEL_X_LENGTH: u32 = 4;
        let target: &mut Percentage4D = write_target.try_into()?;

        let reading_neuron_data = read_target.get_neurons_of(&self.cortical_read_target);
        if reading_neuron_data.is_none() {
            return Ok(false); // No neuron data found, returning false to state that no update was made
        }
        let reading_neuron_data = reading_neuron_data.unwrap();
        const Y_OFFSET: u32 = 0;

        target.a = decode_unsigned_binary_fractional(*cortical_channel, Y_OFFSET, reading_neuron_data);
        target.b = decode_unsigned_binary_fractional(*cortical_channel * CHANNEL_X_LENGTH + 1, Y_OFFSET, reading_neuron_data);
        target.c = decode_unsigned_binary_fractional(*cortical_channel * CHANNEL_X_LENGTH + 2, Y_OFFSET, reading_neuron_data);
        target.d = decode_unsigned_binary_fractional(*cortical_channel * CHANNEL_X_LENGTH + 3, Y_OFFSET, reading_neuron_data);
        Ok(true)
    }

    fn read_neuron_data_multi_channel(&mut self, read_target: &CorticalMappedXYZPNeuronData, time_of_read: Instant, write_target: &mut Vec<WrappedIOData>, channel_changed: &mut Vec<bool>) -> Result<(), FeagiDataError> {

        const CHANNEL_WIDTH: u32 = 8;
        const ONLY_ALLOWED_Y: u32 = 0;

        let neuron_array = read_target.get_neurons_of(&self.cortical_read_target);
        write_target.fill(WrappedIOData::Percentage_4D(Percentage4D::new_zero()));

        if neuron_array.is_none() {
            channel_changed.fill(false);
            return Ok(());
        }

        let mut neuron_array = neuron_array.unwrap();
        if neuron_array.is_empty() {
            channel_changed.fill(false);
            return Ok(());
        }

        for neuron in neuron_array.iter() {
            if neuron.cortical_coordinate.y != ONLY_ALLOWED_Y || neuron.potential == 0.0 {
                continue;
            }


        };



        // TODO it may be possible to chunk this to work in parallel, but it may not be worth it

    }
}

impl Percentage4DLinearNeuronXYZPDecoder {

    pub fn new(cortical_read_target: CorticalID, z_resolution: u32, number_channels: CorticalChannelCount) -> Result<Self, FeagiDataError> {
        const CHANNEL_X_LENGTH: u32 = 4;
        const CHANNEL_Y_LENGTH: u32 = 1;


        Ok(Percentage4DLinearNeuronXYZPDecoder {
            channel_dimensions: CorticalChannelDimensions::new(CHANNEL_X_LENGTH, CHANNEL_Y_LENGTH, z_resolution)?,
            cortical_read_target,
            scratch_space: vec![[; 4]; *number_channels as usize]
        })
    }
}