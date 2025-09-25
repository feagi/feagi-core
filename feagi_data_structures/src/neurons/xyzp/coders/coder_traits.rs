

use std::collections::HashMap;
use crate::FeagiDataError;
use crate::genomic::descriptors::CorticalChannelIndex;
use crate::neurons::xyzp::CorticalMappedXYZPNeuronData;
use crate::wrapped_io_data::{WrappedIOType, WrappedIOData};


pub trait NeuronXYZPEncoder {

    fn get_encodable_data_type(&self) -> WrappedIOType;
    
    fn get_number_of_channels(&self) -> usize;

    fn write_neuron_data_multi_channel(&self, channels_and_values: HashMap<CorticalChannelIndex, &WrappedIOData>, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> ;
}

pub trait NeuronXYZPDecoder {
    fn get_decoded_data_type(&self) -> WrappedIOType;

    fn get_number_of_channels(&self) -> usize;

    /// Reads neuron data (if available in received neuron data) and updates all channel relevant WrappedIOData with it
    fn read_neuron_data_multi_channel(&self, channel_value_target: &mut Vec<&WrappedIOData>, read_target: &CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError>;
}

