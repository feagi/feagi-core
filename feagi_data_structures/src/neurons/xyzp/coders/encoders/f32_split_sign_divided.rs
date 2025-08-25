use crate::FeagiDataError;
use crate::genomic::CorticalID;
use crate::genomic::descriptors::{CorticalChannelDimensions, CorticalChannelIndex};
use crate::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZP, NeuronXYZPArrays, NeuronXYZPEncoder};
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

pub struct F32SplitSignDividedNeuronXYZPEncoder {
    channel_dimensions: CorticalChannelDimensions,
    cortical_write_target: CorticalID
}

impl NeuronXYZPEncoder for F32SplitSignDividedNeuronXYZPEncoder {

    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::F32NormalizedM1To1
    }
    
    fn write_neuron_data_single_channel(&self, wrapped_value: &WrappedIOData, cortical_channel: CorticalChannelIndex, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        // We are not doing any sort of verification checks here, other than ensuring data types

        let value: f32 = wrapped_value.try_into()?;

        const NUMBER_NEURONS_IN_STRUCTURE: usize = 1;

        let generated_neuron_data: &mut NeuronXYZPArrays = write_target.ensure_clear_and_borrow_mut(&self.cortical_write_target, NUMBER_NEURONS_IN_STRUCTURE);
        let channel_offset: u32 = self.channel_dimensions.x * *cortical_channel + { if value.is_sign_positive() { 1 } else { 0 } };
        let p_val: f32 = value.into();
        let neuron: NeuronXYZP = NeuronXYZP::new(
            channel_offset,
            0,
            0,
            p_val.abs()
        );
        generated_neuron_data.push(&neuron);
        Ok(())
    }
}

impl F32SplitSignDividedNeuronXYZPEncoder {

    pub const CHANNEL_X_LENGTH: u32 = 2;
    pub const CHANNEL_Y_LENGTH: u32 = 1;
    pub fn new(cortical_write_target: CorticalID, z_resolution: u32) -> Result<Self, FeagiDataError> {
        Ok(F32SplitSignDividedNeuronXYZPEncoder {
            channel_dimensions: CorticalChannelDimensions::new(Self::CHANNEL_X_LENGTH, Self::CHANNEL_Y_LENGTH, z_resolution)?,
            cortical_write_target,
        })
    }
}