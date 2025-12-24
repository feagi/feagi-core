use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use feagi_structures::FeagiDataError;
use feagi_structures::genomic::cortical_area::descriptors::{CorticalChannelIndex, CorticalUnitIndex};
use feagi_structures::genomic::cortical_area::IOCorticalAreaDataFlag;
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
use crate::data_pipeline::PipelineStageProperties;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputOutputDefinition {
    input_units: HashMap<SensoryCorticalUnit, Vec<UnitDefinition>>,
    output_units: HashMap<MotorCorticalUnit, Vec<UnitDefinition>>
}

impl InputOutputDefinition {
    pub fn get_input_units(&self) -> &HashMap<SensoryCorticalUnit, Vec<UnitDefinition>> {
        &self.input_units
    }

    pub fn get_output_units(&self) -> &HashMap<MotorCorticalUnit, Vec<UnitDefinition>> {
        &self.output_units
    }

    pub fn verify_valid_structure(&self) -> Result<(), FeagiDataError> {
        fn check_units(unit_definitions: &Vec<UnitDefinition>) -> Result<(), FeagiDataError> {
            let mut unit_indexes: Vec<CorticalUnitIndex> = Vec::new();
            for unit in unit_definitions {
                unit.verify_valid_structure()?;
                if unit_indexes.contains(&unit.cortical_unit_index) {
                    return Err(FeagiDataError::DeserializationError("Duplicate cortical unit indexes found!".into()))
                }
                unit_indexes.push(unit.cortical_unit_index);
            }
            Ok(())
        }

        for units in self.input_units.values() {
            check_units(units)?;
        }
        for units in self.output_units.values() {
            check_units(units)?;
        }

        Ok(())
    }

    pub fn insert_motor(&mut self, motor: MotorCorticalUnit, unit_definition: UnitDefinition) {
        if !self.output_units.contains_key(&motor) {
            self.output_units.insert(motor.clone(), vec![unit_definition]);
            return;
        }
        let vec = self.output_units.get_mut(&motor).unwrap();
        vec.push(unit_definition);
    }

    pub fn insert_sensor(&mut self, sensor: SensoryCorticalUnit, unit_definition: UnitDefinition) {
        if !self.input_units.contains_key(&sensor) {
            self.input_units.insert(sensor.clone(), vec![unit_definition]);
            return;
        }
        let vec = self.input_units.get_mut(&sensor).unwrap();
        vec.push(unit_definition);
    }

}



#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnitDefinition {
    pub(crate) friendly_name: String,
    pub(crate) cortical_unit_index: CorticalUnitIndex,
    pub(crate) cortical_area_data_flag: IOCorticalAreaDataFlag,
    pub(crate) device_grouping: Vec<DeviceGrouping>,
}

impl UnitDefinition {
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

            let stages = &device_grouping.pipeline_stages;
            // TODO check stage compatibility
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceGrouping {
    pub(crate) friendly_name: String,
    pub(crate) device_properties: DeviceProperties,
    pub(crate) channel_index_override: Option<CorticalChannelIndex>,
    pub(crate) pipeline_stages: Vec<PipelineStageProperties>
}

/// A Dictionary structure that allows developers to tag custom information to
/// device groupings (channels).
pub type DeviceProperties = HashMap<String, DevicePropertyValue>;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[derive(Debug, Clone)]
pub enum DevicePropertyValue {
    String(String),
    Integer(i32),
    Float(f32),
    Dictionary(DeviceProperties),
}