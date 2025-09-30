use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::CorticalChannelIndex;
use feagi_data_structures::neurons::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPEncoder};
use feagi_data_structures::wrapped_io_data::WrappedIOData;
use crate::data_pipeline::{stage_properties_to_stages, PipelineStage, PipelineStageProperties};
use super::sensory_channel_stream_cache::SensoryChannelStreamCache;

pub(crate) struct SensoryChannelStreamCaches {
    neuron_encoder: Box<dyn NeuronXYZPEncoder>,
    stream_caches: Vec<SensoryChannelStreamCache>,
}

impl SensoryChannelStreamCaches {

    pub fn new(neuron_encoder: Box<dyn NeuronXYZPEncoder>, stage_properties_per_channels: Vec<Vec<Box<dyn PipelineStageProperties>>>) -> Result<Self, FeagiDataError> {
        if stage_properties_per_channels.is_empty() {
            return Err(FeagiDataError::InternalError("SensoryChannelStreamCaches Cannot be initialized with 0 channels!".into()))
        }

        let mut sensory_channel_stream_caches: Vec<SensoryChannelStreamCache> = Vec::with_capacity(stage_properties_per_channels.len());
        for stage_properties_per_channel in &stage_properties_per_channels {
            let stages = stage_properties_to_stages(&stage_properties_per_channel)?;
            sensory_channel_stream_caches.push(SensoryChannelStreamCache::new(stages)?);
        }

        Ok(Self {
            neuron_encoder,
            stream_caches: sensory_channel_stream_caches
        })
    }

    //region Data
    pub fn get_most_recently_cached_sensor_value(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&WrappedIOData, FeagiDataError> {
        self.verify_channel_index(cortical_channel_index)?;
        Ok(self.stream_caches[*cortical_channel_index as usize].get_most_recent_sensor_value())
    }

    pub fn update_cached_sensor_value(&mut self, cortical_channel_index: CorticalChannelIndex, updated_value: WrappedIOData) -> Result<(), FeagiDataError> {
        self.verify_channel_index(cortical_channel_index)?;
        Ok(self.stream_caches[*cortical_channel_index as usize].update_sensor_value(updated_value)?)
    }




    //endregion

    pub fn update_neuron_data_with_recently_updated_cached_sensor_data(&self, neuron_data: &mut CorticalMappedXYZPNeuronData, time_of_burst: Instant) -> Result<(), FeagiDataError> {
        let iterator = self.get_data_and_dat_update_time_iterator();
        self.neuron_encoder.write_neuron_data_multi_channel(iterator, time_of_burst, neuron_data)?;
        Ok(())
    }


    fn verify_channel_index(&self, channel_index: CorticalChannelIndex) -> Result<(), FeagiDataError> {
        if *channel_index >= self.stream_caches.len() as u32 {
            return Err(FeagiDataError::BadParameters(format!("Channel Index {} is out of range for this cortical area of channel count {}!", channel_index, self.stream_caches.len())))
        }
        Ok(())
    }

    fn get_data_and_dat_update_time_iterator(&self) -> impl Iterator<Item = (&WrappedIOData, &Instant)> {
        self.stream_caches.iter().map(|stream_cache| stream_cache.get_most_recent_sensor_value_and_time())
    }



}

