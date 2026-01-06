use crate::data_pipeline::per_channel_stream_caches::{
    MotorPipelineStageRunner, SensoryPipelineStageRunner,
};
use crate::wrapped_io_data::WrappedIOType;
use feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels;
use feagi_structures::FeagiDataError;
use std::fmt::Debug;
use std::time::Instant;
use crate::configuration::jsonable::{JSONDecoderProperties, JSONEncoderProperties};

pub trait NeuronVoxelXYZPEncoder: Debug + Sync + Send {
    #[allow(dead_code)]
    fn get_encodable_data_type(&self) -> WrappedIOType;

    #[allow(dead_code)]
    fn get_as_properties(&self) -> JSONEncoderProperties;

    /// Writes data to NeuronXYZPVoxelArray(s) of the relevant cortical area(s), where each element in pipelines is the channel. Assumes write_target been cleared of neuron data
    fn write_neuron_data_multi_channel_from_processed_cache(
        &mut self,
        pipelines: &[SensoryPipelineStageRunner],
        time_of_previous_burst: Instant,
        write_target: &mut CorticalMappedXYZPNeuronVoxels,
    ) -> Result<(), FeagiDataError>;
}

pub trait NeuronVoxelXYZPDecoder: Debug + Sync + Send {
    #[allow(dead_code)]
    fn get_decodable_data_type(&self) -> WrappedIOType;

    #[allow(dead_code)]
    fn get_as_properties(&self) -> JSONDecoderProperties;

    /// Writes data to the respective channel of PipelineStageRunner to the input cache, and marks if the channel has been changed or not, with data read from the neurons
    fn read_neuron_data_multi_channel_into_pipeline_input_cache(
        &mut self,
        neurons_to_read: &CorticalMappedXYZPNeuronVoxels,
        time_of_read: Instant,
        pipelines_with_data_to_update: &mut Vec<MotorPipelineStageRunner>,
        channel_changed: &mut Vec<bool>,
    ) -> Result<(), FeagiDataError>;

}
