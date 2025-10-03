use std::collections::HashMap;
use std::time::Instant;
use feagi_data_serialization::FeagiByteContainer;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::genomic::MotorCorticalType;
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData};
use crate::caching::per_channel_stream_caches::MotorChannelStreamCaches;
use crate::data_pipeline::{PipelineStageProperties, PipelineStageRunner};
use crate::neuron_coding::xyzp::NeuronXYZPDecoder;
use crate::wrapped_io_data::WrappedIOData;

pub(crate) struct IOMotorCache {
    stream_caches: HashMap<(MotorCorticalType, CorticalGroupIndex), MotorChannelStreamCaches>,
    neuron_data: CorticalMappedXYZPNeuronData,
    byte_data: FeagiByteContainer,
}

impl IOMotorCache {

    pub fn new() -> Self {
        IOMotorCache {
            stream_caches: HashMap::new(),
            neuron_data: CorticalMappedXYZPNeuronData::new(),
            byte_data: FeagiByteContainer::new_empty()
        }
    }

    //region Interactions
    pub fn register(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex,
                    neuron_decoder: Box<dyn NeuronXYZPDecoder>,
                    pipeline_stages_across_channels: Vec<Vec<Box<dyn PipelineStageProperties + Sync + Send>>>)
                    -> Result<(), FeagiDataError> {

        // NOTE: The length of pipeline_stages_across_channels denotes the number of channels!

        if self.stream_caches.contains_key(&(motor_type, group_index)) {
            return Err(FeagiDataError::BadParameters(format!("Already registered motor {} of group index {}!", motor_type, group_index)))
        }

        self.stream_caches.insert(
            (motor_type, group_index),
            MotorChannelStreamCaches::new(neuron_decoder, pipeline_stages_across_channels)?);

        Ok(())
    }


    pub fn try_read_postprocessed_cached_value(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        Ok(motor_stream_caches.try_get_most_recent_postprocessed_motor_value(channel_index)?)
    }

    pub fn try_read_preprocessed_cached_value(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        Ok(motor_stream_caches.try_get_most_recent_preprocessed_motor_value(channel_index)?)
    }

    /*
    pub fn try_get_pipeline_stage_runner(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&PipelineStageRunner, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches(motor_type, group_index)?;
        let motor_stream_cache = motor_stream_caches.try_get_motor_channel_stream_cache(channel_index)?;
        Ok(motor_stream_cache.get_pipeline_runner())
    }

    pub fn try_get_pipeline_stage_runner_mut(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex, channel_index: CorticalChannelIndex) -> Result<&mut PipelineStageRunner, FeagiDataError> {
        let motor_stream_caches = self.try_get_motor_channel_stream_caches_mut(motor_type, group_index)?;
        let motor_stream_cache = motor_stream_caches.try_get_motor_channel_stream_cache_mut(channel_index)?;
        Ok(motor_stream_cache.get_pipeline_runner_mut())
    }
     */
    
    //endregion

    //region Decoding

    pub fn try_import_bytes<F>(&mut self, byte_writing_function: &mut F) -> Result<(), FeagiDataError>
    where F: FnMut(&mut Vec<u8>) -> Result<(), FeagiDataError> {
        self.byte_data.try_write_data_to_container_and_verify(byte_writing_function)?;
        Ok(())
    }

    // Returns true if data was retrieved
    pub fn try_decode_bytes_to_neural_data(&mut self) -> Result<bool, FeagiDataError> {
        self.byte_data.try_update_struct_from_first_found_struct_of_type(&mut self.neuron_data)
    }

    pub fn try_decode_neural_data_into_cache(&mut self, time_of_decode: Instant) -> Result<(), FeagiDataError> {
        for motor_channel_stream_cache in self.stream_caches.values_mut() {
            motor_channel_stream_cache.try_read_neuron_data_to_wrapped_io_data(&mut self.neuron_data, time_of_decode)?;
        };
        Ok(())
    }

    //endregion


    //region Internal
    fn try_get_motor_channel_stream_caches(&self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex) -> Result<&MotorChannelStreamCaches, FeagiDataError> {
        let check = self.stream_caches.get(&(motor_type, group_index));
        if check.is_none() {
            return Err(FeagiDataError::BadParameters(format!("Unable to find {} of cortical group index {} in registered motor's list!", motor_type, group_index)))
        }
        let check = check.unwrap();
        Ok(check)
    }

    fn try_get_motor_channel_stream_caches_mut(&mut self, motor_type: MotorCorticalType, group_index: CorticalGroupIndex) -> Result<&mut MotorChannelStreamCaches, FeagiDataError> {
        let check = self.stream_caches.get_mut(&(motor_type, group_index));
        if check.is_none() {
            return Err(FeagiDataError::BadParameters(format!("Unable to find {} of cortical group index {} in registered motor's list!", motor_type, group_index)))
        }
        let check = check.unwrap();
        Ok(check)
    }
    //endregion

}