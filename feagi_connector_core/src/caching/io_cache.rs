use std::time::Instant;
use feagi_data_structures::FeagiDataError;
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::genomic::SensorCorticalType;
use crate::caching::io_motor_cache::IOMotorCache;
use crate::caching::io_sensor_cache::IOSensorCache;
use crate::data_pipeline::PipelineStageProperties;
use crate::data_pipeline::stage_properties::IdentityStageProperties;
use crate::data_types::descriptors::{MiscDataDimensions, SegmentedXYImageResolutions};
use crate::neuron_coding::xyzp::encoders::{MiscDataNeuronXYZPEncoder, SegmentedImageFrameNeuronXYZPEncoder};
use crate::neuron_coding::xyzp::NeuronXYZPEncoder;
use crate::wrapped_io_data::{WrappedIOData, WrappedIOType};

pub struct IOCache {
    sensors: IOSensorCache,
    motors: IOMotorCache,
}

// prefixes:
// cache_ -> cache encoding / decoding / alteration related function
// sensor_ -> sensor device specific function
// motor_ -> motor device specific function

impl IOCache {

    pub fn new() -> Self {
        IOCache {
            sensors: IOSensorCache::new(),
            motors: IOMotorCache::new()
        }
    }


    //region Sensors

    //region Misc

    pub fn sensor_register_misc_absolute(&mut self, group: CorticalGroupIndex, number_channels: CorticalChannelCount,
                                         dimensions: MiscDataDimensions) -> Result<(), FeagiDataError> {


        let encoder: Box<dyn NeuronXYZPEncoder + 'static > = MiscDataNeuronXYZPEncoder::new_box(group, dimensions, number_channels, true)?;
        let data_type = WrappedIOType::MiscData(Some(dimensions.clone()));

        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::MiscellaneousAbsolute;
        let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
            let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
            for i in 0..*number_channels {
                output.push( vec![IdentityStageProperties::new_box(data_type)?]) // TODO properly implement clone so we dont need to do this
            };
            output
        };
        self.sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
    }

    pub fn sensor_write_misc_absolute(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, data: &WrappedIOData) -> Result<(), FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::MiscellaneousAbsolute;
        self.sensors.try_update_value(SENSOR_TYPE, group, channel, data, Instant::now())
    }
    //endregion


    //region Segmented Vision

    pub fn sensor_register_segmented_vision_absolute(&mut self, group: CorticalGroupIndex, number_channels: CorticalChannelCount, segmented_xyimage_resolutions: SegmentedXYImageResolutions) -> Result<(), FeagiDataError> {

        let encoder: Box<dyn NeuronXYZPEncoder + 'static > = SegmentedImageFrameNeuronXYZPEncoder::m


        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
    }



    //endregion

    //endregion










    //region Motors





    //endregion


    
    
    
}
