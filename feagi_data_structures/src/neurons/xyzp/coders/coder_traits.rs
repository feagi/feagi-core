

use std::collections::HashMap;
use crate::FeagiDataError;
use crate::genomic::descriptors::CorticalChannelIndex;
use crate::neurons::xyzp::CorticalMappedXYZPNeuronData;
use crate::wrapped_io_data::{WrappedIOType, WrappedIOData};


pub trait NeuronXYZPEncoder {

    fn get_encodable_data_type(&self) -> WrappedIOType;


    fn write_neuron_data_single_channel(&self, wrapped_value: &WrappedIOData, cortical_channel: CorticalChannelIndex, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError>;

    fn write_neuron_data_multi_channel(&self, channels_and_values: HashMap<CorticalChannelIndex, &WrappedIOData>, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        for (channel, values) in channels_and_values {
            self.write_neuron_data_single_channel(values, channel, write_target)?;
        };
        Ok(())
    }
}

pub trait NeuronXYZPDecoder {
    fn get_decoded_data_type(&self) -> WrappedIOType;

    fn read_neuron_data_single_channel(&self, read_target: &CorticalMappedXYZPNeuronData,  cortical_channel: CorticalChannelIndex, write_target: &mut WrappedIOData) -> Result<bool, FeagiDataError>;

    //TODO read_neuron_data_multi_channel
    /*
    fn read_neuron_data_multi_channel(&self, channel_value_target: &mut HashMap<CorticalChannelIndex, &WrappedIOData>, read_target: &CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> {
        for (channel, value) in channel_value_target {
            *value = self.read_neuron_data_single_channel(*channel, read_target)?;
        };
        Ok(())
    }

     */

}

