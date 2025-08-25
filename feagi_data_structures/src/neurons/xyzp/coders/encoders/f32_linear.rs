use crate::FeagiDataError;
use crate::genomic::CorticalID;
use crate::genomic::descriptors::{CorticalChannelDimensions, CorticalChannelIndex};
use crate::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZP, NeuronXYZPArrays};
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};
use super::super::{NeuronXYZPEncoder};

pub struct F32LinearNeuronXYZPEncoder {
    channel_dimensions: CorticalChannelDimensions,
    cortical_write_target: CorticalID,
    z_res: f32,
}

impl NeuronXYZPEncoder for F32LinearNeuronXYZPEncoder {
    fn get_encodable_data_type(&self) -> WrappedIOType {
        WrappedIOType::F32Normalized0To1
    }

    fn write_neuron_data_single_channel(&self, wrapped_value: &WrappedIOData, cortical_channel: CorticalChannelIndex, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        // We are not doing any sort of verification checks here, other than ensuring data types
        
        let value: f32 = wrapped_value.try_into()?;
        
        let z_dist: f32 = value * self.z_res;
        let channel_offset: u32 = self.channel_dimensions.x * *cortical_channel;
        let z_index: u32 = z_dist.floor() as u32;
        let neuron: NeuronXYZP = NeuronXYZP::new(
            channel_offset,
            0,
            z_index,
            1.0
        );

        const NUMBER_NEURONS_IN_STRUCTURE: usize = 1;
        let generated_neuron_data: &mut NeuronXYZPArrays = write_target.ensure_clear_and_borrow_mut(&self.cortical_write_target, NUMBER_NEURONS_IN_STRUCTURE);
        generated_neuron_data.push(&neuron);
        Ok(())
    }
}

impl F32LinearNeuronXYZPEncoder {
    
    pub const CHANNEL_X_LENGTH: u32 = 1;
    pub const CHANNEL_Y_LENGTH: u32 = 1;
    
    pub fn new(cortical_write_target: CorticalID, z_resolution: u32) -> Result<Self, FeagiDataError> {
        
        Ok(F32LinearNeuronXYZPEncoder {
            channel_dimensions: CorticalChannelDimensions::new(Self::CHANNEL_X_LENGTH, Self::CHANNEL_Y_LENGTH, z_resolution)?,
            cortical_write_target,
            z_res: z_resolution as f32,
        })
    }
}