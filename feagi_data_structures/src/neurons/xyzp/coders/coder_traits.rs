

use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Instant;
use crate::FeagiDataError;
use crate::genomic::descriptors::{CorticalChannelCount};
use crate::neurons::xyzp::CorticalMappedXYZPNeuronData;
use crate::wrapped_io_data::{WrappedIOType, WrappedIOData};


pub trait NeuronXYZPEncoder: Debug {

    fn get_encodable_data_type(&self) -> WrappedIOType;
    

    fn write_neuron_data_multi_channel<'a>(&self, data_and_update_time_iterator: impl Iterator<Item = (&'a WrappedIOData, &'a Instant)>, time_of_burst: Instant,  write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError> ;
}

pub trait NeuronXYZPDecoder: Debug {
    fn get_decoded_data_type(&self) -> WrappedIOType;

    /// Reads neuron data (if available in received neuron data) and updates all channel relevant WrappedIOData with it
    fn read_neuron_data_multi_channel(&self, channel_value_target: &mut Vec<&mut WrappedIOData>, did_channel_change: &mut Vec<bool>, read_target: &CorticalMappedXYZPNeuronData, number_channels: CorticalChannelCount) -> Result<(), FeagiDataError>;
}
