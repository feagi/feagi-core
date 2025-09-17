use crate::data::MiscData;
use crate::FeagiDataError;
use crate::genomic::CorticalID;
use crate::genomic::descriptors::{CorticalChannelDimensions, CorticalChannelIndex}=     ;
use crate::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPArrays, NeuronXYZPEncoder};
use crate::neurons::xyzp::coders::coder_shared_functions::encode_unsigned_binary_fractional;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

pub struct MiscDataNeuronXYZPEncoder {
    cortical_channel_dimensions: CorticalChannelDimensions,
    cortical_write_target: CorticalID,
    number_elements: usize
}

impl NeuronXYZPEncoder for MiscDataNeuronXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::MiscData(Some(self.cortical_channel_dimensions.into()))
    }

    fn write_neuron_data_single_channel(&self, wrapped_value: &WrappedIOData, cortical_channel: CorticalChannelIndex, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        const P_VALUE_FIRING: f32 = 1.0;
        const Y_OFFSET: u32 = 0;

        let value: MiscData = wrapped_value.try_into()?;
        let values = value.get_internal_data();

        let generated_neuron_data: &mut NeuronXYZPArrays = write_target.ensure_clear_and_borrow_mut(&self.cortical_write_target, self.number_elements);
        let channel_offset: u32 = self.cortical_channel_dimensions.width * *cortical_channel;

        


        Ok(())


    }
}