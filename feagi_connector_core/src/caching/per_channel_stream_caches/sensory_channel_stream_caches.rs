use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex};
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

    //region Properties

    pub fn number_of_channels(&self) -> CorticalChannelCount {
        self.stream_caches.len().into()
    }

    pub fn try_get_sensory_channel_stream_cache(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&SensoryChannelStreamCache, FeagiDataError> {
        let result = self.stream_caches.get(*cortical_channel_index as usize);
        match result {
            Some(stream_cache) => Ok(stream_cache),
            None => Err(FeagiDataError::BadParameters(format!("Channel Index {} out is out of bounds for SensoryChannelStreamCaches with {} channels!", cortical_channel_index, self.stream_caches.len())))

        }
    }

    pub fn try_get_sensory_channel_stream_cache_mut(&mut self, cortical_channel_index: CorticalChannelIndex) -> Result<&mut SensoryChannelStreamCache, FeagiDataError> {
        let result = self.stream_caches.get_mut(*cortical_channel_index as usize);
        match result {
            Some(stream_cache) => Ok(stream_cache),
            None => Err(FeagiDataError::BadParameters(format!("Channel Index {} out is out of bounds for SensoryChannelStreamCaches with {} channels!", cortical_channel_index, self.stream_caches.len())))

        }
    }

    //endregion



    pub fn update_neuron_data_with_recently_updated_cached_sensor_data(&self, neuron_data: &mut CorticalMappedXYZPNeuronData, time_of_burst: Instant) -> Result<(), FeagiDataError> {
        let iterator = self.get_data_and_dat_update_time_iterator();
        self.neuron_encoder.write_neuron_data_multi_channel(iterator, time_of_burst, neuron_data)?;
        Ok(())
    }


    fn get_data_and_update_time_iterator(&self) -> impl Iterator<Item = (&WrappedIOData, &Instant)> {
        self.stream_caches.iter().map(|stream_cache| stream_cache.get_most_recent_sensor_value_and_time())
    }



}

