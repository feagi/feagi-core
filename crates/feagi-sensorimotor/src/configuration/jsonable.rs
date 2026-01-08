use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use feagi_structures::FeagiDataError;
use feagi_structures::genomic::cortical_area::descriptors::{CorticalChannelCount, CorticalChannelIndex, CorticalUnitIndex, NeuronDepth};
use feagi_structures::genomic::cortical_area::{CorticalID, IOCorticalAreaConfigurationFlag};
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::PercentageNeuronPositioning;
use crate::caching::FeedBackRegistration;
use crate::data_pipeline::PipelineStageProperties;
use crate::data_types::descriptors::{ImageFrameProperties, MiscDataDimensions, PercentageChannelDimensionality, SegmentedImageFrameProperties};
use crate::data_types::{GazeProperties, ImageFrame, MiscData, Percentage, Percentage2D, Percentage3D, Percentage4D, SegmentedImageFrame, SignedPercentage, SignedPercentage2D, SignedPercentage3D, SignedPercentage4D};
use crate::neuron_voxel_coding::xyzp::decoders::{GazePropertiesNeuronVoxelXYZPDecoder, MiscDataNeuronVoxelXYZPDecoder, PercentageNeuronVoxelXYZPDecoder};
use crate::neuron_voxel_coding::xyzp::{NeuronVoxelXYZPDecoder, NeuronVoxelXYZPEncoder};
use crate::neuron_voxel_coding::xyzp::encoders::{BooleanNeuronVoxelXYZPEncoder, CartesianPlaneNeuronVoxelXYZPEncoder, MiscDataNeuronVoxelXYZPEncoder, PercentageNeuronVoxelXYZPEncoder, SegmentedImageFrameNeuronVoxelXYZPEncoder};
use crate::wrapped_io_data::WrappedIOData;

/// Top level JSON representation of registered devices and feedbacks

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JSONInputOutputDefinition {
    input_units_and_encoder_properties: HashMap<SensoryCorticalUnit, Vec<(JSONUnitDefinition, JSONEncoderProperties)>>,
    output_units_and_decoder_properties: HashMap<MotorCorticalUnit, Vec<(JSONUnitDefinition, JSONDecoderProperties)>>,
    feedbacks: Vec<FeedBackRegistration>
}

impl JSONInputOutputDefinition {

    pub fn new() -> JSONInputOutputDefinition {
        JSONInputOutputDefinition {
            input_units_and_encoder_properties: HashMap::new(),
            output_units_and_decoder_properties: HashMap::new(),
            feedbacks: Vec::new()
        }
    }

    pub fn get_input_units_and_encoder_properties(&self) -> &HashMap<SensoryCorticalUnit, Vec<(JSONUnitDefinition, JSONEncoderProperties)>> {
        &self.input_units_and_encoder_properties
    }
    
    pub fn get_output_units_and_decoder_properties(&self) -> &HashMap<MotorCorticalUnit, Vec<(JSONUnitDefinition, JSONDecoderProperties)>> {
        &self.output_units_and_decoder_properties
    }
    
    pub fn verify_valid_structure(&self) -> Result<(), FeagiDataError> {
        for units_and_encoders in self.input_units_and_encoder_properties.values() {
            let mut unit_indexes: Vec<CorticalUnitIndex> = Vec::new();
            for unit in units_and_encoders {
                unit.0.verify_valid_structure()?;
                if unit_indexes.contains(&unit.0.cortical_unit_index) {
                    return Err(FeagiDataError::DeserializationError("Duplicate cortical unit indexes found!".into()))
                }
                unit_indexes.push(unit.0.cortical_unit_index);
            }
        }
        for units_and_decoders in self.output_units_and_decoder_properties.values() {
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
    
    pub fn insert_motor(&mut self, motor: MotorCorticalUnit, unit_definition: JSONUnitDefinition, decoder_properties: JSONDecoderProperties) {
        if !self.output_units_and_decoder_properties.contains_key(&motor) {
            self.output_units_and_decoder_properties.insert(motor.clone(), vec![(unit_definition, decoder_properties)]);
            return;
        }
        let vec = self.output_units_and_decoder_properties.get_mut(&motor).unwrap();
        vec.push((unit_definition, decoder_properties));
    }
    
    pub fn insert_sensor(&mut self, sensor: SensoryCorticalUnit, unit_definition: JSONUnitDefinition, encoder_properties: JSONEncoderProperties) {
        if !self.input_units_and_encoder_properties.contains_key(&sensor) {
            self.input_units_and_encoder_properties.insert(sensor.clone(), vec![(unit_definition, encoder_properties)]);
            return;
        }
        let vec = self.input_units_and_encoder_properties.get_mut(&sensor).unwrap();
        vec.push((unit_definition, encoder_properties));
    }

    pub fn get_feedbacks(&self) -> &Vec<FeedBackRegistration> {
        &self.feedbacks
    }
    pub fn set_feedbacks(&mut self, feedbacks: Vec<FeedBackRegistration>) {
        self.feedbacks = feedbacks;
    }

}


/// Defines a cortical unit. Does not include a COder Property directly since the type can vary
/// between input and output
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JSONUnitDefinition {
    pub(crate) friendly_name: Option<String>,
    pub(crate) cortical_unit_index: CorticalUnitIndex,
    pub(crate) io_configuration_flags: serde_json::Map<String, serde_json::Value>, // Due to the diversity contained here, this MUST be a generic dictionary
    pub(crate) device_grouping: Vec<JSONDeviceGrouping>,
}

impl JSONUnitDefinition {
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

    pub fn get_channel_count(&self) -> Result<CorticalChannelCount, FeagiDataError> {
        CorticalChannelCount::new(self.device_grouping.len() as u32)
    }
}

/// Defines a cortical unit's channel implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSONDeviceGrouping {
    pub(crate) friendly_name: Option<String>,
    pub(crate) device_properties: JSONDeviceProperties,
    pub(crate) channel_index_override: Option<CorticalChannelIndex>,
    pub(crate) pipeline_stages: Vec<PipelineStageProperties>
}

/// Middleman for Encoders and Decoders
//region Coder Properties

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JSONEncoderProperties {
    Boolean,
    CartesianPlane(ImageFrameProperties),
    MiscData(MiscDataDimensions),
    Percentage(NeuronDepth, PercentageNeuronPositioning, bool, PercentageChannelDimensionality),
    SegmentedImageFrame(SegmentedImageFrameProperties),
}

impl JSONEncoderProperties {

    pub fn to_box_encoder(&self, number_channels: CorticalChannelCount, cortical_ids: &[CorticalID]) -> Result<Box<dyn NeuronVoxelXYZPEncoder + Sync + Send>, FeagiDataError> {
        match self {
            JSONEncoderProperties::Boolean => {
                if cortical_ids.len() != 1 {
                    return Err(FeagiDataError::InternalError("Expected one cortical id!".to_string()));
                }
                BooleanNeuronVoxelXYZPEncoder::new_box(
                    *cortical_ids.get(0).unwrap(),
                    number_channels,
                )
            }
            JSONEncoderProperties::CartesianPlane(image_frame) => {
                if cortical_ids.len() != 1 {
                    return Err(FeagiDataError::InternalError("Expected one cortical id!".to_string()));
                }
                CartesianPlaneNeuronVoxelXYZPEncoder::new_box(
                    *cortical_ids.get(0).unwrap(),
                    image_frame,
                    number_channels
                )
            }
            JSONEncoderProperties::MiscData(misc_data_dimensions) => {
                if cortical_ids.len() != 1 {
                    return Err(FeagiDataError::InternalError("Expected one cortical id!".to_string()));
                }
                MiscDataNeuronVoxelXYZPEncoder::new_box(
                    *cortical_ids.get(0).unwrap(),
                    *misc_data_dimensions,
                    number_channels
                )
            }
            JSONEncoderProperties::Percentage(neuron_depth, percentage, is_signed, number_dimensions) => {
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
            JSONEncoderProperties::SegmentedImageFrame(segmented_properties) => {
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

    pub fn default_wrapped_value(&self) -> Result<WrappedIOData, FeagiDataError>  {
        match self {
            JSONEncoderProperties::Boolean => {
                Ok(WrappedIOData::Boolean(false))
            }
            JSONEncoderProperties::CartesianPlane(image_frame_properties) => {
                Ok(WrappedIOData::ImageFrame(
                    ImageFrame::new_from_image_frame_properties(image_frame_properties)?
                ))
            }
            JSONEncoderProperties::MiscData(misc_data_dimensions) => {
                Ok(WrappedIOData::MiscData(MiscData::new(misc_data_dimensions)?))
            }
            JSONEncoderProperties::Percentage(neuron_depth, percentage, is_signed, number_dimensions) => {
                match(number_dimensions) {
                    PercentageChannelDimensionality::D1 => {
                        if *is_signed {
                            Ok(WrappedIOData::SignedPercentage(SignedPercentage::new_from_m1_1_unchecked(0.0)))
                        } else {
                            Ok(WrappedIOData::Percentage(Percentage::new_zero()))
                        }
                    }
                    PercentageChannelDimensionality::D2 => {
                        if *is_signed {
                            Ok(WrappedIOData::SignedPercentage_2D(SignedPercentage2D::new_zero()))
                        } else {
                            Ok(WrappedIOData::Percentage_2D(Percentage2D::new_zero()))
                        }
                    }
                    PercentageChannelDimensionality::D3 => {
                        if *is_signed {
                            Ok(WrappedIOData::SignedPercentage_3D(SignedPercentage3D::new_zero()))
                        } else {
                            Ok(WrappedIOData::Percentage_3D(Percentage3D::new_zero()))
                        }
                    }
                    PercentageChannelDimensionality::D4 => {
                        if *is_signed {
                            Ok(WrappedIOData::SignedPercentage_4D(SignedPercentage4D::new_zero()))
                        } else {
                            Ok(WrappedIOData::Percentage_4D(Percentage4D::new_zero()))
                        }
                    }
                }
            }
            JSONEncoderProperties::SegmentedImageFrame(segmented_properties) => {
                Ok(WrappedIOData::SegmentedImageFrame(SegmentedImageFrame::from_segmented_image_frame_properties(segmented_properties)?))
            }
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JSONDecoderProperties {
    CartesianPlane(ImageFrameProperties),
    MiscData(MiscDataDimensions),
    Percentage(NeuronDepth, PercentageNeuronPositioning, bool, PercentageChannelDimensionality),
    GazeProperties(NeuronDepth, NeuronDepth, PercentageNeuronPositioning), // eccentricity z depth, modularity z depth
}

impl JSONDecoderProperties {
    pub fn to_box_decoder(&self, number_channels: CorticalChannelCount, cortical_ids: &[CorticalID]) -> Result<Box<dyn NeuronVoxelXYZPDecoder + Sync + Send>, FeagiDataError> {
        match self {
            JSONDecoderProperties::CartesianPlane(image_frame_properties) => {
                if cortical_ids.len() != 1 {
                    return Err(FeagiDataError::InternalError("Expected one cortical id!".to_string()));
                }
                crate::neuron_voxel_coding::xyzp::decoders::CartesianPlaneNeuronVoxelXYZPDecoder::new_box(
                    *cortical_ids.get(0).unwrap(),
                    image_frame_properties,
                    number_channels,
                )
            }
            JSONDecoderProperties::MiscData(misc_data_dimensions) => {
                if cortical_ids.len() != 1 {
                    return Err(FeagiDataError::InternalError("Expected one cortical id!".to_string()));
                }
                MiscDataNeuronVoxelXYZPDecoder::new_box(
                    *cortical_ids.get(0).unwrap(), // Eccentricity
                    *misc_data_dimensions,
                    number_channels
                )
            }
            JSONDecoderProperties::Percentage(neuron_depth, percentage_neuron_positioning, is_signed, dimension_count) => {
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
            JSONDecoderProperties::GazeProperties(eccentricity_neuron_depth, modularity_neuron_depth, percentage_neuron_positioning) => {
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

    pub fn default_wrapped_value(&self) -> Result<WrappedIOData, FeagiDataError>  {
        match self {
            JSONDecoderProperties::CartesianPlane(image_frame_properties) => {
                Ok(WrappedIOData::ImageFrame(
                    ImageFrame::new_from_image_frame_properties(image_frame_properties)?
                ))
            }
            JSONDecoderProperties::MiscData(misc_data_dimensions) => {
                Ok(WrappedIOData::MiscData(MiscData::new(misc_data_dimensions)?))
            }
            JSONDecoderProperties::Percentage(neuron_depth, percentage, is_signed, number_dimensions) => {
                match(number_dimensions) {
                    PercentageChannelDimensionality::D1 => {
                        if *is_signed {
                            Ok(WrappedIOData::SignedPercentage(SignedPercentage::new_from_m1_1_unchecked(0.0)))
                        } else {
                            Ok(WrappedIOData::Percentage(Percentage::new_zero()))
                        }
                    }
                    PercentageChannelDimensionality::D2 => {
                        if *is_signed {
                            Ok(WrappedIOData::SignedPercentage_2D(SignedPercentage2D::new_zero()))
                        } else {
                            Ok(WrappedIOData::Percentage_2D(Percentage2D::new_zero()))
                        }
                    }
                    PercentageChannelDimensionality::D3 => {
                        if *is_signed {
                            Ok(WrappedIOData::SignedPercentage_3D(SignedPercentage3D::new_zero()))
                        } else {
                            Ok(WrappedIOData::Percentage_3D(Percentage3D::new_zero()))
                        }
                    }
                    PercentageChannelDimensionality::D4 => {
                        if *is_signed {
                            Ok(WrappedIOData::SignedPercentage_4D(SignedPercentage4D::new_zero()))
                        } else {
                            Ok(WrappedIOData::Percentage_4D(Percentage4D::new_zero()))
                        }
                    }
                }
            }
            JSONDecoderProperties::GazeProperties(eccentricity, modularity, PercentageNeuronPositioning) => {
                Ok(WrappedIOData::GazeProperties(GazeProperties::create_default_centered()))
            }
        }
    }
}

//endregion

/// Custom Metadata to allow defining hardware properties per channel
//region Device Properties
/// A Dictionary structure that allows developers to tag custom information to
/// device groupings (channels).

pub type JSONDeviceProperties = HashMap<String, JSONDevicePropertyValue>;

/// User defined key for custom properties per channel, which can be useful in describing hardware

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[derive(Debug, Clone)]
pub enum JSONDevicePropertyValue {
    String(String),
    Integer(i32),
    Float(f32),
    Dictionary(JSONDeviceProperties),
}

//endregion