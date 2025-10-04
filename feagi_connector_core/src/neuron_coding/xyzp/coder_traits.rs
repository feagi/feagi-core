use std::fmt::Debug;
use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount};
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPArrays};
use crate::data_pipeline::PipelineStageRunner;
use crate::wrapped_io_data::{WrappedIOType, WrappedIOData};


pub trait NeuronXYZPEncoder: Debug + Sync + Send {

    fn get_encodable_data_type(&self) -> WrappedIOType;

    fn write_neuron_data_multi_channel(&mut self, pipelines: &Vec<PipelineStageRunner>, time_of_previous_burst: Instant, write_target: &mut CorticalMappedXYZPNeuronData) -> Result<(), FeagiDataError>;
}


pub trait NeuronXYZPDecoder: Debug + Sync + Send {
    fn get_decoded_data_type(&self) -> WrappedIOType;

    fn read_neuron_data_multi_channel(&mut self, read_target: &CorticalMappedXYZPNeuronData, time_of_read: Instant, write_target: &mut Vec<WrappedIOData>, channel_changed: &mut Vec<bool>) -> Result<(), FeagiDataError>;
}
