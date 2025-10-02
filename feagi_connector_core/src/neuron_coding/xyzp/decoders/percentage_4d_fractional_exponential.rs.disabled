use crate::data::{Percentage4D};
use crate::FeagiDataError;
use crate::genomic::CorticalID;
use crate::genomic::descriptors::{CorticalChannelDimensions, CorticalChannelIndex};
use crate::neurons::xyzp::coders::coder_shared_functions::decode_unsigned_binary_fractional;
use crate::neurons::xyzp::coders::NeuronXYZPDecoder;
use crate::neurons::xyzp::CorticalMappedXYZPNeuronData;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

pub struct Percentage4DFractionalExponentialNeuronXYZPDecoder {
    channel_dimensions: CorticalChannelDimensions,
    cortical_read_target: CorticalID
}

impl NeuronXYZPDecoder for Percentage4DFractionalExponentialNeuronXYZPDecoder {
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
}

impl Percentage4DFractionalExponentialNeuronXYZPDecoder {

    pub fn new(cortical_read_target: CorticalID, z_resolution: u32) -> Result<Self, FeagiDataError> {
        const CHANNEL_X_LENGTH: u32 = 4;
        const CHANNEL_Y_LENGTH: u32 = 1;

        Ok(Percentage4DFractionalExponentialNeuronXYZPDecoder {
            channel_dimensions: CorticalChannelDimensions::new(CHANNEL_X_LENGTH, CHANNEL_Y_LENGTH, z_resolution)?,
            cortical_read_target
        })
    }
}