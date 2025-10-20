use std::fmt::Debug;
use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::neuron_voxels::xyzp::{CorticalMappedXYZPNeuronVoxels};
use crate::data_pipeline::PipelineStageRunner;
use crate::wrapped_io_data::{WrappedIOType, WrappedIOData};


pub trait NeuronVoxelXYZPEncoder: Debug + Sync + Send {

    fn get_encodable_data_type(&self) -> WrappedIOType;

    /// Writes data to NeuronXYZPVoxelArray(s) of the relevant cortical area(s), where each element in pipelines is the channel. Assumes write_target been cleared of neuron data
    fn write_neuron_data_multi_channel(&mut self, pipelines: &Vec<PipelineStageRunner>, time_of_previous_burst: Instant, write_target: &mut CorticalMappedXYZPNeuronVoxels) -> Result<(), FeagiDataError>;
}


pub trait NeuronVoxelXYZPDecoder: Debug + Sync + Send {
    fn get_decoded_data_type(&self) -> WrappedIOType;

    fn read_neuron_data_multi_channel(&mut self, neurons_to_read: &CorticalMappedXYZPNeuronVoxels, time_of_read: Instant, pipelines_with_data_to_update: &mut Vec<PipelineStageRunner>, channel_changed: &mut Vec<bool>) -> Result<(), FeagiDataError>;
}
