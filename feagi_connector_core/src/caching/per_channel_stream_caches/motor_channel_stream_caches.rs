use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex};
use feagi_data_structures::neurons::xyzp::NeuronXYZPDecoder;
use feagi_data_structures::wrapped_io_data::WrappedIOData;
use crate::caching::per_channel_stream_caches::MotorChannelStreamCache;
use crate::data_pipeline::{stage_properties_to_stages, PipelineStageProperties};

#[derive(Debug)]
pub(crate) struct MotorChannelStreamCaches {
    neuron_decoder: Box<dyn NeuronXYZPDecoder>,
    stream_caches: Vec<MotorChannelStreamCache>,
}

impl MotorChannelStreamCaches {
    pub fn new(neuron_decoder: Box<dyn NeuronXYZPDecoder>, stage_properties_per_channels: Vec<Vec<Box<dyn PipelineStageProperties + Sync + Send>>>) -> Result<Self, FeagiDataError> {
        if stage_properties_per_channels.is_empty() { // Yes checks exist below, but this error has more context
            return Err(FeagiDataError::InternalError("MotorChannelStreamCaches Cannot be initialized with 0 channels!".into()))
        }

        let mut motor_channel_stream_caches: Vec<MotorChannelStreamCache> = Vec::with_capacity(stage_properties_per_channels.len());
        for stage_properties_per_channel in stage_properties_per_channels {
            motor_channel_stream_caches.push(MotorChannelStreamCache::new(stage_properties_per_channel)?);
        }

        Ok(Self {
            neuron_decoder,
            stream_caches: motor_channel_stream_caches
        })
    }

    //region Properties

    pub fn number_of_channels(&self) -> CorticalChannelCount {
        (self.stream_caches.len() as u32).into()
    }

    pub fn try_get_motor_channel_stream_cache(&self, cortical_channel_index: CorticalChannelIndex) -> Result<&MotorChannelStreamCache, FeagiDataError> {
        let result = self.stream_caches.get(*cortical_channel_index as usize);
        match result {
            Some(stream_cache) => Ok(stream_cache),
            None => Err(FeagiDataError::BadParameters(format!("Channel Index {} out is out of bounds for MotorChannelStreamCaches with {} channels!", cortical_channel_index, self.stream_caches.len())))

        }
    }

    pub fn try_get_motor_channel_stream_cache_mut(&mut self, cortical_channel_index: CorticalChannelIndex) -> Result<&mut MotorChannelStreamCache, FeagiDataError> {
        let count = self.stream_caches.len();
        let result = self.stream_caches.get_mut(*cortical_channel_index as usize);
        match result {
            Some(stream_cache) => Ok(stream_cache),
            None => Err(FeagiDataError::BadParameters(format!("Channel Index {} out is out of bounds for MotorChannelStreamCaches with {} channels!", cortical_channel_index, count)))

        }
    }

    //endregion


    fn get_data_iterator(&mut self) -> impl Iterator<Item = &mut WrappedIOData> {
        self.stream_caches.iter_mut().map(|stream_cache| stream_cache.get_neuron_decode_data_location_ref_mut())
    }

}