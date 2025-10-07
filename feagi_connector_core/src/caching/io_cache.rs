use std::time::Instant;
use feagi_data_structures::{FeagiDataError, FeagiSignalIndex};
use feagi_data_structures::genomic::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalGroupIndex};
use feagi_data_structures::genomic::{MotorCorticalType, SensorCorticalType};
use crate::caching::io_motor_cache::IOMotorCache;
use crate::caching::io_sensor_cache::IOSensorCache;
use crate::data_pipeline::{PipelineStageProperties, PipelineStagePropertyIndex};
use crate::data_pipeline::stage_properties::{IdentityStageProperties, ImageSegmentorStageProperties};
use crate::data_types::descriptors::{GazeProperties, ImageFrameProperties, MiscDataDimensions, SegmentedImageFrameProperties, SegmentedXYImageResolutions};
use crate::data_types::{Percentage4D, SegmentedImageFrame};
use crate::neuron_coding::xyzp::encoders::{MiscDataNeuronVoxelXYZPEncoder, SegmentedImageFrameNeuronVoxelXYZPEncoder};
use crate::neuron_coding::xyzp::{NeuronVoxelXYZPDecoder, NeuronVoxelXYZPEncoder};
use crate::neuron_coding::xyzp::decoders::Percentage4DLinearNeuronVoxelXYZPDecoder;
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

    pub fn sensor_get_bytes(&mut self) -> Result<&[u8], FeagiDataError> {
        _ = self.sensors.try_encode_updated_sensor_data_to_neurons(Instant::now())?;
        _ = self.sensors.try_encode_updated_neuron_data_to_feagi_byte_container(0)?;
        Ok(self.sensors.export_encoded_bytes())
    }


    //region Misc

    pub fn sensor_register_misc_absolute(&mut self, group: CorticalGroupIndex, number_channels: CorticalChannelCount,
                                         dimensions: MiscDataDimensions) -> Result<(), FeagiDataError> {


        let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send > = MiscDataNeuronVoxelXYZPEncoder::new_box(group, dimensions, number_channels, true)?;
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

    pub fn sensor_register_segmented_vision_absolute(&mut self, group: CorticalGroupIndex, number_channels: CorticalChannelCount, input_image_properties: ImageFrameProperties, segmented_image_properties: SegmentedImageFrameProperties, initial_gaze: GazeProperties) -> Result<(), FeagiDataError> {

        let cortical_ids = SegmentedImageFrame::create_ordered_cortical_ids_for_segmented_vision(group, false);
        let encoder: Box<dyn NeuronVoxelXYZPEncoder + Sync + Send > = SegmentedImageFrameNeuronVoxelXYZPEncoder::new_box(cortical_ids, segmented_image_properties, number_channels)?;

        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
            let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
            for i in 0..*number_channels {
                output.push( vec![ImageSegmentorStageProperties::new_box(input_image_properties, segmented_image_properties, initial_gaze)?]) // TODO properly implement clone so we dont need to do this
            };
            output
        };
        self.sensors.register(SENSOR_TYPE, group, encoder, default_pipeline)
    }

    pub fn sensor_write_segmented_vision_absolute(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, data: &WrappedIOData) -> Result<(), FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        self.sensors.try_update_value(SENSOR_TYPE, group, channel, data, Instant::now())?;
        Ok(())
    }

    pub fn sensor_update_stage_segmented_vision_absolute(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, pipeline_stage_property_index: PipelineStagePropertyIndex, stage: Box<dyn PipelineStageProperties + Sync + Send>) -> Result<(), FeagiDataError> {
        const SENSOR_TYPE: SensorCorticalType = SensorCorticalType::ImageCameraCenterAbsolute;
        self.sensors.try_updating_pipeline_stage(SENSOR_TYPE, group, channel, pipeline_stage_property_index, stage)?;
        Ok(())
    }





    //endregion

    //endregion


    //region Motors

    pub fn motor_send_bytes(&mut self, incoming_bytes: &[u8]) -> Result<(), FeagiDataError> {
        let mut byte_writer = |buf: &mut Vec<u8>| -> Result<(), FeagiDataError> {
            buf.clear();
            buf.extend_from_slice(incoming_bytes);
            Ok(())
        };
        self.motors.try_import_bytes(&mut byte_writer)?;
        self.motors.try_decode_bytes_to_neural_data()?;
        self.motors.try_decode_neural_data_into_cache(Instant::now())
    }

    //region Gaze

    pub fn motor_register_gaze_absolute(&mut self, group: CorticalGroupIndex, number_channels: CorticalChannelCount, z_depth: u32) -> Result<(), FeagiDataError> {
        if z_depth == 0 {
            return Err(FeagiDataError::BadParameters("Z depth cant be zero".into()))
        }

        const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::GazeAbsoluteLinear;
        let data_type = WrappedIOType::Percentage_4D;

        let cortical_id = MOTOR_TYPE.to_cortical_id(group);
        let decoder: Box<dyn NeuronVoxelXYZPDecoder + Sync + Send> = Percentage4DLinearNeuronVoxelXYZPDecoder::new_box(cortical_id, z_depth, number_channels)?;


        let default_pipeline: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = {
            let mut output: Vec<Vec<Box<(dyn PipelineStageProperties + Send + Sync + 'static)>>> = Vec::new();
            for i in 0..*number_channels {
                output.push( vec![IdentityStageProperties::new_box(data_type)?]) // TODO properly implement clone so we dont need to do this
            };
            output
        };

        self.motors.register(MOTOR_TYPE, group, decoder, default_pipeline)
    }

    pub fn motor_read_post_processed_gaze_absolute(&self, cortical_group_index: CorticalGroupIndex, cortical_channel_index: CorticalChannelIndex) -> Result<Percentage4D, FeagiDataError> {
        const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::GazeAbsoluteLinear;
        let data = self.motors.try_read_postprocessed_cached_value(MOTOR_TYPE, cortical_group_index, cortical_channel_index)?;
        let percentage: Percentage4D = data.try_into()?;
        Ok(percentage)
    }


    pub fn motor_add_callback_gaze_absolute<F>(&mut self, group: CorticalGroupIndex, channel: CorticalChannelIndex, callback: F) -> Result<FeagiSignalIndex, FeagiDataError>
    where
        F: Fn(&()) + Send + Sync + 'static,
    {
        const MOTOR_TYPE: MotorCorticalType = MotorCorticalType::GazeAbsoluteLinear;
        let index = self.motors.try_register_motor_callback(MOTOR_TYPE, group, channel, callback)?;
        Ok(index)
    }
    //endregion


    //endregion


    
    
    
}
