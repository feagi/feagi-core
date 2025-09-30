use std::collections::HashMap;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::CorticalGroupIndex;
use feagi_data_structures::genomic::SensorCorticalType;
use feagi_data_structures::neurons::xyzp::NeuronXYZPEncoder;
use crate::caching::per_channel_stream_caches::SensoryChannelStreamCaches;
use crate::data_pipeline::PipelineStageProperties;

pub(crate) struct IOSensorCache {
    stream_caches: HashMap<(SensorCorticalType, CorticalGroupIndex), SensoryChannelStreamCaches>,
}

impl IOSensorCache {

    pub fn register_sensor(&mut self, sensor_type: SensorCorticalType, group_index: CorticalGroupIndex,
                           neuron_encoder: Box<dyn NeuronXYZPEncoder>,
                           pipeline_stages_across_channels: Vec<Vec<Box<dyn PipelineStageProperties>>>)
        -> Result<(), FeagiDataError> {

        if self.stream_caches.contains_key(&(sensor_type, group_index)) {
            return Err(FeagiDataError::BadParameters(format!("Already registered sensor {} of group index {}!", sensor_type, group_index))
        }



    }




}













