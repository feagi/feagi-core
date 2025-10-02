use std::fmt::Debug;
use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount};
use feagi_data_structures::neurons::xyzp::CorticalMappedXYZPNeuronData;
use crate::wrapped_io_data::{WrappedIOType, WrappedIOData};


pub trait NeuronXYZPEncoder: Debug {

    fn get_encodable_data_type(&self) -> WrappedIOType;


    fn write_neuron_data_multi_channel(&self, stream_caches: &Vec<StreamCache>, time_of_burst: Instant, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError>;

}


pub trait NeuronXYZPDecoder: Debug {
    fn get_decoded_data_type(&self) -> WrappedIOType;

    /// Reads neuron data (if available in received neuron data) and updates all channel relevant WrappedIOData with it
    fn read_neuron_data_multi_channel(&self, channel_value_target: &mut Vec<&mut WrappedIOData>, did_channel_change: &mut Vec<bool>, read_target: &CorticalMappedXYZPNeuronData, number_channels: CorticalChannelCount) -> Result<(), FeagiDataError>;
}
