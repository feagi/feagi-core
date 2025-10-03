use crate::data::{Percentage, SignedPercentage};
use crate::FeagiDataError;
use crate::genomic::CorticalID;
use crate::genomic::descriptors::{CorticalChannelDimensions, CorticalChannelIndex};
use crate::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPArrays, NeuronXYZPEncoder};
use crate::neurons::xyzp::coders::coder_shared_functions::encode_unsigned_binary_fractional;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

pub struct SignedPercentageFractionalExponentialNeuronXYZPEncoder {
    channel_dimensions: CorticalChannelDimensions,
    cortical_write_target: CorticalID
}

impl NeuronXYZPEncoder for SignedPercentageFractionalExponentialNeuronXYZPEncoder {

    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::SignedPercentage
    }

    fn write_neuron_data_single_channel(&self, wrapped_value: &WrappedIOData, cortical_channel: CorticalChannelIndex, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        // We are not doing any sort of verification checks here, other than ensuring data types

        let signed_percentage: SignedPercentage = wrapped_value.try_into()?;

        // Due to how fractional scaling works, we can just linear interp this into a unsigned percentage value and the value will be as expected
        let percentage = Percentage::new_from_interp_m1_1_unchecked(signed_percentage.get_as_m1_1());

        const NUMBER_NEURONS_IN_STRUCTURE: usize = 1;

        let generated_neuron_data: &mut NeuronXYZPArrays = write_target.ensure_clear_and_borrow_mut(&self.cortical_write_target, NUMBER_NEURONS_IN_STRUCTURE);
        let channel_x_offset: u32 = self.channel_dimensions.width * *cortical_channel;
        const Y_OFFSET: u32 = 0;

        encode_unsigned_binary_fractional(channel_x_offset, Y_OFFSET, self.channel_dimensions.depth, percentage, generated_neuron_data);

        Ok(())
    }
}

impl SignedPercentageFractionalExponentialNeuronXYZPEncoder {




    pub fn new(cortical_write_target: CorticalID, z_resolution: u32) -> Result<Self, FeagiDataError> {
        const CHANNEL_X_LENGTH: u32 = 1;
        const CHANNEL_Y_LENGTH: u32 = 1;

        Ok(SignedPercentageFractionalExponentialNeuronXYZPEncoder {
            channel_dimensions: CorticalChannelDimensions::new(CHANNEL_X_LENGTH, CHANNEL_Y_LENGTH, z_resolution)?,
            cortical_write_target
        })
    }
}