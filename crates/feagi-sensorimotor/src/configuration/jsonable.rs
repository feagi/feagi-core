use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use feagi_structures::FeagiDataError;
use feagi_structures::genomic::cortical_area::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex, NeuronDepth};
use feagi_structures::genomic::cortical_area::{CorticalID, IOCorticalAreaConfigurationFlag};
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
use crate::data_pipeline::PipelineStageProperties;
use crate::data_types::descriptors::{ImageFrameProperties, MiscDataDimensions, PercentageChannelDimensionality, SegmentedImageFrameProperties};
use crate::neuron_voxel_coding::xyzp::decoders::{GazePropertiesNeuronVoxelXYZPDecoder, MiscDataNeuronVoxelXYZPDecoder, PercentageNeuronVoxelXYZPDecoder};
use crate::neuron_voxel_coding::xyzp::{NeuronVoxelXYZPDecoder, NeuronVoxelXYZPEncoder};
use crate::neuron_voxel_coding::xyzp::encoders::{BooleanNeuronVoxelXYZPEncoder, CartesianPlaneNeuronVoxelXYZPEncoder, MiscDataNeuronVoxelXYZPEncoder, PercentageNeuronVoxelXYZPEncoder, SegmentedImageFrameNeuronVoxelXYZPEncoder};

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputOutputDefinition {
    input_units: HashMap<SensoryCorticalUnit, Vec<(UnitDefinition, EncoderProperties)>>,
    output_units: HashMap<MotorCorticalUnit, Vec<(UnitDefinition, DecoderProperties)>>
}

impl InputOutputDefinition {
    #[allow(dead_code)]
    pub fn get_input_units(&self) -> &HashMap<SensoryCorticalUnit, Vec<(UnitDefinition, EncoderProperties)>> {
        &self.input_units
    }

    #[allow(dead_code)]
    pub fn get_output_units(&self) -> &HashMap<MotorCorticalUnit, Vec<(UnitDefinition, DecoderProperties)>> {
        &self.output_units
    }

    #[allow(dead_code)]
    pub fn verify_valid_structure(&self) -> Result<(), FeagiDataError> {
        for units_and_encoders in self.input_units.values() {
            let mut unit_indexes: Vec<CorticalUnitIndex> = Vec::new();
            for unit in units_and_encoders {
                unit.0.verify_valid_structure()?;
                if unit_indexes.contains(&unit.0.cortical_unit_index) {
                    return Err(FeagiDataError::DeserializationError("Duplicate cortical unit indexes found!".into()))
                }
                unit_indexes.push(unit.0.cortical_unit_index);
            }
        }
        for units_and_decoders in self.output_units.values() {
            let mut unit_indexes: Vec<CorticalUnitIndex> = Vec::new();
            for unit in units_and_decoders {
                unit.0.verify_valid_structure()?;
                if unit_indexes.contains(&unit.0.cortical_unit_index) {
                    return Err(FeagiDataError::DeserializationError("Duplicate cortical unit indexes found!".into()))
                }
                unit_indexes.push(unit.0.cortical_unit_index);
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn insert_motor(&mut self, motor: MotorCorticalUnit, unit_definition: UnitDefinition, decoder_properties: DecoderProperties) {
        if !self.output_units.contains_key(&motor) {
            self.output_units.insert(motor.clone(), vec![(unit_definition, decoder_properties)]);
            return;
        }
        let vec = self.output_units.get_mut(&motor).unwrap();
        vec.push((unit_definition, decoder_properties));
    }

    #[allow(dead_code)]
    pub fn insert_sensor(&mut self, sensor: SensoryCorticalUnit, unit_definition: UnitDefinition, encoder_properties: EncoderProperties) {
        if !self.input_units.contains_key(&sensor) {
            self.input_units.insert(sensor.clone(), vec![(unit_definition, encoder_properties)]);
            return;
        }
        let vec = self.input_units.get_mut(&sensor).unwrap();
        vec.push((unit_definition, encoder_properties));
    }

}



#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnitDefinition {
    pub(crate) friendly_name: String,
    pub(crate) cortical_unit_index: CorticalUnitIndex,
    pub(crate) cortical_area_data_flag: IOCorticalAreaConfigurationFlag,
    pub(crate) device_grouping: Vec<DeviceGrouping>,
}

impl UnitDefinition {
    #[allow(dead_code)]
    pub fn verify_valid_structure(&self) -> Result<(), FeagiDataError> {
        if self.device_grouping.is_empty() {
            return Err(FeagiDataError::DeserializationError("Cannot have a cortical unit of 0 device grouping!".to_string()));
        }
        let number_channels = self.device_grouping.len() as u32;
        for device_grouping in &self.device_grouping {
            if let Some(channel_override) = device_grouping.channel_index_override {
                if *channel_override > number_channels {
                    return Err(FeagiDataError::DeserializationError("Device has invalid channel override!".to_string()));
                }
            }

            let _stages = &device_grouping.pipeline_stages;
            // TODO check stage compatibility
        }
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceGrouping {
    pub(crate) friendly_name: String,
    pub(crate) device_properties: DeviceProperties,
    pub(crate) channel_index_override: Option<CorticalChannelIndex>,
    pub(crate) pipeline_stages: Vec<PipelineStageProperties>
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncoderProperties {
    Boolean,
    CartesianPlane(ImageFrameProperties),
    MiscData(MiscDataDimensions),
    Percentage(NeuronDepth, PercentageNeuronPositioning, bool, PercentageChannelDimensionality),
    SegmentedImageFrame(SegmentedImageFrameProperties),
}

impl EncoderProperties {
    #[allow(dead_code)]
    pub fn to_box_encoder(&self, number_channels: CorticalChannelCount, cortical_ids: &[CorticalID]) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        match self {
            EncoderProperties::Boolean => {
                if cortical_ids.len() != 1 {
                    return Err(FeagiDataError::InternalError("Expected one cortical id!".to_string()));
                }
                BooleanNeuronVoxelXYZPEncoder::new_box(
                    *cortical_ids.get(0).unwrap(),
                    number_channels,
                )
            }
            EncoderProperties::CartesianPlane(image_frame) => {
                if cortical_ids.len() != 1 {
                    return Err(FeagiDataError::InternalError("Expected one cortical id!".to_string()));
                }
                CartesianPlaneNeuronVoxelXYZPEncoder::new_box(
                    *cortical_ids.get(0).unwrap(),
                    image_frame,
                    number_channels
                )
            }
            EncoderProperties::MiscData(misc_data_dimensions) => {
                if cortical_ids.len() != 1 {
                    return Err(FeagiDataError::InternalError("Expected one cortical id!".to_string()));
                }
                MiscDataNeuronVoxelXYZPEncoder::new_box(
                    *cortical_ids.get(0).unwrap(),
                    *misc_data_dimensions,
                    number_channels
                )
            }
            EncoderProperties::Percentage(neuron_depth, percentage, is_signed, number_dimensions) => {
                if cortical_ids.len() != 1 {
                    return Err(FeagiDataError::InternalError("Expected one cortical id!".to_string()));
                }
                PercentageNeuronVoxelXYZPEncoder::new_box(
                    *cortical_ids.get(0).unwrap(),
                    *neuron_depth,
                    number_channels,
                    *percentage,
                    *is_signed,
                    *number_dimensions
                )
            }
            EncoderProperties::SegmentedImageFrame(segmented_properties) => {
                if cortical_ids.len() != 9 {
                    return Err(FeagiDataError::InternalError("Expected nine cortical ids!".to_string()));
                }
                let cortical_ids: [CorticalID; 9] = (*cortical_ids).try_into().map_err(|_| FeagiDataError::InternalError("Unable to get cortical ids!".to_string()))?;
                SegmentedImageFrameNeuronVoxelXYZPEncoder::new_box(
                    cortical_ids,
                    *segmented_properties,
                    number_channels
                )
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecoderProperties {
    MiscData(MiscDataDimensions),
    Percentage(NeuronDepth, PercentageNeuronPositioning, bool, PercentageChannelDimensionality),
    GazeProperties(NeuronDepth, NeuronDepth, PercentageNeuronPositioning), // eccentricity z depth, modularity z depth
}

impl DecoderProperties {
    #[allow(dead_code)]
    pub fn to_box_decoder(&self, number_channels: CorticalChannelCount, cortical_ids: &[CorticalID]) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        match self {
            DecoderProperties::MiscData(misc_data_dimensions) => {
                if cortical_ids.len() != 1 {
                    return Err(FeagiDataError::InternalError("Expected one cortical id!".to_string()));
                }
                MiscDataNeuronVoxelXYZPDecoder::new_box(
                    *cortical_ids.get(0).unwrap(), // Eccentricity
                    *misc_data_dimensions,
                    number_channels
                )
            }
            DecoderProperties::Percentage(neuron_depth, percentage_neuron_positioning, is_signed, dimension_count) => {
                if cortical_ids.len() != 1 {
                    return Err(FeagiDataError::InternalError("Expected one cortical id!".to_string()));
                }
                PercentageNeuronVoxelXYZPDecoder::new_box(
                    *cortical_ids.get(0).unwrap(), // Eccentricity
                    *neuron_depth,
                    number_channels,
                    *percentage_neuron_positioning,
                    *is_signed,
                    *dimension_count
                )
            }
            DecoderProperties::GazeProperties(eccentricity_neuron_depth, modularity_neuron_depth, percentage_neuron_positioning) => {
                if cortical_ids.len() != 2 {
                    return Err(FeagiDataError::InternalError("Expected two cortical ids!".to_string()));
                }
                GazePropertiesNeuronVoxelXYZPDecoder::new_box(
                    *cortical_ids.get(0).unwrap(), // Eccentricity
                    *cortical_ids.get(1).unwrap(),  // Modularity
                    *eccentricity_neuron_depth,
                    *modularity_neuron_depth,
                    number_channels,
                    *percentage_neuron_positioning
                )
            }

        }
    }
}


/// A Dictionary structure that allows developers to tag custom information to
/// device groupings (channels).
#[allow(dead_code)]
pub type DeviceProperties = HashMap<String, DevicePropertyValue>;

/// User defined key for custom properties per channel, which can be useful in describing hardware
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[derive(Debug, Clone)]
pub enum DevicePropertyValue {
    String(String),
    Integer(i32),
    Float(f32),
    Dictionary(DeviceProperties),
}